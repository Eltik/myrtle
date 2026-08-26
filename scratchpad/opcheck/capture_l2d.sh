#!/usr/bin/env bash
# =============================================================================
# capture_l2d.sh — capture an Arknights dyn-illust ENTRANCE from the emulator.
#
# ⚠️ REWRITTEN 2026-08-22 to the DOCUMENTED recipe (see memory
# dynchar-lookbook-route / dynchar-fresh-oracle-capture / dynchar-emulator-l2d-capture-flow §6).
# The previous version got idle-framed, unscoreable clips because it (a) captured at
# 1170x540 and (b) recorded the magnifier-open instead of the ▶ Play press.
#
#   ./capture_l2d.sh <name> auto [dur]                # auto-find a scoreable entrance skin
#   ./capture_l2d.sh <name> <scrolls> <tapx> <tapy>   # target a lookbook cell (1170x540 coords)
#
# THE THREE RULES THIS ENCODES
#  1. NATIVE 2340x1080. The game fits the art to the screen, so any other size shows a
#     DIFFERENT AMOUNT OF SCENE and the capture is not comparable to our render.
#  2. Entrance = REVEAL chrome (tap centre) then tap ▶, BOTH inside the recording ~1s apart.
#     A ▶ tap on auto-hidden chrome silently does nothing and yields a static viewer clip.
#  3. VERIFY: a real entrance drives mean luma to ~0 within ~2s of the ▶ tap. Else retake.
#
# Output: REF_NEW/<name>_game_fresh.mp4 (900x416/30, t0-anchored) + <name>_entrance.mp4 (raw).
# =============================================================================
set -uo pipefail
export PATH="$HOME/Library/Android/sdk/platform-tools:$HOME/Library/Android/sdk/emulator:$PATH"
HERE="$(cd "$(dirname "$0")" && pwd)"
SDK="$HOME/Library/Android/sdk"; AVD="ak_cap"; PKG="com.YoStarEN.Arknights"
REFOUT="$HERE/REF_NEW"
NAME="${1:?output basename}"; MODE="${2:?'auto' or scrolls}"
if [ "$MODE" = "auto" ]; then DUR="${3:-30}"; else SCROLLS="$MODE"; TX="${3:?}"; TY="${4:?}"; DUR="${5:-30}"; fi
RAW="/private/tmp/ak_${NAME}_raw.mp4"
log(){ echo "[l2d] $*"; }
pss(){ adb shell dumpsys meminfo "$PKG" 2>/dev/null | grep 'TOTAL PSS' | grep -oE '[0-9]+' | head -1; }
alive(){ [ -n "$(adb shell pidof "$PKG" 2>/dev/null | tr -d '\r')" ]; }

# ---- coords are authored in a 1170x540 base and SCALED to the live display ----
SX=1; SY=1
calc_scale(){
  local wh; wh=$(adb shell wm size 2>/dev/null | tr -d '\r' | tail -1 | grep -oE '[0-9]+x[0-9]+' | tail -1)
  local w="${wh%x*}" h="${wh#*x}"
  SX=$(python3 -c "print($w/1170)"); SY=$(python3 -c "print($h/540)")
  log "display ${w}x${h} → coord scale ${SX}x/${SY}y"
}
t(){ adb shell input tap "$(python3 -c "print(int($1*$SX))")" "$(python3 -c "print(int($2*$SY))")"; }
sw(){ adb shell input swipe "$(python3 -c "print(int($1*$SX))")" "$(python3 -c "print(int($2*$SY))")" \
        "$(python3 -c "print(int($3*$SX))")" "$(python3 -c "print(int($4*$SY))")" "${5:-350}"; }

ensure_emulator(){
  if ! adb devices | grep -q emulator-5554; then
    log "booting $AVD…"
    rm -f "$HOME/.android/avd/$AVD.avd/hardware-qemu.ini.lock" 2>/dev/null
    nohup "$SDK/emulator/emulator" -avd "$AVD" -no-audio -gpu host -memory 8192 -cores 6 \
      -feature -GuestAngle > /tmp/ak_emu.log 2>&1 &
    adb wait-for-device
    for i in $(seq 1 30); do [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ] && break; sleep 5; done
  fi
  adb root >/dev/null 2>&1; sleep 4; adb wait-for-device
  adb shell wm size reset >/dev/null 2>&1        # RULE 1: NATIVE resolution, never an override
  adb shell wm density reset >/dev/null 2>&1
  sleep 2; calc_scale
  local TOKEN; TOKEN=$(cat ~/.emulator_console_auth_token 2>/dev/null)
  { echo "auth $TOKEN"; sleep 0.3; echo "network speed full"; sleep 0.2; echo "quit"; } | nc -w 3 localhost 5554 >/dev/null 2>&1
}

ensure_menu(){
  local tries=0
  while :; do
    alive || { log "launching game"; adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1; sleep 8; }
    local stall=0 last=0
    for i in $(seq 1 40); do
      sleep 6; local p; p=$(pss); p=${p:-0}
      [ "$p" -gt 1330000 ] && { log "main menu (PSS $p)"; return 0; }
      [ "$p" -gt 700000 ] && [ "$p" -lt 1300000 ] && t 585 384     # title START
      [ "$p" -gt 400000 ] && [ "$p" -lt 900000 ]  && t 585 270     # loading tap-to-continue
      local d=$(( p>last ? p-last : last-p )); last=$p
      [ "$d" -lt 6000 ] && stall=$((stall+1)) || stall=0
      [ "$stall" -ge 10 ] && { log "stalled, restarting game"; adb shell am force-stop "$PKG"; sleep 3; break; }
    done
    tries=$((tries+1)); [ "$tries" -ge 4 ] && { log "ERROR: no menu"; return 1; }
  done
}

navigate_to_lookbook(){
  log "menu → Store";        t 778 375; sleep 4
  log "Outfit Store tab";    t 585 80;  sleep 4
  log "swipe carousel → Fashion Gallery"
  local prev="" cur=""
  for k in $(seq 1 30); do
    sw 1000 280 150 280; sleep 0.5      # ⚠️ y must stay well above the home-gesture zone
    cur=$(adb exec-out screencap -p 2>/dev/null | python3 -c "import sys,hashlib;print(hashlib.md5(sys.stdin.buffer.read()[:200000]).hexdigest())" 2>/dev/null)
    [ "$cur" = "$prev" ] && { log "carousel end (${k} swipes)"; break; }; prev="$cur"
  done
  log "tap Fashion Gallery"; t 1076 340; sleep 4
}

# RULE 2 + 3: record, magnifier, reveal chrome, ▶, then assert the blackout
record_entrance(){
  adb shell rm -f /sdcard/l2d.mp4
  adb shell "screenrecord --bit-rate 100000000 --time-limit $((DUR+6)) /sdcard/l2d.mp4" &
  sleep 1.5
  log "magnifier → viewer";        t 29 498;  sleep 3.5
  log "reveal chrome (centre)";    t 585 270; sleep 1.2
  log "▶ PLAY (entrance)";         t 989 505
  local i=0; while [ "$i" -lt "$DUR" ]; do sleep 3; i=$((i+3)); alive || { log "WARN game exited"; break; }; done
  adb shell pkill -INT screenrecord 2>/dev/null; sleep 3
  adb pull /sdcard/l2d.mp4 "$RAW" 2>&1 | tail -1
}

process(){
  mkdir -p "$REFOUT"
  python3 - "$RAW" "$REFOUT/${NAME}_game_fresh.mp4" "$REFOUT/${NAME}_entrance.mp4" <<'PY'
import subprocess, sys, numpy as np
raw, fresh, ent = sys.argv[1:4]
pr=subprocess.run(["ffprobe","-v","error","-select_streams","v:0","-show_entries",
    "stream=width,height","-of","csv=p=0",raw],capture_output=True,text=True).stdout.strip().split(",")
W,H=int(pr[0]),int(pr[1])
dur=float(subprocess.run(["ffprobe","-v","error","-show_entries","format=duration","-of","csv=p=0",raw],
    capture_output=True,text=True).stdout.strip() or 0)
g=subprocess.run(["ffmpeg","-v","error","-i",raw,"-f","rawvideo","-pix_fmt","gray","-"],capture_output=True).stdout
n=len(g)//(W*H); G=np.frombuffer(g,np.uint8)[:n*W*H].reshape(n,H,W)
fps=n/dur if dur>0 else 60.0
# 🚨 FPS GATE. The emulator renders at ~20 fps WHEN ITS WINDOW IS NOT FOCUSED, and the resulting
# clip looks entirely normal: right art, right timing, just fewer frames. Measured on Skadi2 at
# 20 Mbps, same skin and settings minutes apart: 19.76 fps tabbed out -> 60.01 fps focused. That
# supersedes the two explanations this project carried for the 20-23 fps clips (host load from a
# concurrent puppeteer run, and screenrecord bitrate throttling); it retroactively accounts for
# celnew, skanew and eyjanew, and it is why an under-rate clip is RE-TAKEN rather than annotated.
#
# ⚠️ Measure the rate on the RAW recording, here, never on a trimmed copy. `ffmpeg -ss -c copy`
# shifts the frame count against the container duration: cel reads 60.01 on the raw and 47.94 on
# its own `-c copy` trim, so auditing trimmed files invents degradations that never happened.
MIN_FPS = 59.0
if fps < MIN_FPS:
    print(f"[proc] \u26d4 {fps:.2f} fps, under {MIN_FPS:.0f}. FOCUS THE EMULATOR WINDOW AND RETAKE.")
    print("[proc]    Not written: an under-rate clip is retaken, not annotated.")
    sys.exit(5)
lum=G.reshape(n,-1).mean(axis=1)
print(f"[proc] {W}x{H} {n}f {dur:.1f}s {fps:.1f}fps")
# RULE 3: a real entrance BLACKS OUT. Find the darkest run; require it to be near-0.
dark=np.where(lum<6)[0]
if len(dark)==0:
    print(f"[proc] ⛔ NO BLACKOUT (min luma {lum.min():.1f}) — the ▶ tap missed hidden chrome. RETAKE."); sys.exit(3)
b0,b1=int(dark[0]),int(dark[-1])
# t0 = first frame after the blackout where the picture returns (entrance frame 1)
post=np.where(lum[b1:]>12)[0]
t0=int(b1+post[0]) if len(post) else b1
print(f"[proc] blackout frames {b0}-{b1} ({b0/fps:.2f}-{b1/fps:.2f}s) → entrance t0 = frame {t0} ({t0/fps:.2f}s) ✅")
t0s=t0/fps
subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",raw,"-c","copy",ent])
# match the ORIGINAL reference format exactly: 900x416 (non-uniform squash), 30 fps
subprocess.run(["ffmpeg","-y","-v","error","-ss",f"{t0s:.3f}","-i",raw,
    "-vf","scale=900:416:flags=lanczos,fps=30","-c:v","libx264","-crf","12","-pix_fmt","yuv420p",fresh])
print(f"[proc] ✅ wrote {fresh.split('/')[-1]} (900x416/30 from t0) + {ent.split('/')[-1]}")
print("[proc] 🔑 t0 is the blackout-exit; still derive final REFOFF from GEOMETRY when scoring.")
PY
}

ensure_emulator
ensure_menu || exit 1
navigate_to_lookbook
if [ "$MODE" = "auto" ]; then
  log "auto-finding a scoreable entrance skin…"
  python3 "$HERE/autofind_l2d.py" --max-scroll=30 --motion=5 || { log "ERROR: none found"; exit 1; }
  adb shell input keyevent 4; sleep 3          # viewer → preview (record re-opens it)
else
  log "scroll x$SCROLLS → cell ($TX,$TY)"
  for k in $(seq 1 "$SCROLLS"); do sw 500 430 500 110; sleep 0.5; done
  t "$TX" "$TY"; sleep 4
fi
record_entrance
process; rc=$?
log "done (rc=$rc)  [rc=3 → ▶ missed, retake]. Stop the emulator with: adb emu kill"
exit $rc
