#!/bin/zsh
# QA the additive-skip change on every skin it can reach.
#
# 51 systems across 10 skins start DRAWING that previously did not, and 8 of those skins have
# no reference capture — so the only question a corpus render can answer is "does anything blow
# out?", not "is it closer". Renders each affected skin twice (shipped vs `?addskip=1`) and
# diffs, so a blow-out shows as a large mean delta or a saturated frame.
set -e
HERE=${0:A:h}
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
export NODE_PATH=$HERE/node_modules
OUT=${1:?output dir}
SHOTS=${2:-1,4,8,12,17}
mkdir -p $OUT
skins=$(python3 -c "import json;print('\n'.join(json.load(open('/tmp/affected_add.json'))))")
echo "$skins" | while read -r sk; do
  [[ -z "$sk" ]] && continue
  # The skel FILE name is not derivable from the DIRECTORY name — glob it, and skip
  # portrait/_Start the way every other corpus runner here does.
  skel=$(ls "$ROOT/$sk" | grep '\.skel$' | grep -v portrait | grep -v '_Start' | head -1)
  [[ -z "$skel" ]] && { echo "NO-SKEL $sk"; continue; }
  enc=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  encs=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  op=$(python3 -c "
import re,sys;m=re.match(r'(char_\d+_[a-z0-9]+)',sys.argv[1]);print(m.group(1) if m else sys.argv[1])" "$sk")
  bd=$(python3 -c "
import urllib.parse,sys
u='http://localhost:3060/api/assets/textures/skinpack/'+sys.argv[1]+'/'+urllib.parse.quote(sys.argv[2],safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))" "$op" "$sk")
  for variant in new old; do
    d=$OUT/${sk}__${variant}
    [[ -d "$d" && -n "$(ls -A $d 2>/dev/null)" ]] && continue
    rm -rf $d; mkdir -p $d
    ex="$bd"
    [[ $variant == old ]] && ex="addskip=1&$bd"
    EXTRA="$ex" node $HERE/rec.js "/spine/DynIllust/${enc}/${encs}" $d "$SHOTS" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk $variant"
  done
  printf "  %s\n" "$sk"
done
echo "QA-RENDER-DONE"
