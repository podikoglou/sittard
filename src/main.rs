use anyhow::Result;
use clap::Parser;
use sittard::app::SittardApp;
use sittard::audio::cpal_recorder::CpalRecorder;
use sittard::config::{AppConfig, Cli, Commands, Output};
use sittard::input::evdev_listener::EvdevListener;
use sittard::model::huggingface::SherpaOnnxProvider;
use sittard::model::ModelProvider;
use sittard::output::{wayland_output::WaylandOutput, wtype_output::WtypeOutput, TextOutput};
use sittard::transcribe::whisper_engine::SherpaOnnxEngine;
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
            let devices = sittard::audio::list_devices()?;
            if devices.is_empty() {
                println!("no audio input devices found");
            } else {
                for name in &devices {
                    println!("{name}");
                }
            }
        }
        Some(Commands::ListKeys) => {
            let keys = sittard::input::keymap::list_key_names();
            for name in &keys {
                println!("{name}");
            }
            println!();
            println!("modifier aliases (match left or right):");
            let aliases = sittard::input::keymap::list_modifier_aliases();
            for name in &aliases {
                println!("  {name}");
            }
            println!();
            println!("combine with + (e.g. --hotkey \"ctrl+shift+f12\")");
        }
        Some(Commands::DownloadModel { engine }) => {
            let engine = engine.unwrap_or(sittard::config::ModelEngine::Parakeet);
            let provider = SherpaOnnxProvider::new(engine);
            let path = provider.ensure_model().await?;
            println!("model downloaded to {}", path.display());
        }
        None => {
            tracing::info!("sittard starting");

            let config = AppConfig::from_cli(cli);

            let provider = SherpaOnnxProvider::new(config.engine);
            let model_path = provider.ensure_model().await?;
            tracing::info!("using model at {}", model_path.display());

            let recorder = CpalRecorder::new(config.device.as_deref())?;
            let listener = EvdevListener::new(&config.hotkey)?;
            let engine = SherpaOnnxEngine::new(&model_path, config.engine, config.threads)?;
            let output: Box<dyn TextOutput> = match config.output {
                Output::Wayland => Box::new(WaylandOutput::new()?),
                Output::Wtype => Box::new(WtypeOutput::new()?),
            };

            let cancel = CancellationToken::new();

            let mut app = SittardApp::new(
                Box::new(recorder),
                Box::new(listener),
                Arc::new(engine),
                output,
                cancel,
                config.mode,
            );

            app.start().await?;
        }
    }

    Ok(())
}
