Staid is a cross-paltform Rust background daemon for that records voice on hotkey, transcribes locally via whisper.cpp, and pastes text into the focused field.

Build and check: `cargo build`, `cargo clippy -- -D warnings`, `cargo test`
Target: Linux only. Do not introduce cross-platform code.

For architecture and trait boundaries, see [ABSTRACTIONS.md](docs/ABSTRACTIONS.md).
For dependencies and subsystem implementation details, see [LIBRARIES.md](docs/LIBRARIES.md).
For the implementation checklist, see [PLAN.md](docs/PLAN.md).
