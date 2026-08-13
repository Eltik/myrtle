#!/bin/zsh
# Per-LAYER ablation sweep: which single scene layer, if removed, IMPROVES the frame?
#
# This is the method that found Muelsyse's doubled cross-root twins: a layer whose removal
# HELPS is content the game does not draw (or draws differently), and a layer whose removal
# HURTS is content we need. Group ablation cannot see either — it only says "the scene owns
# these pixels".
#
#   layerablate.sh <key> <skindir> <beat> <gameframe> <off> <side> <count>
set -e
HERE=${0:A:h}
export NODE_PATH=$HERE/node_modules
key=${1:?}; skindir=${2:?}; beat=${3:?}; gframe=${4:?}; off=${5:-0}; side=${6:-bg}; count=${7:-24}
op=$(python3 -c "
import re; m=re.match(r'(char_\d+_[a-z0-9]+)','$skindir'); print(m.group(1) if m else '$skindir')")
enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skindir''',safe=''))")
skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$skindir"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$skindir''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
rt=$(python3 -c "print(f'{$beat + $off:.4f}')")
cat > /tmp/la_mad.py <<PYEOF
import glob,sys,subprocess,os,numpy as np
from PIL import Image
gdir='/tmp/la_game_$key'
if not os.path.isdir(gdir):
    os.makedirs(gdir)
    subprocess.run(["ffmpeg","-y","-i","$HERE/REF_NEW/${key}_game_fresh.mp4","-vf","fps=30",f"{gdir}/g%04d.png","-loglevel","error"])
g=sorted(glob.glob(f"{gdir}/*.png"))
G=np.asarray(Image.open(g[$gframe]).convert('RGB')).astype(float)
gy=0.299*G[...,0]+0.587*G[...,1]+0.114*G[...,2]
p=sorted(glob.glob(sys.argv[1]+'/*.png'))
if not p: print("NOFRAME"); sys.exit()
O=np.asarray(Image.open(p[0]).convert('RGB')).astype(float)
oy=0.299*O[...,0]+0.587*O[...,1]+0.114*O[...,2]
print(f"{np.abs(oy-gy)[:,37:864].mean():.2f}")
PYEOF
rm -rf /tmp/la_base; EXTRA="$bd" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" /tmp/la_base "$rt" 900 416 >/dev/null 2>&1
base=$(python3 /tmp/la_mad.py /tmp/la_base)
echo "  BASELINE MAD=$base"
for i in $(seq 0 $count); do
  rm -rf /tmp/la_$i
  EXTRA="abl=$side:$i&$bd" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" /tmp/la_$i "$rt" 900 416 >/dev/null 2>&1
  v=$(python3 /tmp/la_mad.py /tmp/la_$i)
  d=$(python3 -c "print(f'{$v - $base:+.2f}')")
  print -r -- "  $side:$i  MAD=$v  delta=$d"
done
