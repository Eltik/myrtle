#!/bin/zsh
# Score an ARBITRARY skin against its fresh entrance reference. Generic version of one_fresh.sh:
# takes the skin directory instead of a hardcoded key, and globs the skel (the filename is NOT
# always the directory name — see the `_2` skins).
set -e
HERE=${0:A:h}
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
export NODE_PATH=$HERE/node_modules
key=${1:?}; skindir=${2:?}; beats=${3:?}; label=${4:?}; extra=${5:-}; off=${6:-0}
op=$(python3 -c "
import re,sys
s='$skindir'
m=re.match(r'(char_\d+_[a-z0-9]+)',s); print(m.group(1) if m else s)")
skelfile=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$skindir"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$skindir''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
[[ -n "$extra" ]] && extra="$extra&$bd" || extra="$bd"
enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skindir''',safe=''))")
fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skelfile''',safe=''))")
out=$SP/new_${label}_${key}; rm -rf $out; mkdir -p $out
shots=$(python3 -c "print(','.join(f'{float(b)+$off:.4f}' for b in '$beats'.split(',')))")
EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" $out "$shots" 900 416 >/dev/null 2>&1
printf "%-16s %-4s " "$label" "$key"
python3 $HERE/mad.py $out $HERE/REF_NEW/${key}_game_fresh.mp4 "$beats" --inner --offset=$off --dy=0 --fps=30 | grep "MEAN MADC"
