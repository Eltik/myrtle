#!/usr/bin/env python3
"""Operator status from the Arknights wiki (arknights.wiki.gg), a reference for the dead-operator table.

Ian (2026-09-28): "you can also verify with the arknights wiki, which details the status of an
operator." Operator pages carry no status; each operator's `<Name>/Story` page opens with a
`{{Character infobox}}` whose `|status =` field holds it (Theresa: "Deceased {{Spoiler|...}}", Shu:
"Active"). Inline `{{Status|deceased}}` marks family members and is ignored. One request a second,
resumable. Writes eval/reference/wiki_status.jsonl (an eval reference, never served or built from): name, charId, page, status (`wiki_class`: first
word lower case, "mixed" for a living status with a death in its note, null when the field is
absent, which is most operators), statusRaw, fetched.
"""
import json, os, re, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT

OUT = os.path.join(ROOT, 'eval', 'reference', 'wiki_status.jsonl')  # eval reference only (2026-10-04)


def wikitext(page):
    return common.wiki_get_retry({'action': 'parse', 'page': page, 'prop': 'wikitext', 'format': 'json', 'redirects': 1}, page,
                                 parse=lambda t: t['parse']['wikitext']['*'] if 'parse' in t else None)


def wiki_class(raw):
    """deceased | alive | active | ... from the field's first word; "mixed" when a living status carries a
    death in its spoiler note (Beagle: "Alive {{Spoiler|(deceased as of Bolívar Diagnosed, ...)}}")."""
    if not raw:
        return None
    m = re.match(r'[A-Za-z]+', raw)
    first = m.group(0).lower() if m else raw.lower()
    if first != 'deceased' and re.search(r'deceased|\bdied\b|\bdead\b|\bkilled\b', raw, re.I):
        return 'mixed'
    return first


def main():
    ops = common.playable_operators()
    done = {json.loads(l)['charId'] for l in open(OUT)} if os.path.exists(OUT) else set()
    for cid, name in ops:
        if cid in done:
            continue
        page = f'{name}/Story'
        w = wikitext(page)
        raw = None
        if w:
            m = re.search(r'^\|\s*status\s*=\s*(.*)$', w, re.M)
            raw = m.group(1).strip() if m and m.group(1).strip() else None
        status = wiki_class(raw)
        common.append_jsonl(OUT, {'name': name, 'charId': cid, 'page': page if w else None, 'status': status,
                                  'statusRaw': raw, 'fetched': time.strftime('%Y-%m-%d')})
        time.sleep(1)
    R = [json.loads(l) for l in open(OUT)]
    from collections import Counter
    print(f'wiki: {len(R)} operators; no Story page {sum(r["page"] is None for r in R)}; status {dict(Counter(r["status"] for r in R))}')


if __name__ == '__main__':
    main()
