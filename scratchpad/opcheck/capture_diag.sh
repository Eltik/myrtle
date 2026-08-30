#!/usr/bin/env bash
# =============================================================================
# capture_diag.sh — DIAGNOSTIC capture: a look at the game, NEVER a scoring reference.
#
# The 59 fps gate in capture_l2d.sh exists so a clip can be scored against beats; naming what
# is visibly wrong needs a correct FRAME, not a smooth clip. This path records WITHOUT the
# gate, at 20 Mbps (the bit-rate the 60.01 fps calibration take used), and writes ONLY to
# scratchpad/opcheck/DIAG/<name>_DIAGONLY.mp4 so it can never wander into a corpus: every
# scorer resolves references as REF_NEW/<key>_<suffix>.mp4 and the _DIAGONLY suffix plus the
# separate directory keep these out by construction.
#
#   ./capture_diag.sh <name> <duration-s> [bitrate]
#
# Assumes the emulator is BOOTED and the screen already shows what should be recorded (the
# caller navigates first). No taps, no navigation, no processing: record, pull, report fps.
# =============================================================================
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
NAME=${1:?name}; DUR=${2:?duration-s}; BR=${3:-20000000}
OUT="$HERE/DIAG/${NAME}_DIAGONLY.mp4"
adb shell rm -f /sdcard/diag.mp4
adb shell "screenrecord --bit-rate $BR --time-limit $DUR /sdcard/diag.mp4"
sleep 1
adb pull /sdcard/diag.mp4 "$OUT" >/dev/null
adb shell rm -f /sdcard/diag.mp4
python3 - "$OUT" <<'PY'
import subprocess, sys
o=sys.argv[1]
d=float(subprocess.run(["ffprobe","-v","error","-show_entries","format=duration","-of","csv=p=0",o],capture_output=True,text=True).stdout.strip() or 0)
n=int(subprocess.run(["ffprobe","-v","error","-count_frames","-select_streams","v:0","-show_entries","stream=nb_read_frames","-of","csv=p=0",o],capture_output=True,text=True).stdout.strip() or 0)
print(f"[diag] {o}: {n} frames / {d:.2f}s = {n/d if d else 0:.2f} fps. DIAGNOSTIC ONLY, not for scoring.")
PY
