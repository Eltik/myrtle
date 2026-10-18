#!/usr/bin/env python3
"""P3: who is who. Links real names, codenames, aliases and stage names to speaker labels.

The script labels a character by one display name (Haruka Shino speaks as "Momoka" in 66
chunks; "Haruka" appears in 10), so a question using another name misses most of the
character. Stages (artifacts/entities/, resumable):
  cues     deterministic: lines stating an identity ("real name", "also known as",
           "codename", "call me", ...) with one line of context      -> cues.jsonl
  extract  generator model: the two names, the relation and a verbatim quote; kept only
           when the quote is in the lines and both names appear there -> pairs.jsonl
  build    union of verified pairs into identities, keyed by speaker labels -> identities.json
  extract2 (2026-10-03) the same extraction over P4's game-text units and stories (voice lines, modules, IS,
           enemies, outfits, items; never Trevor's generated dossiers, summaries or topics) with more cues
           (CUE2: "knew her as", "the deceased Operator", "her name was", ...), only for lines pairs.jsonl lacks
           -> pairs.v2.jsonl (resumable). Iris's file "Mabel ... was ... the deceased Operator Bluishsilver" had no
           cue, so Bluishsilver and Mabel were never linked.
  ENTITY_V2=1 build: also reads pairs.v2.jsonl and writes identities.v2.json, with the links whose quote does not
           state them (dropped by build) and same-story full names ("Mabel Grimm" beside the linked "Mabel") kept apart
           as labelled inferences under "inferred"; without it build writes identities.json exactly as before.
  who NAME print the identity a name belongs to, with evidence and where it speaks.
Corpus: artifacts/p3a (stories plus operator archives).
"""
import collections, json, os, re, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common, incr
from common import ROOT, read_jsonl, sha16

CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3a'))
OUT = os.path.join(ROOT, 'artifacts', 'entities')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
SLOTS = int(os.environ.get('SLOTS', '2'))

CUE = re.compile(r"\b(real name|true name|birth name|given name|my name is|her name is|his name is|name's|also known as|"
                 r"a\.k\.a\.|codename|code name|goes by|went by|formerly known|used to be called|call me|called herself|"
                 r"called himself|alias|stage name|pen name|under the name|under the stage name|born as|born|n\u00e9e|known as|"
                 r"renamed|changed (?:her|his|their) name)\b", re.I)
SYS = ("You read a few lines from Arknights (a story script or an operator archive) where someone's identity may be "
       "stated: a real name, a codename, an alias, a stage name, a former name or a title. Extract each link between two "
       "names for the SAME person that the lines actually state. Give name_a and name_b exactly as written, the relation "
       "of name_b to name_a (real_name, codename, alias, stage_name, former_name, title), and a short verbatim quote that "
       "states the link. Speaker labels at the start of a line (\"Momoka:\") are names too. Do not guess: if the lines do "
       "not state that two names are the same person, return an empty list. Never use outside knowledge. Example: from the "
       "line 'Profile: Cora, born Coralie Vance, sings under the stage name Blue Tide.' the links are Cora ~ Coralie Vance "
       "(real_name) and Cora ~ Blue Tide (stage_name).")
GRAMMAR = r'''root ::= "{\"links\": [" ( link ( ", " link ){0,3} )? "]}"
link ::= "{\"name_a\": " n ", \"name_b\": " n ", \"relation\": " rel ", \"quote\": " q "}"
rel ::= "\"real_name\"" | "\"codename\"" | "\"alias\"" | "\"stage_name\"" | "\"former_name\"" | "\"title\""
n ::= "\"" ch{1,60} "\""
q ::= "\"" ch{4,200} "\""
ch ::= [^"\\\x00-\x1F]
'''


def norm(s):
    t = str.maketrans({'‘': "'", '’': "'", '“': '"', '”': '"', '–': '-', '—': '-', '…': '.'})
    return ' '.join(s.translate(t).split()).lower()


def chunks():
    return [json.loads(l) for l in open(os.path.join(CORPUS, 'chunks.jsonl'))]


def cue_rows(cs):
    for c in cs:
        lines = c['text'].split('\n')
        for i, x in enumerate(lines):
            if CUE.search(x):
                ctx = '\n'.join(lines[max(0, i - 1):i + 2])
                yield {'chunkId': c['chunkId'], 'storyId': c['storyId'], 'line': x, 'context': ctx}


def stage_cues():
    os.makedirs(OUT, exist_ok=True)
    n = 0
    with open(os.path.join(OUT, 'cues.jsonl'), 'w') as f:
        for r in cue_rows(chunks()):
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
            n += 1
    print('identity cue lines', n)


def extract_stage():
    # Cues are recomputed from the corpus (the function `cues` writes with), so a changed chunk changes
    # this stage's input even before `cues` reruns. The model input is the cue's context lines.
    cues = list(cue_rows(chunks()))
    st = incr.Stage('entities extract', os.path.join(OUT, 'pairs.jsonl'), lambda r: (r['chunkId'], r['line']), sha16(SYS + GRAMMAR))
    for c in cues:
        st.unit((c['chunkId'], c['line']), c['context'])
    return st, [(c['chunkId'], c['line']) for c in cues], None, cues


def extract_one(c, psha):
    out = common.chat(SERVER, SYS, c['context'], 400, 600, common.content, grammar=GRAMMAR)
    try:
        links = json.loads(out)['links']
    except Exception:
        links = []
    ctx = norm(c['context'])
    # A name is a name: two or more lowercase words ("Victoria's nomadic villages", "child of the House
    # of Rostov") means a description, not a name (review, 2026-09-27). "Duke of Cumberland" keeps.
    def is_name(x):
        return sum(1 for w in x.split() if w[:1].islower()) < 2
    links = [l for l in links if is_name(l['name_a']) and is_name(l['name_b'])] + \
            [dict(l, dropped='not a name') for l in links if not (is_name(l['name_a']) and is_name(l['name_b']))]
    links = [l for l in links if 'dropped' not in l]
    kept = [l for l in links if norm(l['quote']).rstrip('.!?, ') in ctx
            and norm(l['name_a']) in ctx and norm(l['name_b']) in ctx and norm(l['name_a']) != norm(l['name_b'])]
    return dict(c, links=kept, rejected=len(links) - len(kept), promptSha=psha)


def stage_extract():
    st, order, live, cues = extract_stage()
    keys = st.plan(order, live)
    if keys is None:
        return
    if incr.IDS_ONLY:
        todo = [c for c in cues if (c['chunkId'], c['line']) not in st.have]
    else:
        first = {}
        for c in cues:
            first.setdefault((c['chunkId'], c['line']), c)
        todo = [first[k] for k in keys]
    psha = st.psha
    print(f'extract: {len(todo)} to do, {len(order) - len(todo)} done, prompt {psha}', flush=True)
    lock = threading.Lock(); n = [0]; t0 = time.time()

    def one(c):
        row = extract_one(c, psha)
        with lock:
            st.put(row)
            n[0] += 1
            if n[0] % 100 == 0:
                print(f'{time.strftime("%H:%M:%S")} extract {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.1f} s each', flush=True)
    common.par(todo, one, SLOTS, lock)
    rows = read_jsonl(st.path)
    print(f"extract: {sum(len(r['links']) for r in rows)} verified links from {len(rows)} lines, "
          f"{sum(r['rejected'] for r in rows)} rejected", flush=True)


CUE2 = re.compile(r"\b(knew (?:her|him|them) as|know (?:her|him|them) as|known to (?:us|you|me|them) as|"
                  r"(?:late|deceased|former|fallen) Operator|went under the codename|operated under|her name was|"
                  r"his name was|their name was|real name's|true identity|is actually|was actually|used the name|"
                  r"took the name)\b", re.I)
GENERATED = ('profile', 'summary', 'topic')


def stage_extract2():
    p4 = os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl')
    have = {(r['chunkId'], r['line']) for r in read_jsonl(os.path.join(OUT, 'pairs.jsonl'))}
    path = os.path.join(OUT, 'pairs.v2.jsonl')
    done = {(r['chunkId'], r['line']) for r in read_jsonl(path)}
    todo, seen = [], set()
    for l in open(p4):
        c = json.loads(l)
        if c['groupId'] in GENERATED:
            continue
        lines = c['text'].split('\n')
        for i, x in enumerate(lines):
            k = (c['chunkId'], x)
            if (CUE.search(x) or CUE2.search(x)) and k not in have and k not in done and k not in seen:
                seen.add(k)
                todo.append({'chunkId': c['chunkId'], 'storyId': c['storyId'], 'line': x,
                             'context': '\n'.join(lines[max(0, i - 1):i + 2])})
    psha = sha16(SYS + GRAMMAR)
    print(f'extract2: {len(todo)} to do, {len(done)} done, prompt {psha}', flush=True)
    lock = threading.Lock(); n = [0]; t0 = time.time()
    out = open(path, 'a')

    def one(c):
        row = extract_one(c, psha)
        with lock:
            out.write(json.dumps(row, ensure_ascii=False) + '\n'); out.flush()
            n[0] += 1
            if n[0] % 100 == 0:
                print(f'{time.strftime("%H:%M:%S")} extract2 {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.1f} s each', flush=True)
    common.par(todo, one, SLOTS, lock)
    out.close()
    rows = read_jsonl(path)
    print(f"extract2: {sum(len(r['links']) for r in rows)} verified links from {len(rows)} lines", flush=True)


PLACEHOLDER = re.compile(r"^(?:code ?name|real name|name|alias|title|nickname|unknown|\?+)$", re.I)


def group_words():
    """Folded names of groups, not people (2026-10-16, probe question ian34 and the side issue after it): races, nations and
    birthplaces of operator_attributes, the handbook teams (organizations, nations) and enemy races of the game tables,
    and every served topic name and alias. A link with one of them is not a person's identity: "Blacksteel" ~
    "Blacksteel Worldwide" made an organization a person (so topics.py never mined "Blacksteel"), and Lucilla ~ Aegir,
    Tessai ~ Oni, Gordon Tannen ~ Savra, Kouhei Mifune ~ Phidia, Sui ~ Feranmut joined a person to their race or kind.
    ENTITY_GROUP_WORDS=0 keeps every link as before."""
    if os.environ.get('ENTITY_GROUP_WORDS') == '0':
        return set()
    words = set()
    for r in read_jsonl(os.path.join(OUT, 'operator_attributes.jsonl')):
        for k in ('race', 'nation', 'birthplace'):
            for x in re.split(r'[/,]', r.get(k) or ''):
                words.add(norm(re.sub(r'\s*\([^)]*\)', '', x)))
    try:
        for t in json.load(open(os.path.join(common.EXCEL, 'handbook_team_table.json')))['Handbook_teams']:
            words |= {norm(t['value']['PowerName']), norm(t['value']['PowerCode'])}
        for r in json.load(open(os.path.join(common.EXCEL, 'enemy_handbook_table.json')))['RaceData']:
            words.add(norm(r['value']['RaceName']))
    except FileNotFoundError:
        print(f'group words: no game tables at {common.EXCEL} (set TREVOR_GAMEDATA); attributes and topics only')
    for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl')) or []:
        words |= {norm(a) for a in [r['topic'], *r.get('aliases', [])]}
    return {w for w in words if len(w) >= 3}


PRONOUNS = {'i', 'me', 'my', 'you', 'he', 'him', 'she', 'her', 'they', 'them', 'we', 'us', 'it'}


def common_noun(name, quote):
    """A one-word name the quote has only in lower case ("Knight" in "her knight title")."""
    return (' ' not in name and re.search(rf'\b{re.escape(name)}\b', quote) is None
            and re.search(rf'\b{re.escape(name.lower())}\b', quote) is not None)


def addressed(name, quote):
    """A name the quote gives the listener: "your standards as the Black Knight", "you as the Doctor"."""
    bare = re.sub(r'^(?i:the)\s+', '', name)
    return re.search(rf"\byou(?:r(?:\s+[\w'-]+){{1,2}})?\s+as\s+(?:(?i:the)\s+)?{re.escape(bare)}\b", quote) is not None


def title_prefix(title, name, quote, lower):
    """A common word (one the corpus writes in lower case 20 times or more) written only as a title before the other
    name: "Operator Ebenholz.", "Code Name: Operator Ron". "Brandt Reiner" and "Texas the Omertosa" are kept."""
    return (' ' not in title and lower[title.lower()] >= 20
            and re.search(rf'\b{re.escape(title)} {re.escape(name)}\b', quote) is not None
            and re.search(rf'\b{re.escape(title)}\b(?! {re.escape(name)})', quote) is None)


def stage_build():
    cs = chunks()
    lower = collections.Counter(w for c in cs for w in re.findall(r"\b[a-z][a-z'\-]+\b", c['text']))
    groupw = group_words()
    grouped = 0
    speaks = collections.Counter(p for c in cs for p in set(c['speakers']))
    parent = {}
    def find(x):
        parent.setdefault(x, x)
        while parent[x] != x:
            parent[x] = parent[parent[x]]; x = parent[x]
        return x
    evidence = collections.defaultdict(list)
    is_name = lambda x: sum(1 for w in x.split() if w[:1].islower()) < 2
    dropped = 0; unstated = []
    v2 = os.environ.get('ENTITY_V2') == '1'
    pair_rows = read_jsonl(os.path.join(OUT, 'pairs.jsonl')) + (read_jsonl(os.path.join(OUT, 'pairs.v2.jsonl')) if v2 else [])
    for r in pair_rows:
        for l in r['links']:
            a, b = l['name_a'].strip(), l['name_b'].strip()
            # A field label is not a name: "[Code Name] Almond" and "[Code Name] Amiya" joined Almond and
            # Mulberry to Amiya through the node "Code Name" (found 2026-09-28; 2 of 159 identities).
            if not (is_name(a) and is_name(b)) or PLACEHOLDER.match(a) or PLACEHOLDER.match(b):
                dropped += 1
                continue
            if norm(a) in groupw or norm(b) in groupw:
                grouped += 1
                continue
            # The quote itself must state the link: an identity cue, or both names. "I can see you, Raidian."
            # only addresses someone and had merged Raidian into Kal'tsit (review, 2026-09-27).
            qn = norm(l['quote'])
            if not (CUE.search(l['quote']) or (norm(a) in qn and norm(b) in qn)):
                unstated.append((a, b, l['quote'], r['storyId'], l['relation']))
                continue
            # A one-word name the quote writes only in lower case is a common noun ("her knight title" had joined Knight
            # to Flametail), and a name after "your ... as" belongs to the person addressed, not the speaker ("By your
            # standards as the Black Knight" had joined Degenbrecher's title to Trilby Asher). 3 of 263 links, 2026-10-19.
            # A pronoun is not a name either: "Call me Brigid." had joined I to Brigid, Mudrock, Sand Reckoner and
            # Doc. Aak (8 links in pairs and pairs.v2). Nor is a one-word name in lower case ("call me 'director'",
            # "not called 'traitor'", "a codename"), or a common word written as a title before the other name
            # ("Operator Ebenholz." had joined Operator to Ebenholz, so any question saying "the operator" drew an
            # Ebenholz scene). ENTITY_LINKS_V1=1 keeps them all.
            if os.environ.get('ENTITY_LINKS_V1') != '1' and (
                    any(common_noun(n, l['quote']) or addressed(n, l['quote']) or n.lower() in PRONOUNS
                        or (' ' not in n and n[:1].islower()) for n in (a, b))
                    or title_prefix(a, b, l['quote'], lower) or title_prefix(b, a, l['quote'], lower)):
                unstated.append((a, b, l['quote'], r['storyId'], l['relation']))
                continue
            parent[find(a)] = find(b)
            evidence[a].append({'other': b, 'relation': l['relation'], 'quote': l['quote'], 'where': r['storyId']})
            evidence[b].append({'other': a, 'relation': l['relation'], 'quote': l['quote'], 'where': r['storyId']})
    groups = collections.defaultdict(set)
    for x in list(parent):
        groups[find(x)].add(x)
    ids = []
    for members in groups.values():
        label = max(sorted(members), key=lambda m: (speaks.get(m, 0), len(m)))  # sorted: a tie no longer depends on the hash seed
        ids.append({'label': label, 'speaksInChunks': speaks.get(label, 0), 'names': sorted(members),
                    'evidence': {m: evidence[m][:4] for m in sorted(members)}})
    ids.sort(key=lambda i: -i['speaksInChunks'])
    print(f'links dropped as not a name: {dropped}; as naming a group, not a person: {grouped}; as not stated by the quote: {len(unstated)}')
    if v2:
        # Labelled inferences, never merged into an identity: the links whose quote does not state them, and a full name
        # whose first word is a linked name in the same story ("Mabel Grimm" in the interlude where Iris looks for the
        # Mabel her file links to Bluishsilver), when no other identity has a name with that first word.
        inferred = [{'name_a': a, 'name_b': b, 'relation': rel, 'quote': q, 'where': w,
                     'basis': 'the extracted quote does not state the link'} for a, b, q, w, rel in unstated]
        linked = {norm(n): i for i in ids for n in i['names']}
        firsts = collections.Counter(norm(n).split()[0] for n in linked if ' ' not in n)
        full = re.compile(r"\b([A-Z][a-z'\-]+) ([A-Z][a-z'\-]+)\b")
        texts = [c['text'] for c in cs]
        # A word the text also writes in lower case is a common word ("Estate", "Family", "Hello", "Operator"), not a
        # name; only rare capitalized words count as a first name or a surname.
        lower = collections.Counter(w for t in texts for w in re.findall(r"\b[a-z][a-z'\-]+\b", t))
        seen_full = {}
        for c in cs:
            for m in full.finditer(c['text']):
                first, last, whole = m.group(1), m.group(2), m.group(0)
                if norm(whole) in linked or norm(first) not in linked or firsts[norm(first)] != 1:
                    continue
                if lower[first.lower()] or lower[last.lower()] or "'" in last or last.endswith('-'):
                    continue
                seen_full.setdefault((whole, first), (c, next((x for x in c['text'].split('\n') if whole in x), '')[:200]))
        for (whole, first), (c, line) in seen_full.items():
            inferred.append({'name_a': first, 'name_b': whole, 'relation': 'full_name', 'quote': line, 'where': c['storyId'],
                             'identity': linked[norm(first)]['label'],
                             'basis': 'a full name whose first word is the only identity name with that first word; not stated'})
        json.dump({'identities': ids, 'inferred': inferred}, open(os.path.join(OUT, 'identities.v2.json'), 'w'), ensure_ascii=False, indent=1)
        print(f"identities.v2.json: {len(ids)} identities, {len(inferred)} labelled inferences "
              f"({sum(1 for x in inferred if x['relation'] == 'full_name')} full names)")
        return
    json.dump(ids, open(os.path.join(OUT, 'identities.json'), 'w'), ensure_ascii=False, indent=1)
    big = [i for i in ids if len(i['names']) > 6]
    # Many names is not an error in itself: Fiammetta went by many codenames (review, 2026-09-27).
    print(f"identities: {len(ids)} from {len(parent)} names; labelled by a speaker: {sum(i['speaksInChunks'] > 0 for i in ids)}; "
          f"with more than 6 names: {len(big)} ({', '.join(i['label'] for i in big)})")


def who(name):
    ids = json.load(open(os.path.join(OUT, 'identities.json')))
    if os.environ.get('ENTITY_V2') == '1':
        ids = json.load(open(os.path.join(OUT, 'identities.v2.json')))['identities']
    q = norm(name)
    hits = [i for i in ids if any(norm(n) == q for n in i['names'])] or [i for i in ids if any(q in norm(n) for n in i['names'])]
    if not hits:
        print(f'No identity link found for {name!r}.'); return
    for i in hits[:3]:
        print(f"{name} -> {i['label']} (speaks in {i['speaksInChunks']} chunks); names: {', '.join(i['names'])}")
        for m, evs in i['evidence'].items():
            for e in evs[:2]:
                print(f"  {m} ~ {e['other']} ({e['relation']}): \"{e['quote']}\" [{e['where']}]")


if __name__ == '__main__':
    if sys.argv[1] == 'who':
        who(' '.join(sys.argv[2:]))
    else:
        {'cues': stage_cues, 'extract': stage_extract, 'extract2': stage_extract2, 'build': stage_build}[sys.argv[1]]()
