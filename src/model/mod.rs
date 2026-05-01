pub mod huggingface;

use anyhow::Result;
use std::path::PathBuf;

#[allow(async_fn_in_trait)]
pub trait ModelProvider {
    #[allow(clippy::missing_errors_doc)]
    fn model_path(&self) -> Result<PathBuf>;
    #[allow(clippy::missing_errors_doc)]
    async fn ensure_model(&self) -> Result<PathBuf>;
}
