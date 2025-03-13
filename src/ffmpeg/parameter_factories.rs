use std::path::PathBuf;
pub mod av1;
pub mod hevc;

pub trait ParameterFactory {
    fn parameters(&self, input: &PathBuf) -> Vec<PathBuf>;
}
