#!/bin/bash
export DYLD_LIBRARY_PATH=/Users/eltik/Documents/Coding/myrtle.moe/assets/unity-rs/src/resources/FMOD/Darwin:$DYLD_LIBRARY_PATH

# Asset paths - modify these to match your setup
ASSETS_DIR="${ASSETS_DIR:-/Users/eltik/Documents/Coding/myrtle.moe/assets/downloader/ArkAssets}"
OUTPUT_DIR="${OUTPUT_DIR:-/Users/eltik/Documents/Coding/myrtle.moe/assets/Unpacked}"

# Performance settings
# THREADS: Number of parallel threads (default: 4, use 1 for lowest memory)
# SKIP_LARGE_MB: Skip files larger than this size in MB (default: 0 = no skip)
#                Set to 100 to skip huge files like token.ab for faster initial extraction
THREADS="${THREADS:-4}"
SKIP_LARGE_MB="${SKIP_LARGE_MB:-0}"

if [ ! -d "$ASSETS_DIR" ]; then
    echo "Error: Assets directory not found: $ASSETS_DIR"
    echo "Please run the downloader first or set ASSETS_DIR environment variable"
    echo "Example: ASSETS_DIR=/path/to/ArkAssets ./run_unpacker.sh"
    exit 1
fi

# Find the resource manifest (.idx file)
MANIFEST=$(find "$ASSETS_DIR" -maxdepth 1 -name "*.idx" | head -1)
if [ -z "$MANIFEST" ]; then
    echo "Warning: No .idx manifest found in $ASSETS_DIR"
fi

echo "Configuration:"
echo "  Assets: $ASSETS_DIR"
echo "  Output: $OUTPUT_DIR"
echo "  Threads: $THREADS"
if [ "$SKIP_LARGE_MB" -gt 0 ]; then
    echo "  Skip files >: ${SKIP_LARGE_MB}MB"
fi
echo ""

# Extract assets from AB files
# - Uses hybrid processing: small files in parallel, large files sequentially
# - Files are sorted by size (smallest first) for better progress visibility
./target/release/assets-unpacker extract \
    --input "$ASSETS_DIR" \
    --output "$OUTPUT_DIR" \
    -j "$THREADS" \
    --skip-large-mb "$SKIP_LARGE_MB"

# Decode FlatBuffer files (character_table, enemy_database, etc.) to JSON
# Files are organized using the manifest path structure (e.g., gamedata/excel/character_table.json)
if [ -n "$MANIFEST" ]; then
    ./target/release/assets-unpacker decode --input "$OUTPUT_DIR/upk" --output "$OUTPUT_DIR/decoded" --manifest "$MANIFEST"
else
    ./target/release/assets-unpacker decode --input "$OUTPUT_DIR/upk" --output "$OUTPUT_DIR/decoded"
fi
