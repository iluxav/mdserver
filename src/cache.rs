use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use dashmap::DashMap;

#[derive(Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime: SystemTime,
    pub title: Option<String>,
    pub headings: Vec<String>,
}

#[derive(Clone)]
pub struct DirIndex {
    pub fingerprint: u64,
    pub rendered: Arc<String>,
}

pub struct Cache {
    pub files: DashMap<PathBuf, FileMeta>,
    pub dirs: DashMap<PathBuf, DirIndex>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            files: DashMap::new(),
            dirs: DashMap::new(),
        }
    }
}
