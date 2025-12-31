# myrtle.moe

A comprehensive Arknights game companion platform featuring operator databases, player profile syncing, tier lists, and advanced game tools—all powered by a high-performance Rust backend and modern React frontend.

[![GitHub license](https://img.shields.io/github/license/Eltik/myrtle.moe)](https://github.com/Eltik/myrtle.moe/blob/main/LICENSE)
[![Frontend](https://img.shields.io/badge/Frontend-Next.js_15-black)](frontend/)
[![Backend](https://img.shields.io/badge/Backend-Rust/Axum-orange)](backend/)
[![Demo](https://img.shields.io/badge/Demo-Live-brightgreen)](https://myrtle.moe)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Quick Start](#quick-start)
  - [Asset Pipeline Setup](#asset-pipeline-setup)
- [Technology Stack](#technology-stack)
- [Architecture](#architecture)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Overview

myrtle.moe is a feature-rich Arknights toolkit designed to provide players with accurate game data, player profile synchronization, community tier lists, and intuitive visualizations. The platform offers a modern, responsive interface that makes it easy to research operators, view detailed statistics, plan upgrades, and access game assets.

Named after the Arknights operator Myrtle, this project aims to be the most comprehensive, accurate, and user-friendly Arknights resource available.

## Features

- **Operator Database**: Complete information on all 400+ Arknights operators including stats, skills, talents, modules, skins, voice lines, and lore
- **Player Profile Sync**: Link your Arknights account to view your roster, inventory, and base layout synced directly from game servers
- **Recruitment Calculator**: Calculate optimal tag combinations for the recruitment system
- **Asset Browser**: Direct access to game artwork, Spine animations, audio files, and other assets
- **Game Data API**: Robust REST API for accessing Arknights game data programmatically
- **Modern UI**: Elegant, responsive interface with dark/light themes and customizable accent colors

## Project Structure

```
myrtle.moe/
├── frontend/           # Next.js 15 web application (React 19, TypeScript, Tailwind CSS v4)
├── backend/            # Rust API server (Axum, PostgreSQL, Redis)
├── assets/             # Game asset processing toolkit
│   ├── downloader/     # Multi-server asset downloader (Rust)
│   ├── unpacker/       # High-performance asset extractor (Rust)
│   ├── unity-rs/       # Unity asset parsing library (Rust port of UnityPy)
│   ├── OpenArknightsFBS/ # FlatBuffers schemas for game data
│   └── Unpacked/       # Extracted game assets (~90GB)
└── .github/            # CI/CD workflows
```

Each component has its own detailed README with specific documentation.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) 20+ or [Bun](https://bun.sh/) 1.0+
- [PostgreSQL](https://www.postgresql.org/) 14+
- [Redis](https://redis.io/) 7+
- ~100GB disk space for game assets (for full asset extraction)

### Quick Start

1. **Clone the repository**
```bash
git clone https://github.com/Eltik/myrtle.git
cd myrtle
```

2. **Set up the backend**
```bash
cd backend

# Create environment file
cat > .env << EOF
DATABASE_URL=postgres://user:password@localhost/myrtle
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your-secret-key-here
DATA_DIR=../assets/Unpacked/gamedata/excel
ASSETS_DIR=../assets/Unpacked
EOF

# Build and run
cargo build --release
cargo run --release
```

3. **Set up the frontend**
```bash
cd frontend

# Create environment file
cat > .env << EOF
BACKEND_URL=http://localhost:3060
EOF

# Install dependencies and run
bun install
bun run dev
```

4. **Access the application**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:3060

### Asset Pipeline Setup

The asset pipeline downloads and extracts game assets from Arknights servers. This is optional but required for serving game images, audio, and animations.

#### 1. Build the Asset Tools

```bash
cd assets

# Run the build script.
# If you encounter any errors (especially on Windows),
# there are respective .md files for common issues.
node build.js
```

#### 2. Download Game Assets

```bash
# Set library path for FMOD audio support (macOS)
export DYLD_LIBRARY_PATH=$(pwd)/unity-rs/src/resources/FMOD/Darwin:$DYLD_LIBRARY_PATH

# Download assets
./downloader/target/release/arknights-downloader \
    --server en \
    --savedir ./ArkAssets \
    --threads 4
```

#### 3. Extract Assets

```bash
# Extract all asset types
./unpacker/target/release/assets-unpacker extract \
    --input ./ArkAssets \
    --output ./Unpacked \
    --image true \
    --text true \
    --audio true \
    --spine true \
    --threads 4

# Combine RGB + alpha textures into RGBA
./unpacker/target/release/assets-unpacker combine \
    --input ./Unpacked/upk \
    --output ./Unpacked/cmb

# Decode encrypted game data (FlatBuffers, AES)
./unpacker/target/release/assets-unpacker decode \
    --input ./Unpacked/upk \
    --output ./Unpacked/decoded
```

#### 4. Link Assets to Backend

```bash
# Symlink or copy assets to backend
ln -s $(pwd)/Unpacked ../backend/assets
ln -s $(pwd)/Unpacked/decoded/gamedata/excel ../backend/data
```

For detailed asset pipeline documentation, see:
- [Downloader README](assets/downloader/README.md)
- [Unpacker README](assets/unpacker/README.md)

## Technology Stack

### Frontend
| Technology | Purpose |
|------------|---------|
| [Next.js 15](https://nextjs.org/) | React framework with App Router |
| [React 19](https://react.dev/) | UI library |
| [TypeScript](https://www.typescriptlang.org/) | Type-safe JavaScript |
| [Tailwind CSS v4](https://tailwindcss.com/) | Utility-first CSS with OKLCH colors |
| [shadcn/ui](https://ui.shadcn.com/) | Radix-based component library |
| [Motion Primitives](https://motion-primitives.com/) | Motion-based component library |
| [PixiJS](https://pixijs.com/) + Pixi-Spine | Spine animation rendering |

### Backend
| Technology | Purpose |
|------------|---------|
| [Rust](https://www.rust-lang.org/) | Systems programming language |
| [Axum](https://github.com/tokio-rs/axum) | Async web framework |
| [SQLx](https://github.com/launchbadge/sqlx) | Compile-time SQL queries |
| [PostgreSQL](https://www.postgresql.org/) | Primary database |
| [Redis](https://redis.io/) | Session cache & rate limiting |
| [JWT](https://jwt.io/) | Authentication tokens |

### Asset Toolkit
| Component | Purpose |
|-----------|---------|
| [unity-rs](assets/unity-rs/) | Rust port of UnityPy for Unity asset parsing |
| [downloader](assets/downloader/) | Multi-server asset downloader with version tracking |
| [unpacker](assets/unpacker/) | High-performance asset extraction (9x faster than Python) |
| [OpenArknightsFBS](assets/OpenArknightsFBS/) | FlatBuffers schemas for 59 game data tables |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            Arknights Servers                            │
│                  (EN, JP, KR, CN Official, Bilibili, TW)               │
└─────────────────────────────────────┬───────────────────────────────────┘
                                      │
                    ┌─────────────────▼─────────────────┐
                    │        Asset Downloader           │
                    │    (Rust, parallel downloads)     │
                    └─────────────────┬─────────────────┘
                                      │
                    ┌─────────────────▼─────────────────┐
                    │        Asset Unpacker             │
                    │  (Textures, Audio, Spine, Data)   │
                    └─────────────────┬─────────────────┘
                                      │
┌─────────────────┐     ┌─────────────▼─────────────┐     ┌─────────────────┐
│    Frontend     │◄────┤        Backend            │◄────┤   PostgreSQL    │
│   (Next.js 15)  │     │      (Rust/Axum)          │     │    Database     │
│                 │     │                           │     └─────────────────┘
│  - Operators    │     │  - REST API               │
│  - Profiles     │     │  - Authentication         │     ┌─────────────────┐
│  - Tier Lists   │     │  - Game Data              │◄────┤     Redis       │
│  - Recruitment  │     │  - CDN Proxy              │     │     Cache       │
│  - Settings     │     │  - Rate Limiting          │     └─────────────────┘
└─────────────────┘     └───────────────────────────┘
                                      │
                        ┌─────────────▼─────────────┐
                        │     Processed Assets      │
                        │  (~90GB extracted data)   │
                        │  - Images (PNG)           │
                        │  - Audio (WAV)            │
                        │  - Spine animations       │
                        │  - Game tables (JSON)     │
                        └───────────────────────────┘
```

### Data Flow

1. **Asset Pipeline**: Downloads raw game files from Arknights CDN, extracts textures/audio/animations, decodes encrypted game data
2. **Backend**: Serves game data via REST API, handles authentication with Yostar OAuth, manages tier lists and user data
3. **Frontend**: Renders operator database, player profiles, tier lists, and tools with real-time data from the backend

## Documentation

| Component | Documentation |
|-----------|---------------|
| Frontend | [frontend/README.md](frontend/README.md) |
| Backend | [backend/README.md](backend/README.md) |
| Asset Downloader | [assets/downloader/README.md](assets/downloader/README.md) |
| Asset Unpacker | [assets/unpacker/README.md](assets/unpacker/README.md) |
| Unity-RS Library | [assets/unity-rs/README.md](assets/unity-rs/README.md) |
| FlatBuffers Schemas | [MooncellWiki/OpenArknightsFBS/tree/YoStar/README.md](https://github.com/MooncellWiki/OpenArknightsFBS/tree/YoStar/README.md) |

## API Reference

The backend exposes a comprehensive REST API:

### Authentication
```
POST /login                    # Login with email + verification code
POST /refresh                  # Refresh session token
POST /send-code                # Request verification code
```

### Game Data
```
GET  /static/operators         # List all operators
GET  /static/operators/{id}    # Get operator details
GET  /static/skills            # List all skills
GET  /static/modules           # List all modules
GET  /static/materials         # List all materials
GET  /static/skins             # List all skins
GET  /static/gacha             # Gacha pool data
```

### User Data
```
GET  /get-user/{uid}           # Get user profile
POST /auth/update-settings     # Update user settings
```

### Tier Lists
```
GET  /tier-lists               # List all tier lists
GET  /tier-lists/{slug}        # Get tier list details
POST /tier-lists               # Create tier list (admin)
POST /tier-lists/{slug}/publish # Publish new version
```

### Assets
```
GET  /cdn/avatar/{id}          # Operator avatar image
GET  /cdn/portrait/{id}        # Character portrait
GET  /cdn/{path}               # Any game asset
```

For complete API documentation, see [backend/README.md](backend/README.md).

## License

TBD

## Acknowledgements

This project builds upon the work of many open-source projects:

- [UnityPy](https://github.com/K0lb3/UnityPy) - Original Python library for Unity asset extraction (unity-rs is a Rust port)
- [OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS) - FlatBuffers schemas
- [ArkPRTS](https://github.com/thesadru/ArkPRTS) - Inspiration for authentication flow
- [isHarryH/Årk-Unpacker](https://github.com/isHarryh/Ark-Unpacker) - Original Python implementation for unpacking (current `/unpacker` is a Rust port)

---

**Note**: This project is not affiliated with Hypergryph, Yostar, or any official Arknights entities. All game data and assets are property of their respective owners.
