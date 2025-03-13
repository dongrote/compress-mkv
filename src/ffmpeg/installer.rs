use std::err::Error;

pub trait FFmpegInstaller {
    fn install_cmd() -> String;

    fn install() -> Result<(), Box<dyn Error>>;
}

struct DebianFFmpegInstaller {
}

impl FFmpegInstaller for DebianFFmpegInstaller {
    fn install_cmd() -> String {
        // get euid; if not 0 then prefix command with sudo
        String::from("apt install ffmpeg")
    }

    fn install() -> Result<(), Box<dyn Error>> {
    }
}
