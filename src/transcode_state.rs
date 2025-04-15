use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct TranscodeState {
    pub path: Option<PathBuf>,
    pub progress: Option<f64>,
    pub status: TranscodeStatus,
    pub message: Option<String>,
    pub source_size: Option<usize>,
    pub current_transcoding_size: Option<usize>,
    pub predicted_transcoded_size: Option<usize>,
    pub frames: Option<usize>,
    pub total_frames: Option<usize>,
}

impl TranscodeState {
    pub fn new() -> Self {
        TranscodeState {
            path: None,
            progress: None,
            message: None,
            status: TranscodeStatus::Idle,
            source_size: None,
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
}
