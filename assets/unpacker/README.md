# Assets-Unpacker

A high-performance Rust implementation of an Arknights game asset extractor and unpacker. This tool extracts and processes Unity asset bundles (.ab files) from the mobile game Arknights, converting them into usable formats like PNG images, JSON data, and audio files.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
  - [Extract Command](#extract-command)
  - [Combine Command](#combine-command)
  - [Decode Command](#decode-command)
- [Output Structure](#output-structure)
- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Comparison with Python Version](#comparison-with-python-version)
- [Performance](#performance)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Features

### Asset Extraction
- **Texture2D Extraction**: Converts Unity textures to PNG format
  - Supports all Unity texture formats (DXT1, DXT5, ETC2, ASTC, BC7, etc.)
  - Handles both embedded and streamed texture data (`.resS` files)
  - Automatic alpha channel detection and separation

- **Sprite Extraction**: Extracts individual sprites from sprite atlases
  - Named sprite extraction (e.g., `logo_victoria.png`)
  - Numbered sprite extraction grouped by texture (e.g., `texture$0.png`, `texture$1.png`)
  - Supports tight packing with mesh-based rendering
  - Handles sprite rotation and packing modes

- **Spine Animation Assets**: Organizes Spine 2D animation data
  - Automatic classification into `BattleFront`, `BattleBack`, `Building`, and `DynIllust` categories
  - Uses animation prefix detection to determine asset type
  - Preserves `.skel`, `.atlas`, and texture files in organized directories

- **Audio Extraction**: Converts audio clips to standard formats
  - FMOD audio bank extraction
  - FSB5 format support
  - WAV output format

- **Text Asset Decoding**: Decrypts and decodes game data files
  - AES-CBC decryption for encrypted assets
  - FlatBuffers schema-based decoding for binary data
  - JSON and BSON format support
  - Automatic format detection

### Image Processing
- **RGB+Alpha Combination**: Automatically combines split RGB and alpha textures
  - Detects `[alpha]` suffix textures
  - Merges into single RGBA images
  - Saves to separate `cmb/` directory with `$0.png` suffix

- **Format Conversion**: Comprehensive texture format support
  - Standard formats: RGB24, RGBA32, ARGB32
  - Compressed formats: DXT1, DXT5, BC4, BC5, BC6H, BC7
  - Mobile formats: ETC2_RGB, ETC2_RGBA, ASTC (4x4 to 12x12)
  - Platform-specific format detection

### Organization
- **Structured Output**: Organizes assets into logical directories
  - `upk/` - Unpacked assets (images, sprites, data)
  - `cmb/` - Combined RGB+alpha images
  - `dta/` - Decoded text assets (future)
  - Preserves source directory structure

- **Incremental Extraction**: Skips unchanged files for faster re-extraction
  - MD5 hash-based change detection
  - Manifest file tracking (`.extraction_manifest.json`)
  - `--force` flag to override and re-extract everything

### Performance
- **Parallel Processing**: Multi-threaded extraction using Rayon
  - Processes multiple asset bundles concurrently
  - Efficient CPU utilization
  - Progress bars for long-running operations

- **Memory Efficient**: Streams large files instead of loading entirely into memory
  - Handles multi-gigabyte asset bundles
  - Resource reader for external `.resS` files

## Installation

### Prerequisites

1. **Rust Toolchain** (1.70+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **FMOD Library** (for audio extraction)
   ```bash
   # macOS
   brew install fmod

   # Linux
   # Download from https://www.fmod.com/download
   # Install to /usr/local/lib

   # Windows
   # Download and install FMOD Studio API
   ```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/Assets-Unpacker.git
cd Assets-Unpacker

# Build release version
cargo build --release

# Binary will be at target/release/assets-unpacker
```

### macOS Additional Setup

After building, add the FMOD library path:

```bash
install_name_tool -add_rpath /usr/local/lib target/release/assets-unpacker
```

## Quick Start

```bash
# Extract all assets from a directory
./target/release/assets-unpacker extract \
  --input /path/to/arknights/assets \
  --output ./extracted

# Extract a single asset bundle
./target/release/assets-unpacker extract \
  --input assets/char_002_amiya.ab \
  --output ./output

# Force re-extraction (ignore cache)
./target/release/assets-unpacker extract \
  --input ./assets \
  --output ./output \
  --force

# Combine RGB and alpha textures
./target/release/assets-unpacker combine \
  --input ./extracted/upk \
  --output ./combined
```

## Usage

### Extract Command

The primary command for extracting assets from Unity asset bundles.

```bash
assets-unpacker extract [OPTIONS] --input <PATH> --output <PATH>
```

#### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--input, -i <PATH>` | Input directory or .ab file | Required |
| `--output, -o <PATH>` | Output directory | Required |
| `--image` | Extract textures and sprites | `true` |
| `--text` | Extract text assets | `true` |
| `--audio` | Extract audio clips | `true` |
| `--spine` | Extract and organize Spine assets | `true` |
| `--group` | Group output by source file | `true` |
| `--delete, -d` | Delete output directory before extraction | `false` |
| `--force` | Force re-extraction (ignore cache) | `false` |

#### Examples

```bash
# Extract only images (no text or audio)
assets-unpacker extract \
  --input ./assets \
  --output ./output \
  --text=false \
  --audio=false

# Extract to flat directory (no grouping by source)
assets-unpacker extract \
  --input ./assets \
  --output ./output \
  --group=false

# Clean extraction (delete existing output first)
assets-unpacker extract \
  --input ./assets \
  --output ./output \
  --delete
```

### Combine Command

Combines separate RGB and alpha textures into single RGBA images.

```bash
assets-unpacker combine --input <PATH> --output <PATH>
```

#### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--input, -i <PATH>` | Input directory with separated textures | Required |
| `--output, -o <PATH>` | Output directory for combined images | Required |
| `--delete, -d` | Delete output directory first | `false` |

This is typically used as a post-processing step after extraction, though the extract command now automatically combines RGB+alpha textures during extraction.

### Decode Command

Decodes encrypted and binary text assets (AES, FlatBuffers, BSON).

```bash
assets-unpacker decode --input <PATH> --output <PATH>
```

#### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--input, -i <PATH>` | Input directory with encoded assets | Required |
| `--output, -o <PATH>` | Output directory for decoded JSON | Required |
| `--delete, -d` | Delete output directory first | `false` |

#### Supported Formats

- **AES-CBC Encrypted**: Game data files with RSA signature (128 bytes) + encrypted content
- **FlatBuffers**: Binary schema-based format with 50+ supported tables:
  - `character_table` - Character data and stats
  - `skill_table` - Skill definitions
  - `uniequip_table` - Module/equipment data
  - `activity_table` - Event/activity data
  - `stage_table` - Stage definitions
  - `enemy_database` - Enemy definitions
  - `item_table` - Item data
  - `gacha_table` - Gacha pool data
  - `skin_table` - Skin definitions
  - `building_data` - Base building data
  - `roguelike_topic_table` - Integrated Strategies data
  - And 40+ more tables (auto-detected from filename)
- **BSON**: Binary JSON format
- **JSON**: Plain JSON (passed through)

The decoder gracefully handles schema mismatches - if a file's actual schema differs from what the filename suggests, it returns the raw encrypted/binary data instead of crashing.

#### CN vs Yostar/EN Schema Fallback

Arknights has different game versions (CN, EN/Global, JP, KR) that sometimes have schema differences in their FlatBuffer data. The decoder automatically handles this:

1. **Primary decode**: Tries CN schemas from `OpenArknightsFBS/FBS/`
2. **Fallback decode**: If CN fails, tries Yostar/EN schemas for supported tables:
   - `character_table` - EN has fewer fields than CN
   - `battle_equip_table` - Module/equipment data
   - `token_table` - Summon/token character data
   - `ep_breakbuff_table` - Element break buff data

This ensures EN/Global assets decode correctly even when they differ from the CN version.

## Output Structure

### Directory Layout

```
output/
├── .extraction_manifest.json    # Incremental extraction tracking
├── upk/                          # Unpacked assets
│   ├── client-2.2/
│   │   ├── chararts-char_002_amiya/
│   │   │   ├── BattleFront/                    # Spine category
│   │   │   │   └── char_002_amiya/
│   │   │   │       ├── char_002_amiya.skel
│   │   │   │       ├── char_002_amiya.atlas
│   │   │   │       └── char_002_amiya.png
│   │   │   ├── BattleBack/
│   │   │   ├── Building/
│   │   │   ├── char_002_amiya.png              # Named texture
│   │   │   ├── char_002_amiya[alpha].png       # Separated alpha
│   │   │   └── build_char_002_amiya.png
│   │   │
│   │   └── spritepack-ui_camp_logo_h2_0/
│   │       ├── logo_victoria.png               # Named sprites
│   │       ├── logo_karlan.png
│   │       ├── SpriteAtlasTexture-UI_CAMP_LOGO_H2_0-512x512-fmt47.png
│   │       ├── SpriteAtlasTexture-UI_CAMP_LOGO_H2_0-512x512-fmt47$0.png  # Numbered
│   │       ├── SpriteAtlasTexture-UI_CAMP_LOGO_H2_0-512x512-fmt47$1.png
│   │       └── ...
│   │
│   └── client-2.4/
│       └── ...
│
└── cmb/                          # Combined RGB+alpha images
    ├── client-2.2/
    │   ├── chararts-char_002_amiya/
    │   │   ├── char_002_amiya$0.png           # Combined RGBA
    │   │   └── build_char_002_amiya$0.png
    │   └── ...
    └── ...
```

### File Naming Conventions

- **Regular textures**: `texture_name.png`
- **Alpha textures**: `texture_name[alpha].png`
- **Combined images**: `texture_name$0.png` (in `cmb/` directory)
- **Numbered sprites**: `texture_name$N.png` (N = sprite index)
- **Spine assets**: Organized in `<Type>/<asset_name>/` directories

### Spine Asset Categories

Assets are automatically categorized based on filename and atlas content:

| Category | Detection Logic |
|----------|----------------|
| **Building** | Name starts with `build_` OR atlas contains "Relax" animation |
| **BattleFront** | Atlas has more `\nf_` or `\nc_` prefixes than `\nb_` prefixes |
| **BattleBack** | Atlas has more `\nb_` prefixes than front prefixes |
| **DynIllust** | Name starts with `dyn_` |

## How It Works

### Asset Bundle Processing

1. **Bundle Loading**
   ```
   .ab file → UnityRs Environment → Parse asset objects
   ```

2. **Object Type Detection**
   - Identifies ClassIDType (Texture2D, Sprite, AudioClip, TextAsset, etc.)
   - Routes to appropriate extractor based on type

3. **Texture Extraction**
   ```
   Texture2D object → Check for StreamData
   ├─ Embedded: Use image_data field
   └─ Streamed: Load from .resS file

   → Parse texture format (DXT5, ETC2, etc.)
   → Decode to RGBA
   → Detect alpha texture pairs
   → Save as PNG
   ```

4. **Sprite Extraction**
   ```
   Sprite object → Find atlas (SpriteAtlas or m_AtlasTags)
   → Get render data (texture, rect, settings)
   → Decode atlas texture (cached)
   → Crop to sprite rectangle
   → Apply packing rotation if needed
   → Handle tight packing with mesh
   → Save as PNG
   ```

5. **Spine Asset Organization**
   ```
   TextAsset (.skel/.atlas) → Collect all instances
   → Match skel+atlas pairs by name
   → Detect asset type from atlas content
   → Group textures by type
   → Save to <Type>/<name>/ directory
   ```

### Texture Format Decoding

The tool supports comprehensive texture format decoding through the `unity-rs` library:

```rust
// Example: DXT5 compressed texture
Compressed DXT5 data (4x4 blocks)
→ Decompress blocks to RGBA
→ Flip vertically (Unity bottom-left → standard top-left)
→ Output RGBA8888 image
```

Supported formats include:
- **Uncompressed**: RGB24, RGBA32, ARGB32, RGB565, RGBA4444
- **DXT/BCn**: DXT1, DXT5, BC4, BC5, BC6H, BC7
- **ETC**: ETC_RGB4, ETC2_RGB, ETC2_RGBA
- **ASTC**: 4x4, 5x4, 5x5, 6x5, 6x6, 8x5, 8x6, 8x8, 10x5, 10x6, 10x8, 10x10, 12x10, 12x12
- **PVRTC**: PVRTC_RGB2, PVRTC_RGBA2, PVRTC_RGB4, PVRTC_RGBA4

### Parallel Processing

The tool uses Rayon for parallel processing:

```rust
files.par_iter().for_each(|file| {
    match ab_resolve(file, destdir) {
        Ok(count) => { /* ... */ }
        Err(e) => { /* ... */ }
    }
});
```

This allows processing multiple asset bundles simultaneously, significantly improving performance on multi-core systems.

## Architecture

### Module Structure

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library exports
├── resolve_ab.rs           # Main asset bundle extraction logic
├── decode_textasset.rs     # TextAsset decryption/decoding
├── flatbuffers_decode.rs   # FlatBuffers schema decoding
├── fb_json_macros.rs       # FlatBufferToJson trait definitions
├── fb_json_auto.rs         # Auto-generated JSON serialization (2500+ impls)
├── fb_json_auto_yostar.rs  # Auto-generated JSON for Yostar/EN schemas
├── combine_rgb.rs          # RGB+alpha combination
├── collect_models.rs       # Spine model collection
├── collect_voice.rs        # Audio collection
├── utils.rs                # Utility functions
├── generated_fbs/          # Generated FlatBuffers code (CN schemas)
│   ├── character_table_generated.rs
│   ├── skill_table_generated.rs
│   ├── activity_table_generated.rs
│   └── ... (50+ schema files)
└── generated_fbs_yostar/   # Generated FlatBuffers code (Yostar/EN schemas)
    ├── character_table_generated.rs
    ├── battle_equip_table_generated.rs
    ├── token_table_generated.rs
    └── ep_breakbuff_table_generated.rs
```

### Key Components

#### UnityRs Integration

The project heavily relies on the `unity-rs` library for Unity asset parsing:

```rust
use unity_rs::env::Environment;
use unity_rs::classes::generated::{Texture2D, Sprite, AudioClip};
use unity_rs::export::{texture_2d_converter, sprite_helper};
```

#### Asset Processing Pipeline

```
Input .ab file
    ↓
Environment::load_file()
    ↓
Extract objects by ClassIDType
    ↓
┌─────────────┬──────────────┬───────────────┬──────────────┐
│  Texture2D  │   Sprite     │  AudioClip    │  TextAsset   │
└─────────────┴──────────────┴───────────────┴──────────────┘
      ↓             ↓               ↓               ↓
texture_to_image  sprite_helper  fmod_helper  decrypt/decode
      ↓             ↓               ↓               ↓
   Save PNG      Save PNG        Save WAV       Save JSON
```

#### Caching Strategy

The tool uses multiple caching strategies:

1. **Extraction Manifest**: MD5-based file change detection
2. **Texture Cache**: Decoded textures cached during sprite extraction
3. **Sprite Atlas Cache**: Loaded atlases shared across sprite extraction

## Comparison with Python Version

This Rust implementation is based on the Python [Ark-Unpacker](https://github.com/ArknightsAssets/Ark-Unpacker) project but offers several improvements:

### Performance

| Operation | Python | Rust | Speedup |
|-----------|--------|------|---------|
| Full extraction (31 files) | ~45s | ~5s | ~9x faster |
| Single large bundle | ~8s | ~1s | ~8x faster |
| Incremental (unchanged) | ~15s | <1s | ~15x+ faster |

*Benchmarks on M1 MacBook Pro, 31 test asset bundles*

### Feature Parity

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Texture extraction | ✅ | ✅ | Full parity |
| Alpha texture handling | ✅ | ✅ | Full parity |
| Sprite extraction | ✅ | ✅ | Full parity |
| Numbered sprites | ✅ | ✅ | Full parity |
| Spine organization | ✅ | ✅ | Full parity |
| RGB+alpha combination | ✅ | ✅ | Full parity |
| Audio extraction | ✅ | ✅ | Full parity |
| TextAsset decryption | ✅ | ✅ | Full parity |
| FlatBuffers decoding | ⚠️ | ✅ | Rust has schema-based decoding |
| Incremental extraction | ❌ | ✅ | Rust-only feature |

### Advantages

**Rust**:
- 8-10x faster extraction
- Lower memory usage (streaming vs loading entire files)
- Incremental extraction with hash-based change detection
- Type-safe schema-based FlatBuffers decoding
- Better error handling and recovery
- No Python runtime dependency

**Python**:
- More mature (longer development time)
- Easier to modify for quick experiments
- Larger ecosystem of image processing libraries

### Output Compatibility

The Rust version produces **identical** output to the Python version:
- Same directory structure (`upk/`, `cmb/`)
- Same file naming conventions
- Same Spine asset organization
- Byte-for-byte identical PNG outputs (same compression)

Differences:
- Rust may extract more numbered sprites (finds additional variations)
- Rust includes `.extraction_manifest.json` for incremental extraction

## Performance

### Benchmarks

Test environment: M1 MacBook Pro, 16GB RAM

```
Dataset: 31 Arknights asset bundles (2.2GB total)
CPU cores used: 8 (parallel processing)

First extraction:    ~5 seconds
Incremental:         <1 second (all files unchanged)
Single large bundle: ~1 second (6MB .ab file)
```

### Memory Usage

- **Python**: ~800MB peak (loads entire bundles into memory)
- **Rust**: ~250MB peak (streaming processing)

### Optimization Tips

1. **Use SSD storage**: I/O is often the bottleneck
2. **Parallel extraction**: Let the tool use all CPU cores (default behavior)
3. **Incremental mode**: Use manifest-based skipping for repeated extractions
4. **Disable unused extractors**: Use `--image=false` etc. to skip unneeded assets

## Development

### Building for Development

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Run with debug logging
RUST_LOG=debug cargo run -- extract --input ./assets --output ./out

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

### Project Structure

```
Assets-Unpacker/
├── Cargo.toml           # Rust dependencies
├── src/                 # Source code
├── tests/               # Test resources
├── scripts/             # Utility scripts
│   └── compare_outputs.py
├── OpenArknightsFBS/    # FlatBuffers schemas
└── README.md
```

### Adding New FlatBuffers Schemas

The project uses auto-generated JSON serialization for all FlatBuffer types. To add a new schema:

1. Add `.fbs` schema to `OpenArknightsFBS/FBS/`
2. Run the regeneration script:
   ```bash
   ./regenerate_fbs.sh
   ```
   This will:
   - Generate Rust FlatBuffer code with `flatc` (CN schemas)
   - Auto-generate `FlatBufferToJson` implementations via `generate_fb_json_impls.py`
   - Clone/update ArknightsFlatbuffers repo for Yostar schemas
   - Generate Yostar/EN schema variants (if different from CN)
   - Auto-generate Yostar `FlatBufferToJson` implementations via `generate_fb_json_impls_yostar.py`
   - Update `mod.rs` files with all schema modules

3. Add filename detection in `flatbuffers_decode.rs`:
   ```rust
   // In guess_root_type() function
   "new_table" => Some("clz_Torappu_NewTableBundle"),
   ```

4. (Optional) If adding a Yostar/EN fallback schema:
   - Add the schema name to the `YOSTAR_SCHEMAS` array in `regenerate_fbs.sh`
   - Add to `has_yostar_schema()` in `flatbuffers_decode.rs`
   - Add to `decode_flatbuffer_yostar()` match block

5. Rebuild:
   ```bash
   cargo build --release
   ```

#### How Auto-Generation Works

The `generate_fb_json_impls.py` script (and `generate_fb_json_impls_yostar.py` for Yostar):
- Parses all `*_generated.rs` files in `src/generated_fbs/` (or `src/generated_fbs_yostar/`)
- Extracts struct definitions (`clz_*`, `dict__*`, `list_*`, `kvp__*`)
- Generates `FlatBufferToJson` trait implementations for each type
- Handles nested types, vectors, enums, and optional fields
- Outputs to `src/fb_json_auto.rs` (2500+ implementations, 480+ enums)
- Yostar outputs to `src/fb_json_auto_yostar.rs` (subset for EN-specific schemas)

### Dependencies

Key dependencies:
- **unity-rs**: Unity asset parsing (local path dependency)
- **image**: Image encoding/decoding
- **rayon**: Parallel processing
- **serde_json**: JSON serialization
- **aes**, **cbc**: AES-CBC decryption
- **flatbuffers**: FlatBuffers support
- **clap**: Command-line parsing

## Troubleshooting

### Common Issues

#### "FMOD library not found"

**macOS:**
```bash
brew install fmod
install_name_tool -add_rpath /usr/local/lib target/release/assets-unpacker
```

**Linux:**
```bash
# Download from https://www.fmod.com/download
sudo cp libfmod.so.13 /usr/local/lib/
sudo ldconfig
```

#### "Texture has no image data"

This usually means:
1. The texture uses StreamData (`.resS` file) that's not available
2. The `.resS` file is in a different location

**Solution**: Ensure `.resS` files are in the same directory as `.ab` files.

#### "Failed to read sprite atlas"

The sprite might be using:
- A newer Unity version with different serialization
- A custom atlas format

**Workaround**: Extract textures only with `--spine=false`

#### Incremental extraction not working

Force re-extraction:
```bash
assets-unpacker extract --input ./assets --output ./out --force
```

Or delete the manifest:
```bash
rm output/.extraction_manifest.json
```

### Debug Logging

Enable detailed logging to diagnose issues:

```bash
# All debug output
RUST_LOG=debug assets-unpacker extract -i ./assets -o ./out

# Specific module only
RUST_LOG=assets_unpacker::resolve_ab=debug assets-unpacker extract -i ./assets -o ./out

# Trace level (very verbose)
RUST_LOG=trace assets-unpacker extract -i ./assets -o ./out
```

### Performance Issues

If extraction is slow:

1. **Check disk I/O**: Use `iostat` or Activity Monitor
2. **Verify parallel processing**: Should use all CPU cores
3. **Check memory**: Ensure enough RAM available
4. **Disable unnecessary extraction**:
   ```bash
   # Images only
   assets-unpacker extract -i ./assets -o ./out --audio=false --text=false
   ```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Add doc comments for public APIs
- Include examples in doc comments
- Keep functions focused and small
- Use descriptive variable names

## License

[Insert your license here - e.g., MIT, Apache 2.0, GPL]

## Acknowledgments

- **Ark-Unpacker**: Original Python implementation that this project is based on
- **UnityPy**: Python Unity asset parser that inspired unity-rs
- **unity-rs**: Rust Unity asset parsing library
- **Arknights**: Game assets belong to Hypergryph/Yostar

## Related Projects

- [Ark-Unpacker](https://github.com/ArknightsAssets/Ark-Unpacker) - Original Python implementation
- [unity-rs](https://github.com/yourusername/unity-rs) - Rust Unity asset parser
- [AssetStudio](https://github.com/Perfare/AssetStudio) - Unity asset extraction tool (C#)
- [UnityPy](https://github.com/K0lb3/UnityPy) - Python Unity asset parser

## Support

For questions, issues, or feature requests:
- Open an issue on GitHub
- Check existing issues for solutions
- Enable debug logging and include output with bug reports

---

**Note**: This tool is for educational and archival purposes only. All game assets remain the property of their respective copyright holders.
