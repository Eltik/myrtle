#!/usr/bin/env python3
"""Speaker labels a story renames by a self-introduction (2026-10-19), for `ask`'s passage labels.

A script often labels a stranger by a description ("Gap-Toothed Girl") until they give their name, after which the
label is the name ("Idida: Dadadeedeeda, I'm called Idida."). A model reading the passage keeps the description and
answers "the Gap-Toothed Girl" when asked for the name (window 6 of 2026-10-19). This finds, per story, a label with a
description word followed within 3 lines by a new label that introduces itself by that name ("I'm called X", "my name
is X", "call me X", "I'm X", "I am X"), where that line is the first line of the new label in the story. Descriptions
like "Passing Girl" recur across stories for different people, so every link is scoped to its story.

Measured on P3b (2026-10-19): 38 description-to-introduction pairs, 11 kept by the first-line condition, all true
renames on reading (Gap-Toothed Girl -> Idida, Gentle Woman -> Mercia, Injured Old Man -> Chen Xiuqing, ...); the 27
dropped hold both true and false ones (Calm Youth -> Red, Soldier -> Alita are different people).

A second pass (2026-10-19) does the same for the label "???" per chunk: a "???:" line followed within 3 lines by a
label's first line of the chunk that introduces itself by that label ("???: ...Kal'tsit?" then "Mon3tr: My name is
Mon3tr."), kept only when no "???" line of the chunk comes after the introduction, because "???" is 2,846 lines in 912
stories and names different people within one story; `ask` renders it "??? (Mon3tr):" and credits the line to that
speaker (`--no-chunk-renames`). On P3b: 20 chunks match, 18 kept, 17 of them right on reading.

  python3 scripts/label_renames.py   -> artifacts/entities/label_renames.json  {storyId: {label: name}}
                                        artifacts/entities/label_renames_chunks.json  {chunkId: {"???": name}}
"""
import collections, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT, read_derived

CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3b'))
OUT = os.path.join(ROOT, 'artifacts', 'entities', 'label_renames.json')
OUT_CHUNKS = os.path.join(ROOT, 'artifacts', 'entities', 'label_renames_chunks.json')
UNKNOWN = '???'
INTRO = re.compile(r"\b(?:I'm called|I am called|my name is|My name is|call me|Call me|I'm|I am)\s+"
                   r"([A-Z][\w'\-]+(?:\s[A-Z][\w'\-]+)?)")
DESCRIPTION = re.compile(r'\b(Girl|Boy|Man|Woman|Youth|Kid|Child|Soldier|Guard|Voice|Stranger|Lady|Gentleman|Old|Young|'
                         r'Mysterious|Hooded)\b')
LABEL = re.compile(r'^([^:\n]{1,40}):\s*(.*)$')


def introduced(lab, i, skip):
    """The label that introduces itself by its name within 3 lines after line `i` of the labelled lines `lab`, on its
    first line there, with that line's index; labels in `skip` are passed over. None when the first introduction
    found is not a first line, or there is none."""
    for j in range(i + 1, min(i + 4, len(lab))):
        s2, t2 = lab[j]
        if not s2 or s2 in skip:
            continue
        m = INTRO.search(t2)
        if m and m.group(1).split()[0] == s2.split()[0]:
            return (s2, j) if not any(x == s2 for x, _ in lab[:j]) else None
    return None


def labelled(text):
    return [(m.group(1), m.group(2)) if (m := LABEL.match(x)) else (None, x) for x in text.split('\n')]


def chunk_renames(chunks):
    """{chunkId: {"???": name}}: a chunk's "???" lines named by the first introduction that follows one of them, when
    every "???" line of the chunk comes before it (so a chunk never gives two names)."""
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        found = sorted((j, n) for i in unknown if (f := introduced(lab, i, {UNKNOWN})) for n, j in [f])
        if found and unknown[-1] < found[0][0]:
            out[c['chunkId']] = {UNKNOWN: found[0][1]}
    return out


def main():
    stories = collections.defaultdict(list)
    chunks = read_derived(os.path.join(CORPUS, 'chunks.jsonl'))
    for c in chunks:
        stories[c['storyId']].append((c.get('ordinal', 0), c['text']))
    out = {}
    for sid, cs in sorted(stories.items()):
        lines = [x for _, t in sorted(cs) for x in t.split('\n')]
        lab = [(m.group(1), m.group(2)) if (m := LABEL.match(x)) else (None, x) for x in lines]
        for i, (s, _) in enumerate(lab):
            if not s or not DESCRIPTION.search(s):
                continue
            for j in range(i + 1, min(i + 4, len(lab))):
                s2, t2 = lab[j]
                if not s2 or s2 == s:
                    continue
                m = INTRO.search(t2)
                if m and m.group(1).split()[0] == s2.split()[0]:
                    if not any(x == s2 for x, _ in lab[:j]):
                        out.setdefault(sid, {})[s] = s2
                    break
    # Kept as it was when the corpus gave no renames at all or lost more than INCR_MAX_DROP percent (KEEP_LAST=0: written).
    if not common.write_keep_last(OUT, out, 'json', ensure_ascii=False, indent=1, sort_keys=True):
        sys.exit(3)  # a refused write of a computed output stops the update (a missing input keeps last and exits 0 earlier)
    pairs = sorted((lab, name) for m in out.values() for lab, name in m.items())
    print(f'label renames: {len(pairs)} in {len(out)} stories -> {os.path.relpath(OUT, ROOT)}: '
          + ', '.join(f'{a} -> {b}' for a, b in pairs))
    by_chunk = chunk_renames(chunks)
    if not common.write_keep_last(OUT_CHUNKS, by_chunk, 'json', ensure_ascii=False, indent=1, sort_keys=True):
        sys.exit(3)
    names = collections.Counter(m[UNKNOWN] for m in by_chunk.values())
    print(f'chunk renames of "???": {len(by_chunk)} chunks, {len(names)} names -> {os.path.relpath(OUT_CHUNKS, ROOT)}')


if __name__ == '__main__':
    main()
