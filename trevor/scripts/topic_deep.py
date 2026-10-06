#!/usr/bin/env python3
"""Deep topic entries by map-reduce (2026-10-03 night 9, Ian: the Sarkaz answer "lacks a ton", the Sami one "eeehhh").

A topic summary of topics.py reads 14 BM25 passages; the Sarkaz are named in 2,000+ chunks of the corpus. This script
gathers a served topic's evidence broadly and summarizes it in two steps:

  evidence  per topic, no model: its forms (name, aliases, merged names of 3+ characters) matched as whole words
            (case-sensitive for a capitalized form) over artifacts/p4x/chunks.jsonl, then
            - the DEEP_GROUPS (10) story groups with the most chunks naming it, each with its DEEP_PER_GROUP (6) chunks
              that name it most, in story order, and the group's P2 event summary as orientation (not citable);
            - operator files naming it, operators of that race, nation or birthplace (operator_attributes.jsonl) first,
              DEEP_RECORDS (8) chunks;
            - other game text naming it (game-text units, IS, modules, items, enemies, outfits), DEEP_RECORDS chunks;
            - the dossiers (generated) of up to 6 characters of that race, nation or birthplace, most story lines first;
            - the game storyline overviews (artifacts/overview) that name it, at most 2, for the reduce only.
  map       generator model on $SERVER: one call per evidence set -> cited notes on the topic, speakers kept, a physical
            trait only when given to the race as a whole, or NOTHING (artifacts/topics/deep_map.jsonl, keyed on input +
            prompt, so a rerun redoes only changed sets).
  reduce    the notes, numbered, with their source names -> a structured entry (What it is, Traits, History, Society and
            politics, Notable people, Key events with where to read them, Open questions), cited by note
            (artifacts/topics/deep.jsonl, keyed on the notes + prompt).
  judge     judge model on $SERVER (Qwen3.5 9B): each sentence of the entry against the notes it cites (all notes when it
            cites none) -> `supported`, `unsupported`; `served` is the entry without its unsupported sentences
            (DEEP_DROP=0 keeps them). ask serves `served` for the topic tool (`--no-deep-topics` serves the summary).

  python3 scripts/topic_deep.py gen [--top N] [--topics A,B] [--minutes M]   # map + reduce, topics most asked first
  python3 scripts/topic_deep.py judge
  PLAN=1 python3 scripts/topic_deep.py gen|judge                              # what a run would redo, no model
"""
import argparse, collections, hashlib, json, os, re, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'topics')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
PLAN = os.environ.get('PLAN') == '1'
GROUPS = int(os.environ.get('DEEP_GROUPS', '10'))
PER_GROUP = int(os.environ.get('DEEP_PER_GROUP', '6'))
RECORDS = int(os.environ.get('DEEP_RECORDS', '8'))
DROP = os.environ.get('DEEP_DROP', '1') != '0'
sha16 = lambda s: hashlib.sha256(s.encode()).hexdigest()[:16]
NOT_STORY = {'archive', 'profile', 'summary', 'topic', 'gametext', 'enemy', 'item', 'voice', 'is', 'module', 'skin'}
OTHER_TEXT = {'gametext', 'is', 'module', 'item', 'enemy', 'skin'}

MAP_SYS = ("You take notes on one topic of the Arknights world from one source (one story, or a set of operator files, "
           "records or character profiles), for a reference entry. Using only the numbered passages, write what they show "
           "about the topic: what it is, traits, history, society and politics, named people who belong to it or act for "
           "it, and events involving it. Write 3 to 10 short bullet points, each ending with the passage number as [n]. "
           "When a claim comes from what a character says rather than from narration or a record, name who says it, "
           "since characters can be wrong or biased. Name a physical trait of a race only when a passage gives it to the "
           "race as a whole; a trait of one person, of another race, or in a figure of speech is not a trait of the race. "
           "Add no outside knowledge. If the passages only mention the topic in passing and say nothing about it, write "
           "NOTHING.")
REDUCE_SYS = ("You write a reference entry on one topic of the Arknights world from numbered notes, each taken from one "
              "named source (a story, operator files, records or character profiles). Use only the notes. Write these "
              "sections, each starting with its heading on its own line, in this order: What it is; Traits; History; "
              "Society and politics; Notable people; Key events; Open questions. Under Key events, give each event with "
              "the source to read it in, by the source name the notes give. Under Open questions, say what the notes "
              "leave unclear or contested. Skip a section only when the notes say nothing for it. Cite notes as [n] after "
              "every sentence or list item. Keep a claim that a note attributes to a character attributed to them. "
              "450 to 700 words. No preamble.")
MSHA, RSHA = sha16(MAP_SYS), sha16(REDUCE_SYS)
SUPPORT_SYS = ("You check one claim about the Arknights world against evidence from the game text. Answer true only if "
               "the evidence states or clearly implies the claim. Answer false otherwise. Answer with true or false only.")
JSHA = sha16(SUPPORT_SYS)
HEADINGS = ('What it is', 'Traits', 'History', 'Society and politics', 'Notable people', 'Key events', 'Open questions')


def read_jsonl(p):
    return [json.loads(l) for l in open(p)] if os.path.exists(p) else []


def write_jsonl(p, rows):
    tmp = p + '.tmp'
    with open(tmp, 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    os.replace(tmp, p)


def call(system, user, max_tokens, grammar=None):
    body = {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
            'temperature': 0, 'seed': 1, 'max_tokens': max_tokens, 'cache_prompt': False}
    if grammar:
        body['grammar'] = grammar
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', json.dumps(body).encode(), {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=900) as r:
                return json.load(r)['choices'][0]['message']['content'].strip()
        except Exception as e:  # one failed decode 500s the request; retry
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def forms_rx(r):
    """The topic's forms as one whole-word pattern, as scripts/topic_groups.py: case-sensitive when capitalized."""
    forms = {r['topic'], *r.get('aliases', []), *r.get('names', [])}
    forms = sorted((f for f in forms if len(f) >= 3), key=len, reverse=True)
    pats = [re.escape(f) if f[0].isupper() else '(?i:' + re.escape(f) + ')' for f in forms]
    return re.compile(r"(?<![\w'])(" + '|'.join(pats) + r")(?![\w'])")


def served_topics():
    rows = [r for r in read_jsonl(os.path.join(OUT, 'topics.jsonl')) if r.get('summary') and not r.get('thin')]
    return sorted(rows, key=lambda r: (-r.get('asked', 0), r['topic']))


class Data:
    def __init__(self):
        self.chunks = read_jsonl(os.path.join(ROOT, 'artifacts', 'p4x', 'chunks.jsonl'))
        self.p2 = {g['groupId']: g['summary'] for g in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'groups.jsonl'))}
        tl = json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))
        self.gname = {g['groupId']: g.get('name') or g['groupId'] for g in tl['groups']}
        self.attrs = read_jsonl(os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl'))
        self.dossiers = read_jsonl(os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl'))
        self.overview = read_jsonl(os.path.join(ROOT, 'artifacts', 'overview', 'overview.jsonl'))

    def members(self, topic):
        t = topic.lower()
        return {a['name'].strip("'") for a in self.attrs if a.get('name') and any(
            (a.get(k) or '').lower() == t for k in ('race', 'nation', 'birthplace'))}

    def evidence(self, r):
        """The evidence sets of one topic: [(source name, kind, [chunk], orientation text)] and the overview notes."""
        rx = forms_rx(r)
        hits = [(len(rx.findall(c['text'])), c) for c in self.chunks]
        hits = [(n, c) for n, c in hits if n]
        by_group = collections.defaultdict(list)
        for n, c in hits:
            if c['groupId'] not in NOT_STORY:
                by_group[c['groupId']].append((n, c))
        sets = []
        top = sorted(by_group, key=lambda g: (-len(by_group[g]), g))[:GROUPS]
        for g in top:
            best = sorted(by_group[g], key=lambda x: (-x[0], x[1]['ordinal']))[:PER_GROUP]
            sets.append((f"story: {self.gname.get(g, g)}", 'story', sorted((c for _, c in best), key=lambda c: c['ordinal']),
                         self.p2.get(g, '')[:800]))
        mem = self.members(r['topic'])
        owner = lambda c: c['text'].split('\n')[0][len('Operator archive: '):].strip() if c['text'].startswith('Operator archive: ') else ''
        arch = sorted(((n, c) for n, c in hits if c['groupId'] == 'archive'),
                      key=lambda x: (owner(x[1]).strip("'") not in mem, -x[0], x[1]['chunkId']))[:RECORDS]
        if arch:
            sets.append(('operator files', 'archive', [c for _, c in arch], ''))
        other = sorted(((n, c) for n, c in hits if c['groupId'] in OTHER_TEXT), key=lambda x: (-x[0], x[1]['chunkId']))[:RECORDS]
        if other:
            sets.append(('game records (story intros, records, IS, modules, items, enemies)', 'records', [c for _, c in other], ''))
        dos = sorted((d for d in self.dossiers if d['name'].strip("'") in mem), key=lambda d: (-int(d.get('chunks') or 0), d['name']))[:6]
        if dos:
            fake = [{'chunkId': f"dossier:{d['name']}", 'storyId': f"dossier of {d['name']}", 'text': f"Profile of {d['name']}: {d['dossier'][:1500]}"} for d in dos]
            sets.append(('character profiles (generated from the story)', 'dossier', fake, ''))
        ov = [o for o in self.overview if o.get('kind') == 'storyline' and rx.search(o.get('summary', ''))]
        ov = sorted(ov, key=lambda o: -len(rx.findall(o['summary'])))[:2]
        return sets, [(f"storyline overview: {o['name']}", o['summary'][:1500]) for o in ov]


def map_input(topic, name, chunks, orient):
    head = f"TOPIC: {topic}\nSOURCE: {name}\n"
    if orient:
        head += f"\nORIENTATION (a generated summary of the whole story; do not cite it):\n{orient}\n"
    return head + "\nPASSAGES\n" + ''.join(f"\n[{i + 1}] ({c['storyId']})\n{c['text'][:3000]}\n" for i, c in enumerate(chunks))


def map_items(data, r):
    sets, ov = data.evidence(r)
    items = []
    for name, kind, chunks, orient in sets:
        user = map_input(r['topic'], name, chunks, orient)
        items.append({'topic': r['topic'], 'source': name, 'kind': kind, 'chunks': [c['chunkId'] for c in chunks],
                      'user': user, 'key': sha16(user + MSHA)})
    return items, ov


def reduce_input(topic, notes, ov):
    body = f"TOPIC: {topic}\n\nNOTES\n"
    n = 0
    for src, text in notes:
        n += 1
        body += f"\n[{n}] (source: {src})\n{text}\n"
    for src, text in ov:
        n += 1
        body += f"\n[{n}] (source: {src}; a generated overview)\n{text}\n"
    return body


def stage_gen(args):
    data = Data()
    rows = served_topics()
    if args.topics:
        want = [t.strip() for t in args.topics.split(',')]
        rows = [r for t in want for r in rows if r['topic'] == t]
    elif args.top:
        rows = rows[:args.top]
    mpath, dpath = os.path.join(OUT, 'deep_map.jsonl'), os.path.join(OUT, 'deep.jsonl')
    cache = {m['key']: m for m in read_jsonl(mpath)}
    deep = {d['topic']: d for d in read_jsonl(dpath)}
    t0 = time.time(); done = 0
    todo = []
    for r in rows:
        items, ov = map_items(data, r)
        miss = [m for m in items if m['key'] not in cache]
        d = deep.get(r['topic'])
        if miss or not d or d.get('mapKeys') != [m['key'] for m in items] or d.get('promptSha') != RSHA:
            todo.append((r, items, ov, len(miss)))
    if PLAN:
        print(f"topic_deep gen: plan {len(todo)} of {len(rows)} topics to do; "
              + ', '.join(f"{r['topic']} ({n} maps)" for r, _, _, n in todo[:15]), flush=True)
        return
    for r, items, ov, _ in todo:
        if args.minutes and time.time() - t0 > args.minutes * 60:
            print(f'time budget reached after {done} topics', flush=True)
            break
        for m in items:
            if m['key'] in cache:
                continue
            out = call(MAP_SYS, m['user'], 500)
            row = {k: m[k] for k in ('topic', 'source', 'kind', 'chunks', 'key')} | {'notes': out, 'promptSha': MSHA}
            cache[m['key']] = row
            with open(mpath, 'a') as f:
                f.write(json.dumps(row, ensure_ascii=False) + '\n')
        notes = [(m['source'], cache[m['key']]['notes']) for m in items if not cache[m['key']]['notes'].strip().upper().startswith('NOTHING')]
        user = reduce_input(r['topic'], notes, ov)
        entry = call(REDUCE_SYS, user, 1400)
        deep[r['topic']] = {'topic': r['topic'], 'kind': r['kind'], 'asked': r.get('asked', 0), 'entry': entry,
                            'sources': [s for s, _ in notes] + [s for s, _ in ov], 'notes': [t for _, t in notes] + [t for _, t in ov],
                            'mapKeys': [m['key'] for m in items], 'inputSha': sha16(user), 'promptSha': RSHA,
                            'chunks': sum(len(m['chunks']) for m in items)}
        write_jsonl(dpath, sorted(deep.values(), key=lambda d: (-d['asked'], d['topic'])))
        done += 1
        print(f"{time.strftime('%H:%M:%S')} {r['topic']}: {len(items)} sets, {len(notes)} with notes, "
              f"{len(entry.split())} words ({(time.time() - t0) / done:.0f} s per topic)", flush=True)
    print(f'gen done: {done} topics written', flush=True)


ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bNo\.)'


def claims(entry):
    """(line index, sentence, cited note numbers) per claim; headings are not claims."""
    out = []
    for i, line in enumerate(entry.split('\n')):
        s = line.strip().lstrip('-*• ').strip()
        if not s or s.strip('#* :').strip() in HEADINGS:
            continue
        for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'])', s):
            cites = [int(n) for g in re.findall(r'\[([\d,;\s]+)\]', x) for n in re.split(r'[,;\s]+', g) if n.strip().isdigit()]
            plain = re.sub(r'\s*\[[\d,;\s]+\]', '', x).strip()
            if len(plain) > 20:
                out.append((i, x, plain, cites))
    return out


def verdict(evidence, claim):
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


def served_text(entry, unsupported):
    """The entry without its unsupported sentences, headings kept when a claim stays under them."""
    bad = set(unsupported)
    lines = []
    for line in entry.split('\n'):
        kept = line
        for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'])', line.strip()):
            if x.strip().lstrip('-*• ').strip() in bad:
                kept = kept.replace(x.strip().lstrip('-*• ').strip(), '')
        kept = re.sub(r'\s{2,}', ' ', kept).rstrip()
        if kept.strip().lstrip('-*• ').strip():
            lines.append(kept)
    out, pending = [], None
    for line in lines:  # a heading whose section lost every claim goes too
        if line.strip('#* :').strip() in HEADINGS:
            pending = line
            continue
        if pending:
            out.append(pending); pending = None
        out.append(line)
    return '\n'.join(out)


def stage_judge(_args):
    path = os.path.join(OUT, 'deep.jsonl')
    deep = read_jsonl(path)
    todo = [d for d in deep if d.get('judgeKey') != sha16(d['entry'] + '\n'.join(d['notes']) + JSHA)]
    if PLAN:
        print(f"topic_deep judge: plan {len(todo)} to do {[d['topic'] for d in todo][:15]}", flush=True)
        return
    t0 = time.time()
    for n, d in enumerate(todo, 1):
        cs = claims(d['entry'])
        bad = []
        for _, raw, plain, cites in cs:
            idx = [c for c in cites if 0 < c <= len(d['notes'])] or list(range(1, len(d['notes']) + 1))
            ev = '\n---\n'.join(f"(source: {d['sources'][c - 1]})\n{d['notes'][c - 1]}" for c in idx)
            if not verdict(ev, plain):
                bad.append(raw.strip().lstrip('-*• ').strip())
        d['sentences'], d['supported'], d['unsupported'] = len(cs), len(cs) - len(bad), bad
        d['served'] = served_text(d['entry'], bad if DROP else [])
        d['servedDrop'] = DROP
        d['judgeKey'] = sha16(d['entry'] + '\n'.join(d['notes']) + JSHA)
        write_jsonl(path, deep)
        print(f"{time.strftime('%H:%M:%S')} {d['topic']}: {d['supported']}/{d['sentences']} supported "
              f"({(time.time() - t0) / n:.0f} s per topic)", flush=True)
    J = [d for d in deep if 'supported' in d]
    s = sum(d['supported'] for d in J); t = sum(d['sentences'] for d in J)
    print(f'judge: {len(J)} entries, {s}/{t} sentences supported = {s / max(t, 1):.3f}', flush=True)


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('stage', choices=['gen', 'judge', 'evidence'])
    ap.add_argument('--top', type=int, default=0)
    ap.add_argument('--topics', default='')
    ap.add_argument('--minutes', type=float, default=0)
    a = ap.parse_args()
    if a.stage == 'evidence':  # no model: print the evidence sets of the chosen topics
        data = Data()
        for r in served_topics():
            if a.topics and r['topic'] not in a.topics.split(','):
                continue
            items, ov = map_items(data, r)
            print(r['topic'], [(m['source'], len(m['chunks']), len(m['user'])) for m in items], [s for s, _ in ov])
            if not a.topics:
                break
        sys.exit()
    {'gen': stage_gen, 'judge': stage_judge}[a.stage](a)
