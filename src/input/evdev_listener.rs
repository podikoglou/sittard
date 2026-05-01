use anyhow::anyhow;
use anyhow::Result;
use evdev::{Device, InputEventKind, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use super::keymap;
use super::HotkeyListener;
use crate::types::InputEvent;

pub struct EvdevListener {
    target_key: Key,
}

impl EvdevListener {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(key_name: &str) -> Result<Self> {
        let target_key = keymap::parse_key_name(key_name)?;
        Ok(Self { target_key })
    }
}

impl HotkeyListener for EvdevListener {
    fn start(&self, sender: UnboundedSender<InputEvent>, cancel: CancellationToken) -> Result<()> {
        let devices: Vec<Device> = evdev::enumerate()
            .filter_map(|(_, device)| {
                device
                    .supported_keys()
                    .is_some_and(|keys| keys.contains(self.target_key))
                    .then_some(device)
            })
            .collect();

        if devices.is_empty() {
            return Err(diagnose_no_devices());
        }

        let cancelled = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel.clone();
        let cancelled_clone = cancelled.clone();

        tokio::spawn(async move {
            cancel_clone.cancelled().await;
            cancelled_clone.store(true, Ordering::Relaxed);
        });

        for device in devices {
            let target_key = self.target_key;
            let sender_clone = sender.clone();
            let cancelled_clone = cancelled.clone();

            std::thread::spawn(move || {
                listen_on_device(device, target_key, sender_clone, cancelled_clone);
            });
        }

        Ok(())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn listen_on_device(
    mut device: Device,
    target_key: Key,
    sender: UnboundedSender<InputEvent>,
    cancelled: Arc<AtomicBool>,
) {
    loop {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }

        let Ok(events) = device.fetch_events() else {
            break;
        };

        for ev in events {
            if cancelled.load(Ordering::Relaxed) {
                return;
            }

            if let InputEventKind::Key(key) = ev.kind() {
                if key == target_key {
                    match ev.value() {
                        1 => {
                            let _ = sender.send(InputEvent::KeyDown);
                        }
                        0 => {
                            let _ = sender.send(InputEvent::KeyUp);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn diagnose_no_devices() -> anyhow::Error {
    let accessible: Vec<_> = evdev::enumerate().collect();

    if accessible.is_empty() {
        return anyhow!(
            "no input devices accessible. add user to input group: sudo usermod -a -G input $USER"
        );
    }

    let keyboards: Vec<_> = accessible
        .iter()
        .filter(|(_, d)| {
            d.supported_keys()
                .is_some_and(|keys| keys.contains(Key::KEY_SPACE))
        })
        .collect();

    if keyboards.is_empty() {
        anyhow!(
            "no keyboard devices accessible. add user to input group: sudo usermod -a -G input $USER"
        )
    } else {
        let names: Vec<String> = keyboards
            .iter()
            .filter_map(|(_, d)| d.name().map(std::string::ToString::to_string))
            .collect();
        anyhow!(
            "found {} keyboard(s) ({}) but none support the target key. try a different key with --hotkey",
            keyboards.len(),
            names.join(", ")
        )
    }
}
