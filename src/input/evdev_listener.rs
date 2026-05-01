use anyhow::anyhow;
use anyhow::Result;
use evdev::{Device, InputEventKind, Key};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use super::keymap;
use super::keymap::HotkeyCombo;
use super::HotkeyListener;
use crate::types::InputEvent;

pub struct EvdevListener {
    combo: HotkeyCombo,
}

impl EvdevListener {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(hotkey_spec: &str) -> Result<Self> {
        let combo = keymap::parse_hotkey_combo(hotkey_spec)?;
        Ok(Self { combo })
    }
}

impl HotkeyListener for EvdevListener {
    fn start(&self, sender: UnboundedSender<InputEvent>, cancel: CancellationToken) -> Result<()> {
        let devices: Vec<Device> = evdev::enumerate()
            .filter_map(|(_, device)| {
                device
                    .supported_keys()
                    .is_some_and(|keys| self.combo.all_keys.iter().any(|k| keys.contains(*k)))
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
            let combo = self.combo.clone();
            let sender_clone = sender.clone();
            let cancelled_clone = cancelled.clone();

            std::thread::spawn(move || {
                listen_on_device(device, combo, sender_clone, cancelled_clone);
            });
        }

        Ok(())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn listen_on_device(
    mut device: Device,
    combo: HotkeyCombo,
    sender: UnboundedSender<InputEvent>,
    cancelled: Arc<AtomicBool>,
) {
    let mut held_keys: HashSet<Key> = HashSet::new();
    let mut matched = false;

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
                if !combo.all_keys.contains(&key) {
                    continue;
                }

                match ev.value() {
                    1 => {
                        held_keys.insert(key);
                    }
                    0 => {
                        held_keys.remove(&key);
                    }
                    _ => continue,
                }

                let now_matched = combo.matches(&held_keys);

                if now_matched && !matched {
                    let _ = sender.send(InputEvent::KeyDown);
                } else if !now_matched && matched {
                    let _ = sender.send(InputEvent::KeyUp);
                }

                matched = now_matched;
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
            "found {} keyboard(s) ({}) but none support the hotkey. try a different key with --hotkey",
            keyboards.len(),
            names.join(", ")
        )
    }
}
