#!/usr/bin/env python3
"""P2: story and group summaries with a local model, resumable.

  story   one paragraph (100-150 words) per story from its full script, every
          scripted story, group by group in story order -> artifacts/p2/stories.jsonl
  group   one summary (300-400 words) per multi-story group, by reducing its
          stories' summaries in order. Method B reduces the model's story
          summaries, method A the game's official synopses, method C both side
          by side; all three are written and `method` says which
          -> artifacts/p2/groups.jsonl
  group --method D [--groups a,b | --coverage-below T] [--dump-prompts]
          method D, sectioned (2026-10-19: method C spent its words on the first
          stories of long chapters, design/experiment-log.md): a group of more than
          GROUP_PARTS_MIN (8) stories is split into 3 to 5 consecutive parts of
          about equal size, each part is summarized in 100 to 130 words
          (prompts/summary.group.part.txt), then one merge call joins the parts
          into the 300 to 400 word summary (prompts/summary.group.merge.txt).
          Only the selected groups get a D row (`parts` holds the part
          summaries); once a group has one, a plain `group` run keeps it up to
          date. Readers serve D over C (common.served_group_rows;
          SUMMARY_METHOD_D=0 serves C). --coverage-below T selects the groups
          scripts/summary_coverage.py flags at --threshold T (coverage.json);
          --dump-prompts renders the part and merge prompts with their token
          counts and calls no model.

Needs a llama-server on $SERVER (default http://127.0.0.1:8081) running the
generator. Every row records the model, the sha of its prompt and the sha of its
input, and a stage skips a unit only when all three still match (scripts/incr.py:
PLAN=1 dry run, KEY_IDS_ONLY=1 the old id-only skip), so a crash or a pause costs
one unit and a changed story redoes its summary and then its group's. The
generator pauses while the Mac runs on battery and resumes on AC.
Design and pilot numbers: design/trevor-retrieval-baseline.md section 9.
"""
import argparse, collections, json, os, subprocess, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common, incr
from common import ROOT, read_jsonl, sha16

OUT = os.path.join(ROOT, 'artifacts', 'p2')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
SLOTS = int(os.environ.get('SLOTS', '2'))
STORY_SYS = open(os.path.join(ROOT, 'prompts', 'summary.story.txt')).read()
GROUP_SYS = open(os.path.join(ROOT, 'prompts', 'summary.group.txt')).read()
# Method D (sectioned). GROUP_SYS is untouched, so methods A, B and C keep their prompt sha; D rows carry D_PSHA, the
# sha of both prompts and both user templates, and a change to any of them redoes only the D rows.
PART_SYS = open(os.path.join(ROOT, 'prompts', 'summary.group.part.txt')).read()
MERGE_SYS = open(os.path.join(ROOT, 'prompts', 'summary.group.merge.txt')).read()
PART_USER = ('Part {i} of {k}, stories {a} to {b} of {n} of {name}. Summarize the events of this part only.\n\n'
             'Story summaries, in order:\n{body}')
MERGE_USER = '{name}: the summaries of its {k} parts, in order, covering its {n} stories.\n\n{body}'
D_PSHA = sha16(json.dumps([PART_SYS, MERGE_SYS, PART_USER, MERGE_USER]))
# A group of more than this many stories is sectioned; a shorter one gets no D row (C serves it).
PARTS_MIN = int(os.environ.get('GROUP_PARTS_MIN', '8'))
PART_TOKENS, MERGE_TOKENS = 260, 700


def on_battery():
    try:
        return "'Battery Power'" in subprocess.run(['pmset', '-g', 'batt'], capture_output=True, text=True).stdout
    except OSError:
        return False


def wait_for_ac():
    if on_battery():
        print(f'{time.strftime("%H:%M:%S")} on battery: pausing until AC power', flush=True)
        while on_battery():
            time.sleep(60)
        print(f'{time.strftime("%H:%M:%S")} AC power: resuming', flush=True)


def chat(system, user, max_tokens):
    # (stripped text, timings); one failed decode 500s every in-flight request, so common.chat retries.
    return common.chat(SERVER, system, user, max_tokens, 1800, lambda d: (common.content(d).strip(), d['timings']))


def load():
    chunks = [json.loads(l) for l in open(incr.chunks_path())]
    pre = {}
    for l in open(os.path.join(ROOT, 'artifacts', 'p1a', 'chunks.jsonl')):
        r = json.loads(l)
        pre.setdefault(r['storyId'], r['prefix'].split('\n'))
    by_story = collections.defaultdict(list)
    for c in chunks:
        by_story[c['storyId']].append(c)
    facts = {}
    for s, lines in pre.items():
        pos = next((l for l in lines if l.startswith('Story ')), 'Story 0 of 0')
        facts[s] = {'header': lines[0],
                    'synopsis': next((l[len('Synopsis: '):] for l in lines if l.startswith('Synopsis: ')), ''),
                    'n': int(pos.split()[1])}
    groups = collections.defaultdict(list)
    for s, v in by_story.items():
        v.sort(key=lambda c: c['ordinal'])
        groups[v[0]['groupId']].append(s)
    for g in groups:
        groups[g].sort(key=lambda s: facts[s]['n'])
    return by_story, facts, groups


def story_input(cs):
    # Keyed on the script text alone, chunks in ordinal order (design/trevor-updates.md); the header
    # line comes from p1a and changes only with a rename.
    return '\n'.join(c['text'] for c in cs)


def story_stage():
    by_story, facts, groups = load()
    st = incr.Stage('p2 story', os.path.join(OUT, 'stories.jsonl'), lambda r: r['storyId'], sha16(STORY_SYS))
    order = [s for g in sorted(groups, key=lambda g: (g.startswith('story_'), g)) for s in groups[g]]
    for s in order:
        st.unit(s, story_input(by_story[s]))
    return st, order, None, by_story, facts


def stage_story():
    st, order, live, by_story, facts = story_stage()
    todo = st.plan(order, live)
    if todo is None:
        return
    os.makedirs(OUT, exist_ok=True)
    model = common.model_id(SERVER, 30); psha = st.psha
    print(f'story: {len(todo)} to do, {len(order) - len(todo)} done, model {model}, prompt {psha}', flush=True)
    lock = threading.Lock(); n = [0]; t0 = time.time()

    def one(s):
        wait_for_ac()
        text = story_input(by_story[s])
        started = time.time()
        out, t = chat(STORY_SYS, f"{facts[s]['header']}\n\nScript:\n{text}", 320)
        row = {'storyId': s, 'groupId': by_story[s][0]['groupId'], 'header': facts[s]['header'],
               'officialSynopsis': facts[s]['synopsis'], 'summary': out, 'model': model, 'promptSha': psha,
               'promptTokens': t['prompt_n'], 'genTokens': t['predicted_n'], 'seconds': round(time.time() - started, 1)}
        with lock:
            st.put(row)
            n[0] += 1
            if n[0] % 25 == 0:
                el = time.time() - t0
                print(f'{time.strftime("%H:%M:%S")} story {n[0]}/{len(todo)}, {el / n[0]:.1f} s each, '
                      f'about {(len(todo) - n[0]) * el / n[0] / 3600:.1f} h left', flush=True)
    common.par(todo, one, SLOTS, lock)
    print(f'story: {n[0]} written in {time.time() - t0:.0f}s', flush=True)


def story_line(s, method, facts, stories):
    src = {'A': facts[s]['synopsis'], 'B': stories.get(s, ''),
           'C': f"Official synopsis: {facts[s]['synopsis']} Summary: {stories.get(s, '')}"}[method]
    return f"{facts[s]['header']}: {src}"


def group_body(g, method, groups, facts, stories):
    return '\n'.join(story_line(s, method, facts, stories) for s in groups[g])


def split_parts(n, min_stories=None, per_part=7, lo=3, hi=5):
    """Method D's parts of a group of n stories: [(start, end)] half-open, consecutive, in story order, sizes within one
    of each other, lo to hi parts of about per_part stories; [] (not sectioned) for min_stories (8) stories or fewer."""
    if n <= (PARTS_MIN if min_stories is None else min_stories):
        return []
    k = max(lo, min(hi, -(-n // per_part)))
    b = [(i * n + k // 2) // k for i in range(k + 1)]
    return list(zip(b, b[1:]))


def group_name(g, groups, facts):
    return facts[groups[g][0]]['header'].split(',')[0].strip()


def part_bodies(g, groups, facts, stories):
    """The user bodies of method D's part calls, in order ([] when the group is not sectioned): the unit's model input."""
    ids, name = groups[g], group_name(g, groups, facts)
    parts = split_parts(len(ids))
    return [PART_USER.format(i=i + 1, k=len(parts), a=a + 1, b=b, n=len(ids), name=name,
                             body='\n'.join(story_line(s, 'C', facts, stories) for s in ids[a:b]))
            for i, (a, b) in enumerate(parts)]


def merge_body(g, groups, facts, part_summaries):
    ids = groups[g]
    parts = split_parts(len(ids))
    body = '\n\n'.join(f'Part {i + 1} of {len(parts)} (stories {a + 1} to {b}): {t}'
                       for i, ((a, b), t) in enumerate(zip(parts, part_summaries)))
    return MERGE_USER.format(name=group_name(g, groups, facts), k=len(parts), n=len(ids), body=body)


def group_stage(pending_stories=None, methods=None, select=None):
    """`methods`: the methods to run, default C, B, A plus D for every group that already has a D row (so a plain run
    keeps the D rows up to date); `select`: the group ids to run (D units are registered for these)."""
    by_story, facts, groups = load()
    stories = {r['storyId']: r['summary'] for r in read_jsonl(os.path.join(OUT, 'stories.jsonl'))}
    st = incr.Stage('p2 group', os.path.join(OUT, 'groups.jsonl'), lambda r: (r['groupId'], r['method']), sha16(GROUP_SYS))
    if incr.PLAN and pending_stories is None:
        pending_stories = story_stage()[0].stale()
    targets = [g for g in sorted(groups) if len(groups[g]) > 1]
    order = [(g, m) for g in targets for m in ('C', 'B', 'A')]
    for g, m in order:
        # The input is the reduced text itself: a redone story summary changes it for B and C, never A.
        st.unit((g, m), group_body(g, m, groups, facts, stories),
                upstream=m != 'A' and any(s in (pending_stories or ()) for s in groups[g]))
    # Method D: a unit for every selected group and every group with a D row, so those rows stay live and follow their
    # stories; a group too short to section has no D unit (an old D row of it is dropped as gone).
    have_d = {k[0] for k in st.have if k[1] == 'D'}
    want_d = set(select or ()) if methods and 'D' in methods else set()
    d_groups = [g for g in targets if (g in have_d or g in want_d) and split_parts(len(groups[g]))]
    for g in d_groups:
        st.unit((g, 'D'), part_bodies(g, groups, facts, stories),
                upstream=any(s in (pending_stories or ()) for s in groups[g]), psha=D_PSHA)
    if methods is None:
        order += [(g, 'D') for g in d_groups if g in have_d]
    else:
        order = [(g, m) for g in targets for m in ('C', 'B', 'A') if m in methods]
        if 'D' in methods:
            order += [(g, 'D') for g in d_groups]
    if select:
        keep = set(select)
        order = [(g, m) for g, m in order if g in keep]
    return st, order, None, groups, facts, stories, targets


def coverage_groups(threshold, path):
    """The groups scripts/summary_coverage.py flags at this threshold: missing share above it, 6 or more stories."""
    cov = common.load_json(path)
    return sorted(g for g, r in cov['groups'].items() if r['stories'] >= cov['minStories'] and r['share'] > threshold)


def token_count(text):
    """Gemma tokens of a text, from the GGUF's vocabulary alone (llama-tokenize loads no weights: about 0.4 s and
    200 MB); None when the tool or the file is missing."""
    gguf = os.environ.get('TOKENIZER_GGUF', os.path.join(ROOT, 'models', 'llm', 'gemma-4-12b-it-qat-q4_0.gguf'))
    try:
        out = subprocess.run(['llama-tokenize', '-m', gguf, '--stdin', '--show-count', '--log-disable'], input=text,
                             capture_output=True, text=True, timeout=120).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    last = [l for l in out.splitlines() if l.startswith('Total number of tokens:')]
    return int(last[-1].split(':')[1]) if last else None


def dump_prompts(targets, groups, facts, stories):
    """Method D's part prompts and merge prompt (the part summaries as placeholders) per group, with the token count of
    system plus user text (the chat template adds a few). Calls no model, writes nothing."""
    for g in targets:
        bodies = part_bodies(g, groups, facts, stories)
        print(f'== {g}: {len(groups[g])} stories in {len(bodies)} parts {split_parts(len(groups[g]))}', flush=True)
        for i, b in enumerate(bodies):
            n = token_count(PART_SYS + '\n\n' + b)
            print(f'-- {g} part {i + 1}: {n} tokens (system + user), at most {PART_TOKENS} generated\n'
                  f'[system]\n{PART_SYS}\n[user]\n{b}\n', flush=True)
        m = merge_body(g, groups, facts, [f'<part {i + 1} summary, 100 to 130 words>' for i in range(len(bodies))])
        n = token_count(MERGE_SYS + '\n\n' + m)
        print(f'-- {g} merge: {n} tokens with placeholders, plus the part summaries (at most {len(bodies)} x '
              f'{PART_TOKENS}), at most {MERGE_TOKENS} generated\n[system]\n{MERGE_SYS}\n[user]\n{m}\n', flush=True)


def parse_group_args(argv):
    ap = argparse.ArgumentParser(prog='p2-summaries.py group')
    ap.add_argument('--method', help='comma list of A, B, C, D (default: C, B, A and the D rows that exist)')
    sel = ap.add_mutually_exclusive_group()
    sel.add_argument('--groups', help='comma list of group ids to run')
    sel.add_argument('--coverage-below', type=float, metavar='T',
                     help='the groups summary_coverage.py flags at --threshold T (missing share above T, 6+ stories)')
    ap.add_argument('--coverage', default=os.path.join(OUT, 'coverage.json'), help='the coverage.json to read')
    ap.add_argument('--dump-prompts', action='store_true', help="print method D's prompts with token counts; no model")
    a = ap.parse_args(argv)
    methods = a.method.split(',') if a.method else None
    if methods and not set(methods) <= set('ABCD'):
        ap.error('--method takes A, B, C and D')
    select = (a.groups.split(',') if a.groups else
              coverage_groups(a.coverage_below, a.coverage) if a.coverage_below is not None else None)
    return a, methods, select


def stage_group(argv=()):
    a, methods, select = parse_group_args(list(argv))
    st, order, live, groups, facts, stories, targets = group_stage(methods=methods, select=select)
    if select:
        unknown = [g for g in select if g not in groups]
        if unknown:
            print(f'group: unknown groups {unknown}', file=sys.stderr, flush=True)
            sys.exit(2)
        short = [g for g in select if not split_parts(len(groups[g]))]
        if short and methods and 'D' in methods:
            print(f'group: not sectioned (at most {PARTS_MIN} stories), C keeps serving: {short}', flush=True)
    if a.dump_prompts:
        dump_prompts([g for g, m in order if m == 'D'], groups, facts, stories)
        return
    todo = st.plan(order, live)
    if todo is None:
        return
    path = st.path
    model = common.model_id(SERVER, 30); psha = st.psha
    print(f'group: {len(targets)} multi-story groups, model {model}, prompt {psha}', flush=True)
    for g, method in todo:
        if method in ('B', 'C', 'D') and not all(s in stories for s in groups[g]):
            print(f'group: {g} skipped, story summaries incomplete', flush=True)
            continue
        wait_for_ac()
        started = time.time()
        if method == 'D':
            parts = []
            for i, ((x, y), body) in enumerate(zip(split_parts(len(groups[g])), part_bodies(g, groups, facts, stories))):
                out, t = chat(PART_SYS, body, PART_TOKENS)
                parts.append({'part': i + 1, 'storyIds': groups[g][x:y], 'summary': out, 'promptTokens': t['prompt_n'],
                              'genTokens': t['predicted_n']})
            out, t = chat(MERGE_SYS, merge_body(g, groups, facts, [p['summary'] for p in parts]), MERGE_TOKENS)
            row = {'groupId': g, 'method': 'D', 'summary': out, 'parts': parts, 'storyIds': groups[g], 'model': model,
                   'promptSha': D_PSHA, 'promptTokens': t['prompt_n'], 'seconds': round(time.time() - started, 1)}
            print(f'group: {g} D, {len(parts)} parts, {len(out.split())} words, {row["seconds"]} s', flush=True)
        else:
            out, t = chat(GROUP_SYS, f'Story summaries, in order:\n{group_body(g, method, groups, facts, stories)}', 700)
            row = {'groupId': g, 'method': method, 'summary': out, 'storyIds': groups[g], 'model': model,
                   'promptSha': psha, 'promptTokens': t['prompt_n'], 'seconds': round(time.time() - started, 1)}
        st.put(row)
    print(f'group: {len(read_jsonl(path))} rows', flush=True)


if __name__ == '__main__':
    {'story': stage_story, 'group': lambda: stage_group(sys.argv[2:])}[sys.argv[1]]()
