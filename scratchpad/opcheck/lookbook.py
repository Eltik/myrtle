#!/usr/bin/env python3
"""Make the FLOT lookbook ADDRESSABLE, and cross-check our dynchar census against gamedata.

WHY. The lookbook grid is the only surface that reaches a skin nobody owns, and its tiles are
labelled by SKIN DISPLAY NAME while everything on our side is keyed by asset id. Walking it by eye
costs a screenshot per scroll and has already burned most of one session. `skin_table_cur.json`
carries the whole mapping, so arrive knowing the brand grid and the tile caption.

  lookbook.py                 census + the disk/table diff
  lookbook.py <key|skinid>..  addressing for named subjects (all8.sh keys work)

ROUTE, for the record, from `capture_l2d.sh::navigate_to_lookbook`:
  menu -> Store (778,375 in 1170x540 base) -> Outfit Store tab (585,80)
  -> swipe the strip LEFT up to 30x, sw (1000,280)->(150,280), stop when the screen hash repeats
  -> the FLOT tile is the LAST cell, past the last purchasable outfit -> tap (1076,340)
🚨 Three swipes is not enough and stopping there reads as "no catalogue exists". It took thirty.
The lookbook then offers `Show Unowned` (checked by default) and `Sort by Release` / `Sort by
Brand`; sort by brand and use `brand` below to pick the grid, `name` to pick the tile.

⚠️ The scratch pull `skin_table_cur.json` beside this script IS STALE and is NOT used: 69
dynIllust skins against this table's 82, missing whitw2 `sale#15` and cetsyr `epoque#50` entirely,
which is why `cet` briefly looked unaddressable. Read the pipeline's table, never the pull.

🔑 THE CROSS-CHECK, and it lands in our favour. On every skin both sources carry, 69 of 69, the
table's `dynEntranceId` agrees exactly with whether disk ships a `_Start.skel`. Zero disagreements.
So our entrance classification is confirmed against the field the game itself keys off, and the
only discrepancy is the pull being behind. ⚠️ The figures "82 dynchar skins, 70 non-entrance" that
appear in a `SceneIllust.tsx` comment and in the notes are STALE: disk now reads 86 and 13, so 73
non-entrance.

⛔ The 24 OP / 21 OP price tier could NOT be checked offline. No pulled table carries a skin price:
`shop_client_table.json`'s `shopGPDataDict` holds pack goods, not outfits, and `skin_table_cur.json`
has no price field. The tier remains an in-UI check only. `dynEntranceId` above is the same
discriminator read from data instead, which is why the offline census is worth more than the tiles.
"""
import json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DYN = "/Users/eltik/Documents/Coding/myrtle/assets/output/en/spine/DynIllust"
# 🚨 THE REPO'S OWN EXTRACTED TABLE, not the scratch pull beside this script.
# `skin_table_cur.json` is a hand pull from a client and it is BEHIND: 69 dynIllust skins where
# this one has 82, missing the two newest entrance skins outright. This file is produced by the
# asset pipeline and is current, so there is nothing to "refresh" and no pull to improvise.
# Shape differs: `CharSkins` is a LIST of {key, value} and the fields are PascalCase.
TABLE = "/Users/eltik/Documents/Coding/myrtle/assets/output/en/gamedata/excel/skin_table.json"

# all8.sh's keys, so a caller can say `wis` rather than the directory name.
KEYS = {
    "ska": "char_1012_skadi2_iteration#2", "exc": "char_1032_excu2_sale#12",
    "cel": "char_245_cello_sale#12", "mly": "char_4064_mlynar_epoque#28",
    "mue": "char_249_mlyss_boc#8", "eyja": "char_1016_agoat2_epoque#34",
    "cet": "char_4134_cetsyr_epoque#50", "wis": "char_1035_wisdel_sale#14",
    "whitw2": "char_1038_whitw2_sale#15", "chyue": "char_2024_chyue_cfa#1",
    "fugue": "char_113_cqbw_epoque#7", "kalts": "char_003_kalts_boc#6",
    "excunew": "char_1032_excu2_sale#12",
}


def dirkey(skin_id):
    """`char_1035_wisdel@sale#14` -> `char_1035_wisdel_sale#14`, the directory name.

    The `@` separates char from skin group. A bare `#2` (the E2 art) is a suffix on the CHAR id,
    not a group, and becomes `_2` on disk. A `#2` inside a group name like `iteration#2` is part of
    the group and must be left alone, which a blind replace gets wrong and which cost one bad diff.
    """
    if "@" in skin_id:
        c, g = skin_id.split("@", 1)
        return f"{c}_{g}"
    return skin_id[:-2] + "_2" if skin_id.endswith("#2") else skin_id


def load():
    cs = [e["value"] for e in json.load(open(TABLE))["CharSkins"]]
    tab = {}
    for s in cs:
        if s.get("DynIllustId"):
            d = s["DisplaySkin"]
            tab[dirkey(s["SkinId"]).lower()] = dict(
                skinId=s["SkinId"], name=d.get("SkinName"), brand=d.get("SkinGroupName"),
                year=d.get("OnYear"), period=d.get("OnPeriod"), sortId=d.get("SortId"),
                entrance=bool(s.get("DynEntranceId")))
    disk = {}
    for d in sorted(os.listdir(DYN)):
        p = os.path.join(DYN, d)
        if os.path.isdir(p) and any(f.startswith("dyn_illust_") and f.endswith(".skel") for f in os.listdir(p)):
            disk[d.lower()] = dict(dir=d, start=any("_Start.skel" in f for f in os.listdir(p)))
    return tab, disk


def main():
    tab, disk = load()
    args = sys.argv[1:]
    if args:
        print(f"{'subject':30s} {'brand grid':26s} {'tile caption':34s} {'entrance':9s} year/period")
        for a in args:
            d = KEYS.get(a, a)
            t = tab.get(d.lower())
            on_disk = disk.get(d.lower())
            if not t:
                miss = "not in the pulled table, find it by eye" if on_disk else "unknown skin"
                print(f"{a:30s} {miss}")
                continue
            print(f"{a:30s} {str(t['brand'])[:24]:26s} {str(t['name'])[:32]:34s} "
                  f"{('YES' if t['entrance'] else 'no'):9s} y{t['year']} p{t['period']}")
        return
    both = set(tab) & set(disk)
    dis = [k for k in both if tab[k]["entrance"] != disk[k]["start"]]
    print(f"disk   {len(disk):3d} dynchar directories, {sum(v['start'] for v in disk.values()):3d} shipping a _Start.skel")
    print(f"table  {len(tab):3d} skins with dynIllustId,  {sum(v['entrance'] for v in tab.values()):3d} with a dynEntranceId")
    print(f"paired {len(both):3d}, entrance-flag disagreements {len(dis)}")
    od = sorted(set(disk) - set(tab))
    print(f"\nON DISK, ABSENT FROM GAMEDATA ({len(od)}), exported art the EN table does not list as skins:")
    for k in od:
        print(f"   {disk[k]['dir']:40s} _Start={disk[k]['start']}")
    ot = sorted(set(tab) - set(disk))
    print(f"\nIN TABLE, ABSENT FROM DISK ({len(ot)}):")
    for k in ot:
        print(f"   {tab[k]['skinId']:40s} entrance={tab[k]['entrance']}")


main()
