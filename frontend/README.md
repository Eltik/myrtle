# Myrtle Frontend

The web client for [myrtle.moe](https://myrtle.moe) - an Arknights companion platform for browsing operators, tracking your roster, planning your base, building tier lists, and scouting community pulls. Built with TanStack Start (SSR React) and powered by the [Myrtle backend](../backend/README.md).

[![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=white)](https://react.dev/)
[![TanStack Start](https://img.shields.io/badge/TanStack-Start-ff4154)](https://tanstack.com/start)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178c6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4-38bdf8?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![Bun](https://img.shields.io/badge/Bun-1.3.12-000000?logo=bun&logoColor=white)](https://bun.sh/)
[![Biome](https://img.shields.io/badge/Biome-2.4-60a5fa?logo=biome&logoColor=white)](https://biomejs.dev/)
[![Frontend CI](https://github.com/Eltik/myrtle/actions/workflows/frontend-ci.yml/badge.svg)](https://github.com/Eltik/myrtle/actions/workflows/frontend-ci.yml)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

## Features

- **Operator Browser** - search and filter every operator, with detail pages for stats, skills, modules, attack ranges, skins, voice lines, and base skills.
- **Dynamic Illustrations (L2D)** - Arknights' animated operator art rendered in the browser: entrance cinematics, particle effects, HDR tonemapping, camera tracks, and scene-mesh backdrops, reconstructed with PixiJS and pixi-spine.
- **Chibi Viewer** - idle, walk, and attack Spine animations, exportable as GIF or MP4.
- **Stage Viewer** - every stage as a tile map with enemy pathing, spawn schedule, waves, and drops.
- **Enemy Database** - enemy stats, abilities, chibis, and the stages they appear in.
- **Base Planner & Optimizer** - RIIC layout optimization inside your profile: dormitory morale, shift rotations, per-skill contribution ledgers, and multi-day facility economics.
- **Operator Planner** - promotion, level, skill, and module targets grouped into plans.
- **DPS & HPS Charts** - interactive damage and healing curves per skill against DEF, RES, and target count, rendered with Recharts.
- **Recruitment Calculator** - guaranteed tag combinations with 1-hour parity.
- **Randomizer** - roll a random squad to break out of the meta.
- **Birthdays** - an operator birthday calendar.
- **Tier Lists** - browse official and community tier lists, or build and publish your own with the drag-and-drop editor.
- **Doctor Profiles** - log in to sync your roster and inventory, then view profiles, search players, and climb the score-based leaderboard.
- **Gacha Tracking** - synced pull history with rarity splits and pity counters, plus community-wide pull statistics.
- **Changelog** - rendered live from the project's GitHub commit history.
- **Admin Panel** - user and permission management, health monitoring, audit logs, operator notes, and official tier-list curation.
- **Dynamic OG Images** - per-page social cards generated on the fly with Satori and resvg.
- **Theming** - light, dark, and auto modes with a customizable accent color.

## Tech Stack

| Component      | Technology                                                          |
|----------------|---------------------------------------------------------------------|
| Framework      | TanStack Start (SSR) on Nitro                                       |
| UI Library     | React 19                                                            |
| Routing        | TanStack Router (file-based)                                        |
| Data Fetching  | TanStack Query + Router loaders                                     |
| State          | TanStack Store                                                      |
| Styling        | Tailwind CSS v4, shadcn/ui (new-york), Base UI, lucide-react         |
| Build Tool     | Vite 7                                                              |
| Language       | TypeScript 5.7 (strict)                                             |
| Validation     | Zod v4, T3Env for typed environment variables                       |
| Graphics       | PixiJS 7 + pixi-spine (chibis and L2D), gif.js + mp4-muxer (export) |
| Charts         | Recharts                                                            |
| Tooling        | Bun 1.3.12, Biome (lint + format)                                   |

TanStack packages track the `latest` tag rather than a pinned version.

## Prerequisites

| Tool   | Version  | Notes                                              |
|--------|----------|----------------------------------------------------|
| Bun    | 1.3.12   | Package manager and runtime, pinned via `packageManager` |
| Backend| -        | A running [Myrtle backend](../backend/README.md), or point at `https://api.myrtle.moe` |

Rust is only needed if you regenerate the API types (see [Generated Types](#generated-types)).

## Getting Started

```bash
# Install dependencies
bun install

# Start the dev server on http://localhost:3000
bun --bun run dev
```

> The `postinstall` step patches `@base-ui/utils` by fetching a known-good tarball if it is missing. This is expected.

## Configuration

Copy the example environment file and adjust the values:

```bash
cp .env.example .env
```

| Variable           | Required | Description                                                        |
|--------------------|----------|--------------------------------------------------------------------|
| `BACKEND_URL`      | Yes      | Backend base URL used by server-side functions.                    |
| `VITE_BACKEND_URL` | Yes      | Backend base URL exposed to the client.                            |
| `VITE_SITE_URL`    | Yes      | Public site URL, used for canonical links and OG images.           |
| `VITE_APP_TITLE`   | No       | Override for the application title.                                |
| `GITHUB_REPO`      | No       | Source repo for the changelog page (`owner/repo`). Defaults to `Eltik/myrtle`. |
| `GITHUB_TOKEN`     | No       | Raises the GitHub API rate limit (60 -> 5000 req/hr).              |
| `GITHUB_BRANCH`    | No       | Pin a branch instead of the repo's default.                       |

Environment variables are validated at runtime via T3Env in `src/env.ts`. The three URL variables are declared optional in the schema but are required in practice. Use them in code through the typed `env` export:

```ts
import { env } from "#/env";

console.log(env.VITE_SITE_URL);
```

`DYNCHAR_ALT_ROOT` is a dev-only extra: a Vite plugin serves that directory read-only under `/altassets/`, so you can test alternate asset exports (paired with `?assetroot=`) without touching the backend's asset tree. It never reaches a production build.

## Development

```bash
bun --bun run dev      # Start the dev server (port 3000)
bun --bun run lint     # Lint with Biome
bun --bun run format   # Format with Biome
bun --bun run check    # Lint + format check with Biome
bunx tsc --noEmit      # Type-check
bun --bun run test     # Vitest
```

Biome is configured for 4-space indent, 320-character lines, double quotes, organized imports, and sorted Tailwind classes.

> **There are no tests yet.** Vitest, Testing Library, and jsdom are installed and wired into CI, but the suite is empty - CI runs `vitest run --passWithNoTests`. Verify changes with `tsc`, Biome, and the browser.

### Routing

Routes use [TanStack Router](https://tanstack.com/router) file-based routing in `src/routes`. Add a file there and the route tree regenerates automatically. The root layout lives in `src/routes/__root.tsx`; authenticated pages sit under `src/routes/_authed.tsx`, which redirects to `/?auth=1&next=...` when there is no user.

### Path Aliases

`#/*` and `@/*` both resolve to `src/*`:

```ts
import { cn } from "#/lib/utils";
```

## Generated Types

Every API type is generated from the Rust backend with `ts-rs` and lands in `src/types/generated/` (341 files). Do not edit them, and do not hand-write a duplicate: change the Rust struct instead.

```bash
bun run gen:types   # Regenerate the bindings from ../backend
bun run gen:check   # Fail if the checked-in bindings are stale
```

`gen:types` deletes `src/types/generated`, then runs `cargo test export_bindings` and `cargo test --test bindings_shape_test` in `../backend`. The shape test catches two errors `ts-rs` makes silently: `bigint` for `i64`/`u64` (JSON gives plain `number`) and bare `any` where the recursive `JsonValue` type belongs.

The generated types are committed so the frontend builds without a Rust toolchain. The hand-written wrappers in `src/types/*.ts` sit on top and apply a `Camelize<T>` transform, because operator and stage endpoints serve the game's original PascalCase keys.

## Building for Production

```bash
# Build the application (heap is raised to avoid an OOM during build)
bun --bun run build

# Run the production server
bun --bun run start
```

The build emits Nitro output to `.output/` (`.output/server/index.mjs` plus `.output/public/`). `bun run start` serves it on `PORT`, default 3000. `ecosystem.config.cjs` runs the same entry point directly under PM2, avoiding an extra fork layer.

The root `dist/` directory is a stale artifact, not the build output.

### Docker

```bash
# Production
docker build -t myrtle-frontend \
  --build-arg VITE_BACKEND_URL=https://api.myrtle.moe \
  --build-arg VITE_SITE_URL=https://myrtle.moe .
docker run -p 3000:3000 myrtle-frontend

# Development
docker build -f Dockerfile.dev -t myrtle-frontend-dev .
docker run -p 3000:3000 -v $(pwd):/src myrtle-frontend-dev
```

`VITE_*` values are compiled into the client bundle, so they are build arguments rather than runtime environment variables.

## Project Structure

```
src/
├── routes/         File-based routes (pages + API/OG endpoints)
├── components/     Feature UI grouped by domain
│   ├── operators/  Operator list and detail, incl. chibi/ and dynillust/ renderers
│   ├── stages/     Stage list plus the tile map and pathing engine
│   ├── enemies/    Enemy database
│   ├── tools/      DPS, HPS, planner, recruitment, randomizer, birthdays
│   ├── tier-lists/ Browsing, detail, "my lists", and the drag-and-drop editor
│   ├── gacha/      Pull history and community stats
│   ├── user/       Profiles (incl. base optimizer), search, and leaderboard
│   ├── admin/      Admin panel screens
│   ├── settings/   Account and appearance settings
│   ├── header/ home/ stats/ changelog/ legal/ export/
│   └── ui/         shadcn/ui primitives
├── lib/            API clients, auth, base planner, theming, search, markdown, OG
├── hooks/          Reusable React hooks
├── integrations/   TanStack Query client and devtools wiring
├── types/          Hand-written wrappers over types/generated/ (ts-rs output)
└── env.ts          Typed, validated environment variables
```

`ds-bundle/`, `.design-sync/`, and `.ds-sync/` are local design-system tooling that packages `src/components` for use in an external design canvas. They are gitignored and are not part of a clone.

## Dynamic Illustrations

`src/components/operators/detail/impl/components/dynillust/` is the largest subsystem in the codebase, and the single biggest body of work since the previous revision of this README. It renders Arknights' Spine-based animated operator art - entrance cinematics, particle systems, shader families, HDR tonemapping and bloom, camera tracks, and layered scene meshes - matched frame by frame against the real client.

Because that matching was empirical, most behaviours ship behind a URL query flag that reverts them, for example `?knee=` (tonemap compression knee), `?settlecam=0` (settled framing), and `?batchpool=0` (PixiJS same-buffer batch uploads). There are dozens of these. They are measurement instruments and revert switches, not user-facing configuration, and none of them are stable API.

`/dyntest` is a standalone debug route that loads the renderer against an arbitrary `.skel`/`.atlas`/`.png` via query parameters, bypassing the operator API entirely.

## Architecture

This is the frontend only. It talks to the separate Rust [Myrtle backend](../backend/README.md) (Axum + PostgreSQL + Redis) for all game data, player sync, calculations, and storage. Authentication uses a Yostar email-code flow for Global (EN), Japan (JP), and Korea (KR), with CN passport and Bilibili login also supported.

## Contributing

Contributions are welcome. Before opening a pull request:

1. Run `bun --bun run check` so Biome lint and format pass.
2. Run `bunx tsc --noEmit` so the type-check passes.
3. If you changed a backend type, run `bun run gen:types` and commit the regenerated bindings.

CI ([`frontend-ci.yml`](../.github/workflows/frontend-ci.yml)) runs Biome, `tsc --noEmit`, Vitest, a production build verifying `.output/` exists, and a non-blocking `bun audit`.

## License

License TBD. See the [root repository](../README.md) for details.
