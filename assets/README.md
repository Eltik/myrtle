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
    └── spine/ (BattleFront, BattleBack, Building, DynIllust)
```

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
- Multi-threaded extraction with rayon (configurable thread pool)
- FlatBuffer schema support for gamedata Excel files

**Runner** (`run.mjs`):
- Interactive CLI for 3 workflows: setup, incremental update, WebSocket server
- Prerequisite checking (Git, Rust 1.85.0+, C compiler)
- Binary build orchestration
- Progress tracking with indicatif spinners

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
1. Prompt for server region, asset directory, output directory, content profile (`full` / `operators`)
2. Fetch server version info (client + resource version)
3. Compare with local `.version` file in asset directory
4. If newer: download + extract + save new version

> **Region nesting:** the runner appends the region key to both directories, so a
> `cn` run saves to `./ArkAssets/cn` and extracts to `./output/cn`. This lets
> multiple regions coexist (e.g. full `en` alongside an `operators`-profile `cn`
> preview). The raw `downloader`/`unpacker` binaries do **not** nest — pass
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
- Asset download directory (nested per region — see Mode 2)
- Extraction output directory (nested per region — see Mode 2)
- Content profile (`full` / `operators`) — also via `--profile` / `WS_PROFILE`
- WebSocket port (default 9160)
- Check interval in minutes (default 30)

> Run one instance per region (separate ports/dirs) to monitor several regions at
> once — e.g. a full `en` server on 9160 and an `operators`-profile `cn` preview on 9161.

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
| `--profile <name>` | string | Content profile applied on top of `--all`: `operators` (gamedata + operator-facing assets only) or `full` (everything). Omit for full. |

**Examples:**

```bash
# Download all packs
downloader --server en download --all

# Download specific packs
downloader --server en download --packages pack/chararts,pack/ui

# 8 concurrent downloads to custom directory
downloader --server cn --threads 8 -d /data/arknights download --all

# Operator-only profile: gamedata + operator art/portraits/avatars/icons
# (~2.5 GB for CN vs ~17 GB full). Used for the "upcoming operators" preview.
downloader --server cn download --all --profile operators
```

### Content Profiles

The `operators` profile filters the hot-update file list (by `abInfo` name) down to
only what's needed to display operators, keeping the download small when full-region
storage is impractical:

- **Kept:** `anon/` (all gamedata — ~80 MB, the `.idx` manifest lives here too),
  `chararts/`, `skinpack/`, `spritepack/char_portrait_*`, `spritepack/ui_char_avatar_*`,
  `spritepack/skill_icons_*`, `spritepack/ui_equip_*`, `arts/charportraits/`.
- **Excluded:** `audio/`, `scenes/`, `avg/`, dynamic art, building, etc.

Defined in `downloader/src/profile.rs`. The `.idx` is matched by extension so the
gamedata manifest is always retained.

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
| `--jobs` | `-j` | number | Parallel threads (default: CPU count) |

**Behavior:**

- If **no type flags** are set: extract everything (image + text + audio + spine + portrait + gamedata)
- If **any type flag** is set: extract only those types
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
- `48–56`: ASTC variants (4x4 to 12x12)

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
│   └── DynIllust/
│       └── ...
└── gamedata/
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

**Benefit:** Subsequent runs only download changed packs (typically 100–500 MB vs. 5–10 GB)

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

## WebSocket Protocol Reference

The WebSocket server broadcasts asset pipeline status and accepts client commands.

### Server → Client Events

#### `status`

Sent on connection and when state changes.

```json
{
  "type": "status",
  "state": "idle|checking|downloading|unpacking",
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

Full download + extract cycle finished.

```json
{
  "type": "update_complete",
  "version": "1.27.6_09_01_10_20",
  "downloaded": 145,
  "failed": 5,
  "exported": 2847
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

**Unpacker** (`unpacker/edition 2024`)
- `cli.rs`: Clap CLI (Extract/List commands)
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
  - `mod.rs`: Export module exports
- `flatbuffers_decode.rs`: Generic FlatBuffer schema decoder → JSON
- `generated_fbs/`: FlatBuffer schema code (60+ table definitions)
- `generated_fbs_yostar/`: Yostar (global) variant schemas
- `main.rs`: Entry point + bundle processing pipeline

### UnityFS Bundle Format

**Header** (big-endian, fixed-size):
```
[C-string]  signature ("UnityFS")
[u32]       version (typically 6–7)
[C-string]  player version (e.g., "2021.3.0f1")
[C-string]  engine version
[i64]       file size (bytes)
[u32]       compressed_block_info_size
[u32]       uncompressed_block_info_size
[u32]       data_flags (compression type in bits 0–5, metadata location in bit 7–8)
[alignment] 16-byte align if version >= 7
```

**Block Info** (compressed):
```
[16 bytes]  hash
[i32]       block_count
[Per block]:
  [u32]     uncompressed_size
  [u32]     compressed_size
  [u16]     flags (compression type in bits 0–5)
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

```bash
# Unit tests (requires asset files)
cargo test --lib

# Integration tests (run full pipeline)
# Requires ./ArkAssets populated from downloader
cargo test --test '*'
```

### Formatting

```bash
cargo fmt --all
cargo clippy --lib --bins
```

---

## License

See LICENSE file in repository.

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

CN gamedata is FlatBuffer-encoded; the decoder is generated from community schemas and
committed under `unpacker/src/generated_fbs/` (CN) and `generated_fbs_yostar/` (global).
Regenerate when a CN update breaks a table (symptom: a `gamedata/excel/*.json` file is
invalid UTF-8 / unparseable):

```bash
cd unpacker
cargo run --bin generate-fbs   # requires flatc (see Prerequisites)
cargo build --release
cp target/release/unpacker ../binaries/unpacker   # run.mjs uses ./binaries
```

`generate-fbs`:
1. Clones/pulls CN schemas from [MooncellWiki/OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS) (`main` branch = CN; a separate `YoStar` branch exists for global) into `OpenArknightsFBS/FBS`, and Yostar schemas from [ArknightsAssets/ArknightsFlatbuffers](https://github.com/ArknightsAssets/ArknightsFlatbuffers).
2. Applies `patch_schemas()` fixes — community schemas occasionally have field-order bugs that misalign FlatBuffers VTables. These pass upstream's JSON validation (JSON is unordered) but corrupt binary decoding.
3. Runs `flatc` over every schema and regenerates `fb_json_auto.rs`, `fb_json_auto_yostar.rs`, and `flatbuffers_decode.rs`.

> **Binary sync:** after any `cargo build`, copy the binary into `./binaries/` — `run.mjs`
> and the documented commands invoke `./binaries/{downloader,unpacker}`, not
> `target/release`. The `Setup` runner mode does this for you.

> **Diagnosing a broken table:** dump the raw FlatBuffer (gamedata is `128-byte RSA
> header + FlatBuffer`) and decode with `flatc -t --strict-json --raw-binary schema.fbs
> -- table.fb`. flatc is the canonical oracle: if it also fails, the bug is in the
> *schema* (field order/count), not the decoder. Compare the binary's real VTable field
> count against the schema, then add/adjust a `patch_schemas()` entry. Note that a
> previously-correct patch can go stale — e.g. removing a field that a later CN binary
> starts shipping — so deletion is sometimes the fix.

---

## TODO: Known FBS Extraction Issues

Audit performed 2026-04-10 after fixing `battle_equip_table`, `skin_table`, `stage_table`, and five other tables affected by upstream commit `4975a03` ("Update 2.7.21", Apr 7 2026) in [MooncellWiki/OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS). All backend-loaded tables are now clean; the items below are residual issues that don't currently block functionality but should be addressed.

**Update 2026-06-09 (CN 2.7.41, upstream commit `6121fa9`):** regenerated all schemas against CN 2.7.41. `skin_table` had re-broken — the historical `patch_schemas()` entry that *removed* `spAvatarId`/`spPortraitId` from `clz_Torappu_CharSkinData` was correct at 2.7.21 (binary lacked them) but wrong at 2.7.41 (binary + upstream both ship them at slots 8/10). Deleting that patch restored a clean 20-field decode (verified end-to-end: `DisplaySkin.SkinName` etc. decode correctly). `shop_client_table` self-resolved (upstream caught up). `activity_table` and `open_server_table` remain broken (below) but are still not backend-loaded.

### Non-loaded tables with invalid UTF-8 output

These produce JSON files containing raw binary bytes (from FlatBuffers VTable misalignment where the decoder follows garbage offsets and writes the resulting memory contents into string fields). `serde_json::from_reader` rejects these files. The backend does not currently load them, so the breakage is cosmetic — but any future consumer will hit a parse error.

- [ ] **`activity_table`** — invalid UTF-8 at byte ~3193552 near `"Id"`. The Apr 7 patch removed `defaultEnemyTag` from `clz_Torappu_ActivityEnemyDuelConstData`, which eliminated most panics, but a second misalignment remains in some nested struct. Needs diagnosis: install a panic-attribution hook in `flatbuffers_decode.rs` and re-extract to pinpoint the offending line in `fb_json_auto.rs`, then find the corresponding struct and add a removal/reorder patch to `patch_schemas()`.

- [ ] **`open_server_table`** — invalid UTF-8 at byte ~17160 near `"BindGPGoodId"`. Likely another field in `clz_Torappu_NewbieCheckInPackageData` or a sibling struct that wasn't covered by the `compensateEndDay` removal. Same diagnosis path as above.

### Non-loaded tables with panic noise but valid output

These emit panic messages to stderr during extraction but produce syntactically valid JSON. The filter_map safety net drops the bad elements. Low priority — output is parseable and backend doesn't read them.

- [ ] **`climb_tower_table`** — ~63 panics. The `recordNoResetStartTime` field added in `4975a03` is appended (forward-compatible), so the panics must originate elsewhere. Root cause unknown.
- [ ] **`charm_table`** — ~44 panics, 6 KB output. Not touched by `4975a03`.
- [ ] **`crisis_v2_table`** — ~4 panics, 3 KB output.
- [ ] **`retro_table`** — ~2 panics, 3.8 MB output (99.9% success rate).
- [ ] **`sandbox_perm_table`** — ~1 panic.

### Known noise in already-fixed tables

- [ ] **`battle_equip_table`** — the CN schema path still panics ~867 times on every extraction because `clz_Torappu_EquipTalentData.validModeIndices` doesn't exist in the current CN binary. The `is_content_empty` check in `flatbuffers_decode.rs` detects the resulting `{"Equips": []}` and correctly falls back to the Yostar schema, producing a populated output file. The noise is cosmetic but clutters logs. A cleaner fix would be to remove `validModeIndices` from the CN schema via `patch_schemas()` so the CN path stops panicking in the first place — then the Yostar fallback isn't needed either.

### stage_table truncation workaround

- [ ] **`stage_table.fbs`** has not been updated upstream since game version 2.7.11 (commit `94bf1f8`), but the live game is at 2.7.21. Ten minor versions of schema drift have accumulated. The immediate symptom was `clz_Torappu_CGGalleryGroupData` producing garbage UTF-8 in `LocationId`, which broke `serde_json::from_reader` for the entire file. The current patch truncates `CGGalleryGroupData` to only its first two fields (`storySetId`, `storylineId`), discarding `locationId` and `displays`. This is safe because the backend's `StageTableFile` only reads the top-level `Stages` field and ignores `CgGalleryGroups` entirely, but it means any future consumer that wants CG gallery metadata will see truncated data. A proper fix requires either:
  - A binary inspection of the actual CN game's `CGGalleryGroupData` VTable layout to determine what fields it contains, or
  - Waiting for upstream `OpenArknightsFBS` to publish a `2.7.21` update for `stage_table.fbs`.
- [ ] Re-audit `stage_table`'s other nested structs (`RuneStageGroups`, `OverrideUnlockInfo`, `SixStarRuneData`, `CgGalleryDisplays` — all currently empty in output) to see whether additional truncations are needed once a similar drift appears in a backend-consumed field.

### Follow-up

- [ ] Add an automated audit step to the asset pipeline: after extraction, attempt `serde_json::from_reader` on every `.json` file in `output/gamedata/excel/` and fail the build (or emit a loud warning) if any backend-loaded table produces invalid UTF-8. This would have caught the silent `stage_table` failure immediately instead of letting `load_table_or_warn` quietly fall back to `Default::default()`.
- [ ] Investigate a generalized "strict UTF-8 string accessor" wrapper in the generated `to_json` code that replaces raw bytes with `\uFFFD` (replacement character) when the FlatBuffers string accessor returns non-UTF-8 data, so that downstream JSON parsers don't break when the decoder misreads a field.
- [ ] Periodically `git fetch` upstream `OpenArknightsFBS` and diff against the pinned commit. When new commits land, re-evaluate which `patch_schemas()` entries are still needed (upstream fixes make our patches no-ops, but the dead code accumulates).

---

Last updated: 2026-06-09
