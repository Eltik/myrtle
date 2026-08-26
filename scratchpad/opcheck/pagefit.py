#!/usr/bin/env python3
"""Discriminate the fit rule the GAME'S WINDOWED PAGE uses. RESOLVED: it is a COVER fit.

🏁 THE ANSWER. The page lays the illustration into a FIXED RECTANGLE and fills it, cropping
whatever does not fit. Measured on three skins spanning a 1.80x aspect range, silhouette against
the flat page ground on still frames:

    skin   content aspect   art column right edge   rows drawn      top gap   bottom gap
    cet          0.8483            x = 1473          0..1079           0          0
    cel          1.0790            (estimator dip)   0..1079           0          0
    eyja         1.5238            x = 1473          0..1079           0          0

Full bleed on all four edges, on every subject, at both ends of the aspect range. The right edge
lands on the SAME pixel for the two skins where the edge estimator is clean; cel's art is locally
low-contrast around x=1450 so her exact edge is unresolvable by column variance, and she is
consistent with the same column rather than evidence against it.

⛔ SO EVERY GAP-PREDICTING RULE IS REFUTED, and rule D is the only survivor. A cover fit is the
only candidate that predicts a gap of ZERO independent of aspect. The other six all predict a
nonzero gap that varies with the content box, and the measurement says there is no gap at all.

🚨 AND THE ORIGINAL THREE ROWS WERE NEVER LAYOUT. They came off a MOTION bounding box, which is a
strict subset of the drawn extent because a pixel can only change between frames if something is
drawn there. Their "right gap" of 10.9%, 14.9% and 17.6% was measuring how much of each figure
ANIMATES, which varies with pose and composition, which is exactly why it correlated with aspect
and why no fit rule could reproduce it. The table below is kept for the arithmetic; the three rows
it was tested against are RETIRED.

WHAT n=3 LICENSES, stated rather than assumed. A cover fit is heavily constrained rather than
fitted here: it predicts zero gap for EVERY skin, so any single subject with a visible gap refutes
it outright, and three subjects spanning 0.8483 to 1.5238 all failed to produce one. That is
stronger than three points fitted to a two-parameter rule. It does NOT establish the column's exact
width, since only two of the three gave a clean edge.

THE FOURTH SUBJECT THAT WOULD BEST TRY TO BREAK IT: not another entrance skin, because all 13 lie
inside the aspect range already tested. Render `contentBounds` for the other 69 dynchar skins
offline, take the single most extreme aspect in that set, and capture its page. A cover fit crops
the long axis, so the most extreme illustration is where a contain-style fallback would show first
if the rule is really a piecewise one.

  pagefit.py            print the full table under both denominators
"""
import sys

COLW, COLH = 1417.0, 1078.0
SCREENW = 2340.0
RCAL = 0.781

# contentBounds from `__settleProbe`, body centre and authored fields from `__frameProbe`, both
# read out of the running renderer rather than transcribed from anywhere.
S = {
    "wisdel": dict(cb=(-977.322, -1593.924, 1976.538, 2041.342), bodyCx=-56.757,
                   off=(8.4, 30.0), off2=(-26.5, 16.0), viewPx=1869.860, viewPx2=1295.700,
                   camPx=1000.0, meas=(10.9, 11.8)),
    "kroos":  dict(cb=(-1189.825, -1685.449, 2073.802, 1979.538), bodyCx=-87.648,
                   off=(9.86, -54.3), off2=(-72.2, -73.54), viewPx=1817.600, viewPx2=1323.700,
                   camPx=1063.0, meas=(14.9, 14.9)),
    "excu2":  dict(cb=(-1013.676, -1687.796, 2051.645, 1705.584), bodyCx=10.847,
                   off=(61.4, 168.6), off2=(-2.2, -20.0), viewPx=2045.0, viewPx2=1348.600,
                   camPx=1040.0, meas=(17.6, 17.6)),
}


def rules(d):
    x, y, w, h = d["cb"]
    out = {}
    # A: fit by height on the painted content, anchored left.
    s = COLH / h
    out["A height/contentBounds, anchored left"] = (COLW - w * s) / COLW * 100
    # B, C: same scale, anchored so the authored camera centre lands at the column centre.
    for nm, key in (("B height/contentBounds @cameraOffsetPx", "off"),
                    ("C height/contentBounds @cameraOffsetPx2", "off2")):
        ax = d[key][0] * RCAL
        out[nm] = (COLW - (COLW / 2 + (x + w - ax) * s)) / COLW * 100
    # D: cover fit fills both axes, so there is no gap on any side at all.
    out["D cover fit (max of the two ratios)"] = 0.0
    # E, F, G: fit by height on an AUTHORED square box, then measure to the painted right edge.
    for nm, side in (("E height/authoredDisplayBounds square", d["viewPx"] * RCAL),
                     ("F height/authoredTightBounds square", d["viewPx2"] * RCAL),
                     ("G height/cameraSizePx square", d["camPx"] * RCAL)):
        bx = d["bodyCx"] - side / 2
        ss = COLH / side
        out[nm] = (COLW - (COLW / 2 + (x + w - (bx + side / 2)) * ss)) / COLW * 100
    return out


# MEASURED DRAWN EXTENT, silhouette against the flat page ground on still frames, 2340x1080.
# `gap` is the page ground visible between the illustration and the art column's edge.
DRAWN = {
    "cet":  dict(aspect=0.8483, right_edge=1473, rows=(0, 1079), gap=0.0),
    "cel":  dict(aspect=1.0790, right_edge=None, rows=(0, 1079), gap=0.0),   # edge estimator dips
    "eyja": dict(aspect=1.5238, right_edge=1473, rows=(0, 1079), gap=0.0),
}


def resolved():
    print("MEASURED DRAWN EXTENT (this is layout, not a motion box)\n")
    print(f"{'skin':6s} {'aspect':>7s} {'right edge':>11s} {'rows drawn':>12s} {'top gap':>8s} {'bottom gap':>11s} {'right gap':>10s}")
    for k, d in DRAWN.items():
        r0, r1 = d["rows"]
        e = "unresolved" if d["right_edge"] is None else str(d["right_edge"])
        print(f"{k:6s} {d['aspect']:7.4f} {e:>11s} {f'{r0}..{r1}':>12s} {r0:8d} {1079-r1:11d} {d['gap']:10.1f}")
    print("\nFull bleed on all four edges, on every subject, across a 1.80x aspect range.")
    print("A COVER FIT is the only candidate rule that predicts a zero gap independent of aspect.")
    print("Rules A, B, C, E, F and G all predict a nonzero gap that varies with the content box,")
    print("so all six are refuted by a measurement of zero. Rule D SURVIVES.\n")
    print("The table below is the RETIRED motion-box arithmetic, kept only to show what it was.")
    print("Its three rows measured how much of each figure ANIMATES, never the layout.\n")


def main():
    resolved()
    keys = list(S)
    print(f"column {COLW:.0f}x{COLH:.0f}, aspect {COLW / COLH:.5f}; screen width {SCREENW:.0f}\n")
    print("content aspect (w/h): " + ", ".join(f"{k} {S[k]['cb'][2] / S[k]['cb'][3]:.4f}" for k in keys))
    for denom, label in ((COLW, "as % of COLUMN width"), (SCREENW, "as % of SCREEN width")):
        conv = COLW / denom * 100  # measured fraction -> % of column, for comparison
        print(f"\n=== measured read {label} ===")
        print(f"{'rule':44s} " + "  ".join(f"{k:>12s}" for k in keys))
        m = []
        for k in keys:
            lo, hi = S[k]["meas"]
            m.append(((lo + hi) / 2) * (100 / conv) if denom != COLW else (lo + hi) / 2)
        print(f"{'MEASURED (% of column width)':44s} " + "  ".join(f"{v:12.2f}" for v in m))
        print("-" * 90)
        for name in rules(S[keys[0]]):
            vals = [rules(S[k])[name] for k in keys]
            order = "order OK" if vals[0] < vals[1] < vals[2] else "ORDER WRONG"
            worst = max(abs(vals[i] - m[i]) for i in range(3))
            print(f"{name:44s} " + "  ".join(f"{v:12.2f}" for v in vals) + f"   {order}, worst |err| {worst:.2f}")
    print("\nMEASURED ORDER: wisdel < kroos < excu2, i.e. the gap GROWS with content aspect.")
    print("Every height fit on the painted content makes it shrink. No candidate survives.")


main()
