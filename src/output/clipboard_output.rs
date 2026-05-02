use anyhow::{ensure, Context, Result};
use std::io::Write;
use std::process::Command;
use std::thread;
use std::time::Duration;

use super::TextOutput;

pub struct ClipboardOutput;

impl ClipboardOutput {
    #[allow(clippy::missing_errors_doc)]
    pub fn new() -> Result<Self> {
        which_wl_copy()?;
        which_wl_paste()?;
        which_wtype()?;
        Ok(Self)
    }
}

fn which_wl_copy() -> Result<()> {
    Command::new("which")
        .arg("wl-copy")
        .output()
        .context("wl-copy not found. install: xbps-install wl-clipboard")?
        .status
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("wl-copy not found. install: xbps-install wl-clipboard"))
}

fn which_wl_paste() -> Result<()> {
    Command::new("which")
        .arg("wl-paste")
        .output()
        .context("wl-paste not found. install: xbps-install wl-clipboard")?
        .status
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("wl-paste not found. install: xbps-install wl-clipboard"))
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

impl TextOutput for ClipboardOutput {
    fn paste(&self, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }

        // Save current clipboard contents
        let original_clipboard = Command::new("wl-paste").output().ok().and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        });

        // Copy new text to clipboard
        let mut child = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("failed to spawn wl-copy")?;

        {
            let stdin = child
                .stdin
                .as_mut()
                .context("failed to open stdin for wl-copy")?;
            stdin
                .write_all(text.as_bytes())
                .context("failed to write to wl-copy stdin")?;
        }

        let status = child.wait().context("failed to wait for wl-copy")?;
        ensure!(status.success(), "wl-copy exited with status: {status}");

        // Wait a bit for the compositor to sync the clipboard
        thread::sleep(Duration::from_millis(50));

        // Paste via Ctrl+V
        let status = Command::new("wtype")
            .args(["-k", "ctrl+v"])
            .status()
            .context("failed to run wtype")?;
        ensure!(status.success(), "wtype exited with status: {status}");

        // Wait a bit after pasting
        thread::sleep(Duration::from_millis(50));

        // Restore original clipboard contents if available
        if let Some(original) = original_clipboard {
            let mut child = Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .ok();

            if let Some(ref mut ch) = child {
                if let Some(stdin) = ch.stdin.as_mut() {
                    let _ = stdin.write_all(original.as_bytes());
                }
                let _ = ch.wait();
            }
        }

        Ok(())
    }
}
