#!/usr/bin/env python3
"""The nation of each place topic, from data (2026-10-03 night 8; the summary-count rule of night 7 mapped Chernobog to Yan).

For every served place topic of artifacts/topics/topics.jsonl, two sources in order:
  1. birthplace: operators of artifacts/entities/operator_attributes.jsonl whose birthplace is the place (or an alias);
     the most common of their nations other than "Rhodes Island" (an employer, not a nation), when it has more operators
     than "Rhodes Island" (Kazdel: 2 Columbia against 11 Rhodes Island, so it falls to the next source).
  2. co-occurrence: the story chunks of artifacts/p3b/chunks.jsonl (operator files, dossiers, summaries and topics left
     out) that name the place as a whole word; for each served nation topic (and each served topic whose name is a nation of the attributes table:
     Ursus is served as a race), the chunks among them that also name it,
     scored count x log(lift), lift = the share of place chunks naming the nation over its share of all chunks; at
     least 3 shared chunks and lift above 1.
None when neither gives a nation. No model, about a minute.

  python3 scripts/place_nation.py    -> artifacts/topics/place_nation.json  {place: {"nation", "basis", "chunks"}}
"""
import json, os, re, math, collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SKIP = {'archive', 'profile', 'summary', 'topic'}


def rx_for(forms):
    forms = sorted((f for f in forms if len(f) >= 3), key=len, reverse=True)
    return re.compile(r"(?<![\w'])(" + '|'.join(re.escape(f) for f in forms) + r")(?![\w'])") if forms else None


def main():
    rows = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/topics/topics.jsonl'))]
    rows = [r for r in rows if 'thin' not in r]
    places = [r for r in rows if r['kind'] == 'place']
    attrs = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/entities/operator_attributes.jsonl'))]
    # Nation topics, plus any served topic whose name is a nation of the attributes table (Ursus is served as a race).
    table = {a['nation'] for a in attrs if a.get('nation')} - {'Rhodes Island'}
    nations = [r for r in rows if r['kind'] == 'nation' or (r['kind'] != 'place' and r['topic'] in table)]
    texts = [c['text'] for c in map(json.loads, open(os.path.join(ROOT, 'artifacts/p3b/chunks.jsonl'))) if c['groupId'] not in SKIP]
    nrx = {n['topic']: rx_for({n['topic'], *n.get('aliases', [])}) for n in nations}
    nhits = {t: {i for i, x in enumerate(texts) if rx and rx.search(x)} for t, rx in nrx.items()}
    out = {}
    for p in places:
        forms = {p['topic'], *p.get('aliases', [])}
        here_born = [a['nation'] for a in attrs if a.get('birthplace') in forms and a.get('nation')]
        born = collections.Counter(n for n in here_born if n != 'Rhodes Island')
        if born and max(born.values()) > here_born.count('Rhodes Island'):
            nation, n = sorted(born.items(), key=lambda x: (-x[1], x[0]))[0]
            if nation not in nrx:
                nation = next((t for t in nrx if nation in {t, *next(r for r in nations if r['topic'] == t).get('aliases', [])}), nation)
            if nation in nrx:
                out[p['topic']] = {'nation': nation, 'basis': f'birthplace of {n} operator(s)', 'chunks': n}
                continue
        rx = rx_for(forms)
        here = {i for i, x in enumerate(texts) if rx and rx.search(x)}
        best = None
        for t, hs in nhits.items():
            if t == p['topic'] or not hs or not here:
                continue
            k = len(here & hs)
            lift = (k / len(here)) / (len(hs) / len(texts))
            if k >= 3 and lift > 1:
                s = k * math.log(lift)
                if best is None or s > best[0]:
                    best = (s, t, k, lift)
        if best:
            out[p['topic']] = {'nation': best[1], 'basis': f'named in {best[2]} of the {len(here)} story chunks naming the place (lift {best[3]:.1f})',
                               'chunks': best[2]}
    with open(os.path.join(ROOT, 'artifacts/topics/place_nation.json'), 'w') as f:
        json.dump(out, f, ensure_ascii=False, sort_keys=True, indent=0)
    print(f'{len(out)} of {len(places)} places mapped')
    for k in sorted(out):
        print(f"{k}\t{out[k]['nation']}\t{out[k]['basis']}")


if __name__ == '__main__':
    main()
