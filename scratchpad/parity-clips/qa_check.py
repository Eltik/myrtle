#!/usr/bin/env python3
"""Flag corpus renders that are visibly broken, without needing a game reference.

The three benchmark skins are measured against captures; the other seventy-nine are not, so a
defect there is invisible to the score. But several classes of failure are self-evident from the
render alone, and this session has already found two that way (a spurious quad on Rosmontis, a
dead entrance on Kalt'sits). This looks for them:

  BLANK        nothing but the viewer's flat environment fill - the skin draws no character
  DEGENERATE   near-uniform frame (pure white / pure black), e.g. a full-frame layer stuck on
  STATIC       every beat identical - an entrance that never plays, or a frozen scene
  CLIPPED      a large fraction of the drawn content pinned at 0 or 255 (blown or crushed)
  BOXY         strong axis-aligned straight edges spanning much of the frame - the signature of
               a mask-less quad drawn as its bounding rectangle

The environment fill is a flat neutral (~77 luma); `--inner` columns are excluded because the
capture's UI border is not present in our own renders (this reads our output only).
"""
import os
import sys
import numpy as np
from PIL import Image

QA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "qa")
ENV = 77.0
lum = lambda a: 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]


def boxiness(g: np.ndarray) -> float:
    """Fraction of frame width spanned by the longest straight axis-aligned edge.

    A painted silhouette has short, curved gradients; a quad drawn as its bounding box has a
    single long straight one. Measured as the longest run of consecutive columns whose vertical
    gradient peaks in the same row band.
    """
    gy = np.abs(np.diff(g, axis=0))
    strong = gy > 18
    rows = np.argmax(strong, axis=0).astype(float)
    has = strong.any(axis=0)
    rows[~has] = -1
    best = run = 0
    for i in range(1, len(rows)):
        if rows[i] >= 0 and abs(rows[i] - rows[i - 1]) <= 1:
            run += 1
            best = max(best, run)
        else:
            run = 0
    return best / max(1, g.shape[1])


rows = []
for sk in sorted(os.listdir(QA)):
    d = os.path.join(QA, sk)
    if not os.path.isdir(d):
        continue
    fs = sorted((f for f in os.listdir(d) if f.endswith(".png")), key=lambda x: float(x[1:-4]))
    if not fs:
        rows.append((sk, "NO-FRAMES", 0, 0, 0, 0))
        continue
    gs = [lum(np.asarray(Image.open(os.path.join(d, f)).convert("RGB"), float)) for f in fs]
    drawn = [float((np.abs(g - ENV) > 6).mean()) for g in gs]
    stds = [float(g.std()) for g in gs]
    means = [float(g.mean()) for g in gs]
    clipped = max(float(((g < 1) | (g > 254)).mean()) for g in gs)
    box = max(boxiness(g) for g in gs)
    # every beat identical -> nothing animates
    static = all(float(np.abs(gs[i] - gs[0]).mean()) < 0.05 for i in range(1, len(gs)))
    flags = []
    if max(drawn) < 0.05:
        flags.append("BLANK")
    # DEGENERATE only when MOST beats are featureless. A single near-black beat is usually a
    # legitimate blackout in the cinematic, not a broken skin: Skadi `iteration#2` blacks out at
    # t=4 (ours mean 3.2/std 2.4) and SO DOES THE CAPTURE (6.1/6.8), while every other beat of
    # hers tracks the game within ~7 luma. The old `min(stds) < 5` flagged the whole skin on
    # that one frame. A genuinely broken skin is featureless throughout.
    if sum(1 for s_ in stds if s_ < 5) > len(stds) / 2:
        flags.append("DEGENERATE")
    elif min(stds) < 5:
        flags.append("dark-beat")   # informational: verify against the capture before believing
    if static:
        flags.append("STATIC")
    # A pure-black frame (mean 0, 100% clipped) is a RECORDER DROPOUT, not a render defect —
    # `gvial2 summer#12` hit one at t=8 and re-rendered clean at 109.6. Always re-render an
    # outlier before believing it; `mad.py` guards this for the 3 references, this sweep cannot.
    if clipped > 0.35:
        flags.append("CLIPPED(re-render before believing)")
    if box > 0.55:
        flags.append("BOXY")
    rows.append((sk, ",".join(flags), max(drawn), min(stds), clipped, box))

bad = [r for r in rows if r[1]]
print(f"{len(rows)} skins rendered; {len(bad)} flagged\n")
print(f"{'skin':<40} {'flags':<28} {'drawn':>6} {'minStd':>7} {'clip':>6} {'boxy':>6}")
for sk, fl, dr, st, cl, bx in sorted(bad, key=lambda r: -len(r[1])):
    print(f"{sk:<40} {fl:<28} {dr:6.2f} {st:7.1f} {cl:6.2f} {bx:6.2f}")
if not bad:
    print("  (nothing flagged)")
print("\nboxiness leaders (mask-less-quad signature), flagged or not:")
for sk, fl, dr, st, cl, bx in sorted(rows, key=lambda r: -r[5])[:10]:
    print(f"   {bx:5.2f}  {sk}")
