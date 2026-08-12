#!/usr/bin/env python3
"""Blow-out analysis for the simSpeed/prewarm corpus sweep.

There is no game reference for 74 of the 77 affected skins, so this deliberately does NOT
score parity. It asks what a corpus render can actually answer: does enabling the fields
DESTROY a skin (content vanishes) or BLOW ONE OUT (saturates / floods the frame)?

Per skin, per beat, against the shipped render:
  dMean   signed change in frame mean luma        (sign = brighter / darker)
  frac    fraction of pixels moving > 8 luma      (how much of the frame is involved)
  dSat    change in fraction of pixels >= 250     (blow-out detector)
  dDark   change in fraction of pixels <= 8       (content-loss detector)
"""
import os, sys, json
import numpy as np
from PIL import Image

SWEEP = os.path.join(os.path.dirname(os.path.abspath(__file__)), "gsweep")


def lum(a):
    return 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]


def main():
    skins = sorted({d.rsplit("__", 1)[0] for d in os.listdir(SWEEP) if "__" in d})
    rows, missing = [], []
    for sk in skins:
        doff, don = f"{SWEEP}/{sk}__off", f"{SWEEP}/{sk}__on"
        if not (os.path.isdir(doff) and os.path.isdir(don)):
            missing.append(sk)
            continue
        frames = sorted(set(os.listdir(doff)) & set(os.listdir(don)))
        frames = [f for f in frames if f.endswith(".png")]
        if not frames:
            missing.append(sk)
            continue
        worst = None
        for f in frames:
            o = lum(np.asarray(Image.open(f"{doff}/{f}").convert("RGB"), float))
            n = lum(np.asarray(Image.open(f"{don}/{f}").convert("RGB"), float))
            if o.shape != n.shape:
                continue
            # A dropped-frame recorder artifact reads as a total wipe; exclude it.
            if o.mean() < 1 or n.mean() < 1:
                continue
            d = n - o
            rec = dict(
                skin=sk, beat=f[1:-4],
                dMean=float(n.mean() - o.mean()),
                frac=float((np.abs(d) > 8).mean()),
                dSat=float((n >= 250).mean() - (o >= 250).mean()),
                dDark=float((n <= 8).mean() - (o <= 8).mean()),
            )
            # rank a skin by its most disturbed beat
            if worst is None or abs(rec["dMean"]) > abs(worst["dMean"]):
                worst = rec
        if worst:
            rows.append(worst)
        else:
            missing.append(sk)

    rows.sort(key=lambda r: -abs(r["dMean"]))
    print(f"{len(rows)} skins compared, {len(missing)} unusable\n")
    print(f"{'skin':44} {'beat':>5} {'dMean':>8} {'frac':>7} {'dSat':>8} {'dDark':>8}")
    for r in rows:
        print(f"{r['skin']:44} {r['beat']:>5} {r['dMean']:+8.2f} {r['frac']:7.3f} "
              f"{r['dSat']:+8.4f} {r['dDark']:+8.4f}")
    if missing:
        print(f"\nunusable ({len(missing)}): " + ", ".join(missing))

    # Blow-out gate: what a reviewer actually needs to look at.
    bad = [r for r in rows if abs(r["dMean"]) > 4 or r["dSat"] > 0.02 or r["dDark"] > 0.02]
    print(f"\n=== FLAGGED FOR EYEBALLING: {len(bad)} ===")
    for r in bad:
        print(f"  {r['skin']:44} beat {r['beat']:>5}  dMean {r['dMean']:+.2f}  "
              f"dSat {r['dSat']:+.4f}  dDark {r['dDark']:+.4f}")
    json.dump(rows, open("/tmp/sweep_rows.json", "w"))


if __name__ == "__main__":
    main()
