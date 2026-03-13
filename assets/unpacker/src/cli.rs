use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "unpacker", about = "Arknights Unity AssetBundle unpacker")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Extract assets from bundle files
    Extract(ExtractArgs),
    /// List objects in a bundle file
    List(ListArgs),
}

#[derive(Parser)]
pub struct ExtractArgs {
    /// Input directory containing bundle files
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output directory for extracted assets
    #[arg(short, long)]
    pub output: PathBuf,

    /// Extract textures only
    #[arg(long)]
    pub image: bool,

    /// Extract text assets only
    #[arg(long)]
    pub text: bool,

    /// Extract audio only
    #[arg(long)]
    pub audio: bool,

    /// Extract gamedata (requires --idx)
    #[arg(long)]
    pub gamedata: bool,

    /// Extract spine animations (organized into BattleFront/BattleBack/Building/DynIllust)
    #[arg(long)]
    pub spine: bool,

    /// Path to resource manifest .idx file (for gamedata extraction)
    #[arg(long)]
    pub idx: Option<PathBuf>,

    /// Disable automatic alpha texture merging (export raw textures as-is)
    #[arg(long)]
    pub no_merge: bool,

    /// Number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
}

impl ExtractArgs {
    /// Returns true if no type filters are set (extract everything)
    pub fn extract_all(&self) -> bool {
        !self.image && !self.text && !self.audio && !self.gamedata && !self.spine
    }
}

#[derive(Parser)]
pub struct ListArgs {
    /// Input bundle file
    #[arg(short, long)]
    pub input: PathBuf,
}
