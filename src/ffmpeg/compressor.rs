use std::{io::{BufRead, BufReader}, path::PathBuf, process::{Command, Stdio}};
use std::fs;
use kdam::{term, tqdm, BarExt};
use human_repr::HumanCount;
use crate::error::CompressorError;
use crate::ffmpeg::probe::probe_file;
use super::parameter_factories::ParameterFactory;


#[derive(Debug)]
struct CompressionProgress {
    pub frame: usize,
    pub fps: f64,
    pub total_size: usize,
}

impl CompressionProgress {
    pub fn new() -> Self {
        CompressionProgress {
            frame: 0,
            fps: 0.0,
            total_size: 0,
        }
    }
}


#[derive(Debug)]
pub struct CompressorOptions {
    pub dry_run: bool,
    pub fast: bool,
    pub overwrite: bool,
    pub codec: String,
}

pub struct FFmpegCompressor {
}

impl FFmpegCompressor {
    pub fn new() -> Self { FFmpegCompressor { } }

    pub fn compress(&self, input: &PathBuf, output: &PathBuf, parameters: &Box<dyn ParameterFactory>) -> Result<(), CompressorError> {
        let mut args = vec![
            PathBuf::from("-hide_banner"), PathBuf::from("-nostats"),
            PathBuf::from("-loglevel"), PathBuf::from("warning"),
            PathBuf::from("-progress"), PathBuf::from("pipe:1"),
            PathBuf::from("-i"), PathBuf::from(input),
        ];

        for param in parameters.parameters(input) {
            args.push(param);
        }

        args.push(PathBuf::from("-c:a"));
        args.push(PathBuf::from("copy"));
        args.push(PathBuf::from("-c:s"));
        args.push(PathBuf::from("copy"));
        args.push(PathBuf::from("-map"));
        args.push(PathBuf::from("0"));
        args.push(output.clone());

        let total_frames = get_total_frames(input);
        let input_size = get_file_size(input);

        println!("ffmpeg {}", args.iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join(" "));

        if let Ok(mut child) = Command::new("ffmpeg")
            .args(args)
            .stdout(Stdio::piped())
            .spawn() {

            term::init(false);

            let mut pbar = tqdm!(
                total = total_frames,
                desc = format!("transcoding {}", input_size.human_count_bytes()),
                position = 0,
                force_refresh = true
            );
            let mut progress = CompressionProgress::new();
            let stdout = child.stdout.take().unwrap();
            let stdout_reader = BufReader::new(stdout);
            for line in stdout_reader.lines() {
                if let Ok(l) = line {
                    let parts: Vec<&str> = l.split('=').collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    match parts[0] {
                        "fps" => progress.fps = parts[1].parse().unwrap_or(progress.fps),
                        "frame" => progress.frame = parts[1].parse().unwrap_or(progress.frame),
                        "total_size" => progress.total_size = parts[1].parse().unwrap_or(progress.total_size),
                        "progress" => {
                            pbar.set_postfix(format!("{} ({})",
                                progress.total_size.human_count_bytes(),
                                predict_compressed_size(progress.total_size, total_frames, progress.frame).human_count_bytes()));
                            let _ = pbar.update_to(progress.frame);
                            ()
                        },
                        _ => continue,
                    }
                }
            }

            println!("");
            if let Ok(status) = child.wait() {
                match status.success() {
                    true => Ok(()),
                    false => {
                        if let Some(code) = status.code() {
                            Err(CompressorError::for_file(input, &format!("ffmpeg exited with {:}", code)))
                        } else {
                            Err(CompressorError::for_file(input, "ffmpeg did not exit successfully."))
                        }
                    },
                }
            } else {
                Err(CompressorError::for_file(input, "There was an error waiting for the ffmpeg process."))
            }
        } else {
            Err(CompressorError::for_file(input, "There was an error executing ffmpeg."))
        }
    }

}

fn get_total_frames(input: &PathBuf) -> usize {
    match probe_file(input) {
        Ok(probe) => probe.total_frames,
        Err(_) => 1,
    }
}

fn get_file_size(input: &PathBuf) -> usize {
    match fs::metadata(input) {
        Ok(fi) => fi.len().try_into().unwrap_or(1),
        Err(_) => 1,
    }
}

fn predict_compressed_size(
    compressed_size: usize,
    total_frames: usize,
    compressed_frame_count: usize) -> usize {
    match compressed_frame_count {
        0 => 0,
        _ => ((compressed_size as f64) * ((total_frames as f64) / (compressed_frame_count as f64))) as usize
    }
}
