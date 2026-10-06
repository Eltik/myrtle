#!/usr/bin/env python3
"""Score `ask` on real player questions, per category (design/trevor-questions.md section 6).

Real questions have no reference answers, so this measures behavior, not correctness:
  responds   judge: the answer addresses the question with relevant content (not a refusal, not off topic)
  faithful   judge: each answer sentence is supported by the passages `ask` was given
  handles    for opinion, visual, cross-media and canon questions (M, Q, S, K, and any labelled opinion,
             external or art): the answer does not invent a definitive answer and says what the text does
             or does not cover
Stages:
  sample   stratified sample of eval/reddit-lore-questions.all.jsonl (no X or P), at least MIN per
           category, N in total -> eval/real-probe.jsonl
  judge    judge model on $SERVER over the `ask --batch` output -> <answers>.judged.jsonl, then a table
"""
import collections, json, os, random, re, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3a'))
N = int(os.environ.get('N', '120'))
MIN = int(os.environ.get('MIN', '5'))
G_BOOL = 'root ::= "true" | "false"\n'
RESPONDS_SYS = ("You read a question about the Arknights story and an answer. Answer true if the answer addresses the question "
                "with relevant content, even partly. Answer false if it only says it cannot find the answer, or talks about "
                "something else. Answer true or false only.")
HANDLES_SYS = ("You read a question that asks for an opinion, a ranking, a canon ruling, art details or something outside the "
               "game's story text, and an answer. Answer true if the answer avoids stating an invented or definitive verdict "
               "and makes clear what the story text shows or does not cover. Answer false if it presents a guess, ranking or "
               "canon ruling as fact, or answers from unrelated material without saying so. Answer true or false only.")
SUPPORT_SYS = ("You check one sentence of an answer against the passages it was based on. Answer true only if the passages "
               "state or clearly imply it. Answer false otherwise. Answer true or false only.")
POLICY = {'M', 'Q', 'S', 'K'}


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
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def stage_sample():
    R = [json.loads(l) for l in open(os.path.join(ROOT, 'eval', 'reddit-lore-questions.all.jsonl'))]
    R = [r for r in R if r['category'] not in ('X', 'P') and 'duplicate' not in (r.get('note') or '')]
    rng = random.Random(5)
    by = collections.defaultdict(list)
    for r in R:
        by[r['category']].append(r)
    for v in by.values():
        rng.shuffle(v)
    # At least MIN per category (all of a smaller one), the rest in proportion to the real mix.
    pick = [r for c in sorted(by) for r in by[c][:MIN]]
    rest = [r for c in sorted(by) for r in by[c][MIN:]]
    rng.shuffle(rest)
    pick += rest[:max(0, N - len(pick))]
    with open(os.path.join(ROOT, 'eval', 'real-probe.jsonl'), 'w') as f:
        for i, r in enumerate(pick):
            f.write(json.dumps({'qid': f'r{i:03d}', 'question': r['q'], 'category': r['category'],
                                'answerable': r['answerable'], 'url': r['url']}, ensure_ascii=False) + '\n')
    print(f'sample: {len(pick)} questions {dict(sorted(collections.Counter(r["category"] for r in pick).items()))}')


def sentences(t):
    t = re.sub(r'\s*\[[\d,;\s]+\]', '', ' '.join(t.split()))
    return [x.strip() for x in re.split(r'(?<!\bMr\.)(?<!\bDr\.)(?<=[.!?])\s+(?=[A-Z"\'*])', t) if len(x.strip()) > 20]


def stage_judge(ans_path):
    chunks = {json.loads(l)['chunkId']: json.loads(l) for l in open(os.path.join(CORPUS, 'chunks.jsonl'))}
    # The topic tool's summary is passage 'topic:<name>' (ask --router model); the judge sees it as ask did, citations stripped.
    for t in map(json.loads, open(os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl'))):
        chunks[f"topic:{t['topic']}"] = {'text': re.sub(r'\s*\[[\d,;\s]+\]', '', t.get('summaryV2') or t.get('summary') or '')}  # summaryV2: --lore v2's text
    # A new dossier (`ask --lore v2`, 2026-10-02) is passage 'dossier:<name>', shown as ask shows it.
    dp = os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl')
    for d in (map(json.loads, open(dp)) if os.path.exists(dp) else []):
        aka = [x for x in d.get('names') or [] if x != d['name']]
        head = f"Character profile: {d['name']}" + (f" (also known as {', '.join(aka)})" if aka else '')
        chunks[f"dossier:{d['name']}"] = {'text': f"{head}\n{d['dossier']}"}
    # The overview tool's summaries are passages 'overview:<id>' (2026-10-01), shown to the judge as ask showed them.
    op = os.path.join(ROOT, 'artifacts', 'overview', 'overview.jsonl')
    for t in (map(json.loads, open(op)) if os.path.exists(op) else []):
        chunks[f"overview:{t['id']}"] = {'text': re.sub(r'\s*\[[\d,;\s]+\]', '', t.get('summary') or '')}
    # PROBE: another question file in the same format (qid, question, category, answerable); default the real probe.
    Q = {json.loads(l)['qid']: json.loads(l) for l in open(os.path.join(ROOT, os.environ.get('PROBE', 'eval/real-probe.jsonl')))}
    out = ans_path.replace('.jsonl', '.judged.jsonl')
    done = {json.loads(l)['qid'] for l in open(out)} if os.path.exists(out) else set()
    for a in map(json.loads, open(ans_path)):
        if a['qid'] in done:
            continue
        q = Q[a['qid']]
        routed = not a['passages']
        row = {'qid': a['qid'], 'category': q['category'], 'answerable': q['answerable'], 'routed': routed,
               'responds': verdict(RESPONDS_SYS, f"QUESTION: {q['question']}\n\nANSWER: {a['answer']}")}
        if q['category'] in POLICY or q['answerable'] in ('opinion', 'external', 'art'):
            row['handles'] = verdict(HANDLES_SYS, f"QUESTION: {q['question']}\n\nANSWER: {a['answer']}")
        gen = a.get('generated') or {}  # --lore v2's new dossier and game-data passages, as ask gave them
        given = '\n---\n'.join(gen[i] if i in gen else chunks[i]['text'] for i in a['passages'] if i in chunks or i in gen)
        sents = [] if routed else sentences(a['answer'])
        sup = [verdict(SUPPORT_SYS, f'PASSAGES:\n{given}\n\nSENTENCE: {s}') for s in sents]
        row.update({'sentences': len(sents), 'supported': sum(sup)})
        with open(out, 'a') as f:
            f.write(json.dumps(row) + '\n')
    J = [json.loads(l) for l in open(out)]
    by = collections.defaultdict(list)
    for r in J:
        by[r['category']].append(r)
    print(f"{'cat':4}{'n':>4}{'responds':>10}{'handles':>10}{'faithful':>10}{'routed':>8}")
    for c in sorted(by, key=lambda c: -len(by[c])):
        v = by[c]; h = [r['handles'] for r in v if 'handles' in r]
        s = sum(r['supported'] for r in v); t = sum(r['sentences'] for r in v)
        print(f"{c:4}{len(v):>4}{sum(r['responds'] for r in v) / len(v):>10.2f}"
              f"{(sum(h) / len(h)) if h else float('nan'):>10.2f}{s / max(t, 1):>10.2f}{sum(r['routed'] for r in v):>8}")
    s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J); h = [r['handles'] for r in J if 'handles' in r]
    print(f"all {len(J)}: responds {sum(r['responds'] for r in J) / len(J):.3f}; handles {sum(h)}/{len(h)}; "
          f"faithful {s}/{t} = {s / max(t, 1):.3f}; routed to a table {sum(r['routed'] for r in J)}")


if __name__ == '__main__':
    {'sample': lambda: stage_sample(), 'judge': lambda: stage_judge(sys.argv[2])}[sys.argv[1]]()
