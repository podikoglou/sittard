use anyhow::Result;

pub trait TextOutput: Send + Sync {
    #[allow(clippy::missing_errors_doc)]
    fn paste(&self, text: &str) -> Result<()>;
}

pub mod wtype_output;
