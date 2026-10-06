#!/usr/bin/env python3
"""Topic summaries for races, nations and concepts: the largest class of real questions.

Worldbuilding and race questions are 25% of 757 real r/arknights lore questions and 19% of 345 Discord ones
(design/trevor-questions.md section 6), answered 0.70 to 0.73 of the time, from passages scattered across the
corpus. A topic summary gathers them, as the character dossiers do for people. Topics come from what players ask,
not from a hand list: every race and nation in the game data (operator_attributes.jsonl) plus concept terms, each
kept when at least MIN_ASKED real worldbuilding or race questions mention it; the count is stored.
For each topic: the best passages by BM25 over P4 (story, operator files, module stories, IS, enemies, items), then
the generator writes a cited summary from those passages only.
  plan       -> artifacts/topics/topics.jsonl (topics, aliases, counts, passages), no model. Since 2026-10-01 it keeps
             each topic's summary and keys from the old file, so gen redoes only topics whose passages or prompt
             changed (TOPIC_PLAN_FRESH=1 writes the file without them, as before), and it leaves Trevor's own topic
             units out of the passages: P4 holds them since 2026-09-30, and a plan over them would summarize the
             summaries (TOPIC_SELF=1 keeps them in, as before).
  PLAN=1     with plan, gen or judge: print what a run would redo and why, write nothing, call no model (update.sh)
  gen        generator model on $SERVER: summaries (resumable, keyed on topic + passages + prompt)
  units      -> artifacts/topics/units.jsonl in scripts/sources.py's unit format (group "topic") for build-units
  judge      judge model on $SERVER (Qwen3.5 9B, a different family from the generator): each sentence of each summary
             against the passages listed for its topic (the generator's whole input) -> artifacts/topics/judged.jsonl
             (resumable, keyed on topic + summary + prompt); per-topic score, mean, worst 5 with unsupported claims
Since 2026-10-01 (Ian: "topics on everything in the lore, chosen from data") the plan adds topics from data to the 46:
  mine       -> artifacts/topics/candidates.jsonl, no model: every race, nation, birthplace, team or group and enemy
             race in the game tables, plus the proper names the story and operator-file text uses mid-sentence in at
             least TOPIC_MINE_MIN chunks (default K, so a topic's passages can all name it) of at least
             TOPIC_MINE_GROUPS story groups (default 3), minus operator and identity-link names and names whose lower-case
             form is the commoner (common words). Each carries 3 lines of context.
  classify   generator model on $SERVER: one short grammar-bound call per candidate -> artifacts/topics/classified.jsonl
             (kind: race, nation, place, organization, event, concept, or person, title, other, which are dropped; and the
             base name a demonym, plural or short form belongs to, which merges aliases); keyed on term + context + prompt
  plan       with TOPIC_SOURCE=v2 (the default): the v1 topics exactly as before, first and unchanged (their keys, so their
             summaries and judgments are reused), then one topic per merged candidate group of a kept kind, with
             source "table" or "mined"; a new topic whose passages name it fewer than THIN_PASSAGES (3) times is marked
             thin and never generated. TOPIC_SOURCE=v1 is the 46-topic plan, byte-identical to the file before.
  thin       after gen and judge: a new topic whose summary is under 100 words, or judged under THIN_SUPPORT (0.8) of its
             sentences supported, is marked thin; thin topics are left out of the units and refused by the topic tool.
             The v1 topics are never marked (Pythia's 38 words were already refused by the tool).
"""
import collections, hashlib, json, math, os, re, sys, time, unicodedata, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'topics')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
MIN_ASKED = int(os.environ.get('MIN_ASKED', '1'))
K = int(os.environ.get('TOPIC_PASSAGES', '14'))
sha16 = lambda s: hashlib.sha256(s.encode()).hexdigest()[:16]
# Concepts and groups players ask about that are not a race or nation field; each is kept only if real questions
# mention it (the counts are in design/trevor-questions.md section 12). Aliases widen retrieval.
CONCEPTS = {'Originium': ['Originium'], 'Oripathy and the Infected': ['Oripathy', 'Infected', 'infection'],
            'Arts': ['Arts', 'Arts unit', 'caster'], 'Catastrophes': ['Catastrophe', 'Catastrophe Messenger'],
            'Seaborn': ['Seaborn', 'Ishar-mla', 'Sea Terror'], 'Mobile cities and landships': ['mobile city', 'landship', 'nomadic city'],
            'Firearms on Terra': ['gun', 'firearm', 'crossbow'], 'Sankta halos': ['halo', 'wings'],
            'Precursors': ['Precursor', 'Ancients', 'Elder'], 'Collapsals': ['Collapsal'], 'Feranmut': ['Feranmut', 'Sui', 'Beast Lord'],
            'Teekaz': ['Teekaz', 'Hundred Holdings'], 'Beast Lords': ['Beast Lord', 'Feranmut'], 'Pythia': ['Pythia'],
            'Wendigo': ['Wendigo', 'Patriot'], 'Nachzehrer': ['Nachzehrer'], 'Banshee': ['Banshee'], 'Draco': ['Draco', 'dragon'],
            'Terra': ['Terra', 'Terran']}
SYS = ("You write a reference entry on one topic of the Arknights world for a reader of the story, using only the numbered "
       "passages given (story scenes, operator files, module stories, item and enemy descriptions). Write 250 to 350 words "
       "of plain prose: what the topic is, its traits or rules, its history and society where the passages say, notable "
       "people or events, and what the passages leave open. Cite passages as [n] after each sentence that uses them. Say "
       "what is stated and what is only suggested; add no outside knowledge. No preamble.")
# 2026-09-29: the first run stated one Ursus soldier's line ("the Sami invaded Ursus territory") as fact, and the judge
# passed it (the passage does contain it). A claim made in dialogue keeps its speaker. TOPIC_ATTRIBUTE=0 = the first prompt.
if os.environ.get('TOPIC_ATTRIBUTE', '1') != '0':
    SYS = SYS.replace(" No preamble.", " When a claim comes from what a character says rather than from narration or a record, "
                      "name who says it (for example: according to Ivan, an Ursus soldier, ...), since characters can be "
                      "wrong or biased. No preamble.")
PSHA = sha16(SYS)
# 2026-10-02 trait audit: Kuranta's summary called them horned (a soldier's line about another person's horns, beside the
# word Kuranta, uncited) and Elafia's gave them "horns and fangs" (the Witch King's figure of speech about Leithanien);
# the judge passed both because the words are in the passages. A row marked `traitAudit` is regenerated with this rule
# into `summaryV2`, which only ask --lore v2 serves (TOPIC_TRAIT_RULE=0: retrait does nothing).
TRAIT_SYS = SYS.replace(" No preamble.", " Name a physical trait of the race only when a passage attributes it to the race as a whole "
                        "(in narration, a record or a description of the race) and cite that passage; a trait of one person, of "
                        "another race, or in a figure of speech is not a trait of the race. No preamble.")
TSHA = sha16(TRAIT_SYS)
TRAIT_KEYS = ('traitAudit', 'summaryV2', 'inputShaV2', 'promptShaV2', 'passagesV2', 'summaryV2Gen', 'traitDropped', 'traitCheckKey')
# The 14 BM25 passages of a race rarely describe its body (Kuranta's held one line about another person's horns and none of
# the "red-maned" or "Wild Mane" lines), so every race topic's v2 summary also reads up to TRAIT_EXTRA passages with a line
# naming the race and a body word.
TRAIT_EXTRA = int(os.environ.get('TRAIT_EXTRA', '4'))
BODY_RE = re.compile(r'\b(manes?|maned|ears?|tails?|horns?|horned|wings?|winged|halos?|hoo(?:f|ves)|fur|furred|scales?|scaled|antlers?|'
                     r'feathers?|feathered|fangs?|claws?|tusks?|beaks?|gills?|fins?)\b', re.I)


def read_jsonl(p):
    return [json.loads(l) for l in open(p)] if os.path.exists(p) else []


def bm25_index(docs):
    tok = lambda s: re.findall(r"[a-z0-9'ǫæ]+", s.lower())
    D = [tok(d) for d in docs]; N = len(D); avg = sum(map(len, D)) / max(N, 1)
    df = collections.Counter(w for d in D for w in set(d))
    tfs = [collections.Counter(d) for d in D]

    def search(query, k):
        q = set(tok(query)); sc = []
        for i, tf in enumerate(tfs):
            s = 0.0
            for w in q:
                if w in tf:
                    s += math.log(1 + (N - df[w] + 0.5) / (df[w] + 0.5)) * tf[w] * 2.2 / (tf[w] + 1.2 * (0.25 + 0.75 * len(D[i]) / avg))
            if s:
                sc.append((s, i))
        return [i for _, i in sorted(sc, reverse=True)[:k]]
    return search


PLAN = os.environ.get('PLAN') == '1'
SOURCE = os.environ.get('TOPIC_SOURCE', 'v2')
MINE_MIN = int(os.environ.get('TOPIC_MINE_MIN', str(K)))
MINE_GROUPS = int(os.environ.get('TOPIC_MINE_GROUPS', '3'))
THIN_PASSAGES = int(os.environ.get('THIN_PASSAGES', '3'))
THIN_SUPPORT = float(os.environ.get('THIN_SUPPORT', '0.8'))
KEEP_KINDS = ('race', 'nation', 'place', 'organization', 'event', 'concept')
FRESH = os.environ.get('TOPIC_PLAN_FRESH') == '1'
EXCLUDE = ('profile', 'summary', 'voice') + (() if os.environ.get('TOPIC_SELF') == '1' else ('topic',))


_P4 = []


def p4_search():
    """P4's chunks minus EXCLUDE and a BM25 search over them, built once per run."""
    if not _P4:
        chunks = [c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl')) if c['groupId'] not in EXCLUDE]
        _P4.extend([chunks, bm25_index([c['text'] for c in chunks])])
    return _P4[0], _P4[1]


def plan_rows():
    """The plan's topic rows: v1's, then (TOPIC_SOURCE=v2) the topics from data."""
    rows, topics = plan_rows_v1()
    if SOURCE == 'v1':
        return rows, topics
    return plan_rows_v2(rows), topics


def plan_rows_v1():
    """The plan's topic rows, with the old file's summary and keys carried over per topic (unless TOPIC_PLAN_FRESH)."""
    qs = [r['q'] for r in read_jsonl(os.path.join(ROOT, 'eval', 'reddit-lore-questions.all.jsonl')) + read_jsonl(
        os.path.join(ROOT, 'eval', 'discord-lore-questions.jsonl')) if r['category'] in ('H', 'R')]
    A = read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))
    skip = {'Unknown', 'Undisclosed', 'Rhodes Island'}
    topics = {}
    for r in A:
        for x in (r['race'] or '').split('/'):
            x = x.strip()
            if x and x not in skip and len(x) > 2:
                topics.setdefault(x, ('race', [x]))
        if r['nation'] and r['nation'] not in skip:
            topics.setdefault(r['nation'], ('nation', [r['nation']]))
    for name, al in CONCEPTS.items():
        topics.setdefault(name, ('concept', al))
    asked = {t: sum(1 for q in qs if any(re.search(r'\b' + re.escape(a) + r's?\b', q, re.I) for a in al)) for t, (_, al) in topics.items()}
    keep = {t: v for t, v in topics.items() if asked[t] >= MIN_ASKED}
    chunks, search = p4_search()
    rows = []
    for t, (kind, al) in sorted(keep.items(), key=lambda x: -asked[x[0]]):
        idx = search(' '.join(al + [t]), K)
        rows.append({'topic': t, 'kind': kind, 'aliases': al, 'asked': asked[t], 'passages': [chunks[i]['chunkId'] for i in idx]})
    if not FRESH:
        old = {r['topic']: r for r in read_jsonl(os.path.join(OUT, 'topics.jsonl'))}
        for r in rows:
            for k in ('summary', 'inputSha', 'promptSha'):
                if k in old.get(r['topic'], {}):
                    r[k] = old[r['topic']][k]
    return rows, topics


# ---------------------------------------------------------------- topics from data (TOPIC_SOURCE=v2, 2026-10-01)

GAMEDATA = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel')
CAP = r"[A-ZÀ-ÞÆ][\w'’\-]*"
NAME_RE = re.compile(CAP + r"(?:(?:\s(?:of|the)\s|\s)" + CAP + r")*")
SPEAKER_RE = re.compile(r"^[^:\n]{1,40}:\s?")
POSS_RE = re.compile(r"(?:['’]s|['’])$")
# "I", "I'm", "But I": the pronoun starts most capitalized runs and is no name.
PRONOUN_RE = re.compile(r"(?:^|\s)I(?:['’]\w+)?(?:\s|$)")


def fold(x):
    x = unicodedata.normalize('NFKD', x.lower())
    x = ''.join(c for c in x if not unicodedata.combining(c)).replace("'", '').replace('’', '')
    return ' '.join(re.sub(r'[^\w]+', ' ', x).split())


def people_names():
    """Folded names of people: every character_table name, the operators, and every identity-link name."""
    ct = json.load(open(os.path.join(GAMEDATA, 'character_table.json')))['Characters']
    names = {c['value'].get('Name') or '' for c in ct}
    names |= {r['name'] for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))}
    for i in json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.json'))):
        names |= {i['label'], *i['names']}
    return {fold(n) for n in names if n}


def table_candidates():
    """(term, data kind or None, source, extra aliases) from the game tables."""
    out = []
    clean = lambda x: re.sub(r'\s*\([^)]*\)', '', x).strip()
    ok = lambda x: x and len(x) > 2 and not re.search(r'unknown|undisclosed', x, re.I) and x.isprintable() and not re.search(r'[■?#]', x)
    for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl')):
        for x in (r['race'] or '').split('/'):
            if ok(clean(x)):
                out.append((clean(x), 'race', 'race', []))
        if ok(r['nation'] or ''):
            out.append((r['nation'], 'nation', 'nation', []))
        if ok(clean(r['birthplace'] or '')):
            out.append((clean(r['birthplace']), 'place', 'birthplace', []))
    for t in json.load(open(os.path.join(GAMEDATA, 'handbook_team_table.json')))['Handbook_teams']:
        v = t['value']
        if v['PowerId'] == 'none':
            continue
        code = [v['PowerCode']] if fold(v['PowerCode']) != fold(v['PowerName']) else []
        out.append((v['PowerName'], 'nation' if v['PowerLevel'] == 0 else 'organization', f"team level {v['PowerLevel']}", code))
    for r in json.load(open(os.path.join(GAMEDATA, 'enemy_handbook_table.json')))['RaceData']:
        out.append((r['value']['RaceName'], None, 'enemy race', []))
    return out


def stage_mine():
    """Candidates from the game tables and the corpus -> candidates.jsonl (deterministic, no model)."""
    C = [c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p3b', 'chunks.jsonl')) if c['groupId'] not in ('profile', 'summary')]
    mid = collections.Counter(); groups = collections.defaultdict(set); ctx = collections.defaultdict(dict)
    lower = collections.Counter()
    for c in C:
        seen = set(); low = set()
        for line in c['text'].split('\n'):
            body = SPEAKER_RE.sub('', line, count=1)
            low.update(re.findall(r"(?<![.!?]\s)(?<!^)\b[a-z][\w'\-]+", body))
            for m in NAME_RE.finditer(body):
                term = POSS_RE.sub('', m.group(0))
                pre = body[:m.start()].rstrip()
                if not pre or pre[-1] in '.!?"…—-(“:' or PRONOUN_RE.search(term) or len(term) < 3:
                    continue
                seen.add(term)
                if c['groupId'] not in ctx[term] and len(ctx[term]) < 3:
                    ctx[term][c['groupId']] = body[max(0, m.start() - 110):m.end() + 110].strip()
        for t in seen:
            mid[t] += 1; groups[t].add(c['groupId'])
        lower.update(low)
    people = people_names()
    cands = {}
    for t, n in mid.items():
        if n < MINE_MIN or len(groups[t]) < MINE_GROUPS or fold(t) in people:
            continue
        if ' ' not in t and lower[t.lower()] > n:
            continue
        cands[fold(t)] = {'term': t, 'sources': ['corpus'], 'dataKind': None, 'aliases': [], 'chunks': n,
                          'groups': len(groups[t]), 'contexts': list(ctx[t].values())}
    n_corpus = len(cands)
    by_source = collections.Counter()
    texts = [(c['groupId'], c['text']) for c in C]
    for term, kind, src, extra in table_candidates():
        by_source[src] += 1
        f = fold(term)
        if f in cands:
            r = cands[f]
            if src not in r['sources']:
                r['sources'].append(src)
            r['dataKind'] = r['dataKind'] or kind
            r['aliases'] = sorted(set(r['aliases']) | set(extra))
            continue
        pat = re.compile(r'\b' + re.escape(term) + r'\b')
        hits = [(g, t) for g, t in texts if pat.search(t)]
        cx = {}
        for g, t in hits:
            if g not in cx and len(cx) < 3:
                m = pat.search(t)
                cx[g] = ' '.join(t[max(0, m.start() - 110):m.end() + 110].split())
        cands[f] = {'term': term, 'sources': [src], 'dataKind': kind, 'aliases': list(extra), 'chunks': len(hits),
                    'groups': len({g for g, _ in hits}), 'contexts': list(cx.values())}
    rows = sorted(cands.values(), key=lambda r: (-r['chunks'], r['term']))
    tab = {s: len({fold(t) for t, _, src, _ in table_candidates() if src == s}) for s in by_source}
    print(f"mine: {len(rows)} candidates; corpus names {n_corpus} (in >= {MINE_MIN} chunks mid-sentence, >= {MINE_GROUPS} groups, "
          f"not a person, not a common word); game tables {sum(tab.values())} distinct per source {tab}; "
          f"table terms also mined {sum(1 for r in rows if len(r['sources']) > 1)}", flush=True)
    concepts = {t: [fold(a) for a in [t] + al] for t, al in CONCEPTS.items()}
    found = [t for t, al in concepts.items() if any(f in cands or f + 's' in cands or f.rstrip('s') in cands for f in al)]
    print(f"mine: the v1 CONCEPTS hand list: {len(found)} of {len(CONCEPTS)} found by mining alone (a name or alias is a candidate); "
          f"missing {sorted(set(CONCEPTS) - set(found))}", flush=True)
    if PLAN:
        old = {r['term'] for r in read_jsonl(os.path.join(OUT, 'candidates.jsonl'))}
        print(f"topics mine: {len(rows)} candidates, {len({r['term'] for r in rows} - old)} new", flush=True)
        return
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, 'candidates.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')


CLS_SYS = ("You classify a name from the Arknights story by what it refers to, using the context lines given. Kinds: race (a "
           "people of Terra, such as Sarkaz or Feline), nation (a country or state), place (a city, region, district, building "
           "or landmark), organization (a company, army, order, family, gang, church, department or other group), event (a war, "
           "incident, festival or other happening), concept (a substance, disease, power, kind of creature, technology, law, "
           "belief or other idea), person (one individual, by name or nickname), title (a rank, honorific or form of address), "
           "other (a common word, a date, an interjection, a fragment of a sentence). Also give the base name: when the name is "
           "a demonym, plural, adjective or short form of another name, that name (Victorian: Victoria; Sarkazs: Sarkaz; "
           "L.G.D.: Lungmen Guard Department); otherwise the name itself. Answer in the JSON form given.")
CLS_GRAMMAR = ('root ::= "{\\"kind\\": \\"" kind "\\", \\"base\\": \\"" base "\\"}"\n'
               'kind ::= "race" | "nation" | "place" | "organization" | "event" | "concept" | "person" | "title" | "other"\n'
               'base ::= [^"\\\\\\n]+\n')
CSHA = sha16(CLS_SYS + CLS_GRAMMAR)


def cls_input(r):
    return f"NAME: {r['term']}\nCONTEXT:\n" + ''.join(f"- {x}\n" for x in r['contexts'])


def stage_classify():
    cands = read_jsonl(os.path.join(OUT, 'candidates.jsonl'))
    path = os.path.join(OUT, 'classified.jsonl')
    done = {r['key'] for r in read_jsonl(path)}
    todo = [r for r in cands if sha16(cls_input(r) + CSHA) not in done]
    if PLAN:
        print(f"topics classify: plan {len(todo)} to do of {len(cands)}", flush=True)
        return
    t0 = time.time()
    for i, r in enumerate(todo, 1):
        body = json.dumps({'messages': [{'role': 'system', 'content': CLS_SYS}, {'role': 'user', 'content': cls_input(r)}],
                           'temperature': 0, 'seed': 1, 'max_tokens': 48, 'grammar': CLS_GRAMMAR}).encode()
        err = None
        for attempt in range(4):
            try:
                req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
                with urllib.request.urlopen(req, timeout=600) as resp:
                    out = json.loads(json.load(resp)['choices'][0]['message']['content'])
                break
            except Exception as e:
                err = e; out = None; time.sleep(5 * (attempt + 1))
        if out is None:
            raise err
        with open(path, 'a') as f:
            f.write(json.dumps({'term': r['term'], 'key': sha16(cls_input(r) + CSHA), 'kind': out['kind'],
                                'base': out['base'].strip()}, ensure_ascii=False) + '\n')
        if i % 50 == 0:
            print(f"{time.strftime('%H:%M:%S')} classify {i}/{len(todo)} ({(time.time() - t0) / i:.2f} s each)", flush=True)
    print(f'classify done: {len(todo)} written', flush=True)


DEMONYM_END = re.compile(r's|es|n|ns|an|ans|ian|ians|ese|ite|ites|ish|ine|ines|er|ers')


def demonym_of(x, y):
    """x is y plus a demonym or plural ending, sharing all but y's last two letters (Columbian: Columbia; Leithanian:
    Leithanien; Laterans: Laterano; Yanese: Yan). Folded names, one or more words."""
    if x == y or len(y) < 3:
        return False
    cp = len(os.path.commonprefix([x, y]))
    return cp >= max(3, len(y) - 2) and 0 < len(x) - cp <= 4 and bool(DEMONYM_END.fullmatch(x[cp:]))


def plan_rows_v2(v1rows):
    """The v1 rows unchanged, then one topic per merged candidate group of a kept kind."""
    cands = read_jsonl(os.path.join(OUT, 'candidates.jsonl'))
    cls = {r['key']: r for r in read_jsonl(os.path.join(OUT, 'classified.jsonl'))}
    unclassified = [r['term'] for r in cands if sha16(cls_input(r) + CSHA) not in cls]
    if unclassified:
        print(f"plan: {len(unclassified)} candidates not classified yet (run classify): {unclassified[:8]}", flush=True)
    old_key = {}
    for i, r in enumerate(v1rows):
        for a in [r['topic']] + r['aliases']:
            old_key.setdefault(fold(a), i)
    find_old = lambda f: old_key.get(f, old_key.get(f[:-1]) if f.endswith('s') else None)
    kept = []
    for r in cands:
        c = cls.get(sha16(cls_input(r) + CSHA))
        if c is None:
            continue
        kind = r['dataKind'] or c['kind']
        if r['dataKind'] == 'nation' and c['kind'] == 'organization':
            kind = 'organization'  # Rhodes Island is a power of the team table, not a state
        if kind in KEEP_KINDS:
            kept.append(dict(r, kind=kind, base=c['base']))
    # Merge: into a v1 topic by name, alias or base name; else candidates by base name (union-find).
    extra = collections.defaultdict(list)
    parent = list(range(len(kept)))
    by_fold = {fold(r['term']): i for i, r in enumerate(kept)}

    def root(i):
        while parent[i] != i:
            parent[i] = parent[parent[i]]; i = parent[i]
        return i
    to_old = {}
    for i, r in enumerate(kept):
        o = find_old(fold(r['term']))
        if o is None:
            o = find_old(fold(r['base']))
        if o is not None:
            to_old[i] = o
    # Demonyms and plurals the base name missed (Gemma gave Columbian and Leithanian as their own bases): a name that is
    # a v1 topic's or a table name's form plus a demonym or plural ending joins it.
    # Also a plural of another candidate (Saluzzos: Saluzzo), a name with a leading "The" (The Shard: Shard), and a
    # table name's short form from the team table (Karlan Trade CO., LTD: Karlan Trade).
    order = sorted(range(len(kept)), key=lambda i: (not kept[i]['dataKind'], -kept[i]['chunks']))
    heads = [(fold(r['topic']), ('old', i)) for i, r in enumerate(v1rows)] + [(fold(kept[i]['term']), ('cand', i)) for i in order]
    for i, r in enumerate(kept):
        for a in r['aliases']:
            j = by_fold.get(fold(a))
            if j is not None and j != i:
                parent[root(i)] = root(j)
    for i, r in enumerate(kept):
        if i in to_old:
            continue
        f = fold(r['term'])
        for h, (kind, j) in heads:
            if kind == 'cand' and j == i:
                continue
            if demonym_of(f, h) or (f.startswith('the ') and f[4:] == h):
                if kind == 'old':
                    to_old[i] = j
                else:
                    parent[root(i)] = root(j)
                break
    for i, r in enumerate(kept):
        j = by_fold.get(fold(r['base']))
        if j is None and fold(r['base']).endswith('s'):
            j = by_fold.get(fold(r['base'])[:-1])
        if j is not None and j != i:
            parent[root(i)] = root(j)
    groups = collections.defaultdict(list)
    for i in range(len(kept)):
        groups[root(i)].append(i)
    rank = {'race': 0, 'nation': 1, 'organization': 2, 'place': 3}
    new = []
    for members in groups.values():
        olds = {to_old[i] for i in members if i in to_old}
        if olds:
            o = min(olds)
            extra[o] += [kept[i]['term'] for i in members] + [a for i in members for a in kept[i]['aliases']]
            continue
        members.sort(key=lambda i: (rank.get(kept[i]['dataKind'], 9), -kept[i]['chunks'], kept[i]['term']))
        head = kept[members[0]]
        al = [head['term']] + [kept[i]['term'] for i in members[1:]] + [a for i in members for a in kept[i]['aliases']]
        al = list(dict.fromkeys(al))
        new.append({'topic': head['term'], 'kind': head['kind'], 'aliases': al,
                    'source': 'table' if any(kept[i]['dataKind'] for i in members) else 'mined',
                    'chunks': max(kept[i]['chunks'] for i in members)})
    new.sort(key=lambda r: (-r['chunks'], r['topic']))
    qs = [r['q'] for r in read_jsonl(os.path.join(ROOT, 'eval', 'reddit-lore-questions.all.jsonl')) + read_jsonl(
        os.path.join(ROOT, 'eval', 'discord-lore-questions.jsonl')) if r['category'] in ('H', 'R')]
    chunks, search = p4_search()
    text = {c['chunkId']: c['text'] for c in chunks}
    old = {r['topic']: r for r in read_jsonl(os.path.join(OUT, 'topics.jsonl'))}
    rows = [dict(r) for r in v1rows]
    for r in rows + new:
        for k in TRAIT_KEYS:
            if k in old.get(r['topic'], {}):
                r[k] = old[r['topic']][k]
    for i, r in enumerate(rows):
        names = [n for n in dict.fromkeys(extra.get(i, [])) if fold(n) not in {fold(a) for a in [r['topic']] + r['aliases']}]
        if names:
            r['names'] = names
    for r in new:
        al = r['aliases']
        r['asked'] = sum(1 for q in qs if any(re.search(r'\b' + re.escape(a) + r's?\b', q, re.I) for a in al))
        r['passages'] = [chunks[i]['chunkId'] for i in search(' '.join(al + [r['topic']]), K)]
        pats = [re.compile(r'(?<!\w)' + re.escape(a) + r'(?!\w)', re.I) for a in al]
        r['naming'] = sum(1 for p in r['passages'] if any(x.search(text[p]) for x in pats))
        if r['naming'] < THIN_PASSAGES:
            r['thin'] = f"passages: {r['naming']} of {len(r['passages'])} name it"
        if not FRESH:
            for k in ('summary', 'inputSha', 'promptSha'):
                if k in old.get(r['topic'], {}):
                    r[k] = old[r['topic']][k]
        rows.append(r)
    return rows


def stage_thin():
    """Mark the new topics whose summary is too short or poorly supported (v1 topics are never marked)."""
    path = os.path.join(OUT, 'topics.jsonl')
    rows = read_jsonl(path)
    J = {j['topic']: j for j in read_jsonl(os.path.join(OUT, 'judged.jsonl'))}
    marks = collections.Counter()
    for r in rows:
        if 'source' not in r:
            continue
        if r.get('thin', '').startswith('passages'):
            marks['passages'] += 1
            continue
        r.pop('thin', None)
        if not r.get('summary'):
            continue
        words = len(CITE_RE.sub('', r['summary']).split())
        j = J.get(r['topic'])
        if words < 100:
            r['thin'] = f'summary: {words} words'
        elif j is None or j['key'] != sha16(r['topic'] + r['summary'] + JSHA):
            r['thin'] = 'not judged'
        elif j['supported'] < THIN_SUPPORT * j['sentences']:
            r['thin'] = f"judge: {j['supported']}/{j['sentences']} supported"
        if 'thin' in r:
            marks[r['thin'].split(':')[0]] += 1
    new = [r for r in rows if 'source' in r]
    print(f"thin: {len(new)} new topics, {len(new) - sum(marks.values())} served, thin {dict(marks)}", flush=True)
    if not PLAN:
        with open(path, 'w') as f:
            for r in rows:
                f.write(json.dumps(r, ensure_ascii=False) + '\n')


CITE_RE = re.compile(r'\s*\[[\d,;\s]+\]')


def stage_plan():
    os.makedirs(OUT, exist_ok=True)
    rows, topics = plan_rows()
    old = {r['topic']: r['passages'] for r in read_jsonl(os.path.join(OUT, 'topics.jsonl'))}
    moved = [r['topic'] for r in rows if old.get(r['topic']) != r['passages']]
    gone = sorted(set(old) - {r['topic'] for r in rows})
    if PLAN:
        print(f"topics plan: {len(rows)} topics; passages changed for {len(moved)} {moved[:12]}; topics gone {gone}", flush=True)
        return
    with open(os.path.join(OUT, 'topics.jsonl'), 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    print(f"plan: {len(rows)} topics of {len(topics)} candidates (asked >= {MIN_ASKED}); "
          f"{collections.Counter(r['kind'] for r in rows)}; top: " + ', '.join(f"{r['topic']} ({r['asked']})" for r in rows[:15]))


def chat(user, system=None):
    body = json.dumps({'messages': [{'role': 'system', 'content': system or SYS}, {'role': 'user', 'content': user}],
                       'temperature': 0, 'seed': 1, 'max_tokens': 700}).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=900) as r:
                return json.load(r)['choices'][0]['message']['content'].strip()
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def gen_input(r, chunks):
    return f"TOPIC: {r['topic']}\n\nPASSAGES\n" + ''.join(
        f"\n[{i + 1}] ({chunks[c]['storyId']})\n{chunks[c]['text'][:3000]}\n" for i, c in enumerate(r['passages']) if c in chunks)


def gen_todo(rows, chunks):
    """Topics gen would redo, with the reason (the key rule of scripts/incr.py: summary, input sha, prompt sha)."""
    why = {}
    for r in rows:
        if r.get('thin', '').startswith('passages'):
            continue  # too few passages name it: never generated
        if not r.get('summary'):
            why[r['topic']] = 'new'
        elif r.get('promptSha') != PSHA:
            why[r['topic']] = 'prompt changed'
        elif r.get('inputSha') != sha16(gen_input(r, chunks)):
            why[r['topic']] = 'input changed'
    return why


def stage_gen():
    path = os.path.join(OUT, 'topics.jsonl')
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    if PLAN:
        # The rows plan would write (it has not run in a dry run), so a changed corpus shows here.
        why = gen_todo(plan_rows()[0], chunks)
        print(f"topics gen: plan {len(why)} to do ({dict(collections.Counter(why.values())) or 'nothing changed'}) {sorted(why)[:12]}", flush=True)
        return
    rows = read_jsonl(path)
    t0 = time.time(); n = 0
    for r in rows:
        if r.get('thin', '').startswith('passages'):
            continue
        user = gen_input(r, chunks)
        key = sha16(user)
        if r.get('summary') and r.get('inputSha') == key and r.get('promptSha') == PSHA:
            continue
        r['summary'] = chat(user); r['inputSha'] = key; r['promptSha'] = PSHA; n += 1
        with open(path, 'w') as f:
            for x in rows:
                f.write(json.dumps(x, ensure_ascii=False) + '\n')
        print(f"{time.strftime('%H:%M:%S')} {r['topic']}: {len(r['summary'].split())} words ({(time.time() - t0) / n:.0f} s each)", flush=True)
    print(f'gen done: {n} written', flush=True)


SUPPORT_SYS = ("You check one claim about the Arknights world against evidence from the game text. Answer true only if "
               "the evidence states or clearly implies the claim. Answer false otherwise. Answer with true or false only.")
JSHA = sha16(SUPPORT_SYS)
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bLt\.)(?<!\bJr\.)(?<!\bSr\.)(?<!\bMt\.)(?<!\bNo\.)(?<!\bVol\.)'


def sentences(t):
    # As dossiers.py: sentences over 20 characters, citations removed so the judge sees the claim only.
    t = re.sub(r'\s*\[[\d,;\s]+\]', '', t)
    return [x.strip() for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'])', ' '.join(t.split())) if len(x.strip()) > 20]


def verdict(evidence, claim):
    # Evidence first, claim last, one request at a time: the server reuses the cached evidence prefix per topic.
    body = json.dumps({'messages': [{'role': 'system', 'content': SUPPORT_SYS},
                                    {'role': 'user', 'content': f'EVIDENCE:\n{evidence}\n\nCLAIM: {claim}'}],
                       'temperature': 0, 'seed': 1, 'max_tokens': 3, 'grammar': 'root ::= "true" | "false"\n'}).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=900) as r:
                return json.load(r)['choices'][0]['message']['content'].strip() == 'true'
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def stage_judge():
    rows = [r for r in read_jsonl(os.path.join(OUT, 'topics.jsonl')) if r.get('summary')]
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    path = os.path.join(OUT, 'judged.jsonl')
    done = {(j['topic'], j['key']) for j in read_jsonl(path)}
    if PLAN:
        pending = set(gen_todo(plan_rows()[0], chunks))
        todo = sorted({r['topic'] for r in rows if (r['topic'], sha16(r['topic'] + r['summary'] + JSHA)) not in done} | pending)
        print(f"topics judge: plan {len(todo)} to do ({len(pending)} upstream pending) {todo[:12]}", flush=True)
        return
    t0 = time.time(); n = 0
    for r in rows:
        key = sha16(r['topic'] + r['summary'] + JSHA)
        if (r['topic'], key) in done:
            continue
        evidence = '\n---\n'.join(chunks[c]['text'][:3000] for c in r['passages'] if c in chunks)
        claims = sentences(r['summary'])
        vs = [(c, verdict(evidence, c)) for c in claims]
        row = {'topic': r['topic'], 'key': key, 'sentences': len(vs), 'supported': sum(v for _, v in vs),
               'unsupported': [c for c, v in vs if not v]}
        with open(path, 'a') as f:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
        n += 1
        print(f"{time.strftime('%H:%M:%S')} {r['topic']}: {row['supported']}/{row['sentences']} supported "
              f"({(time.time() - t0) / n:.0f} s per topic)", flush=True)
    # The trait-audited summaries (summaryV2), as topic '<name>#v2', against the same passages.
    for r in rows:
        if r.get('summaryV2'):
            t2 = r['topic'] + '#v2'; key = sha16(t2 + r['summaryV2'] + JSHA)
            if (t2, key) in done:
                continue
            evidence = '\n---\n'.join(chunks[c]['text'][:3000] for c in r.get('passagesV2', r['passages']) if c in chunks)
            vs = [(c, verdict(evidence, c)) for c in sentences(r['summaryV2'])]
            row = {'topic': t2, 'key': key, 'sentences': len(vs), 'supported': sum(v for _, v in vs), 'unsupported': [c for c, v in vs if not v]}
            with open(path, 'a') as f:
                f.write(json.dumps(row, ensure_ascii=False) + '\n')
            print(f"{t2}: {row['supported']}/{row['sentences']} supported", flush=True)
    keys = {r['topic']: sha16(r['topic'] + r['summary'] + JSHA) for r in rows}
    J = [j for j in read_jsonl(path) if keys.get(j['topic']) == j['key']]
    s = sum(j['supported'] for j in J); t = sum(j['sentences'] for j in J)
    per = sorted(J, key=lambda j: j['supported'] / max(j['sentences'], 1))
    mean = sum(j['supported'] / max(j['sentences'], 1) for j in J) / max(len(J), 1)
    full = sum(j['supported'] == j['sentences'] for j in J)
    print(f'judge: {len(J)} topics, {s}/{t} sentences supported = {s / max(t, 1):.3f}; mean per-topic score {mean:.3f}; '
          f'{full} fully supported', flush=True)
    print('worst 5:')
    for j in per[:5]:
        print(f"  {j['topic']}: {j['supported']}/{j['sentences']}")
        for c in j['unsupported']:
            print(f'    - {c}')


def trait_passages(r, P4):
    """The race's 14 passages plus up to TRAIT_EXTRA passages with a line naming the race and a body word."""
    names = [re.compile(r'\b' + re.escape(a) + r's?\b') for a in dict.fromkeys([r['topic']] + r['aliases'])]
    scored = []
    for i, c in enumerate(P4):
        if c['chunkId'] in r['passages']:
            continue
        n = sum(1 for line in c['text'].split('\n') if BODY_RE.search(line) and any(p.search(line) for p in names))
        if n:
            scored.append((-n, i))
    return r['passages'] + [P4[i]['chunkId'] for _, i in sorted(scored)[:TRAIT_EXTRA]]


def stage_retrait():
    """Every served race topic, and every row marked traitAudit, regenerated into summaryV2 with TRAIT_SYS over its
    passagesV2 (keyed on input and prompt); only ask --lore v2 serves summaryV2."""
    if os.environ.get('TOPIC_TRAIT_RULE', '1') == '0':
        print('retrait: off (TOPIC_TRAIT_RULE=0)'); return
    path = os.path.join(OUT, 'topics.jsonl')
    rows = read_jsonl(path)
    P4, _ = p4_search()
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    v2in = lambda r: gen_input(dict(r, passages=r['passagesV2']), chunks)
    for r in rows:
        if (r.get('kind') == 'race' and r.get('summary') and not r.get('thin')) or r.get('traitAudit'):
            r['passagesV2'] = trait_passages(r, P4)
    todo = [r for r in rows if r.get('passagesV2') and (r.get('promptShaV2') != TSHA or r.get('inputShaV2') != sha16(v2in(r)))]
    if PLAN:
        print(f"topics retrait: plan {len(todo)} to do {[r['topic'] for r in todo]}", flush=True); return
    for r in todo:
        user = v2in(r)
        r['summaryV2'] = chat(user, TRAIT_SYS); r['inputShaV2'] = sha16(user); r['promptShaV2'] = TSHA; r.pop('summaryV2Gen', None); r.pop('traitCheckKey', None)
        with open(path, 'w') as f:
            for x in rows:
                f.write(json.dumps(x, ensure_ascii=False) + '\n')
        print(f"retrait {r['topic']}: {len(r['summaryV2'].split())} words", flush=True)


TRAIT_CHECK_SYS = ("You check one sentence of a summary about a race of the Arknights world against the passages it cites. The "
                   "sentence describes a body feature. Answer true only if the passages give that feature to members of this race "
                   "(in narration, a record, or a character describing their own race), not to one person whose race the passage "
                   "does not give, not to another race, and not in a figure of speech. Answer true or false only.")
CSHA2 = sha16(TRAIT_CHECK_SYS)


def stage_traitcheck():
    """Judge model: each body-feature sentence of a race's summaryV2 against the passages it cites; a sentence that fails
    is removed (the generated text stays in summaryV2Gen). 2026-10-02: Durin got Tomimi's "thick tail" and Savra the horn
    a Savra girl sees on someone else. TOPIC_TRAIT_CHECK=0 leaves summaryV2 as generated."""
    path = os.path.join(OUT, 'topics.jsonl')
    rows = read_jsonl(path)
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    off = os.environ.get('TOPIC_TRAIT_CHECK', '1') == '0'
    n_checked = n_dropped = 0
    for r in rows:
        gen = r.get('summaryV2Gen') or r.get('summaryV2')
        if not gen or r.get('kind') != 'race':
            continue
        r['summaryV2Gen'] = gen
        if off:
            r['summaryV2'] = gen; r.pop('traitDropped', None); continue
        key = sha16(gen + CSHA2)
        if r.get('traitCheckKey') == key:
            continue
        parts = re.split(r'(?<=[.!?])\s+', gen)
        dropped = []
        for i, x in enumerate(parts):
            if not BODY_RE.search(x):
                continue
            cites = [int(n) for g in re.findall(r'\[([\d,;\s]+)\]', x) for n in re.split(r'[,;\s]+', g) if n.strip().isdigit()]
            ps = r['passagesV2']
            ev = '\n---\n'.join(chunks[ps[c - 1]]['text'][:3000] for c in cites if 0 < c <= len(ps) and ps[c - 1] in chunks)
            claim = re.sub(r'\s*\[[\d,;\s]+\]', '', x)
            body = json.dumps({'messages': [{'role': 'system', 'content': TRAIT_CHECK_SYS},
                                            {'role': 'user', 'content': f"RACE: {r['topic']}\nPASSAGES:\n{ev or '(none cited)'}\n\nSENTENCE: {claim}"}],
                               'temperature': 0, 'seed': 1, 'max_tokens': 3, 'grammar': 'root ::= "true" | "false"\n'}).encode()
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=600) as resp:
                ok = json.load(resp)['choices'][0]['message']['content'].strip() == 'true'
            n_checked += 1
            if not ok:
                dropped.append(i)
        r['summaryV2'] = ' '.join(x for i, x in enumerate(parts) if i not in dropped)
        r['traitDropped'] = [parts[i] for i in dropped]
        r['traitCheckKey'] = key
        n_dropped += len(dropped)
        if dropped:
            print(f"traitcheck {r['topic']}: dropped {len(dropped)}: " + ' | '.join(parts[i][:120] for i in dropped), flush=True)
    with open(path, 'w') as f:
        for x in rows:
            f.write(json.dumps(x, ensure_ascii=False) + '\n')
    print(f'traitcheck: {n_checked} body-feature sentences checked, {n_dropped} dropped', flush=True)


TRAIT_CHECK2_SYS = ("You check one sentence of a summary about a race of the Arknights world against the lines of the passages "
                    "it cites that name the feature, each with the line before it. A line is \"Speaker: words\" or narration. Decide "
                    "WHO has the feature in those lines. Answer true only if the lines give the feature to members of this race as a "
                    "group, or to a person the lines themselves show to be of this race (narration, a record, or the person speaking "
                    "of their own body). Answer false when the feature belongs to someone the speaker is addressing or describing "
                    "whose race the lines do not show to be this one (a person of this race talking about another person's tail or "
                    "horn does not give this race that feature), to another race, or is a figure of speech. Answer true or false only.")
CSHA3 = sha16(TRAIT_CHECK2_SYS)


def feature_lines(text, words, limit=14):
    """The lines of a passage that name one of `words`, each after the line before it (who is talking to whom)."""
    lines, out = text.split('\n'), []
    for i, line in enumerate(lines):
        if any(re.search(rf'\b{re.escape(w)}', line, re.I) for w in words):
            if i > 0 and lines[i - 1] not in out:
                out.append(lines[i - 1])
            if line not in out:
                out.append(line)
    return out[:limit]


def stage_traitcheck2():
    """Speaker-aware trait check (2026-10-03): each body-feature sentence of a race's summaryV2 against only the lines of
    its cited passages that name the feature, with the line before each, so the judge sees who has it. Writes summaryV2s
    (summaryV2 minus the sentences this check fails) and traitDroppedS; summaryV2 is untouched, and ask serves summaryV2s
    only with --speaker-trait-check. 2026-10-02's traitcheck passed Durin "thick tails" (Inam about Tomimi's tail) and
    Savra "a horn" (a Savra girl about Philae's horn): it read whole 3,000-character passages."""
    path = os.path.join(OUT, 'topics.jsonl')
    rows = read_jsonl(path)
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    n_checked = n_dropped = 0
    for r in rows:
        base = r.get('summaryV2')
        if not base or r.get('kind') != 'race':
            continue
        key = sha16(base + CSHA3)
        if r.get('traitCheckKeyS') == key:
            continue
        parts = re.split(r'(?<=[.!?])\s+', base)
        dropped = []
        for i, x in enumerate(parts):
            words = sorted({m.lower() for m in BODY_RE.findall(x)})
            if not words:
                continue
            stems = sorted({w.rstrip('s') if len(w) > 3 else w for w in words})
            cites = [int(n) for g in re.findall(r'\[([\d,;\s]+)\]', x) for n in re.split(r'[,;\s]+', g) if n.strip().isdigit()]
            ps = r['passagesV2']
            ev = '\n---\n'.join('\n'.join(feature_lines(chunks[ps[c - 1]]['text'], stems)) for c in cites
                                if 0 < c <= len(ps) and ps[c - 1] in chunks)
            claim = re.sub(r'\s*\[[\d,;\s]+\]', '', x)
            body = json.dumps({'messages': [{'role': 'system', 'content': TRAIT_CHECK2_SYS},
                                            {'role': 'user', 'content': f"RACE: {r['topic']}\nLINES:\n{ev.strip() or '(no cited line names the feature)'}\n\nSENTENCE: {claim}"}],
                               'temperature': 0, 'seed': 1, 'max_tokens': 3, 'cache_prompt': False,
                               'grammar': 'root ::= "true" | "false"\n'}).encode()
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=600) as resp:
                ok = json.load(resp)['choices'][0]['message']['content'].strip() == 'true'
            n_checked += 1
            print(f"traitcheck2 {r['topic']} {'keep' if ok else 'DROP'}: {claim[:140]}", flush=True)
            if not ok:
                dropped.append(i)
        r['summaryV2s'] = ' '.join(x for i, x in enumerate(parts) if i not in dropped)
        r['traitDroppedS'] = [parts[i] for i in dropped]
        r['traitCheckKeyS'] = key
        n_dropped += len(dropped)
    with open(path, 'w') as f:
        for x in rows:
            f.write(json.dumps(x, ensure_ascii=False) + '\n')
    print(f'traitcheck2: {n_checked} body-feature sentences checked, {n_dropped} dropped', flush=True)


def stage_traitrule():
    """Race-trait rule without a model (2026-10-03, item 11): a body-feature sentence of a race's summaryV2 stays only when
    a cited line that names the feature gives it to this race. Per line: the speaker (the "Name:" label) and the subject
    (first person "my/our tail" = the speaker; second person "your tail" = the previous line's speaker; otherwise the
    operators the line names, or the archive's operator for an archive line with no speaker) count by their race in
    operator_attributes.jsonl; narration naming the race itself (outside the speaker label) supports the feature. A feature
    is refuted when its lines give known races and none of them is this one and no line names the race; a sentence with a
    refuted feature is dropped. Writes summaryV2r and traitDroppedR; summaryV2 is untouched, and ask serves summaryV2r only
    with --race-trait-rule. traitcheck2 (Qwen) kept both misattributions this is built for: Durin "thick tails" (Inam and
    Tomimi, Archosauria, about Tomimi's tail) and Savra "a horn" (a Savra girl about Philae's horn; Philae is Sarkaz)."""
    path = os.path.join(OUT, 'topics.jsonl')
    rows = read_jsonl(path)
    chunks = {c['chunkId']: c for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    race_of = {}
    for a in read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl')):
        if a.get('name') and a.get('race'):
            race_of[a['name'].strip("'").lower()] = a['race']
    # Operator names as written (case-sensitive), never a body word ("Horn" is a Lupo operator).
    names_cs = {a['name'].strip("'"): a['race'] for a in read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))
                if a.get('name') and a.get('race') and len(a['name'].strip("'")) >= 3 and not BODY_RE.fullmatch(a['name'].strip("'"))}
    op_pat = re.compile(r"\b(" + '|'.join(sorted((re.escape(n) for n in names_cs), key=len, reverse=True)) + r")\b")
    lab = re.compile(r"^([^:\n]{1,40}):\s")
    n_checked = n_dropped = 0
    for r in rows:
        base = r.get('summaryV2')
        if not base or r.get('kind') != 'race':
            continue
        race = r['topic']
        race_words = [re.compile(r'\b' + re.escape(a) + r"(s|'s)?\b", re.I) for a in dict.fromkeys([race] + r.get('aliases', []))]
        races_ok = {race.lower()} | {a.lower() for a in r.get('aliases', [])}
        parts = re.split(r'(?<=[.!?])\s+', base)
        dropped, notes = [], []
        for i, x in enumerate(parts):
            words = sorted({m.lower() for m in BODY_RE.findall(x)})
            if not words:
                continue
            n_checked += 1
            stems = sorted({w.rstrip('s') if len(w) > 3 else w for w in words})
            cites = [int(n) for g in re.findall(r'\[([\d,;\s]+)\]', x) for n in re.split(r'[,;\s]+', g) if n.strip().isdigit()]
            ps = r['passagesV2']
            refuted = []
            for st in stems:
                support, known = False, set()
                for c in cites:
                    if not (0 < c <= len(ps) and ps[c - 1] in chunks):
                        continue
                    ch = chunks[ps[c - 1]]
                    lines = ch['text'].split('\n')
                    owner = ch['text'].split('\n')[0][len('Operator archive: '):].strip() if ch['text'].startswith('Operator archive: ') else None
                    for j, line in enumerate(lines):
                        if not re.search(rf'\b{re.escape(st)}', line, re.I):
                            continue
                        m = lab.match(line)
                        speaker = m.group(1).strip() if m else None
                        body = line[m.end():] if m else line
                        # the race named as the feature's owner: "the Savra's tail", "Durin tails", "the horns of a Sarkaz"
                        if any(re.search(rf"{p.pattern}\s+(?:\w+\s+){{0,3}}{re.escape(st)}|{re.escape(st)}\w*\s+of\s+(?:an?\s+|the\s+)?{p.pattern}",
                                         body, re.I) for p in race_words):
                            support = True
                            continue
                        who = []
                        if speaker:
                            who.append(speaker)
                        if re.search(rf"\b(my|our)\b[^.!?]{{0,30}}\b{re.escape(st)}", body, re.I):
                            pass
                        elif re.search(rf"\b(your|you)\b[^.!?]{{0,30}}\b{re.escape(st)}", body, re.I):
                            prev = next((lab.match(lines[k]).group(1).strip() for k in range(j - 1, -1, -1)
                                         if lab.match(lines[k]) and lab.match(lines[k]).group(1).strip() != speaker), None)
                            if prev:
                                who.append(prev)
                        else:
                            who.extend(mm.group(1) for mm in op_pat.finditer(body))
                            if not speaker and owner:
                                who.append(owner)
                        for w in who:
                            rc = race_of.get(w.strip("'").lower())
                            if rc:
                                known.add(rc)
                if not support and known and not any(k.lower() in races_ok for k in known):
                    refuted.append((st, sorted(known)))
            if refuted:
                dropped.append(i)
                notes.append(f"{x[:120]} <- {refuted}")
        r['summaryV2r'] = ' '.join(x for i, x in enumerate(parts) if i not in dropped)
        r['traitDroppedR'] = [parts[i] for i in dropped]
        n_dropped += len(dropped)
        for nt in notes:
            print(f"traitrule {race} DROP: {nt}", flush=True)
    if os.environ.get('PLAN') != '1':
        with open(path, 'w') as f:
            for x in rows:
                f.write(json.dumps(x, ensure_ascii=False) + '\n')
    print(f'traitrule: {n_checked} body-feature sentences checked, {n_dropped} dropped', flush=True)


def stage_units():
    """units.jsonl: every served topic (v1's and, by default, those from data); units.v1.jsonl: the 46 v1 topics alone,
    byte for byte the units.jsonl before 2026-10-01, for the v1 lore's P4 (ask --lore v1)."""
    # A thin topic (new topics only, see stage_thin) is not served.
    rows = [r for r in read_jsonl(os.path.join(OUT, 'topics.jsonl')) if r.get('summary') and not r.get('thin')]
    ids = collections.Counter()
    for name, keep in (('units.jsonl', lambda r: True), ('units.v1.jsonl', lambda r: 'source' not in r)):
        with open(os.path.join(OUT, name), 'w') as f:
            for r in filter(keep, rows):
                # Citations point at this entry's own passage list, which a reader of the unit cannot see; drop them.
                body = r['summaryV2'] if name == 'units.jsonl' and r.get('summaryV2') else r['summary']
                text = re.sub(r'\s*\[[\d,;\s]+\]', '', body)
                sid = 'topic_' + re.sub(r'\W+', '_', r['topic']).strip('_').lower()
                ids[sid] += name == 'units.jsonl'
                f.write(json.dumps({'storyId': sid, 'groupId': 'topic',
                                    'title': f"Topic: {r['topic']} (Trevor's summary)", 'text': f"Topic: {r['topic']} (Trevor's summary)\n{text}",
                                    'speakers': []}, ensure_ascii=False) + '\n')
    dup = [k for k, v in ids.items() if v > 1]
    print(f"units: {len(rows)} ({sum('source' not in r for r in rows)} v1)" + (f'; DUPLICATE story ids {dup}' if dup else ''))


if __name__ == '__main__':
    {'plan': stage_plan, 'gen': stage_gen, 'units': stage_units, 'judge': stage_judge, 'mine': stage_mine,
     'classify': stage_classify, 'thin': stage_thin, 'retrait': stage_retrait, 'traitcheck': stage_traitcheck,
     'traitcheck2': stage_traitcheck2, 'traitrule': stage_traitrule}[sys.argv[1]]()
