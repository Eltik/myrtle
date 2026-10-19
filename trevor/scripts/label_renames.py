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

  python3 scripts/label_renames.py   -> artifacts/entities/label_renames.json  {storyId: {label: name}}
"""
import collections, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT, read_derived

CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3b'))
OUT = os.path.join(ROOT, 'artifacts', 'entities', 'label_renames.json')
INTRO = re.compile(r"\b(?:I'm called|I am called|my name is|My name is|call me|Call me|I'm|I am)\s+"
                   r"([A-Z][\w'\-]+(?:\s[A-Z][\w'\-]+)?)")
DESCRIPTION = re.compile(r'\b(Girl|Boy|Man|Woman|Youth|Kid|Child|Soldier|Guard|Voice|Stranger|Lady|Gentleman|Old|Young|'
                         r'Mysterious|Hooded)\b')
LABEL = re.compile(r'^([^:\n]{1,40}):\s*(.*)$')


def main():
    stories = collections.defaultdict(list)
    for c in read_derived(os.path.join(CORPUS, 'chunks.jsonl')):
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
        return
    pairs = sorted((lab, name) for m in out.values() for lab, name in m.items())
    print(f'label renames: {len(pairs)} in {len(out)} stories -> {os.path.relpath(OUT, ROOT)}: '
          + ', '.join(f'{a} -> {b}' for a, b in pairs))


if __name__ == '__main__':
    main()
