# Arknights Asset Pipeline

Complete pipeline for downloading, unpacking, and extracting Arknights game assets from CDN to organized output formats (images, text, audio, animations, gamedata).

## Overview

This monorepo contains two Rust crates coordinated by a Node.js runner:

```
Internet (CDN)
    ↓ download via Downloader crate
Asset bundles (.ab, .dat, .bin files)
    ↓ unpack via Unpacker crate
    ├── textures/ (PNG images)
    ├── text/ (JSON, gamedata Excel)
    ├── audio/ (OGG, WAV, M4A)
    ├── portraits/ (PNG, sliced from atlases)
    ├── gamedata/ (excel/, levels/, story/, battle/, building/)
    └── spine/ (BattleFront, BattleBack, Building, DynIllust, Enemy)
```

In production the runner, Docker, and Compose paths all nest output by region, so
the real tree is `output/<region>/textures/…`. The flat layout above is what you get
invoking the binaries directly with `-o ./output`.

**Downloader** (`downloader/`):
- Fetches version metadata from CDN
- Downloads asset bundles incrementally (MD5-based deduplication)
- Handles hot-update pack listing
- Supports multiple server regions (CN Official, CN Bilibili, Global/EN, Japan, Korea, Taiwan)
- Uses AES-CBC decryption for encrypted transfers
- Parallel downloads with configurable concurrency (default 6)

**Unpacker** (`unpacker/`):
- Parses UnityFS bundle format (header + block info + data blocks)
- Decompresses blocks (LZMA, LZ4, LZ4AK custom variant)
- Deserializes Unity serialized files using type trees
- Exports assets by class ID (Texture2D, TextAsset, AudioClip, MonoBehaviour for Spine)
- Composites `foo` / `foo[alpha]` texture pairs into single RGBA PNGs
- Slices operator portraits out of shared `SpritePacker` atlases
- Un-squashes stage map previews, which ship as forced 512x512 squares
- Reconstructs dynamic-illustration (L2D) scenes: particles, meshes, shader maps, camera curves
- Multi-threaded extraction with rayon (configurable thread pool)
- FlatBuffer schema support for gamedata Excel files

**Runner** (`run.mjs`):
- Interactive CLI for 3 workflows: setup, incremental update, WebSocket server
- Prerequisite checking (Git, Rust 1.85.0+, C compiler)
- Binary build orchestration
- Prunes orphaned bundles and reconciles the manifest after every download
- Detects truncated extracts and retries an OOM-killed unpack single-threaded
- Progress tracking with indicatif spinners

Two more tools sit alongside the pipeline: `il2cpp-locate/` (a Rust crate for locating
methods in an IL2CPP dump) and `il2cpp-recover.sh`. Both exist to recover *compiled
algorithms* that are not present in any asset bundle. See
[`IL2CPP_RECOVERY.md`](IL2CPP_RECOVERY.md). Normal download and extract never needs them.

---

## Prerequisites

| Tool | Version | Installation |
|------|---------|---|
| **Node.js** | 18+ | Download from https://nodejs.org |
| **Git** | Any | `brew install git` (macOS) / `apt install git` (Linux) / https://git-scm.com (Windows) |
| **Rust (rustc)** | 1.85.0+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Cargo** | 1.76+ | Installed with Rust |
| **C Compiler** | Any | `xcode-select --install` (macOS) / `apt install build-essential` (Linux) / Visual Studio Build Tools (Windows) |
| **flatc** | 25.12.19 | _Only_ needed to regenerate FlatBuffer schemas (`generate-fbs`), not for normal download/extract. `brew install flatbuffers` (macOS). Must match the `flatbuffers` crate version pinned in `unpacker/Cargo.toml`. |

The pipeline requires Edition 2024 Rust for async/await and newer language features.

---

## Quick Start

### 1. Clone & Setup

```bash
git clone <repo> assets
cd assets
npm install
node run.mjs
```

Follow the interactive prompts to:
- Check prerequisites
- Build `downloader` and `unpacker` binaries (release mode)

### 2. Download Assets

```bash
./binaries/downloader --server en download --all
```

Downloads all asset packs for the Global/EN server to `./ArkAssets/`.

### 3. Extract Assets

```bash
./binaries/unpacker extract -i ./ArkAssets -o ./output
```

Extracts all asset types to `./output/`.

---

## Interactive Runner (`run.mjs`)

The runner is the primary interface for most workflows. It handles prerequisite checks, builds, and coordinate download/unpack cycles.

### Mode 1: Setup

```bash
npm start
# → Select "Setup (build binaries)"
```

**What it does:**
- Verifies Git, Rust 1.85.0+, Cargo, C compiler
- Prompts to proceed with build
- Compiles `downloader` crate (release)
- Compiles `unpacker` crate (release)
- Copies built binaries to `./binaries/` (gitignored)
- Reports binary paths and quick-start commands

**Output:**
```
Downloader: ./binaries/downloader
Unpacker:   ./binaries/unpacker
```

### Mode 2: Check for Updates & Update

```bash
npm start
# → Select "Check for Updates & Update"
```

**Workflow:**
1. Prompt for server region, asset directory, output directory, content profile (`full`, `operators`, `stages`, `gamedata`, `release`)
2. Fetch server version info (client + resource version)
3. Compare with local `.version` file in asset directory
4. If newer: download + extract + save new version

> **Region nesting:** the runner appends the region key to both directories, so a
> `cn` run saves to `./ArkAssets/cn` and extracts to `./output/cn`. This lets
> multiple regions coexist (e.g. full `en` alongside an `operators`-profile `cn`
> preview). The raw `downloader`/`unpacker` binaries do **not** nest - pass
> already-region-scoped paths to them directly.

**Version File:** `./ArkAssets/<region>/.version` (single-line resource version string)

**Output:**
- Download progress: files/total, percentage
- Extraction progress: bundles/total, percentage
- Final stats: downloaded count, failed count, total bytes

### Mode 3: WebSocket Server (Continuous Monitoring)

```bash
npm start
# → Select "WebSocket Server (continuous updates)"
```

**Configuration:**
- Server region
- Asset download directory (nested per region - see Mode 2)
- Extraction output directory (nested per region - see Mode 2)
- Content profile (`full`, `operators`, `stages`, `gamedata`, `release`) - also via `--profile` / `WS_PROFILE`
- WebSocket port (default 9160)
- Check interval in minutes (default 30)

> Run one instance per region (separate ports/dirs) to monitor several regions at
> once - e.g. a full `en` server on 9160 and an `operators`-profile `cn` preview on 9161.

**Behavior:**
- Starts listening on `ws://localhost:<port>`
- Periodically checks server for updates
- On update available: downloads and extracts automatically
- Broadcasts status/progress to all connected clients
- Graceful shutdown on Ctrl+C

---

## Downloader CLI Reference

**Binary:** `./binaries/downloader`

### Global Flags

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|---|
| `--server` | `-s` | string | `en` | Server region |
| `--savedir` | `-d` | path | `./ArkAssets` | Directory to save bundles |
| `--threads` | `-t` | number | `6` | Concurrent download limit |
| `--verbose` | | bool | false | Debug logging |

### Commands

#### `check-update`

Check server version without downloading.

```bash
downloader --server en check-update
```

Output:
```
Client: 1.27.5  Resources: 1.27.5_08_15_11_51
```

#### `list-packs`

List all available asset packs with sizes.

```bash
downloader --server en list-packs
```

Output:
```
pack/audio/sound_cn               1.2 GB  (50 files)
pack/chararts                     850 MB  (120 files)
...
```

#### `download`

Download asset packs.

**Options:**

| Option | Type | Description |
|--------|------|---|
| `--all` | bool | Download all packs (required unless `--packages` used) |
| `--packages <list>` | string | Comma-separated pack names (e.g., `pack/chararts,pack/ui`) |
| `--profile <name>` | string | Content profile(s) applied on top of `--all`: `full`, `operators`, `stages`, `gamedata`, or `release`. Comma-separate to combine (e.g. `operators,stages`). Omit for `full`. |

**Examples:**

```bash
# Download all packs
downloader --server en download --all

# Download specific packs
downloader --server en download --packages pack/chararts,pack/ui

# 8 concurrent downloads to custom directory
downloader --server cn --threads 8 -d /data/arknights download --all

# Operator-only profile: gamedata + operator art/portraits/avatars/icons.
# Used for the "upcoming operators" preview.
downloader --server cn download --all --profile operators

# Gamedata only: the smallest useful download, ~330 MB on CN
downloader --server cn download --all --profile gamedata

# Combine profiles
downloader --server cn download --all --profile operators,stages
```

### Content Profiles

A profile filters the hot-update file list (by `abInfo` name) down to a subset, keeping
the download small when full-region storage is impractical. Five profiles exist, defined
in `downloader/src/profile.rs`, and they can be OR-combined with a comma-separated list.
Every profile retains `.idx` files unconditionally, so the gamedata manifest is never lost.

| Profile | Contents | Approx. size (CN) |
|---------|----------|-------------------|
| `full` | Everything. The default when `--profile` is omitted | ~90 GB |
| `operators` | Gamedata plus operator-facing art: `chararts/`, `skinpack/`, portraits, avatars, skill and equip icons, elite/potential hubs, camp logos, and player/battle/voice audio | ~2.5 GB |
| `stages` | Stage-viewer level scenes, map-preview thumbnails, and zone/event/IS banner art. Deliberately disjoint from `operators` | - |
| `gamedata` | Only `anon/` bundles plus the `.idx` manifest. Exactly what `unpacker extract --gamedata` consumes | ~330 MB, ~150 files |
| `release` | Release Planner art: gacha banner strips (`ui_home_act_banner_*`), event home-entry key art (`[uc]homeentry`, `ui_zone_home_theme_*`), per-event UI kits (`activity/`), skin shop portraits, brand key visuals and logos. The CN watcher runs `operators,release` | - |

Sizes are order-of-magnitude guidance from `profile.rs`, not guarantees; they grow with
every game update. For reference, a current full extract runs roughly 18-20 GB of
downloaded bundles and 35-38 GB of output per region.

### Server Regions

| Key | Aliases | Label | CDN |
|-----|---------|-------|-----|
| `en` | `global` | Global/EN (Yostar) | `ark-us-static-online.yo-star.com` |
| `jp` | `japan` | Japan (Yostar) | `ark-jp-static-online.yo-star.com` |
| `kr` | `korea` | Korea (Yostar) | `ark-kr-static-online.yo-star.com` |
| `tw` | `taiwan` | Taiwan (Gryphline) | `ark-tw-static-online.yo-star.com` |
| `cn` | `official` | CN Official (Hypergryph) | `ak.hycdn.cn` |
| `bilibili` | `bili`, `b` | CN Bilibili | `ak.hycdn.cn` |

### Incremental Downloads

Downloads are smart about skipping files already downloaded:

1. On first run: loads/creates `persistent_res_list.json` (empty)
2. Fetches `hot_update_list.json` from CDN (lists all files + MD5s)
3. Compares each file's MD5 against saved manifest
4. Downloads only files with mismatched or missing MD5s
5. Extracts ZIP and updates manifest with new MD5

**Manifest File:** `ArkAssets/persistent_res_list.json`
```json
{
  "pack/audio/sound_cn/voice_en/char_001_amiya.ab": "abc123def...",
  "pack/chararts/char_002_amiya.ab": "xyz789..."
}
```

### Download URL Construction

Downloaded files are referenced as `.ab` in hot_update_list but stored as `.dat` on CDN:

```
Input filename:  pack/chararts/char_002_amiya.ab
Transform:       Replace extension: .ab → .dat
Encode path:     Replace / with _, # with __
                 pack/chararts/char_002_amiya.dat → pack_chararts_char_002_amiya.dat
Final URL:       {cdn_base}/v{res_version}/pack_chararts_char_002_amiya.dat
```

---

## Unpacker CLI Reference

**Binary:** `./binaries/unpacker`

### Commands

#### `extract`

Extract assets from bundles.

**Flags:**

| Flag | Short | Type | Description |
|------|-------|------|---|
| `--input` | `-i` | path | Input directory containing `.ab`, `.dat`, `.bin` files |
| `--output` | `-o` | path | Output directory for extracted assets |
| `--image` | | bool | Extract textures only (Texture2D, class 28) |
| `--text` | | bool | Extract text assets only (TextAsset, class 49) |
| `--audio` | | bool | Extract audio only (AudioClip, class 83) |
| `--spine` | | bool | Extract Spine animations |
| `--portrait` | | bool | Extract operator portraits (from `char_portrait` / `charportraits` atlases) |
| `--gamedata` | | bool | Extract gamedata (requires `--idx`) |
| `--idx` | | path | Path to `.idx` manifest file (for gamedata) |
| `--no-merge` | | bool | Do not composite `foo` / `foo[alpha]` texture pairs; write both raw |
| `--jobs` | `-j` | number | Parallel threads (default: CPU count) |

**Behavior:**

- If **no type flags** are set: extract everything (image + text + audio + spine + portrait + gamedata)
- If **any type flag** is set: extract only those types
- Alpha companions are merged by default. Arknights splits transparency into a separate
  `foo[alpha]` texture; the exporter composites the pair into one RGBA `foo.png` and also
  writes the raw alpha. Unpaired textures pass through untouched. A page renamed during
  atlas dedup takes its alpha companion with it.
- `--gamedata` with no `--idx`: auto-searches input dir and parent dir for `.idx` file; errors if none found and `--gamedata` explicitly requested
- Multi-threaded: uses rayon to process files in parallel

**Examples:**

```bash
# Extract all asset types
unpacker extract -i ./ArkAssets -o ./output

# Textures only, 4 threads
unpacker extract -i ./ArkAssets -o ./output --image -j 4

# Spine animations only
unpacker extract -i ./ArkAssets -o ./output --spine

# Gamedata with explicit manifest
unpacker extract -i ./ArkAssets -o ./output --gamedata --idx ./ArkAssets/manifest.idx

# Low-resource mode: single thread, textures only
unpacker extract -i ./ArkAssets -o ./output --image -j 1
```

#### `list`

List all objects in a single bundle file (debugging).

**Flags:**

| Flag | Type | Description |
|------|------|---|
| `--input` | path | Bundle file path |

**Example:**

```bash
unpacker list -i ./ArkAssets/chararts/char_001_amiya.ab
```

**Output:**
```
Bundle: 2 file(s)

--- File 0: CAB-abc123def (1024000 bytes) ---
  Unity 2021.3.0f1, platform 13, 5 objects
  [  28] Texture2D          path_id=12345        size= 2097152  amiya_idle
  [  49] TextAsset         path_id=12346        size=  1024     amiya.skel
  [ 114] MonoBehaviour     path_id=12347        size=   512     SkeletonMecanim
```

#### `verify`

Report which FlatBuffer schema each gamedata table verifies against (CN or Yostar). This
is a diagnostic report, not a gate: it always exits `0`. Use it when a table decodes to
garbage and you need to know whether the wrong schema family is being applied.

**Flags:**

| Flag | Type | Description |
|------|------|---|
| `--input` | path | Input directory containing gamedata bundles |
| `--idx` | path | Path to the `.idx` manifest |

```bash
unpacker verify -i ./ArkAssets --idx ./ArkAssets/manifest.idx
```

---

## Asset Types & Export Formats

### Textures (Texture2D, Class 28)

**Input Format:** Unity Texture2D with various internal formats

**Supported Formats:**
- `1`: Alpha8 (grayscale)
- `3`: RGB24 (8-bit per channel)
- `4`: RGBA32 (8-bit per channel + alpha)
- `5`: ARGB32 (alpha-first ordering)
- `7`: RGB565 (16-bit 5-6-5)
- `13`: RGBA4444 (4-bit per channel)
- `34`: ETC_RGB4 (Ericsson Texture Compression)
- `45`: ETC2_RGB
- `47`: ETC2_RGBA8
- `48-56`: ASTC variants (4x4 to 12x12)

**Output Format:** PNG (RGBA, 8-bit per channel, flipped to top-left origin)

**Data Source:**
- Inline: base64-encoded in `image data` field
- External: `.resS` or `.resource` files referenced by `m_StreamData.path`, with offset/size

**Process:**
1. Decode format-specific pixels to ABGR u32 buffer
2. Convert to RGBA bytes
3. Flip vertically (Unity origin is bottom-left)
4. Save as PNG

### Text Assets (TextAsset, Class 49)

**Detection Chain:**

1. **Raw bytes check:**
   - Try AES-CBC decrypt (128-byte RSA header + IV seed + encrypted data)
   - Key: `UITpAi82pHAWwnzqHRMCwPonJLIB3WCl[0..16]`
   - IV: `data[0..16] XOR key[16..32]`

2. **JSON parse:** Try parsing as JSON

3. **FlatBuffer decode:** Try FlatBuffer schema detection at offset 0 and offset 128

4. **Fallback:**
   - If valid UTF-8: save as `.txt`
   - Otherwise: save as `.bytes` (raw binary)

**Output Formats:**
- `.json` (FlatBuffer-decoded or plain JSON)
- `.txt` (UTF-8 text)
- `.bytes` (binary)

### Audio (AudioClip, Class 83)

**Format Detection (magic bytes):**

| Magic | Format | Extension |
|-------|--------|---|
| `OggS` | Ogg Vorbis | `.ogg` |
| `RIFF` (at 0) | WAV | `.wav` |
| `ftyp` (at offset 4) | MP4 (m4a) | `.m4a` |
| `FSB5` | FMOD SoundBank | `.ogg` (decoded) |
| Other | Unknown | `.bytes` |

**Data Source:**
- Inline: `m_AudioData` (base64-encoded)
- External: `m_Resource.m_Source` + offset/size in `.resS` files

**FSB5 Special Handling:** Vorbis streams are decoded to OGG format

### Spine Animations

Spine animations are organized by category based on bundle path:

| Category | Bundle Paths | Output Dir |
|----------|-------|---|
| `BattleFront` | `chararts/`, `skinpack/` | `spine/BattleFront/` |
| `BattleBack` | (not yet in typical bundles) | `spine/BattleBack/` |
| `Building` | `building/vault/characters/` | `spine/Building/` |
| `DynIllust` | `arts/dynchars`, `arts/dynavatars` | `spine/DynIllust/` |
| `Enemy` | enemy chibi bundles | `spine/Enemy/` |

Atlas pages carry per-page dimensions parsed from the `.atlas` text. When two pages
collide on name, the highest-resolution one is kept and its alpha companion is renamed
with it.

**Reference Chain (MonoBehaviour graph traversal):**

```
SkeletonMecanim (MonoBehaviour)
  → skeletonDataAsset → SkeletonData (MonoBehaviour)
    → skeletonJSON → TextAsset (.skel binary, base64)
    → atlasAssets → Atlas (MonoBehaviour)
      → atlasFile → TextAsset (.atlas text)
      → materials → Material[]
        → m_SavedProperties.m_TexEnvs._MainTex → Texture2D
        → m_SavedProperties.m_TexEnvs._AlphaTex → Texture2D (optional)
```

**Output Structure:**
```
spine/BattleFront/char_002_amiya/
  ├── char_002_amiya.skel
  ├── char_002_amiya.atlas
  ├── char_002_amiya_0.png
  └── char_002_amiya_1.png
```

**Memory Optimization:**
- Phase 1: Extract only Spine-relevant class IDs (114, 49, 21, 28)
- Phase 2: Drop spine objects; extract remaining asset types
- This prevents double-decoding and saves memory on large bundles

### Gamedata (FlatBuffer Excel files)

**Input:** `.bin` bundle files in `anon/` directory with encrypted TextAssets

**Manifest:** `.idx` file (resource manifest) mapping anonymous bundle hashes to output paths

**Format:** Each TextAsset contains FlatBuffer-serialized Excel table (character table, skill table, etc.)

**Process:**
1. Load `.idx` manifest (maps anon bundle hash → `gamedata/excel/character_table.json`)
2. Scan all `.bin` files
3. For each TextAsset: decrypt (if needed) and decode FlatBuffer
4. Save as JSON to output path from manifest

**Output Example:**
```
gamedata/excel/
  ├── character_table.json
  ├── skill_table.json
  ├── stage_table.json
  └── ... (60+ tables)
```

---

## Output Directory Structure

Full tree structure after `unpacker extract -i ./ArkAssets -o ./output`:

```
output/
├── textures/
│   ├── chararts/
│   │   ├── char_001_amiya/
│   │   │   ├── avatar.png
│   │   │   └── idle.png
│   │   ├── char_002_amiya/
│   │   │   └── ...
│   │   └── ...
│   ├── ui/
│   │   ├── logo.png
│   │   └── ...
│   └── building/
│       └── ...
├── text/
│   ├── chararts/char_001_amiya/
│   │   └── battle_data.json
│   ├── ui/
│   │   └── config.json
│   └── ...
├── audio/
│   ├── sound_beta_2/voice_en/
│   │   ├── char_001_amiya_atk.ogg
│   │   └── char_001_amiya_die.wav
│   └── ...
├── spine/
│   ├── BattleFront/
│   │   ├── char_001_amiya/
│   │   │   ├── char_001_amiya.skel
│   │   │   ├── char_001_amiya.atlas
│   │   │   ├── char_001_amiya_0.png
│   │   │   └── char_001_amiya_1.png
│   │   ├── char_002_amiya/
│   │   │   └── ...
│   │   └── ... (100+ characters)
│   ├── BattleBack/
│   │   └── ...
│   ├── Building/
│   │   └── ...
│   ├── DynIllust/
│   │   └── ...
│   └── Enemy/
│       └── ...
├── portraits/
│   └── char_002_amiya_1.png       (sliced from the shared atlas)
├── derived/
│   └── gacha_pool_details.json    (backend-facing derived data)
└── gamedata/
    ├── levels/ story/ battle/ building/
    └── excel/
        ├── character_table.json
        ├── skill_table.json
        ├── stage_table.json
        ├── item_table.json
        ├── enemy_database.json
        ├── activity_table.json
        └── ... (60+ files)
```

---

## Common Workflows

### Full Pipeline (Download + Extract All)

```bash
# Use interactive runner (recommended)
npm start
# → Select "Check for Updates & Update"
# → Choose server, directories
# → Download and extract automatically

# OR manual steps:
downloader --server en download --all
unpacker extract -i ./ArkAssets -o ./output
```

### Spine-Only Extraction

Extract only Spine animations (fast, low memory):

```bash
unpacker extract -i ./ArkAssets -o ./output --spine -j 8
```

**Output:** `output/spine/BattleFront/`, `BattleBack/`, `Building/`, `DynIllust/`

### Gamedata-Only Extraction

Extract only Excel gamedata tables:

```bash
unpacker extract -i ./ArkAssets -o ./output --gamedata --idx ./ArkAssets/manifest.idx
```

**Output:** `output/gamedata/excel/` (JSON files)

### Incremental Updates (Download Only Changed)

```bash
# Previous download version is remembered in .version
downloader --server en download --all

# Only new/modified packs are downloaded
# Manifest tracks MD5s of all files
```

**Benefit:** Subsequent runs only download changed packs (typically 100-500 MB vs. 5-10 GB)

### Low-Resource Mode

For systems with limited CPU/RAM:

```bash
# Single-threaded extraction
unpacker extract -i ./ArkAssets -o ./output --image -j 1

# Then extract remaining types separately
unpacker extract -i ./ArkAssets -o ./output --text -j 1
unpacker extract -i ./ArkAssets -o ./output --audio -j 1
```

### Multi-Region Setup

Download assets for multiple regions:

```bash
# Create separate directories
mkdir assets_{en,jp,cn}

# Download each region
downloader --server en -d ./assets_en download --all
downloader --server jp -d ./assets_jp download --all
downloader --server cn -d ./assets_cn download --all

# Extract to separate outputs
unpacker extract -i ./assets_en -o ./output_en
unpacker extract -i ./assets_jp -o ./output_jp
unpacker extract -i ./assets_cn -o ./output_cn
```

### WebSocket-Based Continuous Monitoring

```bash
# Terminal 1: Start server
npm start
# → Select "WebSocket Server (continuous updates)"
# → Configure port 9160, check interval 30 minutes

# Terminal 2: Connect client
ws://localhost:9160
```

See [WebSocket Protocol Reference](#websocket-protocol-reference) for event formats.

---

## Object Storage Sync

When the backend serves assets from an S3-compatible bucket (Cloudflare R2, AWS S3, MinIO
and friends; the `S3_*` block in [`backend/.env.example`](../backend/.env.example)), the
bucket has to hold what `output/<server>/` holds. `binaries/asset-sync`, a binary of
`backend/crates/storage`, does that, and the watcher runs it on its own.

**What is synced.** The browser-served roots of one region: `textures`, `spine`, `audio`,
`video`, `portraits` (`S3_ROOTS` overrides the list). `gamedata`, `derived`, `text` and
anything else the backend reads for data stay local only. Keys are
`[S3_PREFIX/]<server>/<path under output/<server>/>`.

**When.** In `run.mjs ws`, after every successful extract and its orphan sweep, before
the `update_complete` broadcast. While it runs the watcher reports `state: "syncing"`, its
one-line summary is broadcast as a `status` message, and `update_complete` carries
`synced` and `syncSummary`.

**Retry.** A failed sync never costs a re-download or re-extract. The extract is recorded
as usual (`.version`, `.last_extract`, extract backoff cleared), an `error` with the
summary is broadcast, and `update_complete` is HELD, because the backend must reload only
against a bucket that matches the tree. The failure is persisted as
`ArkAssets/<server>/.sync-pending`:

```json
{"region": "en", "version": "1.27.6_09_01_10_20", "exported": 2847, "attempts": 1,
 "lastCode": 2, "lastSummary": "asset-sync exited with code 2 (...)", "at": "2026-10-22T04:31:35.992Z"}
```

At the top of every tick, before the version check and regardless of the extract backoff,
and before a `force_update`, the watcher retries the sync alone (no download, no extract).
Success deletes the marker and sends the held `update_complete` for that version
(`downloaded: 0`, `synced: true`). Failure bumps `attempts`, broadcasts `error`, and the
tick carries on to the version check, so a newer version still gets a full update whose
own sync replaces the pending one. The retry has no backoff of its own: one attempt per
tick. With `WS_S3_SYNC=0` the marker is left alone and not retried. Watchers survive
restarts with the marker on disk, so a pm2 restart retries on the first tick.

**Seeding.** The first sync uploads everything, so run it by hand per region before
turning on redirect delivery, rather than inside a watcher cycle:

```bash
cd assets
node run.mjs sync --server en --dry-run   # report the diff, change nothing
node run.mjs sync --server en             # EN: about 28 GB / 178k files across the five roots
node run.mjs sync --server cn             # then each other region that has art
```

Later runs hash only files whose size matches the remote object and upload only what
changed, so a re-run after an interruption resumes where it stopped. The same command is
the manual repair: a successful (non `--dry-run`) run rewrites
`ArkAssets/<server>/.sync-pending` (`--savedir` if the watcher uses another one) with
`synced: true` and its `syncSummary`, and leaves it in place. On its next tick the watcher
sees a synced marker, skips the sync stage, sends the held `update_complete` for that
version with the recorded summary, and only then removes the marker, so the backend
reloads as it would have after a clean update. `--dry-run` leaves the marker untouched.

The Docker watchers (`asset-watcher-en`/`-cn` in `docker-compose.yml`) ship neither the
binary nor the repo's `.env` files, so for them the stage is always the logged skip.

**Configuration.**

| Variable | Meaning |
|----------|---------|
| `WS_S3_SYNC` | `0` skips the stage in `run.mjs ws`. Anything else, or unset, runs it. `run.mjs sync` ignores it. |
| `WS_S3_ENV_FILE` | Env file handed to `asset-sync --env-file`, replacing the default list. Default: `../backend/.env` then the repo-root `../.env`, each passed only if it exists (`--env-file ../backend/.env --env-file ../.env`), the same order the backend loads them in; with neither, the binary reads `assets/.env`. `--env-file` on `run.mjs sync` or `restore` overrides it. |

Every file is read the way dotenv 0.15 reads it, which is how the backend and the binary
read it: the first file that sets a variable wins (a variable already in the process
environment beats all of them), a ` #comment` after a value is dropped even after a
quoted one (`S3_TYPE="off" # later` is `off`), and an unquoted value with whitespace in
it is a malformed line. Parsing of that file STOPS at the first malformed line, for all
three readers, so `run.mjs` logs one warning naming the file and line number.

Storage is "configured" when the winning `S3_TYPE` is non-empty and not `off`.
Without the binary, a configured box fails the stage (the bucket would silently fall
behind) and an unconfigured one logs `asset-sync: skipped (binary not installed, S3_TYPE
unset)`. With the binary and no `S3_TYPE`, the binary itself reports `storage disabled`
and exits 0. Under Node, a `--env-file` path that does not exist is rejected by Node
itself (`node: <path>: not found`, exit 9) before `run.mjs` runs; under Bun, or through
`WS_S3_ENV_FILE`, the same mistake exits 1.

**Exit codes** (`run.mjs sync` exits with the binary's code):

| Code | Meaning |
|------|---------|
| 0 | Synced, or storage disabled |
| 1 | Configuration or usage error (missing env file, bad `S3_*` value, binary missing while `S3_TYPE` is set) |
| 2 | One or more objects failed to upload or delete after retries, or another `asset-sync` holds the server's lock (see [stubs](#freeing-local-disk-stubs)) |
| 3 | Delete refused: the local tree holds under 90 % of the remote object count. A truncated extract must not empty the bucket; `node run.mjs sync --server <r> --allow-shrink` overrides it by hand, after checking the tree (see below) |

**Delete refusals (exit 3).** The refusal comes after every upload has landed, so the
bucket is a superset of the tree and a backend reload is safe. The watcher therefore
writes the stamps, broadcasts `error` with the summary, sends `update_complete` with
`synced: "uploads-only"` and the summary, and writes `.sync-pending` with
`needsAllowShrink: true`. That marker is NOT retried automatically: each tick logs one
line asking for `node run.mjs sync --server <r> --allow-shrink`, because only a human can
tell a shrunken release from a truncated tree. That manual run removes the marker on
success directly (the `update_complete` was already sent). A later full update whose own
sync succeeds also clears it.

**Deletes mirror the orphan sweep.** Uploads run first, then every remote key under a
synced root with no local file is deleted, so what the sweep removed locally leaves the
bucket on the same run. A root missing locally is logged and its remote keys are left
alone. `WS_KEEP_ORPHANS=1` therefore also keeps those objects in the bucket.

**Manifest.** `output/<server>/.asset-sync.json` (version 2) records the `bucket` and
`prefix` it was written against, and `{size, md5, etag, evicted}` for every key a sync left
unchanged or uploaded. Multipart objects (above
`S3_MULTIPART_THRESHOLD_BYTES`, default 256 MiB) have an ETag that is not an MD5, so the
manifest is what tells an unchanged large file from a changed one. It is also the only
record of which 0-byte files are [stubs](#freeing-local-disk-stubs): while no stubs exist,
deleting it costs one re-upload of the multipart files, nothing else; with stubs on disk,
run `restore` before deleting it, or the next sync uploads every stub as an empty object
over the real one.

**CORS.** The frontend fetches spine, atlas and audio files cross-origin from the bucket's
public URL, so the bucket needs a CORS rule once:

```bash
binaries/asset-sync cors --allow-origin https://myrtle.moe --env-file ../.env
```

The binary is built with `cargo build --release -p myrtle-storage --bin asset-sync` in
`backend/` and installed to `assets/binaries/asset-sync`; `vps-update.sh` does both.

### Freeing local disk (stubs)

With `S3_EVICT_LOCAL=1`, a sync frees the bytes of every file the bucket is confirmed to
hold (unchanged, or uploaded on this run). The file is truncated in place to 0 bytes,
same path, its original mtime kept, and its manifest record gets `evicted: true`. That 0-byte file is a
**stub**. It is never deleted, because the names are load-bearing:

- The backend builds its asset, chibi, stage, story and music indexes from the file
  names under the served roots at startup and on `update_complete`, and in redirect and
  proxy delivery a local file existing is what decides that a request goes to the bucket.
- `run.mjs` re-extracts when the file count drops under 98 % of the last extract.
- The unpacker skips the expensive re-encodes (video, music AAC, the content-addressed
  `tex/` pool) when an output exists and is newer than its source. A stub keeps the
  mtime its extract gave it, so it passes that check, and the skip path touches it, so
  the orphan sweep keeps it.
- The orphan sweep decides which top-level roots an extract wrote by mtime. Truncation
  therefore keeps the original mtime: a stub stamped "now" would look freshly written and
  skew that decision.

On the next sync a stub whose record says `evicted` and whose object is in the bucket
counts as unchanged: no hashing, no upload. A stub whose object is gone from the bucket
fails the sync with `evicted stub but the object is gone from the store; re-extract to
recover`, because the bytes are then lost on both sides. A truncate that fails is logged
and counted (`evict_failed`) and does not change the exit code: the bytes are still in
both places. With eviction on, the summary line gains `evicted=N (B bytes freed)`; with
it off, the line is unchanged.

**What is never evicted.** Files the backend or the unpacker read for their BYTES, not
just their names. Prefixes under `output/<server>/`:

| Prefix | Read by |
|--------|---------|
| `textures/avg/` | story reader: `hub.json`, `sprites.json`, PNG headers, thumbnails, gallery variants |
| `textures/spritepack/cutin_char_` | story art variants |
| `textures/spritepack/ui_home_act_banner_gacha_` | release art, copied into `derived/release-art` |
| `textures/spritepack/ui_zone_home_theme_` | release art |
| `textures/arts/ui/stage/[uc]homeentry/` | release art |
| `textures/[[<server>]]/arts/ui/homebanners/` | release art: the gacha home banner the archive copies as bytes |
| `audio/m4a_sources.json` | the unpacker's music step |

`S3_EVICT_KEEP` adds prefixes to that list; it cannot remove one. Measured on EN
2026-10-22: the served roots hold 28,929,636 KiB, the keep list 5,041,584 KiB, so about
22.8 GiB is evictable.

**Disk peak.** Eviction lowers the steady state between extracts, not the peak. Every
extract rewrites the tree at full size (all but the skipped re-encodes above), and only
the sync after it turns the files back into stubs, so the box still needs room for a full
tree during an extract.

**Configuration** (in the same env files the sync reads):

| Variable | Meaning |
|----------|---------|
| `S3_EVICT_LOCAL` | `1`, `true`, `yes` or `on` enables eviction. Unset or anything else: off, and sync never truncates a file. |
| `S3_EVICT_KEEP` | Comma list of extra rel-path prefixes to keep, added to the built-in list. Leading `/` trimmed; backslashes rejected. |

The binary's `sync` also takes `--evict` / `--no-evict`, which win over the env.

**Delivery guard.** Eviction refuses to run (exit 1) when the same env resolves to local
delivery (`S3_TYPE` unset or `off`, or `ASSET_DELIVERY=local`), because the backend would
then serve the 0-byte stubs. `asset-sync sync --force-evict` overrides it. With local
delivery and stubs on disk, the backend logs one warning per server at startup.

**Bucket and prefix are pinned.** With stubs on disk, a sync whose `S3_BUCKET` or
`S3_PREFIX` differs from the one recorded in the manifest is refused. Run `restore` under
the OLD settings first; otherwise every stub would be uploaded as an empty object under
the new location.

**One asset-sync per server.** `sync` and `restore` take a non-blocking lock,
`output/<server>/.asset-sync.lock` (`verify` and `ls` do not). A second one on the same
server fails at once with an error naming the lock: `sync` exits 2, `restore` exits 1. So
a manual `node run.mjs sync` or `node run.mjs restore` cannot corrupt a watcher-driven
sync; if the watcher's sync hits the lock instead, it fails like any other sync, writes
`.sync-pending`, and its retry tries again on the next tick.

**Do not evict during an extract.** The watcher runs its sync only after its own extract,
but a manual evicting `sync` while an extract is running on the same server can truncate a
file the unpacker just rewrote. The truncation re-checks size and mtime first, which
narrows the window but does not close it. Run manual evicting syncs between extracts.

**Rolling back the binary.** An `asset-sync` older than eviction has no idea what a stub
is: it sees a 0-byte file whose size differs from the object and uploads it as an empty
object over the real one. Run `restore` with the current binary before installing an
older one.

**Turning it off.** Unsetting `S3_EVICT_LOCAL` stops new stubs. Existing ones stay until a
restore brings them back; an extract rewrites most of them, but not the outputs it skips
(video, music, the `tex/` pool). Before turning storage off
(or switching to `ASSET_DELIVERY=local`), restore first:

```bash
cd assets
node run.mjs restore --server en --dry-run              # count stubs and bytes, change nothing
node run.mjs restore --server en                        # download every stub back, clear its flag
node run.mjs restore --server en --only textures/arts/  # only rel paths under a prefix (repeatable)
```

`restore` reads the same env files as `sync` (`--env-file`, `WS_S3_ENV_FILE`, then the
default list), works with `S3_EVICT_LOCAL` unset, verifies size (and MD5 when the ETag is
not multipart) before replacing a stub, and saves the manifest at the end, so a re-run
after an interruption finds only the remaining stubs. It restores only a path that exists
as a 0-byte file. A flagged path that is missing is counted as `missing` and left alone:
the orphan sweep removed it, so restoring it would resurrect an orphan, and the next
sync's delete step removes its object. A file that gained bytes between selection and the
rename (an extract rewrote it) is counted as `skipped` and not overwritten. It ends with
`asset-sync: restore server=en restored=N (B bytes) failed=N elapsed=S.s`, plus
` missing=N skipped=N` when either is above 0, and exits 0, or 1 on a config error, a held
lock, or any stub not restored.

**Known gap.** A standalone `unpacker cardfields` or `backfill-*` run, outside a full
extract, sees a stub as absent: it finds 0 bytes where it expects data. Restore the paths
it reads (`--only`) before such a run. Not fixed, only documented.

---

## WebSocket Protocol Reference

The WebSocket server broadcasts asset pipeline status and accepts client commands.

### Server → Client Events

#### `status`

Sent on connection and when state changes.

```json
{
  "type": "status",
  "state": "idle|checking|downloading|unpacking|syncing|backing_off",
  "version": {
    "current": "1.27.5_08_15_11_51"
  }
}
```

#### `update_available`

New version detected on CDN.

```json
{
  "type": "update_available",
  "currentVersion": "1.27.5_08_15_11_51",
  "newVersion": "1.27.6_09_01_10_20",
  "clientVersion": "2.1.0"
}
```

#### `download_progress`

Per-file progress during download phase.

```json
{
  "type": "download_progress",
  "completed": 45,
  "total": 150,
  "percent": 30.0
}
```

#### `download_complete`

Download phase finished.

```json
{
  "type": "download_complete",
  "downloaded": 145,
  "failed": 5,
  "totalBytes": 5368709120,
  "totalBytesFormatted": "5.0 GB"
}
```

#### `unpack_progress`

Per-bundle progress during extraction.

```json
{
  "type": "unpack_progress",
  "completed": 120,
  "total": 300,
  "percent": 40.0
}
```

#### `update_complete`

Full download + extract (+ object storage sync) cycle finished. `synced` is true only
when the [sync stage](#object-storage-sync) ran against a configured bucket;
`syncSummary` is its summary line, or null when the stage is off.

```json
{
  "type": "update_complete",
  "version": "1.27.6_09_01_10_20",
  "downloaded": 145,
  "failed": 5,
  "exported": 2847,
  "synced": true,
  "syncSummary": "asset-sync: server=en scanned=178000 unchanged=176500 uploaded=1500 (812345678 bytes) deleted=12 failed=0 elapsed=95.2s"
}
```

#### `resource_list`

Directory listing of extracted assets (on demand).

```json
{
  "type": "resource_list",
  "files": [
    {
      "name": "textures",
      "path": "textures",
      "type": "directory",
      "size": 1073741824,
      "fileCount": 2847,
      "modified": "2024-01-15T10:30:00Z",
      "created": "2024-01-15T10:30:00Z"
    },
    {
      "name": "output.json",
      "path": "output.json",
      "type": "file",
      "size": 102400,
      "modified": "2024-01-15T10:30:00Z",
      "created": "2024-01-15T10:30:00Z"
    }
  ],
  "totalSize": 1073843824,
  "totalSizeFormatted": "1.0 GB"
}
```

#### `error`

Error occurred in pipeline.

```json
{
  "type": "error",
  "message": "Download failed: network timeout"
}
```

### Client → Server Commands

#### `force_update`

Trigger immediate download + extract cycle (bypass periodic check).

```json
{
  "type": "force_update"
}
```

**Response:** Status transitions: `idle` → `downloading` → `unpacking` → `idle`

#### `list_resources`

Request current directory listing.

```json
{
  "type": "list_resources"
}
```

**Response:** `resource_list` event

### Example Session

```
[Client connects]
← {"type": "status", "state": "idle", "version": {"current": "1.27.5_08_15_11_51"}}

[Periodic check triggers - new version available]
← {"type": "update_available", "currentVersion": "1.27.5_08_15_11_51", "newVersion": "1.27.6_09_01_10_20", ...}
← {"type": "status", "state": "downloading", ...}

[Download progresses]
← {"type": "download_progress", "completed": 10, "total": 150, "percent": 6.67}
← {"type": "download_progress", "completed": 20, "total": 150, "percent": 13.33}
...

[Download complete]
← {"type": "download_complete", "downloaded": 145, "failed": 5, "totalBytes": 5368709120, ...}
← {"type": "status", "state": "unpacking", ...}

[Extraction progresses]
← {"type": "unpack_progress", "completed": 50, "total": 300, "percent": 16.67}
← {"type": "unpack_progress", "completed": 100, "total": 300, "percent": 33.33}
...

[Complete]
← {"type": "update_complete", "version": "1.27.6_09_01_10_20", "downloaded": 145, "failed": 5, "exported": 2847}
← {"type": "status", "state": "idle", "version": {"current": "1.27.6_09_01_10_20"}}

[Client requests resource list]
→ {"type": "list_resources"}
← {"type": "resource_list", "files": [...], "totalSize": 1073843824, ...}
```

---

## Architecture

### Crate Structure

**Downloader** (`downloader/edition 2024`)
- `cli.rs`: Clap CLI interface
- `server.rs`: Server enum + CDN URLs + FromStr parsing
- `version.rs`: Version endpoint fetching
- `hot_update.rs`: Hot update list parsing
- `manifest.rs`: Persistent MD5 manifest (incremental download tracking)
- `download.rs`: HTTP client with semaphore-based concurrency control
- `extract.rs`: ZIP extraction (downloaded files are zipped)
- `pipeline.rs`: Orchestration (download → extract → manifest update)
- `progress.rs`: indicatif progress bars
- `types.rs`: Common types (VersionResponse, HotFile, PipelineStats)
- `error.rs`: Error types
- `profile.rs`: Content profiles (`full`, `operators`, `stages`, `gamedata`, `release`) and their filters
- `resource_manifest.rs`: `.idx` resource-manifest handling
- `client_extract.rs`: Offline APK/XAPK/OBB/IPA triage for IL2CPP recovery. Contacts no server

**Unpacker** (`unpacker/edition 2024`)
- `cli.rs`: Clap CLI (Extract/List/Verify commands)
- `unity/`:
  - `bundle.rs`: UnityFS parsing (header → block info → block data)
  - `compression.rs`: Decompression (LZMA, LZ4, LZ4AK)
  - `serialized_file.rs`: Serialized file header + object metadata
  - `type_tree.rs`: Type tree parsing for object deserialization
  - `object_reader.rs`: Deserialize objects using type tree
  - `endian_reader.rs`: Byte reading with endianness
  - `lz4ak.rs`: LZ4AK (Arknights custom) decompression pre-processing
- `export/`:
  - `texture.rs`: Decode formats (RGBA32, ETC2, ASTC, etc.) → PNG
  - `text_asset.rs`: AES decrypt + FlatBuffer detection + JSON/UTF-8 fallback
  - `audio.rs`: Format detection (OGG, WAV, M4A, FSB5) → appropriate format
  - `spine.rs`: MonoBehaviour graph traversal → organized by category
  - `gamedata.rs`: Manifest-based extraction of Excel tables
  - `fsb5.rs`: FMOD SoundBank → OGG decoding
  - `manifest.rs`: Resource manifest parsing (.idx files)
  - `alpha_merge.rs`: Composite `foo` / `foo[alpha]` texture pairs into RGBA
  - `portrait.rs`: Slice operator portraits out of `SpritePacker` atlases
  - `stage_preview.rs`: Un-squash `stage_mappreview_*` from forced 512x512 to true aspect
  - `anim.rs` · `particles.rs` · `mesh.rs` · `fx_textures.rs` · `shader_map.rs`:
    dynamic-illustration (L2D) reconstruction - camera curves, emitters, mesh bases,
    effect textures, GLSL shader extraction
  - `cardfields.rs`: Merge measured `backdropScale` / `backdropOffsetPx` into exported
    DynIllust scenes. These values are solved from real renders, not computable here,
    so re-exports regenerate them rather than losing them
  - `mod.rs`: Export module exports
- `flatbuffers_decode.rs`: Generic FlatBuffer schema decoder → JSON
- `generated_fbs/`: FlatBuffer schema code (CN, ~50 table definitions)
- `generated_fbs_yostar/`: Yostar (global) variant schemas
- `bin/generate_fbs.rs`: Regenerate the schema decoders (`generate-fbs`)
- `examples/probe_*.rs`, `diag_*.rs`: ~130 diagnostic binaries from the L2D work
- `vendor/flatbuffers/`: Patched flatbuffers adding `VerifierOptions.max_alignment`,
  because Hypergryph ships 4-aligned 8-byte scalars that the stock verifier rejects
- `main.rs`: Entry point + bundle processing pipeline

### UnityFS Bundle Format

**Header** (big-endian, fixed-size):
```
[C-string]  signature ("UnityFS")
[u32]       version (typically 6-7)
[C-string]  player version (e.g., "2021.3.0f1")
[C-string]  engine version
[i64]       file size (bytes)
[u32]       compressed_block_info_size
[u32]       uncompressed_block_info_size
[u32]       data_flags (compression type in bits 0-5, metadata location in bit 7-8)
[alignment] 16-byte align if version >= 7
```

**Block Info** (compressed):
```
[16 bytes]  hash
[i32]       block_count
[Per block]:
  [u32]     uncompressed_size
  [u32]     compressed_size
  [u16]     flags (compression type in bits 0-5)
[i32]       directory_entry_count
[Per directory entry]:
  [i64]     offset (into decompressed data)
  [i64]     size
  [u32]     flags
  [C-string] path (e.g., "CAB-abc123def" or "config.json")
```

**Data Blocks** (streamed, compressed):
- Decompressed in sequence
- Concatenated into single data buffer
- Directory offsets index into this buffer

**Compression Types:**
- `0`: None
- `1`: LZMA (5-byte header + stream)
- `2`/`3`: LZ4/LZ4HC
- `4`: LZ4AK (Arknights custom variant that prepends uncompressed size)

### SerializedFile Format

Binary format describing objects inside a bundle entry (e.g., CAB-abc123def):

**Header:**
```
[u32]       metadata_size
[u32]       file_size
[u32]       format_version
[u32]       data_offset (absolute position of object data)
[u8]        endianness (0=little, 1=big)
[3 bytes]   reserved
[if >= 22]:
  [u32]     metadata_size_64
  [u64]     file_size_64
  [u64]     data_offset_64
  [u64]     reserved
```

**Type Metadata:**
```
[C-string]  unity_version
[i32]       target_platform
[bool]      enable_type_tree
[i32]       type_count
[Per type]:
  [i32]     class_id (28=Texture2D, 49=TextAsset, 83=AudioClip, 114=MonoBehaviour)
  [if >= 16]: [bool] is_stripped
  [if >= 17]: [i16] script_type_index
  [16 bytes] type_hash
  [16 bytes] script_hash
  [if enable_type_tree]:
    [Type tree structure]
    [if >= 21]: [i32] dependency_count + dependency list
```

**Objects:**
```
[i32]       object_count
[Per object]:
  [4-byte align if >= 14]
  [i64]     path_id (unique ID)
  [u32|u64] byte_offset (relative to data_offset)
  [u32]     byte_size
  [i32]     type_index (into types array above)
```

### Type Tree System

Type tree is a recursive structure describing the binary layout of an object:

```
[u32]       node_count
[Per node]:
  [C-string] type_name (e.g., "Texture2D", "m_Width")
  [C-string] field_name
  [i32]      byte_offset (relative to object start)
  [i32]      array_size (>0 if array)
  [i32]      type_flags
  [u32]      version (schema version for this type)
```

Used to deserialize objects: given binary data + type tree, recursively read fields into JSON.

---

## Troubleshooting

### "error: invalid server: xyz"

The `--server` flag value is not recognized.

**Fix:** Use a valid server key from the table above. Example: `--server en` or `--server cn`.

### "error: zip extraction failed"

Downloaded file is corrupted or not a valid ZIP.

**Cause:** Network interruption, corrupted CDN file, or filesystem issue.

**Fix:**
```bash
# Clear and retry
rm -rf ArkAssets/persistent_res_list.json
downloader --server en download --all
```

This forces re-download of all files (manifest is cleared).

### "error: no .idx manifest found"

Trying to extract gamedata without the manifest.

**Fix:**
```bash
# Download first (includes manifest)
downloader --server en download --all

# Then specify manifest
unpacker extract -i ./ArkAssets -o ./output --gamedata --idx ./ArkAssets/manifest.idx
```

Or allow auto-detection:
```bash
unpacker extract -i ./ArkAssets -o ./output --gamedata
# Auto-searches input dir and parent for *.idx
```

### "Unpacker exited with code 1"

Generic extraction failure (likely malformed bundle).

**Diagnosis:**
```bash
# List bundle contents to see if parseable
unpacker list -i ./ArkAssets/chararts/char_001_amiya.ab
```

If `list` fails, the bundle is corrupted. Re-download that pack:
```bash
rm -rf ArkAssets/persistent_res_list.json
downloader --server en download --all
```

### "Build failed: Rust 1.84.0 is too old"

Rust version does not meet Edition 2024 requirement (1.85.0+).

**Fix:**
```bash
rustup update
rustc --version  # Should show 1.85.0+
```

### Out of memory during extraction

Unpacker loads entire bundles into RAM. Large bundles (500 MB+) can exhaust memory.

**Fix:**
```bash
# Reduce thread pool
unpacker extract -i ./ArkAssets -o ./output -j 2

# Extract types separately to free memory between runs
unpacker extract -i ./ArkAssets -o ./output --image -j 2
unpacker extract -i ./ArkAssets -o ./output --text -j 2
unpacker extract -i ./ArkAssets -o ./output --audio -j 2
```

### "Not enough data to decode texture" (stderr spam)

Texture has external .resS data that is not fully bundled (e.g., lightmaps).

**Expected behavior:** These are silently skipped (logged to stderr but not an error).

No action needed.

### WebSocket server won't start

Port 9160 is in use.

**Fix:** Use different port in interactive prompt:
```
WebSocket port: 9161
```

Or kill existing process:
```bash
# macOS/Linux
lsof -i :9160
kill -9 <PID>
```

---

## Contributing

### Building from Source

```bash
# Prerequisites
rustc --version  # 1.85.0+
cargo --version

# Build release binaries
cargo build --release -p downloader
cargo build --release -p unpacker

# Run tests
cargo test --lib -p downloader
cargo test --lib -p unpacker
```

### Code Structure Guidelines

**Downloader crate:**
- Keep `cli.rs` focused on argument parsing only
- Move logic to feature modules (download, manifest, etc.)
- Use `anyhow::Result` for error handling
- Log with `tracing` (respect `--verbose` flag)

**Unpacker crate:**
- `unity/` handles parsing only (no IO beyond reading)
- `export/` handles writing (no parsing of source formats)
- Use `serde_json::Value` for deserializer output (flexible)
- Rayon parallelism in `main.rs` only

### Testing

Tests run against **committed fixtures** under `unpacker/tests/assets/` and the
`wiremock` HTTP mock in the downloader. You do **not** need a populated `./ArkAssets`.

```bash
# Downloader: integration tests against a mocked CDN
cd downloader && cargo test --all-targets && cargo test --doc

# Unpacker: bundle, texture, audio, spine, gamedata, portrait, alpha-merge tests
cd unpacker && cargo test --all-targets --jobs 1
```

Two constraints are not optional for the unpacker:

- **`--jobs 1`.** The generated FlatBuffer code makes this a roughly 2M-line crate. A
  full test build peaks near 4.74 GB of RAM; `--jobs 2` gets OOM-killed, which surfaces
  as **exit code 143**, not as a compile error.
- **`RUST_MIN_STACK=32MB`.** FlatBuffer decoding recurses deeply enough to overflow the
  default stack.

The ~130 `examples/probe_*.rs` and `diag_*.rs` binaries are diagnostic instruments from
the dynamic-illustration work. Clippy compiles them; nothing executes them as tests.

### Formatting

```bash
cargo fmt --all
cargo clippy --lib --bins
```

### CI

[`assets-ci.yml`](../.github/workflows/assets-ci.yml) runs rustfmt and Clippy for both
crates, cross-platform unit and doc tests, release builds with a `--help` smoke test, a
syntax and import check on `run.mjs`, a non-blocking `cargo audit`, and an integration job
that exercises `unpacker list` and `extract` against the committed fixtures. That job also
uploads the extracted `gamedata/` artifact that backend CI consumes for its tests.

---

## Deployment

The pipeline ships as a Docker image (`assets/Dockerfile`) that builds both binaries and
pins `flatc` to match the vendored `flatbuffers` crate. The root `docker-compose.yml` uses
it three ways:

| Service | Profile | Role |
|---------|---------|------|
| `asset-watcher-en` | always-on | `run.mjs ws` for EN on port 9160 |
| `asset-watcher-cn` | always-on | `run.mjs ws` for CN on port 9161 |
| `asset-tools` | `tools` | One-off jobs: `docker compose run --rm asset-tools download.sh en 4` |

`download.sh` takes `SERVER THREADS [PROFILE]`; `unpack.sh` takes `REGION THREADS`. Outside
Docker, `ecosystem.config.cjs` runs the same watchers under PM2 as `myrtle-ws-en` and
`myrtle-ws-cn`. The backend consumes the result through `ASSETS_DIR`, `SERVERS`, and
`ASSET_WS_URLS`; see the [backend README](../backend/README.md).

### Environment

| Variable | Applies to | Description |
|----------|-----------|-------------|
| `WS_SERVER` · `WS_SAVEDIR` · `WS_OUTPUT` · `WS_THREADS` · `WS_PROFILE` · `WS_PORT` · `WS_INTERVAL` | `run.mjs ws` | Non-interactive WebSocket-mode configuration |
| `WS_S3_SYNC` · `WS_S3_ENV_FILE` | `run.mjs ws`, `run.mjs sync`, `run.mjs restore` (`WS_S3_ENV_FILE` only) | Object storage sync stage; see [Object Storage Sync](#object-storage-sync) |
| `DYNCHAR_CARDFIELDS` | unpacker | Override the path to `cardfields.json` |
| `DYNCHAR_NO_CARDFIELDS` | unpacker | Skip the card-field merge pass |
| `DYNCHAR_TEX_POOL` | unpacker | Content-addressed texture dedup pool. On by default; `=0` reverts to per-call PNGs |
| `DYNCHAR_PNG_LEVEL` | unpacker | PNG deflate level. Defaults to `fast` |

The source also contains dozens of other `DYNCHAR_*`, `SCENE_*`, and `IDLE_DEBUG` flags.
Those are A/B measurement toggles left over from matching the dynamic-illustration
renderer against the real client. They are not stable, not supported, and not intended as
configuration.

---

## License

License TBD - see the [project root](../README.md). No license file is currently present
in the repository.

---

## Quick Reference

| Task | Command |
|------|---------|
| Setup | `npm start` → Setup |
| Check version | `downloader --server en check-update` |
| Download all | `downloader --server en download --all` |
| Download operators-only | `downloader --server cn download --all --profile operators` |
| Download specific | `downloader --server en download --packages pack/chararts,pack/ui` |
| Regenerate FBS schemas | `cd unpacker && cargo run --bin generate-fbs` (needs `flatc`) |
| Extract all | `unpacker extract -i ./ArkAssets -o ./output` |
| Extract spine | `unpacker extract -i ./ArkAssets -o ./output --spine` |
| List bundle | `unpacker list -i ./ArkAssets/chararts/char_001.ab` |
| Update loop | `npm start` → Check for Updates & Update |
| Monitor server | `npm start` → WebSocket Server (then ws://localhost:9160) |
| Low-resource extract | `unpacker extract -i ./ArkAssets -o ./output -j 1` |

---

## Regenerating FlatBuffer Schemas

CN and Yostar gamedata are FlatBuffer-encoded; the decoder is generated from the schema
store in [`schemas/`](schemas/README.md) and committed under `unpacker/src/generated_fbs/`
(CN) and `generated_fbs_yostar/` (EN/JP/KR variants). The CN schemas are generated from
the official CN Windows client by `unpacker schema-gen` (no third-party repository is
involved); `unpacker schema-sync` resolves the Yostar variants against the live binaries
from the store's own history. The store layout, the lock file, the automation in
`.github/workflows/schema-watch.yml`, and the record of every hand patch that was refuted
are all in that README. The short form:

```bash
cd unpacker
../downloader/target/release/downloader -s cn client-download -d /tmp/ark/client --platform windows
cargo run --release --bin unpacker -- schema-gen --client /tmp/ark/client --out /tmp/ark/gen
cargo run --release --bin unpacker -- schema-sync --generated /tmp/ark/gen \
  --bundles cn=/tmp/ark/cn --bundles en=/tmp/ark/en --bundles jp=/tmp/ark/jp --bundles kr=/tmp/ark/kr
cargo run --release --bin generate-fbs   # requires flatc 25.12.19 (see Prerequisites)
cargo fmt
cargo build --release
cp target/release/unpacker ../binaries/unpacker   # run.mjs uses ./binaries
```

> **Binary sync:** after any `cargo build`, copy the binary into `./binaries/` - `run.mjs`
> and the documented commands invoke `./binaries/{downloader,unpacker}`, not
> `target/release`. The `Setup` runner mode does this for you.

> **Diagnosing a broken table:** `unpacker verify -i <bundles>` prints, per table, which
> schema the live buffer verifies under; `chosen: none` is a table `extract` will skip
> (previous output kept). The fix is a schema that verifies, found by `schema-sync`,
> never a hand edit: every hand patch this project ever carried was refuted on a later
> binary (the record is in `schemas/README.md`).

---

Last updated: 2026-10-15
