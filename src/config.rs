use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Clone, Default, ValueEnum)]
pub enum InteractionMode {
    #[default]
    Hold,
    Toggle,
}

#[derive(Clone, Default, ValueEnum)]
pub enum Output {
    #[default]
    Clipboard,
    Wtype,
}

#[derive(Parser)]
#[command(name = "sittard", about = "Voice-to-text daemon", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, short, value_enum, global = true)]
    pub engine: Option<ModelEngine>,

    #[arg(long, short, global = true)]
    pub device: Option<String>,

    #[arg(long, short, default_value = "en", global = true)]
    pub language: String,

    #[arg(long, default_value = "right_alt", global = true)]
    pub hotkey: String,

    #[arg(long, default_value = "hold", value_enum, global = true)]
    pub mode: InteractionMode,

    #[arg(long, short, default_value_t = num_cpus::get(), global = true)]
    pub threads: usize,

    #[arg(long, short, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[arg(long, default_value = "clipboard", value_enum, global = true)]
    pub output: Output,

    #[arg(long, short = 'D', global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    ListDevices,
    ListKeys,
    DownloadModel {
        #[arg(long, short, value_enum)]
        engine: Option<ModelEngine>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ModelEngine {
    Parakeet,
    Moonshine,
    WhisperTiny,
    WhisperBase,
}

impl fmt::Display for ModelEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelEngine::Parakeet => write!(f, "parakeet"),
            ModelEngine::Moonshine => write!(f, "moonshine"),
            ModelEngine::WhisperTiny => write!(f, "whisper-tiny"),
            ModelEngine::WhisperBase => write!(f, "whisper-base"),
        }
    }
}

pub struct AppConfig {
    pub hotkey: String,
    pub engine: ModelEngine,
    pub device: Option<String>,
    pub language: String,
    pub threads: usize,
    pub mode: InteractionMode,
    pub output: Output,
    pub verbose: u8,
    pub debug: bool,
}

impl AppConfig {
    #[must_use]
    pub fn from_cli(cli: Cli) -> Self {
        let engine = cli.engine.unwrap_or(ModelEngine::Parakeet);
        Self {
            hotkey: cli.hotkey,
            engine,
            device: cli.device,
            language: cli.language,
            threads: cli.threads,
            mode: cli.mode,
            output: cli.output,
            verbose: cli.verbose,
            debug: cli.debug,
        }
    }
}
