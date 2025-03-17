use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub struct InputParseError {
    path: PathBuf,
    msg: String,
}

impl InputParseError {
    pub fn for_file(path: &PathBuf, msg: &str) -> Self {
        InputParseError {
            path: PathBuf::from(path),
            msg: String::from(msg),
        }
    }
}

impl Error for InputParseError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        &self.msg
    }
}

impl Display for InputParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing {:?}: {}", &self.path, &self.msg)
    }
}

#[derive(Debug)]
pub struct CompressorError {
    path: PathBuf,
    msg: String,
}

impl CompressorError {
    pub fn for_file(path: &PathBuf, msg: &str) -> Self {
        CompressorError {
            path: PathBuf::from(path),
            msg: String::from(msg),
        }
    }
}

impl Error for CompressorError {
    fn description(&self) -> &str {
        "The compressor experienced an error."
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for CompressorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error compressing {:?}: {}", &self.path, &self.msg)
    }
}

#[derive(Debug)]
pub struct FilePathHandlerError {
    path: PathBuf,
    msg: String,
}

impl FilePathHandlerError {
    pub fn for_file_path(path: &PathBuf, msg: &str) -> Self {
        FilePathHandlerError {
            path: PathBuf::from(path),
            msg: String::from(msg),
        }
    }
}

impl Error for FilePathHandlerError {
    fn description(&self) -> &str {
        "There was an error handling a file path."
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for FilePathHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error processing {:?}: {}", &self.path, &self.msg)
    }
}
