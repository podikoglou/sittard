use super::Transcriber;
use crate::config::ModelEngine;
use crate::types::AudioSamples;
use anyhow::{anyhow, Result};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

type TranscribeRequest = (AudioSamples, mpsc::Sender<Result<String>>);

pub struct SherpaOnnxEngine {
    tx: mpsc::Sender<TranscribeRequest>,
}

impl SherpaOnnxEngine {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(model_dir: &Path, engine: ModelEngine, threads: usize) -> Result<Self> {
        let model_dir_str = model_dir
            .to_str()
            .ok_or_else(|| anyhow!("model path contains invalid UTF-8"))?
            .to_string();

        let (tx, rx) = mpsc::channel::<TranscribeRequest>();

        std::thread::spawn(move || {
            if let Err(e) = worker_loop(&model_dir_str, engine, threads, &rx) {
                tracing::error!("sherpa-onnx worker thread failed: {e}");
            }
        });

        Ok(Self { tx })
    }
}

fn worker_loop(
    model_dir: &str,
    engine: ModelEngine,
    threads: usize,
    rx: &mpsc::Receiver<TranscribeRequest>,
) -> Result<()> {
    let recognizer = create_recognizer(model_dir, engine, threads)?;
    tracing::info!("sherpa-onnx model loaded");

    while let Ok((samples, reply)) = rx.recv() {
        let result = if samples.0.len() < 1600 {
            Ok(String::new())
        } else {
            transcribe_inner(&recognizer, &samples.0)
        };

        let _ = reply.send(result);
    }

    Ok(())
}

fn create_recognizer(
    model_dir: &str,
    engine: ModelEngine,
    threads: usize,
) -> Result<OfflineRecognizer> {
    let model_dir_path = PathBuf::from(model_dir);
    let mut config = OfflineRecognizerConfig::default();
    config.model_config.num_threads = i32::try_from(threads).unwrap_or(1);

    match engine {
        ModelEngine::Parakeet => {
            let model = model_dir_path.join("model.int8.onnx");
            let tokens = model_dir_path.join("tokens.txt");

            config.model_config.nemo_ctc.model = Some(
                model
                    .to_str()
                    .ok_or_else(|| anyhow!("model path invalid UTF-8"))?
                    .to_string(),
            );

            config.model_config.tokens = Some(
                tokens
                    .to_str()
                    .ok_or_else(|| anyhow!("tokens path invalid UTF-8"))?
                    .to_string(),
            );
            config.model_config.model_type = Some("nemo_ctc".to_string());
        }
        ModelEngine::Moonshine => {
            let encoder = model_dir_path.join("encoder.onnx");
            let decoder = model_dir_path.join("decoder.onnx");
            let tokens = model_dir_path.join("tokens.txt");

            config.model_config.moonshine.encoder = Some(
                encoder
                    .to_str()
                    .ok_or_else(|| anyhow!("encoder path invalid UTF-8"))?
                    .to_string(),
            );
            config.model_config.moonshine.merged_decoder = Some(
                decoder
                    .to_str()
                    .ok_or_else(|| anyhow!("decoder path invalid UTF-8"))?
                    .to_string(),
            );

            config.model_config.tokens = Some(
                tokens
                    .to_str()
                    .ok_or_else(|| anyhow!("tokens path invalid UTF-8"))?
                    .to_string(),
            );
        }
        ModelEngine::WhisperTiny | ModelEngine::WhisperBase => {
            let encoder = model_dir_path.join("encoder.onnx");
            let decoder = model_dir_path.join("decoder.onnx");
            let tokens = model_dir_path.join("tokens.txt");

            config.model_config.whisper.encoder = Some(
                encoder
                    .to_str()
                    .ok_or_else(|| anyhow!("encoder path invalid UTF-8"))?
                    .to_string(),
            );
            config.model_config.whisper.decoder = Some(
                decoder
                    .to_str()
                    .ok_or_else(|| anyhow!("decoder path invalid UTF-8"))?
                    .to_string(),
            );

            config.model_config.tokens = Some(
                tokens
                    .to_str()
                    .ok_or_else(|| anyhow!("tokens path invalid UTF-8"))?
                    .to_string(),
            );
        }
    }

    OfflineRecognizer::create(&config)
        .ok_or_else(|| anyhow!("failed to create sherpa-onnx recognizer"))
}

fn transcribe_inner(recognizer: &OfflineRecognizer, audio: &[f32]) -> Result<String> {
    let stream = recognizer.create_stream();

    stream.accept_waveform(16000, audio);

    recognizer.decode(&stream);

    let result = stream
        .get_result()
        .ok_or_else(|| anyhow!("failed to get transcription result"))?;

    Ok(result.text)
}

impl Transcriber for SherpaOnnxEngine {
    fn transcribe(&self, samples: AudioSamples) -> Result<String> {
        let (reply_tx, reply_rx) = mpsc::channel();

        self.tx
            .send((samples, reply_tx))
            .map_err(|_| anyhow!("sherpa-onnx worker thread died"))?;

        reply_rx
            .recv()
            .map_err(|_| anyhow!("sherpa-onnx worker thread died"))?
    }
}
