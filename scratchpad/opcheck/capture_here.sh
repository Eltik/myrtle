#!/usr/bin/env bash
# capture_here.sh — record a dyn-illust ENTRANCE from an ALREADY-OPEN preview page.
#
# capture_l2d.sh always re-navigates from the main menu, which cannot target a specific skin.
# This is its record+process half, split out so the caller can drive navigation however it
# likes (OCR search, brand grid, …) and then just say "capture what is on screen now".
#
#   ./capture_here.sh <name> [dur]
#
# Encodes the same three rules: native resolution, REVEAL chrome before pressing PLAY (a tap on
# auto-hidden chrome silently does nothing), and assert the blackout afterwards.
set -uo pipefail
export PATH="$HOME/Library/Android/sdk/platform-tools:$PATH"
HERE="$(cd "$(dirname "$0")" && pwd)"
NAME="${1:?output basename}"; DUR="${2:-30}"
REFOUT="$HERE/REF_NEW"; RAW="/private/tmp/ak_${NAME}_raw.mp4"
log(){ echo "[here] $*"; }
wh=$(adb shell wm size | tr -d '\r' | grep -oE '[0-9]+x[0-9]+' | tail -1)
W=${wh%x*}; H=${wh#*x}
SX=$(python3 -c "print($W/1170)"); SY=$(python3 -c "print($H/540)")
log "display ${W}x${H}"
t(){ adb shell input tap "$(python3 -c "print(int($1*$SX))")" "$(python3 -c "print(int($2*$SY))")"; }

adb shell rm -f /sdcard/l2d.mp4
adb shell "screenrecord --bit-rate 100000000 --time-limit $((DUR+6)) /sdcard/l2d.mp4" &
sleep 1.5
log "magnifier → viewer";     t 29 498;  sleep 4.0
log "reveal chrome (centre)"; t 585 270; sleep 1.2
log "PLAY (entrance)";        t 989 505
i=0; while [ "$i" -lt "$DUR" ]; do sleep 3; i=$((i+3)); done
adb shell pkill -INT screenrecord 2>/dev/null; sleep 3
adb pull /sdcard/l2d.mp4 "$RAW" 2>&1 | tail -1
mkdir -p "$REFOUT"
python3 - "$RAW" "$REFOUT/${NAME}_game_fresh.mp4" "$REFOUT/${NAME}_entrance.mp4" <<'PY'
import subprocess, sys, numpy as np
raw, fresh, ent = sys.argv[1:4]
pr=subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries","stream=width,height",
    "-of","csv=p=0",raw],capture_output=True,text=True).stdout.strip().split(",")
W,H=int(pr[0]),int(pr[1])
dur=float(subprocess.run(["ffprobe","-v","error","-show_entries","format=duration","-of","csv=p=0",raw],
    capture_output=True,text=True).stdout.strip() or 0)
g=subprocess.run(["ffmpeg","-v","error","-i",raw,"-f","rawvideo","-pix_fmt","gray","-"],capture_output=True).stdout
n=len(g)//(W*H); G=np.frombuffer(g,np.uint8)[:n*W*H].reshape(n,H,W)
fps=n/dur if dur>0 else 60.0
lum=G.reshape(n,-1).mean(axis=1)
print(f"[proc] {W}x{H} {n} frames @ {fps:.2f}fps  luma {lum.min():.1f}..{lum.max():.1f}")
dark=np.where(lum<12)[0]
if len(dark)==0:
    print("[proc] ❌ NO BLACKOUT — the PLAY tap missed (chrome was hidden). Retake."); sys.exit(3)
# 🚨 ANCHOR ON THE FIRST CONTIGUOUS DARK RUN, never on `dark[-1]`.
# `dark[-1]` is the last sub-12 frame ANYWHERE in the recording, so an entrance with dark
# BEATS in it anchors t0 at the wrong end. Measured on skadi2, whose runs are 7.40-7.66,
# 11.01-11.49, 18.36-18.54 and 22.06-22.28: the old form put t0 at 22.45 s, past the end of
# her entrance, and truncated the clip to nothing usable. Identical result on skins with a
# single dark run, so this is a strict improvement.
runs=[]; s0=dark[0]; prev=dark[0]
for i in dark[1:]:
    if i!=prev+1: runs.append((int(s0),int(prev))); s0=i
    prev=i
runs.append((int(s0),int(prev)))
print("[proc] dark runs (s): "+", ".join(f"{a/fps:.2f}-{b/fps:.2f}" for a,b in runs))
b0,b1=runs[0]
post=np.where(lum[b1:]>12)[0]
t0=int(b1+post[0]) if len(post) else b1
print(f"[proc] blackout frames {b0}-{b1} ({b0/fps:.2f}-{b1/fps:.2f}s) → entrance t0 = frame {t0} ({t0/fps:.2f}s) ✅")
t0s=t0/fps
subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",raw,"-c","copy",ent])
subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",raw,
    "-vf","scale=900:416:flags=lanczos,fps=30","-c:v","libx264","-crf","12","-pix_fmt","yuv420p",fresh])
print(f"[proc] ✅ wrote {fresh.split('/')[-1]} + {ent.split('/')[-1]}")
PY
