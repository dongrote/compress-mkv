use std::path::PathBuf;
use crate::ffmpeg::compressor::CompressorOptions;
use super::ParameterFactory;

pub struct HevcParameterFactory {
    crf: u16,
    preset: String,
}

impl HevcParameterFactory {
    pub fn new(options: &CompressorOptions) -> Self {
        HevcParameterFactory {
            crf: if options.fast { 35 } else { 25 },
            preset: if options.fast { String::from("faster") } else { String::from("slower") },
        }
    }
}

impl ParameterFactory for HevcParameterFactory {
    fn parameters(&self, _input: &PathBuf) -> Vec<PathBuf> {
        vec![
            PathBuf::from("-c:v"), PathBuf::from("libx265"),
            PathBuf::from("-crf"), PathBuf::from(self.crf.to_string()),
            PathBuf::from("-preset"), PathBuf::from(&self.preset),
            PathBuf::from("-tag:v"), PathBuf::from("hvc1"),
        ]
    }
}
