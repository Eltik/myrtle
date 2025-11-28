#!/bin/bash
export DYLD_LIBRARY_PATH=/Users/eltik/Documents/Coding/myrtle.moe/assets/unity-rs/src/resources/FMOD/Darwin:$DYLD_LIBRARY_PATH

# Extract assets from AB files
./target/release/assets-unpacker extract --input ../ArkAssets --output ../Unpacked

# Decode FlatBuffer files (character_table, enemy_database, etc.) to JSON
./target/release/assets-unpacker decode --input ../Unpacked/upk --output ../Unpacked/decoded
