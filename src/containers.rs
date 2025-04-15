#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Container {
    Matroska,
    MP4,
    QuickTime,
}

impl ToString for Container {
    fn to_string(&self) -> String {
        match self {
            Container::Matroska => String::from("matroska"),
            Container::MP4 => String::from("mp4"),
            Container::QuickTime => String::from("mov"),
        }
    }
}

impl Container {
    pub fn extension(container: Container) -> &'static str {
        match container {
            Container::Matroska => "mkv",
            Container::MP4 => "mp4",
            Container::QuickTime => "mov",
        }
    }

    pub fn parameters(container: Container) -> Vec<String> {
        match container {
            Container::Matroska => vec![
                String::from("-f"),
                container.to_string(),
            ],
            Container::QuickTime => vec![
                String::from("-movflags"),
                String::from("faststart"),
                String::from("-f"),
                container.to_string(),
            ],
            Container::MP4 => vec![
                String::from("-movflags"),
                String::from("faststart"),
                String::from("-f"),
                container.to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        assert_eq!(Container::Matroska.to_string(), String::from("matroska"));
        assert_eq!(Container::MP4.to_string(), String::from("mp4"));
        assert_eq!(Container::QuickTime.to_string(), String::from("mov"));
    }
}
