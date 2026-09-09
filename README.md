# myrtle.moe

A comprehensive Arknights companion platform: operator database, player-profile syncing, DPS/HPS calculators, RIIC base optimization, a stage and enemy viewer, community tier lists, gacha tracking, and a scoring leaderboard. Powered by a high-performance Rust backend, a modern SSR React frontend, a from-scratch Rust asset pipeline, and a Discord bot.

[![Backend CI](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml)
[![Frontend CI](https://github.com/Eltik/myrtle/actions/workflows/frontend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/frontend-ci.yml)
[![Assets CI](https://github.com/Eltik/myrtle/actions/workflows/assets-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/assets-ci.yml)
[![Discord CI](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/discord-ci.yml)
[![Demo](https://img.shields.io/badge/Demo-myrtle.moe-brightgreen)](https://myrtle.moe)
[![Backend](https://img.shields.io/badge/Backend-Rust%2FAxum-orange?logo=rust)](backend/)
[![Frontend](https://img.shields.io/badge/Frontend-TanStack_Start-ff4154)](frontend/)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](#license)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Repository Structure](#repository-structure)
- [Tech Stack](#tech-stack)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Overview

myrtle.moe is a feature-rich Arknights toolkit that gives players accurate game data, player-profile synchronization, community tier lists, and interactive visualizations behind a fast, responsive interface. Named after the Arknights operator Myrtle, the project aims to be the most comprehensive, accurate, and user-friendly Arknights resource available.

The platform is built from four independent components: a Rust API server, a TanStack Start web client, a Rust pipeline that downloads and extracts game assets directly from the official CDN, and a Discord bot.

## Features

- **Operator Database** - every operator with stats, skills, talents, modules, attack ranges, skins, voice lines, lore, and Spine chibi animations.
- **Dynamic Illustrations** - Arknights' animated operator art (L2D) reconstructed and rendered in the browser, entrance cinematics and particle effects included.
- **Player Profile Sync** - log in with a Yostar email code (Global/EN, JP, KR), a CN passport, or Bilibili, and sync your roster, inventory, and game state straight from Hypergryph servers.
- **DPS & HPS Calculators** - per-skill damage and healing curves for 200+ operators, transpiled from the upstream Python reference and validated against it in CI.
- **RIIC Base Optimizer** - layout scoring and optimization derived entirely from parsed game data: shift rotations, dormitory morale, per-skill contribution ledgers, and trading-post order economics.
- **Stage & Enemy Viewer** - every stage as a tile map with enemy pathing, waves, and drops, plus an enemy database with chibis.
- **Operator Planner** - promotion, level, skill, and module targets grouped into shareable plans.
- **Tier Lists** - browse official and community lists, or build and publish your own with a drag-and-drop editor, versioning, favorites, and a 4-level permission system.
- **Gacha Tracking** - synced pull history with rarity splits and pity counters, plus community-wide pull statistics.
- **Scoring & Leaderboard** - composite account scores with grades, rankings, biggest movers, and score distribution.
- **Tools** - recruitment tag calculator, squad randomizer, and an operator birthday calendar.
- **Discord Bot** - guild moderation, asset-update announcements, and platform lookups.
- **Asset Pipeline** - download and extract textures, audio, Spine animations, and game-data tables across six server regions, with content profiles for lightweight partial downloads.
- **Modern UI** - light / dark / auto theming with a customizable accent color, plus dynamically generated social cards.

## Architecture

```text
Arknights CDN (EN, JP, KR, CN, Bilibili, TW)
        │
        ▼
  assets/   Rust pipeline: downloader + unpacker
        │   (textures, audio, Spine, gamedata JSON)
        │   serves update events over WebSocket
        ▼
  backend/  Rust + Axum REST API ──► PostgreSQL (users, rosters, tier lists)
        │   game data, player sync,  └─► Redis (cache, optional)
        │   DPS/HPS, base, scoring, gacha
        ├──────────────────────────────► discord/  Rust bot (moderation, lookups)
        ▼
 frontend/  TanStack Start (SSR React 19) web client
```

- **assets** downloads raw bundles from the Arknights CDN, then unpacks Unity assets and decodes FlatBuffer game-data tables to JSON. It also runs a WebSocket server that announces completed updates.
- **backend** loads that game data into memory at startup, serves it over a REST API, handles authentication, and persists user data in PostgreSQL. It also generates the frontend's TypeScript types.
- **frontend** renders the operator database, profiles, stages, tier lists, and tools, talking to the backend for all data and calculations.
- **discord** watches the pipeline's WebSocket, announces updates, and queries the backend.

## Repository Structure

```text
myrtle.moe/
├── backend/            Rust + Axum REST API (PostgreSQL, Redis, DPS/HPS, base optimizer)
├── frontend/           TanStack Start (SSR React 19) web client
├── assets/             Rust asset pipeline (downloader + unpacker) + FBS schemas
│   ├── downloader/     Multi-region CDN downloader with incremental updates
│   └── unpacker/       UnityFS bundle extractor and FlatBuffer decoder
├── discord/            Rust Discord bot (Poise/Serenity, SQLite)
├── .github/workflows/  Backend, frontend, assets, and Discord CI pipelines
├── docker-compose.yml  Full local stack (db, cache, backend, frontend, asset watchers, tools)
├── BUILD.md            Manual (non-Docker) build instructions
└── CONTRIBUTING.md     Full developer setup and workflow
```

Each component has its own README with detailed documentation.

## Tech Stack

| Component | Stack                                                                               |
| --------- | ----------------------------------------------------------------------------------- |
| Backend   | Rust (Edition 2024), Axum 0.8, Tokio, SQLx + PostgreSQL, Redis, JWT, ts-rs           |
| Frontend  | TanStack Start (SSR), React 19, TypeScript 5.7, Tailwind CSS v4, Vite 7, Bun, Biome |
| Assets    | Rust (Edition 2024) downloader + unpacker, Node.js interactive runner                |
| Discord   | Rust (Edition 2024), Poise 0.6 over Serenity 0.12, SQLite                            |

## Quick Start

The fastest path is Docker Compose, which runs PostgreSQL, Redis, the backend, the frontend, and the per-region asset watchers together.

### Prerequisites

- [Docker Engine](https://docs.docker.com/engine/) and [Docker Compose](https://docs.docker.com/compose/)
- ~100 GB of free disk space for asset download and extraction
- Game assets extracted via the asset pipeline (see below)

### 1. Clone with submodules

```bash
git clone https://github.com/Eltik/myrtle.git
cd myrtle
git submodule update --init --recursive
```

### 2. Configure environment

```bash
cp .env.example .env
```

Set `JWT_SECRET`, `SERVICE_KEY`, and `GAME_CREDENTIAL_KEY`; the backend refuses to start without them. `COMPOSE_PROFILES` selects which services run - `true` is the always-on app, and `tools` adds the one-off asset and DPS job containers.

### 3. Build the game assets

```bash
docker compose --profile tools build
docker compose run --rm asset-tools generate_fbs.sh
docker compose run --rm asset-tools download.sh en 4
docker compose run --rm asset-tools unpack.sh en 4
```

`download.sh` takes `SERVER THREADS [PROFILE]`. Pass a profile such as `gamedata` or `operators` for a much smaller download when you do not need the full art set.

### 4. Start the stack

```bash
docker compose build
docker compose up -d
```

The frontend binds to `http://localhost:3000` and the backend to `http://localhost:3060`.

For manual (non-Docker) setup, see [BUILD.md](BUILD.md) and the per-component READMEs.

## Documentation

| Component       | Documentation                                          |
| --------------- | ------------------------------------------------------ |
| Backend API     | [backend/README.md](backend/README.md)                 |
| Frontend        | [frontend/README.md](frontend/README.md)               |
| Asset Pipeline  | [assets/README.md](assets/README.md)                   |
| Discord Bot     | [discord/README.md](discord/README.md)                 |
| Developer Setup | [CONTRIBUTING.md](CONTRIBUTING.md)                     |
| Manual Build    | [BUILD.md](BUILD.md)                                   |
| DPS Calculator  | [backend/DPS_CALCULATOR.md](backend/DPS_CALCULATOR.md) |
| IL2CPP Recovery | [assets/IL2CPP_RECOVERY.md](assets/IL2CPP_RECOVERY.md) |

## Contributing

Contributions are welcome. Clone the repo with submodules, follow the setup in [CONTRIBUTING.md](CONTRIBUTING.md), and run each component's lint, format, and type checks before opening a pull request:

- Backend: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- Frontend: `bun --bun run check`, `bunx tsc --noEmit`
- Assets: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- Discord: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`

The frontend has no test suite yet; verify changes with the type-checker, Biome, and the browser. Frontend API types are generated from the Rust backend - if you change a backend type, run `bun run gen:types` in `frontend/` and commit the result rather than editing `frontend/src/types/generated/` by hand.

Branch from `main` (or the active development branch) and keep changes scoped to a single component where possible.

## License

License TBD. No license file is currently present in the repository.

## Acknowledgements

This project builds on the work of many open-source projects:

- [UnityPy](https://github.com/K0lb3/UnityPy) - reference Python library for Unity asset extraction (the unpacker is a Rust port).
- [OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS) - FlatBuffers schemas for game data.
- [ArkPRTS](https://github.com/thesadru/ArkPRTS) - inspiration for the authentication flow.
- [ArknightsDpsCompare](https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare) - upstream reference for the DPS/HPS calculations.
- [isHarryH/Ark-Unpacker](https://github.com/isHarryH/Ark-Unpacker) - original Python unpacking implementation.

---

**Not affiliated with Hypergryph, Yostar, or any official Arknights entity.** All game data and assets are property of their respective owners.
