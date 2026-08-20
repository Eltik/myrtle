#!/bin/zsh
# Render EVERY deployed skin at a fixed beat set into <outdir>, for before/after corpus QA.
#
# 74 of the 82 deployed skins have no reference capture, so the only question a corpus render
# can answer is "does anything break" — not "is it closer". Run it once against each asset
# tree and diff the frames.
#
#   qa_corpus.sh <outdir> [beats]
#
# NOTE: deliberately NO `set -e`. Under zsh a `cond && continue` guard that evaluates FALSE
# returns 1 and aborts the whole loop with errexit on — this script silently rendered one skin
# and stopped the first time. Handle failures explicitly instead.
HERE=${0:A:h}
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
export NODE_PATH=$HERE/node_modules
OUT=${1:?outdir}
SHOTS=${2:-1,4,8,12,17}
mkdir -p $OUT
i=0
for d in $ROOT/*(/); do
  sk=${d:t}; i=$((i+1))
  o=$OUT/$sk
  if [[ -d "$o" && -n "$(ls -A $o 2>/dev/null)" ]]; then continue; fi
  # The skel FILE name is not derivable from the DIRECTORY name — glob it, and skip
  # portrait/_Start the way every other corpus runner here does.
  skel=$(ls "$d" | grep '\.skel$' | grep -v portrait | grep -v '_Start' | head -1)
  if [[ -z "$skel" ]]; then echo "NO-SKEL $sk"; continue; fi
  enc=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  encs=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  op=$(python3 -c "
import re,sys;m=re.match(r'(char_\d+_[a-z0-9]+)',sys.argv[1]);print(m.group(1) if m else sys.argv[1])" "$sk")
  # gap-fill QA is VACUOUS without a backdrop= — see the sceneCoversFrame note.
  bd=$(python3 -c "
import urllib.parse,sys
u='http://localhost:3060/api/assets/textures/skinpack/'+sys.argv[1]+'/'+urllib.parse.quote(sys.argv[2],safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))" "$op" "$sk")
  rm -rf $o; mkdir -p $o
  # Pinned to DISPLAY density like score_new.sh and mkclip_any.sh: the viewer now ships at the
  # client's own render targets, which is ~3.5x heavier under the harness's software GL. A QA
  # sweep only asks "is anything BROKEN", and blank/blown/dark/flat all read identically at
  # either density — so pay the cheap one. See `dynchar-idle-2048-render-target`.
  EXTRA="rt2048=0&$bd" node $HERE/rec.js "/spine/DynIllust/${enc}/${encs}" $o "$SHOTS" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk"
  printf "\r  %3d %s                    " $i "$sk"
done
echo ""
echo "QA-DONE $OUT"
