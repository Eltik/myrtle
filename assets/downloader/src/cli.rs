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
        /// Content profile(s), comma-separated & OR-combined: "operators"
        /// (operator assets), "stages" (stage-viewer level scenes + preview &
        /// banner art), "gamedata" (the anon/ bundles + .idx manifests, i.e.
        /// exactly what `unpacker extract --gamedata` reads), "release"
        /// (event / banner / skin-brand art for the Release Planner), or
        /// "full" (everything). Omit for full.
        #[arg(long)]
        profile: Option<String>,
        /// Keep only bundles whose name starts with one of these comma-separated
        /// prefixes (applied after `--profile`). `.idx` manifests are always kept.
        /// Example: `--include "spritepack/ui_home_act_banner_,arts/ui/stage/[uc]homeentry"`.
        #[arg(long)]
        include: Option<String>,
    },
    /// Check if server has updates
    CheckUpdate,
    /// List available packs
    ListPacks,
    /// Extract the IL2CPP client binary + global-metadata.dat from a user-provided
    /// app package (APK / XAPK / APKM / OBB / IPA) and report whether a static
    /// dump is viable. Runs fully offline (no server contact).
    ClientExtract {
        /// Path to the app package (.apk/.xapk/.apkm/.obb/.ipa).
        #[arg(short, long)]
        input: PathBuf,

        /// Directory to write libil2cpp.so / the Mach-O + global-metadata.dat into.
        #[arg(short, long, default_value = "./il2cpp-input")]
        output: PathBuf,
    },
}
