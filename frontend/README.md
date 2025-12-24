# Myrtle.moe Frontend

A modern, high-performance Next.js frontend for Arknights game data, player profiles, and tools. Built with React 19, TypeScript, Tailwind CSS v4, and shadcn/ui components with motion primitives for smooth animations.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Pages](#pages)
- [Components](#components)
  - [Layout Components](#layout-components)
  - [Operator Components](#operator-components)
  - [User Components](#user-components)
  - [Tool Components](#tool-components)
  - [UI Components](#ui-components)
- [API Routes](#api-routes)
- [Hooks](#hooks)
- [Types](#types)
- [Utilities](#utilities)
- [Configuration](#configuration)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Features

### Core Features

| Feature | Description |
|---------|-------------|
| Operator Database | Browse 200+ operators with detailed stats, skills, talents, and modules |
| Player Profiles | View player data synced from Arknights servers via Yostar authentication |
| Recruitment Calculator | Calculate optimal tag combinations for operator recruitment |
| Responsive Design | Mobile-first design with breakpoints for all screen sizes |
| Dark Theme | Native dark theme with OKLCH color system |
| Spine Animations | Live chibi/dorm animations using PixiJS + Spine |
| Voice Lines | Multilingual voice line playback (CN, JP, EN, KR) |

### Technical Features

| Feature | Description |
|---------|-------------|
| Next.js 15 | App uses Pages Router with Turbopack for fast development |
| React 19 | Latest React with improved performance |
| TypeScript | Full type safety with strict mode |
| Tailwind CSS v4 | Modern utility-first CSS with CSS variables |
| Motion Primitives | Framer Motion-based animation components |
| shadcn/ui | Radix-based accessible UI components |
| Biome | Fast linting and formatting |
| Server-Side Rendering | All data pages use getServerSideProps |

## Installation

### Prerequisites

- **Bun** 1.0+ (recommended) or Node.js 20+
- **Backend Server** running at configured BACKEND_URL

### Install Bun

```bash
# macOS/Linux
curl -fsSL https://bun.sh/install | bash

# Windows
powershell -c "irm bun.sh/install.ps1 | iex"
```

### Clone and Install

```bash
# Clone the repository
git clone https://github.com/Eltik/myrtle.moe.git
cd myrtle.moe/frontend

# Install dependencies
bun install

# Copy environment file
cp .env.example .env
```

### Configure Environment

Edit `.env` with your backend URL:

```env
BACKEND_URL="http://localhost:3060"
```

## Quick Start

```bash
# Development server with Turbopack
bun run dev

# Production build
bun run build

# Start production server
bun run start

# Type checking
bun run typecheck

# Lint and format
bun run check
bun run lint:fix
```

The development server runs at `http://localhost:3000`.

## Project Structure

```
frontend/
├── src/
│   ├── components/           # React components
│   │   ├── home/            # Homepage components
│   │   ├── layout/          # Header, footer, navigation
│   │   ├── operators/       # Operator list and detail views
│   │   ├── tools/           # Recruitment calculator, etc.
│   │   ├── user/            # Player profile components
│   │   └── ui/              # Base UI components
│   │       ├── shadcn/      # shadcn/ui components
│   │       └── motion-primitives/  # Animation components
│   ├── hooks/               # Custom React hooks
│   ├── lib/                 # Utility functions
│   ├── pages/               # Next.js pages and API routes
│   │   ├── api/             # API endpoints
│   │   ├── operators/       # Operator pages
│   │   ├── tools/           # Tool pages
│   │   └── user/            # User profile pages
│   ├── styles/              # Global CSS
│   └── types/               # TypeScript types
├── public/                  # Static assets
├── biome.jsonc             # Biome config
├── components.json         # shadcn/ui config
├── next.config.js          # Next.js config
├── package.json            # Dependencies
└── tsconfig.json           # TypeScript config
```

## Pages

| Route | Page | Description |
|-------|------|-------------|
| `/` | Home | Landing page with bento grid layout |
| `/operators` | Operators List | Browse all operators with filters |
| `/operators/[id]` | Operator Detail | Detailed operator information with tabs |
| `/user/[id]` | User Profile | Player profile with characters, items, base |
| `/tools/recruitment` | Recruitment Calculator | Calculate tag combinations |

### Data Fetching

All data pages use `getServerSideProps` for server-side rendering:

```typescript
export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const response = await fetch(`${env.BACKEND_URL}/static/operators`);
    const data = await response.json();
    return { props: { data: data.operators } };
};
```

## Components

### Layout Components

Located in `src/components/layout/`:

| Component | Purpose |
|-----------|---------|
| `Layout` | Root wrapper with header, footer, gradient backgrounds |
| `Header` | Navigation bar with desktop/mobile variants |
| `NavDesktop` | Desktop navigation with dropdowns and hover effects |
| `NavMobile` | Mobile slide-out sheet navigation |
| `UserMenu` | User account dropdown with avatar |
| `Login` | Authentication dialog with OTP flow |
| `Footer` | Page footer with links and social icons |

### Operator Components

Located in `src/components/operators/`:

#### List Components (`operators/list/`)

| Component | Purpose |
|-----------|---------|
| `OperatorsList` | Main list container with pagination and view modes |
| `OperatorCard` | Individual operator card (grid/list variants) |
| `OperatorFilters` | Filter controls (class, rarity, faction, etc.) |
| `useOperatorFilters` | Filter state management hook |

#### Detail Components (`operators/detail/`)

| Component | Purpose |
|-----------|---------|
| `OperatorDetail` | Main detail page wrapper |
| `OperatorHero` | Parallax hero section with operator image |
| `OperatorTabs` | Tab navigation (Info, Skills, Level-Up, Skins, Audio, Lore) |

#### Tab Content Components

| Component | Purpose |
|-----------|---------|
| `InfoContent` | Stats, traits, talents, modules with interactive controls |
| `SkillsContent` | Skill details with level comparison mode |
| `LevelUpContent` | Elite/skill/module upgrade costs |
| `SkinsContent` | Skin gallery with chibi preview |
| `AudioContent` | Voice line player with language selection |
| `LoreContent` | Operator lore and archive files |
| `ChibiViewer` | PixiJS Spine animation renderer |

### User Components

Located in `src/components/user/`:

| Component | Purpose |
|-----------|---------|
| `UserHeader` | Profile header with avatar and stats |
| `CharactersGrid` | Operator collection with filters and sorting |
| `CharacterCard` | Individual character with stats and dialog |
| `ItemsGrid` | Inventory table with search and details |
| `BaseView` | Base facilities (trading posts, factories) |

### Tool Components

Located in `src/components/tools/`:

| Component | Purpose |
|-----------|---------|
| `RecruitmentCalculator` | Tag selection and result calculation |
| `TagSelector` | Interactive tag buttons by category |
| `ResultsList` | Sorted recruitment outcomes |
| `OperatorResultCard` | Individual operator result display |

### UI Components

#### shadcn/ui (`src/components/ui/shadcn/`)

40+ accessible components including: Accordion, Alert, Avatar, Badge, Button, Card, Checkbox, Command, Dialog, Dropdown, Form, Input, Label, Popover, Progress, Select, Sheet, Skeleton, Slider, Switch, Table, Tabs, Toast, Tooltip, and more.

#### Motion Primitives (`src/components/ui/motion-primitives/`)

25+ animation components including: AnimatedGroup, AnimatedNumber, BorderTrail, Carousel, Disclosure, Dock, GlowEffect, ImageComparison, InfiniteSlider, InView, Magnetic, MorphingDialog, ProgressiveBlur, ScrollProgress, SlidingNumber, Spotlight, TextEffect, TextLoop, TextMorph, TextScramble, TextShimmer, Tilt, TransitionPanel, and more.

## API Routes

### Authentication

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/auth/send-code` | POST | Send verification code to email |
| `/api/auth/login` | POST | Authenticate with email + code |
| `/api/auth/me` | POST | Get current user from session |
| `/api/auth/logout` | POST | Clear session cookies |

#### Authentication Flow

```typescript
// 1. Request verification code
POST /api/auth/send-code
{ email: "user@example.com", server: "en" }

// 2. Login with code
POST /api/auth/login
{ email: "user@example.com", code: "123456", server: "en" }

// 3. Check session
POST /api/auth/me
// Returns user data if authenticated
```

### Static Data

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/static` | POST | Fetch game data by type |

Supported types: `materials`, `modules`, `operators`, `ranges`, `skills`, `trust`, `handbook`, `skins`, `voices`, `gacha`, `chibis`

```typescript
// Example: Fetch operator
POST /api/static
{ type: "operators", id: "char_002_amiya" }

// Example: Recruitment calculation
POST /api/static
{ type: "gacha", method: "calculate", tags: ["1", "14", "17"] }
```

### CDN Proxy

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/cdn/[...path]` | GET | Proxy requests to backend CDN |

```
GET /api/cdn/upk/chararts/char_002_amiya/char_002_amiya_1.png
GET /api/cdn/avatar/char_002_amiya
```

## Hooks

Located in `src/hooks/`:

### useAuth

Manages authentication state with localStorage caching for instant UI display.

```typescript
const { user, loading, login, logout, fetchUser } = useAuth();

// Login
await login("user@email.com", "123456", "en");

// Logout
await logout();
```

### useIsMobile

Responsive breakpoint detection (768px).

```typescript
const isMobile = useIsMobile();
// Returns true if viewport < 768px
```

### useClickOutside

Detects clicks outside a referenced element.

```typescript
const ref = useRef<HTMLDivElement>(null);
useClickOutside(ref, () => setOpen(false));
```

## Types

Located in `src/types/`:

### API Types (`types/api/`)

| Type | Description |
|------|-------------|
| `Operator` | Full operator data with phases, skills, talents, modules |
| `Skill` | Skill with levels, SP data, blackboard values |
| `Module` | Equipment module with phases and stat bonuses |
| `Skin` | Cosmetic skin with display and battle data |
| `Item` | Material/consumable with drop and production info |
| `Voice` | Voice line with multilingual support |
| `Handbook` | Lore and profile data |
| `User` | Complete player account state |
| `ChibiCharacter` | Spine animation data |

### Frontend Types (`types/frontend/`)

| Type | Description |
|------|-------------|
| `OperatorFromList` | Lightweight operator for list views |
| `NormalizedRange` | Grid-based attack range |
| `UISkin` | Processed skin for UI display |
| `VoiceLine` | Formatted voice line with URL |

## Utilities

Located in `src/lib/`:

### utils.ts

| Function | Description |
|----------|-------------|
| `cn()` | Merge Tailwind classes with conflict resolution |
| `rarityToNumber()` | Convert rarity enum to number (1-6) |
| `formatProfession()` | Format profession code to display name |
| `formatSubProfession()` | Format archetype code to display name |
| `formatNationId()` | Format nation code to display name |
| `getOperatorImageUrl()` | Build operator image URL from skin/phase |
| `getAvatarById()` | Get GitHub avatar URL for character |

### operator-stats.ts

| Function | Description |
|----------|-------------|
| `getOperatorAttributeStats()` | Calculate full stats with trust/potential/module |
| `getStatIncreaseAtTrust()` | Calculate trust stat bonuses |
| `getStatIncreaseAtPotential()` | Calculate potential stat bonuses |
| `getModuleStatIncrease()` | Calculate module stat bonuses |

### description-parser.tsx

| Function | Description |
|----------|-------------|
| `descriptionToHtml()` | Parse skill descriptions with color tags and interpolation |
| `preprocessDescription()` | Balance unbalanced HTML-like tags |

### skill-helpers.ts

| Function | Description |
|----------|-------------|
| `computeSkillDiff()` | Compare two skill levels for differences |
| `formatBlackboardValue()` | Format stat values with percentages |
| `formatSkillLevel()` | Format level index to "Lv.1" or "M1" |
| `getSpTypeLabel()` | Get SP recovery type label |

### operator-helpers.ts

| Function | Description |
|----------|-------------|
| `formatOperatorDescription()` | Format trait with blackboard interpolation |
| `getActiveTalentCandidate()` | Get unlocked talent based on progress |
| `phaseToIndex()` | Convert phase enum to index |

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BACKEND_URL` | Backend API URL | `http://localhost:3060` |
| `NODE_ENV` | Environment mode | `development` |
| `SKIP_ENV_VALIDATION` | Skip env validation (for Docker) | - |

### next.config.js

```javascript
const config = {
    reactStrictMode: true,
    i18n: { locales: ["en"], defaultLocale: "en" },
    images: {
        remotePatterns: [/* Allow all HTTPS domains */]
    }
};
```

### tsconfig.json

- Target: ES2022
- Strict mode enabled
- Path alias: `~/*` → `./src/*`
- Verbatim module syntax

### biome.jsonc

- Indent: 4 spaces
- Line width: 320
- Semicolons: always
- Double quotes
- Organized imports
- Sorted Tailwind classes

### components.json (shadcn/ui)

- Style: new-york
- Base color: neutral
- CSS variables enabled
- Icon library: lucide

## Development

### Scripts

```bash
# Development with Turbopack
bun run dev

# Production build
bun run build

# Start production server
bun run start

# Type checking
bun run typecheck

# Lint
bun run lint
bun run lint:fix

# Format
bun run format

# Check all (lint + format)
bun run check
bun run check:write
```

## Troubleshooting

### Common Issues

#### "BACKEND_URL is required"

Ensure `.env` file exists with valid `BACKEND_URL`:

```bash
cp .env.example .env
# Edit .env with your backend URL
```

#### Build fails with type errors

Run type check to see all errors:

```bash
bun run typecheck
```

#### Styles not applying

Ensure Tailwind is processing your files:

```bash
# Check globals.css imports
cat src/styles/globals.css
```

#### Authentication not working

1. Check backend is running at BACKEND_URL
2. Verify cookies are being set (check browser dev tools)
3. Clear localStorage cache: `localStorage.removeItem('myrtle_user_cache')`

#### Chibi animations not loading

1. Check browser console for CORS errors
2. Verify Spine files exist at CDN path
3. Check PixiJS compatibility with your browser

### Debug Mode

Enable verbose logging:

```bash
# Development with debug output
NODE_OPTIONS='--inspect' bun run dev
```

## Related Projects

- [backend](../backend) - Rust backend API server
- [downloader](../assets/downloader) - Asset downloader tool
- [unpacker](../assets/unpacker) - Asset extraction tool
- [unity-rs](../assets/unity-rs) - Rust Unity asset parser

## License

This project is for educational purposes. All game assets belong to Hypergryph/Yostar.

---

**Note**: This is an unofficial fan project and is not affiliated with Hypergryph or Yostar.
