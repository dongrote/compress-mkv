use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::filelistitem::FileListItem;

pub struct FileScanner {
    pub recursive: bool,
}

impl FileScanner {
    pub fn new(recursive: bool) -> Self {
        FileScanner { recursive, }
    }

    pub fn scan(&self, dirpath: PathBuf, tx: Sender<FileListItem>) {
        let mut dirpaths = vec![dirpath];
        while dirpaths.len() > 0 {
            let current_dir = dirpaths.pop().unwrap();
            match fs::read_dir(&current_dir) {
                Ok(entries) => {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if let Ok(ft) = entry.file_type() {
                            if ft.is_file() {
                                let p = entry.path();
                                if let Some(fli) = FileListItem::new(p) {
                                    let _ = tx.send(fli);
                                }
                            } else if ft.is_dir() {
                                dirpaths.push(entry.path());
                            }
                        }
                    }
                },
                Err(_) => (),
            };
        }
    }
}
