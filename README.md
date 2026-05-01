# sittard

Linux voice-to-text daemon. Hold a hotkey, speak, release. Text gets pasted into whatever field is focused.

All transcription runs locally. Nothing leaves your machine.

## Requirements

- Linux with evdev input devices
- PipeWire or PulseAudio
- [wtype](https://github.com/atx/wtype) (for pasting text on Wayland)
- cmake, C compiler (build time)

## Install

```
cargo install --git https://github.com/podikoglou/sittard
```

## Build

```
cargo build --release
```

## Setup

1. Add your user to the `input` group (needed to read keyboard events):

```
sudo usermod -aG input $USER
```

Log out and back in for this to take effect.

2. Install `wtype`:

```
# Void Linux
xbps-install wtype

# Debian/Ubuntu
sudo apt install wtype
```

## Usage

```
# Run with defaults (right_alt hotkey, hold mode, parakeet engine)
sittard

# Use a different hotkey
sittard --hotkey f13
sittard --hotkey "ctrl+space"
sittard --hotkey "alt+shift+r"

# Toggle mode instead of hold
sittard --mode toggle

# Use a different transcription engine
sittard --engine moonshine
sittard --engine whisper-tiny

# List available keys for hotkey binding
sittard list-keys

# List audio input devices
sittard list-devices

# Download a model explicitly
sittard download-model
sittard download-model --engine moonshine
```

## Hotkeys

Single keys or modifier combos. Combine with `+`.

```
--hotkey right_alt          # single key
--hotkey f13                # function key
--hotkey "ctrl+space"       # modifier combo
--hotkey "alt+shift+r"      # multi-modifier combo
--hotkey super              # meta/win key
```

Modifier aliases match either side: `ctrl` = left or right ctrl. Specific sides work too: `left_ctrl`, `right_alt`, etc.

Use `sittard list-keys` to see all key names.

## Modes

**Hold** (default): Press and hold the hotkey to record. Release to transcribe and paste.

**Toggle**: Press once to start recording. Press again to stop, transcribe, and paste.

## Engines

| Engine | Model | Speed | Notes |
|--------|-------|-------|-------|
| `parakeet` | NVIDIA Parakeet TDT CTC 110M (int8) | Fast | Default. Good accuracy for English. |
| `moonshine` | Useful Sensors Moonshine Tiny (int8) | Very fast | Small model, designed for edge devices. |
| `whisper-tiny` | OpenAI Whisper Tiny EN | Medium | Well-tested, decent accuracy. |
| `whisper-base` | OpenAI Whisper Base EN | Slower | Better accuracy than tiny. |

Models are downloaded automatically on first run to `~/.local/share/sittard/models/`.

## Flags

```
--hotkey <key>       Hotkey binding (default: right_alt)
--mode <mode>        hold or toggle (default: hold)
--engine <engine>    parakeet, moonshine, whisper-tiny, whisper-base (default: parakeet)
--threads <n>        Number of threads for transcription (default: CPU count)
--device <name>      Audio input device name
-v                   Verbose logging (-v info, -vv debug)
-D                   Trace logging
```

## Permissions

If you get permission errors:

```
no input devices accessible. add user to input group: sudo usermod -aG input $USER
```

## License

MIT
