#!/bin/zsh
# CONTENT-CLOCK basis: score every corpus skin twice, container clock and content clock.
#
# The reference captures dropped frames under host load, so some game frames repeat their
# predecessor and their CONTENT is older than their container timestamp. `cclock.py` maps each beat
# to the time its frame's content was last updated; this renders our side at those times too, so
# content is compared against content.
#
# ⚠️ ADDITIVE. The container-clock arm is printed alongside and is the basis every recorded number
# was measured on. Nothing here overwrites it.
#
# Beat sets, skin directories and trims come from all8.sh, never retyped: an earlier run scored
# eight skins on guessed beats and read wis at r 0.237 against her known 0.972.
HERE=${0:A:h}
cd $HERE
SP=/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/063b9339-29f2-4cd8-8382-2512e303467d/scratchpad
# Parse all8.sh's three maps with a real parser rather than re-evaluating shell: its lines pack
# several assignments per line with `;` separators and mixed quoting, and an awk/eval version of
# this silently produced EMPTY beat sets on the first run.
typeset -A DIR BEATS REFOFF
eval "$(python3 - <<'PY'
import re
src = open("all8.sh").read()
for name, pat in (("DIR", r"DIR\[(\w+)\]='([^']+)'"),
                  ("BEATS", r"BEATS\[(\w+)\]=\"([^\"]+)\""),
                  ("REFOFF", r"REFOFF\[(\w+)\]=([-\d.]+)")):
    for k, v in re.findall(pat, src):
        print(f"{name}[{k}]='{v}'")
PY
)"
keys=${@:-"ska exc cel mly mue eyja cet wis kalts whitw2"}
printf "%-8s %-24s %-24s %s\n" "skin" "container clock" "content clock" "delta"
for k in ${=keys}; do
  d=$DIR[$k]; b=$BEATS[$k]; off=${REFOFF[$k]:-0}
  [[ -z "$d" ]] && { echo "$k: no DIR"; continue; }
  ref=$HERE/REF_NEW/${k}_game_fresh.mp4
  [[ -f "$ref" ]] || { echo "$k: no ref"; continue; }
  op=$(echo $d | sed -E 's/^(char_[0-9]+_[a-z0-9]+).*/\1/')
  skel=$(ls "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/$d"/dyn_illust_*.skel | grep -v _Start | head -1 | xargs basename)
  enc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$d''',safe=''))")
  fenc=$(python3 -c "import urllib.parse;print(urllib.parse.quote('''$skel''',safe=''))")
  bd=$(python3 -c "
import urllib.parse
u='http://localhost:3060/api/assets/textures/skinpack/$op/'+urllib.parse.quote('''$d''',safe='')+'.png'
print('backdrop='+urllib.parse.quote(u,safe=''))")
  ct=$(python3 cclock.py "$ref" "$b")
  # Render BOTH sets of times in one pass so the two arms share a render, which removes the
  # renderer itself as a source of difference between them.
  shots=$(python3 -c "
bs='$b'.split(','); cs='$ct'.split(',')
ts=sorted({f'{float(x)+$off:.4f}' for x in bs+cs})
print(','.join(ts))")
  out=$SP/cc_$k; rm -rf $out; mkdir -p $out
  EXTRA="rt2048=0&$bd" node $HERE/rec.js "/spine/DynIllust/$enc/$fenc" $out "$shots" 900 416 >/dev/null 2>&1
  A=$(python3 mad.py $out "$ref" "$b" --inner --offset=$off --fps=30 2>/dev/null | awk '/MEAN MADC/{m=$4} /MEAN r/{r=$4} END{printf "%7.3f  r=%s", m, r}')
  B=$(python3 mad.py $out "$ref" "$b" --inner --offset=$off --fps=30 --ourtimes=$ct 2>/dev/null | awk '/MEAN MADC/{m=$4} /MEAN r/{r=$4} END{printf "%7.3f  r=%s", m, r}')
  D=$(python3 -c "print(f'{${B%% *}-${A%% *}:+7.3f}')" 2>/dev/null || echo "")
  printf "%-8s %-24s %-24s %s\n" "$k" "$A" "$B" "$D"
done
