use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::audio::AudioRecorder;
use crate::config::InteractionMode;
use crate::input::HotkeyListener;
use crate::output::TextOutput;
use crate::transcribe::Transcriber;
use crate::types::{AppEvent, AppState, InputEvent};

pub struct StaidApp {
    recorder: Box<dyn AudioRecorder>,
    listener: Box<dyn HotkeyListener>,
    engine: Arc<dyn Transcriber>,
    output: Box<dyn TextOutput>,
    state: AppState,
    event_rx: UnboundedReceiver<AppEvent>,
    event_tx: UnboundedSender<AppEvent>,
    input_tx: UnboundedSender<InputEvent>,
    input_rx: UnboundedReceiver<InputEvent>,
    cancel: CancellationToken,
    mode: InteractionMode,
}

impl StaidApp {
    pub fn new(
        recorder: Box<dyn AudioRecorder>,
        listener: Box<dyn HotkeyListener>,
        engine: Arc<dyn Transcriber>,
        output: Box<dyn TextOutput>,
        cancel: CancellationToken,
        mode: InteractionMode,
    ) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<AppEvent>();
        let (input_tx, input_rx) = tokio::sync::mpsc::unbounded_channel::<InputEvent>();
        Self {
            recorder,
            listener,
            engine,
            output,
            state: AppState::Idle,
            event_rx,
            event_tx,
            input_tx,
            input_rx,
            cancel,
            mode,
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn start(&mut self) -> Result<()> {
        self.listener
            .start(self.input_tx.clone(), self.cancel.clone())?;

        self.register_signal_handlers();

        tracing::info!("staid ready, listening for hotkey");

        self.run_event_loop().await;

        Ok(())
    }

    fn register_signal_handlers(&self) {
        let tx_int = self.event_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            {
                if sig.recv().await.is_some() {
                    let _ = tx_int.send(AppEvent::Shutdown);
                }
            }
        });

        let tx_term = self.event_tx.clone();
        tokio::spawn(async move {
            if let Ok(mut sig) =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            {
                if sig.recv().await.is_some() {
                    let _ = tx_term.send(AppEvent::Shutdown);
                }
            }
        });
    }

    async fn run_event_loop(&mut self) {
        loop {
            tokio::select! {
                input_event = self.input_rx.recv() => {
                    match input_event {
                        Some(input) => self.handle_input(input),
                        None => break,
                    }
                }
                app_event = self.event_rx.recv() => {
                    match app_event {
                        Some(AppEvent::Shutdown) => {
                            self.handle_shutdown();
                            break;
                        }
                        Some(AppEvent::TranscriptionComplete(text)) => {
                            if !text.trim().is_empty() {
                                if let Err(e) = self.output.paste(&text) {
                                    tracing::error!("paste failed: {e}");
                                } else {
                                    tracing::info!("pasted transcription");
                                }
                            }
                            self.state = AppState::Idle;
                        }
                        Some(AppEvent::Input(_)) => {}
                        None => break,
                    }
                }
                () = self.cancel.cancelled() => {
                    self.handle_shutdown();
                    break;
                }
            }
        }
    }

    fn handle_input(&mut self, event: InputEvent) {
        match self.mode {
            InteractionMode::Hold => self.handle_hold(event),
            InteractionMode::Toggle => self.handle_toggle(event),
        }
    }

    fn handle_hold(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyDown => {
                if let AppState::Idle = self.state {
                    if let Err(e) = self.recorder.start() {
                        tracing::error!("failed to start recording: {e}");
                        return;
                    }
                    self.state = AppState::Recording;
                    tracing::info!("recording started");
                }
            }
            InputEvent::KeyUp => {
                if let AppState::Recording = self.state {
                    self.stop_and_transcribe();
                }
            }
        }
    }

    fn handle_toggle(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyDown => match self.state {
                AppState::Idle => {
                    if let Err(e) = self.recorder.start() {
                        tracing::error!("failed to start recording: {e}");
                        return;
                    }
                    self.state = AppState::Recording;
                    tracing::info!("recording started");
                }
                AppState::Recording => {
                    self.stop_and_transcribe();
                }
                AppState::Transcribing => {}
            },
            InputEvent::KeyUp => {}
        }
    }

    fn stop_and_transcribe(&mut self) {
        match self.recorder.stop() {
            Ok(samples) => {
                tracing::info!("recording stopped");
                self.state = AppState::Transcribing;
                let engine = Arc::clone(&self.engine);
                let tx = self.event_tx.clone();
                tokio::task::spawn_blocking(move || match engine.transcribe(samples) {
                    Ok(text) => {
                        tracing::info!("transcription: {text}");
                        let _ = tx.send(AppEvent::TranscriptionComplete(text));
                    }
                    Err(e) => {
                        tracing::error!("transcription failed: {e}");
                        let _ = tx.send(AppEvent::TranscriptionComplete(String::new()));
                    }
                });
            }
            Err(e) => {
                tracing::error!("failed to stop recording: {e}");
                self.state = AppState::Idle;
            }
        }
    }

    fn handle_shutdown(&mut self) {
        tracing::info!("shutting down");
        self.cancel.cancel();
    }
}
