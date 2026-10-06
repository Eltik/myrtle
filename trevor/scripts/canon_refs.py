#!/usr/bin/env python3
"""Which Integrated Strategies endings later stories treat as having happened: evidence for canon questions.

Ian (2026-09-29): "Based on inferring and thinking, you should be able to determine what endings are canon."
The game never rules on it; players infer it from later stories that refer to one ending's events. This does the
same, with quotes. For each ending of IS2 to IS6 (rogue_1 to rogue_5 in the game data; the game's IS1, Ceobe's
Fungimist, is not in it) (the ending's own cutscene script, gamedata/story/obt/roguelike),
the generator summarizes what concretely happens, BM25 picks the closest passages among stories released after that
run began (main episodes, events, module stories, later IS runs; undated sources such as operator records and voice
lines are left out), and the generator judges each: does this passage refer to events of this ending, beyond the
same characters or setting? A reference is kept only with a verbatim quote from the passage.
  endings    parse the ending scripts and game fields -> artifacts/canon/endings.jsonl
  summarize  generator: what happens in each ending, and search terms (resumable)
  judge      generator: candidate passages per ending (resumable) -> artifacts/canon/refs.jsonl
  discriminate  generator, contrastive: each candidate passage (the union of every ending's top passages in a
             run) is shown with ALL of that run's endings and must pick one, "general" (the run, no specific
             ending) or "none". The first pass judged each ending alone and counted generic lines ("Are we quite
             finished playing yet?") and the same quote for several endings of one run (IS6: 4 of 5 endings shared
             quotes), which cannot tell endings apart. -> artifacts/canon/picks.jsonl
  wide       (2026-09-30) more candidates: per ending, two hybrid searches over P4 (the ending's summary; its name and
             distinctive proper names from its script), fused, kept to later dated non-IS stories, top CANON_WIDE_TOP
             (50) -> artifacts/canon/wide.jsonl. discriminate then also picks for these, into picks_wide.jsonl.
             CANON_WIDE=0 uses refs.jsonl alone (the 223 candidates of 2026-09-29).
  second     Qwen (a second model on $SERVER), the contrastive pick with a stricter question (an outcome unique to the
             ending, not a shared character or image) and the same grammar and quote check, on every passage Gemma gave a
             specific ending -> artifacts/canon/picks_qwen_strict.jsonl (CANON_STRICT=0: Gemma's question, into
             picks_qwen.jsonl). A pick is agreed when both name the same ending, both quotes are found, Gemma's runs to 4
             words, and the passage names one of the ending's distinctive names (CANON_ANCHOR=0 drops that); build lists
             agreed stories in referencedBy and the rest of Gemma's in gemmaOnly.
             CANON_AGREE=0 drops the requirement; CANON_WIDE=0 CANON_AGREE=0 reproduces the 2026-09-29 file.
  build      -> artifacts/canon/is_endings.json (from picks when they exist, else from the first pass), per run: endings with the game's priority, the later stories
             that refer to each (with quotes), and endings no later story refers to
  inputs     (2026-10-01, for scripts/update.sh) compare what this pipeline reads with the stamp of its last run,
             artifacts/canon/inputs.json: 'endings' (the run table, the ending scripts, every prompt and setting) and
             one sha per candidate chunk of P4 (Trevor's own summaries, dossiers and topics are never candidates and are
             left out), plus the release-date and name inputs. Prints the plan: nothing changed, an incremental run
             (chunks changed, removed or new; the endings unchanged) or a full run (the endings changed). Writes nothing.
  prune      before an incremental run: drop the rows of changed or removed chunks from the per-passage and per-story
             files, so their (run, chunk) and (run, story) resume keys are judged again; new chunks need nothing.
             Before a full run update.sh moves every output aside instead.
  stamp      after a successful run (or once, to vouch for the current outputs): write inputs.json.
Needs a llama-server on $SERVER for summarize and judge.
"""
import collections, datetime, hashlib, json, math, os, re, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import GAMEDATA as GD, ROOT, kv, read_jsonl, story_script as script_text

OUT = os.path.join(ROOT, 'artifacts', 'canon')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
TOP = int(os.environ.get('CANON_TOP', '12'))
WIDE = os.environ.get('CANON_WIDE') != '0'
WIDE_TOP = int(os.environ.get('CANON_WIDE_TOP', '50'))
AGREE = os.environ.get('CANON_AGREE') != '0'
# Undated sources (operator archives, voice lines, outfits, operator records) have no release date, so the dated stages
# leave them out; the undated stage searches them per ending with no date filter (2026-09-30). CANON_UNDATED=0 leaves
# them out of discriminate, second and the evidence lists.
UNDATED = os.environ.get('CANON_UNDATED') != '0'
UNDATED_GROUPS = ('archive', 'voice', 'skin')  # plus every story_* group (operator records)
UNDATED_TOP = int(os.environ.get('CANON_UNDATED_TOP', '30'))
# The second opinion asks the stricter question below (2026-09-30: with the first question, 7 of 21 agreed picks read as
# right); CANON_STRICT=0 asks the first question, into picks_qwen.jsonl.
STRICT = os.environ.get('CANON_STRICT') != '0'
# An agreed pick must also name one of the ending's distinctive names (wide.jsonl 'terms'): both models credited Stella
# Caerula to Lone Trail and Chapter 13 for star imagery and Precious Days to Estelle's module for a 'failed subject'
# (2026-09-30 review). CANON_ANCHOR=0 drops the requirement.
ANCHOR = os.environ.get('CANON_ANCHOR') != '0'
# Which evidence fills referencedBy when CANON_AGREE is on: 'passage' (Gemma and Qwen agree on a passage, the filters
# above), 'story-gemma', 'story-qwen' or 'story-agreed' (the story stage). Hand-graded 2026-09-30 by (ending, event):
# passage 5 of 7 right (1 unclear), story-gemma 2 of 8, story-qwen 2 of 7, story-agreed 1 of 3; the others are kept in
# their own fields.
EVIDENCE = os.environ.get('CANON_EVIDENCE', 'passage')
STORY_SCHEMES = ('story-gemma', 'story-qwen', 'story-agreed')
# Which verdict attempt build writes as each run's `verdict` (tag and prompt in verdicts.jsonl), and the second opinion
# it is compared with. CANON_VERDICT=0 leaves verdict and evidence out.
VERDICT = os.environ.get('CANON_VERDICT', f"gemma-on:{os.environ.get('CANON_VERDICT_PROMPT', 'v3')}")
VERDICT_SECOND = os.environ.get('CANON_VERDICT_SECOND', f"qwen-off:{os.environ.get('CANON_VERDICT_PROMPT', 'v3')}")
# The models said 'strong' on one story (2026-09-30, 4 of 5 runs). The written confidence is the lower of the model's and
# a rule over the evidence the verdict cites for its own ending: no cited story none, one weak, two moderate, three or
# more strong only when they span at least two events (else moderate). CANON_CONF_RULE=0 writes the model's alone.
CONF_RULE = os.environ.get('CANON_CONF_RULE') != '0'
CONF_ORDER = ['none', 'weak', 'moderate', 'strong']


def rule_confidence(verdict, run_items, cites):
    """(confidence, cited storyIds, cited events) from the evidence items the verdict cites for its own ending."""
    its = [it for it in run_items if it['endingId'] == verdict['endingId'] and it['story'] in cites]
    sids = sorted({it['storyId'] for it in its}); events = sorted({it['story'] for it in its})
    n = len(sids)
    conf = 'none' if n == 0 else 'weak' if n == 1 else 'moderate' if n == 2 or len(events) < 2 else 'strong'
    return conf, sids, events
SECOND = 'picks_qwen_strict.jsonl' if STRICT else 'picks_qwen.jsonl'
SUM_SYS = ("You read the ending cutscene of a run of Integrated Strategies, a roguelike mode of Arknights. In two or three "
           "sentences, say concretely what happens in this ending: who is involved, what they do, and what changes as a "
           "result. Then, on a new line starting 'TERMS:', list 5 to 10 distinctive names and phrases a later story would "
           "use if it referred to these events.")
JUDGE_SYS = ("You compare one ending of Integrated Strategies (a roguelike mode of Arknights) with a passage from a story "
             "released later. refers is true only if the passage mentions or clearly assumes events that happen "
             "specifically in this ending, not just the same characters, places, or the mode's general setting. quote is "
             "the exact sentence from the passage that shows it, or an empty string.")
GRAMMAR = r'''root ::= "{\"refers\": " ( "true" | "false" ) ", \"quote\": " qstr "}"
qstr ::= "\"" [^"\n]{0,300} "\""
'''


def post(system, user, max_tokens, grammar=None):
    return common.chat(SERVER, system, user, max_tokens, 900, grammar=grammar or None)


norm = common.word_norm


def p4_chunks():
    return read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))


def p4_by_id():
    return {c['chunkId']: c for c in p4_chunks()}


def summarized_endings():
    """The endings with a summary (the ones every later stage works on)."""
    return [r for r in read_jsonl(os.path.join(OUT, 'endings.jsonl')) if r.get('summary')]


def timeline_names():
    """groupId -> event name from chronology v1."""
    return {g['groupId']: g['name'] for g in json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))['groups']}


def stage_endings():
    os.makedirs(OUT, exist_ok=True)
    t = json.load(open(os.path.join(GD, 'excel', 'roguelike_topic_table.json')))
    topics = {x['key']: x['value'] for x in t['Topics']}
    rows = []
    for d in t['Details']:
        rid = d['key']; n = rid.split('_')[-1]
        start = datetime.datetime.utcfromtimestamp(topics[rid]['StartTime']).strftime('%Y-%m-%d')
        for e in kv(d['value']['Endings']).values():
            m = re.search(r'ending_(\d+)$', e['Id'])
            path = os.path.join(GD, 'story', 'obt', 'roguelike', f'ro{n}', f'level_rogue{n}_ending_{m.group(1)}.txt.txt') if m else None
            script = script_text(path) if path and os.path.exists(path) else ''
            rows.append({'run': rid, 'runName': topics[rid].get('Name'), 'start': start, 'endingId': e['Id'],
                         'name': e.get('Name'), 'priority': e.get('Priority'), 'desc': e.get('Desc'), 'script': script})
    common.write_jsonl(os.path.join(OUT, 'endings.jsonl'), rows)
    print(f"endings: {len(rows)} ({sum(bool(r['script']) for r in rows)} with a script); runs {sorted({r['run'] for r in rows})}")


def stage_summarize():
    path = os.path.join(OUT, 'endings.jsonl')
    rows = read_jsonl(path)
    for r in rows:
        if r.get('summary') or not (r['script'] or r['desc']):
            continue
        text = f"Ending: {r['name']}\nIn-game description: {r['desc']}\n\nCutscene:\n{r['script'][:12000]}"
        out = post(SUM_SYS, text, 300)
        summary, _, terms = out.partition('TERMS:')
        r['summary'] = summary.strip(); r['terms'] = terms.strip()
        common.write_jsonl(path, rows)
        print(f"{r['endingId']}: {r['summary'][:100]}", flush=True)


def release_dates():
    """Release date per corpus chunk: events from spoiler.jsonl, main episodes from the game-data dates (main_release.json, reading_guide.py dates; the wiki dates before 2026-10-04), modules from
    uniequip_table, IS units from their run's start. Undated sources return None and are skipped."""
    rel = {}
    for x in read_jsonl(os.path.join(ROOT, 'artifacts', 'p4', 'spoiler.jsonl')):
        t = x.get('groupStartTime')
        if t not in (None, -1, '-1') and int(t) > 1_500_000_000:
            rel[x['groupId']] = datetime.datetime.utcfromtimestamp(int(t)).strftime('%Y-%m-%d')
    for e in json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'main_release.json'))):
        if e.get('global'):
            rel[f"main_{e['episode']}"] = e['global']
    mods = kv(json.load(open(os.path.join(GD, 'excel', 'uniequip_table.json')))['EquipDict'])
    t = json.load(open(os.path.join(GD, 'excel', 'roguelike_topic_table.json')))
    runs = {x['key']: datetime.datetime.utcfromtimestamp(x['value']['StartTime']).strftime('%Y-%m-%d') for x in t['Topics']}

    def date(c):
        g, s = c['groupId'], c['storyId']
        if g == 'module':
            ts = int(mods.get(s[len('module_'):], {}).get('UniEquipGetTime') or 0)
            return datetime.datetime.utcfromtimestamp(ts).strftime('%Y-%m-%d') if ts > 1_500_000_000 else None
        if g == 'is':
            m = re.match(r'is_(rogue_\d+)_', s)
            return runs.get(m.group(1)) if m else None
        if g in ('archive', 'profile', 'summary', 'voice', 'skin', 'item', 'enemy') or g.startswith('story_'):
            return None
        return rel.get(g)
    return date


def bm25(query, docs, k):
    return [i for s, i in common.bm25_rank(query, docs)[:k] if s > 0]


def stage_judge():
    endings = summarized_endings()
    chunks = p4_chunks()
    date = release_dates()
    dated = [(c, date(c)) for c in chunks]
    path = os.path.join(OUT, 'refs.jsonl')
    done = {(r['endingId'], r['chunkId']) for r in read_jsonl(path)}
    for e in endings:
        # Stories released after the run began, not the run's own IS text.
        pool = [(c, d) for c, d in dated if d and d > e['start'] and not c['storyId'].startswith(f"is_{e['run']}_")]
        idx = bm25(f"{e['name']} {e['terms']} {e['summary']}", [c['text'] for c, _ in pool], TOP)
        n_new = 0
        for i in idx:
            c, d = pool[i]
            if (e['endingId'], c['chunkId']) in done:
                continue
            out = json.loads(post(JUDGE_SYS, f"ENDING ({e['runName']}): {e['name']}\n{e['summary']}\n\nLATER PASSAGE "
                                             f"(released {d}):\n{c['text'][:6000]}", 200, GRAMMAR))
            quote = (out.get('quote') or '').strip()
            found = bool(norm(quote)) and norm(quote) in norm(c['text'])
            common.append_jsonl(path, {'endingId': e['endingId'], 'chunkId': c['chunkId'], 'storyId': c['storyId'],
                                       'groupId': c['groupId'], 'released': d, 'refers': bool(out.get('refers')),
                                       'quote': quote, 'quoteFound': found})
            n_new += 1
        print(f"{e['endingId']}: {len(pool)} later passages, judged {n_new} of the top {len(idx)}", flush=True)


PICK_SYS = ("You read the endings of one run of Integrated Strategies (a roguelike mode of Arknights), labelled with "
            "letters, and a passage from a story released later. Answer which ending the passage refers to: a letter "
            "only if the passage mentions or clearly assumes events that happen in that ending and not in the others; "
            "'general' if it refers to the run or its setting without pointing to one ending; 'none' if it does not "
            "refer to the run at all. quote: the exact sentence from the passage that shows it, or an empty string.")


PICK_STRICT_SYS = ("You read the endings of one run of Integrated Strategies (a roguelike mode of Arknights), labelled with "
                   "letters, and a passage from a story released later. Decide whether the passage treats one ending as "
                   "having happened. Answer a letter only if the passage states or clearly assumes an outcome unique to that "
                   "ending: something the ending changes (who lives, dies, leaves, takes a role, is transformed, or what "
                   "becomes of a place) that the other endings do not share. A character, place, object or image that "
                   "appears in the ending is not enough, and neither is a flashback to events before the run. Answer "
                   "'general' if the passage refers to the run or its setting without such an outcome, and 'none' if it "
                   "does not refer to the run. quote: the exact sentence from the passage that states the outcome, or an "
                   "empty string.")


def pick_grammar(letters, general=True):
    """An ending letter, 'general' (passage picks only) or 'none', and a quote."""
    return ('root ::= "{\\"pick\\": \\"" ( ' + ' | '.join(f'"{l}"' for l in letters) +
            (' | "general"' if general else '') + ' | "none" ) "\\", \\"quote\\": " qstr "}"\nqstr ::= "\\"" [^"\\n]{0,300} "\\""\n')


def parse_pick(raw):
    """The pick answer, or None when it was cut off. The grammar lets a quote end in a backslash, which escapes the
    closing '"' for json (1 of the first 93 wide picks, 2026-09-30), so a json failure is read by the grammar's shape.
    Answers that parse as json are unchanged."""
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        m = re.match(r'^\{"pick": "([^"]*)", "quote": "(.*)"\}$', raw, re.S)
        return {'pick': m.group(1), 'quote': m.group(2).replace('\\', '')} if m else None


def discriminate(cands, path, only=None, system=None):
    """Contrastive pick for each (run, chunk) in cands, appended to path. only: a set of (run, chunkId) to restrict to."""
    endings = summarized_endings()
    chunks = p4_by_id()
    done = {(r['run'], r['chunkId']) for r in read_jsonl(path)}
    limit = int(os.environ.get('CANON_LIMIT', '0')); n = 0; t0 = time.time()
    for run in sorted({e['run'] for e in endings}):
        es = [e for e in endings if e['run'] == run]
        letters = 'ABCDEFG'[:len(es)]
        grammar = pick_grammar(letters)
        head = f"RUN: {es[0]['runName']}\n" + '\n'.join(f"{l}. {e['name']}: {e['summary']}" for l, e in zip(letters, es))
        prefix = es[0]['endingId'].split('_ending')[0]
        for cid, rel in sorted(cands[prefix].items()):
            if (run, cid) in done or cid not in chunks or (only is not None and (run, cid) not in only):
                continue
            if limit and n >= limit:
                break
            user = (f"{head}\n\nLATER PASSAGE (released {rel}):\n{chunks[cid]['text'][:6000]}" if rel else
                    f"{head}\n\nPASSAGE (undated: an operator file, voice line, outfit or operator record):\n{chunks[cid]['text'][:6000]}")
            out = parse_pick(post(system or PICK_SYS, user, 200, grammar))
            if out is None:  # cut off at 200 tokens: ask again with room
                out = parse_pick(post(system or PICK_SYS, user, 600, grammar)) or {'pick': None, 'quote': ''}
            pick = out.get('pick'); quote = (out.get('quote') or '').strip()
            e = es[letters.index(pick)] if pick in letters else None
            common.append_jsonl(path, {'run': run, 'chunkId': cid, 'storyId': chunks[cid]['storyId'], 'groupId': chunks[cid]['groupId'],
                                       'released': rel, 'pick': pick, 'endingId': e['endingId'] if e else None,
                                       'quote': quote, 'quoteFound': bool(norm(quote)) and norm(quote) in norm(chunks[cid]['text'])})
            n += 1
        P = [r for r in read_jsonl(path) if r['run'] == run]
        print(f"{run}: {len(P)} passages; picks {dict(collections.Counter(r['pick'] for r in P))}", flush=True)
    if n:
        print(f"{n} new picks in {time.time() - t0:.1f} s ({(time.time() - t0) / n:.2f} s each)", flush=True)


def old_candidates():
    cands = collections.defaultdict(dict)
    for r in read_jsonl(os.path.join(OUT, 'refs.jsonl')):
        cands[r['endingId'].split('_ending')[0]][r['chunkId']] = r['released']
    return cands


def wide_candidates():
    """The wide stage's candidates that the 2026-09-29 candidates (refs.jsonl) do not already hold."""
    old = old_candidates(); cands = collections.defaultdict(dict)
    for r in read_jsonl(os.path.join(OUT, 'wide.jsonl')):
        prefix = r['endingId'].split('_ending')[0]
        if r['chunkId'] not in old[prefix]:
            cands[prefix][r['chunkId']] = r['released']
    return cands


def undated_candidates():
    cands = collections.defaultdict(dict)
    for r in read_jsonl(os.path.join(OUT, 'undated.jsonl')):
        cands[r['endingId'].split('_ending')[0]][r['chunkId']] = None
    return cands


def stage_discriminate():
    discriminate(old_candidates(), os.path.join(OUT, 'picks.jsonl'))
    if WIDE:
        discriminate(wide_candidates(), os.path.join(OUT, 'picks_wide.jsonl'))
    if UNDATED:
        discriminate(undated_candidates(), os.path.join(OUT, 'picks_undated.jsonl'))


def undated(c):
    return c['groupId'] in UNDATED_GROUPS or c['groupId'].startswith('story_')


def stage_undated():
    """The wide stage's two searches per ending, kept to undated sources, top UNDATED_TOP -> undated.jsonl."""
    rows = [r for r in read_jsonl(os.path.join(OUT, 'wide.jsonl'))]
    terms = {r['endingId']: r['terms'] for r in rows}
    endings = summarized_endings()
    by_id = p4_by_id()
    out = []
    for e in endings:
        queries = {'summary': e['summary'], 'names': ' '.join([e['name']] + terms.get(e['endingId'], []))}
        fused = collections.Counter(); via = collections.defaultdict(list)
        for q, text in queries.items():
            rank = 0
            for cid in search(text):
                c = by_id.get(cid)
                if not c or not undated(c):
                    continue
                rank += 1
                fused[cid] += 1.0 / (20 + rank); via[cid].append(f"{q}:{rank}")
        for i, (cid, _) in enumerate(fused.most_common(UNDATED_TOP), 1):
            c = by_id[cid]
            out.append({'endingId': e['endingId'], 'run': e['run'], 'chunkId': cid, 'storyId': c['storyId'], 'groupId': c['groupId'],
                        'released': None, 'rank': i, 'via': via[cid]})
    common.write_jsonl(os.path.join(OUT, 'undated.jsonl'), out)
    u = undated_candidates()
    print(f"undated: {len(out)} rows, {sum(map(len, u.values()))} distinct (run, chunk); "
          f"{dict(collections.Counter(r['groupId'] if not r['groupId'].startswith('story_') else 'story_*' for r in out))}")


def gemma_picks():
    return read_jsonl(os.path.join(OUT, 'picks.jsonl')) + (read_jsonl(os.path.join(OUT, 'picks_wide.jsonl')) if WIDE else [])


def stage_second():
    """The same pick by a second model, only where Gemma named a specific ending."""
    only = {(r['run'], r['chunkId']) for r in gemma_picks() if r['endingId']}
    cands = collections.defaultdict(dict)
    only |= {(r['run'], r['chunkId']) for r in read_jsonl(os.path.join(OUT, 'picks_undated.jsonl')) if r['endingId']} if UNDATED else set()
    for src in (old_candidates(), wide_candidates() if WIDE else {}, undated_candidates() if UNDATED else {}):
        for prefix, d in src.items():
            for k, v in d.items():
                cands[prefix].setdefault(k, v)
    discriminate(cands, os.path.join(OUT, SECOND), only, PICK_STRICT_SYS if STRICT else PICK_SYS)


NAME_STOP = set("""this that there then they them their these those what when where which while with would will your
you yours have here hers him his how into just like maybe more most much must never none nothing now only other our
ours over perhaps please should since some still such than thank thanks the though through thus unless until very well
were whatever whoever why yes yeah also after again all and any are because before being both but can could did does
doing done each even ever every for from had has her how its let many may might mine not off once one onto own same
she shall too upon was who whom whose yet about above across against along among around away back behind below beside
between beyond down during except inside near outside past round toward under within without doctor fortunately
unfortunately therefore hopefully pfft c'mon how's lil""".split())


def name_key(w):
    # 'Mizuki's' and 'Mizuki' are one name; a stutter ('Th-thanks') is not a name.
    w = re.sub(r"['’]s$", '', w.lower().strip("'’-"))
    return '' if re.match(r'^([a-z]{1,2})-\1', w) else w


def distinctive_terms(e, siblings, df, caps, n=10):
    """Proper names the ending's script uses: capitalized in the corpus at least 80% of the time, found in 1 to 400
    non-IS chunks, ranked by count in the script times idf; a name every sibling ending also uses ranks lower."""
    txt = f"{e['name']}\n{e.get('desc') or ''}\n{e['script']}"
    cnt = collections.Counter(name_key(w) for w in re.findall(r"\b[A-Z][A-Za-z'’\-]{3,}", txt))
    N = max(df.values(), default=1) * 10
    sc = []
    for w, k in cnt.items():
        up, total = caps.get(w, (0, 0))
        if not w or w in NAME_STOP or not 1 <= df.get(w, 0) <= 400 or total == 0 or up / total < 0.8:
            continue
        shared = all(w in s for s in siblings) if siblings else False
        sc.append((k * math.log(N / df[w]) * (0.5 if shared else 1.0), w))
    return [w for _, w in sorted(sc, reverse=True)[:n]]


def search(query, k=1500):
    import subprocess
    out = subprocess.run([os.path.join(ROOT, 'target', 'release', 'search'), query, '--corpus', os.path.join(ROOT, 'artifacts', 'p4'),
                          '--mode', 'hybrid', '--k', str(k), '--candidates', str(k), '--json'],
                         cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return [h['chunkId'] for h in json.loads(out)['hits']]


def stage_wide():
    """Candidates by hybrid retrieval per ending: RRF of a summary query and a names query, kept to stories released
    after the run began and outside IS, top WIDE_TOP per ending -> wide.jsonl (rewritten)."""
    endings = summarized_endings()
    chunks = p4_chunks()
    by_id = {c['chunkId']: c for c in chunks}
    date = release_dates()
    df = collections.Counter(); caps = collections.defaultdict(lambda: [0, 0])
    for c in chunks:
        if c['groupId'] == 'is':
            continue
        ws = re.findall(r"\b[A-Za-z][A-Za-z'’\-]{3,}", c['text'])
        for w in set(name_key(x) for x in ws):
            df[w] += 1
        for w in ws:
            x = caps[name_key(w)]; x[1] += 1; x[0] += w[0].isupper()
    caps = {w: tuple(v) for w, v in caps.items()}
    rows = []
    for e in endings:
        sib = [set(name_key(w) for w in re.findall(r"\b[A-Z][A-Za-z'’\-]{3,}", o['script']))
               for o in endings if o['run'] == e['run'] and o['endingId'] != e['endingId'] and o['script']]
        terms = distinctive_terms(e, sib, df, caps)
        queries = {'summary': e['summary'], 'names': ' '.join([e['name']] + terms)}
        fused = collections.Counter(); via = collections.defaultdict(list)
        for q, text in queries.items():
            rank = 0
            for cid in search(text):
                c = by_id.get(cid)
                d = date(c) if c else None
                if not c or c['groupId'] == 'is' or not d or d <= e['start']:
                    continue
                rank += 1
                fused[cid] += 1.0 / (20 + rank); via[cid].append(f"{q}:{rank}")
        top = [cid for cid, _ in fused.most_common(WIDE_TOP)]
        for i, cid in enumerate(top, 1):
            c = by_id[cid]
            rows.append({'endingId': e['endingId'], 'run': e['run'], 'chunkId': cid, 'storyId': c['storyId'], 'groupId': c['groupId'],
                         'released': date(c), 'rank': i, 'via': via[cid], 'terms': terms})
        print(f"{e['endingId']} ({e['name']}): names query {queries['names']!r}", flush=True)
    common.write_jsonl(os.path.join(OUT, 'wide.jsonl'), rows)
    old = old_candidates(); new = wide_candidates()
    for run in sorted({e['run'] for e in endings}):
        prefix = [e for e in endings if e['run'] == run][0]['endingId'].split('_ending')[0]
        print(f"{run}: old {len(old[prefix])}, wide adds {len(new[prefix])}, total {len(old[prefix]) + len(new[prefix])}")
    print(f"total: old {sum(map(len, old.values()))}, wide adds {sum(map(len, new.values()))}")


STORY_SYS = ("You read the endings of one run of Integrated Strategies (a roguelike mode of Arknights), labelled with "
             "letters, and passages from one story released later. Decide which ending's OUTCOME this story treats as "
             "having happened: what the ending changes (who lives, dies, leaves, takes a role, is transformed, or what "
             "becomes of a place) that the other endings do not share. Not which ending it resembles, and not which ending "
             "shares its characters, places or images; a flashback to events before the run is not an outcome. Answer a "
             "letter only when the story states or clearly assumes that outcome, else 'none'. quote: the exact sentence "
             "from the passages that shows the outcome, or an empty string.")
STORY_MAX = 8


def story_units():
    """(run, storyId) pairs: every later story with a Gemma ending pick (old or wide, any quote status, outside IS), and
    every story with 3 or more wide candidates for the run. Passages: the picked ones first, then the story's
    best-ranked wide candidates, up to STORY_MAX, shown in story order."""
    endings = summarized_endings()
    prefix_run = {e['endingId'].split('_ending')[0]: e['run'] for e in endings}
    rank = collections.defaultdict(dict)  # (run, storyId) -> chunkId -> best wide rank
    for r in read_jsonl(os.path.join(OUT, 'wide.jsonl')):
        d = rank[(r['run'], r['storyId'])]
        d[r['chunkId']] = min(d.get(r['chunkId'], 10 ** 9), r['rank'])
    picked = collections.defaultdict(set); rel = {}
    for r in read_jsonl(os.path.join(OUT, 'picks.jsonl')) + read_jsonl(os.path.join(OUT, 'picks_wide.jsonl')):
        if r['endingId'] and r['groupId'] != 'is':
            picked[(r['run'], r['storyId'])].add(r['chunkId']); rel[(r['run'], r['storyId'])] = r['released']
    for r in read_jsonl(os.path.join(OUT, 'wide.jsonl')):
        rel.setdefault((r['run'], r['storyId']), r['released'])
    units = {}
    for key in sorted(set(picked) | {k for k, v in rank.items() if len(v) >= 3}):
        chosen = sorted(picked[key])[:STORY_MAX]
        for cid, _ in sorted(rank[key].items(), key=lambda x: (x[1], x[0])):
            if len(chosen) >= STORY_MAX:
                break
            if cid not in chosen:
                chosen.append(cid)
        units[key] = {'chunks': chosen, 'released': rel[key], 'picked': sorted(picked[key])}
    return endings, units


def stage_story():
    """Story-level pick by the model on $SERVER; argv[2] names it (gemma|qwen) -> story_<name>.jsonl (resumable)."""
    tag = sys.argv[2]
    path = os.path.join(OUT, f'story_{tag}.jsonl')
    endings, units = story_units()
    chunks = p4_by_id()
    done = {(r['run'], r['storyId']) for r in read_jsonl(path)}
    limit = int(os.environ.get('CANON_LIMIT', '0')); n = 0; t0 = time.time()
    for (run, sid), u in units.items():
        if (run, sid) in done:
            continue
        if limit and n >= limit:
            break
        es = [e for e in endings if e['run'] == run]
        letters = 'ABCDEFG'[:len(es)]
        grammar = pick_grammar(letters, general=False)
        head = f"RUN: {es[0]['runName']}\n" + '\n'.join(f"{l}. {e['name']}: {e['summary']}" for l, e in zip(letters, es))
        cs = sorted((chunks[c] for c in u['chunks'] if c in chunks), key=lambda c: (c['ordinal'], c['chunkId']))
        body = '\n\n'.join(f"[passage {i}]\n{c['text'][:3000]}" for i, c in enumerate(cs, 1))
        user = f"{head}\n\nLATER STORY {sid} (released {u['released']}), {len(cs)} passages:\n{body}"
        out = parse_pick(post(STORY_SYS, user, 200, grammar))
        if out is None:
            out = parse_pick(post(STORY_SYS, user, 600, grammar)) or {'pick': None, 'quote': ''}
        pick = out.get('pick'); quote = (out.get('quote') or '').strip()
        e = es[letters.index(pick)] if pick in letters else None
        where = [c['chunkId'] for c in cs if norm(quote) and norm(quote) in norm(c['text'][:3000])]
        common.append_jsonl(path, {'run': run, 'storyId': sid, 'groupId': cs[0]['groupId'] if cs else None, 'released': u['released'],
                                   'chunks': [c['chunkId'] for c in cs], 'picked': u['picked'], 'pick': pick,
                                   'endingId': e['endingId'] if e else None, 'quote': quote, 'quoteFound': bool(where),
                                   'quoteChunk': where[0] if where else None})
        n += 1
    R = read_jsonl(path)
    print(f"{tag}: {len(R)} of {len(units)} stories; picks {dict(collections.Counter(r['pick'] for r in R))}", flush=True)
    if n:
        print(f"{n} new in {time.time() - t0:.1f} s ({(time.time() - t0) / n:.2f} s each)", flush=True)


def is_number(run):
    """The game's IS number: rogue_1 (Phantom) is IS2, because IS1 (Ceobe's Fungimist) is not in the game data."""
    return f"IS{int(run.split('_')[1]) + 1}"


EVIDENCE_SOURCES = [('picks.jsonl', 'gemma', 'passage'), ('picks_wide.jsonl', 'gemma', 'passage'),
                    ('picks_qwen.jsonl', 'qwen', 'passage'), ('picks_qwen_strict.jsonl', 'qwen-strict', 'passage'),
                    ('story_gemma.jsonl', 'gemma', 'story'), ('story_qwen.jsonl', 'qwen', 'story')]


_RECORD_NAMES = {}


def record_name(sid):
    """'story_12fce_set_1_story_1' -> 'Operator record: <operator name>'."""
    if not _RECORD_NAMES:
        chars = kv(json.load(open(os.path.join(GD, 'excel', 'character_table.json')))['Characters'])
        _RECORD_NAMES.update({k.split('_', 2)[-1]: v.get('Name') for k, v in chars.items()})
    key = sid.split('_')[1]
    return f"Operator record: {_RECORD_NAMES.get(key, key)}"


def evidence_items():
    """Every (run, story, ending) any model picked with a quote found in the text and at least 4 words, outside IS, from
    every stage and model; one item per distinct quote, with the models and stages that gave it and the hand grade from
    data/canon_grades.json where one exists."""
    chunks = p4_by_id()
    names = timeline_names()
    gp = os.path.join(ROOT, 'data', 'canon_grades.json')
    grades = {(g['storyId'], g['endingId']): g for g in json.load(open(gp))['grades']} if os.path.exists(gp) else {}
    items = {}
    sources = EVIDENCE_SOURCES + ([('picks_undated.jsonl', 'gemma', 'passage')] if UNDATED else [])
    und = {(r['run'], r['chunkId']) for r in read_jsonl(os.path.join(OUT, 'undated.jsonl'))} if UNDATED else set()
    for fname, model, stage in sources:
        for r in read_jsonl(os.path.join(OUT, fname)):
            if not r.get('endingId') or not r.get('quoteFound') or len(r['quote'].split()) < 4 or r.get('groupId') == 'is':
                continue
            if r.get('released') is None and (r['run'], r.get('chunkId')) not in und:
                continue  # an undated pick from a file this run leaves out
            cid = r.get('quoteChunk') or r.get('chunkId')
            key = (r['run'], r['storyId'], r['endingId'], norm(r['quote']))
            # A module or other unit without an event name is named by its own title line ("Module story: 'Weighing
            # Anchor' (Highmore's module, REA-X)").
            story = (names.get(r['groupId']) or (record_name(r['storyId']) if r['groupId'].startswith('story_') else None)
                     or chunks[cid]['text'].split('\n', 1)[0][:120])
            it = items.setdefault(key, {'run': r['run'], 'storyId': r['storyId'], 'story': story,
                                        'groupId': r['groupId'], 'released': r['released'], 'endingId': r['endingId'],
                                        'quote': r['quote'], 'chunkId': cid, 'models': [], 'stages': []})
            if model not in it['models']:
                it['models'].append(model)
            if stage not in it['stages']:
                it['stages'].append(stage)
    out = []
    for it in sorted(items.values(), key=lambda x: (x['run'], x['released'] or '9999', x['storyId'], x['endingId'])):
        t = chunks[it['chunkId']]['text']; i = norm(t).find(norm(it['quote']))
        # a few lines around the quote, by character position of the normalized match (close enough for context)
        j = t.lower().find(it['quote'][:40].lower())
        j = j if j >= 0 else max(0, int(i * len(t) / max(len(norm(t)), 1)))
        it['context'] = t[max(0, j - 500): j + len(it['quote']) + 500]
        g = grades.get((it['storyId'], it['endingId']))
        it['grade'] = g['grade'] if g else None
        it['gradeNote'] = g.get('note') if g else None
        it['undated'] = it['released'] is None
        out.append(it)
    return out


VERDICT_SYS = ("You weigh evidence about which ending of one run of Integrated Strategies (a roguelike mode of Arknights) the "
               "later stories treat as canon. The game never says so; players infer it from later stories. Each evidence "
               "item is a passage from a story released after the run began that a model linked to one ending, and many "
               "of those links are wrong. Count an item only when its passage states or clearly assumes an OUTCOME unique "
               "to that ending: who lives, dies, leaves, takes a role or is transformed, or what becomes of a place, that "
               "the other endings do not share. Discount items that only share characters, places, images or words with "
               "an ending, and flashbacks to events before the run. Independent stories agreeing count for more than many "
               "passages of one story. Then give the ending the stories treat as canon; 'none' if no item holds up; "
               "'undetermined' if the items that hold up point to different endings about equally. confidence: strong "
               "(an unmistakable outcome, or one repeated across stories), moderate, weak, or none. cites: the story names "
               "whose items you relied on. reasoning: 2 to 5 sentences naming the outcome each cited story shows and why "
               "the rest were discounted.")
VERDICT_CTX = 250
# Prompt v2 (2026-09-30): the answer first judges every evidence passage (holds or not, why), then decides from the
# passages that hold. v1 answered in one step: every verdict was 'strong', and IS3's cited Lone Trail, Chapter 13 and
# Path of Life for star and sea imagery while discounting Highmore's module. CANON_VERDICT_PROMPT=v1 asks v1.
VERDICT_PROMPT = os.environ.get('CANON_VERDICT_PROMPT', 'v3')
# Prompt v3 (2026-09-30): v2 with the run's named characters. v2's IS3 verdict was Stella Caerula, strong, from five
# passages of star and sea imagery in which no character of the run appears, over Highmore's operator file (Mizuki asks
# Rhodes Island to take Highmore in); Ian's canon for IS3 is Precious Days. v3 lists each ending's named characters and
# labels each passage with the ones it names, and says: an ending is shown by the state its characters are left in; a
# passage naming none of them is atmosphere and counts as weak at most; weigh passages, do not count them.
# CANON_VERDICT_PROMPT=v2 asks v2.
VERDICT_V3 = (" An ending is shown by the state its named characters are left in (alive or dead, where they live, in what "
              "form, what role they hold). Each passage is labelled with the run's named characters it mentions. A passage "
              "that mentions none of them (atmosphere, cosmic or sea imagery, other characters' visions, legends or "
              "weapons) can be weak support at most and can never decide the verdict. One passage that shows a named "
              "character in a state only one ending leaves them in outweighs any number of atmospheric passages: weigh "
              "the passages, do not count them.")
VERDICT_V2 = (" First judge each evidence passage on its own: holds is true only if the later story treats an outcome of "
              "that ending as already true because of the run (a character's new state, a group that has left, a role "
              "taken, a death). Things that happen within the later story itself (a weapon firing, a vision, a battle, a "
              "legend, a character's backstory) are not outcomes of the run, even when they look like an ending. Then "
              "decide from the passages that hold only. confidence: strong only if passages from two or more different "
              "stories hold for the same ending; moderate if one story states the outcome plainly; weak if it is only "
              "implied; none if nothing holds.")


_CHARS = {}


def ending_characters():
    """Named characters per ending: the speakers of its script, plus its distinctive names (wide.jsonl terms) that
    speak somewhere in the corpus and appear in at most 150 non-IS chunks (drops species and places: Seaborn, Sui)."""
    if _CHARS:
        return _CHARS
    chunks = p4_chunks()
    spk = collections.Counter(name_key(x) for c in chunks for x in (c.get('speakers') or []))
    df = collections.Counter(w for c in chunks if c['groupId'] != 'is'
                             for w in {name_key(x) for x in re.findall(r"\b[A-Za-z][A-Za-z'’\-]{2,}", c['text'])})
    terms = {r['endingId']: r['terms'] for r in read_jsonl(os.path.join(OUT, 'wide.jsonl'))}
    for e in read_jsonl(os.path.join(OUT, 'endings.jsonl')):
        names = [m.group(1).strip().strip("'") for m in re.finditer(r'^([^:\n]{2,40}): ', e['script'], re.M)]
        names = [n for n in dict.fromkeys(names) if n != '???']
        have = {name_key(n) for n in names}
        names += [t.capitalize() for t in terms.get(e['endingId'], []) if spk[t] >= 3 and df[t] <= 150 and t not in have]
        _CHARS[e['endingId']] = names
    return _CHARS


def verdict_prompt(run, endings, items):
    es = [e for e in endings if e['run'] == run]
    chars = ending_characters() if VERDICT_PROMPT == 'v3' else {}
    head = (f"RUN: {is_number(run)}, {es[0]['runName']} (began {es[0]['start']})\nENDINGS:\n" +
            '\n'.join(f"{e['endingId']} ({e['name']}): {e['summary']}" +
                       (f" Named characters: {', '.join(chars.get(e['endingId']) or ['none named'])}." if chars else '')
                       for e in es))
    run_names = list(dict.fromkeys(n for e in es for n in chars.get(e['endingId'], [])))
    name = {e['endingId']: e['name'] for e in es}
    by_chunk = collections.OrderedDict()
    for it in items:
        if it['run'] == run:
            by_chunk.setdefault(it['chunkId'], []).append(it)
    chunks = {c['chunkId']: c['text'] for c in p4_chunks()} if by_chunk else {}
    lines = []
    for i, (cid, its) in enumerate(by_chunk.items(), 1):
        t = chunks[cid]; spans = []
        for it in its:
            j = t.lower().find(it['quote'][:40].lower()); j = max(j, 0)
            spans.append((max(0, j - VERDICT_CTX), min(len(t), j + len(it['quote']) + VERDICT_CTX)))
        spans.sort(); merged = [list(spans[0])]
        for a, b in spans[1:]:
            if a <= merged[-1][1]:
                merged[-1][1] = max(merged[-1][1], b)
            else:
                merged.append([a, b])
        links = '; '.join(f"{it['endingId']} ({name[it['endingId']]}): \"{it['quote']}\"" for it in its)
        when = f"released {its[0]['released']}" if its[0]['released'] else 'undated'
        shown = ' ... '.join(t[a:b] for a, b in merged)
        label = ''
        if chars:
            named = [n for n in run_names if re.search(r'\b' + re.escape(n) + r'\b', shown)]
            label = f"Mentions the run's characters: {', '.join(named)}\n" if named else "Mentions none of the run's named characters\n"
        lines.append(f"[{i}] {its[0]['story']} ({its[0]['storyId']}, {when})\n"
                     f"Linked to: {links}\n{label}Passage: " + shown)
    body = '\n\n'.join(lines) if lines else '(no evidence items)'
    return head + f"\n\nEVIDENCE ({len(lines)} passages):\n" + body, [e['endingId'] for e in es]


def cite_names(cites, run_items):
    """The verdict's cites as story names: models cite by name, by 'name (storyId, ...)' or by passage number [n] of
    the prompt (passages are numbered by distinct chunk in item order, as verdict_prompt shows them)."""
    order = list(collections.OrderedDict((it['chunkId'], it) for it in run_items).values())
    out = []
    for c in cites:
        c = str(c).strip(); hit = None
        if c.strip('[]').isdigit() and 1 <= int(c.strip('[]')) <= len(order):
            hit = order[int(c.strip('[]')) - 1]['story']
        else:
            for it in run_items:
                base = c.split(' (')[0].strip("'\" ").lower()
                if it['storyId'] in c or (base and base in it['story'].lower()):
                    hit = it['story']; break
        hit = hit or c
        if hit not in out:
            out.append(hit)
    return out


def verdict_grammar(ids):
    if VERDICT_PROMPT in ('v2', 'v3'):
        return ('root ::= "{\\"assessments\\": [" item ( ", " item ){0,39} "], \\"reasoning\\": " rstr ", \\"endingId\\": \\"" ( ' +
                ' | '.join(f'"{i}"' for i in ids) + ' | "none" | "undetermined" ) "\\", \\"confidence\\": \\"" ( "strong" | '
                '"moderate" | "weak" | "none" ) "\\", \\"cites\\": [" ( qstr ( ", " qstr ){0,7} )? "]}"\n'
                'item ::= "{\\"n\\": " [0-9]{1,2} ", \\"holds\\": " ( "true" | "false" ) ", \\"why\\": " wstr "}"\n'
                'wstr ::= "\\"" [^"\\n]{1,200} "\\""\n'
                'rstr ::= "\\"" [^"\\n]{0,1500} "\\""\nqstr ::= "\\"" [^"\\n]{1,80} "\\""\n')
    return ('root ::= "{\\"reasoning\\": " rstr ", \\"endingId\\": \\"" ( ' + ' | '.join(f'"{i}"' for i in ids) +
            ' | "none" | "undetermined" ) "\\", \\"confidence\\": \\"" ( "strong" | "moderate" | "weak" | "none" ) '
            '"\\", \\"cites\\": [" ( qstr ( ", " qstr ){0,7} )? "]}"\n'
            'rstr ::= "\\"" [^"\\n]{0,1500} "\\""\nqstr ::= "\\"" [^"\\n]{1,80} "\\""\n')


def stage_verdict():
    """Per run, one verdict from all evidence items; argv[2] tags the model and reasoning mode (e.g. gemma-on). A tag
    ending in '-on' means the server thinks first, so the answer is read as JSON without a grammar. Appends every
    attempt to verdicts.jsonl with the prompt's hash."""
    tag = sys.argv[2]; thinking = tag.endswith('-on')
    endings = summarized_endings()
    items = evidence_items()
    path = os.path.join(OUT, 'verdicts.jsonl')
    only = os.environ.get('CANON_VERDICT_RUNS')  # e.g. rogue_5: rerun one run (IS6 v3 overflowed a 16,384 context)
    for run in sorted({e['run'] for e in endings}):
        if only and run not in only.split(','):
            continue
        user, ids = verdict_prompt(run, endings, items)
        t0 = time.time()
        system = VERDICT_SYS + (VERDICT_V2 if VERDICT_PROMPT in ('v2', 'v3') else '') + (VERDICT_V3 if VERDICT_PROMPT == 'v3' else '')
        h = hashlib.sha256((system + user).encode()).hexdigest()[:12]
        shape = ('{"assessments": [{"n": 1, "holds": true, "why": ...}, ...], "reasoning": ..., "endingId": ..., "confidence": ..., "cites": [...]}'
                 if VERDICT_PROMPT in ('v2', 'v3') else '{"reasoning": ..., "endingId": ..., "confidence": ..., "cites": [...]}')
        body = {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user +
                (f'\n\nAnswer with one JSON object: {shape}.' if thinking else '')}],
                'temperature': 0, 'seed': 1, 'max_tokens': 7000 if thinking else (3000 if VERDICT_PROMPT in ('v2', 'v3') else 900)}
        if not thinking:
            body['grammar'] = verdict_grammar(ids)
        resp = common.chat_once(SERVER, body, 1800)  # no retry: one attempt per run, recorded with its prompt hash
        msg = resp['choices'][0]['message']; content = (msg.get('content') or '').strip()
        m = re.search(r'\{.*\}', content, re.S)
        try:
            out = json.loads(m.group(0)) if m else {}
        except json.JSONDecodeError:
            out = {}
        rec = {'run': run, 'is': is_number(run), 'tag': tag, 'prompt': VERDICT_PROMPT, 'undated': UNDATED, 'promptHash': h, 'promptChars': len(user),
               'items': sum(1 for it in items if it['run'] == run), 'endingId': out.get('endingId'),
               'confidence': out.get('confidence'), 'cites': out.get('cites') or [], 'reasoning': out.get('reasoning'),
               'assessments': out.get('assessments'),
               'thinkingChars': len(msg.get('reasoning_content') or ''), 'raw': None if out else content[:2000],
               'usage': resp.get('usage'), 'seconds': round(time.time() - t0, 1)}
        common.append_jsonl(path, rec)
        print(f"{tag} {is_number(run)}: {rec['endingId']} ({rec['confidence']}) cites {rec['cites']} in {rec['seconds']} s, "
              f"{(resp.get('usage') or {}).get('prompt_tokens')} prompt tokens", flush=True)


def agreed_picks():
    """(refs, gemma_only): Gemma's ending picks with a quote found (the first pass's refs under CANON_PASS=1), other
    runs' IS data left out; under CANON_AGREE only those the second model agrees on, the rest kept per ending."""
    picks = gemma_picks()
    refs = ([r for r in picks if r['endingId'] and r['quoteFound']] if picks and os.environ.get('CANON_PASS') != '1'
            else [r for r in read_jsonl(os.path.join(OUT, 'refs.jsonl')) if r['refers'] and r['quoteFound']])
    # Another run's IS data is not a later story: the one such pick (2026-09-29) credited IS4's Winterfall from an IS6
    # relic about Sami warriors. CANON_IS_EVIDENCE=1 keeps them.
    if os.environ.get('CANON_IS_EVIDENCE') != '1':
        refs = [r for r in refs if r['groupId'] != 'is']
    # Agreement: the second model (picks_qwen.jsonl) named the same ending with a quote found in the passage.
    # Gemma's quote must also run to 4 words: 'The' was found verbatim and shows nothing.
    qwen = {(r['run'], r['chunkId']): r for r in read_jsonl(os.path.join(OUT, SECOND))}
    terms = {r['endingId']: set(r['terms']) for r in read_jsonl(os.path.join(OUT, 'wide.jsonl'))}
    text = {c['chunkId']: c['text'] for c in p4_chunks()} if AGREE and ANCHOR else {}
    anchored = lambda r: not ANCHOR or bool(terms.get(r['endingId'], set()) &
                                            {name_key(w) for w in re.findall(r"\b[A-Za-z][A-Za-z'’\-]{2,}", text.get(r['chunkId'], ''))})
    agreed = lambda r: (lambda q: bool(q) and q['endingId'] == r['endingId'] and q['quoteFound'] and len(r['quote'].split()) >= 4
                        and anchored(r))(qwen.get((r.get('run'), r['chunkId'])))
    gemma_only = collections.defaultdict(list)
    if AGREE:
        for r in refs:
            if not agreed(r):
                gemma_only[r['endingId']].append(r)
        refs = [r for r in refs if agreed(r)]
    return refs, gemma_only


def evidence_schemes(refs, gemma_only):
    """(refs, gemma_only, schemes): under CANON_AGREE every evidence scheme's picks, and refs replaced by the scheme
    CANON_EVIDENCE names; without it, refs unchanged and no schemes."""
    schemes = {}
    if AGREE:
        # Story stage: a pick counts with a quote found in the story's passages and at least 4 words long.
        sg = {(r['run'], r['storyId']): r for r in read_jsonl(os.path.join(OUT, 'story_gemma.jsonl'))}
        sq = {(r['run'], r['storyId']): r for r in read_jsonl(os.path.join(OUT, 'story_qwen.jsonl'))}
        ok = lambda r: bool(r) and bool(r['endingId']) and r['quoteFound'] and len(r['quote'].split()) >= 4 and r['groupId'] != 'is'
        schemes = {'passage': refs,
                   'story-gemma': [r for r in sg.values() if ok(r)],
                   'story-qwen': [r for r in sq.values() if ok(r)],
                   'story-agreed': [r for k, r in sg.items() if ok(r) and ok(sq.get(k)) and sq[k]['endingId'] == r['endingId']]}
        if EVIDENCE not in schemes:
            sys.exit(f"CANON_EVIDENCE must be one of {sorted(schemes)}")
        if EVIDENCE != 'passage':
            gemma_only = collections.defaultdict(list, {k: v for k, v in gemma_only.items()})
            for r in refs:  # passage-agreed picks are then Gemma picks outside the served scheme too
                gemma_only[r['endingId']].append(r)
        refs = schemes[EVIDENCE]
    return refs, gemma_only, schemes


def ending_runs(endings, refs, gemma_only, schemes, names):
    """Per run, its endings with the later stories that refer to each (and, under CANON_AGREE, the other schemes')."""
    stories_of = lambda rs: sorted({(r['released'], names.get(r['groupId'], r['groupId'])) for r in rs})
    by = collections.defaultdict(list)
    for r in refs:
        by[r['endingId']].append(r)
    runs = collections.defaultdict(list)
    for e in endings:
        rs = sorted(by.get(e['endingId'], []), key=lambda r: r['released'])
        stories = sorted({(r['released'], names.get(r['groupId'], r['groupId'])) for r in rs})
        row = {'endingId': e['endingId'], 'name': e['name'], 'priority': e['priority'], 'summary': e.get('summary'),
               'referencedBy': [{'story': n, 'released': d} for d, n in stories],
               'quotes': [{'story': names.get(r['groupId'], r['groupId']), 'storyId': r['storyId'],
                           'released': r['released'], 'quote': r['quote']} for r in rs[:4]]}
        if AGREE:
            go = sorted({(r['released'], names.get(r['groupId'], r['groupId'])) for r in gemma_only.get(e['endingId'], [])} - set(stories))
            row['gemmaOnly'] = [{'story': n, 'released': d} for d, n in go]
            for k in [k for k in schemes if k != EVIDENCE]:
                rk = sorted((r for r in schemes[k] if r['endingId'] == e['endingId']), key=lambda r: r['released'])
                row[{'passage': 'passageAgreed', 'story-gemma': 'storyGemma', 'story-qwen': 'storyQwen',
                     'story-agreed': 'storyAgreed'}[k]] = [{'story': n, 'released': d, 'quotes': [
                         {'storyId': r['storyId'], 'quote': r['quote']} for r in rk if (r['released'], names.get(r['groupId'], r['groupId'])) == (d, n)][:2]}
                                                          for d, n in stories_of(rk)]
        runs[e['run']].append(row)
    first = {e['run']: e for e in endings}
    out = [{'run': k, 'runName': first[k]['runName'], 'start': first[k]['start'],
            'endings': sorted(v, key=lambda x: x['priority'] or 0)} for k, v in sorted(runs.items())]
    if AGREE:
        for r in out:
            r['is'] = is_number(r['run'])
    return out


def attach_verdicts(out, endings):
    """Each run's verdict (CANON_VERDICT, its confidence capped by the rule unless CANON_CONF_RULE=0, the second opinion)
    and every evidence item, into the rows of `out`."""
    ename = {e['endingId']: e['name'] for e in endings}
    items = evidence_items()
    latest = {}
    for v in read_jsonl(os.path.join(OUT, 'verdicts.jsonl')):
        # The last attempt of each tag and prompt wins, among those made with the same evidence (with or without
        # undated sources; attempts before 2026-09-30 08:30 were made without).
        if v.get('undated', False) == UNDATED:
            latest[(v['run'], f"{v['tag']}:{v.get('prompt', 'v1')}")] = v
    for r in out:
        v = latest.get((r['run'], VERDICT)); v2 = latest.get((r['run'], VERDICT_SECOND))
        if v:
            run_items = [it for it in items if it['run'] == r['run']]
            cites = cite_names(v['cites'], run_items)
            conf = v['confidence']; extra = {}
            if CONF_RULE and v['endingId'] in ename:
                rc, sids, events = rule_confidence(v, run_items, cites)
                if conf in CONF_ORDER:
                    conf = min(conf, rc, key=CONF_ORDER.index)
                extra = {'modelConfidence': v['confidence'], 'ruleConfidence': rc, 'citedStoryIds': sids, 'citedEvents': events}
            r['verdict'] = {'endingId': v['endingId'], 'name': ename.get(v['endingId'], v['endingId']),
                            'confidence': conf, **extra, 'reasoning': v['reasoning'],
                            'cites': cites,
                            'source': VERDICT,
                            'second': {'source': VERDICT_SECOND, 'endingId': v2['endingId'], 'confidence': v2['confidence'],
                                       'agrees': v2['endingId'] == v['endingId']} if v2 else None}
        r['evidence'] = [{'story': it['story'], 'storyId': it['storyId'], 'released': it['released'], 'undated': it['undated'],
                          'endingId': it['endingId'], 'ending': ename.get(it['endingId']), 'quote': it['quote'],
                          'models': it['models'], 'stages': it['stages'], 'grade': it['grade'], 'gradeNote': it['gradeNote']}
                         for it in items if it['run'] == r['run']]


def stage_build():
    endings = read_jsonl(os.path.join(OUT, 'endings.jsonl'))
    refs, gemma_only = agreed_picks()
    names = timeline_names()
    refs, gemma_only, schemes = evidence_schemes(refs, gemma_only)
    out = ending_runs(endings, refs, gemma_only, schemes, names)
    if AGREE and VERDICT != '0':
        attach_verdicts(out, endings)
    note = 'Later stories that refer to each ending, with quotes; the game itself never rules on canon.'
    if AGREE and EVIDENCE == 'passage':
        note += (' referencedBy: stories where Gemma and Qwen both picked this ending among the run\'s endings, each with a quote '
                 'found in the passage; gemmaOnly: stories only Gemma picked it for (weaker evidence).')
    doc = {'note': note, 'runs': out}
    if AGREE and VERDICT != '0':
        note += (' Per run, is: the game\'s IS number (rogue_1 is IS2; IS1, Fungimist, is not in the data). verdict: which ending '
                 'the later stories treat as canon, judged by a model from every evidence item (endingId, "none" or '
                 '"undetermined"; confidence strong, moderate, weak or none; cites: stories relied on; second: another '
                 'model\'s verdict and whether it agrees). evidence: every story passage any model linked to an ending with a '
                 'verbatim quote of 4 or more words, with the models and stages that linked it and a hand grade (right, '
                 'wrong, unclear) where one exists; most links are wrong, so read the grade and the verdict, not the count.')
    if AGREE:
        doc = {'note': note + (' evidenceScheme names what fills referencedBy (passage: both models agree on a passage; story-*: '
                               'one pick per later story from up to 8 of its passages); the other schemes are kept in their '
                               'own fields as weaker evidence.'), 'evidenceScheme': EVIDENCE, 'runs': out}
    json.dump(doc, open(os.path.join(OUT, 'is_endings.json'), 'w'), ensure_ascii=False, indent=1)
    for r in out:
        print(r['runName'], '|', '; '.join(f"{e['name']}: {len(e['referencedBy'])}" + (f" (+{len(e['gemmaOnly'])} Gemma only)" if AGREE else '')
                                         for e in r['endings']))


def fingerprint():
    """(endings key, other-inputs key, {chunkId: sha}) for the stamp; see `inputs` in the module doc."""
    h = lambda b: hashlib.sha256(b if isinstance(b, bytes) else b.encode()).hexdigest()[:16]
    rd = lambda p: open(p, 'rb').read() if os.path.exists(p) else b''
    scripts = os.path.join(GD, 'story', 'obt', 'roguelike')
    ends = [rd(os.path.join(GD, 'excel', 'roguelike_topic_table.json'))]
    for d, _, fs in sorted(os.walk(scripts)):
        ends += [f.encode() + rd(os.path.join(d, f)) for f in sorted(fs) if 'ending' in f]
    settings = json.dumps([SUM_SYS, JUDGE_SYS, GRAMMAR, PICK_SYS, PICK_STRICT_SYS, STORY_SYS, VERDICT_SYS, VERDICT_V2,
                           VERDICT_V3, TOP, WIDE, WIDE_TOP, AGREE, UNDATED, UNDATED_TOP, STRICT, ANCHOR, EVIDENCE, VERDICT,
                           VERDICT_SECOND, CONF_RULE, STORY_MAX])
    other = [rd(os.path.join(ROOT, 'artifacts', 'p4', 'spoiler.jsonl')), rd(os.path.join(ROOT, 'artifacts', 'chrono', 'main_release.json')),
             rd(os.path.join(GD, 'excel', 'uniequip_table.json')), rd(os.path.join(GD, 'excel', 'character_table.json')),
             rd(os.path.join(ROOT, 'data', 'canon_grades.json')),
             json.dumps(timeline_names(), sort_keys=True).encode()]
    chunks = {c['chunkId']: h(c['storyId'] + '\n' + c['groupId'] + '\n' + c['text'])
              for c in p4_chunks() if c['groupId'] not in ('profile', 'summary', 'topic')}
    return h(b'\0'.join(ends) + settings.encode()), h(b'\0'.join(other)), chunks


STAMP = os.path.join(OUT, 'inputs.json')
# Files keyed by (run, chunkId) and by (run, storyId) that prune edits; the derived ones (wide, undated, build) are rewritten.
PER_CHUNK = ('refs.jsonl', 'picks.jsonl', 'picks_wide.jsonl', 'picks_undated.jsonl', 'picks_qwen.jsonl', 'picks_qwen_strict.jsonl')
PER_STORY = ('story_gemma.jsonl', 'story_qwen.jsonl')


def changes():
    ends, other, chunks = fingerprint()
    old = json.load(open(STAMP)) if os.path.exists(STAMP) else None
    if old is None:
        return 'full', 'no stamp', set(), chunks
    if old['endings'] != ends:
        return 'full', 'endings, prompts or settings changed', set(), chunks
    oc = old['chunks']
    moved = {k for k, v in oc.items() if chunks.get(k) != v}
    new = set(chunks) - set(oc)
    if not moved and not new and old['other'] == other:
        return 'none', 'nothing changed', set(), chunks
    return 'incremental', f"{len(moved)} chunks changed or removed, {len(new)} new, other inputs {'changed' if old['other'] != other else 'unchanged'}", moved, chunks


def stage_inputs():
    kind, why, moved, chunks = changes()
    print(f"canon: plan {kind} ({why}); {len(chunks)} candidate chunks" + (f"; e.g. {sorted(moved)[:6]}" if moved else ''), flush=True)
    # The shell reads the first word of the last line.
    print(kind)


def stage_prune():
    kind, why, moved, _ = changes()
    assert kind == 'incremental', f'prune is for an incremental run, not {kind} ({why})'
    stories = {c.split('#')[0] for c in moved}
    for fname in PER_CHUNK + PER_STORY:
        path = os.path.join(OUT, fname)
        if not os.path.exists(path):
            continue
        rows = read_jsonl(path)
        keep = [r for r in rows if r.get('chunkId') not in moved and not (fname in PER_STORY and r.get('storyId') in stories)]
        common.write_jsonl(path, keep)
        print(f'prune {fname}: {len(rows)} -> {len(keep)}', flush=True)


def stage_stamp():
    ends, other, chunks = fingerprint()
    with open(STAMP, 'w') as f:
        json.dump({'endings': ends, 'other': other, 'chunks': chunks, 'written': time.strftime('%Y-%m-%d %H:%M')}, f)
    print(f'stamp: {len(chunks)} candidate chunks -> {STAMP}')


if __name__ == '__main__':
    {'inputs': stage_inputs, 'prune': stage_prune, 'stamp': stage_stamp, 'endings': stage_endings, 'summarize': stage_summarize, 'judge': stage_judge, 'discriminate': stage_discriminate,
     'wide': stage_wide, 'second': stage_second, 'story': stage_story, 'undated': stage_undated, 'verdict': stage_verdict, 'build': stage_build}[sys.argv[1]]()
