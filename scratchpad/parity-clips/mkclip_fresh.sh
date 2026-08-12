#!/bin/zsh
# Side-by-side against the CLEAN 2340x1080 entrance capture. TOP = ours, BOTTOM = the game.
# Differences from mkclip.sh: the reference is REF_NEW/<key>_game_fresh.mp4 (30 fps, trimmed to
# the flash-anchored t0) and OFF is the re-measured per-skin alignment, not the salvage-era one.
set -e
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
export NODE_PATH=$OP/node_modules
key=${1:?}; out=${2:?}
REF=$OP/REF_NEW/${key}_game_fresh.mp4
secs=${3:-$(ffprobe -v error -show_entries format=duration -of csv=p=0 $REF)}
typeset -A SKEL OFF
OFF[mly]=0.033; OFF[cel]=-0.033; OFF[ska]=0.067
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
bd=$(python3 -c "
import urllib.parse
m={'mly':('char_4064_mlynar','char_4064_mlynar_epoque#28'),'cel':('char_245_cello','char_245_cello_sale#12'),'ska':('char_1012_skadi2','char_1012_skadi2_iteration#2')}
op,sk=m['$key']
u='http://localhost:3060/api/assets/textures/skinpack/'+op+'/'+urllib.parse.quote(sk,safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
frames=$out/f_$key; rm -rf $frames; mkdir -p $frames
FPS=${FPS:-30}
shots=$(python3 -c "print(','.join(f'{i/float($FPS)+${OFF[$key]}:.4f}' for i in range(int($FPS*$secs))))")
EXTRA="$bd" node $OP/rec.js "/spine/DynIllust/${SKEL[$key]}" $frames "$shots" 900 416 >/dev/null 2>&1
python3 - "$frames" "$FPS" > $frames/list.txt <<'PY'
import os, sys
d, fps = sys.argv[1], float(sys.argv[2])
fs = sorted((f for f in os.listdir(d) if f.endswith(".png")), key=lambda s: float(s[1:-4]))
for f in fs:
    print(f"file '{os.path.join(d,f)}'"); print(f"duration {1.0/fps:.7f}")
PY
ffmpeg -y -v error -f concat -safe 0 -i $frames/list.txt -r $FPS -crf 14 -pix_fmt yuv420p $out/ours_$key.mp4
ffmpeg -y -v error -i $out/ours_$key.mp4 -i $REF \
  -filter_complex "[0:v]scale=720:333,fps=${FPS}[a];[1:v]scale=720:333,fps=${FPS}[b];[a][b]vstack" \
  -an -r $FPS -c:v libx264 -crf ${SBS_CRF:-24} -preset slow -g $FPS -keyint_min $FPS -profile:v main -pix_fmt yuv420p -movflags +faststart $out/clipf_$key.mp4
rm -rf $frames
echo "  CLIP $key $(du -h $out/clipf_$key.mp4 | cut -f1)"
