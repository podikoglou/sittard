# Architecture

## Pipeline

```
[Hotkey] → [Audio] → [Transcribe] → [Paste]
   │           │           │            │
 input/      audio/    transcribe/    output/
 evdev       cpal       sherpa-onnx   wtype
```

## Modules

| Module | Responsibility |
|--------|---------------|
| `config` | CLI parsing (clap), `AppConfig`, `ModelEngine`, `InteractionMode` |
| `input` | Hotkey detection via evdev. Parses key combos (`ctrl+shift+f12`), spawns a thread per input device |
| `audio` | Mic recording via cpal. Handles device selection, format negotiation (i16/f32), resampling to 16kHz mono |
| `model` | Model download and caching. Fetches sherpa-onnx model tarballs from GitHub, stores in `~/.local/share/sittard/models/` |
| `transcribe` | Speech-to-text via sherpa-onnx `OfflineRecognizer`. Runs on a dedicated thread with channel-based request/reply |
| `output` | Text injection. Currently `wtype` (Wayland). Shells out to `wtype` binary |
| `app` | Event loop tying everything together. Manages `Idle → Recording → Transcribing` state machine |
| `types` | Shared types: `AudioSamples`, `InputEvent`, `AppEvent`, `AppState` |

## Extensibility

Each pipeline stage is a trait, so backends are swappable:

- `HotkeyListener` — swap evdev for another input source
- `AudioRecorder` — swap cpal for another audio backend
- `Transcriber` — swap sherpa-onnx for another STT engine
- `TextOutput` — swap wtype for xdotool, xclip, etc.
- `ModelProvider` — swap HuggingFace for another model source

## Threading

- Main thread: tokio async runtime, event loop (`tokio::select!`)
- Input: one std thread per evdev device → sends `InputEvent` via unbounded channel
- Transcription: one dedicated std thread owns the `OfflineRecognizer` — receives requests via `mpsc::channel`, replies via per-request channel
- Signal handlers: tokio tasks for SIGINT/SIGTERM → send `AppEvent::Shutdown`

## State Machine

```
Idle ──keydown──→ Recording ──keyup──→ Transcribing ──done──→ Idle
```

Hold mode: keydown starts recording, keyup stops. Toggle mode: keydown toggles start/stop, ignores keyup.
