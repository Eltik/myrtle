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
# SCORING BASIS: pinned to DISPLAY density (`rt2048=0`).
#
# The viewer now renders at the CLIENT's own target — a square 2048 RT for the idle, the
# 2340x1080 screen surface for the entrance (see `dynchar-idle-2048-render-target`). That is a
# production-fidelity change, and it deliberately does NOT define the scoring basis:
#
#   * every number in the memory notes and the parity page was measured at display density, and
#     a basis change would silently make all of them incomparable;
#   * MADC PENALISES the sharper render — our fine detail is ~0.80x the capture's and slightly
#     misplaced, so resolving it better costs more per pixel than leaving it soft
#     (`dynchar-supersampling-measured-worse`). Scoring at the shipping density would read as a
#     regression while the render got closer to the client;
#   * it is several times slower (a single cet entrance frame goes 20s -> 70s under the harness's
#     software GL), which would make a corpus sweep impractical.
#
# Measured cost of the shipping density, for the record: ska 10.388 -> 10.466, exc 8.916 -> 8.951,
# cel 18.006 -> 17.941 — small and MIXED, i.e. it helps where our detail is well placed.
# Pass `rt2048=1` in EXTRA to score what actually ships.
case "$extra" in
  *rt2048=*) ;;                                   # caller chose the basis explicitly
  "")        extra="rt2048=0" ;;
  *)         extra="$extra&rt2048=0" ;;
esac
[[ -n "$extra" ]] && extra="$extra&$bd" || extra="$bd"
enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skindir''',safe=''))")
fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skelfile''',safe=''))")
out=$SP/new_${label}_${key}; rm -rf $out; mkdir -p $out
shots=$(python3 -c "print(','.join(f'{float(b)+$off:.4f}' for b in '$beats'.split(',')))")
EXTRA="$extra" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" $out "$shots" 900 416 >/dev/null 2>&1
printf "%-16s %-4s " "$label" "$key"
# Print the registration number on the same line as the score. MADC alone cannot distinguish
# "missing content" from "right content, wrong place" — see mad.py's MEAN r note.
# REFERENCE CLIP. `_game_fresh` is the scoring baseline every recorded number was measured
# against and it is NEVER overwritten. `REF_SUFFIX=_settled` selects a LONGER re-capture of the
# same subject, which `settled.sh` needs because the originals stop within 0.5 to 2.0 s of their
# own hand-off. Two files per subject, so a longer clip can never silently move an all8.sh
# baseline: a re-encode alone shifts MADC even when the framing is identical.
ref=$HERE/REF_NEW/${key}${REF_SUFFIX:-_game_fresh}.mp4
# MADC_DC=1 adds mad.py's signed mean-luma readout to the line. It changes no score, it only
# surfaces a systematic brightness BIAS that an absolute-difference metric cannot see.
python3 $HERE/mad.py $out "$ref" "$beats" --inner --offset=$off --dy=0 --fps=30 ${MADC_DC:+--dc} ${MADC_NODC:+--nodc} \
  | awk '/MEAN MADC/{m=$0} /MEAN r/{r=$4} /MEAN DC/{d=$4} END{printf "%s   r=%s%s\n", m, r, (d==""?"":"   DC=" d)}'
