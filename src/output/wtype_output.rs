use anyhow::{anyhow, Result};
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
        .map_err(|_| anyhow!("wtype not found. install: xbps-install wtype"))?
        .status
        .success()
        .then_some(())
        .ok_or_else(|| anyhow!("wtype not found. install: xbps-install wtype"))
}

impl TextOutput for WtypeOutput {
    fn paste(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        let status = Command::new("wtype")
            .arg(text)
            .status()
            .map_err(|_| anyhow!("wtype not found. install: xbps-install wtype"))?;

        if !status.success() {
            return Err(anyhow!("wtype exited with status: {status}"));
        }

        Ok(())
    }
}
