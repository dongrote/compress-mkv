use std::path::PathBuf;

use crate::{codecs::Codec, containers::Container, probe::AVProbeMetadata, quality::Quality};

#[derive(Clone, Debug)]
pub struct TranscodeTask {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub metadata: AVProbeMetadata,
    pub codec: Codec,
    pub container: Container,
    pub quality: Quality,
}
