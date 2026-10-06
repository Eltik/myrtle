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
import json, os, re, time, urllib.parse, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GD = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel')
OUT = os.path.join(ROOT, 'eval', 'reference', 'wiki_status.jsonl')  # eval reference only (2026-10-04)
API = 'https://arknights.wiki.gg/api.php'


def wikitext(page):
    q = urllib.parse.urlencode({'action': 'parse', 'page': page, 'prop': 'wikitext', 'format': 'json', 'redirects': 1})
    req = urllib.request.Request(f'{API}?{q}', headers={'User-Agent': 'trevor-research/0.1'})
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                t = json.load(r)
            return t['parse']['wikitext']['*'] if 'parse' in t else None
        except Exception:
            time.sleep(5 * (attempt + 1))
    raise RuntimeError(f'wiki unreachable for {page}')


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
    ct = json.load(open(os.path.join(GD, 'character_table.json')))['Characters']
    ops = [(c['key'], c['value']['Name']) for c in ct
           if c['value'].get('Profession') not in ('TOKEN', 'TRAP') and not c['value'].get('IsNotObtainable')]
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
        with open(OUT, 'a') as f:
            f.write(json.dumps({'name': name, 'charId': cid, 'page': page if w else None, 'status': status,
                                'statusRaw': raw, 'fetched': time.strftime('%Y-%m-%d')}, ensure_ascii=False) + '\n')
        time.sleep(1)
    R = [json.loads(l) for l in open(OUT)]
    from collections import Counter
    print(f'wiki: {len(R)} operators; no Story page {sum(r["page"] is None for r in R)}; status {dict(Counter(r["status"] for r in R))}')


if __name__ == '__main__':
    main()
