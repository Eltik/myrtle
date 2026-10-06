#!/usr/bin/env python3
"""Appearance index (2026-10-03, Ian's item 7): per character, every story group where they speak and, apart from
that, every group where they are only mentioned, for `ask`'s `appearances` tool ("What chapters does Elysium appear
in?").

From data only, no model, no per-name rules:
- story groups are the corpus groups that are stories (main episodes, events, operator records, IS and gametext
  scripts are left out: their units carry no speaker labels), from artifacts/p4x/chunks.jsonl;
- a character SPEAKS in a group when a chunk of it lists them in its `speakers` field (the "Name:" labels of the
  script); the speaker labels of one identity (artifacts/entities/identities.json, links of kind real_name, codename,
  former_name, alias or stage_name, never titles: "King of Sarkaz" would make every Theresa line Amiya's) count as
  that character;
- a character is MENTIONED in a group when no chunk of it lists them as a speaker but a chunk names them as a whole
  word (case-sensitive, so "Elysium" the operator and not "elysium" the word), outside the speaker labels;
- groups are named and ordered by the reading guide (artifacts/chrono/reading_guide.json, EN release order) and
  artifacts/goldgen/names.json; an operator record gets its owner from the operator file of the same short id;
- other operators' files, voice lines, modules and outfits that name the character are listed apart ("named in other
  sources").
A name of fewer than 3 characters is never matched as a mention (speaking still counts).

  python3 scripts/appearances.py      -> artifacts/appearances/appearances.json
"""
import json, os, re, collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'appearances')
NOT_STORY = {'archive', 'profile', 'summary', 'topic', 'module', 'voice', 'skin', 'is', 'enemy', 'item', 'gametext', 'art'}
LINKS = {'real_name', 'codename', 'former_name', 'alias', 'stage_name'}


def main():
    names = json.load(open(os.path.join(ROOT, 'artifacts/goldgen/names.json')))
    guide = json.load(open(os.path.join(ROOT, 'artifacts/chrono/reading_guide.json')))['items']
    pos = {g['groupId']: (i, g) for i, g in enumerate(guide)}
    ident = json.load(open(os.path.join(ROOT, 'artifacts/entities/identities.json')))
    alias = {}
    for i in ident:
        for n, ev in i['evidence'].items():
            for e in ev:
                if e['relation'] in LINKS:
                    alias.setdefault(n, set()).add(i['label'])
    # A speaker label maps to its identity's label only when the link is unambiguous.
    canon = {n: next(iter(ls)) for n, ls in alias.items() if len(ls) == 1}
    owner = {}
    chunks = []
    for l in open(os.path.join(ROOT, 'artifacts/p4x/chunks.jsonl')):
        c = json.loads(l)
        if c['groupId'] == 'archive' and c['ordinal'] == 0:
            m = re.match(r'archive_char_\d+_(\w+)$', c['storyId'])
            head = c['text'].split('\n', 1)[0].split(': ', 1)[-1]
            if m:
                owner[m.group(1)] = head
        chunks.append(c)
    speaks = collections.defaultdict(collections.Counter)
    text_by_group = collections.defaultdict(list)
    other = collections.defaultdict(list)
    for c in chunks:
        g = c['groupId']
        if g in NOT_STORY:
            continue
        for s in c.get('speakers') or []:
            speaks[canon.get(s, s)][g] += 1
        text_by_group[g].append(c['text'])
    characters = set(speaks)
    # Mentions: one regex per character over the text with speaker labels removed.
    label_re = re.compile(r'(?m)^[^:\n]{1,40}:')
    stripped = {g: [label_re.sub('', t) for t in ts] for g, ts in text_by_group.items()}
    others = [c for c in chunks if c['groupId'] in ('archive', 'voice', 'module', 'skin')]
    # One pass: a single alternation of every form (longest first) over each text, each match mapped to its character.
    form_of = {}
    for ch in characters:
        for f in {ch} | {n for n, lab in canon.items() if lab == ch}:
            if len(f) >= 3:
                form_of.setdefault(f, ch)
    rx = re.compile(r"(?<![\w'])(" + '|'.join(re.escape(f) for f in sorted(form_of, key=len, reverse=True)) + r")(?![\w])")
    ment = collections.defaultdict(collections.Counter)
    for g, ts in stripped.items():
        for t in ts:
            for ch in {form_of[m.group(1)] for m in rx.finditer(t)}:
                if g not in speaks[ch]:
                    ment[ch][g] += 1
    named = collections.defaultdict(list)
    for c in others:
        for ch in {form_of[m.group(1)] for m in rx.finditer(label_re.sub('', c['text']))}:
            if c['storyId'] not in named[ch] and len(named[ch]) < 40:
                named[ch].append(c['storyId'])
    out = {}
    for ch in sorted(characters):
        forms = sorted(f for f, c in form_of.items() if c == ch)
        out[ch] = {'forms': forms, 'speaks': dict(speaks[ch]), 'mentioned': dict(ment[ch]), 'namedIn': named[ch]}

    def group_row(g):
        i, item = pos.get(g, (None, None))
        sid = next((s for s in names if s.startswith(g + '_') or s == g), None)
        gname = item['name'] if item else (names[sid]['group'] if sid else g)
        kind = item['kind'] if item else ('record' if g.startswith('story_') else 'event')
        if g.startswith('story_'):
            m = re.match(r'story_(\w+?)_set_\d+$', g)
            o = owner.get(m.group(1)) if m else None
            kind = 'operator record' + (f' of {o}' if o else '')
        if g.startswith('main_') and item is None and sid:
            kind = 'main'
        return {'groupId': g, 'name': gname, 'kind': kind, 'episode': item.get('episode') if item else None,
                'order': i if i is not None else 10_000}

    groups = {g: group_row(g) for g in text_by_group}
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, 'appearances.json'), 'w') as f:
        json.dump({'groups': groups, 'characters': out}, f, ensure_ascii=False, sort_keys=True)
    print(f'{len(out)} characters, {len(groups)} story groups -> artifacts/appearances/appearances.json')
    e = out.get('Elysium')
    if e:
        for k in ('speaks', 'mentioned'):
            print(k, [(groups[g]['name'], n) for g, n in sorted(e[k].items(), key=lambda x: groups[x[0]]['order'])])
        print('namedIn', e['namedIn'])


if __name__ == '__main__':
    main()
