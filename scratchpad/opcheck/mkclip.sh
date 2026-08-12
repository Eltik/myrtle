#!/bin/zsh
# Rebuild ONE side-by-side parity clip for the entrance-parity artifact.
#   mkclip.sh <mly|cel|ska> <outdir>
# Top panel = OURS, bottom = the GAME (verify by the black UI side-bars: only the game
# panel has them). Shots are i/15 + OFF, where OFF is the reference-clock offset — capture
# at t+OFF so our frame depicts the same instant as the game frame at t.
#
# This script was lost once already; it lives beside one.sh so it does not vanish again.
set -e
HERE=${0:A:h}
export NODE_PATH=$HERE/node_modules
key=${1:?}; out=${2:?}
typeset -A SKEL OFF OP SKIN
OFF[mly]=-0.085; OFF[cel]=0.0171; OFF[ska]=0
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
OP[mly]=char_4064_mlynar;   SKIN[mly]='char_4064_mlynar_epoque#28'
OP[cel]=char_245_cello;     SKIN[cel]='char_245_cello_sale#12'
OP[ska]=char_1012_skadi2;   SKIN[ska]='char_1012_skadi2_iteration#2'
game=$HERE/REF/${key}_game_salvaged.mp4
dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 $game)
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/${OP[$key]}/'+urllib.parse.quote('${SKIN[$key]}',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
frames=$out/f_$key; rm -rf $frames; mkdir -p $frames
shots=$(python3 -c "
n=int(float('$dur')*15)
print(','.join(f'{i/15.0+${OFF[$key]}:.4f}' for i in range(n)))")
EXTRA="$bd" node $HERE/rec.js "/spine/DynIllust/${SKEL[$key]}" $frames "$shots" 900 416 >/dev/null 2>&1
python3 - "$frames" <<'PY'
import os,sys
d=sys.argv[1]
fs=sorted((f for f in os.listdir(d) if f.endswith(".png")), key=lambda x: float(x[1:-4]))
with open(os.path.join(d,"list.txt"),"w") as o:
    for f in fs:
        o.write(f"file '{f}'\nduration 0.0666667\n")
    if fs: o.write(f"file '{fs[-1]}'\n")
print(f"{len(fs)} frames")
PY
# `-pix_fmt yuv420p` is NOT optional: from PNG input libx264 picks yuvj444p (High 4:4:4
# Predictive), which ffmpeg decodes happily and NO BROWSER CAN PLAY. The artifact showed four
# clips with a correct duration and a blank frame because of exactly this.
ffmpeg -y -v error -f concat -safe 0 -i $frames/list.txt -r 15 -crf 18 -pix_fmt yuv420p $out/mine_$key.mp4
# `-shortest`: without it the stack pads to the longer input and the file balloons.
ffmpeg -y -v error -i $out/mine_$key.mp4 -i $game -filter_complex \
  "[0:v]scale=720:333[a];[1:v]scale=720:333[b];[a][b]vstack" \
  -r 15 -an -c:v libx264 -crf 36 -preset slow -pix_fmt yuv420p -profile:v high -movflags +faststart -shortest $out/clip_$key.mp4
rm -rf $frames
ls -la $out/clip_$key.mp4
