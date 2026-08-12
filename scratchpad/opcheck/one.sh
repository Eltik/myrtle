#!/bin/zsh
# MADC for ONE reference skin, with that skin's own `backdrop=` URL appended automatically.
# The three skins cannot share a single EXTRA once gap fill is involved, because the backdrop
# is per-skin — that is what this exists for.
#   one.sh <mly|cel|ska> <label> [EXTRA] [--bd]
set -e
HERE=${0:A:h}
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
export NODE_PATH=$HERE/node_modules
key=${1:?}; label=${2:?}; extra=${3:-}; wantbd=${4:-}
typeset -A SKEL OFF BEATS OP SKIN
OFF[mly]=-0.085; OFF[cel]=0.0171; OFF[ska]=0
BEATS[mly]="4,7,9,10,11,11.8,12.4,13"; BEATS[cel]="2,5,8,10,12,14,17"; BEATS[ska]="3,5,7,9,13,16,19"
SKEL[mly]="char_4064_mlynar_epoque%2328/dyn_illust_char_4064_mlynar_epoque%2328.skel"
SKEL[cel]="char_245_cello_sale%2312/dyn_illust_char_245_cello_sale%2312.skel"
SKEL[ska]="char_1012_skadi2_iteration%232/dyn_illust_char_1012_skadi2_iteration%232.skel"
OP[mly]=char_4064_mlynar;   SKIN[mly]='char_4064_mlynar_epoque#28'
OP[cel]=char_245_cello;     SKIN[cel]='char_245_cello_sale#12'
OP[ska]=char_1012_skadi2;   SKIN[ska]='char_1012_skadi2_iteration#2'
if [[ "$wantbd" == "--bd" ]]; then
  bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/${OP[$key]}/'+urllib.parse.quote('${SKIN[$key]}',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
  [[ -n "$extra" ]] && extra="$extra&$bd" || extra="$bd"
fi
out=$SP/one_${label}_${key}; rm -rf $out; mkdir -p $out
shots=$(python3 -c "print(','.join(f'{float(b)+${OFF[$key]}:.4f}' for b in '${BEATS[$key]}'.split(',')))")
EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/${SKEL[$key]}" $out "$shots" 900 416 >/dev/null 2>&1
printf "%-18s %-4s " "$label" "$key"
python3 $HERE/mad.py $out $HERE/REF/${key}_game_salvaged.mp4 "${BEATS[$key]}" --inner --offset=${OFF[$key]} --dy=2 | grep "MEAN MADC"
