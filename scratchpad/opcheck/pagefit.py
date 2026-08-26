#!/usr/bin/env python3
"""Discriminate the fit rule the GAME'S WINDOWED PAGE uses, by arithmetic, from three points.

THE QUESTION. Our detail card contain-fits the whole illustration with margins on all four
sides. The game's windowed page does not: it fills the column height, bleeds top, bottom and
LEFT, and leaves a gap only on the RIGHT. If the card's target is the page rather than our own
contain fit, the page's rule has to be derived before anything can be implemented against it.

THE DATA. Three right-gap fractions measured off archive-page captures, recorded in
DYNCHAR_PARITY_REFERENCE.md: wisdel 10.9-11.8%, Kroos 14.9%, excu2 17.6%. The column is
1417x1078 (aspect 1.31447), with 906 px of UI beside it on a 2340x1080 screen.

⚠️ THE THREE NUMBERS ARE NOT RE-DERIVABLE. The clips they came from are gone from disk, only
the summary rows survive, and the doc does not say whether "fraction of window width" means the
1417 column or the 2340 screen. Both denominators are evaluated below and neither rescues any
candidate, so the conclusion does not rest on that ambiguity.

🔑 THE STRUCTURAL RESULT, which is stronger than any single row. The measured gap GROWS with the
content's width-to-height ratio: wisdel 0.9683 -> 11.35%, Kroos 1.0476 -> 14.9%, excu2 1.2029 ->
17.6%. Every height fit on the painted content makes it SHRINK, because wider content under a
fixed vertical scale reaches further right and leaves less room. The whole height-fit-on-content
family is refuted by SIGN, not by magnitude, and no amount of re-anchoring fixes a sign.

Anchors are converted authored px -> render px by RCAL, the same 0.781 `bodyFrameBox` applies to
`viewPx`. `cameraOffsetPx` and `cameraOffsetPx2` are small enough here that the conversion moves
nothing, but doing it wrong would have been a real error rather than a rounding one.

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


def main():
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
