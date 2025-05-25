use std::fmt::Display;
use std::path::PathBuf;

use uuid::Uuid;

use crate::codecs::Codec;

#[derive(Clone, Debug)]
pub struct TranscodeState {
    pub id: Uuid,
    pub path: Option<PathBuf>,
    pub progress: Option<f64>,
    pub status: TranscodeStatus,
    pub message: Option<String>,
    pub source_size: Option<usize>,
    pub source_codec: Option<Codec>,
    pub transcode_codec: Option<Codec>,
    pub current_transcoding_size: Option<usize>,
    pub predicted_transcoded_size: Option<usize>,
    pub frames: Option<usize>,
    pub total_frames: Option<usize>,
}

impl TranscodeState {
    pub fn new() -> Self {
        TranscodeState {
            id: Uuid::new_v4(),
            path: None,
            progress: None,
            message: None,
            status: TranscodeStatus::Idle,
            source_size: None,
            source_codec: None,
            transcode_codec: None,
            current_transcoding_size: None,
            predicted_transcoded_size: None,
            frames: None,
            total_frames: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TranscodeStatus {
    Idle,
    Transcoding,
    Complete,
    Error,
}

impl Display for TranscodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            TranscodeStatus::Idle => "😴",
            TranscodeStatus::Transcoding => "▶️",
            TranscodeStatus::Complete => "✅",
            TranscodeStatus::Error => "🚫",
        };
        write!(f, "{}", status_str)
    }
}