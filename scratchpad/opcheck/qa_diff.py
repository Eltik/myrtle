#!/usr/bin/env python3
"""Diff two corpus QA renders and flag anything that looks broken rather than merely different.

74 of the 82 deployed skins have no reference capture, so "closer to the game" is unanswerable
for them. What a corpus render CAN answer is whether a change destroys a skin, and the failure
modes worth flagging are specific:

  BLACK      the frame renders empty (all zero) where it did not before
  BLOWOUT    a large jump in the fraction of the frame at/near pure white
  VANISHED   the drawn content collapses (mean level falls toward the bare environment fill)
  LARGE      a big mean shift, which is not necessarily wrong but must be eyeballed
  DROPOUT    a frame present in one render and missing in the other

Usage: qa_diff.py <before-dir> <after-dir> [--top N]
"""
import os
import sys

import numpy as np
from PIL import Image

ENV_FILL = 77.0  # the viewer's flat #4d4d4e background


def luma(a: np.ndarray) -> np.ndarray:
    return 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]


def main() -> None:
    before, after = sys.argv[1], sys.argv[2]
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 25

    rows, flagged, missing = [], [], []
    skins = sorted(set(os.listdir(before)) | set(os.listdir(after)))
    for sk in skins:
        bd, ad = os.path.join(before, sk), os.path.join(after, sk)
        if not os.path.isdir(bd) or not os.path.isdir(ad):
            missing.append(f"{sk} (dir missing on one side)")
            continue
        bf, af = set(os.listdir(bd)), set(os.listdir(ad))
        for f in sorted(bf ^ af):
            missing.append(f"{sk}/{f}")
        for f in sorted(bf & af):
            if not f.endswith(".png"):
                continue
            a = np.array(Image.open(os.path.join(bd, f)).convert("RGB")).astype(float)
            b = np.array(Image.open(os.path.join(ad, f)).convert("RGB")).astype(float)
            la, lb = luma(a), luma(b)
            d = (b - a).mean()
            wa, wb = (la > 250).mean() * 100, (lb > 250).mean() * 100
            # "content" = how far the frame sits from a bare environment fill.
            ca, cb = np.abs(la - ENV_FILL).mean(), np.abs(lb - ENV_FILL).mean()
            flag = ""
            if lb.max() < 1 and la.max() >= 1:
                flag = "BLACK"
            elif wb - wa > 3:
                flag = "BLOWOUT"
            elif ca > 3 and cb < ca * 0.5:
                flag = "VANISHED"
            elif abs(d) > 6:
                flag = "LARGE"
            rows.append((abs(d), sk, f, d, wa, wb, ca, cb, flag))
            if flag:
                flagged.append((sk, f, flag, d, wa, wb, ca, cb))

    rows.sort(reverse=True)
    print(f"compared {len(rows)} frames across {len(skins)} skins\n")
    print(f"{'skin':40s} {'beat':>6} {'d mean':>8} {'blown% b->a':>14} {'content b->a':>16}  flag")
    for _, sk, f, d, wa, wb, ca, cb, flag in rows[:top]:
        print(f"{sk:40s} {f[1:-4]:>6} {d:+8.2f} {wa:6.2f}->{wb:<6.2f} {ca:7.1f}->{cb:<7.1f}  {flag}")
    print(f"\nmissing/dropout: {missing or 'none'}")
    print(f"FLAGGED: {len(flagged)}")
    for sk, f, flag, d, wa, wb, ca, cb in flagged:
        print(f"  {flag:9s} {sk:40s} {f[1:-4]:>6} d={d:+.2f} blown {wa:.2f}->{wb:.2f} content {ca:.1f}->{cb:.1f}")


if __name__ == "__main__":
    main()
