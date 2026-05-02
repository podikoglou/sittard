use super::Transcriber;
use crate::config::ModelEngine;
use crate::types::AudioSamples;
use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::sync::mpsc;
use transcribe_rs::onnx::canary::CanaryModel;
use transcribe_rs::onnx::parakeet::ParakeetModel;
use transcribe_rs::onnx::Quantization;
use transcribe_rs::{SpeechModel, TranscribeOptions};

type TranscribeRequest = (AudioSamples, mpsc::Sender<Result<String>>);

enum LoadedModel {
    Parakeet(ParakeetModel),
    Canary(CanaryModel),
}

impl LoadedModel {
    fn transcribe(&mut self, audio: &[f32]) -> Result<String> {
        let options = TranscribeOptions::default();

        let result = match self {
            LoadedModel::Parakeet(model) => model
                .transcribe_raw(audio, &options)
                .map_err(|e| anyhow!("parakeet transcription failed: {e}"))?,
            LoadedModel::Canary(model) => model
                .transcribe_raw(audio, &options)
                .map_err(|e| anyhow!("canary transcription failed: {e}"))?,
        };

        Ok(result.text)
    }
}

pub struct SherpaOnnxEngine {
    tx: mpsc::Sender<TranscribeRequest>,
}

impl SherpaOnnxEngine {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(model_dir: &Path, engine: ModelEngine, _threads: usize) -> Result<Self> {
        let model_dir_path = model_dir.to_path_buf();

        let (tx, rx) = mpsc::channel::<TranscribeRequest>();

        std::thread::spawn(move || {
            if let Err(e) = worker_loop(&model_dir_path, engine, &rx) {
                tracing::error!("transcribe-rs worker thread failed: {e}");
            }
        });

        Ok(Self { tx })
    }
}

fn worker_loop(
    model_dir: &Path,
    engine: ModelEngine,
    rx: &mpsc::Receiver<TranscribeRequest>,
) -> Result<()> {
    let mut model = load_model(model_dir, engine)?;
    tracing::info!("transcribe-rs model loaded");

    while let Ok((samples, reply)) = rx.recv() {
        let result = if samples.0.len() < 1600 {
            Ok(String::new())
        } else {
            model.transcribe(&samples.0)
        };

        let _ = reply.send(result);
    }

    Ok(())
}

fn load_model(model_dir: &Path, engine: ModelEngine) -> Result<LoadedModel> {
    match engine {
        ModelEngine::Parakeet => {
            let model = ParakeetModel::load(model_dir, &Quantization::Int8)
                .map_err(|e| anyhow!("failed to load parakeet model: {e}"))?;
            Ok(LoadedModel::Parakeet(model))
        }
        ModelEngine::Canary => {
            let model = CanaryModel::load(model_dir, &Quantization::Int8)
                .map_err(|e| anyhow!("failed to load canary model: {e}"))?;
            Ok(LoadedModel::Canary(model))
        }
        _ => Err(anyhow!(
            "only Parakeet and Canary models are supported via transcribe-rs"
        )),
    }
}

impl Transcriber for SherpaOnnxEngine {
    fn transcribe(&self, samples: AudioSamples) -> Result<String> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.tx
            .send((samples, reply_tx))
            .context("transcribe-rs worker thread died")?;

        reply_rx
            .recv()
            .context("transcribe-rs worker thread died")?
    }
}
