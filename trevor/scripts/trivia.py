#!/usr/bin/env python3
"""Held-out character-trivia questions in Ian's style (eval/ian-questions.jsonl is the pattern).

Ian's questions (2026-09-30) ask about one character's favorite food, who they like, their personality, marriage and
family, embarrassing moments, hobbies and habits, casually and with short names. The gold set asks almost none of
these, so this builds a set that does, from facts Trevor's corpus holds, with the question worded away from the source
text so vocabulary gaps show (Ian's "pee" against the story's "piss").

  sample   ~160 candidate chunks, seeded, at most one per operator, spread over operator archives, operator records,
           voice lines, dossiers and story scenes; gold v2 stories are left out -> artifacts/trivia/sample.jsonl
  gen      Gemma on $SERVER: one trivia fact per chunk (or skip), a casual question that does not reuse the chunk's
           distinctive words, and a short reference answer -> artifacts/trivia/gen.jsonl (resumable)
  verify   Qwen on $SERVER: keep a row only if the chunk supports the reference answer to the question
           -> artifacts/trivia/verify.jsonl (resumable), then eval/trivia.jsonl
           {qid, question, expected, chunkId, kind, operator, source}

  judge ANSWERS     Qwen: correct against `expected` plus the source chunk, declined, and faithfulness per
                    sentence, with answer-eval.py's prompts -> ANSWERS.judged.jsonl, totals by kind and source
  diagnose ANSWERS  no model: for every miss, the source chunk's rank for the raw question in P3b and P4 (top 8,
                    40, 100 or beyond), whether it reached the passages, whether the answer came from a table, and
                    whether the question names its operator -> ANSWERS.diag.jsonl and a breakdown by cause

Env: CORPUS (default artifacts/p4), N (sample size, default 160), SEED (default 20260930), SERVER.
"""
import collections, json, os, random, re, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p4'))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
N = int(os.environ.get('N', '160'))
SEED = int(os.environ.get('SEED', '20260930'))
OUT = os.path.join(ROOT, 'artifacts', 'trivia')
KINDS = ['food', 'relationship', 'personality', 'family', 'embarrassing', 'hobby', 'habit', 'other']
# Share of the sample per source (a trade: operator files and records hold most personal trivia).
MIX = {'archive': 0.30, 'record': 0.20, 'voice': 0.20, 'profile': 0.10, 'scene': 0.20}


def chunks():
    for l in open(os.path.join(CORPUS, 'chunks.jsonl')):
        yield json.loads(l)


def source_of(c):
    g, sid = c['groupId'], c['storyId']
    if g in ('archive', 'voice', 'profile'):
        return g
    if sid.startswith('story_') and '_set_' in sid:
        return 'record'
    if g.startswith(('act', 'main', '1stact')):
        return 'scene'
    return None


def stage_sample():
    ops = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/entities/operator_attributes.jsonl'))]
    code_of = {o['charId'].split('_', 2)[2]: o['name'] for o in ops if o.get('charId')}
    by_name = {o['name'].lower(): o['name'] for o in ops}
    gold_stories = {a['story_id'] for l in open(os.path.join(ROOT, 'eval/goldset_v2.jsonl'))
                    for a in json.loads(l).get('anchors', [])}
    pools = collections.defaultdict(list)
    for c in chunks():
        src = source_of(c)
        if src is None or c['storyId'] in gold_stories or len(c['text']) < 300:
            continue
        sid = c['storyId']
        op = None
        if src in ('archive', 'voice'):
            op = code_of.get(sid.split('_', 3)[3] if sid.count('_') >= 3 else '')
        elif src == 'record':
            op = code_of.get(sid.split('_')[1])
        elif src == 'profile':
            first = c['text'].split('\n', 1)[0].removeprefix('Character profile: ').split(' (also known as')[0]
            op = by_name.get(first.lower())
        else:
            # A scene counts for the operator who speaks most in it (at least 4 lines).
            sp = collections.Counter(m.group(1) for m in re.finditer(r'(?:^|\n| )([A-Z][\w\' .-]{1,30}): ', c['text']))
            top = [(n, k) for n, k in sp.most_common(3) if n.lower() in by_name and k >= 4]
            op = by_name[top[0][0].lower()] if top else None
        if op:
            pools[src].append((op, c['chunkId'], c['text']))
    rng = random.Random(SEED)
    for p in pools.values():
        rng.shuffle(p)
    used, rows = set(), []
    for src, share in MIX.items():
        want = round(N * share)
        for op, cid, text in pools[src]:
            if want == 0:
                break
            if op in used:
                continue
            used.add(op)
            rows.append({'operator': op, 'source': src, 'chunkId': cid, 'text': text})
            want -= 1
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, 'sample.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    print(f"sample: {len(rows)} chunks, {len(used)} operators; per source "
          f"{dict(collections.Counter(r['source'] for r in rows))}; pools {{k: len(v) for k, v in pools.items()}}"
          .replace('{k: len(v) for k, v in pools.items()}', str({k: len(v) for k, v in pools.items()})))


def chat(system, user, schema=None, max_tokens=300):
    body = {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
            'temperature': 0, 'seed': 1, 'max_tokens': max_tokens,
            'chat_template_kwargs': {'enable_thinking': False}}
    if schema:
        body['json_schema'] = schema
    data = json.dumps(body).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', data, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=600) as r:
                return json.load(r)['choices'][0]['message']['content'].strip()
        except Exception as e:
            err = e
            time.sleep(5 * (attempt + 1))
    raise err


GEN_SYS = (
    "You write trivia questions about Arknights characters for testing a lore assistant. You get one passage and the "
    "operator it is about. Find ONE personal trivia fact about that operator that the passage states plainly: a "
    "favorite food or drink, who they like, love or dislike, a personality trait, their family or marriage, an "
    "embarrassing or funny moment, a hobby, or a habit. If the passage states no such fact about them, set skip to "
    "true. Otherwise write the question the way a player would casually ask it on Discord: short, informal, sometimes "
    "lowercase, possibly using a common short form of the operator's name. The question must NOT reuse the passage's "
    "distinctive words for the fact: use everyday words or synonyms a player would use instead (if the passage says "
    "'piss', ask about peeing; if it says 'confectionery', ask about sweets). Do not hint at the answer. The expected "
    "answer is one or two short sentences stating the fact as the passage gives it.")
GEN_SCHEMA = {'type': 'object', 'properties': {
    'skip': {'type': 'boolean'},
    'kind': {'type': 'string', 'enum': KINDS},
    'fact': {'type': 'string', 'maxLength': 300},
    'question': {'type': 'string', 'maxLength': 200},
    'expected': {'type': 'string', 'maxLength': 300}},
    'required': ['skip', 'kind', 'fact', 'question', 'expected']}


def done_ids(path):
    return {json.loads(l)['chunkId'] for l in open(path)} if os.path.exists(path) else set()


def stage_gen():
    rows = [json.loads(l) for l in open(os.path.join(OUT, 'sample.jsonl'))]
    path = os.path.join(OUT, 'gen.jsonl')
    done = done_ids(path)
    t0 = time.time()
    with open(path, 'a') as f:
        for i, r in enumerate(x for x in rows if x['chunkId'] not in done):
            out = chat(GEN_SYS, f"OPERATOR: {r['operator']}\n\nPASSAGE:\n{r['text']}", GEN_SCHEMA, 400)
            try:
                g = json.loads(out)
            except json.JSONDecodeError:
                g = {'skip': True, 'error': out[:200]}
            f.write(json.dumps({**{k: r[k] for k in ('operator', 'source', 'chunkId')}, **g}, ensure_ascii=False) + '\n')
            f.flush()
            if (i + 1) % 20 == 0:
                print(f"gen {i + 1}, {(time.time() - t0) / (i + 1):.1f} s each", flush=True)
    g = [json.loads(l) for l in open(path)]
    print(f"gen: {len(g)} rows, {sum(not x.get('skip') for x in g)} with a question")


VERIFY_SYS = (
    "You check a trivia item against the passage it was written from. Answer true only if the passage states or "
    "clearly implies the expected answer, and the expected answer actually answers the question about that operator. "
    "Answer false otherwise. Answer true or false only.")


def distinctive_reuse(q, text, operator, df, n_chunks):
    """Content words of the question (5+ letters, not the operator's name) that the passage also uses and that are
    rare in the corpus (in under 1% of chunks): the question copied the passage's wording."""
    name = set(re.findall(r"[a-z']+", operator.lower()))
    words = {w for w in re.findall(r"[a-z']{5,}", q.lower())} - name
    t = set(re.findall(r"[a-z']{5,}", text.lower()))
    return sorted(w for w in words & t if df.get(w, 0) < n_chunks / 100)


def stage_verify():
    gen = [json.loads(l) for l in open(os.path.join(OUT, 'gen.jsonl'))]
    text = {c['chunkId']: c['text'] for c in chunks()}
    df, n = collections.Counter(), len(text)
    for t in text.values():
        df.update(set(re.findall(r"[a-z']{5,}", t.lower())))
    path = os.path.join(OUT, 'verify.jsonl')
    done = done_ids(path)
    with open(path, 'a') as f:
        for g in gen:
            if g.get('skip') or g['chunkId'] in done or not g.get('question'):
                continue
            ok = chat(VERIFY_SYS, f"OPERATOR: {g['operator']}\n\nPASSAGE:\n{text[g['chunkId']]}\n\nQUESTION: {g['question']}\n\n"
                      f"EXPECTED ANSWER: {g['expected']}", None, 3).lower().startswith('true')
            reuse = distinctive_reuse(g['question'], text[g['chunkId']], g['operator'], df, n)
            f.write(json.dumps({'chunkId': g['chunkId'], 'supported': ok, 'reuse': reuse}) + '\n')
            f.flush()
    v = {json.loads(l)['chunkId']: json.loads(l) for l in open(path)}
    keep = [g for g in gen if not g.get('skip') and v.get(g['chunkId'], {}).get('supported')]
    with open(os.path.join(ROOT, 'eval', 'trivia.jsonl'), 'w') as f:
        for i, g in enumerate(keep):
            f.write(json.dumps({'qid': f't{i:03d}', 'question': g['question'], 'expected': g['expected'],
                                'chunkId': g['chunkId'], 'kind': g['kind'], 'operator': g['operator'],
                                'source': g['source'], 'reuse': v[g['chunkId']]['reuse']}, ensure_ascii=False) + '\n')
    asked = sum(not g.get('skip') and bool(g.get('question')) for g in gen)
    print(f"verify: {len(keep)} kept of {asked} questions ({len(gen)} generated); reuse a rare passage word: "
          f"{sum(bool(v[g['chunkId']]['reuse']) for g in keep)}; by kind {dict(collections.Counter(g['kind'] for g in keep))}; "
          f"by source {dict(collections.Counter(g['source'] for g in keep))}")


def load_answer_eval():
    import importlib.util
    spec = importlib.util.spec_from_file_location('answer_eval', os.path.join(ROOT, 'scripts', 'answer-eval.py'))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


def stage_judge(ans_path):
    ae = load_answer_eval()
    items = {json.loads(l)['qid']: json.loads(l) for l in open(os.path.join(ROOT, 'eval', 'trivia.jsonl'))}
    text = {c['chunkId']: c['text'] for c in chunks()}
    for t in (json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/topics/topics.jsonl'))):
        text[f"topic:{t['topic']}"] = re.sub(r'\s*\[[\d,;\s]+\]', '', t.get('summary') or '')
    out_path = ans_path.replace('.jsonl', '.judged.jsonl')
    done = {json.loads(l)['qid'] for l in open(out_path)} if os.path.exists(out_path) else set()
    for a in (json.loads(l) for l in open(ans_path)):
        if a['qid'] in done:
            continue
        g = items[a['qid']]
        first = re.split(r'(?<=[.!?])\s+', a['answer'].strip(), maxsplit=1)[0]
        declined = bool(ae.DECLINE.search(first)) and ae.verdict(ae.ABSTAIN_SYS, a['answer'])
        ev = f"{g['expected']}\n---\n{text.get(g['chunkId'], '')}"
        correct = (not declined) and ae.verdict(ae.CORRECT_SYS, f"QUESTION: {g['question']}\n\nREFERENCE EVIDENCE:\n{ev}\n\nANSWER: {a['answer']}")
        given = '\n---\n'.join(text[i] for i in a['passages'] if i in text)
        sents = [] if declined or not a['passages'] else ae.sentences(a['answer'])
        sup = [ae.verdict(ae.SUPPORT_SYS, f'PASSAGES:\n{given}\n\nSENTENCE: {x}') for x in sents]
        row = {'qid': a['qid'], 'kind': g['kind'], 'source': g['source'], 'declined': declined, 'correct': correct,
               'inPassages': g['chunkId'] in a['passages'], 'cited': g['chunkId'] in a['cited'], 'table': not a['passages'],
               'sentences': len(sents), 'supported': sum(sup)}
        with open(out_path, 'a') as f:
            f.write(json.dumps(row) + '\n')
    R = [json.loads(l) for l in open(out_path)]
    s_, t_ = sum(r['supported'] for r in R), sum(r['sentences'] for r in R)
    print(f"trivia {len(R)}: correct {sum(r['correct'] for r in R)} ({sum(r['correct'] for r in R) / len(R):.3f}); "
          f"declined {sum(r['declined'] for r in R)}; source chunk in passages {sum(r['inPassages'] for r in R)}; "
          f"faithful {s_}/{t_} = {s_ / max(t_, 1):.3f}")
    for key in ('kind', 'source'):
        for k, v in sorted(collections.defaultdict(list, {k: [r for r in R if r[key] == k] for k in {r[key] for r in R}}).items()):
            print(f"  {key} {k:13} n {len(v):3}  correct {sum(r['correct'] for r in v):3}  declined {sum(r['declined'] for r in v):3}  "
                  f"in passages {sum(r['inPassages'] for r in v):3}")


def rank(corpus, q, cid):
    import subprocess
    o = subprocess.run([os.path.join(ROOT, 'target/release/search'), '--corpus', corpus, '--k', '100', '--json', q],
                       capture_output=True, text=True, cwd=ROOT).stdout
    h = json.loads(o)
    ids = [x['chunkId'] for x in (h['hits'] if isinstance(h, dict) else h)]
    return ids.index(cid) + 1 if cid in ids else None


def stage_diagnose(ans_path):
    items = {json.loads(l)['qid']: json.loads(l) for l in open(os.path.join(ROOT, 'eval', 'trivia.jsonl'))}
    J = {json.loads(l)['qid']: json.loads(l) for l in open(ans_path.replace('.jsonl', '.judged.jsonl'))}
    A = {json.loads(l)['qid']: json.loads(l) for l in open(ans_path)}
    names = collections.defaultdict(set)
    for r in (json.loads(l) for l in open(os.path.join(ROOT, 'artifacts/entities/real_names.jsonl'))):
        if r.get('realName'):
            names[r['name']].add(r['realName'])
    ident = json.load(open(os.path.join(ROOT, 'artifacts/entities/identities.json')))
    for i in ident:
        for n in i['names']:
            names[n].update(i['names'])
    rows = []
    for qid, j in J.items():
        if j['correct']:
            continue
        g, a = items[qid], A[qid]
        ql = g['question'].lower()
        cand = {g['operator']} | names.get(g['operator'], set())
        named = any(re.search(r'(?<![\w])' + re.escape(n.lower()) + r'(?![\w])', ql) for n in cand if len(n) >= 3)
        r3, r4 = rank('artifacts/p3b', g['question'], g['chunkId']), rank('artifacts/p4', g['question'], g['chunkId'])
        best = min(x for x in (r3, r4, 999) if x)
        if a['passages'] == []:
            cause = 'routing (table answer)'
        elif j['inPassages']:
            cause = 'answer step (source chunk in passages)'
        elif not named:
            cause = 'name resolution (operator not named as such)'
        elif best <= 8:
            cause = 'retrieval: top 8 for the raw question but not in passages (route, form or budget)'
        elif best <= 40:
            cause = 'retrieval: rank 9 to 40'
        elif best <= 100:
            cause = 'retrieval: rank 41 to 100'
        else:
            cause = 'retrieval: beyond 100'
        rows.append({'qid': qid, 'kind': g['kind'], 'source': g['source'], 'cause': cause, 'rankP3b': r3, 'rankP4': r4,
                     'named': named, 'declined': j['declined'], 'question': g['question'], 'reuse': g.get('reuse')})
    with open(ans_path.replace('.jsonl', '.diag.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    print(f"misses {len(rows)} of {len(J)}")
    for c, n in collections.Counter(r['cause'] for r in rows).most_common():
        print(f"  {n:3}  {c}")
    print("by kind (misses / items):")
    kinds = collections.Counter(items[q]['kind'] for q in J)
    for k, n in collections.Counter(r['kind'] for r in rows).most_common():
        print(f"  {k:13} {n}/{kinds[k]}")


if __name__ == '__main__':
    stages = {'sample': stage_sample, 'gen': stage_gen, 'verify': stage_verify}
    if sys.argv[1] in ('judge', 'diagnose'):
        {'judge': stage_judge, 'diagnose': stage_diagnose}[sys.argv[1]](sys.argv[2])
    else:
        stages[sys.argv[1]]()
