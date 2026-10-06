#!/usr/bin/env python3
"""P4 answer bank: precomputed answers the site serves before retrieval-only answers.

Every entry carries its spoiler gate: the stories whose content it depends on (`gateStories`), the
union of their `requiredStages` from spoiler.jsonl, and the operator files it uses (`gateChars`).
The site shows an entry only to a reader who has cleared every gate story.
  seed       deterministic entries from existing artifacts, no model:
               who      "Who is X?" per dossier (200), gate = the dossier's stories + archive
               story    "What happens in <story>?" per P2 story summary, gate = that story
               event    "What happens in <event>?" per method-C group summary, gate = its stories
               primer   "What should I know before starting <event>?", only when PRIMERS=1; gate =
                        the prior groups' stories. Only sentences the `primers` stage found supported
                        by the prior summaries are kept (PRIMER_FILTER=0 keeps the whole primer): on
                        the 20-primer sample the unsupported 10/253 held both real leaks (Near Light)
                        and inventions ("the death of Emperor")
  primers    judge model: support of every primer sentence -> artifacts/bank/primer_support.jsonl
  questions  generator model: one factual question per sampled chunk (N_QUESTIONS, default 900);
             stories anchored by gold set v2 are excluded so the eval stays clean
  answer     runs `target/release/ask --batch` on the questions (generator model on the server)
  merge      generated answers that do not decline and cite something join the bank,
             gate = source story + cited stories
  judge      judge model: faithfulness of a sample of generated answers to their passages
Output: artifacts/bank/bank.jsonl (id, kind, question, aliases, answer, sources, gateStories,
requiredStages, gateChars). The corpus spec calls this file answers.jsonl; bank.jsonl avoids a
clash with `ask --batch` output.
"""
import collections, json, os, random, re, subprocess, sys, time, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import incr

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'bank')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3b'))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
N_Q = int(os.environ.get('N_QUESTIONS', '900'))
SAMPLE = int(os.environ.get('SAMPLE', '60'))
G_BOOL = 'root ::= "true" | "false"\n'
Q_SYS = ("You read one passage from the Arknights story and write one question a fan might ask that this passage "
         "answers. Name the characters, place or event instead of using pronouns, so the question makes sense on its "
         "own. Ask about what happens, why, or who someone is, not about wording. Output the question only.")
SUPPORT_SYS = ("You check one sentence of an answer against the passages it was based on. Answer true only if the "
               "passages state or clearly imply it. Answer false otherwise. Answer true or false only.")
DECLINE = re.compile(r"(could not find|couldn't find|cannot find|can't find|not (?:mentioned|stated|found|in the)|"
                     r"no information|does not (?:say|state|mention|contain)|doesn't (?:say|state|mention))", re.I)
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bMt\.)(?<!\bNo\.)'


def read_jsonl(p):
    return [json.loads(l) for l in open(p)] if os.path.exists(p) else []


def chat(system, user, max_tokens, grammar=None):
    body = {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
            'temperature': 0, 'seed': 1, 'max_tokens': max_tokens}
    if grammar:
        body['grammar'] = grammar
    data = json.dumps(body).encode(); err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', data, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=1200) as r:
                return json.load(r)['choices'][0]['message']['content'].strip()
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def spoilers():
    return {r['storyId']: r for r in read_jsonl(os.path.join(CORPUS, 'spoiler.jsonl'))}


def gate(stories, chars, sp):
    stories = sorted({s for s in stories if not s.startswith('archive_')})
    req = sorted({x for s in stories for x in (sp.get(s, {}).get('requiredStages') or [])})
    missing = [s for s in stories if s not in sp]
    return {'gateStories': stories, 'requiredStages': req, 'gateChars': sorted({c for c in chars if c}),
            **({'gateUnknown': missing} if missing else {})}


def stage_seed():
    os.makedirs(OUT, exist_ok=True)
    sp = spoilers()
    tl = {g['groupId']: g for g in json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))['groups']}
    stories = read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'stories.jsonl'))
    by_group = collections.defaultdict(list)
    for s in stories:
        by_group[s['groupId']].append(s['storyId'])
    # Aliases come from the current identity links, not the names frozen into each dossier: the dossiers
    # were written before the build rules dropped 72 bad links (Kal'tsit carried "Raidian" and "Serafina").
    ids = {i['label']: i['names'] for i in json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.json')))}
    rows = []
    for d in read_jsonl(os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl')):
        rows.append({'id': f"who:{d['name']}", 'kind': 'who', 'question': f"Who is {d['name']}?",
                     'aliases': [f'Who is {n}?' for n in ids.get(d['name'], []) if n != d['name']], 'answer': d['dossier'],
                     'sources': d['stories'] + ([f"archive_{d['charId']}"] if d['hasArchive'] else []),
                     **gate(d['stories'], [d['charId']] if d['hasArchive'] else [], sp)})
    for s in stories:
        rows.append({'id': f"story:{s['storyId']}", 'kind': 'story', 'question': f"What happens in {s['header']}?",
                     'aliases': [], 'answer': s['summary'], 'sources': [s['storyId']], **gate([s['storyId']], [], sp)})
    for g in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'groups.jsonl')):
        if g['method'] != 'C':
            continue
        name = tl.get(g['groupId'], {}).get('name', g['groupId'])
        rows.append({'id': f"event:{g['groupId']}", 'kind': 'event', 'question': f'What happens in {name}?',
                     'aliases': [f'Summarize {name}.'], 'answer': g['summary'], 'sources': g['storyIds'],
                     **gate(g['storyIds'], [], sp)})
    if os.environ.get('PRIMERS') == '1':
        support = {r['groupId']: r['sentences'] for r in read_jsonl(os.path.join(OUT, 'primer_support.jsonl'))}
        for p in read_jsonl(os.path.join(ROOT, 'artifacts', 'primers', 'primers.jsonl')):
            if not p['prior']:
                continue
            src = [s for h in p['prior'] for s in by_group.get(h, [])]
            text = p['primer']
            if os.environ.get('PRIMER_FILTER') != '0':
                if p['groupId'] not in support:
                    continue
                text = ' '.join(x for x, ok in support[p['groupId']] if ok)
            rows.append({'id': f"primer:{p['groupId']}", 'kind': 'primer',
                         'question': f"What should I know before starting {p['name']}?",
                         'aliases': [f"What happened before {p['name']}?"], 'answer': text, 'sources': p['prior'],
                         **gate(src, [], sp)})
    with open(os.path.join(OUT, 'seed.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    c = collections.Counter(r['kind'] for r in rows)
    print(f'seed: {len(rows)} entries {dict(c)}; with unknown gate stories {sum("gateUnknown" in r for r in rows)}')


def primers_module():
    import importlib.util
    spec = importlib.util.spec_from_file_location('primers', os.path.join(ROOT, 'scripts', 'primers.py'))
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m


def stage_primers():
    os.makedirs(OUT, exist_ok=True)
    pm = primers_module()
    _, summ, name = pm.plan()
    path = os.path.join(OUT, 'primer_support.jsonl')
    done = {r['groupId'] for r in read_jsonl(path)}
    for p in read_jsonl(os.path.join(ROOT, 'artifacts', 'primers', 'primers.jsonl')):
        if not p['prior'] or p['groupId'] in done:
            continue
        src = '\n'.join(f"- {name.get(h, h)}: {summ[h]}" for h in p['prior']) + f"\n- Opening blurb of {p['name']}: {p['blurb']}"
        sents = pm.sentences(p['primer'])
        ok = [chat(pm.SUPPORT_SYS, f'SUMMARIES:\n{src}\n\nSENTENCE: {x}', 3, G_BOOL) == 'true' for x in sents]
        with open(path, 'a') as f:
            f.write(json.dumps({'groupId': p['groupId'], 'sentences': list(zip(sents, ok))}, ensure_ascii=False) + '\n')
    R = read_jsonl(path)
    t = sum(len(r['sentences']) for r in R); k = sum(ok for r in R for _, ok in r['sentences'])
    print(f'primers: {len(R)} primers, {k}/{t} sentences supported and kept ({k / max(t, 1):.3f})')


def questions_stage():
    chunks = read_jsonl(os.path.join(CORPUS, 'chunks.jsonl'))
    gold = {an['story_id'] for g in read_jsonl(os.path.join(ROOT, 'eval', 'goldset_v2.jsonl')) for an in g.get('anchors', [])}
    # Round-robin over groups so a long event does not crowd out the rest; stories in the gold set are
    # skipped whole so no bank question paraphrases an eval item.
    by_g = collections.defaultdict(list)
    for c in chunks:
        # Profile and summary units are model-written; a question drawn from one would grade Trevor on its
        # own output, so questions come from the script and the operator archives only.
        if c['storyId'] not in gold and len(c['text']) > 400 and c['groupId'] not in ('profile', 'summary'):
            by_g[c['groupId']].append(c)
    st = incr.Stage('bank questions', os.path.join(OUT, 'questions.jsonl'), lambda r: r['chunkId'], incr.sha16(Q_SYS))
    pool_path = os.path.join(OUT, 'question_pool.json')
    eligible = [c for v in by_g.values() for c in v]
    if st.rows and not os.environ.get('BANK_RESAMPLE'):
        # After the first build the sample is kept: re-drawing the round-robin after an update reshuffles it
        # (a new group moves every pick) and would regenerate most of the 900 questions. Kept picks stay
        # (a changed chunk is redone by its key); chunks of stories new since the pool was recorded are
        # drawn at the first build's rate by a hash of the chunk id, so a draw never moves.
        # BANK_RESAMPLE=1 restores the whole-corpus redraw.
        had = {r['chunkId'] for r in st.rows}
        pool = set(json.load(open(pool_path))['stories']) if os.path.exists(pool_path) else {c['storyId'] for c in eligible}
        rate = float(os.environ.get('BANK_NEW_RATE', str(N_Q / 13_954)))  # 900 of 13,954 eligible at the first build
        picked = [c for c in eligible if c['chunkId'] in had or
                  (c['storyId'] not in pool and int(incr.sha16(c['chunkId']), 16) % 10_000 < rate * 10_000)]
    else:
        rng = random.Random(4)
        for v in by_g.values():
            rng.shuffle(v)
        order = sorted(by_g); rng.shuffle(order)
        picked = []
        while len(picked) < N_Q and any(by_g.values()):
            for g in order:
                if by_g[g] and len(picked) < N_Q:
                    picked.append(by_g[g].pop())
    if not os.environ.get('PLAN'):
        with open(pool_path, 'w') as f:
            json.dump({'stories': sorted({c['storyId'] for c in eligible})}, f)
    for c in picked:
        st.unit(c['chunkId'], c['text'])
    # A question stays while its chunk is in the corpus, picked or not, as before; only a vanished chunk
    # drops its question.
    return st, [c['chunkId'] for c in picked], {c['chunkId'] for c in chunks}, picked


def stage_questions():
    os.makedirs(OUT, exist_ok=True)
    st, order, live, picked = questions_stage()
    ids = st.plan(order, live)
    if ids is None:
        return
    ids = set(ids)
    todo = [c for c in picked if c['chunkId'] in ids]
    print(f'questions: {len(todo)} to do, {len(st.rows)} done', flush=True)
    # A redone question gets a fresh qid, so `ask --batch` (which resumes on qid) answers it again and
    # `merge` drops the old answer, whose qid is gone. KEY_IDS_ONLY keeps the old count-based numbering.
    base = len(st.rows) if incr.IDS_ONLY else max((int(r['qid'][1:]) for r in st.rows), default=0)
    psha = st.psha
    t0 = time.time()
    for n, c in enumerate(todo, 1):
        q = chat(Q_SYS, c['text'][:6000], 80).split('\n')[0].strip()
        st.put({'qid': f"b{base + n:05d}", 'question': q, 'chunkId': c['chunkId'],
                'storyId': c['storyId'], 'groupId': c['groupId'], 'promptSha': psha})
        if n % 50 == 0:
            print(f'{time.strftime("%H:%M:%S")} questions {n}/{len(todo)}, {(time.time() - t0) / n:.1f} s each', flush=True)
    print('questions done', flush=True)


def stage_answer():
    cmd = [os.path.join(ROOT, 'target', 'release', 'ask'), '--corpus', CORPUS, '--no-spawn', '--server', SERVER,
           '--batch', os.path.join(OUT, 'questions.jsonl'), '--out', os.path.join(OUT, 'answers.jsonl')]
    sys.exit(subprocess.call(cmd, cwd=ROOT))


def stage_merge():
    sp = spoilers()
    Q = {r['qid']: r for r in read_jsonl(os.path.join(OUT, 'questions.jsonl'))}
    rows = read_jsonl(os.path.join(OUT, 'seed.jsonl')); kept = dropped = 0
    for a in read_jsonl(os.path.join(OUT, 'answers.jsonl')):
        q = Q.get(a['qid'])
        first = re.split(r'(?<=[.!?])\s+', a['answer'].strip(), maxsplit=1)[0]
        if not q or DECLINE.search(first) or not a['cited']:
            dropped += 1
            continue
        cited = sorted({c.split('#')[0] for c in a['cited']})
        chars = [s[len('archive_'):] for s in cited if s.startswith('archive_')]
        rows.append({'id': f"ask:{a['qid']}", 'kind': 'ask', 'question': q['question'], 'aliases': [],
                     'answer': a['answer'], 'sources': a['cited'], 'passages': a['passages'],
                     **gate(cited + [q['storyId']], chars, sp)})
        kept += 1
    with open(os.path.join(OUT, 'bank.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    c = collections.Counter(r['kind'] for r in rows)
    print(f'bank: {len(rows)} entries {dict(c)}; generated kept {kept}, dropped {dropped} (declined or uncited)')


def stage_judge():
    chunks = {r['chunkId']: r for r in read_jsonl(os.path.join(CORPUS, 'chunks.jsonl'))}
    A = [r for r in read_jsonl(os.path.join(OUT, 'bank.jsonl')) if r['kind'] == 'ask']
    random.Random(29).shuffle(A)
    path = os.path.join(OUT, 'judged.jsonl')
    done = {r['id'] for r in read_jsonl(path)}
    for r in A[:SAMPLE]:
        if r['id'] in done:
            continue
        given = '\n---\n'.join(chunks[i]['text'] for i in r['passages'] if i in chunks)
        t = re.sub(r'\s*\[[\d,;\s]+\]', '', ' '.join(r['answer'].split()))
        sents = [x.strip() for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'*])', t) if len(x.strip()) > 20]
        sup = [chat(SUPPORT_SYS, f'PASSAGES:\n{given}\n\nSENTENCE: {s}', 3, G_BOOL) == 'true' for s in sents]
        with open(path, 'a') as f:
            f.write(json.dumps({'id': r['id'], 'sentences': len(sents), 'supported': sum(sup),
                                'unsupported': [s for s, v in zip(sents, sup) if not v]}, ensure_ascii=False) + '\n')
    J = read_jsonl(path)
    s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J)
    print(f'judge: {len(J)} generated entries, {s}/{t} sentences supported = {s / max(t, 1):.3f}; '
          f'entries fully supported {sum(r["supported"] == r["sentences"] for r in J)}')


if __name__ == '__main__':
    {'seed': stage_seed, 'primers': stage_primers, 'questions': stage_questions, 'answer': stage_answer, 'merge': stage_merge,
     'judge': stage_judge}[sys.argv[1]]()
