use std::{io::{BufRead, BufReader}, path::PathBuf, process::{Command, Stdio}};
use std::fs;
use kdam::{term, tqdm, BarExt};
use crate::ffmpeg::compressors::FFmpegCompressor;
use crate::ffmpeg::probe::probe_file;

pub struct Av1FFmpegCompressor {
}

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

impl FFmpegCompressor for Av1FFmpegCompressor {
    fn compress(&self, input: &std::path::PathBuf, output: &std::path::PathBuf, options: &super::CompressorOptions) -> Result<(), Box<dyn std::error::Error>> {
        let crf = match options.fast {
            true => "35",
            false => "25",
        };

        let preset = match options.fast {
            true => "12",
            false => "2",
        };

        let total_frames = get_total_frames(input);
        let _input_size = get_file_size(input);

        let mut args = vec![
            PathBuf::from("-hide_banner"), PathBuf::from("-nostats"),
            PathBuf::from("-loglevel"), PathBuf::from("warning"),
            PathBuf::from("-progress"), PathBuf::from("pipe:1"),
            PathBuf::from("-i"), input.clone(),
            PathBuf::from("-c:v"), PathBuf::from("libsvtav1"),
            PathBuf::from("-crf"), PathBuf::from(crf),
            PathBuf::from("-preset"), PathBuf::from(preset),
            PathBuf::from("-pix_fmt"), PathBuf::from("yuv420p10le"),
            PathBuf::from("-svtav1-params"), PathBuf::from("tune=0"),
            PathBuf::from("-c:a"), PathBuf::from("copy"),
            PathBuf::from("-c:s"), PathBuf::from("copy"),
            PathBuf::from("-map"), PathBuf::from("0"),
        ];

        args.push(output.clone());

        println!("ffmpeg {:?}", args);

        let mut child = Command::new("ffmpeg")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;

        {
            term::init(false);

            let mut pbar = tqdm!(
                total = total_frames,
                desc = "transcoding ",
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
                            let _ = pbar.update_to(progress.frame);
                            ()
                        },
                        _ => continue,
                    }
                }
            }

            println!("");
        }

        let status = child.wait()?;
        match status.success() {
            true => Ok(()),
            false => {
                if let Some(code) = status.code() {
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("ffmpeg exited with {:}", code))))
                } else {
                    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("ffmpeg did not exit successfully"))))
                }
            },
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
