use std::path::PathBuf;
use crate::ffmpeg::compressor::{CompressorOptions, FFmpegCompressor};
use crate::error::{CompressorError, InputParseError};
use crate::ffmpeg::parameter_factories::av1::Av1ParameterFactory;
use crate::ffmpeg::parameter_factories::hevc::HevcParameterFactory;
use crate::ffmpeg::parameter_factories::ParameterFactory;

pub fn compress_file(input: &PathBuf, _output: &PathBuf, options: &CompressorOptions) -> std::result::Result<(), CompressorError> {
    if let Ok(parameters) = create_parameter_factory(input, options) {
        let compressor = FFmpegCompressor::new();
        let output = generate_output_filename(&input, &options.codec);
        compressor.compress(input, &output, &parameters)?;
        Ok(())
    } else {
        Err(CompressorError::for_file(input, &format!("Unable to create {} compressor.", options.codec)))
    }
}

fn create_parameter_factory(input: &PathBuf, options: &CompressorOptions) -> Result<Box<dyn ParameterFactory>, InputParseError> {
    dbg!(options);
    match options.codec.as_str() {
        "av1" => Ok(Box::new(Av1ParameterFactory::new(options))),
        "hevc" => Ok(Box::new(HevcParameterFactory::new(options))),
        _ => Err(InputParseError::for_file(input, &format!("Unsupported output codec: {}.", options.codec))),
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
