#!/usr/bin/env bash
# =============================================================================
# repull_dynchars.sh — pull NEW dynchar AssetBundles from the running emulator,
# export + deploy them. SAFE: existing bundles are NOT touched, so the 8
# reference-skin baselines are preserved. Bundles that CHANGED size in the
# current client are REPORTED (they'd shift baselines) but only pulled with
# --force-changed.
#
# Why the device: the game's downloaded bundles live at
#   /storage/emulated/0/Android/data/<pkg>/files/Bundles/  (raw UnityFS, same
#   names + format as assets/ArkAssets/en). ⚠️ PREVIEW-ONLY skins (Fashion
#   Gallery) are NOT persisted there — they cannot be re-pulled/scored.
#
#   ./repull_dynchars.sh [--force-changed]
# =============================================================================
set -uo pipefail
export PATH="$HOME/Library/Android/sdk/platform-tools:$PATH"
PKG=com.YoStarEN.Arknights
DEV="/storage/emulated/0/Android/data/$PKG/files/Bundles"
ROOT=/Users/eltik/Documents/Coding/myrtle
AK="$ROOT/assets/ArkAssets/en"
OUT="$ROOT/assets/output/en/spine/DynIllust"
UNPACK="$ROOT/assets/unpacker"
IN="/private/tmp/ak_repull_in"; EXP="/private/tmp/ak_repull_out"
FORCE=0; [ "${1:-}" = "--force-changed" ] && FORCE=1
log(){ echo "[repull] $*"; }

adb devices | grep -q emulator-5554 || { log "ERROR: no emulator (boot it first)"; exit 1; }
adb root >/dev/null 2>&1; sleep 2

log "diffing device dynchars vs ArkAssets…"
adb shell "ls -la $DEV/arts/dynchars/ 2>/dev/null" | tr -d '\r' | awk 'NF>=8&&/\.ab$/{print $5" "$NF}' | sort -k2 > /tmp/rp_dev.txt
ls -la "$AK/arts/dynchars/" | awk 'NF>=8&&/\.ab$/{print $5" "$NF}' | sort -k2 > /tmp/rp_ak.txt
NEW=$(comm -23 <(cut -d' ' -f2- /tmp/rp_dev.txt|sort) <(cut -d' ' -f2- /tmp/rp_ak.txt|sort))
CHANGED=$(join -j2 <(sort -k2 /tmp/rp_ak.txt) <(sort -k2 /tmp/rp_dev.txt) | awk '$2!=$3{print $1}')
log "NEW bundles:"; echo "$NEW" | sed 's/^/    /'
[ -n "$CHANGED" ] && { log "CHANGED bundles (baseline risk — skipped unless --force-changed):"; echo "$CHANGED" | sed 's/^/    /'; }

PULL="$NEW"; [ "$FORCE" = 1 ] && PULL="$NEW
$CHANGED"
PULL=$(echo "$PULL" | grep -v '^$' | sort -u)
[ -z "$PULL" ] && { log "nothing to pull — up to date."; exit 0; }

rm -rf "$IN" "$EXP"; mkdir -p "$IN/arts/dynchars" "$IN/refs"
while read -r b; do [ -z "$b" ] && continue; adb pull "$DEV/arts/dynchars/$b" "$IN/arts/dynchars/$b" >/dev/null 2>&1 && log "pulled $b"; done <<< "$PULL"
adb pull "$DEV/[uc]shaders.ab" "$IN/[uc]shaders.ab" >/dev/null 2>&1
adb pull "$DEV/refs/fx" "$IN/refs/fx" >/dev/null 2>&1

log "exporting…"
( cd "$UNPACK" && cargo run --release -- extract -i "$IN" -o "$EXP" --spine ) 2>&1 | tail -3

log "deploying (new bundles → ArkAssets, new skin dirs → output)…"
for f in "$IN"/arts/dynchars/*.ab; do cp "$f" "$AK/arts/dynchars/"; done
for d in "$EXP"/spine/DynIllust/*/; do s=$(basename "$d"); [ -d "$OUT/$s" ] && { [ "$FORCE" = 1 ] && cp -R "$d" "$OUT/$s"; } || cp -R "$d" "$OUT/$s"; done

log "new DYNAMIC skins (have _Start = scoreable entrance):"
for d in "$EXP"/spine/DynIllust/*/; do s=$(basename "$d"); find "$d" -name '*_Start*.skel' -print -quit 2>/dev/null | grep -q . && echo "    $s  ← DYNAMIC"; done
log "done. Capture a new dynamic skin in-game with capture_l2d.sh, then it is scoreable."
