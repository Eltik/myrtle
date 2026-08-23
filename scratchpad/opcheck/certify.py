#!/usr/bin/env python3
"""Certify the DEPLOYED dynchar tree against a fresh full-corpus export.

Two independent failure classes, both of which have shipped to users before:
  STALE       deployed JSON differs from what the current exporter produces
  UNDER-STAGED a scene has far fewer textures than layers (the signature from
               `dynchar-under-staged-deployed-exports`: chyue_cfa#1 shipped ZERO
               scene layers, yu_nian 11 of 30)
"""
import json, os, sys, glob, hashlib

NEW = sys.argv[1]
DEP = sys.argv[2]

def h(p):
    return hashlib.sha256(open(p, 'rb').read()).hexdigest()

def scene_stats(p):
    try:
        d = json.load(open(p))
    except Exception as e:
        return None
    layers = d.get('layers') or []
    texs = set()
    for l in layers:
        t = l.get('tex')
        if isinstance(t, int):
            texs.add(t)
    return len(layers), len(texs)

new_skins = sorted(os.path.basename(x) for x in glob.glob(NEW + '/spine/DynIllust/*'))
dep_skins = sorted(os.path.basename(x) for x in glob.glob(DEP + '/*'))

print(f"exported {len(new_skins)} skins, deployed {len(dep_skins)}")
only_new = set(new_skins) - set(dep_skins)
only_dep = set(dep_skins) - set(new_skins)
if only_new: print("  NOT DEPLOYED:", sorted(only_new))
if only_dep: print("  DEPLOYED BUT NOT EXPORTED:", sorted(only_dep))

stale, missing, understaged, ok = [], [], [], 0
for s in new_skins:
    if s not in dep_skins:
        continue
    nf = sorted(x for x in glob.glob(f"{NEW}/spine/DynIllust/{s}/**/*", recursive=True) if os.path.isfile(x))
    for f in nf:
        b = os.path.relpath(f, f"{NEW}/spine/DynIllust/{s}")
        d = f"{DEP}/{s}/{b}"
        if not os.path.exists(d):
            missing.append(f"{s}/{b}")
            continue
        if h(f) != h(d):
            stale.append(f"{s}/{b}")
        else:
            ok += 1
    # under-staging check on the DEPLOYED scene jsons
    for d in glob.glob(f"{DEP}/{s}/*[[]scene[]].json"):
        st = scene_stats(d)
        if st is None:
            understaged.append((s, os.path.basename(d), 'UNREADABLE', ''))
            continue
        nl, nt = st
        if nl == 0:
            understaged.append((s, os.path.basename(d), nl, nt))
        elif nl >= 8 and nt * 3 < nl:
            understaged.append((s, os.path.basename(d), nl, nt))

print(f"\nidentical files: {ok}")
print(f"STALE  (deployed != fresh export): {len(stale)}")
for x in stale[:60]: print("   ", x)
if len(stale) > 60: print(f"    ... and {len(stale)-60} more")
print(f"MISSING from deploy: {len(missing)}")
for x in missing[:40]: print("   ", x)
print(f"UNDER-STAGED suspects (layers vs distinct textures): {len(understaged)}")
for s, b, nl, nt in understaged[:40]: print(f"    {s:<40} {b}  layers={nl} tex={nt}")
