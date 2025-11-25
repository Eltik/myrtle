#!/bin/bash
export DYLD_LIBRARY_PATH=/Users/eltik/Documents/Coding/myrtle.moe/assets/unity-rs/src/resources/FMOD/Darwin:$DYLD_LIBRARY_PATH
exec ./target/release/assets-unpacker extract --input ../ArkAssets --output ../Unpacked
