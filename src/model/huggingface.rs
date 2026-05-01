use crate::config::ModelEngine;
use anyhow::{anyhow, ensure, Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub struct SherpaOnnxProvider {
    engine: ModelEngine,
}

impl SherpaOnnxProvider {
    #[must_use]
    pub fn new(engine: ModelEngine) -> Self {
        Self { engine }
    }

    fn model_dir() -> Result<PathBuf> {
        let base = dirs::data_dir().ok_or_else(|| anyhow!("cannot determine data directory"))?;
        Ok(base.join("sittard").join("models"))
    }

    fn model_filename(&self) -> String {
        match &self.engine {
            ModelEngine::Parakeet => {
                "sherpa-onnx-nemo-parakeet_tdt_ctc_110m-en-36000-int8".to_string()
            }
            ModelEngine::Moonshine => "sherpa-onnx-moonshine-tiny-en-int8".to_string(),
            ModelEngine::WhisperTiny => "sherpa-onnx-whisper-tiny.en".to_string(),
            ModelEngine::WhisperBase => "sherpa-onnx-whisper-base".to_string(),
        }
    }

    fn download_url(&self) -> String {
        format!(
            "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/{}.tar.bz2",
            self.model_filename()
        )
    }

    async fn download_model(&self) -> Result<PathBuf> {
        let dir = Self::model_dir()?;
        std::fs::create_dir_all(&dir)?;

        let filename = self.model_filename();
        let final_dir = dir.join(&filename);
        let temp_path = dir.join(format!("{filename}.tar.bz2.tmp"));

        if final_dir.exists() {
            tracing::info!("model already extracted at {}", final_dir.display());
            return Ok(final_dir);
        }

        let url = self.download_url();
        tracing::info!("downloading model from {}", url);

        let response = reqwest::get(&url).await?;
        ensure!(
            response.status().is_success(),
            "download failed: HTTP {}",
            response.status()
        );

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )
            .context("failed to build progress bar style")?
            .progress_chars("#>-"),
        );

        let mut file = tokio::fs::File::create(&temp_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            pb.inc(chunk.len() as u64);
        }

        file.flush().await?;
        pb.finish_with_message("done");

        tracing::info!("extracting model archive");
        let extract_dir = dir.clone();
        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&temp_path)?;
            let decoder = bzip2::read::BzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&extract_dir)?;
            std::fs::remove_file(&temp_path)?;
            Ok::<(), anyhow::Error>(())
        })
        .await??;

        tracing::info!("model extracted to {}", final_dir.display());
        Ok(final_dir)
    }
}

impl super::ModelProvider for SherpaOnnxProvider {
    fn model_path(&self) -> Result<PathBuf> {
        let dir = Self::model_dir()?;
        let path = dir.join(self.model_filename());
        ensure!(path.exists(), "model not found at {}", path.display());
        Ok(path)
    }

    async fn ensure_model(&self) -> Result<PathBuf> {
        match self.model_path() {
            Ok(path) => {
                tracing::info!("model already exists at {}", path.display());
                Ok(path)
            }
            Err(_) => self.download_model().await,
        }
    }
}
