#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{IoSliceMut, Read},
    ops::Range,
    str::FromStr,
    sync::Arc,
    time::Instant,
    u64, usize,
};

use notify::{RecommendedWatcher, Watcher};
use parking_lot::RwLock;

pub struct FileWatcher {
    filename: Arc<RwLock<Option<String>>>,
    watcher: RecommendedWatcher,
    contents: Arc<RwLock<Vec<u8>>>,
}

impl FileWatcher {
    fn trigger_update(&mut self) {}

    pub fn new() -> Self {
        Self::with_buf(Vec::new())
    }

    pub fn with_buf(buf: Vec<u8>) -> Self {
        let filename = Arc::new(RwLock::new(None));
        let contents = Arc::new(RwLock::new(buf));
        let watcher = create_watcher(filename.clone(), contents.clone());
        Self {
            filename,
            contents,
            watcher,
        }
    }

    pub fn try_update_active_file(&mut self, raw_filepath: String) -> Option<String> {
        let expanded_path_raw = match shellexpand::full(&raw_filepath) {
            Ok(path) => path.to_string(),
            Err(_) => String::new(),
        };

        let path = std::path::Path::new(expanded_path_raw.as_str());

        if !path.exists() || !path.is_file() {
            return None;
        }

        let dir = path.parent().unwrap();
        dbg!(&dir);

        *self.filename.write() = Some(expanded_path_raw.clone());
        let watch = self.watcher.watch(dir, notify::RecursiveMode::NonRecursive);
        dbg!(watch).ok();

        self.update_contents();

        dbg!("exists", &expanded_path_raw);
        Some(expanded_path_raw)
    }

    pub fn update_contents(&mut self) {
        let path_string = self.filename.read().to_owned().unwrap();
        let path = std::path::Path::new(&path_string);
        let read_contents = std::fs::read(path).unwrap();
        *self.contents.write() = read_contents;
    }

    pub fn buf(
        &mut self,
    ) -> parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, Vec<u8>> {
        self.contents.read()
    }
}

fn create_watcher(
    path: Arc<RwLock<Option<String>>>,
    contents: Arc<RwLock<Vec<u8>>>,
) -> RecommendedWatcher {
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
            let path_string = path.read().to_owned().unwrap();
            let path = std::path::PathBuf::from_str(&path_string).unwrap();
            dbg!(&path);
            let read_contents = std::fs::read(path).unwrap_or_default();
            let len_read = read_contents.len();
            dbg!(len_read);
            *contents.write() = read_contents;
            dbg!("wrote = ", len_read);
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

pub struct FileSource {
    page_size: usize,
    last_requested_range: Option<Range<usize>>,
    pages: HashMap<usize, Arc<[u8]>>,
    file: Option<std::fs::File>,
}

impl FileSource {
    pub fn new() -> Self {
        Self {
            page_size: page_size::get(),
            last_requested_range: None,
            pages: HashMap::new(),
            file: None,
        }
    }

    pub fn update_byte_range(&mut self, start_byte: usize, end_byte: usize) {
        let file_size = match self.file.as_mut() {
            None => return,
            Some(f) => f.metadata().unwrap().len() as usize,
        };
        let max_page = self.byte_to_page(file_size);

        let start_page = self.byte_to_page(start_byte);
        if max_page < start_page {
            return; // Want to read atleast one page
        }
        let end_page = self.byte_to_page(end_byte).min(max_page);

        let len_last_page = file_size - self.page_to_offset(end_page);
        let mut bufs: Vec<Vec<u8>> = (start_page..end_page)
            .map(|_| self.page_size)
            .chain(std::iter::once(len_last_page))
            .map(|len| vec![0u8; len])
            .collect();
        let mut slices: Vec<_> = bufs.iter_mut().map(|v| IoSliceMut::new(v)).collect();
        let f = self.file.as_mut().expect("already checked existence");
        match f.read_vectored(&mut slices) {
            Ok(bytes_read) => drop(dbg!(bytes_read)),
            Err(e) => drop(dbg!(e)),
        }
        for (offset, buf) in bufs.into_iter().enumerate() {
            self.pages.insert(start_page + offset, Arc::from(buf));
        }
    }

    pub fn page_to_offset(&self, page: usize) -> usize {
        page * self.page_size
    }

    pub fn byte_to_page(&self, byte: usize) -> usize {
        byte / self.page_size
    }
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
