#!/bin/bash
# Regenerate all FlatBuffer Rust files with serde::Serialize support
# Requires flatc 25.9.23 or later

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FBS_DIR="/Users/eltik/Documents/Coding/myrtle.moe/assets/OpenArknightsFBS/FBS"
OUTPUT_DIR="$SCRIPT_DIR/src/generated_fbs"

# Check flatc version
FLATC_VERSION=$(flatc --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
echo "flatc version: $FLATC_VERSION"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate Rust files for all FBS schemas
echo "Generating Rust files from FBS schemas..."
for fbs in "$FBS_DIR"/*.fbs; do
    name=$(basename "$fbs" .fbs)
    echo "  Processing: $name"
    flatc --rust --gen-object-api --rust-serialize -o "$OUTPUT_DIR" "$fbs" 2>&1 | grep -v "warning:" || true
done

# Generate mod.rs
echo "Generating mod.rs..."
cat > "$OUTPUT_DIR/mod.rs" << 'EOF'
// Auto-generated FlatBuffer schemas for Arknights
// Generated with: flatc --rust --gen-object-api --rust-serialize
// These types implement serde::Serialize for direct JSON conversion

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unreachable_patterns)]
#![allow(clippy::all)]

EOF

for fbs in "$FBS_DIR"/*.fbs; do
    name=$(basename "$fbs" .fbs)
    echo "pub mod ${name}_generated;" >> "$OUTPUT_DIR/mod.rs"
done

echo "Done! Generated $(ls "$OUTPUT_DIR"/*_generated.rs 2>/dev/null | wc -l | tr -d ' ') files"
