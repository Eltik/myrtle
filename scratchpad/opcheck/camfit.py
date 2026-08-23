#!/usr/bin/env python3
"""Where is our frame, relative to the game's? Feature-matched, not template-matched.

  camfit.py <game.mp4> <ours_dir> <beats> <refoff> [px_per_authored]

Estimates a similarity transform (scale, rotation, translation) between our render and the
capture with SIFT + RANSAC, then converts the translation into AUTHORED px so it can be compared
directly against the exported `entranceCamCenterCurve`.

WHY NOT `geomsearch.py`: template matching needs the game's patch to actually be inside our
frame. When the camera is badly misplaced the two frames barely overlap — chyue's overlap is
13.7% — and every scale search then locks onto noise (hers pinned at the search edge and reported
a bogus 3.1-3.8x "zoom error"). Feature matching does not care about overlap fraction: it found
her true transform in one shot, scale 0.9973, 294/308 inliers.

CALIBRATION. `px_per_authored` (default 0.5616) converts render px to authored px. Measure it per
skin rather than trusting the default: render twice with `EXTRA=camdy=0` and `EXTRA=camdy=100`
and run this script between them — the reported translation is 100 authored px. Cross-check that
`2*orthoCurve[0]/skeletonScale * px_per_authored` equals the viewport height.

READ THE CONTROL FIRST. On a well-aligned skin this must return ~0: wisdel measures (-0.8, -0.4)
authored px with 1600+ inliers. A non-zero result is only meaningful against that.
"""
import subprocess, sys
import numpy as np, cv2
from PIL import Image

if len(sys.argv) < 5:
    print(__doc__)
    raise SystemExit(2)
GAME, OURS, BEATS, OFF = sys.argv[1], sys.argv[2], sys.argv[3], float(sys.argv[4])
K = float(sys.argv[5]) if len(sys.argv) > 5 else 0.5616

pr = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
                     "stream=width,height", "-of", "csv=p=0", GAME],
                    capture_output=True, text=True).stdout.strip().split(",")
GW, GH = int(pr[0]), int(pr[1])
raw = subprocess.run(["ffmpeg", "-v", "error", "-i", GAME, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
                     capture_output=True).stdout
n = len(raw) // (GW * GH * 3)
F = np.frombuffer(raw, np.uint8)[:n * GW * GH * 3].reshape(n, GH, GW, 3)

sift = cv2.SIFT_create(nfeatures=6000)
bf = cv2.BFMatcher()
print(f"{'beat':>7} {'inliers':>11} {'scale':>7} {'rot':>7} {'render dT':>20} {'authored offset':>22}")
for b in [float(x) for x in BEATS.split(",")]:
    gi = int(round(b * 30))
    if gi >= n:
        continue
    try:
        o = cv2.cvtColor(np.asarray(Image.open(f"{OURS}/t{b + OFF:.2f}.png").convert("RGB")), cv2.COLOR_RGB2GRAY)
    except FileNotFoundError:
        continue
    g = cv2.cvtColor(np.asarray(Image.fromarray(F[gi]).resize((o.shape[1], o.shape[0]), Image.LANCZOS)),
                     cv2.COLOR_RGB2GRAY)
    k1, d1 = sift.detectAndCompute(o, None)
    k2, d2 = sift.detectAndCompute(g, None)
    if d1 is None or d2 is None:
        print(f"{b:>7}  no features"); continue
    good = [m for m, nn in bf.knnMatch(d1, d2, k=2) if m.distance < 0.75 * nn.distance]
    if len(good) < 12:
        print(f"{b:>7}  too few matches ({len(good)})"); continue
    src = np.float32([k1[m.queryIdx].pt for m in good]).reshape(-1, 1, 2)
    dst = np.float32([k2[m.trainIdx].pt for m in good]).reshape(-1, 1, 2)
    M, inl = cv2.estimateAffinePartial2D(src, dst, method=cv2.RANSAC, ransacReprojThreshold=3.0,
                                         maxIters=20000, confidence=0.999)
    if M is None:
        print(f"{b:>7}  RANSAC failed"); continue
    s = float(np.hypot(M[0, 0], M[1, 0]))
    rot = float(np.degrees(np.arctan2(M[1, 0], M[0, 0])))
    print(f"{b:>7} {int(inl.sum()):>5}/{len(good):<5} {s:>7.4f} {rot:>+7.2f} "
          f"({M[0, 2]:+7.1f},{M[1, 2]:+7.1f}) ({-M[0, 2] / K:+10.1f},{-M[1, 2] / K:+10.1f})")
