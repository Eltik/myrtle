#!/usr/bin/env python3
"""A summary tree for whole-story questions (2026-10-01): one summary per game storyline, then the whole story and Terra.

"Summarize the Arknights story in one word" was declined: retrieval finds passages about one scene, and nothing in the
corpus states the whole story at once. This builds the missing top of the tree offline, from Trevor's own summaries:
  storyline  one summary per storyline of the game's Movements view (artifacts/chrono/storylines.json, the main story
             "For Tomorrow" included), from its events' P2 group summaries (artifacts/p2/groups.jsonl, method C) in the
             game's order
  story      one overview of the whole Arknights story, from the storyline summaries
  world      one overview of the world of Terra, from the storyline summaries and the topic summaries
             (artifacts/topics/topics.jsonl)
  gen        generator model on $SERVER (Gemma): every unit above, in that order (resumable, keyed on the exact input
             and the prompt, as scripts/incr.py; PLAN=1 prints what would be redone)
  judge      judge model on $SERVER (Qwen3.5 9B): each sentence of each summary against that summary's own input
             -> artifacts/overview/judged.jsonl (keyed on id + summary + prompt), per-unit score and the worst 5
-> artifacts/overview/overview.jsonl, one row per unit: id, kind, name, summary, sources, inputSha, promptSha.
"""
import collections, os, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import CITE, ROOT, read_jsonl, sha16

OUT = os.path.join(ROOT, 'artifacts', 'overview')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
PLAN = os.environ.get('PLAN') == '1'

STORYLINE_SYS = (
    "You write an overview of one storyline of the Arknights story for a reader who wants the whole of it at once, using "
    "only the numbered event summaries given, which are in the game's order. Write 250 to 400 words of plain prose: what "
    "the storyline is about, its main people, places and conflicts, how it develops from event to event, and where it "
    "stands at the end of the last event. If the type is DISCRETE, the events are separate stories grouped together: say "
    "so, give what they have in common, and name each event with one clause on what it is about. Cite events as [n] after "
    "each sentence that uses them. Add no outside knowledge. No preamble.")
STORY_SYS = (
    "You write an overview of the whole Arknights story for a reader who asks what the story is about as a whole, using "
    "only the numbered storyline summaries given. Passage [1] is the main story; the others are side storylines. Write "
    "300 to 450 words of plain prose: the main story's arc from beginning to the latest event, its central conflicts and "
    "themes, then how the side storylines widen it (which nations and groups they follow, and how they connect to the "
    "main story). Cite passages as [n] after each sentence that uses them. Add no outside knowledge. No preamble.")
WORLD_SYS = (
    "You write an overview of the world of Terra, the setting of Arknights, for a reader new to it, using only the "
    "numbered passages given (storyline summaries, then topic summaries of races, nations and concepts). Write 300 to 450 "
    "words of plain prose: what Terra is like (Originium, Oripathy and the Infected, Catastrophes, mobile cities, Arts), "
    "its main nations and peoples and how they relate, and the forces the story turns on. Cite passages as [n] after each "
    "sentence that uses them. Add no outside knowledge. No preamble.")
PROMPTS = {'storyline': STORYLINE_SYS, 'story': STORY_SYS, 'world': WORLD_SYS}
# A topic summary is cut to its first TOPIC_WORDS words in the world unit's input (46 topics in full are 12,857 words; at 80 the world input is about 8,700 words, inside a 16,384-token slot).
TOPIC_WORDS = int(os.environ.get('OVERVIEW_TOPIC_WORDS', '80'))


def chat(system, user, max_tokens=900):
    return common.chat(SERVER, system, user, max_tokens, 1800)


def storyline_units():
    s = common.load_json(os.path.join(ROOT, 'artifacts', 'chrono', 'storylines.json'))
    groups = {r['groupId']: r['summary'] for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'p2', 'groups.jsonl')) if r['method'] == 'C'}
    units = []
    for l in s['storylines']:
        evs = [(g, n) for g, n in zip(l['groups'], l['names']) if groups.get(g)]
        user = (f"STORYLINE: {l['name']} (type {l['type']})\n\nEVENT SUMMARIES\n" +
                ''.join(f"\n[{i}] {n}\n{CITE.sub('', groups[g])}\n" for i, (g, n) in enumerate(evs, 1)))
        units.append({'id': l['storylineId'], 'kind': 'storyline', 'name': l['name'], 'sources': [g for g, _ in evs], 'input': user})
    return units


def top_units(done):
    """The story and world units, from the storyline summaries in `done` (id -> row with summary)."""
    sl = [r for r in done.values() if r['kind'] == 'storyline' and r.get('summary')]
    sl.sort(key=lambda r: (r['id'] != 'mainLine', r['id']))
    lines = ''.join(f"\n[{i}] Storyline: {r['name']}\n{CITE.sub('', r['summary'])}\n" for i, r in enumerate(sl, 1))
    story = {'id': 'story', 'kind': 'story', 'name': 'The Arknights story', 'sources': [r['id'] for r in sl],
             'input': f"STORYLINE SUMMARIES\n{lines}"}
    # The 46 topics only (rows without a `source`): the topics from data (2026-10-01) would not fit the world unit's
    # input, and the world unit stays as it was (OVERVIEW_ALL_TOPICS=1 takes every served topic).
    topics = [t for t in read_jsonl(os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl')) if len((t.get('summary') or '').split()) >= 100
              and ('source' not in t or (os.environ.get('OVERVIEW_ALL_TOPICS') == '1' and not t.get('thin')))]
    tl = ''.join(f"\n[{len(sl) + i}] Topic: {t['topic']}\n{' '.join(CITE.sub('', t['summary']).split()[:TOPIC_WORDS])}\n"
                 for i, t in enumerate(topics, 1))
    world = {'id': 'world', 'kind': 'world', 'name': 'The world of Terra', 'sources': [r['id'] for r in sl] + ['topic:' + t['topic'] for t in topics],
             'input': f"PASSAGES\n{lines}{tl}"}
    return [story, world]


def stale(row, unit):
    if not row or not row.get('summary'):
        return 'new'
    if row.get('promptSha') != sha16(PROMPTS[unit['kind']]):
        return 'prompt changed'
    if row.get('inputSha') != sha16(unit['input']):
        return 'input changed'
    return None


def stage_gen():
    os.makedirs(OUT, exist_ok=True)
    path = os.path.join(OUT, 'overview.jsonl')
    rows = {r['id']: r for r in read_jsonl(path)}

    def save():
        common.write_jsonl(path, rows.values())

    t0 = time.time(); n = 0
    for phase in ('storyline', 'top'):
        units = storyline_units() if phase == 'storyline' else top_units(rows)
        why = {u['id']: stale(rows.get(u['id']), u) for u in units}
        # Under PLAN a pending storyline makes both top units pending (their input is its summary).
        if PLAN and phase == 'top' and any(stale(rows.get(u['id']), u) for u in storyline_units()):
            why = {u['id']: why[u['id']] or 'upstream pending' for u in units}
        todo = [u for u in units if why[u['id']]]
        if PLAN:
            print(f"overview {phase}: plan {len(todo)} to do of {len(units)} "
                  f"({dict(collections.Counter(w for w in why.values() if w)) or 'nothing changed'})", flush=True)
            continue
        for u in todo:
            summary = chat(PROMPTS[u['kind']], u['input'])
            rows[u['id']] = {'id': u['id'], 'kind': u['kind'], 'name': u['name'], 'sources': u['sources'], 'summary': summary,
                             'inputSha': sha16(u['input']), 'promptSha': sha16(PROMPTS[u['kind']])}
            save(); n += 1
            print(f"{time.strftime('%H:%M:%S')} {u['id']} ({u['name']}): {len(summary.split())} words from "
                  f"{len(u['input'].split())} ({(time.time() - t0) / n:.0f} s each)", flush=True)
    if not PLAN:
        print(f'gen done: {n} written', flush=True)


SUPPORT_SYS = ("You check one claim about the Arknights story against evidence: summaries of its events. Answer true only "
               "if the evidence states or clearly implies the claim. Answer false otherwise. Answer with true or false only.")
JSHA = sha16(SUPPORT_SYS)
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bLt\.)(?<!\bJr\.)(?<!\bSr\.)(?<!\bMt\.)(?<!\bNo\.)(?<!\bVol\.)'


def sentences(t):
    # As topics.py: sentences over 20 characters, citations removed so the judge sees the claim only.
    return common.split_sentences(CITE.sub('', t), ABBR)


def verdict(evidence, claim):
    return common.verdict(SERVER, SUPPORT_SYS, f'EVIDENCE:\n{evidence}\n\nCLAIM: {claim}', 900)


def stage_judge():
    rows = {r['id']: r for r in read_jsonl(os.path.join(OUT, 'overview.jsonl')) if r.get('summary')}
    inputs = {u['id']: u['input'] for u in storyline_units() + top_units(rows)}
    path = os.path.join(OUT, 'judged.jsonl')
    done = {(j['id'], j['key']) for j in read_jsonl(path)}
    t0 = time.time(); n = 0
    for r in rows.values():
        key = sha16(r['id'] + r['summary'] + JSHA)
        if (r['id'], key) in done:
            continue
        if PLAN:
            n += 1; continue
        vs = [(c, verdict(inputs[r['id']], c)) for c in sentences(r['summary'])]
        row = {'id': r['id'], 'key': key, 'sentences': len(vs), 'supported': sum(v for _, v in vs), 'unsupported': [c for c, v in vs if not v]}
        common.append_jsonl(path, row)
        n += 1
        print(f"{time.strftime('%H:%M:%S')} {r['id']}: {row['supported']}/{row['sentences']} supported ({(time.time() - t0) / n:.0f} s each)", flush=True)
    if PLAN:
        print(f'overview judge: plan {n} to do of {len(rows)}', flush=True)
        return
    keys = {r['id']: sha16(r['id'] + r['summary'] + JSHA) for r in rows.values()}
    J = [j for j in read_jsonl(path) if keys.get(j['id']) == j['key']]
    s = sum(j['supported'] for j in J); t = sum(j['sentences'] for j in J)
    print(f'judge: {len(J)} units, {s}/{t} sentences supported = {s / max(t, 1):.3f}; '
          f"{sum(j['supported'] == j['sentences'] for j in J)} fully supported", flush=True)
    for j in sorted(J, key=lambda j: j['supported'] / max(j['sentences'], 1))[:5]:
        print(f"  {j['id']}: {j['supported']}/{j['sentences']}")
        for c in j['unsupported']:
            print(f'    - {c}')


if __name__ == '__main__':
    {'gen': stage_gen, 'judge': stage_judge}[sys.argv[1]]()
