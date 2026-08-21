#!/bin/zsh
# Cello t=10 CAMERA-LEAD zero-crossing — the one experiment her defect needs.
#
# Her t=10 is a pure vertical shift of +22 px (afit: sx 0.9995, sy 1.0025, dx 0, MAD_Y
# 42.36 -> 17.14) and per-block search says it is the WHOLE FRAME, i.e. the camera. Her camera
# centre moves 20.31 screen px/frame there — 10-40x faster than any other reference — so this is
# the only place in the corpus where a camera-TIMING correction is measurable at all.
#
# ⚠️ ONE BEAT, single-shot list, on purpose. A 7-beat score run averages this against six beats
# whose camera is static, and the mean then minimises at lead=0 — which is exactly how `?camlead`
# was refuted before. See `dynchar-camera-phase-only-measurable-on-cel`.
#
# Usage: ./celt10.sh            (needs the backend on :3060 and vite on :3000)
set -e
HERE=${0:A:h}
export NODE_PATH=$HERE/node_modules
D='char_245_cello_sale#12'
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
skel=$(ls $ROOT/$D/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$D''',safe=''))")
fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/char_245_cello/'+urllib.parse.quote('''$D''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
[ -d /tmp/celref ] || { mkdir -p /tmp/celref
  ffmpeg -v error -i $HERE/REF_NEW/cel_game_fresh.mp4 -vf fps=30 -start_number 0 /tmp/celref/%05d.png; }
for v in 0 0.0167 0.0333 0.0500; do
  rm -rf /tmp/celt10_$v
  EXTRA="camlead=$v&rt2048=0&$bd" node $HERE/rec.js \
        "/spine/DynIllust/$enc/$fenc" /tmp/celt10_$v "9.967" 900 416 >/dev/null 2>&1 || true
done
python3 - "$HERE" <<'PY'
import sys, glob, numpy as np
sys.path.insert(0, sys.argv[1])
from afit import y, fit
from PIL import Image
g = y(np.asarray(Image.open('/tmp/celref/00300.png').convert('RGB')).astype(float))
print(f"{'camlead':>9} {'sx':>7} {'sy':>7} {'dx':>4} {'dy':>4} {'MAD@id':>8} {'MAD@fit':>8}")
for v in ('0', '0.0167', '0.0333', '0.0500'):
    p = sorted(glob.glob(f'/tmp/celt10_{v}/*.png'))
    if not p:
        print(f"{v:>9}   NO FRAME (backend down? a black run is a capture failure)"); continue
    o = y(np.asarray(Image.open(p[0]).convert('RGB')).astype(float))
    if o.mean() < 1:
        print(f"{v:>9}   BLACK (capture failed)"); continue
    sx, sy, dx, dy, mad = fit(o, g)
    base = np.abs(o[20:-20, 56:844] - g[20:-20, 56:844]).mean()
    print(f"{v:>9} {sx:7.4f} {sy:7.4f} {dx:4} {dy:4} {base:8.2f} {mad:8.2f}")
PY
