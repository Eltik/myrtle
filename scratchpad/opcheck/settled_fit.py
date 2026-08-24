#!/usr/bin/env python3
"""IDENTIFY THE SETTLED IDLE: how does our post-hand-off shot register against the capture?

Every parity number we have scores the ENTRANCE — `mad.py`'s beats all land before the
hand-off, so the settled idle is measured by nothing. This fits our settled frame to the
game's with SIFT + RANSAC (the same estimator `camfit.py` uses on entrance beats) and
reports scale / rotation / translation per skin.

  scale 1.00, dT ~0     -> the settled framing is right
  scale != 1.00         -> wrong EXTENT   (which authored view px we settle to)
  dT != 0               -> wrong CENTRE   (what the frame is centred on)

Sample times are derived, not hardcoded: `entranceDuration` from the exported `_Start`
scene + a margin, clipped to what the reference clip actually contains.

Usage:  settled_fit.py [skin-key ...]      (default: every key in all8.sh with a capture)
"""
import json, os, re, subprocess, sys, urllib.parse, glob
import numpy as np, cv2

HERE = os.path.dirname(os.path.abspath(__file__))
DEPLOY = "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust"
TMP = os.environ.get("SETTLED_TMP", "/Users/eltik/.claude/jobs/07b9c766/tmp/settled")
W, H, FPS = 900, 416, 30

def parse_all8():
    """DIR / REFOFF / entrance duration live in all8.sh — read them, never restate them."""
    txt = open(os.path.join(HERE, "all8.sh")).read()
    d = dict(re.findall(r"DIR\[(\w+)\]='([^']+)'", txt))
    off = {k: float(v) for k, v in re.findall(r"REFOFF\[(\w+)\]=(-?[\d.]+)", txt)}
    return d, off

def ent_duration(skin_dir):
    for f in glob.glob(os.path.join(DEPLOY, skin_dir, "*_Start[[]scene[]].json")):
        j = json.load(open(f))
        v = j.get("entranceDuration")
        if v: return float(v)
    return None

def clip_len(path):
    out = subprocess.run(["ffprobe","-v","error","-show_entries","format=duration",
                          "-of","default=nw=1:nk=1", path], capture_output=True, text=True)
    try: return float(out.stdout.strip())
    except ValueError: return None

def degenerate(path):
    """A frame with almost no distinct colours never drew. ⚠️ The FIRST render after a deploy
    comes back PURE BLACK — cold vite/module cache — and silently poisons a whole sweep."""
    import numpy as _np
    from PIL import Image as _I
    a = _np.asarray(_I.open(path).convert("RGB"))
    return len(_np.unique(a.reshape(-1, 3), axis=0)) < 100

def render(skin_dir, times, outdir):
    skels = [f for f in os.listdir(os.path.join(DEPLOY, skin_dir))
             if f.endswith(".skel") and "portrait" not in f and "_Start" not in f]
    if not skels: return False
    q = urllib.parse.quote
    os.makedirs(outdir, exist_ok=True)
    env = dict(os.environ, NODE_PATH=os.path.join(HERE, "node_modules"))
    subprocess.run(["node", os.path.join(HERE, "rec.js"),
                    f"/spine/DynIllust/{q(skin_dir, safe='')}/{q(skels[0], safe='')}",
                    outdir, ",".join(f"{t:g}" for t in times), str(W), str(H)],
                   env=env, capture_output=True)
    subprocess.run(["pkill", "-f", "puppeteer/chrome"], capture_output=True)
    pngs = glob.glob(os.path.join(outdir, "*.png"))
    if pngs and any(degenerate(p) for p in pngs):   # cold-cache black frame: render again
        subprocess.run(["node", os.path.join(HERE, "rec.js"),
                        f"/spine/DynIllust/{q(skin_dir, safe='')}/{q(skels[0], safe='')}",
                        outdir, ",".join(f"{t:g}" for t in times), str(W), str(H)],
                       env=env, capture_output=True)
        subprocess.run(["pkill", "-f", "puppeteer/chrome"], capture_output=True)
    return True

def game_frame(clip, t):
    p = os.path.join(TMP, "g.png")
    subprocess.run(["ffmpeg","-v","error","-ss",f"{t:.3f}","-i",clip,"-frames:v","1",
                    "-vf",f"scale={W}:{H}","-y",p], capture_output=True)
    return cv2.imread(p)

def fit(ours_bgr, game_bgr, centre=0.0):
    """`centre` > 0 restricts BOTH images to that central fraction of the width.

    ⚠️ A full-frame fit is NOT trustworthy on a self-similar scene. Kal'tsit's backdrop is
    repeating crystal shards, and unrestricted SIFT locks onto them one repeat off: it returns
    scale 2.40 with 90 inliers where a character-anchored fit returns 1.01. The character is the
    only structure in these shots that cannot alias, so fit on it and treat a full-frame answer
    that disagrees as the aliased one."""
    a = cv2.cvtColor(ours_bgr, cv2.COLOR_BGR2GRAY)
    b = cv2.cvtColor(game_bgr, cv2.COLOR_BGR2GRAY)
    # mask blown-out canvas so keys land on real art, not on uncovered frame
    mask = (a < 245).astype(np.uint8) * 255
    maskb = None
    if centre > 0:
        h, w = a.shape
        x0, x1 = int(w * (1 - centre) / 2), int(w * (1 + centre) / 2)
        band = np.zeros_like(mask); band[:, x0:x1] = 255
        mask = cv2.bitwise_and(mask, band)
        maskb = band
    sift = cv2.SIFT_create(nfeatures=4000)
    k1, d1 = sift.detectAndCompute(a, mask)
    k2, d2 = sift.detectAndCompute(b, maskb)
    if d1 is None or d2 is None or len(k1) < 8 or len(k2) < 8: return None
    good = [m for m, n in cv2.BFMatcher().knnMatch(d1, d2, k=2) if m.distance < 0.75 * n.distance]
    if len(good) < 8: return None
    src = np.float32([k1[g.queryIdx].pt for g in good]).reshape(-1,1,2)
    dst = np.float32([k2[g.trainIdx].pt for g in good]).reshape(-1,1,2)
    M, inl = cv2.estimateAffinePartial2D(src, dst, method=cv2.RANSAC, ransacReprojThreshold=3.0)
    if M is None: return None
    return (float(np.hypot(M[0,0], M[1,0])), float(np.degrees(np.arctan2(M[1,0], M[0,0]))),
            float(M[0,2]), float(M[1,2]), int(inl.sum()), len(good))

def main():
    DIR, OFF = parse_all8()
    keys = sys.argv[1:] or sorted(DIR)
    os.makedirs(TMP, exist_ok=True)
    print(f"{'skin':<8}{'t':>6}{'inl':>6}{'scale':>8}{'rot':>7}{'dX':>7}{'dY':>7}   verdict")
    for k in keys:
        sd = DIR.get(k)
        clip = os.path.join(HERE, "REF_NEW", f"{k}_game_fresh.mp4")
        if not sd or not os.path.exists(clip): continue
        dur, cl = ent_duration(sd), clip_len(clip)
        if dur is None or cl is None:
            print(f"{k:<8}   -- no entranceDuration or clip"); continue
        off = OFF.get(k, 0.0)
        times = [t for t in (dur + 1.0, dur + 2.5) if t + off < cl - 0.25]
        if not times:
            print(f"{k:<8}   -- capture ends at {cl:.1f}s, entrance runs to {dur:.1f}s (no settled frames)")
            continue
        out = os.path.join(TMP, k)
        subprocess.run(["rm","-rf",out], capture_output=True)
        if not render(sd, times, out): continue
        for t in times:
            fs = [f for f in glob.glob(f"{out}/*.png")
                  if abs(float(re.search(r"t([\d.]+)\.png", os.path.basename(f)).group(1)) - t) < 0.01]
            if not fs: print(f"{k:<8}{t:6.1f}   no render"); continue
            ours_img, game_img = cv2.imread(fs[0]), game_frame(clip, t + off)
            rc = fit(ours_img, game_img, centre=0.55)   # character-anchored: cannot alias
            rf = fit(ours_img, game_img)                # full frame: may alias on repeats
            if rc is None: print(f"{k:<8}{t:6.1f}      --   (no fit)"); continue
            sc, rot, dx, dy, inl, tot = rc
            bad = []
            if abs(sc - 1) > 0.03: bad.append("EXTENT")
            if abs(dx) > 20 or abs(dy) > 20: bad.append("CENTRE")
            v = "ok" if not bad else " + ".join(bad)
            if rf and abs(rf[0] - sc) > 0.05:
                v += f"  [full-frame fit disagrees: {rf[0]:.2f} — aliased]"
            print(f"{k:<8}{t:6.1f}{inl:6d}{sc:8.4f}{rot:+7.2f}{dx:+7.1f}{dy:+7.1f}   {v}")

if __name__ == "__main__":
    main()
