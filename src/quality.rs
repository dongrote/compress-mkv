use crate::codecs::Codec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Quality {
    Insane,
    Excellent,
    Great,
    Good,
    Fast,
}


impl Quality {
    pub fn parameters(codec: Codec, quality: Quality) -> Vec<String> {
        match codec {
            Codec::Unknown(_) => vec![],
            Codec::AV1 => av1_parameters(quality),
            Codec::HEVC => hevc_parameters(quality),
            Codec::H264 => h264_parameters(quality),
        }
    }
}

fn av1_parameters(quality: Quality) -> Vec<String> {
    match quality {
        Quality::Insane => vec![
            String::from("-c:v"), String::from("libsvtav1"),
            String::from("-crf"), String::from("8"),
            String::from("-preset"), String::from("2"),
        ],
        Quality::Excellent => vec![
            String::from("-c:v"), String::from("libsvtav1"),
            String::from("-crf"), String::from("18"),
            String::from("-preset"), String::from("2"),
        ],
        Quality::Great => vec![
            String::from("-c:v"), String::from("libsvtav1"),
            String::from("-crf"), String::from("22"),
            String::from("-preset"), String::from("3"),
        ],
        Quality::Good => vec![
            String::from("-c:v"), String::from("libsvtav1"),
            String::from("-crf"), String::from("25"),
            String::from("-preset"), String::from("8"),
        ],
        Quality::Fast => vec![
            String::from("-c:v"), String::from("libsvtav1"),
            String::from("-crf"), String::from("25"),
            String::from("-preset"), String::from("12"),
        ],
    }
}

fn hevc_parameters(quality: Quality) -> Vec<String> {
    match quality {
        Quality::Insane => vec![
            String::from("-c:v"), String::from("libx265"),
            String::from("-crf"), String::from("18"),
            String::from("-preset"), String::from("veryslow"),
        ],
        Quality::Excellent => vec![
            String::from("-c:v"), String::from("libx265"),
            String::from("-crf"), String::from("20"),
            String::from("-preset"), String::from("slower"),
        ],
        Quality::Great => vec![
            String::from("-c:v"), String::from("libx265"),
            String::from("-crf"), String::from("25"),
            String::from("-preset"), String::from("medium"),
        ],
        Quality::Good => vec![
            String::from("-c:v"), String::from("libx265"),
            String::from("-crf"), String::from("28"),
            String::from("-preset"), String::from("fast"),
        ],
        Quality::Fast => vec![
            String::from("-c:v"), String::from("libx265"),
            String::from("-crf"), String::from("30"),
            String::from("-preset"), String::from("veryfast"),
        ],
    }
}

fn h264_parameters(quality: Quality) -> Vec<String> {
    match quality {
        Quality::Insane => vec![
            String::from("-c:v"), String::from("libx264"),
            String::from("-crf"), String::from("8"),
            String::from("-preset"), String::from("veryslow"),
        ],
        Quality::Excellent => vec![
            String::from("-c:v"), String::from("libx264"),
            String::from("-crf"), String::from("16"),
            String::from("-preset"), String::from("slower"),
        ],
        Quality::Great => vec![
            String::from("-c:v"), String::from("libx264"),
            String::from("-crf"), String::from("18"),
            String::from("-preset"), String::from("medium"),
        ],
        Quality::Good => vec![
            String::from("-c:v"), String::from("libx264"),
            String::from("-crf"), String::from("22"),
            String::from("-preset"), String::from("fast"),
        ],
        Quality::Fast => vec![
            String::from("-c:v"), String::from("libx264"),
            String::from("-crf"), String::from("24"),
            String::from("-preset"), String::from("veryfast"),
        ],
    }
}
