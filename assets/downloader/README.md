# Arknights Asset Downloader

A high-performance Rust implementation for downloading and unpacking Arknights game assets. This tool handles downloading, unpacking, and version management of game assets with production-grade reliability.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
  - [Basic Usage](#basic-usage)
  - [Advanced Usage](#advanced-usage)
  - [Version Checking](#version-checking)
- [Command Line Interface](#command-line-interface)
- [Architecture](#architecture)
- [Asset Processing](#asset-processing)
- [Production Deployment](#production-deployment)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

### Core Features

| Feature | Description |
|---------|-------------|
| Multi-Server Support | Official (Global) and Bilibili (CN) servers |
| Smart Caching | MD5-based duplicate detection and skip |
| Parallel Processing | Concurrent downloads, unzipping, and unpacking |
| Asset Conversion | Automatic conversion to usable formats |
| Format Support | Images (PNG), Audio (WAV/OGG), Text (JSON), Fonts (TTF/OTF), Meshes (OBJ) |
| Decryption | Built-in AES decryption for game data |
| Progress Tracking | Real-time progress bars and statistics |
| Selective Download | Interactive package selection |
| Version Monitoring | Automatic update detection and tracking |
| Production Ready | Designed for CI/CD, cron jobs, and automation |

### Asset Processing Capabilities

#### Supported Asset Types

| Asset Type | Output Format | Features |
|------------|--------------|----------|
| Texture2D | PNG | Alpha channel merging, Sprite atlas support |
| Sprite | PNG | Alpha channel support, Proper coordinate handling |
| AudioClip | WAV/OGG/etc | All Unity audio formats, Named sample extraction |
| TextAsset | JSON/TXT | AES decryption, BSON support, Format auto-detection |
| Font | TTF/OTF | TrueType and OpenType, Auto format detection |
| Mesh | OBJ | 3D model export, Wavefront OBJ format |

## Installation

### Prerequisites

- **Rust** 1.70+ (stable)
- **Cargo** (comes with Rust)
- **Internet connection** (for downloading assets)

### Install Rust

```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
# Download and run: https://rustup.rs/
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Eltik/arknights-downloader.git
cd arknights-downloader

# Build release binary
cargo build --release

# Binary will be at: ./target/release/arknights-downloader
```

### Development Build

```bash
# Faster compilation, slower runtime
cargo build

# Run without building binary
cargo run -- --help
```

## Quick Start

### Basic Download (All Assets)

```bash
# Download all assets from official server
cargo run --release -- --server official --savedir ./ArkAssets

# Download from Bilibili server
cargo run --release -- --server bilibili --savedir ./ArkAssets
```

### Selective Download

```bash
# Download specific packages (non-interactive)
cargo run --release -- --server official \
  --savedir ./ArkAssets \
  --packages "gamedata/excel,gamedata/levels,arts/characters"
```

### Interactive Mode

```bash
# Launch interactive package selector
cargo run --release -- --server official --savedir ./ArkAssets

# Follow prompts:
# - A = Download All
# - C = Custom Selection (use arrow keys)
# - Other = Cancel
```

### Check for Updates

```bash
# Run the version checker
cargo run --release --example check_updates

# Automated monitoring
cargo run --release --example automated_monitor
```

## Usage

### Basic Usage

#### 1. Download All Assets

```rust
use arknights_downloader::downloader::{ArkAssets, Servers};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize downloader
    let assets = ArkAssets::new(Servers::OFFICIAL)?;

    // Download everything (interactive mode)
    assets.download("./ArkAssets")?;

    Ok(())
}
```

#### 2. Download Specific Packages

```rust
use arknights_downloader::downloader::{ArkAssets, Servers};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = ArkAssets::new(Servers::OFFICIAL)?;

    // Download only character art and game data
    let packages = vec![
        "arts/characters".to_string(),
        "gamedata/excel".to_string(),
    ];

    assets.download_fromlist(&packages, "./ArkAssets", 6)?;

    Ok(())
}
```

### Advanced Usage

#### Automatic Update Detection and Download

```rust
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

fn auto_update() -> Result<(), Box<dyn std::error::Error>> {
    let server = Servers::OFFICIAL;
    let savedir = "./ArkAssets";

    // Check for updates
    match ArkAssets::check_for_updates(server, savedir)? {
        UpdateStatus::NoUpdate => {
            println!("Already up to date!");
        }
        UpdateStatus::ResourceUpdate { old_version, new_version } => {
            println!("Update detected: {} -> {}", old_version, new_version);

            // Automatically download new assets
            let assets = ArkAssets::new(server)?;
            let all_packages: Vec<String> = assets
                .hot_update_list
                .keys()
                .cloned()
                .collect();

            assets.download_fromlist(&all_packages, savedir, 6)?;
            println!("Update complete!");
        }
        _ => {}
    }

    Ok(())
}
```

#### Custom Error Handling

```rust
use arknights_downloader::downloader::{ArkAssets, Servers};
use std::process;

fn main() {
    let assets = match ArkAssets::new(Servers::OFFICIAL) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to initialize: {}", e);
            eprintln!("Possible causes:");
            eprintln!("  - No internet connection");
            eprintln!("  - Arknights servers down");
            eprintln!("  - Firewall blocking requests");
            process::exit(1);
        }
    };

    if let Err(e) = assets.download("./ArkAssets") {
        eprintln!("Download failed: {}", e);
        // Implement retry logic, logging, notifications, etc.
        process::exit(1);
    }
}
```

### Version Checking

#### Simple Version Check

```rust
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status = ArkAssets::check_for_updates(Servers::OFFICIAL, "./ArkAssets")?;

    match status {
        UpdateStatus::NoUpdate => println!("Up to date"),
        UpdateStatus::FirstCheck => println!("Version baseline created"),
        UpdateStatus::ResourceUpdate { new_version, .. } => {
            println!("New resource version: {}", new_version);
        }
        UpdateStatus::BothUpdated { new_res_version, new_client_version, .. } => {
            println!("Major update!");
            println!("  Resources: {}", new_res_version);
            println!("  Client: {}", new_client_version);
        }
        _ => {}
    }

    Ok(())
}
```

#### Load Cached Version Info

```rust
use arknights_downloader::downloader::{ArkAssets, Servers};

if let Some(version) = ArkAssets::load_version_cache(Servers::OFFICIAL, "./ArkAssets") {
    println!("Current version: {}", version.resVersion);
    println!("Client version: {}", version.clientVersion);
    println!("Last checked: {}", version.lastChecked);
} else {
    println!("No cached version - run check_for_updates() first");
}
```

## Command Line Interface

### Synopsis

```bash
arknights-downloader [OPTIONS]
```

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--server` | `-s` | String | `official` | Server to download from (`official` or `bilibili`) |
| `--savedir` | `-d` | String | `./ArkAssets` | Directory to save downloaded assets |
| `--packages` | `-p` | String | None | Comma-separated list of packages to download |

### Examples

```bash
# Default: Interactive mode, official server
cargo run --release

# Bilibili server
cargo run --release -- --server bilibili

# Custom save directory
cargo run --release -- --savedir /data/arknights

# Non-interactive: specific packages
cargo run --release -- --packages "gamedata/excel,arts/characters"

# Combination
cargo run --release -- \
  --server bilibili \
  --savedir /mnt/storage/ark \
  --packages "gamedata/excel,gamedata/levels"
```

## Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     ArkAssets Instance                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Version    │  │  Hot Update  │  │   Package    │      │
│  │   Manager    │  │     List     │  │   Metadata   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
          ┌───────────────────────────────────────┐
          │     Download Orchestration Engine     │
          │  (Rayon Parallel Iterator)            │
          └───────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           ▼                  ▼                  ▼
    ┌───────────┐      ┌───────────┐     ┌───────────┐
    │ Download  │      │ Download  │     │ Download  │
    │ Thread 1  │      │ Thread 2  │ ... │ Thread N  │
    └───────────┘      └───────────┘     └───────────┘
           │                  │                  │
           └──────────────────┼──────────────────┘
                              ▼
                   ┌─────────────────────┐
                   │   Unzip Thread Pool  │
                   │  (Spawned Threads)   │
                   └─────────────────────┘
                              │
                              ▼
                   ┌─────────────────────┐
                   │  Unpack Thread Pool  │
                   │  (Unity Processing)  │
                   └─────────────────────┘
                              │
                              ▼
                   ┌─────────────────────┐
                   │   Asset Conversion   │
                   │  (PNG/JSON/OBJ/etc)  │
                   └─────────────────────┘
```

### Thread Model

1. **Main Thread**: Orchestration and UI
2. **Download Threads**: Rayon parallel pool (configurable size)
3. **Unzip Threads**: Spawned per download (fire-and-forget → joined)
4. **Unpack Threads**: Nested spawns for asset extraction

### Data Flow

```
Server Request → Download → Cache Check → Unzip → Unpack → Convert → Save
     │                            │
     └─ MD5 Hash                  └─ Skip if exists
```

### Key Components

#### 1. Version Manager (`src/downloader.rs:167-281`)

```rust
// Fetches and caches version information
get_version() -> (resVersion, clientVersion)
check_for_updates() -> UpdateStatus
load_version_cache() -> Option<VersionInfo>
save_version_cache() -> Result<()>
```

#### 2. Hot Update List (`src/downloader.rs:283-384`)

Fetches package metadata from CDN:
- Package names and sizes
- File MD5 hashes
- Download URLs
- Total resource size

#### 3. Download Manager (`src/downloader.rs:570-1102`)

- Parallel downloads with Rayon
- Progress tracking with indicatif
- Automatic retry on failure
- Thread-safe state management

#### 4. Asset Unpacker (`src/downloader.rs:741-1002`)

Uses `unity-rs` library to:
- Parse Unity asset bundles
- Extract objects by ClassID
- Convert to standard formats
- Handle encryption/compression

## Asset Processing

### Processing Pipeline

Each downloaded `.dat` file goes through:

1. **Download** → Binary data from CDN
2. **Unzip** → Extract Unity bundle from ZIP
3. **Parse** → Load Unity asset file structure
4. **Extract** → Process each object by type
5. **Convert** → Transform to usable format
6. **Save** → Write to `[unpack]<filename>` directory

### Asset Type Processing

#### Texture2D & Sprites

```
Input:  Texture2D object in Unity bundle
Output: PNG file with merged alpha channel

Process:
1. Extract RGB data
2. Extract alpha channel (if separate)
3. Merge alpha into RGBA
4. Encode as PNG
5. Save to [unpack]<bundle>/(<Type>)<Name>.png
```

**Example Output:**
```
gamedata/excel/character_table.ab
└── [unpack]character_table/
    ├── (Texture2D)char_002_amiya.png
    ├── (Sprite)char_002_amiya_portrait.png
    └── ...
```

#### TextAssets

```
Input:  TextAsset with encrypted/compressed data
Output: JSON or TXT file

Process:
1. Extract byte stream
2. Decrypt if necessary (AES-128-CBC)
3. Try parse as JSON
4. If JSON fails, try BSON
5. If both fail, save as raw .txt
6. Save with proper extension
```

**Encryption Handling:**
- Level data: RSA + AES decryption
- Game data: AES decryption only
- Enemy data: No decryption

**Example Output:**
```
gamedata/excel/character_table.ab
└── [unpack]character_table/
    └── character_table.json  (decrypted and formatted)
```

#### AudioClips

```
Input:  AudioClip object with embedded audio
Output: WAV/OGG/MP3 (format depends on source)

Process:
1. Extract audio samples
2. Detect format (WAV, OGG, FSB, etc.)
3. Write raw audio data
4. Name by clip name
```

#### Fonts

```
Input:  Font object with embedded font data
Output: TTF or OTF file

Process:
1. Extract font byte data
2. Check header for format (OTTO = OTF, else TTF)
3. Write with appropriate extension
```

#### Meshes

```
Input:  Mesh object with vertices/faces
Output: Wavefront OBJ file

Process:
1. Extract vertex positions
2. Extract normals and UVs
3. Extract face indices
4. Format as OBJ text
5. Save as .obj
```

### Alpha Channel Merging

Arknights stores some textures with separate alpha channels:

```
Example:
  - char_002_amiya        (RGB data)
  - char_002_amiya[alpha] (Alpha channel)

Process:
  1. Load both textures
  2. Combine RGB + Alpha → RGBA
  3. Save merged result
  4. Delete temporary files
```

This ensures transparency is properly preserved in character sprites and UI elements.

## Production Deployment

### Use Cases

- **Automated Data Mining**: Extract game data for wikis/databases
- **Analytics**: Track asset changes over time
- **Game Tools**: Build character viewers, team builders
- **Update Notifications**: Alert users when new content drops
- **Asset Archival**: Maintain historical records of game assets
- **Research**: Study game design and asset structure

### Deployment Patterns

#### 1. Cron Job (Scheduled Updates)

```bash
#!/bin/bash
# /etc/cron.d/arknights-updater
# Check for updates every hour

0 * * * * /usr/local/bin/ark-update.sh >> /var/log/arknights-updater.log 2>&1
```

**ark-update.sh:**
```bash
#!/bin/bash
set -euo pipefail

cd /opt/arknights-downloader

# Check for updates
/usr/local/bin/cargo run --release --example automated_monitor
EXIT_CODE=$?

# Exit codes:
# 0  = No updates
# 10 = Resource update
# 11 = Client update
# 12 = Both updated

if [ $EXIT_CODE -eq 10 ] || [ $EXIT_CODE -eq 12 ]; then
    echo "[$(date)] Update detected - triggering download"

    # Optional: Send notification
    curl -X POST https://discord.com/api/webhooks/YOUR_WEBHOOK \
      -H "Content-Type: application/json" \
      -d '{"content": "Arknights update detected - download starting"}'

    # Download will happen automatically if ARK_AUTO_DOWNLOAD=true
fi
```

#### 2. Docker Container

**Dockerfile:**
```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release --example automated_monitor

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/examples/automated_monitor /usr/local/bin/

# Create data directory
RUN mkdir -p /data/arknights

# Environment variables
ENV ARK_SERVER=official
ENV ARK_SAVEDIR=/data/arknights
ENV ARK_AUTO_DOWNLOAD=true

# Run checker every 30 minutes
CMD while true; do \
    /usr/local/bin/automated_monitor; \
    sleep 1800; \
done
```

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  arknights-monitor:
    build: .
    container_name: arknights-updater
    restart: unless-stopped
    volumes:
      - ./data:/data/arknights
    environment:
      - ARK_SERVER=official
      - ARK_AUTO_DOWNLOAD=true
      - RUST_LOG=info
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

**Run:**
```bash
docker-compose up -d
docker-compose logs -f arknights-monitor
```

#### 3. Kubernetes Deployment

**k8s/deployment.yaml:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: arknights-monitor
spec:
  replicas: 1
  selector:
    matchLabels:
      app: arknights-monitor
  template:
    metadata:
      labels:
        app: arknights-monitor
    spec:
      containers:
      - name: monitor
        image: your-registry/arknights-downloader:latest
        env:
        - name: ARK_SERVER
          value: "official"
        - name: ARK_AUTO_DOWNLOAD
          value: "true"
        - name: ARK_SAVEDIR
          value: "/data"
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: arknights-data-pvc
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: arknights-data-pvc
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 50Gi
```

#### 4. GitHub Actions (CI/CD)

**.github/workflows/monitor.yml:**
```yaml
name: Arknights Asset Monitor

on:
  schedule:
    # Run every hour
    - cron: '0 * * * *'
  workflow_dispatch:

jobs:
  check-updates:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Cache Cargo
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/bin/
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Check for updates
      id: check
      run: |
        cargo run --release --example automated_monitor
        echo "exit_code=$?" >> $GITHUB_OUTPUT
      env:
        ARK_AUTO_DOWNLOAD: false

    - name: Notify Discord
      if: steps.check.outputs.exit_code == '10' || steps.check.outputs.exit_code == '12'
      run: |
        curl -X POST ${{ secrets.DISCORD_WEBHOOK }} \
          -H "Content-Type: application/json" \
          -d '{"content": "Arknights update detected!"}'

    - name: Upload assets (if updated)
      if: steps.check.outputs.exit_code == '10' || steps.check.outputs.exit_code == '12'
      uses: actions/upload-artifact@v3
      with:
        name: arknights-assets
        path: ./ArkAssets
```

#### 5. AWS Lambda (Serverless)

**handler.rs:**
```rust
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

async fn handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    let status = ArkAssets::check_for_updates(Servers::OFFICIAL, "/tmp")?;

    match status {
        UpdateStatus::ResourceUpdate { new_version, .. } => {
            // Upload to S3, send SNS notification, etc.
            Ok(serde_json::json!({
                "statusCode": 200,
                "body": format!("Update detected: {}", new_version)
            }))
        }
        UpdateStatus::NoUpdate => {
            Ok(serde_json::json!({
                "statusCode": 200,
                "body": "No updates"
            }))
        }
        _ => Ok(serde_json::json!({"statusCode": 200, "body": "OK"}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(handler)).await
}
```

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `ARK_SERVER` | Server to use | `official` | `bilibili` |
| `ARK_SAVEDIR` | Save directory | `./ArkAssets` | `/data/arknights` |
| `ARK_AUTO_DOWNLOAD` | Auto-download on update | `false` | `true` |
| `RUST_LOG` | Logging level | - | `info`, `debug` |

### Monitoring and Alerting

#### Exit Codes (automated_monitor example)

| Code | Meaning | Action |
|------|---------|--------|
| `0` | No updates / Success | Continue monitoring |
| `1` | Error occurred | Check logs, send alert |
| `2` | First check | Baseline created |
| `10` | Resource update | Trigger download |
| `11` | Client update | Information only |
| `12` | Both updated | Trigger full download |

#### Health Check Endpoint

```rust
use warp::Filter;
use arknights_downloader::downloader::{ArkAssets, Servers};

#[tokio::main]
async fn main() {
    let health = warp::path("health")
        .map(|| {
            match ArkAssets::load_version_cache(Servers::OFFICIAL, "./data") {
                Some(version) => {
                    warp::reply::json(&serde_json::json!({
                        "status": "healthy",
                        "version": version.resVersion,
                        "lastChecked": version.lastChecked
                    }))
                }
                None => {
                    warp::reply::json(&serde_json::json!({
                        "status": "unknown",
                        "error": "No version cache"
                    }))
                }
            }
        });

    warp::serve(health).run(([0, 0, 0, 0], 8080)).await;
}
```

### Performance Tuning

#### Thread Pool Size

```rust
// Default: 6 threads (configurable)
assets.download_fromlist(&packages, savedir, 6)?;

// More threads = faster downloads (if bandwidth allows)
assets.download_fromlist(&packages, savedir, 12)?;

// Fewer threads = lower resource usage
assets.download_fromlist(&packages, savedir, 3)?;
```

**Recommendations:**
- **4-8 cores**: 6 threads
- **8-16 cores**: 12 threads
- **16+ cores**: 16+ threads
- **Bandwidth limited**: 4 threads

#### Memory Usage

**Typical usage:**
- Base: ~100 MB
- Per download thread: ~50-100 MB
- Peak during unpack: ~500 MB - 1 GB

**For constrained environments:**
```rust
// Reduce thread count
assets.download_fromlist(&packages, savedir, 2)?;

// Download in batches
for batch in packages.chunks(10) {
    assets.download_fromlist(batch, savedir, 4)?;
}
```

## API Reference

### ArkAssets

Main struct for asset management.

#### Constructor

```rust
pub fn new(server: Servers) -> Result<Self, Box<dyn std::error::Error>>
```

Creates a new `ArkAssets` instance by fetching version and package metadata.

**Parameters:**
- `server`: Server to use (`Servers::OFFICIAL` or `Servers::BILIBILI`)

**Returns:**
- `Ok(ArkAssets)`: Successfully initialized
- `Err(...)`: Network error, server error, or parsing error

**Example:**
```rust
let assets = ArkAssets::new(Servers::OFFICIAL)?;
```

#### download()

```rust
pub fn download(&self, savedir: &str) -> Result<(), Box<dyn std::error::Error>>
```

Interactive download mode with package selection.

**Parameters:**
- `savedir`: Directory to save assets

**Flow:**
1. Display all available packages with sizes
2. Prompt user: Download All / Custom Selection / Cancel
3. If custom: Interactive arrow-key selection
4. Download selected packages

**Example:**
```rust
assets.download("./ArkAssets")?;
```

#### download_fromlist()

```rust
pub fn download_fromlist(
    &self,
    keys: &[String],
    savedir: &str,
    threading_count: usize,
) -> Result<(), Box<dyn std::error::Error>>
```

Download specific packages (non-interactive).

**Parameters:**
- `keys`: Package names to download
- `savedir`: Directory to save assets
- `threading_count`: Number of download threads (recommended: 4-12)

**Returns:**
- `Ok(())`: All downloads completed successfully
- `Err(...)`: Download or processing error

**Example:**
```rust
let packages = vec!["gamedata/excel".to_string()];
assets.download_fromlist(&packages, "./ArkAssets", 6)?;
```

#### check_for_updates()

```rust
pub fn check_for_updates(
    server: Servers,
    savedir: &str,
) -> Result<UpdateStatus, Box<dyn std::error::Error>>
```

Check for version updates.

**Parameters:**
- `server`: Server to check
- `savedir`: Directory containing version cache

**Returns:**
- `Ok(UpdateStatus)`: Update status enum
- `Err(...)`: Network error or cache error

**Example:**
```rust
match ArkAssets::check_for_updates(Servers::OFFICIAL, "./data")? {
    UpdateStatus::ResourceUpdate { new_version, .. } => {
        println!("New version: {}", new_version);
    }
    _ => {}
}
```

#### load_version_cache()

```rust
pub fn load_version_cache(server: Servers, savedir: &str) -> Option<VersionInfo>
```

Load cached version information.

**Parameters:**
- `server`: Server to load cache for
- `savedir`: Directory containing cache

**Returns:**
- `Some(VersionInfo)`: Cached version data
- `None`: No cache exists or parsing failed

**Example:**
```rust
if let Some(version) = ArkAssets::load_version_cache(Servers::OFFICIAL, "./data") {
    println!("Current version: {}", version.resVersion);
}
```

#### save_version_cache()

```rust
pub fn save_version_cache(
    server: Servers,
    savedir: &str,
    res_version: &str,
    client_version: &str,
) -> Result<(), Box<dyn std::error::Error>>
```

Manually save version cache (rarely needed - `check_for_updates` does this automatically).

### Enums

#### Servers

```rust
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Servers {
    OFFICIAL = 0,  // Global server
    BILIBILI = 1,  // CN server
}
```

#### UpdateStatus

```rust
pub enum UpdateStatus {
    NoUpdate,
    FirstCheck,
    ResourceUpdate { old_version: String, new_version: String },
    ClientUpdate { old_version: String, new_version: String },
    BothUpdated {
        old_res_version: String,
        new_res_version: String,
        old_client_version: String,
        new_client_version: String
    },
}
```

### Structs

#### VersionInfo

```rust
pub struct VersionInfo {
    pub resVersion: String,      // Resource version (e.g., "24-02-02-10-18-07-831840")
    pub clientVersion: String,   // Client version (e.g., "1.9.51")
    pub lastChecked: String,     // ISO 8601 timestamp
}
```

## Examples

The project includes comprehensive examples in the `examples/` directory:

### check_updates.rs

Simple version checking with friendly output.

```bash
cargo run --example check_updates
```

**Output:**
```
=== Arknights Version Update Checker ===

Checking for updates...

No updates available - you're on the latest version!

Current Versions:
  Resource Version: 24-02-02-10-18-07-831840
  Client Version: 1.9.51
  Last Checked: 2024-02-15T14:30:00Z

=== Version check complete ===
```

### automated_monitor.rs

Production-ready monitoring with exit codes.

```bash
# Basic usage
cargo run --example automated_monitor

# With auto-download
ARK_AUTO_DOWNLOAD=true cargo run --example automated_monitor

# Custom server and directory
ARK_SERVER=bilibili ARK_SAVEDIR=/data/ark cargo run --example automated_monitor
```

**Output:**
```
[Monitor] Arknights Update Monitor Starting...
[Monitor] Server: OFFICIAL
[Monitor] Save Directory: ./ArkAssets
[Monitor] Auto-download: true

[Monitor] RESOURCE UPDATE DETECTED!
[Monitor]   Old: 24-02-02-10-18-07-831840
[Monitor]   New: 24-02-09-15-22-33-941721
[Monitor] Auto-download enabled - starting download...
[Download] Initializing asset downloader...
[Download] Download completed successfully!
```

## Performance

### Benchmarks

Hardware: MacBook Pro M1 Max, 64GB RAM, 1Gbps connection

| Operation | Time | Notes |
|-----------|------|-------|
| Version check | ~200ms | Single HTTP request |
| Package list fetch | ~500ms | JSON parsing |
| Download 1GB assets | ~15s | 6 threads, ~67 MB/s |
| Full download (20GB) | ~5-8 min | Parallel processing |
| Unzip 1000 files | ~30s | Concurrent unzipping |
| Unpack 1000 bundles | ~2-3 min | Unity parsing + conversion |

### Optimization Tips

1. **Increase threads** if you have bandwidth and CPU cores
2. **Use SSD** for faster file I/O during unpacking
3. **Batch downloads** if memory constrained
4. **Skip existing files** (automatic with MD5 check)
5. **Use release builds** (`--release`) for 10-20x speedup

### Resource Usage

| Metric | Typical | Peak |
|--------|---------|------|
| CPU | 50-80% (multi-core) | 100% |
| Memory | 500 MB - 1 GB | 2 GB |
| Disk I/O | 100-200 MB/s | 500 MB/s |
| Network | 50-100 MB/s | Bandwidth limit |

## Troubleshooting

### Common Issues

#### "Failed to initialize ArkAssets"

**Cause:** Cannot fetch version from server.

**Solutions:**
- Check internet connection
- Verify Arknights servers are up
- Check firewall/proxy settings
- Try different server: `--server bilibili`

#### "Download failed: Connection refused"

**Cause:** CDN is blocking requests.

**Solutions:**
- Use VPN (for region-locked content)
- Wait and retry later
- Check if CDN URL has changed

#### "Unzip error" or "Unpack error"

**Cause:** Corrupted download or incompatible Unity version.

**Solutions:**
- Delete corrupted file and retry
- Update unity-rs dependency
- Check disk space
- Verify file MD5 hash

#### Downloads hang at 100%

**Cause:** This was a threading issue - fixed in latest version.

**Solution:** Update to latest version.

#### "No such file or directory" errors

**Cause:** Save directory doesn't exist.

**Solution:**
```bash
mkdir -p ./ArkAssets
cargo run --release -- --savedir ./ArkAssets
```

#### High memory usage

**Cause:** Too many concurrent operations.

**Solutions:**
- Reduce thread count: `download_fromlist(..., 4)`
- Download in batches
- Close other applications

#### Version cache always shows "FirstCheck"

**Cause:** Cache file not being created or wrong path.

**Solutions:**
- Check `{savedir}/version_cache_official.json` exists
- Verify write permissions
- Check `savedir` path is correct

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run --release
```

This will show:
- HTTP requests
- File operations
- Thread spawning
- Error stack traces

## Development

### Project Structure

```
arknights-downloader/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library root
│   ├── downloader.rs     # Core download logic
│   └── utils.rs          # Helper functions
├── examples/
│   ├── check_updates.rs  # Simple version checker
│   └── automated_monitor.rs # Production monitor
├── Cargo.toml            # Dependencies
├── README.md             # This file
└── VERSION_CHECKING.md   # Version system docs
```

### Building

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Run tests
cargo test

# Run with optimizations
cargo run --release

# Build specific example
cargo build --release --example check_updates
```

### Dependencies

Key dependencies (see `Cargo.toml` for full list):

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client for downloads |
| `rayon` | Parallel iterators |
| `unity-rs` | Unity asset parsing |
| `serde` / `serde_json` | JSON serialization |
| `indicatif` | Progress bars |
| `aes` / `cbc` | Decryption |
| `image` | Image processing |
| `chrono` | Timestamp handling |
| `clap` | CLI argument parsing |

### Adding Features

Example: Add new asset type support

```rust
// In download_fromlist(), add new ClassIDType match arm
ClassIDType::YourNewType => {
    if let Ok(data) = obj_mut.read(false) {
        if let Ok(asset) = serde_json::from_value::<YourNewType>(data) {
            // Process and save
            let save_path = unpack_dir.join(format!("{}.ext", asset.name));
            // ... processing logic
        }
    }
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_version_check

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

## Contributing

Contributions are welcome! Here's how you can help:

### Reporting Bugs

Open an issue with:
- Rust version (`rustc --version`)
- OS and version
- Full error message
- Steps to reproduce

### Suggesting Features

Open an issue describing:
- Use case
- Expected behavior
- Alternative solutions considered

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Ensure `cargo test` passes
6. Ensure `cargo clippy` passes
7. Format code (`cargo fmt`)
8. Commit changes (`git commit -m 'Add amazing feature'`)
9. Push to branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

### Code Style

- Follow Rust conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Add documentation comments for public APIs
- Write tests for new functionality

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ArkAssetsTool](https://github.com/ChaomengOrion/ArkAssetsTool) - Original implementation that this project is based on
- [unity-rs](https://github.com/Eltik/UnityRs) - Rust Unity asset parsing library
- Arknights - Game assets belong to Hypergryph/Yostar
- Rust Community - Amazing ecosystem and tools

## Related Projects

- [ArkAssetsTool](https://github.com/ChaomengOrion/ArkAssetsTool) - Original Rust implementation
- [Assets-Unpacker](https://github.com/Eltik/Assets-Unpacker) - Rust asset extraction tool
- [unity-rs](https://github.com/Eltik/UnityRs) - Rust Unity asset parser
- [Ark-Unpacker](https://github.com/ArknightsAssets/Ark-Unpacker) - Python implementation

## Support

For questions, issues, or feature requests:
- Open an issue on GitHub
- Check existing issues for solutions
- Enable debug logging and include output with bug reports

---

**Note**: This tool is for educational and archival purposes only. All game assets remain the property of their respective copyright holders.
