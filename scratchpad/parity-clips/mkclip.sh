#!/bin/zsh
# Rebuild one side-by-side parity clip. TOP = ours, BOTTOM = the game.
# Usage: mkclip.sh <mly|cel|ska> <outdir> [seconds]
set -e
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
export NODE_PATH=$OP/node_modules
key=${1:?}; out=${2:?}
# Clip length is PER SKIN — derive it from the reference capture rather than assuming one
# number. Hardcoding 18 truncated Skadi, whose entrance runs 22.3 s, so her clip lost 4 s and
# the game half of the stack ran past ours.
typeset -A REFK; REFK[mly]=mly; REFK[cel]=cel; REFK[ska]=ska
secs=${3:-$(ffprobe -v error -show_entries format=duration -of csv=p=0 $OP/REF/${key}_game_salvaged.mp4)}
typeset -A SKEL OFF
OFF[mly]=-0.085; OFF[cel]=0.0171; OFF[ska]=0
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
bd=$(python3 -c "
import urllib.parse,sys
m={'mly':('char_4064_mlynar','char_4064_mlynar_epoque#28'),'cel':('char_245_cello','char_245_cello_sale#12'),'ska':('char_1012_skadi2','char_1012_skadi2_iteration#2')}
op,sk=m['$key']
u='http://localhost:3060/api/assets/textures/skinpack/'+op+'/'+urllib.parse.quote(sk,safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
frames=$out/f_$key; rm -rf $frames; mkdir -p $frames
# OUR half is a SIMULATION — it can be sampled at any rate, unlike the 15 fps capture. At 15
# fps the entrance genuinely stutters (median inter-frame |dY| ~13 luma), which is what read as
# "skipping": the container was already perfect CFR with zero irregular gaps. Render ours at
# FPS and lift the game half to the same rate so the stack stays in sync.
FPS=${FPS:-30}
shots=$(python3 -c "print(','.join(f'{i/float($FPS)+${OFF[$key]}:.4f}' for i in range(int($FPS*$secs))))")
EXTRA="$bd" node $OP/rec.js "/spine/DynIllust/${SKEL[$key]}" $frames "$shots" 900 416 >/dev/null 2>&1
python3 - "$frames" "$FPS" > $frames/list.txt <<'PY'
import os, sys
d, fps = sys.argv[1], float(sys.argv[2])
fs = sorted((f for f in os.listdir(d) if f.endswith(".png")), key=lambda s: float(s[1:-4]))
for f in fs:
    print(f"file '{os.path.join(d,f)}'")
    print(f"duration {1.0/fps:.7f}")
PY
ffmpeg -y -v error -f concat -safe 0 -i $frames/list.txt -r $FPS -crf 18 -pix_fmt yuv420p $out/ours_$key.mp4
ffmpeg -y -v error -i $out/ours_$key.mp4 -i $OP/REF/${key}_game_salvaged.mp4 \
  -filter_complex "[0:v]scale=720:333,fps=${FPS}[a];[1:v]scale=720:333,fps=${FPS}[b];[a][b]vstack" \
  -an -r $FPS -c:v libx264 -crf 36 -preset slow -g $FPS -keyint_min $FPS -profile:v main -pix_fmt yuv420p -movflags +faststart $out/clip_$key.mp4
echo "CLIP-DONE $key $(du -h $out/clip_$key.mp4 | cut -f1)"
