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
    # `--dc` adds the SIGNED mean-luma readout; it changes no score.
    dc = "--dc" in flags
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
    # `--srcaspect=W:H` corrects the reference clips' GEOMETRIC DISTORTION. **ON BY DEFAULT.**
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
    # ENABLED 2026-08-20. This is a property of the REFERENCE CLIPS, not of any one scorer, so it
    # defaults on here rather than being pinned per-caller — every path that scores against
    # `REF_NEW/` inherits it. `--srcaspect=none` reproduces the historical (distorted) basis.
    #
    # ⚠️ Every figure recorded before that date is on the OLD basis and is ~0.44 MADC pessimistic.
    # The re-baselined eight: wis 5.537 · exc 6.185 · ska 10.084 · eyja 10.410 · mue 11.640 ·
    # cel 17.018 · cet 17.693 · mly 16.999 — mean 11.946 (was 12.383).
    #
    # ⚠️ The correction assumes the reference was scaled from a 2340x1080 device frame, which is
    # what `capture_oracle.sh` produces. It is WRONG for the legacy `REF/` salvage clips (1280x624)
    # — pass `--srcaspect=none` for those.
    srcaspect = next((f.split("=", 1)[1] for f in flags if f.startswith("--srcaspect=")), "2340:1080")
    if srcaspect.lower() in ("none", "off", "0"):
        srcaspect = None
    beats = [float(t) for t in times.split(",")]
    # `--ourtimes=t1,t2,...` overrides WHICH of our rendered frames each beat reads, leaving the
    # GAME frame index alone. It exists for the CONTENT-CLOCK basis (see `cclock.py`): the
    # reference captures dropped frames under host load, so a game frame can hold content the
    # capture took up to several frames earlier, and comparing our render at container time
    # against their content from earlier charges us for the recorder. One entry per beat.
    #
    # ⚠️ Not a beat move and not a trim change. `--offset` still applies on top, so a caller passes
    # game-clock times here exactly as it would beats.
    ourtimes = next((f.split("=", 1)[1] for f in flags if f.startswith("--ourtimes=")), None)
    ourtimes = [float(t) for t in ourtimes.split(",")] if ourtimes else None
    if ourtimes is not None and len(ourtimes) != len(beats):
        print(f"--ourtimes has {len(ourtimes)} entries for {len(beats)} beats")
        sys.exit(2)

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

    ys, cs, ms, rs, dcs = [], [], [], [], []
    for bi, b in enumerate(beats):
        ot = b if ourtimes is None else ourtimes[bi]
        o_img = Image.open(f"{ours_dir}/t{ot + off:.2f}.png").convert("RGB")
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
        # PEARSON r between the two luma planes. MADC is a per-pixel MAGNITUDE difference and is
        # BLIND to "right content, wrong place": two frames can share a mean and a std and agree
        # nowhere. Kal'tsit sat at r ~= 0.05 for three sessions while every investigation read her
        # diff map as a missing atmospheric veil; the real fault was a dropped camera track, and
        # r is what exposed it. A well-registered skin reads 0.87-0.99 (wisdel 0.99).
        # A FLAT beat (a fade-to-white/black hold) has no spatial structure to correlate and
        # yields nan — excu2's t=6 is exactly this (Y=2.000, C=0.000). Excluded from the mean
        # rather than poisoning it; a real frame always clears this bar by orders of magnitude.
        r = (
            float(np.corrcoef(yo.ravel(), yg.ravel())[0, 1])
            if yo.std() > 1e-3 and yg.std() > 1e-3
            else float("nan")
        )
        if np.isfinite(r):
            rs.append(r)
        ys.append(mad_y)
        cs.append(mad_c)
        ms.append(mad_y + mad_c)
        # SIGNED mean-luma difference, ours minus game. MADC is an absolute value and is blind to
        # a systematic brightness BIAS: a render uniformly too bright and one uniformly too dark
        # score the same. That blindness is why the "DC excess" has been a suspicion in the
        # ramMainTex notes for a long time without ever being a number. Collected always, printed
        # only under `--dc`, so no existing output moves.
        dcs.append(float(yo.mean() - yg.mean()))
        rtxt = f"{r:6.3f}" if np.isfinite(r) else "  flat"
        if dc:
            print(f"  t={b:<6} DC(ours-game) = {dcs[-1]:+8.3f}   ours {yo.mean():7.3f}  game {yg.mean():7.3f}")
        print(f"  t={b:<6} MADC={mad_y + mad_c:7.3f}  (Y={mad_y:7.3f}  C={mad_c:6.3f}  r={rtxt})")
    if ms:
        print(f"MEAN MAD(luma-only, legacy) = {np.mean(ys):.3f}  over {len(ms)} beats")
        print(f"MEAN CHROMA = {np.mean(cs):.3f}  over {len(ms)} beats")
        # Self-identify the basis: a bare number is otherwise indistinguishable from a
        # pre-2026-08-20 figure, which is ~0.44 pessimistic.
        print(f"  [basis: {'aspect-corrected 2340:1080' if srcaspect else 'RAW reference aspect (historical)'}]")
        print(f"MEAN MADC = {np.mean(ms):.3f}  over {len(ms)} beats")
        if dc:
            print(f"MEAN DC(ours-game) = {np.mean(dcs):+.3f}  over {len(ms)} beats")
        mr = float(np.mean(rs)) if rs else float("nan")
        # Every well-behaved reference sits at 0.87-0.99 (wisdel 0.99, cel 0.90, cet 0.88,
        # mlynar 0.87). Below ~0.70 the frames genuinely do not line up, which is EITHER a
        # geometry fault (camera track, framing, zoom) OR content large enough to dominate the
        # picture. A scale+translation cross-correlation search separates the two; do that before
        # reading a diff map, because a diff map cannot tell them apart.
        warn = "   <== LOW: run a scale/translation search before blaming content" if mr < 0.70 else ""
        nflat = len(ms) - len(rs)
        flat = f"  [{nflat} flat beat(s) excluded]" if nflat else ""
        print(f"MEAN r = {mr:.3f}  (registration; corpus refs run 0.87-0.99){flat}{warn}")


if __name__ == "__main__":
    main()
