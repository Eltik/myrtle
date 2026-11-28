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

- [Rust](https://rustup.rs/) 1.94+ and [Bun](https://bun.sh/) 1.4.2
- PostgreSQL, plus Redis if you want the shared cache
- [Docker engine](https://docs.docker.com/engine/) and [Docker Compose](https://docs.docker.com/compose/), for the asset tools (and for Postgres and Redis, if you would rather not install them)
- ~100GB of free space on disk for assets downloading and extraction.

> Note: Docker engine and compose are included in Docker Desktop.

Before getting started, be sure to clone the repository and initialize the submodules.

```shell
git clone https://github.com/Eltik/myrtle.git
cd myrtle
git submodule update --init --recursive
```

---

## Getting Started

### 1. Environment Setup

Copy `.env.example` into a new `.env` file at the root of the project. Compose reads it.

```shell
cp .env.example .env
```

The options that still apply:

```env
ENABLE_CACHE=true
ENABLE_DB=true
POSTGRES_DB=myrtle
POSTGRES_USER=myrtle
POSTGRES_PASSWORD=myrtle
```

- `ENABLE_CACHE`: Run the Redis container
- `ENABLE_DB`: Run the Postgres container
- `POSTGRES_*`: Name and credentials of the database the Postgres container creates

The backend does not read this file's `SERVERS`, `ASSET_WS_URLS`, or `GAME_DATA_DIR`. Its configuration is `backend/config/myrtle.toml` plus the secrets in `backend/.env`, described in [backend/README.md](backend/README.md#configuration). There is no `backend/.env.example`.

### 2. Assets Pipeline

The `asset-tools` service runs in an isolated Rust container and operates on the `tools` profile so it doesn't run during normal startup.

#### 1. Build the tools container

```shell
docker compose --profile tools build
```

#### 2. Generate FlatBuffer schemas

```shell
docker compose run --rm asset-tools generate_fbs.sh
```

#### 3. Download Arknights Assets

```shell
docker compose run --rm asset-tools download.sh [server] [threads]
# Examples: 
# docker compose run --rm asset-tools download.sh en 4
# docker compose run --rm asset-tools download.sh cn 2
```

#### 4. Unpack and Decode Assets

```shell
docker compose run --rm asset-tools unpack.sh [server] [threads]
# Example: docker compose run --rm asset-tools unpack.sh en 4
```

> **NOTE**: The asset downloader and extractor can also be run directly without the utility scripts.
>
> ```shell
> docker compose run --rm asset-tools downloader
> #OR
> docker compose run --rm asset-tools unpacker
> ```

For detailed asset pipeline documentation, see the [assets README](assets/README.md).

#### 5. Regenerate DPS Calculations

This runs on the host, from `backend/`. It needs the `external/ArknightsDpsCompare` submodule and Python 3.10+ with upstream's dependencies.

```shell
cargo run -p myrtle-dps-transpile              # DPS: formulas, transpile, expected values
cargo run -p myrtle-dps-transpile -- --healing # HPS: the same three steps
```

It parses the Python DPS source and transpiles the formulas into `backend/crates/dps-formulas`. The compose `dps-tools` service predates the backend rewrite and no longer builds.

---

### 3. Running the Application

Docker Compose does not run the whole stack at the moment. `docker-compose.yml` still describes the previous backend: its `backend` service builds from a `backend/Dockerfile` the rewrite removed, and the `frontend` service depends on it. Use compose for the database and cache, and run the two applications on the host.

#### Start Postgres and Redis

```shell
docker compose up -d postgres redis
```

#### Run the backend

Follow the [backend Quick Start](backend/README.md#quick-start). In short: load the migrations into a scratch database, build against it, copy `config/myrtle.example.toml` to `config/myrtle.toml`, put `DATABASE_URL`, `JWT_SECRET`, and `GAME_CREDENTIAL_KEY` in `backend/.env`, then:

```shell
cd backend
cargo run -p myrtle-api -- config/myrtle.toml
cargo run -p myrtle-worker -- config/myrtle.toml   # In a second terminal. Run exactly one
```

With the compose Postgres, the URL is `postgres://myrtle:myrtle@localhost:5432/myrtle`.

#### Run the frontend

```shell
cd frontend
cp .env.example .env   # Point BACKEND_URL and VITE_BACKEND_URL at http://localhost:3060
bun install
bun --bun run dev
```

The frontend will bind to `http://localhost:3000` and the backend to `http://localhost:3060`.

#### Stop the containers

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
├── frontend/           # TanStack Start web application (React 19, TypeScript, Tailwind CSS v4)
├── backend/            # Rust API server, job worker and CLI (Axum, PostgreSQL, Redis)
├── assets/             # Game asset processing toolkit
│   ├── downloader/     # Multi-server asset downloader (Rust)
│   ├── unpacker/       # High-performance asset extractor (Rust)
│   ├── schemas/        # FlatBuffers schema store (see assets/schemas/README.md)
│   └── output/         # Extracted game assets, one directory per server (gitignored)
├── discord/            # Discord bot (Rust, Poise/Serenity, SQLite)
└── .github/            # CI/CD workflows
```

Each component has its own detailed README with specific documentation.

## Technology Stack

### Frontend

| Technology                                          | Purpose                             |
| --------------------------------------------------- | ----------------------------------- |
| [TanStack Start](https://tanstack.com/start)        | React framework, SSR on Vite        |
| [React 19](https://react.dev/)                      | UI library                          |
| [TypeScript](https://www.typescriptlang.org/)       | Type-safe JavaScript                |
| [Tailwind CSS v4](https://tailwindcss.com/)         | Utility-first CSS with OKLCH colors |
| [shadcn/ui](https://ui.shadcn.com/)                 | Base UI-based component library     |
| [PixiJS](https://pixijs.com/) + Pixi-Spine          | Spine animation rendering           |

### Backend

| Technology                                  | Purpose                       |
| ------------------------------------------- | ----------------------------- |
| [Rust](https://www.rust-lang.org/)          | Systems programming language  |
| [Axum](https://github.com/tokio-rs/axum)    | Async web framework           |
| [SQLx](https://github.com/launchbadge/sqlx) | Compile-time SQL queries      |
| [PostgreSQL](https://www.postgresql.org/)   | Primary database              |
| [Redis](https://redis.io/)                  | Shared cache (optional)       |
| [JWT](https://jwt.io/)                      | Session tokens                |

### Asset Toolkit

| Component                                                                        | Purpose                                                   |
| -------------------------------------------------------------------------------- | --------------------------------------------------------- |
| [downloader](assets/downloader/)                                                 | Multi-server asset downloader with version tracking       |
| [unpacker](assets/unpacker/)                                                     | High-performance asset extraction (9x faster than Python) |
| [schemas](assets/schemas/)                                                       | FlatBuffers schema store, generated from the game client   |

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
│ (TanStack Start)│     │      (Rust/Axum)          │     │    Database     │
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
2. **Backend**: Serves game data via REST API, signs players in against the game servers (Yostar email code, CN passport, Bilibili), manages tier lists and user data, and runs background jobs in a separate worker
3. **Frontend**: Renders operator database, player profiles, tier lists, and tools with real-time data from the backend

## Documentation

| Component           | Documentation                                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Frontend            | [frontend/README.md](frontend/README.md)                                                                                      |
| Backend             | [backend/README.md](backend/README.md)                                                                                        |
| OAuth Provider      | [backend/OAUTH.md](backend/OAUTH.md)                                                                                          |
| Asset Pipeline      | [assets/README.md](assets/README.md)                                                                                          |
| FlatBuffers Schemas | [assets/schemas/README.md](assets/schemas/README.md)                                                                          |
| Discord Bot         | [discord/README.md](discord/README.md)                                                                                        |

## API Reference

Every route is served under `/api`. A sample; there is no OpenAPI document, so the route modules in `backend/crates/api/src/routes/` are the full list.

### Authentication

```text
POST /api/login/send-code      # Request a Yostar verification code
POST /api/login                # Login with email + verification code
POST /api/refresh              # Re-sync the account from the game server
GET  /api/auth/verify          # Check a session token
POST /api/auth/update-settings # Update user settings
```

### Game Data

```text
GET  /api/operators/index      # List all operators
GET  /api/operators/{id}       # Get operator details
GET  /api/skins/{id}           # An operator's skins
GET  /api/voices/{id}          # An operator's voice lines
GET  /api/static/{resource}    # Static tables: skills, materials, ranges, enemies, ...
```

### User Data

```text
GET  /api/get-user             # Get a user profile
GET  /api/get-user-score       # Get a user's account grade
```

### Tier Lists

```text
GET  /api/tier-lists           # List all tier lists
GET  /api/tier-lists/{slug}    # Get tier list details
```

### Assets

```text
GET  /api/avatar/{id}          # Operator avatar image
GET  /api/portrait/{id}        # Character portrait
GET  /api/assets/{path}        # Any game asset
```

For complete API documentation, see [backend/README.md](backend/README.md).
