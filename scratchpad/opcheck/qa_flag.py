#!/usr/bin/env python3
"""Flag BROKEN skins in a corpus QA render — the ship gate for the 74 skins with no capture.

74 of 82 deployed skins have no reference, so the only answerable question is "is anything
broken", not "is it closer". Flags, per skin:

  BLANK      every frame is effectively empty (mean < 2) — the skin renders nothing
  DROPOUT    some frames blank, others not — an intermittent recorder/render failure
  BLOWN      a frame is >=40% saturated (>=250) — a blow-out that hides the character
  DARK       a frame is >=85% near-black (<=8) — content loss
  FLAT       a frame has std < 3 — a uniform fill where art should be

Usage: qa_flag.py <qadir>
"""
import sys, os
import numpy as np
from PIL import Image

d = sys.argv[1]
skins = sorted(x for x in os.listdir(d) if os.path.isdir(os.path.join(d, x)))
flagged, ok, nofr = [], 0, []
for s in skins:
    sd = os.path.join(d, s)
    fs = sorted(f for f in os.listdir(sd) if f.endswith(".png"))
    if not fs:
        nofr.append(s); continue
    issues = []
    means = []
    for f in fs:
        a = np.asarray(Image.open(os.path.join(sd, f)).convert("L")).astype(float)
        m, sd_, sat, blk = a.mean(), a.std(), (a >= 250).mean(), (a <= 8).mean()
        means.append(m)
        if sat >= 0.40: issues.append(f"BLOWN({f[:6]} {100*sat:.0f}%)")
        elif blk >= 0.85: issues.append(f"DARK({f[:6]} {100*blk:.0f}%)")
        elif sd_ < 3: issues.append(f"FLAT({f[:6]} std{sd_:.1f})")
    blanks = sum(1 for m in means if m < 2)
    if blanks == len(means): issues = [f"BLANK(all {len(means)})"]
    elif blanks: issues.insert(0, f"DROPOUT({blanks}/{len(means)})")
    if issues: flagged.append((s, issues))
    else: ok += 1

print(f"skins rendered : {len(skins)}")
print(f"clean          : {ok}")
print(f"no frames      : {len(nofr)}" + (f"  {nofr[:6]}" if nofr else ""))
print(f"FLAGGED        : {len(flagged)}")
for s, iss in flagged:
    print(f"   {s[:44]:44} {', '.join(iss[:3])}")
