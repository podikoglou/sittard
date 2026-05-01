use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::types::InputEvent;

pub trait HotkeyListener {
    #[allow(clippy::missing_errors_doc)]
    fn start(&self, sender: UnboundedSender<InputEvent>, cancel: CancellationToken) -> Result<()>;
}

pub mod evdev_listener;
pub mod keymap;
