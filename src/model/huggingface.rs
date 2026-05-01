use crate::config::ModelSize;
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub struct HuggingFaceProvider {
    model_size: ModelSize,
}

impl HuggingFaceProvider {
    #[must_use]
    pub fn new(model_size: ModelSize) -> Self {
        Self { model_size }
    }

    fn model_dir() -> Result<PathBuf> {
        let base = dirs::data_dir().ok_or_else(|| anyhow!("cannot determine data directory"))?;
        Ok(base.join("staid").join("models"))
    }

    fn model_filename(&self) -> String {
        format!("ggml-{}.bin", self.model_size)
    }

    fn download_url(&self) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
            self.model_filename()
        )
    }

    async fn download_model(&self) -> Result<PathBuf> {
        let dir = Self::model_dir()?;
        std::fs::create_dir_all(&dir)?;

        let filename = self.model_filename();
        let final_path = dir.join(&filename);
        let temp_path = dir.join(format!("{filename}.tmp"));

        let url = self.download_url();
        tracing::info!("downloading model from {}", url);

        let response = reqwest::get(&url).await?;
        if !response.status().is_success() {
            return Err(anyhow!("download failed: HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
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

        std::fs::rename(&temp_path, &final_path)?;
        tracing::info!("model saved to {}", final_path.display());
        Ok(final_path)
    }
}

impl super::ModelProvider for HuggingFaceProvider {
    fn model_path(&self) -> Result<PathBuf> {
        let dir = Self::model_dir()?;
        let path = dir.join(self.model_filename());
        if path.exists() {
            Ok(path)
        } else {
            Err(anyhow!("model not found at {}", path.display()))
        }
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
