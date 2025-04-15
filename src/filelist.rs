use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::filelistitem::{FileListItem, FileListItemStatus};

pub struct FileList {
    pub hashmap: HashMap<PathBuf, Arc<Mutex<FileListItem>>>,
    pub items: Vec<Arc<Mutex<FileListItem>>>,
}

impl FileList {
    pub fn new() -> Self {
        FileList {
            hashmap: HashMap::new(),
            items: vec![],
        }
    }

    pub fn insert(&mut self, file_list_item: FileListItem) -> Arc<Mutex<FileListItem>> {
        let key = file_list_item.path.clone();
        let fli = Arc::new(Mutex::new(file_list_item));
        self.hashmap.insert(key, Arc::clone(&fli));
        self.items.push(Arc::clone(&fli));
        Arc::clone(&fli)
    }

    pub fn snapshot(&self) -> Vec<FileListItem> {
        let mut snapshot = vec![];
        for i in &self.items {
            let item = i.lock().unwrap();
            snapshot.push((*item).clone());
        }

        snapshot
    }

    pub fn get_path(&self, index: usize) -> Option<PathBuf> {
        match self.items.get(index) {
            None => None,
            Some(item) => {
                let i = item.lock().unwrap();
                Some((*i).path.clone())
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<Arc<Mutex<FileListItem>>> {
        match self.items.get(index) {
            None => None,
            Some(item) => Some(Arc::clone(item)),
        }
    }

    pub fn set_status(&mut self, path: &PathBuf, status: FileListItemStatus) {
        if let Some(item) = self.hashmap.get(path) {
            let mut i = item.lock().unwrap();
            (*i).status = status;
        }
    }
}
