#!/usr/bin/env bash
#
# Turn the CriWare USM cutscenes under `assets/ArkAssets/<server>/raw/video/` into web-playable
# `.webm` and `.mp4` under `assets/output/<server>/video/`, keeping the directory shape.
#
#   assets/transcode-video.sh [server] [--force] [path/to/one.usm ...]
#
# Defaults to the `en` server. Idempotent: a clip whose two outputs are both newer than its `.usm`
# is skipped, so a re-run after a partial extract costs only the stat calls.
#
# THE EN CLIPS ARE NOT ENCRYPTED. Measured 2026-09-24: all 16 `.usm` under en/raw/video demux
# key-free and decode with zero ffmpeg errors, and the game agrees. `CriWareInitializer` is
# AddComponent'd at runtime by `VideoManager.OnInitSDK` (RVA 0x42015fc) with `useDecrypter` left at
# its constructor default of false, so `CriWareDecrypter.Initialize` runs the empty-key overload and
# Sofdec2 decryption is never armed. ffmpeg's own `.usm` demuxer fails on these files ("Invalid
# frame marker", "zero_bit out of range"); that is a demuxer defect, not encryption. It emits USM
# chunk fragments as frames instead of reassembling the multi-chunk VP9 frames, so we demux with
# WannaCRI and hand ffmpeg the elementary streams.
#
# ARK_USM_KEY is honoured in case a future server or patch does arm the decrypter: set it to the
# 64-bit key as decimal or `0x`-prefixed hex and it is passed through to WannaCRI. Leave it unset
# for everything that ships today. Never commit a key into this repo. Should you ever need to
# re-derive one, the game computes it as `0x00D47EB533AEF7E5 XOR Convert.ToUInt64(decrypterConfig.key)`
# inside `CriWareDecrypter.Initialize` (RVA 0x4d95cec).
#
# OUTPUT NAMES ARE LOWERCASE. Story scripts reference these clips with the artist's capitalisation,
# `[video(res="video/act38side/PV01.mp4")]`, while the raw file is `pv01.usm`. We write the raw
# file's lowercase stem, so the resolver must match case-insensitively, the way the reader's audio
# resolver already does.
#
# Requirements: ffmpeg/ffprobe on PATH, and a Python with WannaCRI importable. Point `ARK_PY` at
# that interpreter, or install it into the default `python3` with `pip install wannacri`.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER="en"
FORCE=0
ONLY=""

for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    -*) echo "unknown flag: $arg" >&2; exit 2 ;;
    *.usm) ONLY="$ONLY$arg
" ;;
    *) SERVER="$arg" ;;
  esac
done

RAW_DIR="$REPO_ROOT/assets/ArkAssets/$SERVER/raw/video"
OUT_DIR="$REPO_ROOT/assets/output/$SERVER/video"

[ -d "$RAW_DIR" ] || { echo "no raw video tree for server '$SERVER' at $RAW_DIR" >&2; exit 1; }

PY="${ARK_PY:-python3}"
command -v "$PY" >/dev/null 2>&1 || { echo "python interpreter '$PY' not found; set ARK_PY" >&2; exit 1; }
"$PY" -c 'import wannacri' 2>/dev/null || {
  echo "WannaCRI is not importable from '$PY'. Install it (pip install wannacri) or set ARK_PY to an interpreter that has it." >&2
  exit 1
}
command -v ffmpeg >/dev/null 2>&1 || { echo "ffmpeg not on PATH" >&2; exit 1; }
command -v ffprobe >/dev/null 2>&1 || { echo "ffprobe not on PATH" >&2; exit 1; }

USM_KEY="${ARK_USM_KEY:-}"

# x264 quality for the .mp4 fallback. 20 is quality-first and costs bytes: act49side/ta02 lands at
# 41,083,136 B at crf 20, 31,021,023 B at 23 and 23,565,582 B at 26, against a 21,004,672 B source
# and an 18,089,803 B .webm. The .webm is the stream browsers actually take, so the fallback is
# where to spend less if delivery bytes start to matter.
CRF="${ARK_X264_CRF:-20}"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/ark-usm.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# WannaCRI insists on decoding the authoring filenames embedded in the CRID metadata, and Hypergryph
# shipped them in at least three encodings (utf-8, cp932, gbk). Walk the list until one opens, since
# the names themselves are never used: the outputs are named from the .usm path.
demux() {
  local usm="$1" dest="$2"
  ARK_ONE_USM="$usm" ARK_ONE_DEST="$dest" ARK_USM_KEY="$USM_KEY" "$PY" - <<'PYEOF'
import os, sys
from wannacri.usm import Usm

src, dest = os.environ["ARK_ONE_USM"], os.environ["ARK_ONE_DEST"]
raw_key = os.environ.get("ARK_USM_KEY", "").strip()
key = int(raw_key, 0) if raw_key else None

usm = None
for enc in ("utf-8", "cp932", "gbk", "latin-1"):
    try:
        usm = Usm.open(src, encoding=enc, key=key) if key is not None else Usm.open(src, encoding=enc)
        break
    except UnicodeDecodeError:
        continue
if usm is None:
    sys.exit(f"no encoding in utf-8/cp932/gbk/latin-1 opens {src}")

os.makedirs(dest, exist_ok=True)
for _ in usm.demux(path=dest, save_video=True, save_audio=True):
    pass
PYEOF
}

# Apparent size, not `du`: on APFS the allocated size drifts by a block and made two identical
# runs print different numbers for a byte-identical file.
human() {
  wc -c < "$1" 2>/dev/null | awk '{ b=$1
    if (b >= 1073741824) printf "%.1fG", b/1073741824
    else if (b >= 1048576) printf "%.1fM", b/1048576
    else if (b >= 1024) printf "%.1fK", b/1024
    else printf "%dB", b }'
}

printf '%-34s %9s %9s %9s %8s %7s %6s\n' FILE IN WEBM MP4 DURATION FRAMES WALL

total_start=$(date +%s)
processed=0
skipped=0

LIST="$WORK/usm.list"
if [ -n "$ONLY" ]; then
  printf '%s' "$ONLY" > "$LIST"
else
  find "$RAW_DIR" -type f -name '*.usm' | sort > "$LIST"
fi

while IFS= read -r usm; do
  [ -n "$usm" ] || continue
  rel="${usm#"$RAW_DIR"/}"
  rel_lower="$(printf '%s' "$rel" | tr '[:upper:]' '[:lower:]')"
  stem="${rel_lower%.usm}"
  webm="$OUT_DIR/$stem.webm"
  mp4="$OUT_DIR/$stem.mp4"

  if [ "$FORCE" -eq 0 ] && [ -f "$webm" ] && [ -f "$mp4" ] && [ "$webm" -nt "$usm" ] && [ "$mp4" -nt "$usm" ]; then
    printf '%-34s %9s %9s %9s %8s %7s %6s\n' "$stem" "$(human "$usm")" "$(human "$webm")" "$(human "$mp4")" - - skip
    skipped=$((skipped + 1))
    continue
  fi

  start=$(date +%s)
  scratch="$WORK/$(printf '%s' "$stem" | tr '/' '_')"
  rm -rf "$scratch"
  demux "$usm" "$scratch"

  ivf="$(find "$scratch" -type f -name '*.ivf' | head -1)"
  sfa="$(find "$scratch" -type f \( -name '*.sfa' -o -name '*.adx' -o -name '*.hca' \) | head -1)"
  [ -n "$ivf" ] || { echo "no video stream demuxed from $usm" >&2; exit 1; }

  mkdir -p "$(dirname "$webm")"

  if [ -n "$sfa" ]; then
    channels="$(ffprobe -v error -select_streams a:0 -show_entries stream=channels -of csv=p=0 "$sfa" | head -1)"
    channels="${channels:-2}"
    # VP9 is copied straight out of the USM: the .webm is a remux, not a re-encode.
    ffmpeg -nostdin -v error -y -i "$ivf" -i "$sfa" -map 0:v:0 -map 1:a:0 \
      -c:v copy -c:a libopus -b:a 128k -ac "$channels" "$webm"
    ffmpeg -nostdin -v error -y -i "$ivf" -i "$sfa" -map 0:v:0 -map 1:a:0 \
      -c:v libx264 -preset medium -crf "$CRF" -pix_fmt yuv420p \
      -c:a aac -b:a 160k -ac "$channels" -movflags +faststart "$mp4"
  else
    # The mixstory backdrops are silent loops and carry no @SFA stream.
    ffmpeg -nostdin -v error -y -i "$ivf" -map 0:v:0 -c:v copy "$webm"
    ffmpeg -nostdin -v error -y -i "$ivf" -map 0:v:0 \
      -c:v libx264 -preset medium -crf "$CRF" -pix_fmt yuv420p -movflags +faststart "$mp4"
  fi

  rm -rf "$scratch"
  wall=$(( $(date +%s) - start ))
  duration="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$mp4")"
  frames="$(ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=nb_read_frames -of csv=p=0 "$mp4")"
  printf '%-34s %9s %9s %9s %8s %7s %5ss\n' "$stem" "$(human "$usm")" "$(human "$webm")" "$(human "$mp4")" "$duration" "$frames" "$wall"
  processed=$((processed + 1))
done < "$LIST"

echo "$processed transcoded, $skipped skipped, $(( $(date +%s) - total_start ))s total -> $OUT_DIR"
