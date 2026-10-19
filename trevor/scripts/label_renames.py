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

More chunk rules (2026-10-20), after the introduction rule in this order, a chunk keeping the first rule's answer, each
needing the chunk's "???" lines within 4 lines of each other (one conversation, `one_stretch`). Census of the 2,814
"???" runs: 1,572 an exchange (the same speaker before and after), 628 followed by a label the story already has, 333
by a label new to the story with no "???" after it, 98 by a new label with a later "???", 183 by no label. Precision
read against the text (R of R+W):
  self       the "???" line introduces itself and that label speaks next ("???: My name is Red." "Red: ...")
             7 chunks; all 7 read, 7 right
  called     the line before the run mentions the next label, new to the story ("Guard: Tonya Alliluyeva!")
             9 chunks; all 10 matches read, 9 right
  addressed  the next speaker opens by calling a name and that label speaks next ("Lavinia: Eirene?" "Eirene: ...")
             61 chunks; 23 of 25 on a fresh sample before the guard against a label that spoke during the "???"
             lines, 48 right and 0 wrong over three samples after it
  reveal     the label after the last "???" line speaks for the first time in the story (five guards, see the
             function): 113 chunks more, 20 of 24 on a fresh sample (83%): under the 90% bar, so OPT-IN only.
On P3b the default writes 95 chunks (81 names): intro 18, self 7, called 9, addressed 61.
LABEL_RENAMES_CHUNK_RULES=intro writes the 18-chunk file of 2026-10-19 byte for byte (with KEEP_LAST=0 when it
replaces a larger file: the shrink guard keeps the larger one and exits 3); `intro,self,called,addressed,reveal`
adds the opt-in rule. The story file is the same under every setting.

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
    every "???" line of the chunk comes before it (so a chunk never gives two names). The introduction rule."""
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        found = sorted((j, n) for i in unknown if (f := introduced(lab, i, {UNKNOWN})) for n, j in [f])
        if found and unknown[-1] < found[0][0]:
            out[c['chunkId']] = {UNKNOWN: found[0][1]}
    return out


STRETCH = 4  # the most lines between two "???" lines of a chunk a new rule names together


def one_stretch(unknown):
    """Whether a chunk's "???" line indices form one conversation, no two more than STRETCH lines apart: a chunk gets
    one name for all its "???" lines, and a longer silence can hold a second stranger ("act33side_st03#0002": child
    Amiya, then Oracle eight lines on)."""
    return all(b - a <= STRETCH for a, b in zip(unknown, unknown[1:]))


def self_intro_renames(chunks):
    """{chunkId: {"???": name}}: the self-introduction rule (2026-10-20). A "???" line that introduces itself by
    name ("???: Hello. My name is Red.") names the chunk's "???" lines when the next label within 3 lines is that
    name on its first line of the chunk ("Red: Back in my Ursus days..."), and no "???" line comes after it. The
    introduction rule's mirror: there the named label introduces itself, here the "???" line does."""
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        if not one_stretch(unknown):
            continue
        for i in unknown:
            m = INTRO.search(lab[i][1])
            j = next((j for j in range(i + 1, min(i + 4, len(lab))) if lab[j][0] and lab[j][0] != UNKNOWN), None)
            if not m or j is None:
                continue
            s = lab[j][0]
            if s.split()[0] == m.group(1).split()[0] and not any(x == s for x, _ in lab[:j]) and unknown[-1] < j:
                out[c['chunkId']] = {UNKNOWN: s}
                break
    return out


def name_words(name):
    """The capitalized first and last words of a label, 3 characters or more, as a script calls the speaker."""
    return [w for w in {name.split()[0], name.split()[-1]} if len(w) >= 3 and w[0].isupper()]


def answered_call_renames(chunks):
    """{chunkId: {"???": name}}: the answered-call rule (2026-10-20). A speaker calls a name, a "???" answers, and
    the label with that name speaks next for the first time in the story ("Guard: Alliluyeva!" "???: Oh, yes."
    "Tonya: I'm here..."). Kept when the labelled line before the chunk's last "???" run mentions the first or last
    word of the next label, that label speaks at most one narration line after the run, it is new to the story, and
    no "???" line comes after it."""
    before = earlier_labels(chunks)
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        if not unknown or not one_stretch(unknown):
            continue
        last = unknown[-1]
        nxt = next((j for j in range(last + 1, len(lab)) if lab[j][0]), None)
        if nxt is None or nxt - last > REVEAL_GAP:
            continue
        st = last
        while st > 0 and lab[st - 1][0] in (UNKNOWN, None):
            st -= 1
        if st == 0:
            continue
        name = lab[nxt][0]
        seen = before.get(c['chunkId'], frozenset()) | {s for s, _ in lab[:nxt] if s}
        if name in seen or '?' in name:
            continue
        if any(re.search(r'(?<!\w)' + re.escape(w) + r'(?!\w)', lab[st - 1][1]) for w in name_words(name)):
            out[c['chunkId']] = {UNKNOWN: name}
    return out


OPENING_NAME = re.compile(r"[\s.\u2026\u2014'\"(]*([A-Z][\w'\-]+)[,.!?\u2026]")


def addressed_renames(chunks):
    """{chunkId: {"???": name}}: the addressed rule (2026-10-20). The speaker after the chunk's last "???" line opens
    by calling the stranger by name and the label with that name speaks next, so the "???" lines were theirs
    ("???: Stay here, Lavinia." "Lavinia: Eirene?" "Eirene: It's all wise guys."). Kept when the named label speaks
    at most one narration line after the one who calls it, the called word is its first or last word, it has not
    spoken since the chunk's first "???" line, and no "???" line comes after."""
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        if not unknown or not one_stretch(unknown):
            continue
        last = unknown[-1]
        nxt = next((j for j in range(last + 1, len(lab)) if lab[j][0]), None)
        if nxt is None or nxt - last > REVEAL_GAP:
            continue
        m = OPENING_NAME.match(lab[nxt][1])
        then = next((j for j in range(nxt + 1, len(lab)) if lab[j][0]), None)
        if not m or then is None or then - nxt > REVEAL_GAP:
            continue
        name = lab[then][0]
        if name in (UNKNOWN, lab[nxt][0]) or '?' in name or m.group(1) not in name_words(name):
            continue
        if any(s == name for s, _ in lab[unknown[0]:nxt]):
            continue  # the called label spoke while "???" did: someone else ("Rosmontis: How... you blocked my sword")
        out[c['chunkId']] = {UNKNOWN: name}
    return out


WHO = re.compile(r"\b[Ww]ho(?:'re| are| is|'s)\b")
REVEAL_GAP = 2  # the named label's line at most this many lines after the last "???" line (one narration line between)
REVEAL_AFTER = 4  # lines after the named label's first line in which no other new label may appear


def addresses(line, labels):
    """Whether `line` opens by calling a known label by name ("Ascalon... Hah, should have figured")."""
    m = re.match(r"[\s.\u2026\u2014'\"]*([A-Z][\w'\-]+)[,.!?\u2026]", line)
    return bool(m) and any(m.group(1) in (x.split()[0], x.split()[-1]) for x in labels if x.split())


def earlier_labels(chunks):
    """{chunkId: labels of the story's earlier chunks (by ordinal)}, for "first appearance in the story"."""
    stories = collections.defaultdict(list)
    for c in chunks:
        stories[c['storyId']].append(c)
    out = {}
    for cs in stories.values():
        seen = set()
        for c in sorted(cs, key=lambda c: c.get('ordinal', 0)):
            out[c['chunkId']] = frozenset(seen)
            seen |= {s for s, _ in labelled(c['text']) if s}
    return out


def reveal_renames(chunks):
    """{chunkId: {"???": name}}: the reveal rule (2026-10-20). A script labels a stranger "???" until the story names
    them, then continues under the name, so the label that speaks right after a chunk's last "???" line (at most one
    narration line between) names the chunk's "???" lines when that is its first line in the story. Five guards
    drop the shapes where the new label is the stranger's interlocutor: its first line asks who the stranger is
    ("Who're you?!"), another label new to the story speaks within the 4 lines after it (the stranger named a line
    later, "Guard Operator: I'm here, Dr. Warfarin." then "Warfarin: ..."), the last "???" line ends on a question
    (the new label answers it), the new label's first line opens by calling a label the story already has by name
    ("Table Salt: Ascalon... Hah"), or the label itself has a "?" in it."""
    before = earlier_labels(chunks)
    out = {}
    for c in chunks:
        lab = labelled(c['text'])
        unknown = [i for i, (s, _) in enumerate(lab) if s == UNKNOWN]
        if not unknown or not one_stretch(unknown):
            continue
        last = unknown[-1]
        nxt = next((j for j in range(last + 1, len(lab)) if lab[j][0]), None)
        if nxt is None or nxt - last > REVEAL_GAP:
            continue
        name, line = lab[nxt]
        seen = before.get(c['chunkId'], frozenset()) | {s for s, _ in lab[:nxt] if s}
        if name in seen or '?' in name or WHO.search(line):
            continue
        if any(s and s != name and s != UNKNOWN and s not in seen for s, _ in lab[nxt + 1:nxt + 1 + REVEAL_AFTER]):
            continue
        if '?' in lab[last][1].rstrip()[-3:] or addresses(line, seen - {name, UNKNOWN}):
            continue
        out[c['chunkId']] = {UNKNOWN: name}
    return out


# The chunk rules in order; a chunk keeps the first rule's answer. LABEL_RENAMES_CHUNK_RULES (comma-separated names)
# picks them: unset is DEFAULT_RULES, `intro` is the kill switch for the rules after the first (the file is the
# introduction rule's alone, byte for byte as on 2026-10-19), and `reveal` is opt-in: 127 more chunks at 20 of 24
# right on a fresh reading, under the 90% bar the other rules meet.
CHUNK_RULES = (('intro', chunk_renames), ('self', self_intro_renames), ('called', answered_call_renames),
               ('addressed', addressed_renames), ('reveal', reveal_renames))
DEFAULT_RULES = ('intro', 'self', 'called', 'addressed')


def all_chunk_renames(chunks, rules=None):
    """({chunkId: {"???": name}}, {rule: chunks it named}) over the rules named in `rules` (default: the
    LABEL_RENAMES_CHUNK_RULES list, comma-separated; unset: DEFAULT_RULES)."""
    if rules is None:
        env = os.environ.get('LABEL_RENAMES_CHUNK_RULES')
        rules = [r.strip() for r in env.split(',')] if env else DEFAULT_RULES
    out, counts = {}, {}
    for name, rule in CHUNK_RULES:
        if name not in rules:
            continue
        got = {k: v for k, v in rule(chunks).items() if k not in out}
        counts[name] = len(got)
        out.update(got)
    return out, counts


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
    by_chunk, per_rule = all_chunk_renames(chunks)
    if not common.write_keep_last(OUT_CHUNKS, by_chunk, 'json', ensure_ascii=False, indent=1, sort_keys=True):
        sys.exit(3)
    names = collections.Counter(m[UNKNOWN] for m in by_chunk.values())
    print(f'chunk renames of "???": {len(by_chunk)} chunks, {len(names)} names -> {os.path.relpath(OUT_CHUNKS, ROOT)} ('
          + ', '.join(f'{k} {v}' for k, v in per_rule.items()) + ')')


if __name__ == '__main__':
    main()
