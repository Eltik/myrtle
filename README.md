# myrtle.moe

A comprehensive Arknights game information and calculation platform with an elegant, modern interface.

## Notice:
There will be a pause of development as I am working on recoding the entire project from TypeScript to Rust. However, due to the fact that I don't know Rust very well, it will take a bit of time. I will update the entire README.md file once everything is finished.

[![GitHub license](https://img.shields.io/github/license/Eltik/myrtle.moe)](https://github.com/Eltik/myrtle.moe/blob/main/LICENSE)
[![Frontend](https://img.shields.io/badge/Frontend-Next.js-blue)](frontend/)
[![Backend](https://img.shields.io/badge/Backend-Bun-black)](backend/)
[![Demo](https://img.shields.io/badge/Demo-Live-brightgreen)](https://myrtle.moe)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Running the Application](#running-the-application)
- [Technology Stack](#technology-stack)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

## Overview

myrtle.moe is a feature-rich Arknights toolkit designed to provide players with accurate game data, advanced calculations, and intuitive visualizations. The platform offers a modern, responsive interface that makes it easy to research operators, calculate DPS, plan upgrades, and access game assets.

Named after the beloved Arknights operator Myrtle, this project aims to be the most comprehensive, accurate, and user-friendly Arknights resource available.

## Features

- **Operator Database**: Complete information on all Arknights operators including stats, skills, talents, and modules
- **DPS Calculator**: Advanced damage calculation system with support for all skill mechanics, talents, and buffs
- **Asset Browser**: Direct access to game artwork, animations, audio, and other assets
- **Team Builder**: Create and share team compositions with detailed synergy analysis
- **Interactive Tools**: Intuitive tools for planning operator upgrades, material farming, and more
- **Game Data API**: Robust API for accessing Arknights game data programmatically
- **Modern UI**: Elegant, responsive interface built with Next.js and Tailwind CSS

## Project Structure

The myrtle.moe project is divided into three main components:

- **[Frontend](frontend/)**: The user interface built with Next.js, T3 Stack, and Tailwind CSS
- **[Backend](backend/)**: The API server and data processing engine built with Bun and TypeScript
- **[Assets](assets/)**: Tools for downloading, unpacking, and processing Arknights game assets

Each component has its own detailed README.md with specific setup instructions.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+ or [Bun](https://bun.sh/)
- [PostgreSQL](https://www.postgresql.org/) database
- [Redis](https://redis.io/) for caching
- [Python 3.8+](https://www.python.org/downloads/) for asset processing
- Sufficient disk space (~10GB) for game assets

### Installation

1. Clone the repository:
```bash
git clone https://github.com/Eltik/myrtle.moe.git
cd myrtle.moe
```

2. Set up and install each component:

```bash
###
# Assets Toolkit
# Necessary for the backend and frontend to work.
###
# Set up assets toolkit
cd assets

# Download assets to ./assets/Unpacked directory
python assets/downloader/ark-downloader.py --output assets/ArkAssets

# Install unpacker dependencies
cd assets/unpacker/unpacker
poetry install

# Build unpacker
python Build.py # For Windows

chmod +x build_macos.sh # For MacOS
./build_macos.sh

# Run unpacker. If the unpacker starts stalling, stop the unpacker and add the --resume argument.
./build/dist/ArkUnpacker-v4.0.0 -m ab -i ../../ArkAssets --image --text --audio --spine -o ../../Unpacked

# Combine alpha assets
cd ../../../ # Move back to main directory
python assets/unpacker/helper/combine_alpha.py --input-dir ./assets/Unpacked --delete-alpha

# Rename assets to proper naming convention
python assets/unpacker/helper/rename_assets.py ./assets/Unpacked

# Find missing assets, as when unpacking sometimes (especially if the unpacker stalled)
# there will be assets that weren't unpacked properly.
python assets/unpacker/missing/find-assets.py --find-missing --source-dir ./assets/ArkAssets --extracted-dir ./assets/Unpacked -o ./assets/unpacker/missing

# Extract those missing assets
python assets/unpacker/unpacker/Main.py -m ab -i ./assets/ArkAssets -o ./assets/Unpacked --target-list ./assets/unpacker/missing/missing_assets.json --image --spine --text --audio --resume --skip-problematic --timeout 20

###
# Backend
###

# Set up backend
cd ../backend
cp .env.example .env
# Edit .env with your configuration
bun install

# Set up frontend
cd ../frontend
cp .env.example .env
# Edit .env with your configuration
bun install
```
```

### Running the Application

1. Start the backend server:
```bash
cd backend
bun run dev
```

2. Start the frontend development server:
```bash
cd frontend
bun run dev
```

3. Open your browser and visit [http://localhost:3000](http://localhost:3000)

For detailed instructions on running each component, please refer to their individual README files.

## Technology Stack

- **Frontend**:
  - [Next.js](https://nextjs.org/) - React framework
  - [T3 Stack](https://create.t3.gg/) - Modern web development stack
  - [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS
  - [shadcn/ui](https://ui.shadcn.com/) - UI component library
  - [Zustand](https://zustand-demo.pmnd.rs/) - State management
  - [Framer Motion](https://www.framer.com/motion/) - Animations

- **Backend**:
  - [Bun](https://bun.sh/) - JavaScript runtime
  - [TypeScript](https://www.typescriptlang.org/) - Type-safe JavaScript
  - [PostgreSQL](https://www.postgresql.org/) - Database
  - [Redis](https://redis.io/) - Caching
  - [Zod](https://github.com/colinhacks/zod) - Schema validation

- **Assets Toolkit**:
  - [Python](https://www.python.org/) - Primary language
  - [UnityPy](https://pypi.org/project/UnityPy/) - Unity asset extraction
  - [Pillow](https://pypi.org/project/Pillow/) - Image processing

## Architecture

myrtle.moe follows a modern web application architecture:

```
┌────────────┐      ┌────────────┐      ┌────────────┐
│  Frontend  │◄────►│  Backend   │◄────►│ PostgreSQL │
│  (Next.js) │      │ (Bun/TS)   │      │ Database   │
└────────────┘      └────────────┘      └────────────┘
                          │
                          ▼
                    ┌────────────┐      ┌────────────┐
                    │   Redis    │      │ Processed  │
                    │   Cache    │      │  Assets    │
                    └────────────┘      └────────────┘
                                              ▲
                                              │
                                        ┌────────────┐
                                        │   Assets   │
                                        │  Toolkit   │
                                        └────────────┘
                                              ▲
                                              │
                                        ┌────────────┐
                                        │ Arknights  │
                                        │ Game Data  │
                                        └────────────┘
```

- The **Frontend** provides the user interface and communicates with the backend API
- The **Backend** processes requests, performs calculations, and serves data
- The **Assets Toolkit** downloads and processes game assets for use by the backend
- **PostgreSQL** stores structured game data, user data, and calculation results
- **Redis** provides caching for frequently accessed data to improve performance

## Contributing

Contributions are welcome! Please see the individual component READMEs for specific contribution guidelines.

## License

TBD

## Acknowledgements

This project was inspired by existing Arknights tools and would not be possible without:

- [ArkPRTS](https://github.com/thesadru/ArkPRTS) - Inspiration for many features
- [Aceship/Arknights-Toolkit](https://github.com/Aceship/Arknights-Toolkit) - Amazing resource for game data
- [isHarryh/Ark-Unpacker](https://github.com/isHarryh/Ark-Unpacker) - Essential tool for asset unpacking
- [ChaomengOrion/ArkAssetsTool](https://github.com/ChaomengOrion/ArkAssetsTool) - Foundation for our asset downloader
- The Arknights community for their continued support and feedback
