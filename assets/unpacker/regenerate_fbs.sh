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

echo "Generated $(ls "$OUTPUT_DIR"/*_generated.rs 2>/dev/null | wc -l | tr -d ' ') FlatBuffer files"

# Generate FlatBufferToJson implementations
echo ""
echo "Generating FlatBufferToJson implementations..."
python3 "$SCRIPT_DIR/generate_fb_json_impls.py"

# ==============================================================================
# Yostar/EN Schema Generation (fallback for EN-only schemas)
# ==============================================================================
echo ""
echo "=== Yostar/EN Schema Generation ==="

YOSTAR_FBS_DIR="/tmp/ArknightsFlatbuffers/yostar"
YOSTAR_OUTPUT_DIR="$SCRIPT_DIR/src/generated_fbs_yostar"

# Clone or update ArknightsFlatbuffers repo
if [ ! -d "/tmp/ArknightsFlatbuffers" ]; then
    echo "Cloning ArknightsFlatbuffers repository..."
    git clone --depth 1 https://github.com/ArknightsAssets/ArknightsFlatbuffers.git /tmp/ArknightsFlatbuffers
else
    echo "Updating ArknightsFlatbuffers repository..."
    (cd /tmp/ArknightsFlatbuffers && git pull --rebase) || true
fi

# Create Yostar output directory
mkdir -p "$YOSTAR_OUTPUT_DIR"

# Yostar-specific schemas (EN/Global versions that differ from CN)
YOSTAR_SCHEMAS=(
    "character_table"
    "battle_equip_table"
    "token_table"
    "ep_breakbuff_table"
)

echo "Generating Rust files from Yostar FBS schemas..."
for name in "${YOSTAR_SCHEMAS[@]}"; do
    fbs="$YOSTAR_FBS_DIR/${name}.fbs"
    if [ -f "$fbs" ]; then
        echo "  Processing (Yostar): $name"
        flatc --rust --gen-object-api --rust-serialize -o "$YOSTAR_OUTPUT_DIR" "$fbs" 2>&1 | grep -v "warning:" || true
    else
        echo "  Warning: $fbs not found"
    fi
done

# Generate Yostar mod.rs
echo "Generating Yostar mod.rs..."
cat > "$YOSTAR_OUTPUT_DIR/mod.rs" << 'EOF'
// Auto-generated FlatBuffer schemas for Arknights (Yostar/EN version)
// These schemas are used as fallback when CN schemas fail to decode EN assets
// Generated with: flatc --rust --gen-object-api --rust-serialize

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unreachable_patterns)]
#![allow(clippy::all)]

EOF

for name in "${YOSTAR_SCHEMAS[@]}"; do
    if [ -f "$YOSTAR_OUTPUT_DIR/${name}_generated.rs" ]; then
        echo "pub mod ${name}_generated;" >> "$YOSTAR_OUTPUT_DIR/mod.rs"
    fi
done

echo "Generated $(ls "$YOSTAR_OUTPUT_DIR"/*_generated.rs 2>/dev/null | wc -l | tr -d ' ') Yostar FlatBuffer files"

# Generate Yostar FlatBufferToJson implementations
echo ""
echo "Generating Yostar FlatBufferToJson implementations..."
python3 "$SCRIPT_DIR/generate_fb_json_impls_yostar.py"

echo ""
echo "Done! Run 'cargo build --release' to compile."
