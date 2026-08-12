#!/usr/bin/env python3
"""Derive each entrance capture's t0 GEOMETRICALLY, from the authored white-flash ramp.

Never fitted to the score — that is the standing rule, because fitting t0 to MADC makes every
later measurement circular. The flash is a full-frame white overlay, so observed luma is LINEAR
in the layer's alpha (`L = L0 + a*(255-L0)`), which lets the authored alpha curve be matched
against the measured luma ramp without reference to our renderer at all.

  1. Find the flash layer in <skin>_Start[scene].json: the colorCurve whose alpha ramps 0 -> ~1.
  2. Take the authored time at 50% of the RISING edge (50%, not the peak, because the observed
     peak saturates around 253).
  3. In the capture, take the pre-flash baseline L0 and the peak, and interpolate the crossing of
     L0 + 0.5*(Lmax - L0) — sub-frame.
  4. t0 = t_measured - t_authored.

Luma is measured on the CENTRAL 60% of the frame, so Civilight Eterna's black pillarbox columns
cannot drag the mean and bias the crossing.

  python3 flash_t0.py <key> <skin-dir> [<key> <skin-dir> ...]
"""
import json
import os
import subprocess
import sys

import numpy as np

DYN = "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust"
RAW = os.path.join(os.path.dirname(os.path.abspath(__file__)), "REF_NEW")


def authored_flash(skin_dir):
    """(t_50, layer_index, amplitude) for the layer whose alpha ramps hardest to ~1."""
    d = os.path.join(DYN, skin_dir)
    f = [x for x in os.listdir(d) if x.endswith("_Start[scene].json")]
    if not f:
        return None
    j = json.load(open(os.path.join(d, f[0])))
    best = None
    for i, layer in enumerate(j.get("layers", [])):
        cur = layer.get("colorCurve")
        if not cur or len(cur) < 3:
            continue
        ts = [k[0] for k in cur]
        al = [k[4] for k in cur]
        lo, hi = min(al), max(al)
        if hi < 0.85 or hi - lo < 0.6:
            continue
        # rising edge: the LAST index where alpha is still below the half level, walking to the peak
        half = lo + 0.5 * (hi - lo)
        pk = int(np.argmax(al))
        t50 = None
        for k in range(pk, 0, -1):
            if al[k - 1] <= half <= al[k]:
                span = al[k] - al[k - 1]
                fr = 0.0 if span <= 0 else (half - al[k - 1]) / span
                t50 = ts[k - 1] + fr * (ts[k] - ts[k - 1])
                break
        if t50 is None:
            continue
        # The flash is the ramp NEAREST THE ENTRANCE END, not the largest one. Picking by
        # amplitude chose Mlynar's layer 9 (t 12.533) over his actual flash at layer 18
        # (13.828) and put his t0 1.36 s out.
        if best is None or t50 > best[0]:
            best = (t50, i, hi - lo)
    return best


def luma_trace(path, fps=30):
    """Mean luma per frame over the central 60%, as one ffmpeg pass (fast, whole clip)."""
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height",
         "-of", "csv=p=0", path], capture_output=True, text=True).stdout.strip().split(",")
    # FULL frame, deliberately. A constant black pillarbox scales L0, Lmax and the half-level by
    # the same factor, so the 50% crossing time is unchanged — and the full frame is what the
    # original three t0 values were derived from.
    _ = probe
    out = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-vf",
         f"fps={fps},scale=1:1:flags=area,format=gray", "-f", "rawvideo", "-"],
        capture_output=True).stdout
    return np.frombuffer(out, dtype=np.uint8).astype(np.float32), fps


def measured_flash(path, fps=30):
    """Sub-frame time of the 50% crossing on the rising edge of the clip's strongest flash."""
    L, fps = luma_trace(path, fps)
    if L.size < 10:
        return None
    pk = int(np.argmax(L))
    Lmax = float(L[pk])
    # baseline: median of the second before the ramp starts climbing
    j = pk
    while j > 0 and L[j - 1] < L[j]:
        j -= 1
    b0 = max(0, j - fps)
    L0 = float(np.median(L[b0:j])) if j > b0 else float(L[0])
    half = L0 + 0.5 * (Lmax - L0)
    for k in range(pk, 0, -1):
        if L[k - 1] <= half <= L[k]:
            span = L[k] - L[k - 1]
            fr = 0.0 if span <= 0 else (half - L[k - 1]) / span
            return (k - 1 + fr) / fps, L0, Lmax, pk / fps
    return None


def main():
    args = sys.argv[1:]
    print(f"{'key':5} {'raw clip':26} {'t_auth':>8} {'lyr':>4} {'t_meas':>8} {'L0':>6} {'Lmax':>6} {'t0':>8}")
    for key, skin in zip(args[0::2], args[1::2]):
        raw = os.path.join(RAW, f"{key}_entrance.mp4")
        if not os.path.exists(raw):
            print(f"{key:5} {'<no raw capture>':26}")
            continue
        a = authored_flash(skin)
        m = measured_flash(raw)
        if a is None or m is None:
            print(f"{key:5} {os.path.basename(raw):26} {'<no flash>' if a is None else '<no ramp>':>8}")
            continue
        t0 = m[0] - a[0]
        print(f"{key:5} {os.path.basename(raw):26} {a[0]:>8.3f} {a[1]:>4} {m[0]:>8.3f} {m[1]:>6.1f} {m[2]:>6.1f} {t0:>8.3f}")


if __name__ == "__main__":
    main()
