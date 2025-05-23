use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::codecs::Codec;
use crate::error::InputParseError;

#[derive(Clone, Debug)]
pub struct Resolution {
    pub width: u64,
    pub height: u64,
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[derive(Clone, Debug)]
pub struct AVProbeMetadata {
    pub video_codec: Codec,
    pub video_codec_tag: Option<String>,
    pub file_size: Option<usize>,
    pub resolution: Resolution,
    pub total_frames: usize,
    pub frame_rate: u64,
    pub interlaced: bool,
}

impl AVProbeMetadata {
    pub fn empty() -> Self {
        AVProbeMetadata {
            video_codec: Codec::Unknown(String::new()),
            video_codec_tag: None,
            file_size: None,
            resolution: Resolution { width: 0, height: 0 },
            total_frames: 0,
            frame_rate: 300,
            interlaced: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FFProbeJsonOutput {
    pub streams: Vec<FFProbeJsonStream>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FFProbeJsonStream {
    pub codec_name: String,
    pub codec_tag_string: Option<String>,
    pub field_order: Option<String>,
    pub width: u64,
    pub height: u64,
    pub pix_fmt: String,
    pub nb_read_packets: Option<String>,
    pub avg_frame_rate: String,
}

pub fn probe_codec(path: &PathBuf) -> Result<Codec, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        Ok(Codec::from_str(deserialized.streams[0].codec_name.as_str()))
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

pub fn probe_codec_tag(path: &PathBuf) -> Result<Option<String>, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        Ok(deserialized.streams[0].codec_tag_string.clone())
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

pub fn probe_interlaced(path: &PathBuf) -> Result<bool, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        Ok(match &deserialized.streams[0].field_order {
            Some(s) => s != "progressive",
            None => true,
        })
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

pub fn probe_resolution(path: &PathBuf) -> Result<Resolution, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        Ok(Resolution {
            width: deserialized.streams[0].width,
            height: deserialized.streams[0].height,
        })
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

pub fn probe_file_fast(path: &PathBuf) -> Result<AVProbeMetadata, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        let field_order = match &deserialized.streams[0].field_order {
            Some(s) => s,
            None => "progressive",
        };
        Ok(AVProbeMetadata {
            video_codec: Codec::from_str(deserialized.streams[0].codec_name.as_str()),
            video_codec_tag: deserialized.streams[0].codec_tag_string.clone(),
            file_size: match fs::metadata(&path) {
                Err(_) => None,
                Ok(metadata) => Some(metadata.len() as usize),
            },
            resolution: Resolution {
                width: deserialized.streams[0].width,
                height: deserialized.streams[0].height,
            },
            total_frames: 0,
            frame_rate: get_frame_rate(path, &deserialized.streams[0]).unwrap_or(300),
            interlaced: field_order != "progressive",
        })
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

pub fn probe_file(path: &PathBuf) -> Result<AVProbeMetadata, Box<dyn Error>> { 
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            &PathBuf::from("-count_packets"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        let field_order = match &deserialized.streams[0].field_order {
            Some(s) => s,
            None => "progressive",
        };
        Ok(AVProbeMetadata {
            video_codec: Codec::from_str(deserialized.streams[0].codec_name.as_str()),
            video_codec_tag: deserialized.streams[0].codec_tag_string.clone(),
            file_size: match fs::metadata(&path) {
                Err(_) => None,
                Ok(metadata) => Some(metadata.len() as usize),
            },
            resolution: Resolution {
                width: deserialized.streams[0].width,
                height: deserialized.streams[0].height,
            },
            total_frames: match &deserialized.streams[0].nb_read_packets {
                None => 1,
                Some(tf) => tf.parse().unwrap_or(1),
            },
            frame_rate: get_frame_rate(path, &deserialized.streams[0]).unwrap_or(300),
            interlaced: field_order != "progressive",
        })
    } else {
        Err(Box::new(InputParseError::for_file(path, "ffprobe did not exit successfully.")))
    }
}

fn get_frame_rate(path: &PathBuf, stream: &FFProbeJsonStream) -> Result<u64, InputParseError> {
    let splits: Vec<&str> = stream.avg_frame_rate.split("/").collect();
    match splits.len() {
        2 => {
            if let Ok(num) = splits[0].parse::<f32>() {
                if let Ok(denom) = splits[1].parse::<f32>() {
                    Ok((num / denom).round() as u64)
                } else {
                    Err(InputParseError::for_file(path, &format!("denominator '{}' from '{}' is not a number.", splits[1], &stream.avg_frame_rate)))
                }
            } else {
                    Err(InputParseError::for_file(path, &format!("numerator '{}' from '{}' is not a number.", splits[0], &stream.avg_frame_rate)))
            }
        },
        _ => Err(InputParseError::for_file(path, &format!("Unexpected avg_frame_rate format: '{}'", stream.avg_frame_rate))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_frame_rate() {
        assert_eq!(get_frame_rate(&PathBuf::from(""), &ffprobe_json_stream_from_frame_rate("25/1")).unwrap(), 25);
        assert_eq!(get_frame_rate(&PathBuf::from(""), &ffprobe_json_stream_from_frame_rate("24000/1001")).unwrap(), 24);
        assert_eq!(get_frame_rate(&PathBuf::from(""), &ffprobe_json_stream_from_frame_rate("60/1")).unwrap(), 60);
    }

    fn ffprobe_json_stream_from_frame_rate(frame_rate: &str) -> FFProbeJsonStream {
        FFProbeJsonStream {
            codec_name: String::new(),
            codec_tag_string: None,
            width: 0,
            height: 0,
            pix_fmt: String::new(),
            nb_read_packets: None,
            avg_frame_rate: String::from(frame_rate),
            field_order: None,
        }
    }
}
