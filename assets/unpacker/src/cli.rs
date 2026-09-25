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
    /// Report which `FlatBuffer` schema each gamedata table verifies against
    Verify(VerifyArgs),
    /// Write only the `avg/characters` sprite-hub `hub.json` files into an
    /// existing output tree, without re-extracting any texture
    BackfillHubs(BackfillHubsArgs),
    /// Write only the story image trees' `sprites.json` files (rect, pixels
    /// per unit and pivot) into an existing output tree
    BackfillSprites(BackfillSpritesArgs),
    /// Write only the Story Collection's art (`spritepack/mixstory_*`: the
    /// chapter/event key visuals, titles, shelf glyphs, arc icons and chapter
    /// decos) into an existing output tree
    BackfillStoryArt(BackfillStoryArtArgs),
    /// Transcode the `raw/video/**/*.usm` cutscenes into `video/<rel>.webm`
    /// and `.mp4` in an existing output tree (the same step `extract` runs
    /// last)
    BackfillVideo(BackfillVideoArgs),
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

    /// Extract portraits from `SpritePacker` atlases (charportraits)
    #[arg(long)]
    pub portrait: bool,

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
    pub const fn extract_all(&self) -> bool {
        !self.image && !self.text && !self.audio && !self.gamedata && !self.spine && !self.portrait
    }
}

#[derive(Parser)]
pub struct ListArgs {
    /// Input bundle file
    #[arg(short, long)]
    pub input: PathBuf,
}

/// `unpacker verify` — verification only. No decode, no output files, always
/// exits 0: it is a report, not a gate.
#[derive(Parser)]
pub struct VerifyArgs {
    /// Input directory containing bundle files
    #[arg(short, long)]
    pub input: PathBuf,

    /// Path to resource manifest .idx file (auto-detected in the input dir)
    #[arg(long)]
    pub idx: Option<PathBuf>,
}

/// `unpacker backfill-hubs` — the hub half of a texture extract on its own.
/// It writes `hub.json` into sprite folders that already hold their PNGs and
/// touches nothing else, so an output tree can gain face placement without a
/// multi-hour re-extract.
#[derive(Parser)]
pub struct BackfillHubsArgs {
    /// `ArkAssets` root for one server (the directory holding `avg/`)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output tree for that server (the directory holding `textures/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Also write into sprite folders that do not exist yet (default: skip
    /// them, so the walk never invents a folder with no PNGs in it)
    #[arg(long)]
    pub create_missing: bool,

    /// Number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
}

/// `unpacker backfill-sprites` — the `sprites.json` half of a texture extract
/// on its own. It writes the per-sprite rect, pixels-per-unit and pivot into
/// the `avg/bg`, `avg/imgs`, `avg/items`, `avg/backgrounds` and
/// `spritepack/cutin_char_*` folders that already hold their PNGs, and touches
/// nothing else: an output tree gains plate sizing without a multi-hour
/// re-extract.
#[derive(Parser)]
pub struct BackfillSpritesArgs {
    /// `ArkAssets` root for one server (the directory holding `avg/`)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output tree for that server (the directory holding `textures/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Also write into folders that do not exist yet (default: skip them, so
    /// the walk never invents a folder with no PNGs in it)
    #[arg(long)]
    pub create_missing: bool,

    /// Number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
}

/// `unpacker backfill-story-art` — the Story Collection half of a texture
/// extract on its own. It writes the `spritepack/mixstory_*` PNGs (key
/// visuals, title logotypes, shelf glyphs and logos, arc icons, chapter
/// decos, shelf backgrounds) into an existing output tree and touches nothing
/// else, so `/stories` can gain its banners and icons without a multi-hour
/// re-extract. Every one of these is a plain `Texture2D` + full-rect `Sprite`
/// pair, so nothing is cut out of an atlas.
#[derive(Parser)]
pub struct BackfillStoryArtArgs {
    /// `ArkAssets` root for one server (the directory holding `spritepack/`)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output tree for that server (the directory holding `textures/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Disable automatic alpha texture merging (export raw textures as-is)
    #[arg(long)]
    pub no_merge: bool,

    /// Number of parallel threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs")]
    pub jobs: Option<usize>,
}

/// `unpacker backfill-video` — the video step of an extract on its own. It
/// demuxes every `raw/video/**/*.usm` and has `ffmpeg` write
/// `video/<rel>.webm` (VP9 copied, Opus) and `video/<rel>.mp4` (x264, AAC),
/// one clip at a time, skipping a clip whose two outputs are both newer than
/// its `.usm`. Exits 1 when `ffmpeg` is missing or a clip fails, where
/// `extract` only warns.
#[derive(Parser)]
pub struct BackfillVideoArgs {
    /// `ArkAssets` root for one server (the directory holding `raw/video/`)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output tree for that server (the directory that gains `video/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Transcode every clip even when its outputs are newer than the `.usm`
    #[arg(long)]
    pub force: bool,
}
