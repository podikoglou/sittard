use crate::types::AudioSamples;
use anyhow::Result;

#[allow(clippy::missing_errors_doc)]
pub trait AudioRecorder {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<AudioSamples>;
}

pub mod cpal_recorder;

#[allow(clippy::missing_errors_doc)]
pub fn list_devices() -> Result<Vec<String>> {
    cpal_recorder::list_devices()
}
