#!/usr/bin/env python3
"""P3 dossiers: a grounded profile per major character.

Characters: the 200 speakers with the most story chunks among operators (character_table),
identity labels (scripts/entities.py) and names speaking in 4+ groups, minus generic role
labels ("Reunion Member", "Medic Operator"); since 2026-10-01 (DOSSIER_SOURCE=v2) also every other
operator and the NPCs chosen by story line counts (characters_v2). Inputs per character: their operator archive (if
any), the P2 summaries of the 12 stories they speak most in, in in-world order (chronology v1
storyline year, then release), and their known names. Output ~300 words.
  gen    generator model -> artifacts/dossiers/dossiers.jsonl (resumable)
  judge  judge model: each sentence against the top 5 BM25 passages of the character's own
         story chunks and archive (plus neighbours) -> artifacts/dossiers/judged.jsonl
  sets   -> artifacts/dossiers/dossiers.v1.jsonl, the v1 characters' rows (the P3b of ask --lore v1)
"""
import collections, hashlib, json, math, os, random, re, sys, threading, time, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import incr

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'dossiers')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3a'))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
N = int(os.environ.get('N_CHARACTERS', '200'))
SAMPLE = int(os.environ.get('SAMPLE', '30'))
# DOSSIER_SOURCE=v2 (default since 2026-10-01): the 200 plus every other operator and the NPCs who speak as much as the
# least-speaking of the 200; v1 = the 200 alone, as before. DOSSIER_ONLY_NEW=1: gen writes only characters with no row
# (leaves stale v1 rows for a later update). DOSSIER_JUDGE=new: the judge samples SAMPLE of the characters added in v2.
SOURCE = os.environ.get('DOSSIER_SOURCE', 'v2')
ONLY_NEW = os.environ.get('DOSSIER_ONLY_NEW') == '1'
JUDGE_SET = os.environ.get('DOSSIER_JUDGE', 'all')
sha16 = lambda s: hashlib.sha256(s.encode()).hexdigest()[:16]
ROLE = re.compile(r"\b(Member|Operator|Warrior|Mercenary|Soldier|Guard|Mob|Citizen|Officer|Worker|Staff|Villager|Student|"
                  r"Merchant|Crowd|Voice|Man|Woman|Girl|Boy|Kid|Child|Agent|Scout|Civilian|Resident|Thug|Bandit|"
                  r"Squire|Passerby|Customer|Clerk|Crew|Recruit|Trooper|Messenger|Shopkeeper|Attendant)s?\b")
GEN_SYS = ("You write a dossier on one Arknights character for a reader of the story, using only the material given: their "
           "operator archive and summaries of stories they appear in, listed in in-world order. Write about 300 words in "
           "plain prose: who they are and what they are known as, their affiliations, key relationships, and what happens "
           "to them across these stories, in order. Use names, not pronouns, when a sentence could be ambiguous. Do not add "
           "outside knowledge or speculation. No preamble.")
SUPPORT_SYS = ("You check one claim about an Arknights character against evidence from the game text. Answer true only if "
               "the evidence states or clearly implies the claim. Answer false otherwise. Answer with true or false only.")


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


def load():
    return [json.loads(l) for l in open(os.path.join(CORPUS, 'chunks.jsonl'))]


def characters(rows):
    spk = collections.Counter(); grp = collections.defaultdict(set)
    for r in rows:
        if r['groupId'] == 'archive':
            continue
        for p in set(r['speakers']):
            spk[p] += 1; grp[p].add(r['groupId'])
    ct = json.load(open(os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel', 'character_table.json')))['Characters']
    ops = {c['value'].get('Name'): c['key'] for c in ct if c['value'].get('Name')}
    ids = {i['label']: i for i in json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.json')))}
    out = []
    for p, n in spk.most_common():
        if len(out) >= N:
            break
        if not (p in ops or p in ids or len(grp[p]) >= 4) or ROLE.search(p) or not re.match(r"^[A-Z0-9]", p):
            continue
        out.append({'name': p, 'chunks': n, 'charId': ops.get(p), 'names': ids[p]['names'] if p in ids else [p]})
    if SOURCE == 'v1':
        return out
    return characters_v2(rows, out, spk, ops, ids)


def characters_v2(rows, v1, spk, ops, ids):
    """The v1 characters first and unchanged, then every operator without a dossier, then the NPCs who speak at least as
    many story lines as the v1 character who speaks least (the floor is measured, not set), speaker labels merged into
    a person by the identity links and by a title before a known name ("Inquisitor Irene" is Irene)."""
    lines = collections.Counter()
    for r in rows:
        if r['groupId'] == 'archive':
            continue
        for x in r['text'].split('\n'):
            m = re.match(r"^([^:\n]{1,40}):", x)
            if m:
                lines[m.group(1)] += 1
    alias = {}
    for label, i in ids.items():
        for n in i['names']:
            alias.setdefault(n, label)
    known = set(ops) | set(ids) | set(alias)
    # A speaker label that is the whole-word tail of exactly one operator name is that operator ("Red" is Projekt Red).
    tails = collections.defaultdict(set)
    for o in ops:
        w = o.split(' ')
        for k in range(1, len(w)):
            tails[' '.join(w[k:])].add(o)

    def person(s, links=True):
        if links and s in alias:
            return alias[s]
        if s not in known and len(tails.get(s, ())) == 1:
            return next(iter(tails[s]))
        w = s.split(' ')
        for k in range(1, len(w)):
            tail = ' '.join(w[k:])
            if tail in known:
                return alias.get(tail, tail)
        return s
    merged = collections.Counter()
    for s, n in lines.items():
        merged[person(s)] += n
    have = {c['name'] for c in v1}
    have_names = {n for c in v1 for n in c['names']}
    floor = min(lines[c['name']] for c in v1)  # 145 on 2026-10-01: each v1 character's own label
    out = list(v1)
    # Operators: every character_table operator with an archive (407 in operator_attributes), alternates included.
    attrs = read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))
    label = {}
    for s_, n in lines.items():
        p = person(s_)
        if n > lines.get(label.get(p), 0):
            label[p] = s_
    # An operator's lines are its own label's, or a label the tail or title rule gives it; never an identity link's
    # (links joined Estelle to Perfumer and Shirayuki to Midnight on 2026-10-01, so they only merge NPC labels).
    op_label = {}
    for s_, n in lines.items():
        p = person(s_, links=False)
        if p in ops and n > lines.get(op_label.get(p), 0):
            op_label[p] = s_
    for a in sorted(attrs, key=lambda a: (-merged[a['name']], a['name'])):
        if a['name'] in have or a['name'] in have_names:
            continue
        sp = op_label.get(a['name'], a['name'])
        names = ids[a['name']]['names'] if a['name'] in ids else [a['name']]
        c = {'name': a['name'], 'chunks': spk.get(sp, 0), 'charId': a['charId'], 'names': list(dict.fromkeys(names + [sp])), 'kind': 'operator'}
        if sp != a['name']:
            c['speaks'] = sp  # the label their story lines carry
        out.append(c)
        have.add(a['name'])
    npcs = 0
    for p, n in merged.most_common():
        if n < floor:
            break
        if p in have or p in have_names or ROLE.search(p) or not re.match(r"^[A-Z0-9]", p) or p in ops or p not in label:
            continue
        if label[p] in ops or any(n in ops for n in (ids[p]['names'] if p in ids else [])):
            continue  # an operator's person: the operator's own dossier covers them
            continue
        c = {'name': p, 'chunks': spk.get(label[p], 0), 'charId': None, 'names': ids[p]['names'] if p in ids else [p], 'kind': 'npc'}
        if label[p] != p:
            c['speaks'] = label[p]
        out.append(c)
        have.add(p); npcs += 1
    print(f"characters v2: {len(v1)} v1, {len(out) - len(v1) - npcs} operators added, {npcs} NPCs added "
          f"(at least {floor} story lines, the v1 floor); {len(out)} in all", flush=True)
    return out


def characters_v1_names(rows):
    global SOURCE
    keep, SOURCE = SOURCE, 'v1'
    try:
        return characters(rows)
    finally:
        SOURCE = keep


def gen_input(c, lines_in, summ, gkey, arch):
    top = [s for s, _ in lines_in[c.get('speaks', c['name'])].most_common(12) if s in summ]
    top.sort(key=lambda s: gkey.get(summ[s]['groupId'], (9999, 0)))
    return gen_text(c, top, summ, arch)


def gen_text(c, top, summ, arch):
    archive = '\n'.join(arch.get(f"archive_{c['charId']}", []))[:6000] if c['charId'] else ''
    others = ', '.join(x for x in c['names'] if x != c['name']) or 'none'
    parts = [f"Character: {c['name']} (also named: {others})"]
    if archive:
        parts.append('Operator archive:\n' + archive)
    parts.append('Stories they appear in, in in-world order:\n' + '\n'.join(
        f"- {summ[s]['header']}: {summ[s]['summary']}" for s in top))
    return '\n\n'.join(parts), top, archive


def gen_stage():
    rows = load()
    chars = characters(rows)
    arch = collections.defaultdict(list)
    for r in rows:
        if r['groupId'] == 'archive':
            arch[r['storyId']].append(r['text'])
    lines_in = collections.defaultdict(collections.Counter)
    for r in rows:
        if r['groupId'] == 'archive':
            continue
        for x in r['text'].split('\n'):
            lines_in[x.split(':', 1)[0]][r['storyId']] += 1
    summ = {s['storyId']: s for s in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'stories.jsonl'))}
    tl = json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))
    # Episodes 0 to 9, 15 and 16 carry no release time in timeline_v1 (rebuilt 2026-09-27 18:58, after
    # these dossiers); None beside an int made the sort raise, and 0 files them as the oldest content.
    gkey = {g['groupId']: (g['storylineYear'], g['releaseTime'] or 0) for g in tl['groups']}
    pend = incr.p2_pending()[0] if incr.PLAN else set()
    st = incr.Stage('dossiers gen', os.path.join(OUT, 'dossiers.jsonl'), lambda r: r['name'], sha16(GEN_SYS))
    for c in chars:
        # A story whose summary P2 will redo (or write for the first time) can enter the top 12.
        top = [s for s, _ in lines_in[c.get('speaks', c['name'])].most_common(12) if s in summ or s in pend]
        st.unit(c['name'], gen_input(c, lines_in, summ, gkey, arch)[0], upstream=any(s in pend for s in top))
    # Backfill rebuilds each old row's input from the names and stories the row records, not from today's
    # inputs: identities.json (2026-09-28 07:55) and timeline_v1.json (09-27 18:58) were rebuilt after the
    # dossiers (09-27 00:55), which changed the names of 32 of 200 and the story order of 25.
    st.row_input = lambda r: gen_text(r, r['stories'], summ, arch)[0]
    # A dossier's unit exists while its name still speaks in the corpus; a smaller N_CHARACTERS drops none.
    live = {p for r in rows if r['groupId'] != 'archive' for p in r['speakers']}
    # v2 adds operators who never speak in a story; their rows stay while they are characters (a superset of
    # v1's live set, so a v1 file loses nothing it kept before).
    live |= {c['name'] for c in chars} | {a['name'] for a in read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))}
    return st, [c['name'] for c in chars], live, chars, lines_in, summ, gkey, arch


def stage_gen():
    os.makedirs(OUT, exist_ok=True)
    st, order, live, chars, lines_in, summ, gkey, arch = gen_stage()
    names = st.plan(order, live)
    if names is None:
        return
    names = set(names)
    if ONLY_NEW:
        names -= {k for k in st.have}
    todo = [c for c in chars if c['name'] in names]
    psha = st.psha
    print(f'gen: {len(todo)} characters to do, {len(order) - len(todo)} done, prompt {psha}', flush=True)
    lock = threading.Lock(); it = iter(todo); n = [0]; t0 = time.time()

    def work():
        while True:
            with lock:
                c = next(it, None)
            if c is None:
                return
            user, top, archive = gen_input(c, lines_in, summ, gkey, arch)
            out = chat(GEN_SYS, user, 600)
            row = {'name': c['name'], 'charId': c['charId'], 'names': c['names'], 'chunks': c['chunks'],
                   'stories': top, 'hasArchive': bool(archive), 'dossier': out, 'promptSha': psha}
            with lock:
                st.put(row)
                n[0] += 1
                if n[0] % 20 == 0:
                    print(f'{time.strftime("%H:%M:%S")} gen {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.1f} s each', flush=True)
    th = [threading.Thread(target=work) for _ in range(2)]
    [t.start() for t in th]; [t.join() for t in th]
    print('gen done', flush=True)


ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bLt\.)(?<!\bJr\.)(?<!\bSr\.)(?<!\bMt\.)(?<!\bMs\.)(?<!\bNo\.)(?<!\bVol\.)'


def sentences(t):
    return [x.strip() for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'])', ' '.join(t.split())) if len(x.strip()) > 20]


def bm25_top(query, docs, k):
    tok = lambda s: re.findall(r"[a-z0-9']+", s.lower())
    D = [tok(d) for d in docs]; Nd = len(D); avg = sum(map(len, D)) / max(Nd, 1)
    df = collections.Counter(w for d in D for w in set(d)); q = set(tok(query)); sc = []
    for i, d in enumerate(D):
        tf = collections.Counter(d); s = 0.0
        for w in q:
            if w in tf:
                s += math.log(1 + (Nd - df[w] + 0.5) / (df[w] + 0.5)) * tf[w] * 2.2 / (tf[w] + 1.2 * (0.25 + 0.75 * len(d) / avg))
        sc.append((s, i))
    return [i for _, i in sorted(sc, reverse=True)[:k]]


def stage_judge():
    rows = load()
    D = read_jsonl(os.path.join(OUT, 'dossiers.jsonl'))
    if JUDGE_SET == 'new':
        v1 = {c['name'] for c in characters_v1_names(rows)}
        D = [d for d in D if d['name'] not in v1]
    random.Random(27).shuffle(D)
    path = os.path.join(OUT, 'judged.jsonl')
    done = {r['name'] for r in read_jsonl(path)}
    by_story = collections.defaultdict(list)
    for r in rows:
        by_story[r['storyId']].append(r)
    for d in D[:SAMPLE]:
        if d['name'] in done:
            continue
        pool = [c for s in d['stories'] for c in sorted(by_story[s], key=lambda c: c['ordinal'])]
        if d['charId']:
            pool += by_story.get(f"archive_{d['charId']}", [])
        verdicts = []; lock = threading.Lock(); claims = sentences(d['dossier']); it = iter(claims)

        def work():
            while True:
                with lock:
                    cl = next(it, None)
                if cl is None:
                    return
                ev = set()
                for i in bm25_top(cl, [c['text'] for c in pool], 5):
                    for j in (i - 1, i, i + 1):
                        if 0 <= j < len(pool) and pool[j]['storyId'] == pool[i]['storyId']:
                            ev.add(j)
                v = chat(SUPPORT_SYS, 'EVIDENCE:\n' + '\n---\n'.join(pool[j]['text'] for j in sorted(ev)) + f'\n\nCLAIM: {cl}', 3,
                         'root ::= "true" | "false"\n') == 'true'
                with lock:
                    verdicts.append((cl, v))
        th = [threading.Thread(target=work) for _ in range(2)]
        [t.start() for t in th]; [t.join() for t in th]
        row = {'name': d['name'], 'words': len(d['dossier'].split()), 'sentences': len(claims),
               'supported': sum(v for _, v in verdicts), 'unsupported': [c for c, v in verdicts if not v]}
        with open(path, 'a') as f:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
        print(f"{d['name']}: {row['supported']}/{row['sentences']} supported, {row['words']} words", flush=True)
    J = read_jsonl(path)
    s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J)
    print(f'judge: {len(J)} dossiers, {s}/{t} sentences supported = {s / max(t, 1):.3f}', flush=True)
    if JUDGE_SET == 'new':
        names = {d['name'] for d in D}
        J = [r for r in J if r['name'] in names]
        s = sum(r['supported'] for r in J); t = sum(r['sentences'] for r in J)
        full = sum(r['supported'] == r['sentences'] for r in J)
        print(f'judge, added in v2: {len(J)} dossiers, {s}/{t} sentences supported = {s / max(t, 1):.3f}; {full} fully supported', flush=True)
        for r in sorted(J, key=lambda r: r['supported'] / max(r['sentences'], 1))[:5]:
            print(f"  {r['name']}: {r['supported']}/{r['sentences']}; " + ' | '.join(r['unsupported'][:3]), flush=True)


def stage_sets():
    """dossiers.v1.jsonl: the rows of the v1 characters in file order, for the v1 lore's P3b (build-profiles --dossiers);
    with no v2 rows it is dossiers.jsonl byte for byte."""
    v1 = {c['name'] for c in characters_v1_names(load())}
    src = os.path.join(OUT, 'dossiers.jsonl')
    lines = [l for l in open(src) if json.loads(l)['name'] in v1]
    with open(os.path.join(OUT, 'dossiers.v1.jsonl'), 'w') as f:
        f.writelines(lines)
    print(f'sets: {len(lines)} v1 dossiers of {sum(1 for _ in open(src))}', flush=True)


if __name__ == '__main__':
    {'gen': stage_gen, 'judge': stage_judge, 'sets': stage_sets}[sys.argv[1]]()
