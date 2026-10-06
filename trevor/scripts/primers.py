#!/usr/bin/env python3
"""P2 primers: "what you need to know before starting" each main chapter and event.

Ian's decision (2026-09-26): chronology is in-world order. For each target group (main chapters
and multi-story events), the prior context is the groups that come before it in chronology v1
(storyline year, then main-story episode, then release), ranked by how many of the target's
recurring speakers they share; the six highest are summarized from their method-C group summaries.
The target's own official opening blurb says what matters without revealing its plot.
  gen    generator model -> artifacts/primers/primers.jsonl (resumable)
  judge  judge model: faithfulness of each sentence to the prior summaries given, and spoiler
         leakage: does a sentence describe something that happens in the target itself?
"""
import collections, hashlib, json, os, random, re, sys, time, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import incr

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'primers')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
PRIOR = int(os.environ.get('PRIOR', '6'))
SAMPLE = int(os.environ.get('SAMPLE', '20'))
sha16 = lambda s: hashlib.sha256(s.encode()).hexdigest()[:16]
GEN_SYS = ("You write a short primer for an Arknights reader about to start a story event: what they need to know "
           "beforehand. You get the event's own opening blurb, and summaries of earlier events in in-world order. "
           "Write about 250 words: the earlier events and people that matter for this event, and where things stand "
           "when it begins. Never describe what happens in the event itself beyond its opening blurb. Use only the "
           "material given, no outside knowledge. No preamble.")
# v2 (default): the target is described only through its opening blurb; v1 leaked the target's plot
# (the judge flagged 0.284 of sentences). PRIMER_PROMPT=v1 restores the first prompt.
if os.environ.get('PRIMER_PROMPT') != 'v1':
    GEN_SYS += (" Mention the upcoming event only in the words of its opening blurb; every other sentence must be "
                "about an earlier event, in the past tense. Do not guess what will happen in the upcoming event.")
SUPPORT_SYS = ("You check one sentence of a primer against the summaries it was written from. Answer true only if the "
               "summaries state or clearly imply it. Answer false otherwise. Answer true or false only.")
LEAK_SYS = ("You check whether a sentence from a spoiler-free primer reveals what happens in the event it introduces. "
            "You get the event's official opening blurb, which the primer is allowed to state, the event's full summary, "
            "and the sentence. Answer true only if the sentence reveals a plot development of the event that the opening "
            "blurb does not already state; content of the opening blurb, or about earlier events, is not a leak. Answer "
            "true or false only.")
G_BOOL = 'root ::= "true" | "false"\n'


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


def plan():
    tl = json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))
    key = {g['groupId']: (g['storylineYear'], g.get('chapter') or 0, g['releaseTime'] or 0) for g in tl['groups']}
    rel = {g['groupId']: g['releaseTime'] for g in tl['groups']}
    chap = {g['groupId']: g.get('chapter') for g in tl['groups']}
    main10 = rel.get('main_10') or 0

    def released_before(h, g):
        # Spoiler safety: a primer may only draw on content released no later than its target, or a
        # later-released summary can recap the target (v1 leaked 0.284 of sentences by the judge).
        if rel.get(h) and rel.get(g):
            return rel[h] <= rel[g]
        if rel.get(h) is None and chap.get(h) is not None:
            # Episodes 0 to 9 carry no release time; they are the oldest content, safe for a later
            # main chapter or any target released after Episode 10.
            return (chap.get(g) is not None and chap[g] > chap[h]) or (rel.get(g) or 0) >= main10
        return False
    name = {g['groupId']: g['name'] for g in tl['groups']}
    summ = {g['groupId']: g['summary'] for g in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'groups.jsonl')) if g['method'] == 'C'}
    stories = read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'stories.jsonl'))
    # Groups with one story have no group summary; their story summary stands in.
    for s in stories:
        summ.setdefault(s['groupId'], s['summary'])
    first_blurb = {}
    for s in stories:
        first_blurb.setdefault(s['groupId'], s['officialSynopsis'])
    speakers = collections.defaultdict(collections.Counter)
    for l in open(incr.chunks_path()):
        r = json.loads(l)
        speakers[r['groupId']].update(set(r['speakers']))
    targets = [g for g in summ if not g.startswith('story_') and g in key]
    order = sorted(key, key=key.get)
    pos = {g: i for i, g in enumerate(order)}
    out = []
    for g in targets:
        cast = {p for p, n in speakers[g].items() if n >= 2}
        before = [h for h in order[:pos[g]] if h in summ and not h.startswith('story_') and
                  (os.environ.get('NO_RELEASE_RULE') or released_before(h, g))]
        ranked = sorted(before, key=lambda h: (-len(cast & set(speakers[h])), -pos[h]))[:PRIOR]
        ranked.sort(key=lambda h: pos[h])
        out.append({'groupId': g, 'name': name.get(g, g), 'blurb': first_blurb.get(g, ''), 'prior': ranked})
    return out, summ, name


def gen_input(it, summ, name):
    prior = '\n'.join(f"- {name.get(h, h)}: {summ[h]}" for h in it['prior'])
    return f"Event: {it['name']}\nOpening blurb: {it['blurb']}\n\nEarlier events, in in-world order:\n{prior}"


def gen_stage():
    items, summ, name = plan()
    pend = incr.p2_pending()[1] if incr.PLAN else set()
    st = incr.Stage('primers gen', os.path.join(OUT, 'primers.jsonl'), lambda r: r['groupId'], sha16(GEN_SYS))
    for it in items:
        # The prior list is part of the input: a new event that ranks among the six changes it.
        st.unit(it['groupId'], gen_input(it, summ, name), upstream=any(h in pend for h in it['prior']))
    return st, [it['groupId'] for it in items], None, items, summ, name


def stage_gen():
    os.makedirs(OUT, exist_ok=True)
    st, order, live, items, summ, name = gen_stage()
    ids = st.plan(order, live)
    if ids is None:
        return
    ids = set(ids)
    psha = st.psha
    todo = [i for i in items if i['groupId'] in ids]
    print(f'gen: {len(todo)} primers to do, {len(order) - len(todo)} done, prompt {psha}', flush=True)
    t0 = time.time()
    for n, it in enumerate(todo, 1):
        if not it['prior']:
            text = 'Nothing earlier in the story is needed: this is where it begins.'
        else:
            text = chat(GEN_SYS, gen_input(it, summ, name), 500)
        row = dict(it, primer=text, promptSha=psha)
        st.put(row)
        if n % 10 == 0:
            print(f'{time.strftime("%H:%M:%S")} gen {n}/{len(todo)}, {(time.time() - t0) / n:.1f} s each', flush=True)
    print('gen done', flush=True)


def sentences(t):
    ab = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bMt\.)(?<!\bNo\.)'
    return [x.strip() for x in re.split(ab + r'(?<=[.!?])\s+(?=[A-Z"\'])', ' '.join(t.split())) if len(x.strip()) > 20]


def stage_judge():
    _, summ, name = plan()
    P = [p for p in read_jsonl(os.path.join(OUT, 'primers.jsonl')) if p['prior']]
    random.Random(28).shuffle(P)
    ids = os.environ.get('JUDGE_IDS')
    if ids:
        want = json.load(open(ids)); P = [p for p in P if p['groupId'] in want] + [p for p in P if p['groupId'] not in want]
    path = os.path.join(OUT, 'judged.jsonl')
    done = {r['groupId'] for r in read_jsonl(path)}
    for p in P[:SAMPLE]:
        if p['groupId'] in done:
            continue
        src = '\n'.join(f"- {name.get(h, h)}: {summ[h]}" for h in p['prior']) + f"\n- Opening blurb of {p['name']}: {p['blurb']}"
        sents = sentences(p['primer'])
        sup = [chat(SUPPORT_SYS, f'SUMMARIES:\n{src}\n\nSENTENCE: {s}', 3, G_BOOL) == 'true' for s in sents]
        # The first leak judge saw no blurb and flagged the blurb itself (Episode 5's primer quoted it).
        leak = [chat(LEAK_SYS, f"OPENING BLURB (allowed): {p['blurb']}\n\nEVENT SUMMARY:\n{summ[p['groupId']]}\n\nSENTENCE: {s}",
                     3, G_BOOL) == 'true' for s in sents]
        row = {'groupId': p['groupId'], 'sentences': len(sents), 'supported': sum(sup), 'leaks': sum(leak),
               'leaked': [s for s, v in zip(sents, leak) if v], 'unsupported': [s for s, v in zip(sents, sup) if not v]}
        with open(path, 'a') as f:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
        print(f"{p['groupId']}: supported {row['supported']}/{row['sentences']}, leaks {row['leaks']}", flush=True)
    J = read_jsonl(path)
    s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J); lk = sum(r['leaks'] for r in J)
    print(f'judge: {len(J)} primers, {s}/{t} sentences supported = {s / max(t, 1):.3f}; '
          f'{lk} sentences leak the event ({lk / max(t, 1):.3f}); primers with any leak {sum(r["leaks"] > 0 for r in J)}', flush=True)


if __name__ == '__main__':
    {'gen': stage_gen, 'judge': stage_judge}[sys.argv[1]]()
