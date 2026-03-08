use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "downloader", about = "Arknights asset downloader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Server to download from
    #[arg(short, long, default_value = "en", global = true)]
    pub server: String,

    /// Output directory
    #[arg(short = 'd', long, default_value = "./ArkAssets", global = true)]
    pub savedir: PathBuf,

    /// Concurrent download limit
    #[arg(short = 't', long, default_value = "6", global = true)]
    pub threads: usize,

    /// Enable verbose logging
    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download assets
    Download {
        /// Download all packs
        #[arg(long)]
        all: bool,
        /// Comma-separated pack names
        #[arg(short, long)]
        packages: Option<String>,
    },
    /// Check if server has updates
    CheckUpdate,
    /// List available packs
    ListPacks,
}
