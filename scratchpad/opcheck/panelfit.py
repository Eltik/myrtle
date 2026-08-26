#!/usr/bin/env python3
"""Grade the PANEL settle of the thirteen entrance skins against the contain-fit target.

WHY THIS EXISTS, AND WHY IT IS NOT AN EYEBALL CHECK
The product requirement is that a detail card plays the `_Start` cinematic and then lands in
the SAME box the seventy non-entrance skins land in. "Lands in the same box" is a pixel claim,
so it needs a stated tolerance declared BEFORE the numbers, not a judgement made after seeing
them.

🚨 THE TOLERANCE IS NOT ZERO, AND THAT IS A MEASUREMENT, NOT A CONCESSION.
The idle animation BREATHES: the drawn content box moves while the camera is provably static.
Sampling the same build twice, 1.5 s apart, at the settled idle, the four margins move by up
to 10 px (excu2 bottom +10, chyue (5,10,4,0), whitw2 top -7, agoat2 right -6). So a 1-to-5 px
difference from the target is animation PHASE, not framing, and grading by eye reads those as
failures while missing that a 12 px one is real.

  TOL = 10 px per edge at 900x675, and BOTH samples must pass on ALL FOUR edges.

A single sample cannot distinguish "framed wrong" from "sampled at a different phase", which
is why two are mandatory. The drift column is printed so a marginal call can be read rather
than argued.

🚨 NEVER GRADE ON A GUESSED SETTLE TIME. The post-hand-off dolly runs for
`max(0.5, entranceDuration - entranceTransform)` seconds, which is 12.833 s on Skadi2. A first
pass sampled at `entranceDuration + 6` graded three skins mid-flight and read them as framing
failures; Skadi2 went from (0,0,0,0) wrong to an EXACT match once sampled post-dolly. This
script takes the times from the `_Start[scene].json` the renderer itself reads, and asserts
`__settleProbe().dolly is None` at the sample. A sample with a live dolly is a HARD FAIL of the
measurement, not a failing skin.

RESULTS ON RECORD, so the next run does not re-derive them.

  ⛔ REFUTED: "the entrance's own nodes inflate the box the contain fit is given". `--control`
     says NO on all thirteen, bit-identical to six decimals with the `_Start` composite built and
     not built. Structural, and it could have been read off the file: `main` is built at
     SceneIllust.tsx `const main = await buildComposite(...)`, the entrance HUNDREDS of lines
     later, and `paintedLocalBounds` measures the composite's OWN container.

  ⛔ REFUTED: the residual-transform successors (`entrancePullOut`, `handoffPanDelta`,
     `centerBlend` composing with the dolly target). The `cam` column compares the live world
     transform against a contain fit on `contentBounds` computed here, independently, and it reads
     ds +0.00000 dtx +0.00 dty +0.00 on all thirteen. Nothing survives into the terminus.

  🚨 THE MARGIN TABLE CANNOT GRADE FRAMING AT 10 px ON EVERY SKIN, and that is a property of the
     metric, not of the renderer. With the camera PINNED to one box by `?framebox=` so it cannot
     move at all, the drawn bbox still travels with the idle's phase:

       cqbw   bottom margin 40 -> 1 -> 40 -> 1 across four phases (39 px)
       wisdel left 102..125, right 99..127 across four phases (23 / 28 px)
       mlynar left 107 at five phases out of six, 93 at the sixth (14 px)

     Two samples 1.5 s apart bound the LOCAL drift, which is what the 10 px envelope is for. They
     do not bound the drift between two DIFFERENT phases, and the target table was measured at
     idle phase 9.60 while a post-dolly sample lands wherever the cinematic's length puts it. Grade
     framing on the `cam` column, which is phase-free; read the margins as corroboration.

  🔑 The idle's phase is NOT the wall clock once a cinematic plays. `spineRef.current.update(dt)`
     advances only the ACTIVE composite, and `main` is activated at the hand-off, so idle phase =
     wall time minus `entranceDuration`. A margin measured at wall 9.60 with no cinematic and one
     measured at wall 9.60 with one are 9.60 s apart in the animation.

  panelfit.py [skin ...] [--jobs=N] [--control] [--tol=N]

`--control` additionally renders each skin with `?statcam=1`, which declines to build the
`_Start` composite at all, and prints its `contentBounds` beside the normal one. That is the
falsifier for "the entrance's own nodes inflate the box the contain fit is given".
"""
import json, os, re, shutil, subprocess, sys, concurrent.futures as cf
import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
DYN = "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust"
OUT = os.environ.get("PF_OUT", "/private/tmp/claude-501/-Users-eltik-Documents-Coding-myrtle/751065f0-d8ca-4714-a615-795c1f6925be/scratchpad/pf")
W, H = 900, 675
TOL = 10          # px, per edge, both samples. See the header.
BGTHR = 12        # a pixel differs from the page backdrop by more than this on some channel

# The target: margins (top, bottom, left, right) each skin settled at on the build that framed
# the panel with NO entrance loaded (58296889), at 900x675. This is the box the 70 non-entrance
# skins use, so it is the definition of "same box", not a preference.
TARGETS = {
    "char_003_kalts_boc#6":         (21, 20, 51, 48),
    "char_1012_skadi2_iteration#2": (20, 20, 68, 74),
    "char_1016_agoat2_epoque#34":   (63, 61, 23, 21),
    "char_1032_excu2_sale#12":      (19, 18, 64, 69),
    "char_1035_wisdel_sale#14":     (16, 24, 116, 95),
    "char_1038_whitw2_sale#15":     (0, 19, 92, 92),
    "char_113_cqbw_epoque#7":       (20, 40, 81, 40),
    "char_2024_chyue_cfa#1":        (0, 4, 80, 81),
    "char_245_cello_sale#12":       (23, 20, 107, 109),
    "char_249_mlyss_boc#8":         (30, 18, 94, 77),
    "char_4064_mlynar_epoque#28":   (19, 22, 108, 109),
    "char_4134_cetsyr_epoque#50":   (19, 11, 177, 176),
    "char_472_pasngr_epoque#17":    (17, 17, 143, 141),
}


def settle_time(skin):
    """When the post-hand-off dolly is over, from the gamedata the renderer itself reads.

    `openStandingIdle` starts the dolly with `delay` 0.15 and duration
    `entrancePullOut.dur = max(0.5, entranceDuration - entranceTransform)`, or 2.0 when the skin
    ships no `_transform` beat. The hand-off itself fires at `entranceDuration`. The +2.0 is
    slack for the crossfade, not a fitted constant.
    """
    d = os.path.join(DYN, skin)
    f = [x for x in os.listdir(d) if x.endswith("_Start[scene].json")]
    j = json.load(open(os.path.join(d, f[0])))
    dur = j.get("entranceDuration") or 0.0
    tr = j.get("entranceTransform")
    dolly = max(0.5, dur - tr) if tr else 2.0
    return dur + 0.15 + dolly + 2.0


def margins(png):
    a = np.asarray(Image.open(png).convert("RGB")).astype(int)
    bg = a[0, 0]
    d = np.abs(a - bg).max(axis=2) > BGTHR
    ys, xs = np.where(d)
    if len(xs) == 0:
        return None
    return (int(ys.min()), int(a.shape[0] - 1 - ys.max()), int(xs.min()), int(a.shape[1] - 1 - xs.max()))


def render(skin, shots, extra, tag):
    # 🚨 `dyn_illust_`, not just "a .skel": every skin directory also ships a `dyn_portrait_`
    # skeleton, and `os.listdir` order put it first on four of the thirteen. It renders something
    # that looks plausible, so the mistake is silent. `score_new.sh` globs `dyn_illust_*` for the
    # same reason.
    skel = sorted(x for x in os.listdir(os.path.join(DYN, skin)) if x.startswith("dyn_illust_") and x.endswith(".skel") and "_Start" not in x)[0]
    import urllib.parse as up
    url = f"/spine/DynIllust/{up.quote(skin, safe='')}/{up.quote(skel, safe='')}"
    od = os.path.join(OUT, f"{skin}_{tag}")
    # 🚨 WIPE IT. The frame filenames are a pure function of the sample time, so a run whose
    # renderer died silently reads the PREVIOUS run's frame and grades it as this run's. That
    # happened: an aborted pass left `dyn_portrait_` frames on disk at the same times and three
    # skins were graded against them.
    shutil.rmtree(od, ignore_errors=True)
    os.makedirs(od, exist_ok=True)
    env = dict(os.environ, NODE_PATH=os.path.join(HERE, "node_modules"), EXTRA=extra, PROBE="__settleProbe")
    p = subprocess.run(["node", os.path.join(HERE, "rec.js"), url, od, ",".join(f"{s:.2f}" for s in shots), str(W), str(H)],
                       capture_output=True, text=True, env=env)
    if p.returncode != 0:
        print(f"[warn] {skin}/{tag}: rec.js exit {p.returncode}: {p.stderr.strip().splitlines()[-1:] }", file=sys.stderr)
    probes = {}
    for line in p.stdout.splitlines():
        m = re.match(r"^PROBE (\S+) (.*)$", line)
        if m:
            try:
                probes[m.group(1)] = json.loads(m.group(2))
            except Exception:
                probes[m.group(1)] = None
    return od, probes


def one(skin, control):
    t1 = settle_time(skin)
    t2 = t1 + 1.5
    od, pr = render(skin, [t1, t2], "surface=panel", "panel")
    rows = []
    for t in (t1, t2):
        f = os.path.join(od, f"t{t:.2f}.png")
        rows.append((margins(f) if os.path.exists(f) else None, pr.get(f)))
    ctl = None
    if control:
        cod, cpr = render(skin, [1.0], "surface=panel&statcam=1", "ctl")
        ctl = next(iter(cpr.values()), None)
    return skin, t1, rows, ctl


# DYNAMIC_FIT_MARGIN, from chibi/helpers.ts: `contain` fits the box then insets by this.
FIT_MARGIN = 0.95


def cam_check(p):
    """Did the camera end where a contain fit on `contentBounds` would put it?

    This separates the two ways the settle can miss. If the camera matches the prediction to
    sub-pixel, the box was right and any margin error is CONTENT (animation phase, or something
    drawn that was not drawn in the reference build). If it does not match, a residual transform
    composed with the dolly target and the box never took effect.
    """
    if not isinstance(p, dict) or not p.get("contentBounds") or not p.get("root"):
        return None
    c, r, sc = p["contentBounds"], p["root"], p["screen"]
    s = min(sc["w"] / c["width"], sc["h"] / c["height"]) * FIT_MARGIN
    tx = sc["w"] / 2 - (c["x"] + c["width"] / 2) * s
    ty = sc["h"] / 2 - (c["y"] + c["height"] / 2) * s
    return {"ds": r["a"] - s, "dtx": r["tx"] - tx, "dty": r["ty"] - ty}


def cb_str(p):
    if not isinstance(p, dict) or not p.get("contentBounds"):
        return "None"
    c = p["contentBounds"]
    return f"({c['x']:.3f},{c['y']:.3f},{c['width']:.3f},{c['height']:.3f})"


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    fl = [a for a in sys.argv[1:] if a.startswith("--")]
    jobs = int(next((f.split("=")[1] for f in fl if f.startswith("--jobs=")), 3))
    control = any(f == "--control" for f in fl)
    tol = int(next((f.split("=")[1] for f in fl if f.startswith("--tol=")), TOL))
    skins = args or list(TARGETS)

    print(f"TOLERANCE: {tol} px per edge, BOTH samples, all four edges. Samples 1.5 s apart at the")
    print(f"post-dolly settle time derived per skin from `_Start[scene].json`. Margins are")
    print(f"(top, bottom, left, right) at {W}x{H}. A live dolly at the sample is a MEASUREMENT FAIL.\n")
    print(f"{'skin':34s} {'target':>20s} {'t1':>20s} {'t2':>20s} {'drift':>18s}  verdict")

    res = {}
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        for skin, t1, rows, ctl in ex.map(lambda s: one(s, control), skins):
            res[skin] = (t1, rows, ctl)
            with open(os.path.join(OUT, f"{skin}_probes.json"), "w") as fh:
                json.dump({"t1": t1, "probes": [r[1] for r in rows], "ctl": ctl}, fh, indent=1)

    npass = 0
    for skin in skins:
        t1, rows, ctl = res[skin]
        tgt = TARGETS[skin]
        (m1, p1), (m2, p2) = rows
        if m1 is None or m2 is None:
            print(f"{skin:34s} {'':>20s} RENDER FAILED")
            continue
        live = [p for p in (p1, p2) if p and p.get("dolly")]
        drift = tuple(m1[i] - m2[i] for i in range(4))
        worst = max(max(abs(m1[i] - tgt[i]), abs(m2[i] - tgt[i])) for i in range(4))
        ok = worst <= tol and not live
        npass += ok
        edges = "TBLR"
        bad = ",".join(f"{edges[i]}{m1[i]-tgt[i]:+d}/{m2[i]-tgt[i]:+d}" for i in range(4) if max(abs(m1[i]-tgt[i]), abs(m2[i]-tgt[i])) > tol)
        v = "PASS" if ok else ("DOLLY LIVE" if live else f"FAIL {bad}")
        cc = cam_check(p2) or cam_check(p1)
        cs = "cam n/a" if not cc else f"cam ds{cc['ds']:+.5f} dtx{cc['dtx']:+.2f} dty{cc['dty']:+.2f}"
        print(f"{skin:34s} {str(tgt):>20s} {str(m1):>20s} {str(m2):>20s} {str(drift):>18s}  {v}  (worst {worst})  {cs}")
    print(f"\n{npass}/{len(skins)} within {tol} px on all four edges, both samples.")

    if control:
        print(f"\nFALSIFIER: does building the `_Start` composite change `main.contentBounds`?")
        print(f"{'skin':34s} {'entrance BUILT':>44s} {'entrance NOT built (statcam=1)':>44s}  same")
        for skin in skins:
            t1, rows, ctl = res[skin]
            a = cb_str(rows[0][1]); b = cb_str(ctl)
            print(f"{skin:34s} {a:>44s} {b:>44s}  {'YES' if a == b else 'NO'}")


main()
