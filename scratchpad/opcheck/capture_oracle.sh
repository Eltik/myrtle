#!/bin/zsh
# Capture a REAL oracle from the game running in the emulator, replacing the salvaged
# A/B-sign-only reference.  See memory: dynchar-fresh-oracle-capture.
#
#   capture_oracle.sh record <mly|cel|ska>     start recording; Ctrl-C or `stop` when done
#   capture_oracle.sh stop   <mly|cel|ska>     stop + pull + convert to the 900x416 15fps format
#
# CRITICAL: the emulator must be 2340x1080.  The originals are 900x416 (aspect 2.1635) which is
# a 2340x1080 device; the game FITS the illustration to the screen, so any other aspect shows a
# different amount of scene and the capture is not comparable to our renderer's framing.
set -e
HERE=${0:A:h}
key=${2:?usage: capture_oracle.sh <record|stop> <mly|cel|ska>}
RAW=/sdcard/cap_${key}.mp4
OUT=$HERE/REF_NEW
mkdir -p $OUT

want_w=2340; want_h=1080
size=$(adb shell wm size | tr -d '\r' | awk -F': ' '/Physical/{print $2}')
if [[ "$size" != "${want_w}x${want_h}" && "$size" != "${want_h}x${want_w}" ]]; then
  echo "REFUSING: display is $size, need ${want_w}x${want_h}."
  echo "Set hw.lcd.width=2340 / hw.lcd.height=1080 in the AVD config and reboot."
  exit 1
fi

case ${1:?} in
record)
  adb shell rm -f $RAW
  echo "recording $key at 100 Mbps -> $RAW   (run '$0 stop $key' when the cinematic ends)"
  # 100 Mbps is effectively lossless next to the CRF36 salvage (~200 kbps).
  adb shell screenrecord --bit-rate 100000000 --size ${want_w}x${want_h} --time-limit 180 $RAW &
  echo $! > /tmp/screenrec_$key.pid
  ;;
stop)
  adb shell pkill -INT screenrecord 2>/dev/null || true
  sleep 3                       # screenrecord needs a moment to finalise the container
  adb pull $RAW $OUT/${key}_raw.mp4
  adb shell rm -f $RAW
  # Match the historical reference format exactly: 900x416, 15 fps.
  #
  # 🚨 THIS SIZE IS GEOMETRICALLY WRONG, and every reference in REF_NEW/ carries the error.
  # The device frame is 2340x1080 (aspect 2.166667); forcing 900x416 scales x by 900/2340 =
  # 0.384615 and y by 416/1080 = 0.385185 — a NON-UNIFORM scale that stretches every reference
  # vertically by 1.001481. Measured against it with an anisotropic registration fit, sy lands on
  # 1.0000 and sx on 0.9985 (= 1/1.001481) at nearly every beat on every skin, and correcting it
  # is worth MAD_Y -0.44 corpus-wide. `mad.py --srcaspect=2340:1080` compensates at scoring time.
  #
  # An exact-aspect size is 1170x540 (2340/2 x 1080/2). Changing this line would make new
  # references incomparable with the existing set, so it is left alone deliberately — re-capture
  # the WHOLE set at 1170x540 or none of it.
  # `REF_SIZE=1170x540 REF_FPS=30` re-records at the EXACT aspect (2340/2 x 1080/2), which needs
  # `mad.py --srcaspect=none` when scoring against it — the correction exists to undo the 900x416
  # squash and would double-apply here. Default stays 900x416 so a one-off re-record cannot
  # silently produce a reference incomparable with the other eight.
  size=${REF_SIZE:-900x416}; fps=${REF_FPS:-15}
  ffmpeg -y -v error -i $OUT/${key}_raw.mp4 -vf "scale=${size/x/:}:flags=lanczos,fps=${fps}" \
         -c:v libx264 -crf 12 -pix_fmt yuv420p $OUT/${key}_game_fresh.mp4
  echo "wrote $OUT/${key}_game_fresh.mp4"
  ffprobe -v error -select_streams v:0 \
          -show_entries stream=width,height,nb_frames,r_frame_rate -of csv=p=0 \
          $OUT/${key}_game_fresh.mp4
  echo
  echo "NEXT: derive t0 from GEOMETRY, never from the score -- fitting t0 to the metric makes"
  echo "every later measurement circular.  The existing offsets (mly -0.085, cel +0.0171, ska 0)"
  echo "were derived that way and are calibrated to the OLD trims; they do not carry over."
  ;;
*) echo "usage: $0 <record|stop> <mly|cel|ska>"; exit 1;;
esac
