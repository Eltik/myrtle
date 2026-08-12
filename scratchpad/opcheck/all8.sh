#!/bin/zsh
# Score ALL EIGHT captured reference skins in one call, each on its own beat set.
#
# This mapping (key -> skin directory -> beat set) had been re-derived by hand or dug out of
# the transcript once per session. It is the single source of truth now; score_new.sh stays the
# per-skin primitive and this is the corpus wrapper over it.
#
#   all8.sh <label> [EXTRA]            score every skin with the same query string
#   all8.sh <label> [EXTRA] <keys...>  score only the named skins
#
# EXTRA is the dyntest query string (e.g. "nohdr=1"), passed verbatim to every skin; the
# per-skin `backdrop=` is appended by score_new.sh, so never pass one here.
#
# Baselines as of 2026-08-10 (empty EXTRA):
#   ska 10.447  exc  8.919  cel 18.021  mly 18.073  mue 21.908  eyja 22.123  cet 18.087  wis 12.645
# (cet/exc/wis numbers are NOT comparable to anything before 2026-08-11 — see REFOFF below.)
set -e
HERE=${0:A:h}
label=${1:?label required}; extra=${2:-}; shift 2 2>/dev/null || shift 1

# ⚠️ REFOFF is LOAD-BEARING, not cosmetic. The three original reference clips were re-trimmed
# 1-2 frames off their previous cut; these are the per-reference TRIM ERROR, corroborated by each
# skin reproducing its independently-validated baseline. Omitting them costs
# ska +8.9 / cel +8.6 / mly +5.8 — big enough to be mistaken for a renderer regression.
typeset -A DIR BEATS REFOFF
REFOFF[mly]=0.033; REFOFF[cel]=-0.033; REFOFF[ska]=0.067
# cet added 2026-08-11. Her clip is trimmed 2-3 frames LATE relative to our t0 -- our render runs
# AHEAD. Corroborated THREE ways, none of them the score: the pillarbox fade alpha (a pure time
# shift cuts mean |d-alpha| from 0.0575 to 0.0055, a 10x reduction, best at 0.067), the mid-clip
# blackout 11.0-13.0 (best 0.100) and the first blackout 5.0-6.5 (best 0.100). Took the cleanest
# instrument's value. ⚠️ This changes ZERO pixels -- it only compares the right frames.
REFOFF[cet]=-0.067
# exc added 2026-08-11. Her clip is trimmed 3 frames EARLY -- our render runs BEHIND, the opposite
# sign to cet. Corroborated INDEPENDENTLY of the score by the scope's lit-disc radius: the game
# releases the aperture at t=5.10 where we release at 5.20, and shifting the whole radius
# trajectory over 4.5-5.3 cuts mean |dr| from 24.71px to 6.32px, best at 0.100. The score minimum
# then lands on the SAME 0.100 and is symmetric (9.343 at both +0.067 and +0.133), which is what a
# real trim error looks like and a tuned constant does not. ⚠️ Changes ZERO pixels.
REFOFF[exc]=0.100
# wis added 2026-08-11, and this one is BIG: SIX frames. Our render runs 0.200s AHEAD of her
# reference cut. ⚠️ The flash-anchor check that validated the other trims to ±0.083s did NOT catch
# this -- wis was one of the 3-of-8 it could not anchor, so "the trims are verified" never covered
# her. Corroborated two ways: (a) FIVE of her six beats independently pick -0.200, each collapsing
# to 6.4-7.5 from 33.6, with 2-3x degradation ONE frame either side, and the pick is CONSTANT
# across t=2..10 -- a rate error would ramp; (b) a dense 31-sample per-pixel MADC sweep over
# 1.0-4.0s, independent of the beat set, minimises at exactly 0.200 (30.727 -> 8.066) and is
# symmetric (18.005 / 8.066 / 17.756). ⚠️ Changes ZERO pixels -- the renderer did not improve.
# 🔑 t=12 (41.66) is now her ONLY bad beat; the other five are among the best in the corpus.
REFOFF[wis]=-0.200

DIR[ska]='char_1012_skadi2_iteration#2';  BEATS[ska]="3,5,7,9,13,16,19"
DIR[exc]='char_1032_excu2_sale#12';       BEATS[exc]="1,2,3,4,5,6"
DIR[cel]='char_245_cello_sale#12';        BEATS[cel]="2,5,8,10,12,14,17"
DIR[mly]='char_4064_mlynar_epoque#28';    BEATS[mly]="4,7,9,10,11,11.8,12.4,13"
DIR[mue]='char_249_mlyss_boc#8';          BEATS[mue]="3,6,9,12,15,18"
DIR[eyja]='char_1016_agoat2_epoque#34';   BEATS[eyja]="2,4,6,8"
DIR[cet]='char_4134_cetsyr_epoque#50';    BEATS[cet]="2,5,8,11,14,17,18.5"
DIR[wis]='char_1035_wisdel_sale#14';      BEATS[wis]="2,4,6,8,10,12"

# NB: `${@:-a b c}` expands the default as a SINGLE word in zsh — spell the branch out.
if (( $# )); then keys=($@); else keys=(ska exc cel mly mue eyja cet wis); fi

for k in $keys; do
  d=${DIR[$k]}
  [[ -z "$d" ]] && { echo "unknown key: $k" >&2; continue; }
  want=$(python3 -c "print(len('${BEATS[$k]}'.split(',')))")
  # A score with FEWER beats than requested is a DROPOUT (black frame), never a reading.
  # Retry once before believing it.
  for attempt in 1 2; do
    line=$($HERE/score_new.sh $k "$d" "${BEATS[$k]}" "$label" "$extra" ${REFOFF[$k]:-0} 2>/dev/null || true)
    got=$(print -r -- "$line" | grep -oE "over [0-9]+" | grep -oE "[0-9]+" || echo 0)
    [[ "$got" == "$want" ]] && break
  done
  if [[ "$got" != "$want" ]]; then
    printf "%-16s %-4s DROPOUT (%s/%s beats)\n" "$label" "$k" "$got" "$want"
  else
    print -r -- "$line"
  fi
done
