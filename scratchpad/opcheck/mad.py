"""Mean absolute Y'CbCr error per pixel, ours vs the in-game capture.

REBUILT 2026-08-01 after the original was lost with the job tmp directory. Faithful to the
documented metric so historical numbers stay comparable — BUT only when pointed at the
ORIGINAL captures. See the salvage note at the bottom.

Usage: python3 mad.py <ours_dir> <game_mp4> <t1,t2,...> [--inner] [--offset=S] [--dy=N]

METRIC
------
    MADC = MAD_Y + W_C * (MAD_Cb + MAD_Cr) / 2      with W_C = 1.0

All three terms are mean absolute differences in 8-bit code values over the same stride-2
pixel subsample, so they are directly commensurable.

* MAD_Y is BT.601 luma on the gamma-encoded sRGB values, Y' = 0.299R + 0.587G + 0.114B,
  integer-rounded — bit-for-bit the legacy luma-only metric, printed on its own line so every
  historical number stays comparable.
* Cb = 128 + 0.564*(B - Y'), Cr = 128 + 0.713*(R - Y') — full-range BT.601, the same colour
  space the luma term already lived in. Chroma is computed in float.

W_C = 1.0 because the old luma-only metric was effectively W_C = 0: blue carries weight 0.114
in Y', so a +22 code-value error confined to blue moves Y' by only 2.5 while being plainly
visible as a hue shift.

`--inner` restricts BOTH frames to columns 36..863. All three captures carry a STATIC UI
border outside that window which is not part of the render and would otherwise dominate.

`--offset=S` is the per-recording reference-clock offset (Mlynar +0.051, Virtuosa +0.0171,
Skadi 0). OUR frame for beat b is read from `t{b+S:.2f}.png`; the GAME frame is index
round(b*FPS). The offset is derived from GEOMETRY, never from this score.

`--fps=N` is the REFERENCE clip's frame rate, default 15 (the historical salvage format). The
fresh 2340x1080 entrance captures are rebuilt at 30 so the alignment grid is 0.033 s instead of
0.067 s — MADC moves several points per frame at 15 fps, so the grid was a real limit on how
finely t0 could be pinned. Getting this wrong does not error, it silently reads the WRONG frame
(at 30 fps with the default 15 every beat lands at half its intended time).
"""

import subprocess
import sys

import numpy as np
from PIL import Image


def ycc(a):
    y = 0.299 * a[..., 0] + 0.587 * a[..., 1] + 0.114 * a[..., 2]
    return y, 128 + 0.564 * (a[..., 2] - y), 128 + 0.713 * (a[..., 0] - y)


def main():
    ours_dir, game_mp4, times = sys.argv[1], sys.argv[2], sys.argv[3]
    flags = sys.argv[4:]
    inner = "--inner" in flags
    off = next((float(f.split("=", 1)[1]) for f in flags if f.startswith("--offset=")), 0.0)
    # `--dy=N` corrects a VERTICAL misalignment between our render and the reference by
    # comparing our row i against the reference's row i+N (cropping instead of wrapping).
    #
    # The salvaged REF clips need N = 2. They were cut from the bottom half of the old
    # side-by-side videos and that cut is two rows off, so every score has been computed against
    # a reference shifted 2px down. It is a HARNESS artifact, not a render error: the optimum is
    # dy=+2 on ALL THREE skins (ska 13.72->10.61, cel 23.23->19.97, mly 30.81->30.23), which a
    # per-skin framing bug could not produce. Left uncorrected it inflates every score by ~1-3
    # MADC and systematically penalises any change that moves or sharpens content.
    dy = int(next((float(f.split("=", 1)[1]) for f in flags if f.startswith("--dy=")), 0))
    fps = float(next((f.split("=", 1)[1] for f in flags if f.startswith("--fps=")), 15))
    # `--srcaspect=W:H` corrects the reference clips' GEOMETRIC DISTORTION.
    #
    # `capture_oracle.sh` encodes with `scale=900:416`, which forces BOTH dimensions from the
    # device's 2340x1080 frame. That is a NON-UNIFORM scale: x shrinks by 900/2340 = 0.384615
    # and y by 416/1080 = 0.385185, so every reference frame is stretched vertically by
    # 1.001481 relative to what the game actually showed. Our render has square pixels, so the
    # two are compared in different geometries and we are penalised for it.
    #
    # Measured with an anisotropic (sx, sy) registration fit: sy lands on 1.0000 at nearly every
    # beat on both Mlynar and Skadi while sx lands on 0.9985 — i.e. the error is purely
    # horizontal and exactly the predicted 1/1.001481 = 0.998519. Correcting it is worth
    # MAD_Y -0.442 corpus-wide (wis -1.119, mly -0.865, cel -0.461, ska -0.311, cet -0.218,
    # mue -0.151, eyja +0.031).
    #
    # DEFAULT OFF. Every recorded number predates this and a silent basis change would make them
    # all incomparable — the same reason the render density is pinned in `score_new.sh`.
    srcaspect = next((f.split("=", 1)[1] for f in flags if f.startswith("--srcaspect=")), None)
    beats = [float(t) for t in times.split(",")]

    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", game_mp4],
        capture_output=True, text=True,
    ).stdout.strip().split(",")
    GW, GH = int(probe[0]), int(probe[1])
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", game_mp4, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
        capture_output=True,
    ).stdout
    n = len(raw) // (GW * GH * 3)
    G = np.frombuffer(raw, np.uint8)[: n * GW * GH * 3].reshape(n, GH, GW, 3)

    ys, cs, ms = [], [], []
    for b in beats:
        o_img = Image.open(f"{ours_dir}/t{b + off:.2f}.png").convert("RGB")
        gi = int(round(b * fps))
        if gi >= n:
            print(f"  t={b:<6} SKIPPED (game frame {gi} beyond {n})")
            continue
        g_img = Image.fromarray(G[gi])
        if g_img.size != o_img.size:
            g_img = g_img.resize(o_img.size, Image.LANCZOS)
        o = np.asarray(o_img).astype(np.float32)
        g = np.asarray(g_img).astype(np.float32)
        if srcaspect:
            sw, sh = (float(v) for v in srcaspect.split(":"))
            H0, W0 = o.shape[:2]
            # Put OUR frame into the reference's (distorted) geometry: the reference squashed x
            # by W0/sw and y by H0/sh, so the residual horizontal factor is their ratio.
            k = (W0 / sw) / (H0 / sh)
            xs = np.clip((np.arange(W0) - W0 / 2.0) / k + W0 / 2.0, 0, W0 - 1.001)
            x0 = xs.astype(int)
            fx = (xs - x0)[None, :, None]
            o = o[:, x0] * (1 - fx) + o[:, x0 + 1] * fx
        # RECORDER DROPOUT GUARD. The headless capture occasionally writes a pure BLACK
        # frame; scored normally it contributes ~90 MADC and silently inflates the mean
        # (one dropout at Skadi's t=7 read 24.484 against her true 10.530, with every
        # other frame bit-identical). A real render is never this dark, so treat it as a
        # failed capture: shout, and drop the beat rather than the whole run.
        if o.mean() < 1.0:
            print(f"  t={b:<6} DROPPED FRAME (mean {o.mean():.3f}) -- capture failed, beat EXCLUDED; re-run")
            continue
        if dy > 0:
            o, g = o[:-dy], g[dy:]
        elif dy < 0:
            o, g = o[-dy:], g[:dy]
        if inner:
            o, g = o[:, 36:864], g[:, 36:864]
        o, g = o[::2, ::2], g[::2, ::2]
        yo, cbo, cro = ycc(o)
        yg, cbg, crg = ycc(g)
        mad_y = np.abs(np.round(yo) - np.round(yg)).mean()
        mad_c = (np.abs(cbo - cbg).mean() + np.abs(cro - crg).mean()) / 2
        ys.append(mad_y)
        cs.append(mad_c)
        ms.append(mad_y + mad_c)
        print(f"  t={b:<6} MADC={mad_y + mad_c:7.3f}  (Y={mad_y:7.3f}  C={mad_c:6.3f})")
    if ms:
        print(f"MEAN MAD(luma-only, legacy) = {np.mean(ys):.3f}  over {len(ms)} beats")
        print(f"MEAN CHROMA = {np.mean(cs):.3f}  over {len(ms)} beats")
        print(f"MEAN MADC = {np.mean(ms):.3f}  over {len(ms)} beats")


if __name__ == "__main__":
    main()
