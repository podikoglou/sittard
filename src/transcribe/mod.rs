use crate::types::AudioSamples;
use anyhow::Result;

pub trait Transcriber: Send + Sync {
    #[allow(clippy::missing_errors_doc)]
    fn transcribe(&self, samples: AudioSamples) -> Result<String>;
}

pub mod whisper_engine;
