#!/usr/bin/env python3
"""Training pairs (question, passage) for fine-tuning the gte embedder on Trevor's corpus.

Ian (2026-09-29): fine-tune the embedder first (the research survey's next retrieval experiment; GPL reported up
to +9.3 nDCG@10 from generated in-domain pairs). The gold set generation files are not used: gold set v2 was
selected from them, so training on them would leak the evaluation. Instead the generator writes one question per
sampled passage from the story scripts and operator archives, excluding every story gold set v2 anchors to, in
two styles alternately: a full fan question (the answer bank's prompt) and a short casual one like the real
Discord questions ("Jie died?"). The answer bank's 900 questions (same exclusions) are added at train time.
  questions   generator model on $SERVER -> artifacts/embedft/pairs.jsonl (resumable; N_PAIRS, default 5000)
"""
import collections, os, random, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT, read_jsonl

OUT = os.path.join(ROOT, 'artifacts', 'embedft')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
N = int(os.environ.get('N_PAIRS', '5000'))
FULL = ("You read one passage from the Arknights story and write one question a fan might ask that this passage "
        "answers. Name the characters, place or event instead of using pronouns, so the question makes sense on its "
        "own. Ask about what happens, why, or who someone is, not about wording. Output the question only.")
# The first casual prompt allowed questions with no name at all ("how much money did the worker get"), which no
# retriever can place; real casual questions name who they are about ("Jie died?").
CASUAL = ("You read one passage from the Arknights story. Write one short question a player might type in a chat "
          "about it, the way players do: casual, often under 10 words, can skip punctuation, but it must name the "
          "character, place or event it is about. It must be answerable from this passage. Output the question only.")


def chat(system, user):
    """The first line of the model's answer: one question."""
    return common.chat(SERVER, system, user, 80, 600, lambda d: common.content(d).strip().split('\n')[0].strip())


def gold_stories():
    return {an['story_id'] for g in read_jsonl(os.path.join(ROOT, 'eval', 'goldset_v2.jsonl')) for an in g.get('anchors', [])}


def stage_questions():
    os.makedirs(OUT, exist_ok=True)
    gold = gold_stories()
    by_g = collections.defaultdict(list)
    for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'p3a', 'chunks.jsonl')):
        if c['storyId'] not in gold and len(c['text']) > 300:
            by_g[c['groupId']].append(c)
    rng = random.Random(9)
    for v in by_g.values():
        rng.shuffle(v)
    order = sorted(by_g); rng.shuffle(order)
    picked = []
    while len(picked) < N and any(by_g.values()):
        for g in order:
            if by_g[g] and len(picked) < N:
                picked.append(by_g[g].pop())
    path = os.path.join(OUT, 'pairs.jsonl')
    done = {r['chunkId'] for r in read_jsonl(path)}
    todo = [(i, c) for i, c in enumerate(picked) if c['chunkId'] not in done]
    print(f'pairs: {len(todo)} to do of {len(picked)} (gold stories excluded: {len(gold)})', flush=True)
    lock = threading.Lock(); n = [0]; t0 = time.time()

    def one(x):
        i, c = x
        style = 'casual' if i % 2 else 'full'
        q = chat(CASUAL if style == 'casual' else FULL, c['text'][:6000])
        with lock:
            common.append_jsonl(path, {'qid': f'e{i:05d}', 'question': q, 'style': style, 'chunkId': c['chunkId'],
                                       'storyId': c['storyId']})
            n[0] += 1
            if n[0] % 250 == 0:
                print(f'{time.strftime("%H:%M:%S")} pairs {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.2f} s each', flush=True)
    common.par(todo, one, 2, lock)
    print('pairs done', flush=True)


if __name__ == '__main__':
    {'questions': stage_questions}[sys.argv[1]]()
