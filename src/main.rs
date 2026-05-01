use anyhow::Result;
use clap::Parser;
use staid::config::{Cli, Commands};
use staid::model::huggingface::HuggingFaceProvider;
use staid::model::ModelProvider;

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
    tracing_subscriber::fmt().with_max_level(level).try_init().ok();
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("PIPEWIRE_LOG_LEVEL", "0");
    std::env::set_var("JACK_NO_START_SERVER", "1");
    std::env::set_var("JACK_NO_AUDIO_RESERVATION", "1");

    let cli = Cli::parse();
    init_tracing(&cli);

    match &cli.command {
        Some(Commands::ListDevices) => {
            println!("Listing audio devices... (not yet implemented)");
        }
        Some(Commands::ListKeys) => {
            println!("Listing key names... (not yet implemented)");
        }
        Some(Commands::DownloadModel { model }) => {
            let model_size = model.clone().or_else(|| cli.model.clone())
                .unwrap_or(staid::config::ModelSize::BaseEn);
            let provider = HuggingFaceProvider::new(model_size);
            let path = provider.ensure_model().await?;
            println!("model downloaded to {}", path.display());
        }
        None => {
            tracing::info!("staid starting");

            let model_size = cli.model.clone().unwrap_or(staid::config::ModelSize::BaseEn);
            let provider = HuggingFaceProvider::new(model_size);
            let model_path = provider.ensure_model().await?;
            tracing::info!("using model at {}", model_path.display());
        }
    }

    Ok(())
}
