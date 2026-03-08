# Myrtle Developer Docs

## Table of Contents

- [Getting Started](#getting-started)
  - [Environment Setup](#1-environment-setup)
  - [Assets Pipeline](#2-assets-pipeline)
  - [Running the Application](#3-running-the-application)
- [Project Structure](#project-structure)
- [Technology Stack](#technology-stack)
- [Architecture](#architecture)
- [Documentation](#documentation)

## Prerequisites

- [Docker engine](https://docs.docker.com/engine/)
- [Docker Compose](https://docs.docker.com/compose/)
- ~100GB of free space on disk for assets downloading and extraction.

> Note: Docker engine and compose are included in Docker Desktop.

Before getting started, be sure to clone the repository.

```shell
git clone https://github.com/Eltik/myrtle.git
cd myrtle
```

---

## Getting Started

### 1. Environment Setup

First, configure your environment variables.
Copy `.env.example` into a new `.env` file at the root of the project.

```shell
cp .env.example .env
```

Following docker-specific options are to be configured in the `.env` file.

```env
DOCKERFILE=Dockerfile.dev
ENABLE_CACHE=true
ENABLE_DB=true
RUST_LOG=info
```

- `DOCKERFILE`
  - `Dockerfile` for production (release binary for backend and standalone build for frontend)
  - `Dockerfile.dev` for development (debug binary for backend and dev server for frontend) (default)
- `ENABLE_CACHE`: Enable Redis cache (if false, must update the redis cache URI in backend vars)
- `ENABLE_DB`: Enable Postgres database (if false, must update the postgres db URI in backend vars)
- `RUST_LOG`: Rust log level for the backend

Once configured, build the images.

```shell
docker compose build
```

This will build the images for `postgres` and `redis` if enabled along with backend and frontend.

> **NOTE:** If the `DOCKERFILE` is changed (switching from prod to dev or dev to prod) the images must be built with this command again.

### 2. Assets Pipeline

The `asset-tools` service runs in an isolated Rust container and operates on the `tools` profile so it doesn't run during normal startup.

#### 1. Build the tools container

```shell
docker compose --profile tools build
```

#### 2. Download Arknights Assets

```shell
docker compose run --rm asset-tools download.sh [server] [threads]
# Examples: 
# docker compose run --rm asset-tools download.sh en 4
# docker compose run --rm asset-tools download.sh cn 2
```

#### 3. Unpack and Decode Assets

```shell
docker compose run --rm asset-tools unpack.sh [threads]
# Example: docker compose run --rm asset-tools unpack.sh 4
```

> **NOTE**: The asset downloader and extractor can also be run directly without the utility scripts.
>
> ```shell
> docker compose run --rm asset-tools arknights-downloader
> #OR
> docker compose run --rm asset-tools assets-unpacker
> ```

For detailed asset pipeline documentation, see:

- [Downloader README](assets/downloader/README.md)
- [Unpacker README](assets/unpacker/README.md)

---

### 3. Running the Application

#### Start the entire stack

```shell
# Starts postgres, redis, backend, and frontend
docker compose up -d
```

The frontend will bind to `http://localhost:3000` and the backend to `http://localhost:3060`.

#### Stop the stack

```shell
# Stop services
docker compose down

# Stop services AND wipe database volumes
docker compose down -v
```

---

## Project Structure

```text
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

## Technology Stack

### Frontend

| Technology | Purpose |
| ------------ | --------- |
| [Next.js 15](https://nextjs.org/) | React framework with App Router |
| [React 19](https://react.dev/) | UI library |
| [TypeScript](https://www.typescriptlang.org/) | Type-safe JavaScript |
| [Tailwind CSS v4](https://tailwindcss.com/) | Utility-first CSS with OKLCH colors |
| [shadcn/ui](https://ui.shadcn.com/) | Radix-based component library |
| [Motion Primitives](https://motion-primitives.com/) | Motion-based component library |
| [PixiJS](https://pixijs.com/) + Pixi-Spine | Spine animation rendering |

### Backend

| Technology | Purpose |
| ------------ | --------- |
| [Rust](https://www.rust-lang.org/) | Systems programming language |
| [Axum](https://github.com/tokio-rs/axum) | Async web framework |
| [SQLx](https://github.com/launchbadge/sqlx) | Compile-time SQL queries |
| [PostgreSQL](https://www.postgresql.org/) | Primary database |
| [Redis](https://redis.io/) | Session cache & rate limiting |
| [JWT](https://jwt.io/) | Authentication tokens |

### Asset Toolkit

| Component | Purpose |
| ----------- | --------- |
| [unity-rs](assets/unity-rs/) | Rust port of UnityPy for Unity asset parsing |
| [downloader](assets/downloader/) | Multi-server asset downloader with version tracking |
| [unpacker](assets/unpacker/) | High-performance asset extraction (9x faster than Python) |
| [OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS/tree/YoStar) | FlatBuffers schemas for 59 game data tables |

## Architecture

```text
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
| ----------- | --------------- |
| Frontend | [frontend/README.md](frontend/README.md) |
| Backend | [backend/README.md](backend/README.md) |
| Asset Downloader | [assets/downloader/README.md](assets/downloader/README.md) |
| Asset Unpacker | [assets/unpacker/README.md](assets/unpacker/README.md) |
| Unity-RS Library | [assets/unity-rs/README.md](assets/unity-rs/README.md) |
| FlatBuffers Schemas | [MooncellWiki/OpenArknightsFBS/tree/YoStar/README.md](https://github.com/MooncellWiki/OpenArknightsFBS/tree/YoStar/README.md) |

## API Reference

The backend exposes a comprehensive REST API:

### Authentication

```text
POST /login                    # Login with email + verification code
POST /refresh                  # Refresh session token
POST /send-code                # Request verification code
```

### Game Data

```text
GET  /static/operators         # List all operators
GET  /static/operators/{id}    # Get operator details
GET  /static/skills            # List all skills
GET  /static/modules           # List all modules
GET  /static/materials         # List all materials
GET  /static/skins             # List all skins
GET  /static/gacha             # Gacha pool data
```

### User Data

```text
GET  /get-user/{uid}           # Get user profile
POST /auth/update-settings     # Update user settings
```

### Tier Lists

```text
GET  /tier-lists               # List all tier lists
GET  /tier-lists/{slug}        # Get tier list details
POST /tier-lists               # Create tier list (admin)
POST /tier-lists/{slug}/publish # Publish new version
```

### Assets

```text
GET  /cdn/avatar/{id}          # Operator avatar image
GET  /cdn/portrait/{id}        # Character portrait
GET  /cdn/{path}               # Any game asset
```

For complete API documentation, see [backend/README.md](backend/README.md).
