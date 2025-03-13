use std::path::PathBuf;
use crate::ffmpeg::compressor::CompressorOptions;
use crate::ffmpeg::probe::probe_file;
use super::ParameterFactory;

pub struct Av1ParameterFactory {
    crf: u16,
    preset: u16,
}

impl Av1ParameterFactory {
    pub fn new(options: &CompressorOptions) -> Self {
        Av1ParameterFactory {
            crf: if options.fast { 35 } else { 25 },
            preset: if options.fast { 12 } else { 2 },
        }
    }
}

impl ParameterFactory for Av1ParameterFactory {
    fn parameters(&self, input: &PathBuf) -> Vec<PathBuf> {
        let mut params = vec![
            PathBuf::from("-c:v"), PathBuf::from("libsvtav1"),
            PathBuf::from("-crf"), PathBuf::from(self.crf.to_string()),
            PathBuf::from("-preset"), PathBuf::from(self.preset.to_string()),
            PathBuf::from("-svtav1-params"), PathBuf::from("tune=0"),
        ];

        match probe_file(input) {
            Ok(probe) => {
                params.push(PathBuf::from("-g"));
                params.push(PathBuf::from(probe.frame_rate.to_string()));
            },
            Err(_) => (),
        };

        params
    }
}
