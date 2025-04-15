use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Codec {
    Unknown(String),
    AV1,
    HEVC,
    H264,
}

impl Codec {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "av1" => Codec::AV1,
            "hevc" => Codec::HEVC,
            "h264" => Codec::H264,
            _ => Codec::Unknown(String::from(s)),
        }
    }

    pub fn cv_parameter(codec: Codec) -> Option<String> {
        match codec {
            Codec::Unknown(_) => None,
            Codec::AV1 => Some(String::from("libsvtav1")),
            Codec::HEVC => Some(String::from("libx265")),
            Codec::H264 => Some(String::from("libx264")),
        }
    }
}

impl Default for Codec {
    fn default() -> Self {
        Codec::Unknown(String::new())
    }
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Codec::Unknown(codec) => write!(f, "{}", codec.to_lowercase()),
            _ => write!(f, "{}", format!("{:?}", self).to_lowercase()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Codec::AV1), "av1");
        assert_eq!(format!("{}", Codec::HEVC), "hevc");
        assert_eq!(format!("{}", Codec::H264), "h264");
    }
}
