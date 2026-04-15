# myrtle.moe

A comprehensive Arknights game companion platform featuring operator databases, player profile syncing, tier lists, and advanced game tools—all powered by a high-performance Rust backend and modern React frontend.

[![GitHub license](https://img.shields.io/github/license/Eltik/myrtle.moe)](https://github.com/Eltik/myrtle.moe/blob/main/LICENSE)
[![Frontend](https://img.shields.io/badge/Frontend-Next.js_15-black)](frontend/)
[![Backend](https://img.shields.io/badge/Backend-Rust/Axum-orange)](backend/)
[![Demo](https://img.shields.io/badge/Demo-Live-brightgreen)](https://myrtle.moe)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
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

## License

TBD

## Acknowledgements

This project builds upon the work of many open-source projects:

- [UnityPy](https://github.com/K0lb3/UnityPy) - Original Python library for Unity asset extraction (unity-rs is a Rust port)
- [OpenArknightsFBS](https://github.com/MooncellWiki/OpenArknightsFBS) - FlatBuffers schemas
- [ArkPRTS](https://github.com/thesadru/ArkPRTS) - Inspiration for authentication flow
- [isHarryH/Ark-Unpacker](https://github.com/isHarryh/Ark-Unpacker) - Original Python implementation for unpacking (current `/unpacker` is a Rust port)

---

**Note**: This project is not affiliated with Hypergryph, Yostar, or any official Arknights entities. All game data and assets are property of their respective owners.
