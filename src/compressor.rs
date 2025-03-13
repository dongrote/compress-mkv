use std::error::Error;
use std::path::PathBuf;
use std::fmt::{Display, Formatter, Result};

use crate::ffmpeg::compressors::{CompressorOptions, FFmpegCompressor, FFmpegCompressorFactory};

#[derive(Debug)]
pub struct CompressorError {
}

impl Error for CompressorError {
    fn description(&self) -> &str {
        "The compressor experienced and error."
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for CompressorError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "ruh roh")
    }
}

pub fn compress_file(input: &PathBuf, _output: &PathBuf, options: &CompressorOptions) -> std::result::Result<(), Box<dyn Error>> {
    if let Ok(compressor) = FFmpegCompressorFactory::create_compressor("av1") {
        let output = generate_output_filename(&input, &options.codec);
        compressor.compress(input, &output, options)?;
        Ok(())
    } else {
        Err(Box::new(CompressorError { }))
    }
}

fn generate_output_filename(path: &PathBuf, codec: &str) -> PathBuf {
    match path.file_stem() {
        Some(file_stem) => {
            let mut out = PathBuf::from(path);
            out.set_file_name(file_stem);
            out.set_extension(format!("{}.mkv", codec));
            out
        },
        None => path.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_output_filename() {
        assert_eq!(generate_output_filename(&PathBuf::from("/foo/bar/baz.mkv"), "av1"), PathBuf::from("/foo/bar/baz.av1.mkv"));
        assert_eq!(generate_output_filename(&PathBuf::from("bar/baz.mkv"), "hevc"), PathBuf::from("bar/baz.hevc.mkv"));
    }
}
