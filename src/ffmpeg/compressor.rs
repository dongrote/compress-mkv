use std::cell::RefCell;
use std::{io::{BufRead, BufReader, Read}, path::PathBuf, process::{Child, ChildStderr, Command, Stdio}};
use std::fs;
use std::rc::Rc;
use std::sync::mpsc;
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

enum FFmpegStdoutResult {
    Continue,
    Render,
}

#[derive(Clone, Debug)]
pub struct CompressorOptions {
    pub dry_run: bool,
    pub fast: bool,
    pub extreme: bool,
    pub overwrite: bool,
    pub codec: String,
    pub container: String,
}

pub struct FFmpegCompressor {
    events: Rc<RefCell<mpsc::Receiver<bool>>>,
    options: CompressorOptions,
}

impl FFmpegCompressor {
    pub fn new(options: CompressorOptions, events: Rc<RefCell<mpsc::Receiver<bool>>>) -> Self {
        FFmpegCompressor {
            events,
            options,
        }
    }

    pub fn compress(&self, input: &PathBuf, output: &PathBuf, parameters: &Box<dyn ParameterFactory>) -> Result<(), CompressorError> {
        match probe_file(input) {
            Ok(probe) => {
                if probe.video_codec == self.options.codec {
                    println!("{:?} is already encoded with {}; skipping", input, self.options.codec);
                    return Ok(())
                }
                let total_frames = probe.total_frames;
                let input_size = get_file_size(input);
                let mut args = vec![
                    PathBuf::from("-i"), PathBuf::from(input),
                ];
                for param in parameters.parameters(input, &probe) {
                    args.push(param);
                }
                args.push(PathBuf::from("-c:a"));
                args.push(PathBuf::from("copy"));
                args.push(PathBuf::from("-c:s"));
                args.push(PathBuf::from("copy"));
                args.push(PathBuf::from("-map"));
                args.push(PathBuf::from("0"));
                args.push(output.clone());
                println!("ffmpeg {}", args.iter().map(|s| format!("{:?}", s)).collect::<Vec<String>>().join(" "));

                // insert our pipe processing magic after showing the user the ffmpeg
                // command in case they want to copypasta it for use on their own
                args.insert(0, PathBuf::from("pipe:1"));
                args.insert(0, PathBuf::from("-progress"));
                args.insert(0, PathBuf::from("warning"));
                args.insert(0, PathBuf::from("-loglevel"));
                args.insert(0, PathBuf::from("-nostats"));
                args.insert(0, PathBuf::from("-hide_banner"));
                if !self.options.dry_run {
                    if let Ok(mut child) = Command::new("ffmpeg")
                        .args(args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
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
                                match self.handle_ffmpeg_stdout_line(l, &mut progress) {
                                    FFmpegStdoutResult::Continue => continue,
                                    FFmpegStdoutResult::Render => {
                                        pbar.set_postfix(format!("{} ({})",
                                            progress.total_size.human_count_bytes(),
                                            predict_compressed_size(progress.total_size, total_frames, progress.frame).human_count_bytes()));
                                        let _ = pbar.update_to(progress.frame);
                                    },
                                }
                            }

                            self.check_for_stop(&mut child);
                        }

                        println!("Waiting for ffmpeg to exit.");
                        if let Ok(status) = child.wait() {
                            match status.success() {
                                true => Ok(()),
                                false => {
                                    if let Some(stderr) = read_stderr_to_end(&mut child.stderr.take()) {
                                        print!("{}", stderr);
                                    }
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
                } else {
                    println!("dry-run mode; skipping transcode operation");
                    Ok(())
                }
            },
            Err(err) => {
                println!("Probe failed for {:?}; {:?}.\nSkipping {:?}.", input, err, input);
                Ok(())
            }
        }
    }

    fn handle_ffmpeg_stdout_line(&self, line: String, progress: &mut CompressionProgress) -> FFmpegStdoutResult {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            match parts[0] {
                "fps" => {
                    progress.fps = parts[1].parse().unwrap_or(progress.fps);
                    FFmpegStdoutResult::Continue
                },
                "frame" => {
                    progress.frame = parts[1].parse().unwrap_or(progress.frame);
                    FFmpegStdoutResult::Continue
                },
                "total_size" => {
                    progress.total_size = parts[1].parse().unwrap_or(progress.total_size);
                    FFmpegStdoutResult::Continue
                },
                "progress" => FFmpegStdoutResult::Render,
                _ => FFmpegStdoutResult::Continue,
            }
        } else {
            FFmpegStdoutResult::Continue
        }
    }

    fn check_for_stop(&self, child: &mut Child) {
        if let Ok(rx) = self.events.try_borrow_mut() {
            if let Ok(stop) = rx.try_recv() {
                if stop {
                    println!("Caught stop signal; killing ffmpeg!");
                    if let Err(err) = child.kill() {
                        println!("error killing ffmpeg process ({}) {err:?}", child.id());
                    } else {
                        println!("killed ffmpeg process ({})", child.id());
                    }
                }
            }
        }
    }
}

fn read_stderr_to_end(stderr: &mut Option<ChildStderr>) -> Option<String> {
    let mut buf = Vec::new();
    match stderr {
        Some(stream) => match BufReader::new(stream).read_to_end(&mut buf) {
            Ok(_) => match String::from_utf8(buf) {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            Err(_) => None,
        },
        None => None,
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
