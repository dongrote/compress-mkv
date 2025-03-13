use std::process::Command;
pub mod compressors;
pub mod probe;

pub struct FFmpeg {
}

impl FFmpeg {
    pub fn new() -> Self {
        FFmpeg {  }
    }

    pub fn is_installed(&self) -> bool {
        let cmd = Command::new("ffmpeg")
            .arg("-codecs")
            .output();
        match cmd {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}
