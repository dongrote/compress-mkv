use core::f64;
use std::{fmt::Display, path::PathBuf};
use std::fs;
use humanize_bytes::humanize_bytes_decimal;

use crate::codecs::Codec;
use crate::probe::{probe_file_fast, AVProbeMetadata};

#[derive(Clone, Debug)]
pub struct FileListItem {
    pub path: PathBuf,
    pub avmetadata: AVProbeMetadata,
    pub status: FileListItemStatus,
    pub size: Option<usize>,
    pub codec: Option<Codec>,
    pub resolution: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileListItemStatus {
    Invalid,
    Unknown,
    Candidate,
    Enqueued,
    Transcoding,
    Transcoded,
    Analyzing,
}

impl FileListItem {
    pub fn new(path: PathBuf) -> Option<Self>{
        match probe_file_fast(&path) {
            Err(_) => None,
            Ok(probe) => match probe.video_codec {
                Codec::AV1 => None,
                Codec::H264 => Some(FileListItem::from(path, probe)),
                Codec::Unknown(_) => None,
                Codec::HEVC => match &probe.video_codec_tag {
                    Some(tag) => match tag.as_str() {
                        "hvc1" => None,
                        _ => Some(FileListItem::from(path, probe)),
                    },
                    None => Some(FileListItem::from(path, probe)),
                },
            }
        }
    }

    fn from(path: PathBuf, probe: AVProbeMetadata) -> FileListItem {
        let size = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };
        FileListItem {
            path,
            avmetadata: probe.clone(),
            status: FileListItemStatus::Candidate,
            size: Some(size as usize),
            codec: Some(probe.video_codec),
            resolution: Some(format!("{}", probe.resolution)) }
    }

    pub fn set_candidate(&mut self) {
        self.status = FileListItemStatus::Candidate;
    }

    pub fn set_transcoding(&mut self) {
        self.status = FileListItemStatus::Transcoding;
    }

    pub fn set_transcoded(&mut self) {
        self.status = FileListItemStatus::Transcoded;
    }

    pub fn set_enqueued(&mut self) {
        self.status = FileListItemStatus::Enqueued;
    }
}

impl Display for FileListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_glyph = format!("{}", self.status);

        let resolution_str = match &self.resolution {
            None => "----x----",
            Some(res) => &format!("{:<9}", &res),
        };

        let codec_str = match &self.codec {
            None => "--------",
            Some(c) => &format!("{:<8}", c),
        };

        let size_str = match self.size {
            None => "------ B",
            Some(s) => &format!("{:<8}", humanize_bytes_decimal!(s)),
        };

        write!(
            f,
            "{} {} {} {} {}",
            status_glyph,
            resolution_str,
            codec_str,
            size_str,
            self.path.display())
    }
}

impl Display for FileListItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            FileListItemStatus::Unknown => "🤷",
            FileListItemStatus::Invalid => "🚫",
            FileListItemStatus::Candidate => "☐",
            FileListItemStatus::Enqueued => "☑",
            FileListItemStatus::Transcoding => "🚧",
            FileListItemStatus::Transcoded => "✅",
            FileListItemStatus::Analyzing => "🔎",
        };
        write!(f, "{}", status_str)
    }
}