use clap::{Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Parser)]
#[command(name = "staid", about = "Voice-to-text daemon", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long, short, value_enum, global = true)]
    pub model: Option<ModelSize>,

    #[arg(long, short, global = true)]
    pub device: Option<String>,

    #[arg(long, short, default_value = "en", global = true)]
    pub language: String,

    #[arg(long, default_value = "right_alt", global = true)]
    pub hotkey: String,

    #[arg(long, short, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[arg(long, short = 'D', global = true)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    ListDevices,
    ListKeys,
    DownloadModel {
        #[arg(long, short, value_enum)]
        model: Option<ModelSize>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ModelSize {
    TinyEn,
    BaseEn,
    SmallEn,
}

impl fmt::Display for ModelSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelSize::TinyEn => write!(f, "tiny.en"),
            ModelSize::BaseEn => write!(f, "base.en"),
            ModelSize::SmallEn => write!(f, "small.en"),
        }
    }
}

pub struct AppConfig {
    pub hotkey: String,
    pub model_size: ModelSize,
    pub device: Option<String>,
    pub language: String,
    pub threads: usize,
    pub verbose: u8,
    pub debug: bool,
}

impl AppConfig {
    #[must_use]
    pub fn from_cli(cli: Cli) -> Self {
        let model_size = cli.model.unwrap_or(ModelSize::BaseEn);
        Self {
            hotkey: cli.hotkey,
            model_size,
            device: cli.device,
            language: cli.language,
            threads: num_cpus::get(),
            verbose: cli.verbose,
            debug: cli.debug,
        }
    }
}
