#!/bin/zsh
# Corpus QA render: every deployed skin at 5 beats, for anomaly detection.
# Skel filename is NOT derivable from the directory name - glob it, skip portrait/_Start.
set -e
OP=/Users/eltik/Documents/Coding/myrtle/scratchpad/opcheck
ROOT=/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust
export NODE_PATH=$OP/node_modules
OUT=${0:A:h}/qaNEWNEW
mkdir -p $OUT
i=0
for d in $ROOT/*(/); do
  sk=${d:t}
  i=$((i+1))
  o=$OUT/$sk
  [[ -d "$o" && -n "$(ls -A $o 2>/dev/null)" ]] && continue
  skel=$(ls "$d" | grep '\.skel$' | grep -v portrait | grep -v '_Start' | head -1)
  [[ -z "$skel" ]] && { echo "NO-SKEL $sk"; continue; }
  enc=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$sk")
  encs=$(python3 -c "import urllib.parse,sys;print(urllib.parse.quote(sys.argv[1],safe=''))" "$skel")
  rm -rf $o; mkdir -p $o
  EXTRA="" node $OP/rec.js "/spine/DynIllust/${enc}/${encs}" $o "1,4,8,12,17" 900 416 >/dev/null 2>&1 || echo "RENDER-FAIL $sk"
  printf "\r  %3d %s" $i "$sk"
done
echo ""
echo "QA-DONE"
