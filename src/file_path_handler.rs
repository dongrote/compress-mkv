use std::fs::read_dir;
use std::path::PathBuf;
use std::rc::Rc;
use crate::compressor::Compressor;
use crate::fstools::{classify_file, DirEntryCategory};
use crate::error::FilePathHandlerError;

pub struct FilePathHandler {
    path: PathBuf,
    compressor: Rc<Box<Compressor>>,
}

impl FilePathHandler {
    pub fn for_pathbuf(path: PathBuf, compressor: &Rc<Box<Compressor>>) -> Self {
        FilePathHandler {
            path,
            compressor: Rc::clone(compressor),
        }
    }

    pub fn handle(&self) -> Result<(), FilePathHandlerError> {
        match classify_file(&PathBuf::from(&self.path)) {
            DirEntryCategory::Unknown => {
                println!("Unable to classify {:?}.", self.path);
                Err(FilePathHandlerError::for_file_path(&self.path, &format!("Unable to classify {:?}", self.path)))
            },
            DirEntryCategory::DoesNotExist => {
                println!("{:?} does not exist.", self.path);
                Err(FilePathHandlerError::for_file_path(&self.path, &format!("Path does not exist {:?}", self.path)))
            },
            DirEntryCategory::SymbolicLink => {
                println!("{:?} is a symlink.", self.path);
                Ok(()) // don't error, just do nothing
            },
            DirEntryCategory::Directory => {
                println!("{:?} is a directory.", self.path);
                match read_dir(&self.path) {
                    Ok(entries) => {
                        for entry in entries.filter_map(|e| e.ok()) {
                            if let Err(err) = FilePathHandler::for_pathbuf(entry.path(), &self.compressor).handle() {
                                return Err(err);
                            }
                        }
                        Ok(())
                    },
                    Err(_) => Err(FilePathHandlerError::for_file_path(&self.path, "not implemented")),
                }
            },
            DirEntryCategory::RegularFile => self.compressor
                .compress_file(&PathBuf::from(&self.path), &PathBuf::from(""))
                .or_else(|e| Err(FilePathHandlerError::for_file_path(&self.path, &format!("Error compressing regular file: {:?}.", e)))),
        }
    }
}
