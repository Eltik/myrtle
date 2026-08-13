#!/bin/zsh
# Per-PARTICLE-SYSTEM ablation sweep — the `?psoff=` twin of layerablate.sh.
#   psablate.sh <key> <skindir> <beat> <gameframe> <off> <from> <to>
set -e
HERE=${0:A:h}
export NODE_PATH=$HERE/node_modules
key=${1:?}; skindir=${2:?}; beat=${3:?}; gframe=${4:?}; off=${5:-0}; from=${6:-0}; to=${7:-40}
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
sed "s|__GFRAME__|$gframe|; s|__KEY__|$key|; s|__HERE__|$HERE|" > /tmp/ps_mad.py <<'PYEOF'
import glob,sys,subprocess,os,numpy as np
from PIL import Image
gdir='/tmp/ps_game___KEY__'
if not os.path.isdir(gdir):
    os.makedirs(gdir)
    subprocess.run(["ffmpeg","-y","-i","__HERE__/REF_NEW/__KEY___game_fresh.mp4","-vf","fps=30",f"{gdir}/g%04d.png","-loglevel","error"])
g=sorted(glob.glob(f"{gdir}/*.png"))
G=np.asarray(Image.open(g[__GFRAME__]).convert('RGB')).astype(float)
gy=0.299*G[...,0]+0.587*G[...,1]+0.114*G[...,2]
p=sorted(glob.glob(sys.argv[1]+'/*.png'))
if not p: print("NOFRAME"); sys.exit()
O=np.asarray(Image.open(p[0]).convert('RGB')).astype(float)
oy=0.299*O[...,0]+0.587*O[...,1]+0.114*O[...,2]
print(f"{np.abs(oy-gy)[:,37:864].mean():.2f}")
PYEOF
rm -rf /tmp/psa_base; EXTRA="$bd" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" /tmp/psa_base "$rt" 900 416 >/dev/null 2>&1
base=$(python3 /tmp/ps_mad.py /tmp/psa_base)
print -r -- "  BASELINE MAD=$base"
for i in $(seq $from $to); do
  rm -rf /tmp/psa_$i
  EXTRA="psoff=$i&$bd" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" /tmp/psa_$i "$rt" 900 416 >/dev/null 2>&1
  v=$(python3 /tmp/ps_mad.py /tmp/psa_$i)
  d=$(python3 -c "print(f'{$v - $base:+.2f}')")
  [[ "$d" == "+0.00" ]] || print -r -- "  ps:$i  MAD=$v  delta=$d"
done
print -r -- "  SWEEP DONE"
