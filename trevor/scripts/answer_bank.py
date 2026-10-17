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
             stories anchored by gold set v2 are excluded so the eval stays clean; scripts/bank_regen.py
             calls stage_questions(only=chunk ids) to question exactly the chunks an update changed
  answer     runs `target/release/ask --batch` on the questions (generator model on the server)
  merge      generated answers that do not decline and cite something join the bank,
             gate = source story + cited stories. Since 2026-10-17 (contract v1.1 section 3a) an answer
             whose evidence row (artifacts/bank/evidence.jsonl, scripts/bank_regen.py: the sha of its
             question's chunk and of every cited chunk when it was made) no longer matches the corpus is
             dropped, and an answer bank_regen.py made is kept only when one citation sits at or before
             its question's own story by gatePos; REGEN_ANSWERS=0 is the merge before, byte for byte
  judge      judge model: faithfulness of a sample of generated answers to their passages
Output: artifacts/bank/bank.jsonl (id, kind, question, aliases, answer, sources, gateStories,
requiredStages, gateChars). The corpus spec calls this file answers.jsonl; bank.jsonl avoids a
clash with `ask --batch` output.
"""
import collections, json, os, random, re, subprocess, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common, incr
from common import ROOT, read_jsonl, sha16

OUT = os.path.join(ROOT, 'artifacts', 'bank')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3b'))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
N_Q = int(os.environ.get('N_QUESTIONS', '900'))
SAMPLE = int(os.environ.get('SAMPLE', '60'))
Q_SYS = ("You read one passage from the Arknights story and write one question a fan might ask that this passage "
         "answers. Name the characters, place or event instead of using pronouns, so the question makes sense on its "
         "own. Ask about what happens, why, or who someone is, not about wording. Output the question only.")
SUPPORT_SYS = ("You check one sentence of an answer against the passages it was based on. Answer true only if the "
               "passages state or clearly imply it. Answer false otherwise. Answer true or false only.")
DECLINE = re.compile(r"(could not find|couldn't find|cannot find|can't find|not (?:mentioned|stated|found|in the)|"
                     r"no information|does not (?:say|state|mention|contain)|doesn't (?:say|state|mention))", re.I)
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bMt\.)(?<!\bNo\.)'


def chat(system, user, max_tokens):
    return common.chat(SERVER, system, user, max_tokens, 1200)


def supported(system, user):
    return common.verdict(SERVER, system, user, 1200)


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
    common.write_jsonl(os.path.join(OUT, 'seed.jsonl'), rows)
    c = collections.Counter(r['kind'] for r in rows)
    print(f'seed: {len(rows)} entries {dict(c)}; with unknown gate stories {sum("gateUnknown" in r for r in rows)}')


def primers_module():
    return common.load_script('primers')


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
        ok = [supported(pm.SUPPORT_SYS, f'SUMMARIES:\n{src}\n\nSENTENCE: {x}') for x in sents]
        common.append_jsonl(path, {'groupId': p['groupId'], 'sentences': list(zip(sents, ok))})
    R = read_jsonl(path)
    t = sum(len(r['sentences']) for r in R); k = sum(ok for r in R for _, ok in r['sentences'])
    print(f'primers: {len(R)} primers, {k}/{t} sentences supported and kept ({k / max(t, 1):.3f})')


def eligible_chunk(c, gold):
    """A chunk a bank question may be drawn from: script or operator archive text (profile and summary units are
    model-written), over 400 characters, outside the gold set's stories."""
    return c['storyId'] not in gold and len(c['text']) > 400 and c['groupId'] not in ('profile', 'summary')


def gold_stories():
    return {an['story_id'] for g in read_jsonl(os.path.join(ROOT, 'eval', 'goldset_v2.jsonl')) for an in g.get('anchors', [])}


def questions_stage(only=None):
    """`only` (a set of chunk ids, scripts/bank_regen.py): the units are exactly those chunks that are eligible, and
    every other row is kept; the sample and its pool are left alone."""
    chunks = read_jsonl(os.path.join(CORPUS, 'chunks.jsonl'))
    gold = gold_stories()
    # Round-robin over groups so a long event does not crowd out the rest; stories in the gold set are
    # skipped whole so no bank question paraphrases an eval item.
    by_g = collections.defaultdict(list)
    for c in chunks:
        # Profile and summary units are model-written; a question drawn from one would grade Trevor on its
        # own output, so questions come from the script and the operator archives only.
        if eligible_chunk(c, gold):
            by_g[c['groupId']].append(c)
    st = incr.Stage('bank questions', os.path.join(OUT, 'questions.jsonl'), lambda r: r['chunkId'], incr.sha16(Q_SYS))
    pool_path = os.path.join(OUT, 'question_pool.json')
    eligible = [c for v in by_g.values() for c in v]
    if only is not None:
        picked = [c for c in eligible if c['chunkId'] in only]
        for c in picked:
            st.unit(c['chunkId'], c['text'])
        return st, [c['chunkId'] for c in picked], {c['chunkId'] for c in chunks}, picked
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


def stage_questions(only=None, chat_fn=None, show=True):
    """`only`: question only these chunk ids (bank_regen.py); `chat_fn(system, user, max_tokens)` -> text replaces
    the local server. Returns the rows written."""
    os.makedirs(OUT, exist_ok=True)
    st, order, live, picked = questions_stage(only)
    ids = st.plan(order, live, show=show)
    if ids is None:
        return []
    ids = set(ids)
    todo = [c for c in picked if c['chunkId'] in ids]
    print(f'questions: {len(todo)} to do, {len(st.rows)} done', flush=True)
    # A redone question gets a fresh qid, so `ask --batch` (which resumes on qid) answers it again and
    # `merge` drops the old answer, whose qid is gone. KEY_IDS_ONLY keeps the old count-based numbering.
    base = len(st.rows) if incr.IDS_ONLY else max((int(r['qid'][1:]) for r in st.rows), default=0)
    psha = st.psha
    t0 = time.time()
    out = []
    for n, c in enumerate(todo, 1):
        q = (chat_fn or chat)(Q_SYS, c['text'][:6000], 80).split('\n')[0].strip()
        row = {'qid': f"b{base + n:05d}", 'question': q, 'chunkId': c['chunkId'],
               'storyId': c['storyId'], 'groupId': c['groupId'], 'promptSha': psha}
        st.put(row)
        out.append(row)
        if n % 50 == 0:
            print(f'{time.strftime("%H:%M:%S")} questions {n}/{len(todo)}, {(time.time() - t0) / n:.1f} s each', flush=True)
    if show:
        print('questions done', flush=True)
    return out


def stage_answer():
    cmd = [os.path.join(ROOT, 'target', 'release', 'ask'), '--corpus', CORPUS, '--no-spawn', '--server', SERVER,
           '--batch', os.path.join(OUT, 'questions.jsonl'), '--out', os.path.join(OUT, 'answers.jsonl')]
    sys.exit(subprocess.call(cmd, cwd=ROOT))


def evidence_shas(art):
    """chunk id -> sha16 of its text over the corpora a bank answer can cite: P4x, then CORPUS (P3b) over it (P3b's
    16,050 chunks are in P4x with the same text). Missing files count as empty."""
    out = {}
    for d in ('p4x', os.path.relpath(CORPUS, os.path.join(ROOT, 'artifacts'))):
        p = os.path.join(art, d, 'chunks.jsonl')
        if os.path.exists(p):
            for r in read_jsonl(p):
                out[r['chunkId']] = sha16(r['text'])
    return out


def evidence_ledger():
    """qid -> its newest row of artifacts/bank/evidence.jsonl (scripts/bank_regen.py): the sha of every chunk the
    answer rests on (the question's chunk and the cited ones) when it was made, and how it was made."""
    return {r['qid']: r for r in read_jsonl(os.path.join(OUT, 'evidence.jsonl'))}


def within_gate(a, q, sp_pos):
    """The 2026-10-17 keep rule for regenerated answers (contract v1.1 section 3a): at least one cited chunk sits at
    or before the question's own chunk on the gatePos scale, so the answer does not rest only on later stories."""
    own = sp_pos.get(q['storyId'])
    return own is not None and any(sp_pos.get(c.split('#')[0], 10**9) <= own for c in a['cited'])


def stage_merge():
    sp = spoilers()
    Q = {r['qid']: r for r in read_jsonl(os.path.join(OUT, 'questions.jsonl'))}
    rows = read_jsonl(os.path.join(OUT, 'seed.jsonl')); kept = dropped = 0
    # REGEN_ANSWERS=0 is the kill switch: no evidence check and no gate rule, the merge before 2026-10-17.
    check = os.environ.get('REGEN_ANSWERS') != '0'
    ledger = evidence_ledger() if check else {}
    cur = evidence_shas(os.path.join(ROOT, 'artifacts')) if check and ledger else {}
    sp_pos = gate_positions() if check and ledger else {}
    why = collections.Counter()
    for a in read_jsonl(os.path.join(OUT, 'answers.jsonl')):
        q = Q.get(a['qid'])
        first = re.split(r'(?<=[.!?])\s+', a['answer'].strip(), maxsplit=1)[0]
        if not q or DECLINE.search(first) or not a['cited']:
            dropped += 1
            if check and not q:
                why['question redone (its chunk changed)'] += 1
            continue
        ev = ledger.get(a['qid'])
        if check and ev is None:
            why['unverified (no evidence row, kept)'] += 1
        elif check:
            moved = [c for c, h in ev['shas'].items() if cur.get(c) != h]
            if moved:
                why['evidence vanished' if any(c not in cur for c in moved) else 'evidence changed'] += 1
                dropped += 1
                continue
            if ev.get('made') == 'regen' and not within_gate(a, q, sp_pos):
                why['cites only past its gate'] += 1
                dropped += 1
                continue
        cited = sorted({c.split('#')[0] for c in a['cited']})
        chars = [s[len('archive_'):] for s in cited if s.startswith('archive_')]
        rows.append({'id': f"ask:{a['qid']}", 'kind': 'ask', 'question': q['question'], 'aliases': [],
                     'answer': a['answer'], 'sources': a['cited'], 'passages': a['passages'],
                     **gate(cited + [q['storyId']], chars, sp)})
        kept += 1
    common.write_jsonl(os.path.join(OUT, 'bank.jsonl'), rows)
    c = collections.Counter(r['kind'] for r in rows)
    print(f'bank: {len(rows)} entries {dict(c)}; generated kept {kept}, dropped {dropped} (declined or uncited'
          + (', or evidence changed' if check else '') + ')' + (f'; {dict(why)}' if why else ''))
    return {'entries': len(rows), 'kept': kept, 'dropped': dropped, **{k: v for k, v in why.items()}}


def gate_positions():
    """story or source id -> gatePos from artifacts/chrono/gate_order.json (storyOrder index + 1, sourcePos)."""
    p = os.path.join(ROOT, 'artifacts', 'chrono', 'gate_order.json')
    if not os.path.exists(p):
        return {}
    g = json.load(open(p))
    pos = {s: i + 1 for i, s in enumerate(g['storyOrder'])}
    pos.update({k: v for k, v in (g.get('sourcePos') or {}).items() if k not in pos})
    return pos


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
        sents = common.split_sentences(common.strip_cites(' '.join(r['answer'].split())), ABBR, '[A-Z"\'*]')
        sup = [supported(SUPPORT_SYS, f'PASSAGES:\n{given}\n\nSENTENCE: {s}') for s in sents]
        common.append_jsonl(path, {'id': r['id'], 'sentences': len(sents), 'supported': sum(sup),
                                   'unsupported': [s for s, v in zip(sents, sup) if not v]})
    J = read_jsonl(path)
    s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J)
    print(f'judge: {len(J)} generated entries, {s}/{t} sentences supported = {s / max(t, 1):.3f}; '
          f'entries fully supported {sum(r["supported"] == r["sentences"] for r in J)}')


if __name__ == '__main__':
    {'seed': stage_seed, 'primers': stage_primers, 'questions': stage_questions, 'answer': stage_answer, 'merge': stage_merge,
     'judge': stage_judge}[sys.argv[1]]()
