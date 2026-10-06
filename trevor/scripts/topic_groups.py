#!/usr/bin/env python3
"""The story groups that talk most about each topic (2026-10-03, Ian's items 3 and 4: "Who are the Sarkaz?" and "What is
Sami guarding?" were answered from one topic summary of 14 passages and one search).

For every served topic of artifacts/topics/topics.jsonl (its name, aliases and merged names of 3 or more characters),
count the story chunks of each group in artifacts/p3b/chunks.jsonl (operator files, dossiers, summaries and topics left
out) that name it as a whole word (case-sensitive for a capitalized form, so "Sami" and not "sami"), keep the groups that
have a P2 event summary (artifacts/p2/groups.jsonl), and write the 5 with the most such chunks, ties by group id. No
model, seconds. `ask` serves the summaries of the first 3 after the topic summary (`--no-topic-events` turns it off).

  python3 scripts/topic_groups.py    -> artifacts/topics/topic_groups.json
"""
import json, os, re, collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SKIP = {'archive', 'profile', 'summary', 'topic'}


def main():
    have = {json.loads(l)['groupId'] for l in open(os.path.join(ROOT, 'artifacts/p2/groups.jsonl'))}
    rows = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/topics/topics.jsonl'))]
    rows = [r for r in rows if 'thin' not in r]
    texts = collections.defaultdict(list)
    for l in open(os.path.join(ROOT, 'artifacts/p3b/chunks.jsonl')):
        c = json.loads(l)
        if c['groupId'] not in SKIP and c['groupId'] in have:
            texts[c['groupId']].append(c['text'])
    out = {}
    for r in rows:
        forms = {r['topic'], *r.get('aliases', []), *r.get('names', [])}
        forms = sorted((f for f in forms if len(f) >= 3), key=len, reverse=True)
        if not forms:
            continue
        pats = [re.escape(f) if f[0].isupper() else '(?i:' + re.escape(f) + ')' for f in forms]
        rx = re.compile(r"(?<![\w'])(" + '|'.join(pats) + r")(?![\w'])")
        counts = {g: sum(1 for t in ts if rx.search(t)) for g, ts in texts.items()}
        top = sorted(((n, g) for g, n in counts.items() if n), key=lambda x: (-x[0], x[1]))[:5]
        out[r['topic']] = [{'groupId': g, 'chunks': n, 'of': len(texts[g])} for n, g in top]
    with open(os.path.join(ROOT, 'artifacts/topics/topic_groups.json'), 'w') as f:
        json.dump(out, f, ensure_ascii=False, sort_keys=True, indent=0)
    print(f'{len(out)} topics -> artifacts/topics/topic_groups.json')
    for t in ('Sarkaz', 'Sami', 'Dossoles', 'Bolívar', 'Khaganquest'):
        print(t, [(x['groupId'], x['chunks']) for x in out.get(t, [])])


if __name__ == '__main__':
    main()
