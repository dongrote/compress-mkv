use std::path::PathBuf;
use std::fs;

pub enum DirEntryCategory {
    DoesNotExist,
    RegularFile,
    SymbolicLink,
    Directory,
    Unknown,
}

pub fn classify_file(path: &PathBuf) -> DirEntryCategory {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_symlink() {
                DirEntryCategory::SymbolicLink
            } else if metadata.is_file() {
                DirEntryCategory::RegularFile
            } else if metadata.is_dir() {
                DirEntryCategory::Directory
            } else {
                DirEntryCategory::Unknown
            }
        },
        Err(_) => DirEntryCategory::DoesNotExist,
    }
}
