// #![allow(unused_variables)]
// #![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    ops::Range,
    sync::Arc,
    time::Instant,
};

use parking_lot::{Mutex, RwLock};

use notify::{RecommendedWatcher, Watcher};

pub struct FileWatcher {
    file_handle: Arc<RwLock<Option<std::fs::File>>>,
    watcher: RecommendedWatcher,
    content: Arc<Mutex<HashMap<usize, Vec<u8>>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        let file_handle = Arc::new(RwLock::new(None));
        let contents = Arc::new(Mutex::new(HashMap::new()));
        let watcher = create_watcher(contents.clone());
        Self {
            file_handle,
            watcher,
            content: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn try_update_active_file(&mut self, raw_filepath: String) -> Option<String> {
        let expanded_path_raw = match shellexpand::full(&raw_filepath) {
            Ok(path) => path.to_string(),
            Err(_) => String::new(),
        };

        let path = std::path::Path::new(expanded_path_raw.as_str());
        let file = std::fs::File::open(path).ok()?;
        *self.file_handle.write() = Some(file);

        let dir = path.parent().unwrap();
        dbg!(&dir);

        let watch = self.watcher.watch(dir, notify::RecursiveMode::NonRecursive);
        dbg!(watch).ok();

        // On file change old content is no longer relevant
        self.content.lock().clear();

        dbg!("exists", &expanded_path_raw);
        Some(expanded_path_raw)
    }

    pub fn file_len(&self) -> usize {
        let file_guard = self.file_handle.write();
        match file_guard.as_ref() {
            Some(f) => f.metadata().map(|meta| meta.len()).unwrap_or(0) as usize,
            None => return 0,
        }
    }

    const PAGE_SIZE: usize = 1024 * 4;
    fn index_to_page_number(index: usize) -> usize {
        // assume a page is 4K for now
        // TODO: determine page length at comptime
        index / Self::PAGE_SIZE
    }

    fn offset_in_page(index: usize) -> usize {
        index % Self::PAGE_SIZE
    }

    pub fn get_range_within_page(
        &mut self,
        range: Range<usize>,
        output_buf: &mut [u8],
    ) -> Option<usize> {
        let Range { start, end } = range;

        let page_number = Self::index_to_page_number(start);
        // Ensure same page, otherwise we can't return a slice
        if page_number != Self::index_to_page_number(end) {
            return None;
        }

        let file_len = self.file_len();
        if start >= file_len || start > end {
            return None;
        }

        let mut file_guard: parking_lot::lock_api::RwLockWriteGuard<
            parking_lot::RawRwLock,
            Option<std::fs::File>,
        > = self.file_handle.as_ref().write();
        let file_handle = file_guard.as_mut()?;

        let mut lock = self.content.lock_arc();
        let num_pages = lock.len();
        let page = lock.entry(page_number).or_insert_with(|| {
            let _ = file_handle
                .seek(SeekFrom::Start((page_number * Self::PAGE_SIZE) as u64))
                .expect("Seek failed????");
            let mut buffer = vec![0; Self::PAGE_SIZE];
            let _ = file_handle.read(&mut buffer).expect("IO error");
            dbg!(num_pages + 1);

            buffer
        });

        let start_offset = Self::offset_in_page(start);
        let end_offset = Self::offset_in_page(end);

        let output_len = page.len().min(end_offset - start_offset);
        if output_buf.len() < output_len {
            return None;
        }

        for i in 0..output_len {
            output_buf[i] = page[start_offset + i];
        }

        Some(output_len)
    }
}

fn create_watcher(content: Arc<Mutex<HashMap<usize, Vec<u8>>>>) -> RecommendedWatcher {
    let watcher = notify::recommended_watcher(move |event_res| {
        dbg!("event", &event_res);
        let event: notify::Event = match event_res {
            Ok(e) => e,
            Err(_) => todo!(),
        };
        let do_update = match event.kind {
            notify::EventKind::Any => true,
            notify::EventKind::Access(_) => false,
            notify::EventKind::Create(_) => true,
            notify::EventKind::Modify(_) => true,
            notify::EventKind::Remove(_) => true,
            notify::EventKind::Other => true,
        };
        dbg!(do_update);

        let updated_paths = event.paths;

        if do_update {
            content.lock_arc().clear();
        }

        assert!(updated_paths.len() > 0);
    })
    .unwrap();

    watcher
}

// Does not belong in this file
pub struct FrameCounter {
    last_frame: Instant,
    tick_number: u64,
    framerate: u32,
}

impl FrameCounter {
    const FRAMERATE_UPDATE_INTERVAL: u64 = 10;

    pub fn new() -> Self {
        FrameCounter {
            last_frame: Instant::now(),
            tick_number: 0,
            framerate: 0,
        }
    }

    pub fn register_tick(&mut self) {
        self.tick_number += 1;
        if self.tick_number % Self::FRAMERATE_UPDATE_INTERVAL == 0 {
            self.update_framerate();
            self.update_delta_time();
        }
    }

    fn update_framerate(&mut self) -> u32 {
        let delta_time = self.last_frame.elapsed().as_millis();
        self.framerate = match delta_time == 0 {
            true => 99,
            false => ((Self::FRAMERATE_UPDATE_INTERVAL as u128 * 1_000) / delta_time) as u32,
        };
        self.framerate
    }

    fn update_delta_time(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn fps(&self) -> u32 {
        self.framerate
    }
}
