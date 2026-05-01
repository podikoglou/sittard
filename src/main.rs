use anyhow::Result;
use clap::Parser;
use staid::config::{Cli, Commands};

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
    tracing_subscriber::fmt().with_max_level(level).init();
}

fn main() -> Result<()> {
    std::env::set_var("PIPEWIRE_LOG_LEVEL", "0");
    std::env::set_var("JACK_NO_START_SERVER", "1");
    std::env::set_var("JACK_NO_AUDIO_RESERVATION", "1");

    let cli = Cli::parse();
    init_tracing(&cli);

    match &cli.command {
        Some(Commands::ListDevices) => {
            println!("Listing audio devices... (not yet implemented)");
            return Ok(());
        }
        Some(Commands::ListKeys) => {
            println!("Listing key names... (not yet implemented)");
            return Ok(());
        }
        Some(Commands::DownloadModel { model }) => {
            println!("Downloading model... (not yet implemented)");
            return Ok(());
        }
        None => {}
    }

    tracing::info!("staid starting");

    Ok(())
}
