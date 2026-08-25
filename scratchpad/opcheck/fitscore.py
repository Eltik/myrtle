#!/usr/bin/env python3
"""Score a VIEWER capture (Fashion-Gallery L2D viewer) against our render by first
solving the (scale, dx, dy) framing transform between them.

Why: the in-game L2D viewer frames the art differently from our renderer (measured
~1.67x on chyue), so a raw MADC is dominated by that geometry and is flat across
every t0 offset. `?statcam=1` is NOT the right transform either (it scored worse).
Since the viewer shows the SETTLED IDLE, the art is static — a 3-parameter fit is
well posed.

  fitscore.py <ours_dir> <game.mp4> <beats> [--offset=S] [--fps=30] [--fit-beat=B]
              [--inner] [--save=fit.json]

`ours_dir` holds our rendered frames named `t{beat+offset:.2f}.png` (as score_new.sh
writes them). The transform is fitted ONCE on `--fit-beat` (default: the last beat,
i.e. most settled) and then applied to EVERY beat — a per-skin constant, not a
per-frame refit, so it cannot silently absorb real error.

Anti-cheat: uncovered area (where the shrunk render doesn't reach) is filled with the
GAME's own sampled background, and MADC is computed over the FULL crop. Shrinking to
a dot therefore scores badly instead of winning.
"""
import subprocess, sys, os, json
import numpy as np
from PIL import Image

def ycc(a):
    R, G, B = a[..., 0], a[..., 1], a[..., 2]
    Y = 0.299 * R + 0.587 * G + 0.114 * B
    Cb = -0.168736 * R - 0.331264 * G + 0.5 * B + 128
    Cr = 0.5 * R - 0.418688 * G - 0.081312 * B + 128
    return Y, Cb, Cr

def madc(o, g, x0, x1):
    oY, oCb, oCr = ycc(o[:, x0:x1]); gY, gCb, gCr = ycc(g[:, x0:x1])
    return float(np.abs(oY-gY).mean() + (np.abs(oCb-gCb).mean() + np.abs(oCr-gCr).mean()) / 2)

def warp(img, s, dx, dy, fill):
    """Scale `img` by s about its centre, translate by (dx,dy); fill uncovered with `fill`."""
    H, W = img.shape[:2]; cx, cy = W / 2.0, H / 2.0
    a = 1.0 / s
    c = cx - (cx + dx) / s
    f = cy - (cy + dy) / s
    return np.asarray(Image.fromarray(img.astype("uint8")).transform(
        (W, H), Image.AFFINE, (a, 0, c, 0, a, f), resample=Image.BILINEAR,
        fillcolor=tuple(int(v) for v in fill))).astype(np.float32)

def bg_of(g):
    """The viewer backdrop: median of a border ring (art sits centre-ish)."""
    ring = np.concatenate([g[:12].reshape(-1,3), g[-12:].reshape(-1,3),
                           g[:, :12].reshape(-1,3), g[:, -12:].reshape(-1,3)])
    return np.median(ring, axis=0)

def game_frames(mp4, beats, fps):
    probe = subprocess.run(["ffprobe","-v","error","-select_streams","v:0",
        "-show_entries","stream=width,height","-of","csv=p=0",mp4],
        capture_output=True, text=True).stdout.strip().split(",")
    W, H = int(probe[0]), int(probe[1])
    raw = subprocess.run(["ffmpeg","-v","error","-i",mp4,"-f","rawvideo","-pix_fmt","rgb24","-"],
                         capture_output=True).stdout
    n = len(raw)//(W*H*3)
    G = np.frombuffer(raw, np.uint8)[:n*W*H*3].reshape(n, H, W, 3)
    out = {}
    for b in beats:
        i = int(round(b*fps))
        if i < n: out[b] = G[i].astype(np.float32)
    return out, (W, H)

def main():
    ours_dir, mp4, beats_s = sys.argv[1], sys.argv[2], sys.argv[3]
    fl = sys.argv[4:]
    def opt(k, d):
        v = next((a.split("=",1)[1] for a in fl if a.startswith(k+"=")), None)
        return type(d)(v) if v is not None else d
    off  = opt("--offset", 0.0); fps = opt("--fps", 30.0)
    inner = any(a == "--inner" for a in fl)
    save = opt("--save", "")
    beats = [float(b) for b in beats_s.split(",")]
    fit_beat = opt("--fit-beat", beats[-1])

    G, (W, H) = game_frames(mp4, beats, fps)
    x0, x1 = (36, 864) if inner else (0, W)

    O = {}
    for b in beats:
        p = f"{ours_dir}/t{b+off:.2f}.png"
        if os.path.exists(p) and b in G:
            O[b] = np.asarray(Image.open(p).convert("RGB").resize((W, H), Image.LANCZOS)).astype(np.float32)
    if fit_beat not in O:
        fit_beat = sorted(O)[-1]
    print(f"[fit] {len(O)} beats usable | fitting on t={fit_beat} (settled) | crop x[{x0},{x1})")

    go, oo = G[fit_beat], O[fit_beat]
    fill = bg_of(go)
    print(f"[fit] game backdrop fill = {fill.round(0)}")

    # ---- coarse (downsampled for speed) then fine ----
    def score(s, dx, dy, gf, of, xs0, xs1):
        return madc(warp(of, s, dx, dy, fill), gf, xs0, xs1)
    # Coarse: for each scale, find the BEST translation over the WHOLE plane via FFT
    # cross-correlation (a bounded grid ran to its own edge and mis-fit).
    from numpy.fft import rfft2, irfft2
    def best_shift(gf, of):
        a = gf - gf.mean(); b = of - of.mean()
        Hh, Ww = a.shape
        cc = irfft2(rfft2(a) * np.conj(rfft2(b)), s=(Hh, Ww))
        iy, ix = np.unravel_index(int(np.argmax(cc)), cc.shape)
        if iy > Hh//2: iy -= Hh
        if ix > Ww//2: ix -= Ww
        return ix, iy                      # shift to apply to `of` to match `gf`
    sc = 0.25
    gs = np.asarray(Image.fromarray(go.astype("uint8")).convert("L").resize((int(W*sc), int(H*sc)), Image.LANCZOS)).astype(np.float32)
    best = (1e9, 1.0, 0.0, 0.0)
    for s in np.arange(0.35, 1.66, 0.05):
        sm = warp(oo, s, 0, 0, fill)
        smg = np.asarray(Image.fromarray(sm.astype("uint8")).convert("L").resize((int(W*sc), int(H*sc)), Image.LANCZOS)).astype(np.float32)
        ix, iy = best_shift(gs, smg)
        dx, dy = ix/sc, iy/sc
        m = score(s, dx, dy, go, oo, x0, x1)
        if m < best[0]: best = (m, s, dx, dy)
    print(f"[fit] coarse(FFT): s={best[1]:.2f} dx={best[2]:.0f} dy={best[3]:.0f} (MADC {best[0]:.2f})")
    _, s0, dx0, dy0 = best
    best = (1e9, s0, dx0, dy0)
    for s in np.arange(s0-0.06, s0+0.061, 0.01):
        for dx in np.arange(dx0-12, dx0+12.1, 3):
            for dy in np.arange(dy0-12, dy0+12.1, 3):
                m = score(s, dx, dy, go, oo, x0, x1)
                if m < best[0]: best = (m, s, dx, dy)
    m_fit, s, dx, dy = best
    print(f"[fit] FIT: scale={s:.3f} dx={dx:+.0f} dy={dy:+.0f}  (MADC at fit beat {m_fit:.3f})")

    # 🚨 COVERAGE, and it is not cosmetic. The anti-cheat fill stops a shrinking fit from
    # winning, but it also means every pixel it fills is GAME-BACKDROP vs GAME, containing
    # none of our render. Any statistic computed on the fitted frame is contaminated there,
    # in proportion to how much of the frame the transform vacates.
    #
    # This cost a whole finding. A "bottom-fifth over-brightness shared by all five settled
    # clips, +16.646 to +57.261" was measured on fitted frames whose bottom fifth our render
    # covered 0.00% of on three of the five, 15.72% on a fourth. The shared vertical structure
    # it rested on (row-profile pairwise r +0.639) collapses to +0.015 once the fill is
    # excluded, and fugue's bottom band FLIPS SIGN, +41.750 -> -18.885. Nothing we drew was in
    # the band being measured, which is why every group ablation, the spine included, left the
    # number bit-identical.
    #
    # Read the fill share before quoting any regional number off a fitted frame. Above ~20% the
    # fitted MADC is substantially not about our renderer.
    mask = np.asarray(Image.fromarray(np.full((H, W), 255, np.uint8)).transform(
        (W, H), Image.AFFINE, (1.0/s, 0, W/2.0 - (W/2.0+dx)/s, 0, 1.0/s, H/2.0 - (H/2.0+dy)/s),
        resample=Image.BILINEAR, fillcolor=0)) > 200
    mi = mask[:, x0:x1]
    rowcov = mi.mean(axis=1)
    print(f"[fit] COVERAGE by our render: whole frame {mi.mean()*100:.2f}%, "
          f"top fifth {rowcov[:len(rowcov)//5].mean()*100:.2f}%, "
          f"bottom fifth {rowcov[-len(rowcov)//5:].mean()*100:.2f}%")
    _oy, _ob, _or = ycc(warp(oo, s, dx, dy, fill)[:, x0:x1])
    _gy, _gb, _gr = ycc(go[:, x0:x1])
    _d = np.abs(_oy-_gy) + (np.abs(_ob-_gb) + np.abs(_or-_gr)) / 2
    _fill_share = _d[~mi].sum() / _d.sum() if _d.sum() > 0 else 0.0
    print(f"[fit] FILL SHARE of the fitted residual: {_fill_share*100:.1f}%"
          f"{'   ⚠️ regional statistics off this frame are NOT about our renderer' if _fill_share > 0.20 else ''}")

    # ---- apply the SAME transform to every beat ----
    print(f"\n{'beat':>6}  {'raw':>8}  {'fitted':>8}")
    raws, fits = [], []
    for b in sorted(O):
        raw_m = madc(O[b], G[b], x0, x1)
        fit_m = madc(warp(O[b], s, dx, dy, bg_of(G[b])), G[b], x0, x1)
        raws.append(raw_m); fits.append(fit_m)
        print(f"{b:>6}  {raw_m:8.3f}  {fit_m:8.3f}")
    print(f"\nMEAN raw    MADC = {np.mean(raws):.3f}")
    print(f"MEAN FITTED MADC = {np.mean(fits):.3f}   (scale {s:.3f}, dx {dx:+.0f}, dy {dy:+.0f})")
    if save:
        json.dump({"scale": float(s), "dx": float(dx), "dy": float(dy),
                   "fit_beat": float(fit_beat), "mean_raw": float(np.mean(raws)),
                   "mean_fitted": float(np.mean(fits))}, open(save, "w"), indent=1)
        print(f"[fit] saved {save}")

main()
