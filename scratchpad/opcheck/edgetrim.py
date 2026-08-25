#!/usr/bin/env python3
"""Derive a reference clip's REFOFF from GEOMETRY rather than from the score.

Render our side at off=0, then for each beat sweep the GAME frame index over a window at
1-frame granularity and take the frame whose EDGE MAP correlates best. Edges respond to
STRUCTURE, not to tone, so this aligns geometry instead of minimising MADC. That matters
because a score sweep can only find the offset that makes our render look least wrong,
which is not the same question as which two frames show the same instant.

Edge map is |dx| + |dy| on luma, z-normalised, over mad.py's `--inner` crop, on the
aspect-corrected basis mad.py uses by default (the reference clips are stretched
vertically by 1.001481 and correcting it is worth ~0.44 MADC corpus-wide).

🚨 SIGN. `score_new.sh` renders `t{beat+off}.png` and compares it to the GAME frame at
`beat`. This sweep answers "our time b matches game time g", so game b matches our
b+(b-g), i.e. REFOFF = b - g. Scoring the negated value reads 36-38 on fugue and would
falsely condemn a good capture.

🚨 NEVER sweep across a FLASH beat. Both Executor clips blow up identically at off 0.200,
exc 8.458 -> 41.367 and excunew 9.716 -> 42.881, because our render whites out at track
5.200 (her `entranceTransform` 5.199999809265137) where the game flashes at clip 5.0333.
A sweep crossing that reads a render artefact as a lag. Same family as the ffmpeg select
trap: verify the tool reproduces a KNOWN value before trusting a new one.

  edgetrim.py <ours_dir> <game.mp4> <beats> [--win=0.6] [--fps=30] [--srcaspect=none]

`ours_dir` holds our frames named `t{beat:.2f}.png`, i.e. rendered with off=0.
"""
import subprocess, sys
import numpy as np
from PIL import Image


def edges(a):
    """|dx| + |dy| on luma, z-normalised. Returns a flat vector."""
    y = 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]
    gx = np.abs(np.diff(y, axis=1))[:-1, :]
    gy = np.abs(np.diff(y, axis=0))[:, :-1]
    e = gx + gy
    s = e.std()
    return (e - e.mean()) / s if s > 1e-6 else None


def main():
    ours_dir, mp4, beats_s = sys.argv[1], sys.argv[2], sys.argv[3]
    fl = sys.argv[4:]
    win = float(next((f.split("=", 1)[1] for f in fl if f.startswith("--win=")), 0.6))
    fps = float(next((f.split("=", 1)[1] for f in fl if f.startswith("--fps=")), 30))
    sa = next((f.split("=", 1)[1] for f in fl if f.startswith("--srcaspect=")), "2340:1080")
    sa = None if sa.lower() in ("none", "off", "0") else sa
    beats = [float(b) for b in beats_s.split(",")]

    pr = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
                         "stream=width,height", "-of", "csv=p=0", mp4],
                        capture_output=True, text=True).stdout.strip().split(",")
    GW, GH = int(pr[0]), int(pr[1])
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", mp4, "-f", "rawvideo",
                          "-pix_fmt", "rgb24", "-"], capture_output=True).stdout
    n = len(raw) // (GW * GH * 3)
    G = np.frombuffer(raw, np.uint8)[: n * GW * GH * 3].reshape(n, GH, GW, 3)

    picks = []
    print(f"[edge] game {GW}x{GH} {n} frames | window +/-{win:.3f}s at 1/{fps:.0f} | "
          f"basis {'aspect-corrected ' + sa if sa else 'RAW'}")
    for b in beats:
        o_img = Image.open(f"{ours_dir}/t{b:.2f}.png").convert("RGB")
        o = np.asarray(o_img).astype(np.float32)
        if sa:
            sw, sh = (float(v) for v in sa.split(":"))
            H0, W0 = o.shape[:2]
            k = (W0 / sw) / (H0 / sh)
            xs = np.clip((np.arange(W0) - W0 / 2.0) / k + W0 / 2.0, 0, W0 - 1.001)
            x0 = xs.astype(int)
            fx = (xs - x0)[None, :, None]
            o = o[:, x0] * (1 - fx) + o[:, x0 + 1] * fx
        eo = edges(o[:, 36:864])
        if eo is None:
            print(f"  t={b:<7} FLAT render, no structure to correlate -- beat EXCLUDED")
            continue
        row = []
        for k2 in range(int(round((b - win) * fps)), int(round((b + win) * fps)) + 1):
            if k2 < 0 or k2 >= n:
                continue
            g = np.asarray(Image.fromarray(G[k2]).resize(o_img.size, Image.LANCZOS)).astype(np.float32)
            eg = edges(g[:, 36:864])
            if eg is None:
                continue
            row.append((k2 / fps, float((eo * eg).mean())))
        if not row:
            print(f"  t={b:<7} no usable game frames in window")
            continue
        row.sort(key=lambda r: -r[1])
        gbest, ncc = row[0]
        lo = min(r[0] for r in row)
        hi = max(r[0] for r in row)
        interior = lo + 1e-9 < gbest < hi - 1e-9
        picks.append((b, b - gbest))
        top = "  ".join(f"{t:.4f}:{v:.3f}" for t, v in row[:5])
        print(f"  t={b:<7} best game {gbest:8.4f}  implied {b - gbest:+.4f}  edgeNCC {ncc:.3f}  "
              f"{'interior' if interior else '⚠️ AT WINDOW EDGE'}   top5 {top}")
    if picks:
        bt = np.array([b for b, _ in picks])
        p = np.array([v for _, v in picks])
        step = 1.0 / fps
        print(f"\n[edge] per-beat implied: {', '.join(f'{v:+.4f}' for v in p)}")
        print(f"[edge] MEDIAN {np.median(p):+.4f}   MEAN {p.mean():+.4f}   SPREAD {p.max() - p.min():.4f}"
              f"   frames {np.median(p) * fps:+.2f}")
        print(f"[edge] => REFOFF = {np.median(p):+.4f}   (sign: REFOFF = beat - best_game_time)")
        # RATE CHECK. A trim is a CONSTANT; a rate error is LINEAR in t. Eyeballing "the picks
        # ramp" cannot tell them apart, because the picks are quantised to one frame and a
        # monotone 2,2,3,4 fits a line trivially. Fit one and report whether it survives.
        #
        # 🚨 The bar is `|slope| * lever > one frame` AND a CI that excludes zero. Executor's
        # four beats over a 3.00 s lever read +0.023310 s/s at R2 0.891 with the CI excluding
        # zero, which looked like a 2.331% rate error. Re-sampled at NINE beats over 4.00 s it
        # collapsed to +0.011123 at R2 0.093 with the CI including zero, and dropping the single
        # weakest pick (edgeNCC 0.270, against 0.36-0.71 elsewhere) left +0.002112 at R2 0.019.
        # It was two frames of quantised movement on the shortest baseline in the corpus.
        # ⛔ Do not read a slope off a short lever. wis is the control: 0.000000 over 10.00 s.
        if len(p) > 2 and bt.max() > bt.min():
            A = np.vstack([bt, np.ones(len(bt))]).T
            (m, c), _, _, _ = np.linalg.lstsq(A, p, rcond=None)
            resid = p - A @ np.array([m, c])
            ss_t = ((p - p.mean()) ** 2).sum()
            ss_r = (resid ** 2).sum()
            r2 = 1 - ss_r / ss_t if ss_t > 1e-12 else float("nan")
            se = np.sqrt(ss_r / (len(p) - 2) / ((bt - bt.mean()) ** 2).sum()) if ss_r > 0 else 0.0
            ci = 1.96 * se
            lever = bt.max() - bt.min()
            excl = (m - ci) * (m + ci) > 0
            print(f"[rate] slope {m:+.6f} s/s ({100 * m:+.3f}%)  R2 {r2:.3f}  resid RMS "
                  f"{np.sqrt((resid ** 2).mean()) / step:.2f} frames  lever {lever:.2f}s")
            rms_fr = np.sqrt((resid ** 2).mean()) / step
            move_fr = abs(m) * lever / step
            ciy = "degenerate, residuals exactly zero" if ss_r <= 0 else ("EXCLUDES zero" if excl else "includes zero")
            print(f"[rate] 95% CI {m - ci:+.6f}..{m + ci:+.6f} ({ciy})"
                  f"   |slope|*lever = {move_fr:.2f} frames")
            # Four conditions, and ALL must hold. Each one has a specific failure behind it.
            #   n >= 6      : four quantised monotone picks fit a line trivially. Fugue's
            #                 2,2,3,3 over 6.00s reads +0.667% at R2 0.800 with the CI excluding
            #                 zero, and she is not drifting.
            #   move >= 2fr : one frame of movement is the measurement floor, not a signal.
            #   CI excl 0   : and ss_r > 0, because a PERFECT fit gives se = 0 and a collapsed
            #                 CI that "excludes zero" while proving flatness. wis is exactly that.
            #   rms < 1fr   : a slope drawn through scattered picks is describing the scatter.
            ok = len(p) >= 6 and move_fr >= 2.0 and excl and ss_r > 0 and rms_fr < 1.0
            fails = []
            if len(p) < 6:
                fails.append(f"n={len(p)}<6")
            if move_fr < 2.0:
                fails.append(f"move {move_fr:.2f}fr<2")
            if ss_r <= 0:
                fails.append("perfect fit, se=0")
            elif not excl:
                fails.append("CI spans 0")
            if rms_fr >= 1.0:
                fails.append(f"resid {rms_fr:.2f}fr>=1")
            print(f"[rate] => {'REAL RATE ERROR' if ok else 'FLAT (consistent with a constant trim)'}"
                  f"{'' if ok else '   [' + ', '.join(fails) + ']'}")


main()
