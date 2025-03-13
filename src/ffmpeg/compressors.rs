use std::path::PathBuf;
use std::error::Error;

pub mod av1;

pub struct CompressorOptions {
    pub dry_run: bool,
    pub fast: bool,
    pub overwrite: bool,
    pub codec: String,
}

pub trait FFmpegCompressor {
    fn compress(&self, input: &PathBuf, output: &PathBuf, options: &CompressorOptions) -> Result<(), Box<dyn Error>>;
}

pub struct FFmpegCompressorFactory {
}

impl FFmpegCompressorFactory {
    pub fn create_compressor(codec: &str) -> Result<impl FFmpegCompressor, String> {
        match codec {
            "av1" => Ok(av1::Av1FFmpegCompressor { }),
            _ => Err(format!("unknown codec {:?}", codec)),
        }
    }
}
