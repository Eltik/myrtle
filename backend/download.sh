#!/bin/bash

# Arknights Asset Downloader Script
# This script will run the TypeScript downloader

# Get the script directory
SCRIPT_DIR=$(dirname "$(realpath "$0")")

# Default download path
DOWNLOAD_PATH="${1:-$SCRIPT_DIR/ArkAssets}"

echo "=========================================="
echo "  Arknights Asset Downloader (Launcher)  "
echo "=========================================="
echo "Download path: $DOWNLOAD_PATH"
echo ""

# Run the TypeScript file directly with Bun
bun run "$SCRIPT_DIR/src/lib/impl/local/impl/assets/impl/ark-downloader.ts" "$DOWNLOAD_PATH" 