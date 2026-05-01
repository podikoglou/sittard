Sittard is a cross-paltform Rust background daemon for that records voice on hotkey, transcribes locally via whisper.cpp, and pastes text into the focused field.

Build and check: `cargo build`, `cargo clippy -- -D warnings`, `cargo test`
Target: Linux only. Do not introduce cross-platform code.
