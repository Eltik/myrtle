#!/bin/bash
export DYLD_LIBRARY_PATH=/Users/eltik/Documents/Coding/myrtle.moe/assets/unity-rs/src/resources/FMOD/Darwin:$DYLD_LIBRARY_PATH

# Find the resource manifest (.idx file)
MANIFEST=$(find ../ArkAssets -maxdepth 1 -name "*.idx" | head -1)
if [ -z "$MANIFEST" ]; then
    echo "Warning: No .idx manifest found in ../ArkAssets"
fi

# Extract assets from AB files
./target/release/assets-unpacker extract --input ../ArkAssets --output ../Unpacked

# Decode FlatBuffer files (character_table, enemy_database, etc.) to JSON
# Files are organized using the manifest path structure (e.g., gamedata/excel/character_table.json)
if [ -n "$MANIFEST" ]; then
    ./target/release/assets-unpacker decode --input ../Unpacked/upk --output ../Unpacked/decoded --manifest "$MANIFEST"
else
    ./target/release/assets-unpacker decode --input ../Unpacked/upk --output ../Unpacked/decoded
fi
