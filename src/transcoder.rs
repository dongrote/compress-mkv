use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{ChildStderr, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc::Sender};

use crate::codecs::Codec;
use crate::containers::Container;
use crate::quality::Quality;
use crate::probe::{probe_file, AVProbeMetadata};
use crate::transcode_state::TranscodeStatus;

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

#[derive(Debug)]
pub struct TranscodeProgressMessage {
    pub frames: usize,
    pub total_frames: usize,
    pub progress: f64,
    pub current_size: usize,
    pub predicted_compressed_size: usize,
    pub message: String,
}

pub struct Transcoder {
    stop: Option<Arc<Mutex<bool>>>,
    pub codec: Codec,
    pub container: Container,
    pub quality: Quality,
    pub status: TranscodeStatus,
}

impl Transcoder {
    pub fn default(stop: Option<Arc<Mutex<bool>>>) -> Self {
        Transcoder {
            stop,
            status: TranscodeStatus::Idle,
            codec: Codec::AV1,
            container: Container::Matroska,
            quality: Quality::Great,
        }
    }

    pub fn codec(mut self, codec: Codec) -> Self {
        self.codec = codec;
        self
    }

    pub fn quality(mut self, quality: Quality) -> Self {
        self.quality = quality;
        self
    }

    pub fn container(mut self, container: Container) -> Self {
        self.container = container;
        self
    }

    pub fn transcode(
        &self,
        source: PathBuf,
        _source_probe: Option<AVProbeMetadata>,
        destination: PathBuf,
        progress_tx: Option<Sender<TranscodeProgressMessage>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut delete_destination = false;
        let probe = probe_file(&source)?;
        let mut child = Command::new("ffmpeg")
            .args(self.build_args(&source, &destination))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if self.consume_stdout(child.stdout.take().unwrap(), probe, progress_tx) {
            let _ = child.kill();
            delete_destination = true;
        }

        if let Ok(status) = child.wait() {
            match status.success() {
                true => (),
                false => {
                    delete_destination = true;
                    if let Some(_stderr) = read_stderr_to_end(&mut child.stderr.take()) {
                        // tx stderr
                    }
                    if let Some(_code) = status.code() {
                        // tx ffmpeg exit status
                    } else {
                        // tx unexpected error
                    }
                },
            };
        }

        if delete_destination {
            let _ = fs::remove_file(&destination);
        }

        Ok(())
    }

    fn build_args(&self, source: &PathBuf, destination: &PathBuf) -> Vec<PathBuf> {
        fn pbs(s: &str) -> PathBuf { PathBuf::from(s) }

        let mut args = vec![
            pbs("-hide_banner"),
            pbs("-nostats"),
            pbs("-loglevel"), pbs("warning"),
            pbs("-progress"), pbs("pipe:1"),
            pbs("-i"), source.clone()
        ];
        let mut quality_args: Vec<PathBuf> = Quality::parameters(self.codec.clone(), self.quality)
            .iter().map(|s| pbs(s)).collect();
        let mut container_args: Vec<PathBuf> = Container::parameters(self.container)
            .iter().map(|s| pbs(s)).collect();
        args.append(&mut quality_args);

        // use copy for audio streams
        args.push(pbs("-c:a")); args.push(pbs("copy"));

        // use copy for subtitle streams
        args.push(pbs("-c:s")); args.push(pbs("copy"));

        // map all streams to output
        // this is where we would create explicit -map #
        // cmdline arguments to choose particular streams in order to
        // filter out unwanted languages to decrease overall file size
        args.push(pbs("-map")); args.push(pbs("0"));

        // explicity set container format, regardless of destination extension
        args.append(&mut container_args);

        args.push(destination.clone());
        args
    }

    fn consume_stdout(&self, stdout: ChildStdout, probe: AVProbeMetadata, progress_tx: Option<Sender<TranscodeProgressMessage>>) -> bool {
        let mut progress = CompressionProgress::new();
        let total_frames = probe.total_frames;
        let stdout_reader = BufReader::new(stdout);
        for line in stdout_reader.lines() {
            if let Ok(l) = line {
                match handle_ffmpeg_stdout_line(l, &mut progress) {
                    FFmpegStdoutResult::Continue => continue,
                    FFmpegStdoutResult::Render => match &progress_tx {
                        None => (),
                        Some(tx) => {
                            let _ = tx.send(TranscodeProgressMessage {
                                total_frames,
                                frames: progress.frame,
                                progress: f64::min(1.0, (progress.frame as f64) / (total_frames as f64)),
                                current_size: progress.total_size,
                                predicted_compressed_size: predict_compressed_size(progress.total_size, total_frames, progress.frame),
                                message: format!("{}/{} = {}", progress.frame, total_frames, (progress.frame as f64) / (total_frames as f64)),
                            });
                        },
                    },
                }
            }

            let should_stop = match &self.stop {
                None => false,
                Some(s) => *s.lock().unwrap(),
            };

            if should_stop {
                return true;
            }
        }

        false
    }
}

fn handle_ffmpeg_stdout_line(line: String, progress: &mut CompressionProgress) -> FFmpegStdoutResult {
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

fn predict_compressed_size(
    compressed_size: usize,
    total_frames: usize,
    compressed_frame_count: usize) -> usize {
    match compressed_frame_count {
        0 => 0,
        _ => ((compressed_size as f64) * ((total_frames as f64) / (compressed_frame_count as f64))) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let _ = Transcoder::default(None);
    }

    #[test]
    fn test_configure_default() {
        let transcoder = Transcoder::default(None)
            .codec(Codec::HEVC)
            .quality(Quality::Insane)
            .container(Container::MP4);
        assert_eq!(transcoder.codec, Codec::HEVC);
        assert_eq!(transcoder.quality, Quality::Insane);
        assert_eq!(transcoder.container, Container::MP4);
    }
}
