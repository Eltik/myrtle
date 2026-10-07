#!/usr/bin/env python3
"""EN release dates for the P4x sources the reading guide does not place (2026-10-07, spoiler horizon coverage).

Writes artifacts/chrono/source_dates.json: {"dates": {storyId: unix seconds}, "note": ...}. A date is when the source
was in the EN game at the latest, so a horizon that admits a source by this date never admits it before its release:
  - operator records (story_<x>_set_<n>): handbook_info_table StoryGetTime of the set;
  - operator files and voice lines (archive_/voice_<charId>): the operator's earliest record date, else the earliest
    non-initial module date (both upper bounds on the operator's EN release);
  - modules (module_<equipId>): uniequip_table UniEquipGetTime;
  - outfits (skin_<skinId with @ as _>): skin_table GetTime when set, else the operator's date;
  - game-text units naming a guide group (gametext_..._<groupId>_...): that group's release date.
Enemies, items, IS units, profiles, summaries and topics get no date and stay hidden under a horizon.

Usage: python3 scripts/source_dates.py [--gamedata ../assets/output/en/gamedata/excel]
"""
import argparse
import collections
import datetime
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def kv(rows):
    """The EN tables store maps as [{"key", "value"}]."""
    return {r["key"]: r["value"] for r in rows} if isinstance(rows, list) else rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--gamedata", default=os.path.join(ROOT, "..", "assets/output/en/gamedata/excel"))
    a = ap.parse_args()
    load = lambda n: json.load(open(os.path.join(a.gamedata, n + ".json")))

    hb = kv(load("handbook_info_table")["HandbookDict"])
    record = {}
    op_record = collections.defaultdict(list)
    for cid, v in hb.items():
        for s in v.get("HandbookAvgList") or []:
            t = s.get("StoryGetTime") or 0
            if t > 0:
                record[s["StorySetId"]] = t
                op_record[cid].append(t)
    equips = kv(load("uniequip_table")["EquipDict"])
    module = {k: v["UniEquipGetTime"] for k, v in equips.items() if (v.get("UniEquipGetTime") or 0) > 0}
    op_module = collections.defaultdict(list)
    for k, v in equips.items():
        if v.get("Type_") != "INITIAL" and (v.get("UniEquipGetTime") or 0) > 0:
            op_module[v["CharId"]].append(v["UniEquipGetTime"])
    op_date = {c: min(op_record.get(c) or op_module[c]) for c in set(op_record) | set(op_module)
               if op_record.get(c) or op_module.get(c)}
    skins = kv(load("skin_table")["CharSkins"])
    skin = {}
    for sid, v in skins.items():
        t = (v.get("DisplaySkin") or {}).get("GetTime") or 0
        skin[sid.replace("@", "_")] = t if t > 0 else op_date.get(v.get("CharId"))

    guide = json.load(open(os.path.join(ROOT, "artifacts/chrono/reading_guide.json")))["items"]
    group_date = {}
    placed = {i["groupId"] for i in guide}
    for i in guide:
        m = re.search(r"\d{4}-\d{2}-\d{2}", i["released"])
        if not m:  # "not dated in the game data": its game text stays undated
            continue
        d = m.group(0)
        group_date[i["groupId"]] = int(datetime.datetime.strptime(d, "%Y-%m-%d")
                                       .replace(tzinfo=datetime.timezone.utc).timestamp())
    groups_by_len = sorted(group_date, key=len, reverse=True)

    dates, kinds = {}, collections.Counter()
    seen = set()
    for line in open(os.path.join(ROOT, "artifacts/p4x/chunks.jsonl")):
        c = json.loads(line)
        sid, g = c["storyId"], c["groupId"]
        if sid in seen or g in placed:
            continue
        seen.add(sid)
        t = None
        if g.startswith("story_") and "_set_" in g:
            t = record.get(g)
        elif g in ("archive", "voice"):
            t = op_date.get(sid.split("_", 1)[1])
        elif g == "module":
            t = module.get(sid[len("module_"):])
        elif g == "skin":
            t = skin.get(sid[len("skin_"):])
        elif g == "gametext":
            name = sid[len("gametext_"):]
            hit = next((x for x in groups_by_len if re.search(r"(^|_)" + re.escape(x) + r"(_|$)", name)), None)
            t = group_date.get(hit)
        kind = "record" if g.startswith("story_") else g
        kinds[(kind, t is not None)] += 1
        if t:
            dates[sid] = t
    out = {"note": "EN release dates (unix s, an upper bound) of P4x sources the reading guide does not place; "
                   "scripts/source_dates.py", "dates": dict(sorted(dates.items()))}
    json.dump(out, open(os.path.join(ROOT, "artifacts/chrono/source_dates.json"), "w"), indent=0)
    for k in sorted({k for k, _ in kinds}):
        print(f"{k:10s} dated {kinds[(k, True)]:5d} undated {kinds[(k, False)]:5d}")
    print("sources dated", len(dates))


if __name__ == "__main__":
    main()
