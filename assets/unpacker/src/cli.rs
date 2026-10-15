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
    /// Rewrite only the `avg/characters` PNGs whose alpha the hub's
    /// per-sprite `alphaTex` pairs differently from the `foo[alpha]` name rule
    BackfillAvgFaces(BackfillAvgFacesArgs),
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
    /// Write an AAC `.m4a` beside every in-game music `.ogg` in an existing
    /// output tree (the same step `extract` runs after the audio export)
    BackfillMusic(BackfillMusicArgs),
    /// Resolve which `FlatBuffers` schema every gamedata table needs from the
    /// live bundles, the generated CN schemas and the vendored history, and
    /// write it into the schema store (`assets/schemas`)
    SchemaSync(SchemaSyncArgs),
    /// Regenerate the gamedata `.fbs` schemas from the CN Windows client
    /// (`GameAssembly.dll` + `global-metadata.dat`, from `downloader -s cn
    /// client-download`)
    SchemaGen(SchemaGenArgs),
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

/// `unpacker backfill-avg-faces` — the AVG character alpha merge on its own.
/// It decodes every `avg/characters` bundle, pairs each colour texture with
/// the alpha its hub sprite's `alphaTex` names (the name rule where a sprite
/// carries none), and rewrites ONLY the PNGs whose pairing differs from what
/// the name rule gave, so a tree extracted before 2026-09-29 gains the
/// transparent faces without a multi-hour re-extract and nothing else in it is
/// touched.
#[derive(Parser)]
pub struct BackfillAvgFacesArgs {
    /// `ArkAssets` root for one server (the directory holding `avg/`)
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output tree for that server (the directory holding `textures/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Report the pairing census and write nothing
    #[arg(long)]
    pub dry_run: bool,

    /// Write the re-paired textures, one `<folder>/<colour>\t<alpha>` per
    /// line, to this file
    #[arg(long)]
    pub list: Option<PathBuf>,

    /// Rewrite EVERY texture of every bundle (the extract's texture pass for
    /// these bundles), not only the re-paired ones: for an A/B into an
    /// alternate output root
    #[arg(long)]
    pub all: bool,

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

/// `unpacker backfill-music`: the music step of an extract on its own. It has
/// `ffmpeg` write `<clip>.m4a` (AAC-LC 192k, source rate, `+faststart`) beside
/// every `.ogg` under `audio/audio/sound_beta_2/music/` and every `m_*.ogg`
/// under `audio/anon/`, for browsers without a Vorbis decoder. A clip whose
/// `.m4a` is newer than its `.ogg`, or whose `.ogg` matches the fingerprint in
/// `audio/m4a_sources.json`, is skipped. Exits 1 when `ffmpeg` is missing or a
/// clip fails, where `extract` only warns.
#[derive(Parser)]
pub struct BackfillMusicArgs {
    /// Output tree for one server (the directory holding `audio/`)
    #[arg(short, long)]
    pub output: PathBuf,

    /// Transcode every clip even when its `.m4a` is current
    #[arg(long)]
    pub force: bool,
}

/// `unpacker schema-sync`. The contract (candidate order, scoring, lock
/// format) is `assets/schemas/README.md`. Exit codes: 0 resolved; 2 a table
/// that verified in the previous lock now verifies under no candidate on a
/// server where it did; 1 error.
#[derive(Parser)]
pub struct SchemaSyncArgs {
    /// `unpacker schema-gen` output (`<stem>.fbs`, `client.json`,
    /// `schema-gen.json`): the CN candidates. Without it CN is the stored `cn/`
    #[arg(long)]
    pub generated: Option<PathBuf>,

    /// The schema store (default: `assets/schemas` beside this crate)
    #[arg(long)]
    pub store: Option<PathBuf>,

    /// `<server>=<dir>` gamedata-profile download (anon/*.ab + .idx); repeat
    /// per server. `cn` is the CN side, every other name a Yostar server
    #[arg(long = "bundles", value_parser = parse_bundles)]
    pub bundles: Vec<(String, PathBuf)>,

    /// Write the before/after matrix as Markdown here
    #[arg(long)]
    pub report: Option<PathBuf>,

    /// How many buffers of a directory-routed table (`prts___levels`, the first
    /// N in sorted path order) must verify
    #[arg(long, default_value_t = 20)]
    pub levels_sample: usize,

    /// Binary-schema cache (default: `unpacker-bfbs-cache` in the temp dir)
    #[arg(long)]
    pub cache: Option<PathBuf>,

    /// Instead of syncing, verify every gamedata buffer through the generated
    /// decoder AND the reflection verifier under the store's current schemas,
    /// and print where they disagree (exit 1 on any disagreement)
    #[arg(long)]
    pub self_check: bool,

    /// Compare the resulting Yostar choice with another tree of `.fbs` files
    /// (a measurement aid)
    #[arg(long, hide = true)]
    pub compare_dir: Option<PathBuf>,
}

fn parse_bundles(s: &str) -> Result<(String, PathBuf), String> {
    let (server, dir) = s
        .split_once('=')
        .ok_or_else(|| format!("expected <server>=<dir>, got `{s}`"))?;
    if server.is_empty() || dir.is_empty() {
        return Err(format!("expected <server>=<dir>, got `{s}`"));
    }
    Ok((server.to_lowercase(), PathBuf::from(dir)))
}

/// `unpacker schema-gen`; see `unpacker::schema::gen`. Exit codes: 0 success;
/// 1 error; 2 `--compare` found a file neither IDENTICAL nor in
/// `--allow-diff`; 3 the client's binary layout is not what the reader knows
/// (a human must look).
#[derive(Parser)]
pub struct SchemaGenArgs {
    /// Directory holding `GameAssembly.dll`, `global-metadata.dat` and
    /// `client.json`
    #[arg(long)]
    pub client: PathBuf,

    /// Where to write `<stem>.fbs`, `client.json` and `schema-gen.json`
    #[arg(long, required_unless_present = "discover")]
    pub out: Option<PathBuf>,

    /// Root map (default: `assets/schemas/roots.json` beside this crate)
    #[arg(long)]
    pub roots: Option<PathBuf>,

    /// Compare the output with every `.fbs` in this directory (comments and
    /// whitespace ignored) and print IDENTICAL / DIFF / MISSING per file
    #[arg(long)]
    pub compare: Option<PathBuf>,

    /// Comma-separated stems allowed to be DIFF or MISSING under `--compare`
    #[arg(long, value_delimiter = ',')]
    pub allow_diff: Vec<String>,

    /// Only list the client's `ConstTable` / `SimpleKVTable` classes whose
    /// root the root map does not name, as JSON, and exit
    #[arg(long)]
    pub discover: bool,
}
