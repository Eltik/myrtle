#!/usr/bin/env python3
"""Boss enemy handbook entries and the story stages they are fought in (2026-10-17, panel identity coverage).

Writes artifacts/chrono/enemy_intros.json, which panels.py reads for "Who's here" identity lines of story speakers who
are also boss enemies (Talulah, W, FrostNova, Crownslayer...). The handbook description is official game text that a
player sees once they have fought the enemy, so a panel may show it only after the first story stage that fields it:
  {"schema": 1, "note": ..., "inputs": {"enemyHandbook": sha16, "levels": <level files read>},
   "enemies": {name: [{"enemyId", "description", "stages": [{"level", "story", "after"}]}]}}
  - only EnemyLevel BOSS entries that are not HideInHandbook (generic NORMAL/ELITE names such as "Sniper", "Agent" or
    the drone "Frost" collide with story speakers);
  - a stage is a level file whose basename names a story of spoiler.jsonl (`<group>_<level>_beg|_end`); roguelike,
    hard and other levels with no story are never used, so an enemy fought only there gets no line;
  - `story` is the stage's _end story (the battle comes before it, after = false) or, when the stage has only a
    _beg story, that story with after = true: the line is visible from the NEXT gatePos on.
panels.py turns each stage into a gatePos with the shared gate order (gatePos of `story`, plus 1 when `after`) and
uses an enemy's earliest stage; this file holds story ids only, so it stays valid when the order changes.

Usage: python3 scripts/enemy_intros.py [--gamedata ../assets/output/en/gamedata]
"""
import argparse, hashlib, json, os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "artifacts", "chrono", "enemy_intros.json")
STAGE = re.compile(r"_(level_[a-z0-9_\-]+?)_(beg|end)$")


def kv(rows):
    """The EN tables store maps as [{"key", "value"}]."""
    return {r["key"]: r["value"] for r in rows} if isinstance(rows, list) else rows


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--gamedata", default=os.path.join(ROOT, "..", "assets", "output", "en", "gamedata"))
    ap.add_argument("--out", default=OUT)
    a = ap.parse_args()
    hb_path = os.path.join(a.gamedata, "excel", "enemy_handbook_table.json")
    if not os.path.exists(hb_path):
        sys.exit(f"enemy_intros: {hb_path} is missing (pass --gamedata)")
    hb = kv(json.load(open(hb_path, encoding="utf-8"))["EnemyData"])
    stories = {}
    for line in open(os.path.join(ROOT, "artifacts", "spoiler.jsonl"), encoding="utf-8"):
        if line.strip():
            sid = json.loads(line)["storyId"]
            m = STAGE.search(sid)
            if m:
                stories.setdefault(m.group(1), {})[m.group(2)] = sid
    stages, read = {}, 0
    for root, dirs, files in os.walk(os.path.join(a.gamedata, "levels")):
        dirs[:] = sorted(d for d in dirs if d not in ("roguelike", "enemydata"))
        for f in sorted(files):
            base = f[:-5].lower() if f.endswith(".json") else None
            if base not in stories:
                continue
            read += 1
            st = stories[base]
            stage = {"level": base, "story": st.get("end") or st["beg"], "after": "end" not in st}
            for r in json.load(open(os.path.join(root, f), encoding="utf-8")).get("EnemyDbRefs") or []:
                stages.setdefault(r["Id"], {})[base] = stage
    enemies = {}
    for eid in sorted(stages):
        v = hb.get(eid)
        if (not v or v.get("HideInHandbook") or v.get("EnemyLevel") != "BOSS"
                or not (v.get("Description") or "").strip()):
            continue
        enemies.setdefault(v["Name"], []).append({"enemyId": eid, "description": v["Description"].strip(),
                                                  "stages": [stages[eid][b] for b in sorted(stages[eid])]})
    doc = {"schema": 1,
           "note": "boss enemy handbook entries and the story stages that field them; scripts/enemy_intros.py",
           "inputs": {"enemyHandbook": hashlib.sha256(open(hb_path, "rb").read()).hexdigest()[:16], "levels": read},
           "enemies": enemies}
    tmp = a.out + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(doc, f, ensure_ascii=False, indent=1, sort_keys=True)
        f.write("\n")
    os.replace(tmp, a.out)
    print(f"enemy_intros: {len(enemies)} boss names ({sum(len(v) for v in enemies.values())} entries) from {read} "
          f"story level files -> {a.out}")


if __name__ == "__main__":
    main()
