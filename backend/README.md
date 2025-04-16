# Backend

A robust Arknights API inspired by [ArkPRTS](https://github.com/thesadru/ArkPRTS), built with modern TypeScript and Bun runtime. This backend service provides comprehensive data and calculations for the Arknights game, including operator statistics, DPS calculations, and game asset management.

## Table of Contents
- [Features](#features)
- [Prerequisites](#prerequisites)
  - [Core Requirements](#core-requirements)
  - [Asset Management Requirements](#asset-management-requirements)
  - [Development Tools](#development-tools)
- [Installation](#installation)
- [Development](#development)
  - [Available Scripts](#available-scripts)
  - [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Dependencies](#dependencies)
- [Testing](#testing)
- [CDN Service](#cdn-service)
  - [CDN Features](#cdn-features)
  - [CDN Usage](#cdn-usage)
  - [Cache Configuration](#cache-configuration)
  - [CDN Monitoring](#cdn-monitoring)
  - [CDN Security](#cdn-security)
  - [CDN Environment Configuration](#cdn-environment-configuration)
  - [Supported File Types](#supported-file-types)
- [License](#license)
- [Contributing](#contributing)

## Features

- **Modern TypeScript-based API server**: Built with [Bun runtime](https://bun.sh/) for optimal performance
- **[Redis](https://redis.io/) caching**: Implements efficient caching for frequently accessed data
- **[PostgreSQL](https://www.postgresql.org/) database**: Robust data storage and management
- **Asset management**: Complete pipeline for downloading, unpacking, and processing Arknights assets
- **CDN service**: Secure delivery of game assets with intelligent caching
- **DPS Calculator**: Comprehensive damage calculation system for operators
- **Event-driven architecture**: Efficient event handling and data processing
- **Data import/export**: Tools for managing game data

## Prerequisites

### Core Requirements
- [Bun](https://bun.sh/) (Latest version) - Primary runtime
- [Node.js](https://nodejs.org/) 18+ - Required for development tools
- [PostgreSQL](https://www.postgresql.org/) - Database server
- [Redis](https://redis.io/) - Caching server

### Asset Management Requirements
- [Python 3.8+](https://www.python.org/downloads/) - Required for asset downloading
- Python Dependencies:
  - [`requests`](https://pypi.org/project/requests/) - For HTTP requests
  - [`tqdm`](https://pypi.org/project/tqdm/) - For progress bars
  - [`beautifulsoup4`](https://pypi.org/project/beautifulsoup4/) - For HTML parsing
  - [`lxml`](https://pypi.org/project/lxml/) - For XML parsing
  - [`pillow`](https://pypi.org/project/Pillow/) - For image processing
  - [`cryptography`](https://pypi.org/project/cryptography/) - For asset verification
- Sufficient disk space for game assets (~10GB recommended)

### Development Tools
- [Git](https://git-scm.com/) - Version control
- A code editor with TypeScript support ([VS Code](https://code.visualstudio.com/) recommended)

## Installation

1. Move into the `./backend` directory:
```bash
cd backend
```

2. Install dependencies:
```bash
bun install
```

3. Install Python dependencies:
```bash
pip install requests tqdm beautifulsoup4 lxml pillow cryptography
```

4. Set up environment configuration:
```bash
cp .env.example .env
```

5. Configure the following environment variables in `.env`:
- `PORT`: Server port (default: 3060)
- `LOAD_AK_CONFIG`: Whether to load Arknights configuration
- `REDIS_URL`: Redis connection URL
- `DATABASE_URL`: PostgreSQL connection URL
- `REDIS_CACHE_TIME`: Cache duration in seconds (default: 3600)

## Development

### Available Scripts

#### Core Development
- `bun run dev` - Start development server with hot reload
- `bun run build` - Build the project for production
- `bun run start` - Start the production server
- `bun run lint` - Run linting and formatting

#### Asset Management
- `bun run download` - Download Arknights assets using Python script
  - Options:
    - `--quiet`: Less verbose output
    - `--help`: Show help message
- `bun run unpack` - Unpack downloaded assets
- `bun run export` - Export processed data
- `bun run import` - Import data into the database

#### Testing
- `bun run test:dps` - Run DPS calculation tests
  - Tests operator damage calculations
  - Validates operator skill mechanics
  - Checks operator talent interactions

### Project Structure

```
backend/
├── src/
│   ├── app/         # Application logic and API endpoints
│   ├── database/    # Database models, migrations, and queries
│   ├── events/      # Event system and handlers
│   ├── helper/      # Utility functions and helpers
│   ├── lib/         # Core libraries and implementations
│   │   ├── impl/
│   │   │   ├── dps-calculator/  # DPS calculation engine
│   │   │   ├── local/          # Local data management
│   │   │   └── assets/         # Asset processing
│   ├── scripts/     # Utility scripts for data management
│   ├── tests/       # Test files and test utilities
│   └── types/       # TypeScript type definitions
├── dist/            # Compiled output
├── data/            # Processed game data
├── exports/         # Data export directory
├── ArkAssets/       # Downloaded game assets
└── unpacked/        # Unpacked game assets
```

## Configuration

The project uses several configuration files for different aspects of development:

- `tsconfig.json` - TypeScript configuration and compiler options
- `.prettierrc.json` - Code formatting rules and style preferences
- `eslint.config.mjs` - Linting rules and code quality standards
- `.env` - Environment variables and runtime configuration
- `.prettierignore` - Files to exclude from formatting
- `.gitignore` - Files to exclude from version control

## Dependencies

### Main Dependencies
- [`cheerio`](https://github.com/cheeriojs/cheerio) - HTML parsing for web scraping
- [`crypto-js`](https://github.com/brix/crypto-js) - Cryptographic operations for asset verification
- [`ioredis`](https://github.com/luin/ioredis) - Redis client for caching
- [`pg`](https://node-postgres.com/) - PostgreSQL client for database operations
- [`zod`](https://github.com/colinhacks/zod) - Schema validation for data integrity
- [`eventemitter2`](https://github.com/EventEmitter2/EventEmitter2) - Event handling system
- [`colors`](https://github.com/Marak/colors.js) - Terminal output formatting
- [`mime-types`](https://github.com/jshttp/mime-types) - MIME type detection for CDN service

### Development Dependencies
- [TypeScript](https://www.typescriptlang.org/) - Type safety and modern JavaScript features
- [ESLint](https://eslint.org/) - Code linting and quality control
- [Prettier](https://prettier.io/) - Code formatting
- [Bun Types](https://bun.sh/docs/typescript) - TypeScript definitions for Bun

## Testing

The project includes comprehensive testing, particularly for the DPS calculator:

```bash
bun run test:dps
```

The DPS tests verify:
- Operator damage calculations
- Skill mechanics and interactions
- Talent effects and combinations
- Equipment and buff calculations
- Enemy defense and resistance handling

## CDN Service

The backend includes a modern and secure CDN service for serving assets from the `unpacked` directory.

### CDN Features

- **Path Validation**: Protection against path traversal attacks
- **Security**: Restricted file types and proper validation
- **Caching**: Intelligent cache control based on file types
- **Conditional Requests**: Support for If-Modified-Since headers
- **Metrics**: Built-in request tracking and monitoring
- **Performance**: Optimized for speed with Bun's file handling

### CDN Usage

The CDN service is available at:

```
/cdn/<asset_path>
```

For example, to serve an image at `/unpacked/images/my-image.png`:

```
https://your-domain.com/cdn/images/my-image.png
```

### Cache Configuration

Different file types have different cache configurations:

- **Images** (.jpg, .jpeg, .png, .gif, .webp, .svg): 30 days
- **Audio/Video** (.mp3, .ogg, .wav, .mp4, .webm): 7 days
- **Data files** (.json, .xml, .csv, .txt, .pdf): 1 day

### CDN Monitoring

Basic CDN metrics are available at:

```
/cdn/info
```

This endpoint provides information about:
- Total requests
- Successful/failed requests
- Bytes sent
- Requests by file type

### CDN Security

The CDN implements several security measures:
- Path validation to prevent directory traversal
- File type allowlist
- Size validation
- Proper MIME type detection

### CDN Environment Configuration

The CDN uses the `UNPACKED_DIR` environment variable to determine where to serve files from. 
By default, it's set to `./unpacked`.

To customize:

```
UNPACKED_DIR=/path/to/your/assets
```

### Supported File Types

- Images: .jpg, .jpeg, .png, .gif, .webp, .svg
- Audio: .mp3, .ogg, .wav
- Video: .mp4, .webm
- Documents: .pdf, .json, .txt, .csv, .xml

## License

TBD

## Contributing

TBD