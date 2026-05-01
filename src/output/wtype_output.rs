use anyhow::{ensure, Context, Result};
use std::process::Command;

use super::TextOutput;

pub struct WtypeOutput;

impl WtypeOutput {
    #[allow(clippy::missing_errors_doc)]
    pub fn new() -> Result<Self> {
        which_wtype()?;
        Ok(Self)
    }
}

fn which_wtype() -> Result<()> {
    Command::new("which")
        .arg("wtype")
        .output()
        .context("wtype not found. install: xbps-install wtype")?
        .status
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("wtype not found. install: xbps-install wtype"))
}

impl TextOutput for WtypeOutput {
    fn paste(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        let status = Command::new("wtype")
            .arg(text)
            .status()
            .context("wtype not found. install: xbps-install wtype")?;

        ensure!(status.success(), "wtype exited with status: {status}");

        Ok(())
    }
}
