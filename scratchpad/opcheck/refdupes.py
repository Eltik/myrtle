"""Reference-clip QUALITY: how much of each capture is DUPLICATED frames, and which SCORED BEATS
land on one.

    python3 refdupes.py REF_NEW            clip-level survey
    python3 refdupes.py REF_NEW --beats    per-beat contamination audit

WHY THIS EXISTS. Cello's t=10 reads MADC 46.052 at r 0.644 while the beats either side read 5.432
(r 0.978) and 18.355 (r 0.926). It is not an under-draw and it is not a trim error: sweeping OUR
render across 9.467..10.667 against that one game frame puts the best match at t=9.97, which is
exactly the time her trim already assigns, so the two clocks agree. What is wrong is the
REFERENCE. Her clip alternates frames that change ~1600 luma/s with frames that change ~0.1
luma/s, and the frame at t=10.000 is one of the latter: a bit-duplicate of t=9.967, sampled in the
middle of a 33-layer `rootRevealFrom` at 10.300. MADC then charges our render for the capture
failing to advance.

THE HAZARD IS THE PAIR, a duplicate frame AND high local motion. A stall during a static hold
costs nothing at all, which is why `--beats` prints both and flags only the intersection.

⚠️ These are MEASUREMENT defects, not rendering defects. A contaminated beat must not be used as
evidence for or against a render change, and any recorded number leaning on one is suspect.

Thresholds. A frame pair is a DUPLICATE below 0.5 mean |luma delta| (15.0 once expressed per
second); real 30 fps cinematic frames essentially never sit that low. Local motion is the max
per-second delta within +/-3 frames, and 300/s is where a one-frame stall starts to cost more
than the differences we are trying to measure.
"""

import glob
import os
import re
import subprocess
import sys

import numpy as np

# A frame that did not meaningfully advance. Used for the CLIP SURVEY, where the question is how
# much of a capture is stale.
DUP_PER_S = 15.0
# A frame the recorder wrote TWICE: bit-exact, not merely slow. Only these are candidates for the
# content-clock correction, and the distinction is load-bearing. Measured 2026-08-27: whitw2 t=2
# reads 1.2109/s, which 15.0 flagged as a duplicate and which the frame sweep then showed is NOT
# one (her optimum is 3 frames early and her curve is non-monotone, because her t=2 has a separate
# 48.71-luma rendering error). Every genuine drop reads below 0.4/s.
DROP_PER_S = 1.0
MOTION_PER_S = 300.0


def deltas(path):
    """Mean |luma delta| per frame pair, expressed PER SECOND, plus the frame count."""
    p = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0",
         "-show_entries", "stream=width,height", "-of", "csv=p=0", path],
        capture_output=True, text=True,
    ).stdout.strip().split(",")
    w, h = int(p[0]), int(p[1])
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "gray", "-"],
        capture_output=True,
    ).stdout
    n = len(raw) // (w * h)
    if n < 3:
        return None, 0
    a = np.frombuffer(raw, np.uint8)[: n * w * h].reshape(n, h, w).astype(np.float32)
    return np.abs(np.diff(a, axis=0)).mean(axis=(1, 2)) * 30.0, n


def survey(refdir):
    for f in sorted(glob.glob(refdir + "/*_game_fresh.mp4")):
        d, n = deltas(f)
        if d is None:
            continue
        dup = d < DUP_PER_S
        run = best = 0
        for v in dup:
            run = run + 1 if v else 0
            best = max(best, run)
        print(f"  {os.path.basename(f):<34} {n:5d} frames   duplicate {dup.mean() * 100:5.1f}%   "
              f"longest stall {best:2d} ({best / 30:.3f}s)   mean |d|/s {d.mean():7.2f}")


def beats(refdir):
    # Beat sets and trims come from all8.sh rather than being retyped: an earlier run scored eight
    # skins on GUESSED beats and read wis at r 0.237 against her known 0.972 before the mean-r
    # control caught it.
    src = open("all8.sh").read()
    be = dict(re.findall(r"BEATS\[(\w+)\]=\"([^\"]+)\"", src))
    print(f"{'skin':<9}{'beat':>7}{'gframe':>8}  {'drop?':<6}{'|d|/s here':>11}{'|d|/s near':>11}  verdict")
    bad = []
    for k in sorted(be):
        ref = f"{refdir}/{k}_game_fresh.mp4"
        if not os.path.exists(ref):
            continue
        d, n = deltas(ref)
        if d is None:
            continue
        for b in (float(x) for x in be[k].split(",")):
            gi = int(round(b * 30))
            if gi < 1 or gi >= n:
                continue
            dup = d[gi - 1] < DROP_PER_S
            loc = float(np.max(d[max(0, gi - 3): min(len(d), gi + 3)]))
            hit = dup and loc > MOTION_PER_S
            if hit:
                bad.append((k, b, loc))
            print(f"{k:<9}{b:7.1f}{gi:8d}  {'YES' if dup else 'no':<6}{d[gi - 1]:11.4f}{loc:11.1f}  "
                  f"{'CONTAMINATED' if hit else ('bit-exact, but static here' if dup else '')}")
    print(f"\nCONTAMINATED beats: {len(bad)}")
    for k, b, l in bad:
        print(f"   {k} t={b}  local motion {l:.0f}/s")


if __name__ == "__main__":
    d = sys.argv[1] if len(sys.argv) > 1 else "REF_NEW"
    (beats if "--beats" in sys.argv else survey)(d)
