use anyhow::Result;
use clap::Parser;
use staid::app::StaidApp;
use staid::audio::cpal_recorder::CpalRecorder;
use staid::config::{AppConfig, Cli, Commands};
use staid::input::evdev_listener::EvdevListener;
use staid::model::huggingface::HuggingFaceProvider;
use staid::model::ModelProvider;
use staid::output::wtype_output::WtypeOutput;
use staid::transcribe::whisper_engine::WhisperEngine;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

fn init_tracing(cli: &Cli) {
    let level = if cli.debug {
        tracing::Level::TRACE
    } else {
        match cli.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            _ => tracing::Level::DEBUG,
        }
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .try_init()
        .ok();
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("PIPEWIRE_LOG_LEVEL", "0");
    std::env::set_var("JACK_NO_START_SERVER", "1");
    std::env::set_var("JACK_NO_AUDIO_RESERVATION", "1");

    let cli = Cli::parse();
    init_tracing(&cli);

    match cli.command {
        Some(Commands::ListDevices) => {
            let devices = staid::audio::list_devices()?;
            if devices.is_empty() {
                println!("no audio input devices found");
            } else {
                for name in &devices {
                    println!("{name}");
                }
            }
        }
        Some(Commands::ListKeys) => {
            let keys = staid::input::keymap::list_key_names();
            for name in &keys {
                println!("{name}");
            }
            println!();
            println!("modifier aliases (match left or right):");
            let aliases = staid::input::keymap::list_modifier_aliases();
            for name in &aliases {
                println!("  {name}");
            }
            println!();
            println!("combine with + (e.g. --hotkey \"ctrl+shift+f12\")");
        }
        Some(Commands::DownloadModel { model }) => {
            let model_size = model.unwrap_or(staid::config::ModelSize::BaseEn);
            let provider = HuggingFaceProvider::new(model_size);
            let path = provider.ensure_model().await?;
            println!("model downloaded to {}", path.display());
        }
        None => {
            tracing::info!("staid starting");

            let config = AppConfig::from_cli(cli);

            let model_size = config.model_size.clone();
            let provider = HuggingFaceProvider::new(model_size);
            let model_path = provider.ensure_model().await?;
            tracing::info!("using model at {}", model_path.display());

            let recorder = CpalRecorder::new(config.device.as_deref())?;
            let listener = EvdevListener::new(&config.hotkey)?;
            let engine = WhisperEngine::new(&model_path, config.threads)?;
            let output = WtypeOutput::new()?;

            let cancel = CancellationToken::new();

            let mut app = StaidApp::new(
                Box::new(recorder),
                Box::new(listener),
                Arc::new(engine),
                Box::new(output),
                cancel,
                config.mode,
            );

            app.start().await?;
        }
    }

    Ok(())
}
