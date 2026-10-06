#!/usr/bin/env python3
"""In-world chronology: infer when each story and event takes place, from the script.

Release order is not in-world order (it agrees on 0.774 of reference pairs), and
explicit years appear in only 80 of 1,861 stories, so time is inferred from
cues: dates, "N years ago/later", references to named events, flashbacks, ages.

Stages (artifacts/chrono/, each resumable):
  cues      deterministic: per story, the speaker lines holding a time cue plus
            one line either side                               -> cues.jsonl
  extract   generator model: temporal facts per story, each with a verbatim
            quote, checked against the lines; unverified facts are dropped
                                                                -> facts.jsonl
Stories are limited to $CHRONO_GROUPS (comma-separated) when set.
"""
import collections, json, os, re, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common, incr
from common import GAMEDATA, ROOT, read_jsonl, sha16

OUT = os.path.join(ROOT, 'artifacts', 'chrono')
REF = os.path.join(ROOT, 'eval', 'reference')  # wiki references (reference.json, timeline.wikitext): eval only
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
SLOTS = int(os.environ.get('SLOTS', '2'))

CUE = re.compile(
    r"\b(1[01]\d\d)\b|\b(January|February|March|April|May|June|July|August|September|October|November|December)\b"
    r"|\b(\d+|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|twenty|thirty|a few|several|many|a couple of)"
    r"\s+(years?|months?|decades?|centuries)\b"
    r"|\b(ago|anniversary|back then|back when|long ago|last year|next year|this year|years old|when I was (a|young|little)"
    r"|since the|after the|before the)\b", re.I)

EXTRACT_SYS = (
    "You extract evidence about WHEN things happen in the Arknights world, from selected lines of one story script. "
    "List up to 8 facts. Each fact has a type: \"date\" (the lines state a calendar year or date), \"relative\" (a time "
    "given relative to a named event or to the present, such as two years after the Chernobog incident), \"flashback\" "
    "(the lines depict or recount an earlier time), or \"age\" (a character's age at some time). Give the quote: the exact "
    "words from the lines, copied verbatim. Give when: the time the fact points to, in a few plain words, such as "
    "\"present, 1097\" or \"past, two years before the Chernobog incident\". Give year: the Terra calendar year only if the "
    "quote states it, else null. Use only what the lines say, never outside knowledge. If the lines hold no evidence about "
    "time, return an empty list.")
GRAMMAR = r'''root ::= "{\"facts\": [" ( fact ( ", " fact ){0,7} )? "]}"
fact ::= "{\"type\": " type ", \"quote\": " q200 ", \"when\": " q120 ", \"year\": " year "}"
type ::= "\"date\"" | "\"relative\"" | "\"flashback\"" | "\"age\""
q200 ::= "\"" ch{4,200} "\""
q120 ::= "\"" ch{3,120} "\""
ch ::= [^"\\\x00-\x1F]
year ::= "null" | "1" [0-9] [0-9] [0-9]
'''


def norm(s):
    t = str.maketrans({'‘': "'", '’': "'", 'ʼ': "'", '“': '"', '”': '"',
                       '–': '-', '—': '-', '―': '-', '…': '.'})
    return ' '.join(s.translate(t).split()).lower()


def quote_ok(quote, text):
    q = norm(quote).rstrip('.,!? '); t = norm(text)
    if q and q in t:
        return True
    # A quote that runs on past the real words still counts when 30+ chars are verbatim.
    lo, hi = 0, len(q)
    while lo < hi:
        m = (lo + hi + 1) // 2
        if q[:m] in t: lo = m
        else: hi = m - 1
    return lo >= 30


def load_stories():
    stories = collections.defaultdict(list)
    for l in open(incr.chunks_path()):
        r = json.loads(l)
        stories[r['storyId']].append(r)
    for v in stories.values():
        v.sort(key=lambda c: c['ordinal'])
    return stories


def story_groups():
    """storyId -> groupId over the corpus chunks."""
    return {r['storyId']: r['groupId'] for r in (json.loads(l) for l in open(incr.chunks_path()))}


def p1a_rows():
    return (json.loads(l) for l in open(os.path.join(ROOT, 'artifacts', 'p1a', 'chunks.jsonl')))


def group_names(strip=True):
    """groupId -> its name, the first field of its first P1a prefix line (`strip=False`: as the place and exceptions
    prompts have always shown it, unstripped)."""
    names = {}
    for r in p1a_rows():
        name = r['prefix'].split('\n')[0].split(',')[0]
        names.setdefault(r['groupId'], name.strip() if strip else name)
    return names


def c_groups():
    """groupId -> its method-C P2 summary."""
    return {g['groupId']: g['summary'] for g in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'groups.jsonl')) if g['method'] == 'C'}


def selected(stories):
    g = os.environ.get('CHRONO_GROUPS')
    if not g:
        return sorted(stories)
    keep = set(g.split(','))
    return sorted(s for s, v in stories.items() if v[0]['groupId'] in keep)


def cue_lines(cs):
    """A story's speaker lines holding a time cue, plus one line either side."""
    lines = [x for c in cs for x in c['text'].split('\n')]
    keep = set()
    for i, x in enumerate(lines):
        if CUE.search(x):
            keep |= {i - 1, i, i + 1}
    return [lines[i] for i in sorted(keep) if 0 <= i < len(lines)]


def stage_cues():
    os.makedirs(OUT, exist_ok=True)
    stories = load_stories()
    common.write_jsonl(os.path.join(OUT, 'cues.jsonl'),
                       ({'storyId': s, 'groupId': stories[s][0]['groupId'], 'lines': cue_lines(stories[s])} for s in sorted(stories)))
    print('cues written for', len(stories), 'stories')


def chat(user):
    """(unstripped text, timings) of one extraction call, with retries."""
    return common.chat(SERVER, EXTRACT_SYS, user, 900, 900, lambda d: (common.content(d), d['timings']), grammar=GRAMMAR)


def extract_input(head_pre, lines):
    if not lines:
        return ''  # no cue lines: no model call, the row is empty
    syn = next((l for l in head_pre if l.startswith('Synopsis: ')), '')
    return f'{head_pre[0]}\n{syn}\n\nLines:\n' + '\n'.join(lines)


def extract_stage():
    # Cue lines are recomputed from the chunks (the same function `cues` writes), so a changed story
    # changes this stage's input even before `cues` reruns.
    stories = load_stories()
    cues = {s: {'groupId': v[0]['groupId'], 'lines': cue_lines(v)} for s, v in stories.items()}
    pre = {}
    for r in p1a_rows():
        pre.setdefault(r['storyId'], r['prefix'].split('\n'))
    st = incr.Stage('chrono extract', os.path.join(OUT, 'facts.jsonl'), lambda r: r['storyId'], sha16(EXTRACT_SYS + GRAMMAR))
    for s in sorted(stories):
        st.unit(s, extract_input(pre.get(s, ['']), cues[s]['lines']))
    return st, selected(stories), None, cues, pre


def stage_extract():
    st, order, live, cues, pre = extract_stage()
    todo = st.plan(order, live)
    if todo is None:
        return
    psha = st.psha
    print(f'extract: {len(todo)} to do, {len(order) - len(todo)} done, prompt {psha}', flush=True)
    lock = threading.Lock(); n = [0]; t0 = time.time()

    def one(s):
        lines = cues[s]['lines']
        row = {'storyId': s, 'groupId': cues[s]['groupId'], 'promptSha': psha, 'facts': [], 'rejected': 0}
        if lines:
            out, t = chat(extract_input(pre[s], lines))
            try:
                facts = json.loads(out)['facts']
            except Exception:
                facts = []; row['unparsed'] = True
            text = '\n'.join(lines)
            row['facts'] = [f for f in facts if quote_ok(f['quote'], text)]
            row['rejected'] = len(facts) - len(row['facts'])
        with lock:
            st.put(row)
            n[0] += 1
            if n[0] % 25 == 0:
                el = time.time() - t0
                print(f'{time.strftime("%H:%M:%S")} extract {n[0]}/{len(todo)}, {el / n[0]:.1f} s each', flush=True)
    common.par(todo, one, SLOTS, lock)
    print(f'extract: {n[0]} in {time.time() - t0:.0f}s', flush=True)


PLACE_SYS = (
    "You place one Arknights story event on the in-world timeline, in Terra calendar years. You get reference points: "
    "other events with the year their own scripts date them to. Then the target event: its summary and time evidence "
    "quoted from its script. Decide the year in which the target event's present takes place. Use an explicit date for "
    "the present first. Otherwise relate the target to a reference point it mentions or follows from, for example an "
    "incident it calls two years past. If nothing in the given text supports a year, answer null. Flashbacks to earlier "
    "years are not the present. Never use knowledge from outside the given text.")
PLACE_GRAMMAR = r'''root ::= "{\"year\": " year ", \"basis\": " basis ", \"reason\": " q "}"
year ::= "null" | "1" [0-9] [0-9] [0-9]
basis ::= "\"explicit\"" | "\"relative\"" | "\"none\""
q ::= "\"" ch{10,300} "\""
ch ::= [^"\\\x00-\x1F]
'''


def place_stage():
    facts = collections.defaultdict(list)
    for r in read_jsonl(os.path.join(OUT, 'facts.jsonl')):
        for f in r['facts']:
            facts[r['groupId']].append((r['storyId'], f))
    names = group_names(strip=False)
    summ = c_groups()
    pend = incr.p2_pending()[1] if incr.PLAN else set()
    dated = place_explicit()
    ref_lines = []
    for g in sorted(dated, key=dated.get):
        first = ' '.join(summ.get(g, '').split()[:30])
        ref_lines.append(f"- {names.get(g, g)}: {int(dated[g])}. {first}...")
    ref_block = '\n'.join(ref_lines)
    # Every placement reads the reference block, so a redone summary of any dated group redoes them all.
    ref_pending = any(g in pend for g in dated)
    st = incr.Stage('chrono place', os.path.join(OUT, 'placed.jsonl'), lambda r: r['groupId'], sha16(PLACE_SYS + PLACE_GRAMMAR))
    targets = [g for g in sorted(facts) if not g.startswith('story_')]
    users = {}
    for g in targets:
        ev = '\n'.join(f'- ({f["type"]}) "{f["quote"]}" -> {f["when"]}' for _, f in facts[g][:40])
        users[g] = (f'Reference points (year from each script):\n{ref_block}\n\nTarget event: {names.get(g, g)}\n'
                    f'Summary: {summ.get(g, "(none)")}\n\nTime evidence from its script:\n{ev or "(none)"}')
        st.unit(g, users[g], upstream=ref_pending or g in pend)
    # No backfill: placed.jsonl (2026-09-26 09:19) predates the facts.jsonl it reads (09-26 21:14), so
    # today's inputs are not the ones these 52 rows were built from; the stage redoes them.
    st.no_backfill = 'placed.jsonl predates facts.jsonl'
    return st, targets, None, users, len(ref_lines)


def stage_place():
    st, targets, live, users, n_ref = place_stage()
    targets = st.plan(targets, live)
    if targets is None:
        return
    psha = st.psha
    print(f'place: {len(targets)} groups, {n_ref} reference points, prompt {psha}', flush=True)
    for g in targets:
        out = common.content(common.chat_once(SERVER, common.chat_body(PLACE_SYS, users[g], 200, grammar=PLACE_GRAMMAR), 900))
        row = json.loads(out); row.update({'groupId': g, 'promptSha': psha})
        st.put(row)
        print(f"{g}: {row['year']} ({row['basis']}) {row['reason'][:120]}", flush=True)


YEAR_RE = re.compile(r"\b(?:year|in|of|since|by|until|around|circa|ca\.)\s+(\d{3,4})\b|\b(1[01]\d\d)\b"
                     r"|\b(\d{3,4})\s*(?:AD|BC|BCE|CE|A\.D\.)\b", re.I)
EVENT_SYS = (
    "You read one line from Arknights (a story script or an operator archive) that contains a number which may be a year "
    "of the Terra calendar. Decide whether the number is a calendar year. It is not a year when it is a count, an issue "
    "number, a price, a score, a model number or part of a name. If it is a year, describe in one short sentence what "
    "happened or holds in that year according to the line, naming people and places instead of pronouns; give the month "
    "or season if the line states one; and say whether the line presents that year as the scene's own present (a dated "
    "caption, log entry or letter heading) rather than a mention of the past or future. Use only the line and its context.")
EVENT_GRAMMAR = r'''root ::= "{\"is_year\": " bool ", \"year\": " year ", \"when\": " q60 ", \"event\": " q200 ", \"present\": " bool "}"
bool ::= "true" | "false"
year ::= "null" | [1-9] [0-9] [0-9] | "1" [0-9] [0-9] [0-9]
q60 ::= "\"" ch{0,60} "\""
q200 ::= "\"" ch{0,220} "\""
ch ::= [^"\\\x00-\x1F]
'''


def year_candidates():
    """Every script and archive line with a number that could be a Terra year (300 to 1110), with one line of context."""
    out = []
    for s, cs in load_stories().items():
        lines = [x for c in cs for x in c['text'].split('\n')]
        for i, x in enumerate(lines):
            ys = [int(next(g for g in m.groups() if g)) for m in YEAR_RE.finditer(x)]
            if any(300 <= y <= 1110 for y in ys):
                out.append({'source': s, 'kind': 'story', 'line': x,
                            'context': '\n'.join(lines[max(0, i - 1):i + 2])})
    gd = os.environ.get('GAMEDATA', GAMEDATA)
    hb = json.load(open(os.path.join(gd, 'excel', 'handbook_info_table.json')))['HandbookDict']
    for e in hb:
        v = e['value']
        texts = [st['StoryText'] for sta in v.get('StoryTextAudio', []) for st in sta.get('Stories', [])]
        lines = '\n'.join(texts).split('\n')
        for i, x in enumerate(lines):
            ys = [int(next(g for g in m.groups() if g)) for m in YEAR_RE.finditer(x)]
            if any(300 <= y <= 1110 for y in ys):
                out.append({'source': 'archive:' + e['key'], 'kind': 'archive', 'line': x,
                            'context': '\n'.join(lines[max(0, i - 1):i + 2])})
    return out


def event_input(c):
    return f"Line: {c['line']}\n\nContext:\n{c['context']}"


def events_stage():
    st = incr.Stage('chrono events', os.path.join(OUT, 'events.jsonl'), lambda r: (r['source'], r['line']),
                    sha16(EVENT_SYS + EVENT_GRAMMAR))
    cands = year_candidates()
    # A line repeated in one story or archive is one unit keyed on its first context (1 of 197 rows,
    # archive:char_4207_branch "Date: 1101-..." was written twice by the first run).
    for c in cands:
        st.unit((c['source'], c['line']), event_input(c))
    return st, [(c['source'], c['line']) for c in cands], None, cands


def stage_events():
    st, order, live, cands = events_stage()
    todo = st.plan(order, live)
    if todo is None:
        return
    if incr.IDS_ONLY:
        todo = [c for c in cands if (c['source'], c['line']) not in st.have]
    else:
        first = {}
        for c in cands:
            first.setdefault((c['source'], c['line']), c)
        todo = [first[k] for k in todo]
    psha = st.psha
    print(f'events: {len(todo)} candidate lines, prompt {psha}', flush=True)
    def one(c):
        row = json.loads(common.content(common.chat_once(SERVER, common.chat_body(EVENT_SYS, event_input(c), 220, grammar=EVENT_GRAMMAR), 600)))
        # The year must appear in the line itself, or it is not an explicit year.
        if row['is_year'] and (row['year'] is None or str(row['year']) not in c['line']):
            row['is_year'] = False; row['rejected'] = 'year not in line'
        row.update({'source': c['source'], 'kind': c['kind'], 'line': c['line'], 'promptSha': psha})
        st.put(row)
    common.par(todo, one, SLOTS)
    rows = read_jsonl(st.path)
    print(f"events: {sum(r['is_year'] for r in rows)} calendar years of {len(rows)} candidates", flush=True)


EXC_SYS = (
    "You decide where one Arknights story event sits in in-world time. You are told the year the ongoing storyline has "
    "reached around this event, and given the event's summary and every dated line from its script. Decide whether the "
    "event's own present, the time its main storyline happens in, is that storyline period, or clearly earlier (a "
    "historical or prequel event) or clearly later. Flashback scenes, memories, dated songs, books, inscriptions and "
    "records inside a present-day event do not make the event historical. Choose earlier or later only when the "
    "evidence shows the main storyline itself is set then, and give its year only if a dated line supports it.")
EXC_GRAMMAR = r'''root ::= "{\"setting\": " setting ", \"year\": " year ", \"reason\": " q "}"
setting ::= "\"storyline\"" | "\"earlier\"" | "\"later\""
year ::= "null" | [1-9] [0-9] [0-9] | "1" [0-9] [0-9] [0-9]
q ::= "\"" ch{10,300} "\""
ch ::= [^"\\\x00-\x1F]
'''


def release_times():
    rel = {}
    for x in (json.loads(l) for l in open(os.path.join(ROOT, 'artifacts', 'spoiler.jsonl'))):
        t = x.get('groupStartTime')
        if t not in (None, -1, '-1'):
            rel[x['groupId']] = int(t)
    return rel


def backbone():
    """Monotone year-by-release fit through groups' latest dated scene caption, dropping anchors
    more than 3 years under the trend (flashback-only captions), refitted until stable."""
    gof = story_groups()
    cap = collections.defaultdict(list)
    for e in read_jsonl(os.path.join(OUT, 'events.jsonl')):
        if e['is_year'] and e['present'] and e['kind'] == 'story':
            cap[gof[e['source']]].append(e['year'])
    rel = release_times()
    # Operator records are backstory and their release date says nothing about when they are set,
    # so only main and event groups anchor the storyline.
    pts = sorted((rel[g], max(v), g) for g, v in cap.items() if g in rel and not g.startswith('story_'))
    dropped = []
    while True:
        xs = [p for p in pts if p[2] not in dropped]
        fit = pav([y for _, y, _ in xs])
        bad = [xs[i][2] for i in range(len(xs)) if xs[i][1] < fit[i] - 3]
        # PAV pools an outlier with its neighbours, so compare against a leave-one-out fit instead.
        loo = []
        for i in range(len(xs)):
            rest = [y for j, (_, y, _) in enumerate(xs) if j != i]
            f = pav(rest)
            left = f[i - 1] if i > 0 else f[0]
            if xs[i][1] < left - 3:
                loo.append(xs[i][2])
        new = [g for g in set(bad) | set(loo) if g not in dropped]
        if not new:
            break
        dropped += new
    anchors = [(t, y, g) for t, y, g in pts if g not in dropped]
    return anchors, dropped, rel


def pav(ys):
    blocks = [[y, 1] for y in ys]
    i = 0
    while i < len(blocks) - 1:
        if blocks[i][0] / blocks[i][1] > blocks[i + 1][0] / blocks[i + 1][1]:
            blocks[i][0] += blocks[i + 1][0]; blocks[i][1] += blocks[i + 1][1]; del blocks[i + 1]
            i = max(i - 1, 0)
        else:
            i += 1
    out = []
    for s_, n in blocks:
        out += [s_ / n] * n
    return out


def storyline_year(t, anchors):
    """Year of the ongoing storyline at release time t: the latest anchor at or before t, else the first anchor."""
    fit = pav([y for _, y, _ in anchors])
    best = fit[0]
    for (at, _, _), fy in zip(anchors, fit):
        if at <= t:
            best = fy
    return best


def exceptions_stage():
    anchors, dropped, rel = backbone()
    names = group_names(strip=False)
    summ = c_groups()
    pend = incr.p2_pending()[1] if incr.PLAN else set()
    gof = story_groups()
    dated = collections.defaultdict(list)
    for e in read_jsonl(os.path.join(OUT, 'events.jsonl')):
        if e['is_year'] and e['kind'] == 'story':
            dated[gof[e['source']]].append(e)
    st = incr.Stage('chrono exceptions', os.path.join(OUT, 'exceptions.jsonl'), lambda r: r['groupId'], sha16(EXC_SYS + EXC_GRAMMAR))
    targets = [g for g in sorted(rel, key=rel.get) if g in summ]
    users, sy = {}, {}
    for g in targets:
        sy[g] = storyline_year(rel[g], anchors)
        lines = '\n'.join(f"- {e['year']}: {e['event']} (\"{e['line'][:160]}\")" for e in dated[g]) or '(no dated lines)'
        users[g] = (f'The ongoing storyline has reached about year {sy[g]:.0f} around this event.\n\nEvent: {names.get(g, g)}\n'
                    f'Summary: {summ[g]}\n\nDated lines from its script:\n{lines}')
        st.unit(g, users[g], upstream=g in pend)
    return st, targets, None, users, sy, anchors, dropped


def stage_exceptions():
    st, targets, live, users, sy, anchors, dropped = exceptions_stage()
    print('backbone anchors', [(g, y) for _, y, g in anchors], 'dropped', dropped, flush=True)
    targets = st.plan(targets, live)
    if targets is None:
        return
    psha = st.psha
    print(f'exceptions: {len(targets)} groups, prompt {psha}', flush=True)
    for g in targets:
        row = json.loads(common.content(common.chat_once(SERVER, common.chat_body(EXC_SYS, users[g], 220, grammar=EXC_GRAMMAR), 900)))
        row.update({'groupId': g, 'storylineYear': round(sy[g], 2), 'promptSha': psha})
        st.put(row)
        if row['setting'] != 'storyline':
            print(f"{g}: {row['setting']} {row['year']} (storyline {sy[g]:.0f}) {row['reason'][:120]}", flush=True)
    print('exceptions done', flush=True)


def timeline_v2():
    """Present year per group: the storyline year at its release, unless the exception pass
    moved it with a dated line; ties broken by release order."""
    anchors, _, rel = backbone()
    exc = {r['groupId']: r for r in read_jsonl(os.path.join(OUT, 'exceptions.jsonl'))}
    out = {}
    for g, t in rel.items():
        y = storyline_year(t, anchors)
        e = exc.get(g)
        if e and e['setting'] != 'storyline' and e.get('year'):
            y = float(e['year'])
        out[g] = y + (t - 1.5e9) / 1e12  # release order as a sub-year tiebreak
    return out


def reference():
    """Eval-only reference years per group: T1 = the wiki's 'event during which X takes
    place' entries, T2 = groups cited under exactly one year (never main chapters,
    whose citations are mostly flashbacks). Built from reference.json."""
    R = json.load(open(os.path.join(REF, 'reference.json')))
    by = collections.defaultdict(list)
    for r in R:
        if r['kind'] == 'g' and not r['id'].startswith('story_'):
            by[r['id']].append(r)
    ref = {}
    for g, rs in by.items():
        st = [r for r in rs if r['tier'] == 'setting']
        if st:
            ref[g] = ('T1', st[0]['year'] + (st[0]['frac'] or 0.0))
        elif len({r['year'] for r in rs}) == 1 and not g.startswith('main_'):
            ref[g] = ('T2', float(rs[0]['year']) + (rs[0]['frac'] or 0.0))
    return ref


def place_explicit():
    """Model-free baseline: an event sits at the majority year its own facts give its present."""
    votes = collections.defaultdict(collections.Counter)
    for r in read_jsonl(os.path.join(OUT, 'facts.jsonl')):
        for f in r['facts']:
            if f.get('year') and f['when'].lower().startswith('present'):
                votes[r['groupId']][f['year']] += 1
    return {g: float(c.most_common(1)[0][0]) for g, c in votes.items()}


def score(pred, ref, label):
    import itertools
    gs = [g for g in ref if g in pred]
    exact = sum(int(pred[g]) == int(ref[g][1]) for g in gs)
    pairs = agree = ties = 0
    for a, b in itertools.combinations(gs, 2):
        if int(ref[a][1]) == int(ref[b][1]):
            continue
        pairs += 1
        if pred[a] == pred[b]:
            ties += 1
        elif (ref[a][1] < ref[b][1]) == (pred[a] < pred[b]):
            agree += 1
    decided = pairs - ties
    print(f'{label}: placed {len(gs)}/{len(ref)} reference groups, exact year {exact}/{len(gs)}; '
          f'year-distinct pairs {pairs}: correct {agree}, wrong {decided - agree}, tied {ties}; '
          f'accuracy on decided {agree / max(decided, 1):.3f}, on all {agree / max(pairs, 1):.3f}')


def stage_eval():
    ref = reference()
    for tier in ('T1', 'T1+T2'):
        rr = {g: v for g, v in ref.items() if tier == 'T1+T2' or v[0] == 'T1'}
        print(f'-- {tier}: {len(rr)} groups')
        spo = [json.loads(l) for l in open(os.path.join(ROOT, 'artifacts', 'spoiler.jsonl'))]
        rel = {}
        for x in spo:
            t = x.get('groupStartTime')
            if t not in (None, -1, '-1'):
                rel[x['groupId']] = float(t)
        # Release order as a year-free ranking: map ranks to pseudo-years so ties never happen.
        order = sorted(rel, key=rel.get)
        score({g: 2000 + i for i, g in enumerate(order)}, rr, 'release order')
        score(place_explicit(), rr, 'explicit present-year majority')
        score(timeline_v2(), rr, 'timeline v2 (backbone + exceptions)')
        pl = os.path.join(OUT, 'placed.jsonl')
        if os.path.exists(pl):
            score({r['groupId']: float(r['year']) for r in read_jsonl(pl) if r.get('year')}, rr, 'model placement')


def place_main_chapters(groups, fb, dated):
    """Main chapters have no release time in the index, so the backbone never placed them. In-world order follows
    chapter order; the script states no year for Episodes 0 to 9 (0 year facts, 0 dated lines), so they are bounded by
    the nearest placed chapter, never given a year of their own. Appends to `groups`."""
    names = group_names()
    placed = {g['groupId']: g for g in groups}
    main_ids = sorted({r['groupId'] for r in (json.loads(l) for l in open(incr.chunks_path()))
                       if r['groupId'].startswith('main_')}, key=lambda g: int(g.split('_')[1]))
    known = [(int(g.split('_')[1]), placed[g]['storylineYear']) for g in main_ids if g in placed]
    for g in main_ids:
        n = int(g.split('_')[1])
        if g in placed:
            placed[g]['chapter'] = n
            continue
        later = [(k, y) for k, y in known if k > n]; earlier = [(k, y) for k, y in known if k < n]
        if later:
            k, y = min(later)
            basis = f'main-story order: before Episode {k} (about {y:.0f}); the script states no year'
            bound = 'at most'
        else:
            k, y = max(earlier)
            basis = f'main-story order: after Episode {k} (about {y:.0f}); the script states no year'
            bound = 'at least'
        groups.append({'groupId': g, 'name': names.get(g, g), 'releaseTime': None, 'chapter': n, 'storylineYear': round(y, 2),
                       'yearBound': bound, 'basis': basis, 'captionOnlyFlashbacks': False, 'candidateException': None,
                       'flashbacks': fb[g] if g in fb else None, 'datedLines': dated.get(g, [])})


def stage_build():
    """Chronology v1: every group at its backbone storyline year (release position with years attached),
    with the dated evidence behind it, plus the explicit-year Terra events. Exceptions are not applied:
    the model pass that proposes them is refuted (design/trevor-chronology.md section 5)."""
    anchors, dropped, rel = backbone()
    anchor_ids = {g for _, _, g in anchors}
    names = group_names()
    gof = story_groups()
    ev = [e for e in read_jsonl(os.path.join(OUT, 'events.jsonl')) if e['is_year']]
    dated = collections.defaultdict(list)
    for e in ev:
        if e['kind'] == 'story':
            dated[gof[e['source']]].append({'year': e['year'], 'when': e['when'], 'event': e['event'],
                                           'sceneCaption': e['present'], 'story': e['source'], 'line': e['line']})
    # Candidate exceptions from the reasoning pass (Qwen3.5 9B, thinking on), kept only when a dated line
    # of the group states the year. Annotations only: moving them costs 11 reference pairs (0.830 -> 0.811),
    # all from one flashback-only caption (act28side, 1019), so ordering stays on the backbone.
    dy = collections.defaultdict(set)
    for e in ev:
        if e['kind'] == 'story':
            dy[gof[e['source']]].add(e['year'])
    cand = {}
    # Only groups chosen from the data (disagree.json: their own dated lines disagree with the backbone by more than
    # 3 years); the reasoning pass also ran on the wiki's T1 groups, as eval controls, never served (2026-10-04:
    # the wiki is a reference). CHRONO_EXC_ALL=1 reads every row, the file before; both give the same 3 candidates.
    picked = None if os.environ.get('CHRONO_EXC_ALL') == '1' else set(json.load(open(os.path.join(OUT, 'disagree.json'))))
    for r in read_jsonl(os.path.join(OUT, 'exceptions_reasoning.jsonl')):
        if picked is not None and r['groupId'] not in picked:
            continue
        if r['setting'] != 'storyline' and r.get('year') in dy[r['groupId']]:
            cand[r['groupId']] = {'setting': r['setting'], 'year': r['year'], 'reason': r['reason']}
    fb = collections.defaultdict(lambda: {'depicts': [], 'recounts': 0})
    for r in read_jsonl(os.path.join(OUT, 'flashbacks.jsonl')):
        if r['tag'] == 'depicts':
            fb[r['groupId']]['depicts'].append({'storyId': r['storyId'], 'years': r['depictedYears'],
                                                 'evidence': r['evidence'][:1]})
        elif r['tag'] == 'recounts':
            fb[r['groupId']]['recounts'] += 1
    groups = []
    for g, t in sorted(rel.items(), key=lambda x: x[1]):
        y = storyline_year(t, anchors)
        groups.append({'groupId': g, 'name': names.get(g, g), 'releaseTime': t, 'storylineYear': round(y, 2),
                       'basis': 'dated caption (anchor)' if g in anchor_ids else 'release position',
                       'captionOnlyFlashbacks': g in dropped, 'candidateException': cand.get(g),
                       'flashbacks': fb[g] if g in fb else None,
                       'datedLines': dated.get(g, [])})
    place_main_chapters(groups, fb, dated)
    history = [{'year': e['year'], 'when': e['when'], 'event': e['event'], 'sceneCaption': e['present'],
                'source': e['source'], 'line': e['line'], 'derived': None} for e in ev]
    # Derived years: 'N years ago' counted from a dated scene, the event's dated anchor, or (lowest
    # confidence) the storyline estimate. On events matched to the wiki, 6 of 6 agree on the year.
    for d in read_jsonl(os.path.join(OUT, 'derived.jsonl')):
        history.append({'year': d['year'], 'when': d['when'], 'event': d['quote'], 'sceneCaption': False,
                        'source': d['storyId'], 'line': d['quote'],
                        'derived': {'basis': d['basis'], 'from': d['from'], 'offset': d['offset'],
                                    'approximate': d.get('approximate', False)}})
    history.sort(key=lambda x: (x['year'], x['derived'] is not None, x['source']))
    out = {'version': 'chrono-v1', 'anchors': [(g, y) for _, y, g in anchors], 'groups': groups, 'history': history}
    json.dump(out, open(os.path.join(OUT, 'timeline_v1.json'), 'w'), ensure_ascii=False, indent=1)
    print(f"timeline_v1: {len(groups)} groups ({len(anchor_ids)} anchored), {len(history)} dated history lines")


def stage_history():
    t = json.load(open(os.path.join(OUT, 'timeline_v1.json')))
    names = group_names()
    gof = story_groups()
    lines = ['# Terra history from explicitly dated lines', '',
             'Stated years come from a line of the game text that states the year (scene captions marked). Derived years '
             'come from "N years ago/later" counted from a dated scene, the event\'s dated anchor, or the storyline estimate, '
             'and say which. Built by `scripts/chrono.py`.', '']
    by_c = collections.defaultdict(list)
    for h in t['history']:
        by_c[(h['year'] // 100) * 100].append(h)
    for c in sorted(by_c):
        lines.append(f'## {c}s')
        for h in by_c[c]:
            src = h['source']
            where = ('operator archive ' + src.split(':', 1)[1]) if src.startswith('archive:') else names.get(gof.get(src, ''), src)
            tag = ' (scene dated)' if h['sceneCaption'] else ''
            d = h.get('derived')
            if d:
                conf = {'scene caption': 'from a dated scene', 'event anchor': "from the event's dated anchor",
                        'storyline estimate': 'from the storyline estimate, low confidence'}[d['basis']]
                ca = 'ca. ' if d.get('approximate') else ''
                lines.append(f"- **{ca}{h['year']}** (derived: {abs(d['offset'])} years {'after' if d['offset'] > 0 else 'before'} "
                             f"{d['from']:.0f}, {conf}): \"{h['event']}\" ({where})")
            else:
                lines.append(f"- **{h['year']}**{tag}: {h['event']} ({where})")
        lines.append('')
    open(os.path.join(OUT, 'terra_history.md'), 'w').write('\n'.join(lines))
    print('\n'.join(lines[:40]))


def stage_order(a, b):
    t = json.load(open(os.path.join(OUT, 'timeline_v1.json')))
    def find(q):
        ql = q.lower()
        hits = [g for g in t['groups'] if ql == g['groupId'].lower() or ql == g['name'].lower()]
        hits = hits or [g for g in t['groups'] if ql in g['name'].lower()]
        if not hits:
            raise SystemExit(f'no event matches {q!r}')
        return hits[0]
    x, y = find(a), find(b)
    def key(g, use_exc):
        ce = g.get('candidateException')
        yr = ce['year'] if (use_exc and ce) else g['storylineYear']
        return (yr, g.get('chapter') or 0, g['releaseTime'] or 0)
    def say(k1, k2):
        f, s_ = (x, y) if k1 < k2 else (y, x)
        kf, ks = (k1, k2) if k1 < k2 else (k2, k1)
        return f"{f['name']} (about {kf[0]:.0f}) comes before {s_['name']} (about {ks[0]:.0f})"
    exc = [g for g in (x, y) if g.get('candidateException')]
    if exc:
        by_dates = say(key(x, True), key(y, True)); by_pos = say(key(x, False), key(y, False))
        print(f'By their dated scenes: {by_dates}.')
        for g in exc:
            ce = g['candidateException']
            print(f"  {g['name']}'s own dated scenes place its main storyline {ce['setting']} than the storyline around its release: {ce['reason']}")
        if by_dates != by_pos:
            print(f'By storyline position alone the order would be: {by_pos}.')
        print('Confidence: uncertain. Chronology v1 reads these exceptions from dated scenes, and one flashback-only '
              'caption has misled it before (design/trevor-chronology.md).')
    else:
        print(say(key(x, False), key(y, False)) + '.')
        if all(g['basis'].startswith('dated') for g in (x, y)) and int(x['storylineYear']) != int(y['storylineYear']):
            print('Basis: both are dated by their own scripts.')
        else:
            print('Basis: storyline position (release order with dated anchors); likely, not certain.')
    for g in (x, y):
        f = g.get('flashbacks')
        if f and f['depicts']:
            ys = sorted({yy for d in f['depicts'] for yy in d['years']})
            print(f"  Note: {g['name']} has scenes dated well before its storyline position ({', '.join(map(str, ys))}); "
                  f"the order above is about its main storyline.")
        elif f and f['recounts']:
            print(f"  Note: {f['recounts']} of {g['name']}'s stories recount the past without dating it.")
    for g in (x, y):
        for d in g['datedLines'][:3]:
            print(f"  {g['name']}: {d['year']}{' (scene dated)' if d['sceneCaption'] else ''}: {d['line'][:110]}")


def stage_flashbacks():
    """Per story: does it depict or recount a time well before its event's storyline year?
    depicts = a scene caption dated more than 3 years before the storyline year (strong);
    recounts = only flashback facts or past-dated facts (weak). Writes flashbacks.jsonl."""
    t = json.load(open(os.path.join(OUT, 'timeline_v1.json')))
    sy = {g['groupId']: g['storylineYear'] for g in t['groups']}
    stories = load_stories()
    caps = collections.defaultdict(list); past = collections.defaultdict(list)
    for e in read_jsonl(os.path.join(OUT, 'events.jsonl')):
        if e['is_year'] and e['kind'] == 'story':
            (caps if e['present'] else past)[e['source']].append(e)
    facts = {r['storyId']: r['facts'] for r in read_jsonl(os.path.join(OUT, 'facts.jsonl'))}
    rows = []
    for s_, cs in sorted(stories.items()):
        g = cs[0]['groupId']; y = sy.get(g)
        # Main chapters have no release time in the index; their own latest caption stands in.
        if y is None:
            own = [e['year'] for x in stories if stories[x][0]['groupId'] == g for e in caps.get(x, [])]
            y = max(own) if own else None
        dep = [e for e in caps.get(s_, []) if y is not None and e['year'] < y - 3]
        fb = [f for f in facts.get(s_, []) if f['type'] == 'flashback']
        pd = [e for e in past.get(s_, []) if y is not None and e['year'] < y - 3]
        tag = 'depicts' if dep else ('recounts' if (fb or pd) else ('none' if s_ in facts else 'unextracted'))
        rows.append({'storyId': s_, 'groupId': g, 'storylineYear': y, 'tag': tag,
                     'depictedYears': sorted({e['year'] for e in dep}),
                     'evidence': [e['line'][:160] for e in dep][:3] + [f['quote'][:160] for f in fb][:3]})
    common.write_jsonl(os.path.join(OUT, 'flashbacks.jsonl'), rows)
    c = collections.Counter(r['tag'] for r in rows)
    print('flashback tags:', dict(c))
    # Eval against the wiki's story-level citations (stage codes), eval only.
    ref = [r for r in json.load(open(os.path.join(REF, 'reference.json'))) if r['kind'] == 's']
    by = {r['storyId']: r for r in rows}
    far, near = [], []
    for r in ref:
        x = by.get(r['id'])
        if not x or x['storylineYear'] is None or x['tag'] == 'unextracted':
            continue
        (far if r['year'] < x['storylineYear'] - 3 else near).append((r['id'], r['year'], x))
    far = list({i: (i, y_, x) for i, y_, x in far}.values()); near = list({i: (i, y_, x) for i, y_, x in near}.values())
    hit_any = sum(x['tag'] in ('depicts', 'recounts') for _, _, x in far)
    hit_dep = sum(x['tag'] == 'depicts' for _, _, x in far)
    fp_dep = sum(x['tag'] == 'depicts' for _, _, x in near)
    print(f'wiki cites {len(far)} stories for a year well before their storyline: tagged depicts or recounts {hit_any} '
          f'({hit_any / max(len(far), 1):.3f}), depicts {hit_dep}')
    print(f'wiki cites {len(near)} stories near their storyline year: wrongly tagged depicts {fp_dep} ({fp_dep / max(len(near), 1):.3f})')
    for i, y_, x in far:
        if x['tag'] == 'none':
            print(f'  missed: {i} (wiki {y_}, storyline {x["storylineYear"]})')


WORDNUM = {w: i for i, w in enumerate('zero one two three four five six seven eight nine ten eleven twelve thirteen '
                                          'fourteen fifteen sixteen seventeen eighteen nineteen'.split())}
WORDNUM.update({'a': 1, 'an': 1, 'twenty': 20, 'thirty': 30, 'forty': 40, 'fifty': 50, 'sixty': 60, 'seventy': 70,
                'eighty': 80, 'ninety': 90, 'hundred': 100})
_UNIT = r"(?:zero|one|two|three|four|five|six|seven|eight|nine)"
_TEEN = r"(?:ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen)"
_TENS = r"(?:twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety)"
# Compound numbers first ("twenty-three"), so a bare "three" never matches inside one: the first version
# read "twenty-three years ago" as three years ago.
REL_RE = re.compile(r"(?<![\w-])(\d+|" + _TENS + r"[\s-]" + _UNIT + r"|" + _TENS + r"|" + _TEEN + r"|" + _UNIT +
                    r"|a|an|hundred)(?:[\s-]+(?:and\s+a\s+half|odd))?[\s-]+(years?|decades?|centuries|century)\s+"
                    r"(ago|later|before|after|earlier|prior|since|back)\b", re.I)


def number(tok):
    tok = tok.lower()
    if tok.isdigit():
        return int(tok)
    parts = re.split(r'[\s-]+', tok)
    return sum(WORDNUM[p] for p in parts) if all(p in WORDNUM for p in parts) else None


def stage_derive():
    """Years computed from 'N years ago/later' facts: the scene's present is the nearest dated caption before
    the quote in the same story (an 'ago' inside a flashback counts from the flashback), else the group's own
    dated anchor, else the storyline estimate (low confidence). Vague amounts are skipped."""
    t = json.load(open(os.path.join(OUT, 'timeline_v1.json')))
    grp = {g['groupId']: g for g in t['groups']}
    stories = load_stories()
    caps = collections.defaultdict(list)
    for e in read_jsonl(os.path.join(OUT, 'events.jsonl')):
        if e['is_year'] and e['present'] and e['kind'] == 'story':
            caps[e['source']].append(e)
    rows = []; skipped = collections.Counter()
    for r in read_jsonl(os.path.join(OUT, 'facts.jsonl')):
        s_ = r['storyId']; g = r['groupId']
        lines = [norm(x) for c in stories[s_] for x in c['text'].split('\n')]
        def pos(q):
            qn = norm(q)[:60]
            return next((i for i, x in enumerate(lines) if qn and qn in x), None)
        cap_pos = sorted((pos(e['line']), e['year']) for e in caps.get(s_, []) if pos(e['line']) is not None)
        for f in r['facts']:
            if f['type'] not in ('relative', 'flashback'):
                continue
            m = REL_RE.search(f['quote'])
            if not m:
                continue
            # A quote that states its own year ("Four Years Ago... Summer, 1090") is the explicit layer's.
            if re.search(r'\b1[01]\d\d\b', f['quote']):
                skipped['states its own year'] += 1
                continue
            n = number(m.group(1))
            if n is None:
                continue
            unit = m.group(2).lower()
            n = n * (10 if unit.startswith('decade') else 100 if unit.startswith('centur') else 1)
            sign = +1 if m.group(3).lower() in ('later', 'after') else -1
            p_ = pos(f['quote'])
            prior = [y for cp, y in cap_pos if p_ is not None and cp <= p_]
            if prior:
                base, basis = prior[-1], 'scene caption'
            elif grp.get(g, {}).get('basis', '').startswith('dated'):
                base, basis = grp[g]['storylineYear'], 'event anchor'
            elif g in grp:
                base, basis = grp[g]['storylineYear'], 'storyline estimate'
            else:
                skipped['no present year'] += 1
                continue
            # Round amounts are not precise offsets: "a century ago" for Siracusa's independence meant about
            # 131 years (Ian's review, 2026-09-27). Centuries, decades and hedged amounts are marked approximate.
            approx = (unit.startswith(('centur', 'decade')) or n % 10 == 0 and n >= 30 or
                      re.search(r"\b(almost|nearly|about|around|over|more than|some|roughly|close to)\b", f['quote'], re.I) is not None)
            rows.append({'storyId': s_, 'groupId': g, 'year': int(round(base + sign * n)), 'offset': sign * n, 'approximate': approx,
                         'from': round(base, 2), 'basis': basis, 'quote': f['quote'], 'when': f['when']})
    common.write_jsonl(os.path.join(OUT, 'derived.jsonl'), rows)
    print(f'derived: {len(rows)} years', dict(collections.Counter(x["basis"] for x in rows)), 'skipped', dict(skipped))
    # Eval: coarse (the wiki has an entry that year) and precise (the wiki cites this very story under that year).
    w = open(os.path.join(REF, 'timeline.wikitext')).read()
    wy = set(); cur = None
    for line in w.splitlines():
        mm = re.match(r'^=+\s*(\d{3,4})', line)
        if mm: cur = int(mm.group(1)); continue
        if re.match(r'^==[^=]', line): cur = None
        if line.lstrip().startswith('*'):
            wy |= {int(y) for y in re.findall(r'\b(\d{3,4})\b', re.sub(r'<ref>.*?</ref>', '', line)) if 300 <= int(y) <= 1110}
    cited = collections.defaultdict(set)
    for rr in json.load(open(os.path.join(REF, 'reference.json'))):
        if rr['kind'] == 's':
            cited[rr['id']].add(rr['year'])
    for b in ('scene caption', 'event anchor', 'storyline estimate'):
        xs = [x for x in rows if x['basis'] == b and x['year'] < 1097]
        if not xs:
            continue
        coarse = sum(x['year'] in wy for x in xs)
        pc = [x for x in xs if x['storyId'] in cited]
        exact = sum(x['year'] in cited[x['storyId']] for x in pc)
        near = sum(any(abs(x['year'] - y) <= 1 for y in cited[x['storyId']]) for x in pc)
        print(f'  {b}: {len(xs)} derived years before 1097; wiki has that year {coarse} ({coarse / len(xs):.3f}); '
              f'story also cited by the wiki {len(pc)}: same year {exact}, within 1 year {near}')


if __name__ == '__main__':
    if sys.argv[1] == 'order':
        stage_order(sys.argv[2], sys.argv[3]); sys.exit()
    {'derive': stage_derive, 'flashbacks': stage_flashbacks, 'build': stage_build, 'history': stage_history, 'cues': stage_cues, 'extract': stage_extract, 'eval': stage_eval, 'place': stage_place, 'events': stage_events, 'exceptions': stage_exceptions}[sys.argv[1]]()
