#!/usr/bin/env python3
"""Judge P2 summaries: faithfulness, a false-positive control, and coverage.

Needs a llama-server on $SERVER running the JUDGE model (a different family
from the generator). Reads artifacts/p2/{stories,groups}.jsonl, writes
artifacts/p2/judged.jsonl (resumable) and prints totals per kind and method.

  faithfulness  each summary sentence is judged against the top 5 BM25 chunks
                of its source (story, or every story of the group) plus their
                neighbours. Sentences are split deterministically, so there is
                no model decomposition to fail (a grammar-bound decomposer
                produced fragments in the pilot).
  control       for group summaries, 10 sentences from another group's summary
                judged against this group's evidence; every "true" is a false
                positive of the judge.
  beats         for group summaries, each story's official synopsis judged as
                an event the summary does or does not cover.
  operators     of the 5 playable operators who speak in the most chunks of the
                source, how many the summary names (NPC leads are not counted).

Sample: SAMPLE_GROUPS multi-story groups (every method each) and SAMPLE_STORIES
story summaries, both seeded.
"""
import collections, json, math, os, random, re, sys, threading, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
P2 = os.path.join(ROOT, 'artifacts', 'p2')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
SAMPLE_GROUPS = int(os.environ.get('SAMPLE_GROUPS', '15'))
SAMPLE_STORIES = int(os.environ.get('SAMPLE_STORIES', '60'))
GAMEDATA = os.environ.get('GAMEDATA', os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata'))

SUPPORT_SYS = ("You check one claim about an Arknights story against evidence from the story script. Answer true only if "
               "the evidence states or clearly implies the claim. Answer false if the evidence contradicts it or does not "
               "mention it. Answer with true or false only.")
BEAT_SYS = ("You check whether a summary covers one event. Answer true if the summary states or clearly implies the "
            "event, false otherwise. Answer with true or false only.")
G_BOOL = 'root ::= "true" | "false"\n'
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bLt\.)(?<!\bSgt\.)(?<!\bJr\.)(?<!\bSr\.)(?<!\bMt\.)(?<!\bMs\.)(?<!\bNo\.)(?<!\bVol\.)(?<!\bvs\.)'


def verdict(system, user):
    body = json.dumps({'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
                       'temperature': 0, 'seed': 1, 'max_tokens': 3, 'grammar': G_BOOL}).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=600) as r:
                return json.load(r)['choices'][0]['message']['content'].strip() == 'true'
        except Exception as e:
            err = e
            time.sleep(5 * (attempt + 1))
    raise err


def sentences(t):
    parts = re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'])', ' '.join(t.split()))
    return [x.strip() for x in parts if len(x.strip()) > 20]


def bm25_top(query, docs, k):
    tok = lambda s: re.findall(r"[a-z0-9']+", s.lower())
    D = [tok(d) for d in docs]; N = len(D); avg = sum(map(len, D)) / max(N, 1)
    df = collections.Counter(w for d in D for w in set(d))
    q = set(tok(query)); sc = []
    for i, d in enumerate(D):
        tf = collections.Counter(d); s = 0.0
        for w in q:
            if w in tf:
                idf = math.log(1 + (N - df[w] + 0.5) / (df[w] + 0.5))
                s += idf * tf[w] * 2.2 / (tf[w] + 1.2 * (0.25 + 0.75 * len(d) / avg))
        sc.append((s, i))
    return [i for _, i in sorted(sc, reverse=True)[:k]]


def par(items, fn, n=2):
    it = iter(items); lock = threading.Lock()
    def w():
        while True:
            with lock:
                x = next(it, None)
            if x is None:
                return
            fn(x)
    th = [threading.Thread(target=w) for _ in range(n)]
    [t.start() for t in th]; [t.join() for t in th]


def read_jsonl(p):
    return [json.loads(l) for l in open(p)] if os.path.exists(p) else []


def main():
    chunks = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts', 'chunks.jsonl'))]
    by_story = collections.defaultdict(list)
    for c in chunks:
        by_story[c['storyId']].append(c)
    for v in by_story.values():
        v.sort(key=lambda c: c['ordinal'])
    table = json.load(open(os.path.join(GAMEDATA, 'excel', 'character_table.json')))['Characters']
    operators = {v['value'].get('Name') for v in table if v['value'].get('Name')}
    stories = {r['storyId']: r for r in read_jsonl(os.path.join(P2, 'stories.jsonl'))}
    groups = read_jsonl(os.path.join(P2, 'groups.jsonl'))
    gids = sorted({g['groupId'] for g in groups})
    random.Random(25).shuffle(gids)
    pick_g = set(gids[:SAMPLE_GROUPS])
    sids = sorted(stories); random.Random(26).shuffle(sids)
    units = [{'kind': 'group', 'id': f"{g['groupId']}/{g['method']}", 'method': g['method'], 'text': g['summary'],
              'storyIds': g['storyIds'], 'groupId': g['groupId']} for g in groups if g['groupId'] in pick_g]
    units += [{'kind': 'story', 'id': s, 'method': 'story', 'text': stories[s]['summary'], 'storyIds': [s],
               'groupId': stories[s]['groupId']} for s in sids[:SAMPLE_STORIES]]
    b_text = {g['groupId']: g['summary'] for g in groups if g['method'] == 'B'}
    out = os.path.join(P2, 'judged.jsonl')
    done = {r['id'] for r in read_jsonl(out)}
    lock = threading.Lock()
    print(f'judge: {len(units)} units, {len(done)} done', flush=True)
    for u in units:
        if u['id'] in done:
            continue
        pool = [c for s in u['storyIds'] for c in by_story[s]]
        claims = sentences(u['text'])
        controls = []
        if u['kind'] == 'group' and u['method'] == 'B':
            others = [t for g, t in sorted(b_text.items()) if g != u['groupId']]
            random.Random(u['groupId']).shuffle(others)
            controls = [x for t in others[:3] for x in sentences(t)][:10]
        faith, ctrl = [], []
        def check(item):
            cl, is_ctrl = item
            ev = set()
            for i in bm25_top(cl, [c['text'] for c in pool], 5):
                for j in (i - 1, i, i + 1):
                    if 0 <= j < len(pool) and pool[j]['storyId'] == pool[i]['storyId']:
                        ev.add(j)
            v = verdict(SUPPORT_SYS, 'EVIDENCE:\n' + '\n---\n'.join(pool[j]['text'] for j in sorted(ev)) + f'\n\nCLAIM: {cl}')
            with lock:
                (ctrl if is_ctrl else faith).append((cl, v))
        par([(c, False) for c in claims] + [(c, True) for c in controls], check)
        beats = []
        if u['kind'] == 'group':
            def beat(s):
                syn = stories[s]['officialSynopsis'] if s in stories else ''
                if syn:
                    v = verdict(BEAT_SYS, f"SUMMARY:\n{u['text']}\n\nEVENT: {syn}")
                    with lock:
                        beats.append(v)
            par(u['storyIds'], beat)
        spk = collections.Counter(p for c in pool for p in set(c['speakers']) if p in operators)
        top = [p for p, _ in spk.most_common(5)]
        row = {'id': u['id'], 'kind': u['kind'], 'method': u['method'], 'words': len(u['text'].split()),
               'sentences': len(claims), 'supported': sum(v for _, v in faith),
               'unsupported': [c for c, v in faith if not v], 'controls': len(ctrl),
               'controlFalsePositives': sum(v for _, v in ctrl), 'beats': len(beats), 'beatsCovered': sum(beats),
               'topOperators': top, 'operatorsNamed': sum(p.lower() in u['text'].lower() for p in top)}
        with open(out, 'a') as f:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
        print(f"{u['id']}: faith {row['supported']}/{row['sentences']}, beats {row['beatsCovered']}/{row['beats']}, "
              f"operators {row['operatorsNamed']}/{len(top)}, control FP {row['controlFalsePositives']}/{row['controls']}", flush=True)
    rows = read_jsonl(out)
    agg = collections.defaultdict(lambda: collections.Counter())
    for r in rows:
        a = agg[r['method']]
        for k in ('sentences', 'supported', 'beats', 'beatsCovered', 'controls', 'controlFalsePositives', 'operatorsNamed', 'words'):
            a[k] += r[k]
        a['ops'] += len(r['topOperators']); a['n'] += 1
    for m, a in sorted(agg.items()):
        print(f"{m}: n {a['n']}, faithful {a['supported']}/{a['sentences']} = {a['supported']/max(a['sentences'],1):.3f}, "
              f"beats {a['beatsCovered']}/{a['beats']} = {a['beatsCovered']/max(a['beats'],1):.3f}, "
              f"operators {a['operatorsNamed']}/{a['ops']}, control FP {a['controlFalsePositives']}/{a['controls']}, "
              f"mean words {a['words']/a['n']:.0f}", flush=True)


if __name__ == '__main__':
    main()
