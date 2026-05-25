# Myrtle Frontend

The web client for [myrtle.moe](https://myrtle.moe) - an Arknights companion platform for browsing operators, tracking your roster, building tier lists, and scouting community pulls. Built with TanStack Start (SSR React) and powered by the [Myrtle backend](../backend/README.md).

[![React](https://img.shields.io/badge/React-19-61dafb?logo=react&logoColor=white)](https://react.dev/)
[![TanStack Start](https://img.shields.io/badge/TanStack-Start-ff4154)](https://tanstack.com/start)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178c6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-v4-38bdf8?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![Bun](https://img.shields.io/badge/Bun-1.3-000000?logo=bun&logoColor=white)](https://bun.sh/)
[![Biome](https://img.shields.io/badge/Biome-2.4-60a5fa?logo=biome&logoColor=white)](https://biomejs.dev/)
[![License](https://img.shields.io/badge/License-TBD-lightgrey)](../README.md)

## Features

- **Operator Browser** - search and filter every operator, with detail pages for stats, skills, modules, attack ranges, skins, voice lines, and Spine chibi animations.
- **DPS & HPS Charts** - interactive damage and healing curves per skill, rendered with Recharts.
- **Recruitment Calculator** - guaranteed tag combinations with 1-hour parity.
- **Randomizer** - roll a random squad to break out of the meta.
- **Tier Lists** - browse official and community tier lists, or build and publish your own with the drag-and-drop editor.
- **Doctor Profiles** - log in to sync your roster and inventory, then view profiles, search players, and climb the score-based leaderboard.
- **Gacha Tracking** - synced pull history with rarity splits and pity counters, plus community-wide pull statistics.
- **Enemy Database** - browse enemy stats and abilities.
- **Changelog** - rendered live from the project's GitHub commit history.
- **Admin Panel** - user and permission management, health monitoring, audit logs, operator notes, and official tier-list curation.
- **Dynamic OG Images** - per-page social cards generated on the fly with Satori and resvg.
- **Theming** - light, dark, and auto modes with a customizable accent color.

## Tech Stack

| Component      | Technology                                                        |
|----------------|-------------------------------------------------------------------|
| Framework      | TanStack Start (SSR) on Nitro                                      |
| UI Library     | React 19                                                          |
| Routing        | TanStack Router (file-based)                                       |
| Data Fetching  | TanStack Query + Router loaders                                    |
| State          | TanStack Store                                                     |
| Styling        | Tailwind CSS v4, shadcn/ui (new-york), Base UI, lucide-react       |
| Build Tool     | Vite 7                                                             |
| Language       | TypeScript 5.7 (strict)                                            |
| Validation     | Zod v4, T3Env for typed environment variables                     |
| Graphics       | PixiJS + pixi-spine (chibis), gif.js + mp4-muxer (animation export)|
| Charts         | Recharts                                                          |
| Tooling        | Bun, Biome (lint + format), Vitest                                |

## Prerequisites

| Tool   | Version  | Notes                                              |
|--------|----------|----------------------------------------------------|
| Bun    | 1.3.12+  | Package manager and runtime                        |
| Backend| -        | A running [Myrtle backend](../backend/README.md), or point at `https://api.myrtle.moe` |

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

Environment variables are validated at runtime via T3Env in `src/env.ts`. Use them in code through the typed `env` export:

```ts
import { env } from "#/env";

console.log(env.VITE_SITE_URL);
```

## Development

```bash
bun --bun run dev      # Start the dev server (port 3000)
bun --bun run test     # Run tests with Vitest
bun --bun run lint     # Lint with Biome
bun --bun run format   # Format with Biome
bun --bun run check    # Lint + format check with Biome
```

### Routing

Routes use [TanStack Router](https://tanstack.com/router) file-based routing in `src/routes`. Add a file there and the route tree regenerates automatically. The root layout lives in `src/routes/__root.tsx`.

### Path Aliases

`#/*` and `@/*` both resolve to `src/*`:

```ts
import { cn } from "#/lib/utils";
```

## Building for Production

```bash
# Build the application
bun --bun run build

# Run the production server
bun --bun run start
```

## Project Structure

```
src/
├── routes/         File-based routes (pages + API/OG endpoints)
├── components/     Feature UI grouped by domain
│   ├── operators/  Operator list and detail views
│   ├── tools/      DPS, HPS, recruitment, and randomizer
│   ├── tier-lists/ Tier-list browsing and editing
│   ├── gacha/      Pull history and community stats
│   ├── user/       Profiles, search, and leaderboard
│   ├── admin/      Admin panel screens
│   └── ui/         shadcn/ui primitives
├── lib/            API clients, auth, theming, search, OG, scoring
├── hooks/          Reusable React hooks
├── types/          Shared TypeScript types
└── env.ts          Typed, validated environment variables
```

## Architecture

This is the frontend only. It talks to the separate Rust [Myrtle backend](../backend/README.md) (Axum + PostgreSQL + Redis) for all game data, player sync, calculations, and storage. Authentication uses a Yostar email-code flow; sign-in is available for the Global (EN), Japan (JP), and Korea (KR) servers.

## Contributing

Contributions are welcome. Before opening a pull request, run `bun --bun run check` to ensure linting and formatting pass, and `bun --bun run test` for the test suite.

## License

License TBD. See the [root repository](../README.md) for details.
