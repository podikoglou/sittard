Sittard is a cross-platform Rust daemon that records voice on hotkey, transcribes locally via sherpa-onnx, and pastes text into the focused field.

Build and check: `cargo fmt`, `cargo clippy -- -D warnings -W clippy::pedantic`, `cargo test`
Linting/formatting is enforced via lefthook pre-commit hooks.
