pub mod av1;
pub mod hevc;

use std::path::PathBuf;
use crate::ffmpeg::probe::AVProbeMetadata;

pub trait ParameterFactory {
    fn parameters(&self, input: &PathBuf, probe: &AVProbeMetadata) -> Vec<PathBuf>;
}
