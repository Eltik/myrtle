#!/bin/zsh
# What are the `ram.kind == "disturb"` systems WORTH as we currently draw them?
#
# 787 of them across 36 skins, 237 with a sheet 300 px or larger. whitw2's t=2 defect is one of
# them drawn as opaque geometry instead of displacing what is behind it, and culling HER sheet
# overshoots (115.06 against the game's 99.61), so the contribution is real. Before building a
# displacement pass that would touch all 787, this measures the stakes: score each corpus skin with
# every disturb-kind system ABLATED, against its own baseline.
#
# A cull is NOT the proposed fix. It is the cheap lower bound on how load-bearing these systems are
# as currently drawn, which is what decides whether a displacement pass must reproduce their
# contribution or merely stop them painting.
HERE=${0:A:h}
cd $HERE
typeset -A DIR BEATS REFOFF
eval "$(python3 - <<'PY'
import re
src=open("all8.sh").read()
for name,pat in (("DIR",r"DIR\[(\w+)\]='([^']+)'"),("BEATS",r"BEATS\[(\w+)\]=\"([^\"]+)\""),("REFOFF",r"REFOFF\[(\w+)\]=([-\d.]+)")):
    for k,v in re.findall(pat,src): print(f"{name}[{k}]='{v}'")
PY
)"
keys=${@:-"ska exc cel mly mue eyja cet wis kalts whitw2 fugue chyue"}
printf "%-8s %-6s %-24s %-24s %s\n" "skin" "n" "baseline" "disturb ABLATED" "delta"
for k in ${=keys}; do
  d=$DIR[$k]; b=$BEATS[$k]; off=${REFOFF[$k]:-0}
  [[ -z "$d" ]] && continue
  idx=$(python3 - "$d" <<'PY'
import json,os,sys
d=sys.argv[1]
base=f"/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust/{d}"
f=[x for x in os.listdir(base) if x.endswith("_Start[particles].json")]
if not f: print(""); raise SystemExit
j=json.load(open(os.path.join(base,f[0])))
sysl=j['systems'] if isinstance(j,dict) and 'systems' in j else (j if isinstance(j,list) else [v for v in j.values() if isinstance(v,list)][0])
print(",".join(str(i) for i,s in enumerate(sysl) if (s.get('ram') or {}).get('kind')=='disturb'))
PY
)
  n=$([[ -z "$idx" ]] && echo 0 || echo $idx | tr ',' '\n' | wc -l | tr -d ' ')
  A=$($HERE/score_new.sh $k "$d" "$b" base "" $off 2>/dev/null | grep -oE "MEAN MADC = [0-9.]+ +over [0-9]+ beats +r=[-0-9.]+" | sed -E 's/MEAN MADC = ([0-9.]+).*r=(.*)/\1 r=\2/')
  if [[ "$n" == "0" ]]; then printf "%-8s %-6s %-24s %-24s %s\n" "$k" "$n" "$A" "(no disturb systems)" "0.000"; continue; fi
  B=$($HERE/score_new.sh $k "$d" "$b" distoff "psoff=$idx" $off 2>/dev/null | grep -oE "MEAN MADC = [0-9.]+ +over [0-9]+ beats +r=[-0-9.]+" | sed -E 's/MEAN MADC = ([0-9.]+).*r=(.*)/\1 r=\2/')
  D=$(python3 -c "
a='$A'.split()[0]; b='$B'.split()[0]
print(f'{float(b)-float(a):+8.3f}' if a and b else '   n/a')" 2>/dev/null)
  printf "%-8s %-6s %-24s %-24s %s\n" "$k" "$n" "$A" "$B" "$D"
done
