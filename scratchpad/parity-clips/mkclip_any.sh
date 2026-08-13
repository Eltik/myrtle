#!/bin/zsh
# Side-by-side for ANY skin against its fresh entrance reference. TOP = ours, BOTTOM = the game.
# Generic version of mkclip_fresh.sh: takes the skin directory, globs the skel (the filename is not
# always the directory name), and derives the length from the reference clip.
set -e
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
export NODE_PATH=$OP/node_modules
key=${1:?}; skindir=${2:?}; out=${3:?}; off=${4:-0}
REF=$OP/REF_NEW/${key}_game_fresh.mp4
secs=$(ffprobe -v error -show_entries format=duration -of csv=p=0 $REF)
op=$(python3 -c "
import re; m=re.match(r'(char_\d+_[a-z0-9]+)','$skindir'); print(m.group(1) if m else '$skindir')")
skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$skindir"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$skindir''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skindir''',safe=''))")
fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
frames=$out/f_$key; rm -rf $frames; mkdir -p $frames
FPS=${FPS:-30}
shots=$(python3 -c "print(','.join(f'{i/float($FPS)+$off:.4f}' for i in range(int($FPS*$secs))))")
# Clips render on the SAME pinned basis as the scorer (`rt2048=0`, display density). The viewer
# now ships at the client's own render targets, which is several times heavier — a clip built at
# that density takes minutes per skin under the harness's software GL, and would no longer match
# the numbers on the page. See `dynchar-idle-2048-render-target`.
EXTRA="rt2048=0&$bd" node $OP/rec.js "/spine/DynIllust/$enc/$fenc" $frames "$shots" 900 416 >/dev/null 2>&1
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
rm -rf $frames $out/ours_$key.mp4
echo "  CLIP $key $(du -h $out/clipf_$key.mp4 | cut -f1)"
