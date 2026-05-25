# myrtle.moe

A comprehensive Arknights companion platform: operator database, player-profile syncing, DPS/HPS calculators, community tier lists, gacha tracking, and a scoring leaderboard. Powered by a high-performance Rust backend, a modern SSR React frontend, and a from-scratch Rust asset pipeline.

[![Backend CI](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/backend-ci.yml)
[![Assets CI](https://github.com/Eltik/myrtle/actions/workflows/assets-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/assets-ci.yml)
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

The platform is built from three independent components: a Rust API server, a TanStack Start web client, and a Rust pipeline that downloads and extracts game assets directly from the official CDN.

## Features

- **Operator Database** - every operator with stats, skills, talents, modules, attack ranges, skins, voice lines, lore, and Spine chibi animations.
- **Player Profile Sync** - log in with a Yostar email code (Global/EN, JP, KR) to sync your roster, inventory, and game state straight from Hypergryph servers.
- **DPS & HPS Calculators** - per-skill damage and healing curves for 200+ operators, transpiled from the upstream Python reference and validated against it in CI.
- **Tier Lists** - browse official and community lists, or build and publish your own with a drag-and-drop editor, versioning, favorites, and a 4-level permission system.
- **Gacha Tracking** - synced pull history with rarity splits and pity counters, plus community-wide pull statistics.
- **Scoring & Leaderboard** - composite account scores with grades, rankings, biggest movers, and score distribution.
- **Tools** - recruitment tag calculator and a squad randomizer.
- **Asset Pipeline** - download and extract textures, audio, Spine animations, and game-data tables across six server regions.
- **Modern UI** - light / dark / auto theming with a customizable accent color, plus dynamically generated social cards.

## Architecture

```text
Arknights CDN (EN, JP, KR, CN, Bilibili, TW)
        │
        ▼
  assets/   Rust pipeline: downloader + unpacker
        │   (textures, audio, Spine, gamedata JSON)
        ▼
  backend/  Rust + Axum REST API ──► PostgreSQL (users, rosters, tier lists)
        │   game data, player sync,  └─► Redis (cache, optional)
        │   DPS/HPS, scoring, gacha
        ▼
 frontend/  TanStack Start (SSR React 19) web client
```

- **assets** downloads raw bundles from the Arknights CDN, then unpacks Unity assets and decodes FlatBuffer game-data tables to JSON.
- **backend** loads that game data into memory at startup, serves it over a REST API, handles Yostar authentication, and persists user data in PostgreSQL.
- **frontend** renders the operator database, profiles, tier lists, and tools, talking to the backend for all data and calculations.

## Repository Structure

```text
myrtle.moe/
├── backend/            Rust + Axum REST API (PostgreSQL, Redis, DPS/HPS engine)
├── frontend/           TanStack Start (SSR React 19) web client
├── assets/             Rust asset pipeline (downloader + unpacker) + FBS schemas
│   ├── downloader/     Multi-region CDN downloader with incremental updates
│   └── unpacker/       UnityFS bundle extractor and FlatBuffer decoder
├── .github/workflows/  Backend and assets CI pipelines
├── docker-compose.yml  Full local stack (db, cache, backend, frontend, tools)
├── BUILD.md            Manual (non-Docker) build instructions
└── CONTRIBUTING.md     Full developer setup and workflow
```

Each component has its own README with detailed documentation.

## Tech Stack

| Component | Stack |
|-----------|-------|
| Backend | Rust (Edition 2024), Axum 0.8, Tokio, SQLx + PostgreSQL, Redis, JWT |
| Frontend | TanStack Start (SSR), React 19, TypeScript 5.7, Tailwind CSS v4, Vite 7, Bun, Biome |
| Assets | Rust (Edition 2024) downloader + unpacker, Node.js interactive runner |

## Quick Start

The fastest path is Docker Compose, which runs PostgreSQL, Redis, the backend, and the frontend together.

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

### 3. Build the game assets

```bash
docker compose --profile tools build
docker compose run --rm asset-tools generate_fbs.sh
docker compose run --rm asset-tools download.sh en 4
docker compose run --rm asset-tools unpack.sh 4
```

### 4. Start the stack

```bash
docker compose build
docker compose up -d
```

The frontend binds to `http://localhost:3000` and the backend to `http://localhost:3060`.

For manual (non-Docker) setup, see [BUILD.md](BUILD.md) and the per-component READMEs.

## Documentation

| Component | Documentation |
|-----------|---------------|
| Backend API | [backend/README.md](backend/README.md) |
| Frontend | [frontend/README.md](frontend/README.md) |
| Asset Pipeline | [assets/README.md](assets/README.md) |
| Developer Setup | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Manual Build | [BUILD.md](BUILD.md) |
| DPS Calculator | [backend/DPS_CALCULATOR.md](backend/DPS_CALCULATOR.md) |

## Contributing

Contributions are welcome. Clone the repo with submodules, follow the setup in [CONTRIBUTING.md](CONTRIBUTING.md), and run each component's lint, format, and test checks before opening a pull request:

- Backend: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
- Frontend: `bun --bun run check`, `bun --bun run test`

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
