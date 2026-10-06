#!/usr/bin/env python3
"""P2: story and group summaries with a local model, resumable.

  story   one paragraph (100-150 words) per story from its full script, every
          scripted story, group by group in story order -> artifacts/p2/stories.jsonl
  group   one summary (300-400 words) per multi-story group, by reducing its
          stories' summaries in order. Method B reduces the model's story
          summaries, method A the game's official synopses, method C both side
          by side; all three are written and `method` says which
          -> artifacts/p2/groups.jsonl

Needs a llama-server on $SERVER (default http://127.0.0.1:8081) running the
generator. Every row records the model, the sha of its prompt and the sha of its
input, and a stage skips a unit only when all three still match (scripts/incr.py:
PLAN=1 dry run, KEY_IDS_ONLY=1 the old id-only skip), so a crash or a pause costs
one unit and a changed story redoes its summary and then its group's. The
generator pauses while the Mac runs on battery and resumes on AC.
Design and pilot numbers: design/trevor-retrieval-baseline.md section 9.
"""
import collections, json, os, subprocess, sys, threading, time, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common, incr
from common import ROOT, read_jsonl, sha16

OUT = os.path.join(ROOT, 'artifacts', 'p2')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
SLOTS = int(os.environ.get('SLOTS', '2'))
STORY_SYS = open(os.path.join(ROOT, 'prompts', 'summary.story.txt')).read()
GROUP_SYS = open(os.path.join(ROOT, 'prompts', 'summary.group.txt')).read()


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


def model_name():
    with urllib.request.urlopen(f'{SERVER}/v1/models', timeout=30) as r:
        return os.path.basename(json.load(r)['data'][0]['id'])


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
    model = model_name(); psha = st.psha
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


def group_body(g, method, groups, facts, stories):
    src = {s: {'A': facts[s]['synopsis'], 'B': stories.get(s, ''),
               'C': f"Official synopsis: {facts[s]['synopsis']} Summary: {stories.get(s, '')}"}[method]
           for s in groups[g]}
    return '\n'.join(f"{facts[s]['header']}: {src[s]}" for s in groups[g])


def group_stage(pending_stories=None):
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
    return st, order, None, groups, facts, stories, targets


def stage_group():
    st, order, live, groups, facts, stories, targets = group_stage()
    todo = st.plan(order, live)
    if todo is None:
        return
    todo = set(todo)
    path = st.path
    model = model_name(); psha = st.psha
    print(f'group: {len(targets)} multi-story groups, model {model}, prompt {psha}', flush=True)
    for g in targets:
        for method in ('C', 'B', 'A'):
            if (g, method) not in todo:
                continue
            if method in ('B', 'C') and not all(s in stories for s in groups[g]):
                print(f'group: {g} skipped, story summaries incomplete', flush=True)
                continue
            wait_for_ac()
            body = group_body(g, method, groups, facts, stories)
            started = time.time()
            out, t = chat(GROUP_SYS, f'Story summaries, in order:\n{body}', 700)
            row = {'groupId': g, 'method': method, 'summary': out, 'storyIds': groups[g], 'model': model,
                   'promptSha': psha, 'promptTokens': t['prompt_n'], 'seconds': round(time.time() - started, 1)}
            st.put(row)
    print(f'group: {len(read_jsonl(path))} rows', flush=True)


if __name__ == '__main__':
    {'story': stage_story, 'group': stage_group}[sys.argv[1]]()
