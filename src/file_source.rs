// #![allow(unused_variables)]
// #![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    ops::Range,
    sync::Arc,
};

use parking_lot::{Mutex, RwLock};

use notify::{RecommendedWatcher, Watcher};

pub struct FileWatcher {
    file_handle: Arc<RwLock<Option<std::fs::File>>>,
    watcher: RecommendedWatcher,
    page_size: usize,
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
            content: contents,
            page_size: page_size::get(),
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

        let _ = self.watcher.watch(dir, notify::RecursiveMode::NonRecursive);

        // On file change old content is no longer relevant
        self.content.lock().clear();

        Some(expanded_path_raw)
    }

    pub fn file_len(&self) -> usize {
        let file_guard = self.file_handle.write();
        match file_guard.as_ref() {
            Some(f) => f.metadata().map(|meta| meta.len()).unwrap_or(0) as usize,
            None => return 0,
        }
    }

    fn index_to_page_number(&self, index: usize) -> usize {
        index / self.page_size
    }

    fn offset_in_page(&self, index: usize) -> usize {
        index % self.page_size
    }

    pub fn get_range_within_page(
        &mut self,
        range: Range<usize>,
        output_buf: &mut [u8],
    ) -> Option<usize> {
        let Range { start, end } = range;

        let page_number = self.index_to_page_number(start);
        // Ensure same page, otherwise we can't return a slice
        if page_number != self.index_to_page_number(end) {
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
        let page = lock.entry(page_number).or_insert_with(|| {
            let _ = file_handle
                .seek(SeekFrom::Start((page_number * self.page_size) as u64))
                .expect("Seek failed????");
            let mut buffer = vec![0; self.page_size];
            let bytes_read = file_handle.read(&mut buffer).expect("IO error");
            buffer.resize_with(bytes_read, || unreachable!("Only sizes down, never up"));

            buffer
        });

        let page_len = page.len();
        let start_offset = self.offset_in_page(start);
        let end_offset = self.offset_in_page(end).min(page_len);

        let output_len = page_len.min(end_offset - start_offset);
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

        let updated_paths = event.paths;

        if do_update {
            let mut content_guard = content.lock();
            content_guard.clear();
            content_guard.len();
        }

        assert!(updated_paths.len() > 0);
    })
    .unwrap();

    watcher
}
