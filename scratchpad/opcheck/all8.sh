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
# REGISTRATION baseline (`r`, printed on every line since 2026-08-23). MADC is blind to
# "right content, wrong place"; r is the check. Every healthy reference sits high:
#   wis .972  fugue .972  eyja .968  ska .950  exc .932  mue .928  cel .915  cet .892  mly .888
#   kalts .510   whitw2 .287     <- the only two that do not line up, and both are understood
#                                   (kalts: content, her mist; whitw2: parallax, perspective rig)
# A DROP in r means geometry moved — run `geomsearch.py` before reading a diff map.
#
# Baselines as of 2026-08-22 (empty EXTRA), on the ASPECT-CORRECTED basis:
#   wis 5.537  exc 6.266  eyja 8.693  ska 10.084  mue 11.521  cel 17.075  mly 17.192  cet 17.701
#   corpus mean 11.759                             cet 17.701 is the worst of the DEFAULT EIGHT.
#
# ⬆ 2026-08-22: EFFECT_SCENE_GAIN now tames ADDITIVE layers ONLY (11.946 -> 11.759). A
#   normal-blend layer's alpha is COVERAGE, not intensity, so damping it only dims art the game
#   draws at full strength. eyja -1.717, mue -0.119, ska/wis 0, cet +0.008, cel +0.057,
#   exc +0.081, mly +0.193. `?gainadd=0` restores the previous behaviour EXACTLY (verified:
#   it reproduces every baseline above it to 3dp), so any older measurement stays reachable.
#   Also shipped: VEIL_SAT_MAX=0.25 — `isVeil` keyed on whiteness alone and demoted fugue's
#   ORANGE flame behind the spine. Corpus-invisible (every corpus veil is sat <0.10), fugue
#   14.081 -> 13.617. `?veilsat=0` restores the whiteness-only rule.
# Previous basis, for comparing against notes written before that change:
#   wis 5.537  exc 6.185  ska 10.084  eyja 10.410  mue 11.640  mly 16.999  cel 17.018  cet 17.693
#   corpus mean 11.946
#
# Opt-in extras (NOT in the default key list, so the mean above stays comparable):
#   kalts 32.361 (8 beats, REFOFF 0.200) — captured 2026-08-23. Was 48.614 until two exporter
#                fixes landed the same day: the DROPPED CAMERA TRACK (-15.48) and the
#                windowless-meshExt entrance planes (-0.77).
#                🚨 EVERY ablation/admission result recorded on her before 2026-08-23 is VOID —
#                they were measured through a badly misaligned camera and nothing could show
#                through. Re-tested since and still refuted: cross-root particles (33.840),
#                gainadd=0 (33.875), noveil (bit-identical); scenegain=0.5 (32.790) and
#                nohdr=1 (32.671) are small compensations, not mechanisms.
#                🔑 Remaining error is a MIST the game paints over the bottom quarter that we
#                render only as a brief pulse (our `wenli` planes peak alpha 0.968 at t=3.70 and
#                are 0.000 by t=4.47; the game's mist is strong at t=2,5,6,8). Residual bottom-band
#                deficit is t=5 (+41.5) and t=8 (+24.2) luma; t=2 and t=6 are essentially solved.
#                ⚠️ Use PEARSON CORRELATION, not MADC, to check whether content is missing —
#                hers read ~0.05 against the capture (wis is 0.98) while mean and std MATCHED.
#                🔑 Her error is ONE THING: a large-scale atmospheric MIST the game sweeps across
#                the scene and we barely draw. The diff map is broad BLUE bands (game brighter)
#                through the middle with RED on the crystal shards (we brighter, because the
#                game's mist veils them). ⛔ Nothing we draw is WRONG — ablating scenebg (58.1),
#                scenefg (52.6) or the spine (45.8) all make it worse or barely move it, and
#                `psoff=0-200` is inert. It is missing content, not mis-drawn content.
#                ⛔ Also refuted on her: the sort-100 transition plane is correctly gated
#                (activeFrom 13.8, matching the game's second white-out), her post-process volume
#                is genuinely weight-0 with no clip driving it, and her dissolve amounts are all
#                0.0-0.35 so the masks are not over-carving.
#                ⚠️ `?gainadd=0` is worth **-1.499** on her — she is the biggest single loser
#                from the additive-only scene gain (see the note above).
#   fugue 9.784 (8 tuned beats, REFOFF 0.083) — captured autonomously 2026-08-22. 11.064 until
#                the composite-gamma range fix, then 10.424, then -0.640 from the meshExt planes.
#                🚨 Her deployed export was STALE (pre-2026-08-12: no `entrancePostFx.params`).
#                A correctly-staged re-export (WHOLE refs/ tree — see the exporter memory) adds
#                them and is DEPLOYED. ⛔ 2026-08-22's "27.701 -> 16.805 -> 13.617" was BOGUS: it
#                came from an UNDER-STAGED export that silently dropped 2 of her 7 layers (L5, an
#                additive pure-red glow, and L6, a sort-100 full-frame white sheet). Her true
#                baseline on correct assets is 28.064.
#                🏆 SOLVED 2026-08-23: her whole early error was ONE layer — `baizhuanchang`
#                (白转场, "white transition"), sort 100, on shader `Torappu/Particles/Dissolve/
#                Dissolve AB Double`. `is_l2d_compositor` matched only `Particles-L2D/`, so its
#                `_DissolveTex_01/_02` were never applied and it drew as a full opaque rectangle.
#                t=1.0 53.0->21.3, t=2.8 51.3->20.3. Then the two-map dissolve slots
#                (`_DissolveTex_02` never resolved cross-bundle) 15.497 -> 14.270, and extending
#                GAMMA_CAL past 1111 (she is cs 1250 and was inheriting Mlynar's 0.93; measures
#                1.03) 14.270 -> 11.064. ⛔ REFUTED as causes:
#                the layer that dies at 2.533 (its colorCurve alpha is already 0.0 by t=1.0),
#                whole-frame Gaussian blur (flat response, sigma 0..25 moves t=1.0 only 47.2->45.5),
#                composite gamma, and the particle systems (`psoff=0-40` shifts 2585 px, 0.000 MADC).
#                🔑 Remaining VISIBLE defect: the campfire at t=5.5/8.2 renders a dull dark-red
#                ember where the game has a bright orange flame.
#                (On the old generic 2,4,6,8 beats she read 23.870 — that set skipped the bad phase.)
#   whitw2 46.975 (7 beats, REFOFF 0.550) — the corpus's ONLY perspective entrance camera. 92.9 when
#                her entrance read as a static camera; then the dolly fix, the post-process volume,
#                the animator-root camera track and finally the perspective FRAME SIZE (her frame
#                extent was sqrt(3) = 1.732x too wide: the client read `2*orthoCurve[0]/skelScale`,
#                but on a perspective rig that curve is the DOLLY and its keys are DISTANCES).
#                Then the REFERENCE TRIM (see REFOFF above, -22) and CAMERA-LOCKED overlays (-2.5).
#                ⚠️ The "residual is parallax, architectural" note here was WRONG and is retracted:
#                a per-region shift test showed the top/mid/bottom bands all wanted the SAME dx
#                (t=6: -112/-117/-119), i.e. one rigid shift — a global timing error, not parallax.
#
# 🚨 NOT COMPARABLE to any figure recorded before 2026-08-20. `mad.py` now corrects the reference
# clips' geometric distortion by default: `capture_oracle.sh` encodes with `scale=900:416` from a
# 2340x1080 device frame, a NON-UNIFORM scale that stretches every reference vertically by
# 1.001481. Correcting it is worth ~0.44 MADC corpus-wide and moves every skin the same way, so
# every older number is that much PESSIMISTIC. `mad.py --srcaspect=none` reproduces the old basis.
# See `dynchar-reference-clips-aspect-distorted`.
#
# For the record, the same eight on the old (distorted) basis:
#   wis 6.750  exc 6.233  ska 10.427  eyja 10.412  mue 11.845  cel 17.518  cet 17.921  mly 17.954
#   corpus mean 12.383
# ✅ ALL EIGHT references are trim-swept. mue is the only one needing no offset.
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
# whitw2: her t0 was anchored to the HARD CUT out of black (first non-black frame). That cut is
# not the animation start — ~24 BLACK frames (0.8 s) precede it and the cinematic is already
# running behind them, so the anchor is systematically LATE. Measured optimum +0.55 s, and the
# proof is geometric rather than a score minimum: at +0.55 five of seven beats align at
# scale 1.00 / dx 0 / dy 0 with NCC 0.86-0.94, against scale 1.6-1.8 at offset 0.
REFOFF[whitw2]=0.550
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
# eyja added 2026-08-11: 3 frames, our render runs BEHIND (same sign as exc). 22.123 -> 10.409, a
# HALVING. Corroborated two ways: a dense 31-sample per-pixel sweep over 1.5-7.5s, independent of
# the beat set, minimises at exactly -0.100 in its convention (23.631 -> 11.797), and the 4-beat
# score minimises at the same +0.100 with a sharp rise either side (12.585 / 10.409 / 12.952).
REFOFF[eyja]=0.100
# fugue added 2026-08-22 — the FIRST reference captured autonomously (see
# dynchar-emulator-l2d-capture-flow). Derived GEOMETRICALLY, not from the score: render at off=0,
# then per beat sweep the GAME frame index +/-0.6s at 1-frame granularity and take the frame whose
# EDGE MAP (|dx|+|dy|, z-normalised, inner crop) correlates best — edges track STRUCTURE, so this
# aligns geometry rather than fitting MADC. All four beats agreed within one frame and landed on
# exact 1/30 multiples: t=2 -> -0.067 (edgeNCC 0.534), t=4 -> -0.067 (0.536), t=6 -> -0.100
# (0.679), t=8 -> -0.100 (0.877); median 0.083.
# 🚨 SIGN: score_new.sh renders t{beat+off} and compares it to the game frame at `beat`. The sweep
# measures "our time b matches game time b-0.083", so game b matches our b+0.083 => REFOFF=+0.083.
# Scoring the NEGATED value reads 36-38 and would falsely condemn the capture.
# The geometric value also BEATS the coarse score sweep (+0.083 -> 23.870 vs +0.10 -> 25.165); a
# 0.05-step sweep stepped over the true optimum. ⚠️ Changes ZERO pixels.
REFOFF[fugue]=0.083
REFOFF[kalts]=0.200
# ⛔ mue was swept and needs NO offset -- the score is BEST at 0.000 (21.918) and degrades
# monotonically (+0.017 -> 23.263, +0.033 -> 24.190). A dense sweep over 2.0-8.0s did prefer +0.033
# by 0.51 on a base of 11.3, but that did NOT survive on her actual beat set. 🔑 Her trim is
# CORRECT, so her error was a genuine RENDERER error -- and was duly found and fixed (the settled
# ground, 21.918 -> 17.216).

DIR[ska]='char_1012_skadi2_iteration#2';  BEATS[ska]="3,5,7,9,13,16,19"
DIR[exc]='char_1032_excu2_sale#12';       BEATS[exc]="1,2,3,4,5,6"
DIR[cel]='char_245_cello_sale#12';        BEATS[cel]="2,5,8,10,12,14,17"
DIR[mly]='char_4064_mlynar_epoque#28';    BEATS[mly]="4,7,9,10,11,11.8,12.4,13"
DIR[mue]='char_249_mlyss_boc#8';          BEATS[mue]="3,6,9,12,15,18"
DIR[eyja]='char_1016_agoat2_epoque#34';   BEATS[eyja]="2,4,6,8"
DIR[cet]='char_4134_cetsyr_epoque#50';    BEATS[cet]="2,5,8,11,14,17,18.5"
DIR[wis]='char_1035_wisdel_sale#14';      BEATS[wis]="2,4,6,8,10,12"
# NINTH reference, added 2026-08-12 and deliberately OUT of the default key list below.
# `whitw2_game_fresh.mp4` was built from the previously-unscored `whitw2_entrance.mp4` capture
# (2340x1080 @60, already on disk). t0 = 3.9667 s, derived GEOMETRICALLY from the hard black->lit
# cut at the start of the cinematic (frames 95..118 are pure 0.0, frame 119 is 95.69) and
# cross-checked against the white-out: entranceDuration 14.5 puts the fade end within 0.17 s.
# ⚠️ She currently scores ~92.9 because her ENTRANCE DOES NOT RENDER (see
# dynchar-whitw2-entrance-not-rendered): the framing is static-wide for the whole cinematic and
# no white-out is produced. Scoring her in the default set would swamp the corpus mean with one
# broken skin, so run her explicitly:  ./all8.sh <label> "" whitw2
DIR[whitw2]='char_1038_whitw2_sale#15';   BEATS[whitw2]="2,4,6,8,10,12,13.5"
# TENTH entry / NINTH working reference, added 2026-08-22 — Ch'en the Holungday "Fugue".
# ✅ Unlike whitw2 this one RENDERS correctly and scores sanely (23.870, REFOFF derived
# geometrically above). Captured autonomously from the emulator: Store -> Outfit Store -> Fashion
# Gallery -> Sort by Brand -> EPOQUE -> tile -> magnifier -> reveal chrome -> ▶ Play, recorded at
# native 2340x1080 and converted to the standard 900x416/30. The capture was verified genuine by
# the luma BLACKOUT assertion (min 0.0) and is plainly the cinematic (black fade-in -> distant
# snowy scene -> camera push-in -> close-up at the campfire), not the settled idle.
# ⚠️ HELD OUT of the default key list ON PURPOSE — the documented corpus mean 11.946 is over the
# EIGHT skins below, and silently making it nine would break comparability with every recorded
# figure. Promote it into `keys` only together with a deliberate corpus re-baseline.
# ✅ BEATS TUNED 2026-08-22 to her AUTHORED events, read from the `_Start[scene]` export rather
# than guessed: entranceDuration 9.767 · entranceTransform 8.0 · camera ortho 3.0->1.5 with the
# push-in over t=0-0.23 and essentially still after 1.50 · HGMobileBlur window 1.40->7.00
# (peak 0.70 at 1.40) · sole layer activeUntil switch 2.533. The old generic 2/4/6/8 missed the
# push-in entirely and left the 8.0-9.767 tail unsampled.
#   1.0 post push-in · 1.8 blur on · 2.8 post layer-switch · 4.0/5.5 mid-blur ·
#   6.8 pre blur-end · 7.5 post-blur · 8.2 post-transform
# ⚠️ Deliberately STOPS at 8.2: the end white-out ramps from t~8.3 (luma 84->253 by 9.5) and a
# fade beat measures fade-constant error, not renderer error (cf. exc's t=6, 38.79 and unshippable).
#   Run her explicitly:  ./all8.sh <label> "" fugue
DIR[fugue]='char_113_cqbw_epoque#7';      BEATS[fugue]="1,1.8,2.8,4,5.5,6.8,7.5,8.2"

# kalts — captured 2026-08-23, the TENTH reference and by far the WORST (48.6 vs a corpus of
# 5.5-17.7). Chosen because she is the densest carrier of the features the other references
# cannot see: 33 UV-SCROLL components and a bound `_DisturTex_02`. ⚠️ Her assets were also one
# of the six under-staged exports fixed the same day (42->44 layers, 8->18 textures), so this
# capture is their first real validation.
#
# 🚨 Her reference has TWO WHITE-OUTS the beats must avoid — mean luma peaks 252 at t=9.6 and
# 253 at t=14.1 (clean windows 0-8.7 and 11.1-13.2). Scoring inside one measures the fade
# constant, not the renderer (cf. exc t=6 and fugue's 8.2 cutoff). t=10 alone reads 99.4 and
# t=14 reads 115.9 for that reason and BOTH are excluded.
#
# REFOFF 0.200 is a swept minimum on the clean beats (-0.3 52.04 / -0.1 50.92 / 0 50.54 /
# 0.1 49.52 / [0.2 48.62] / 0.3 49.98). ⚠️ 0.25 scores 47.59 but over SEVEN beats — it pushes
# t=12 past the render range, so it is not comparable; always check the beat count.
#   Run her explicitly:  ./all8.sh <label> "" kalts
DIR[kalts]='char_003_kalts_boc#6';       BEATS[kalts]="1,2,3,4,5,6,8,12"

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
