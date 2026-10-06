#!/usr/bin/env python3
"""Typed source units from the game data that the story corpus lacks, for source-specific questions.

Ian (2026-09-28): source-specific questions ("what does Kal'tsit's module say", "what does Exusiai say when poked",
IS endings) were answered 0.14 of the time, and the first probe answered a module question from story passages as
if they were the module. These units carry their source in `groupId` and in a heading line, so retrieval can find
them and `ask` can say where a passage comes from:
  module  module stories (uniequip_table UniEquipDesc), one unit per module
  voice   voice lines (charword_table, each operator's default voice set), one unit per operator
  is      Integrated Strategies (roguelike_topic_table): per run, its encounters, endings, relics with flavor
          text, stages and zones
  enemy   enemy entries (enemy_handbook_table), one unit per enemy
  skin    outfit descriptions (skin_table), one unit per outfit with text
  item    item flavor text (item_table), one unit per item with a description of 8 words or more
  is      also the IS cutscene scripts (gamedata/story/obt/roguelike ending scenes and obt/rogue monthly squad
          stories), which the story corpus build never fetched
Deterministic, seconds. Writes artifacts/sources/units.jsonl ({storyId, groupId, title, text, speakers});
`target/release/build-units` appends them to a corpus copy.
"""
import collections, glob, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import EXCEL as GD, ROOT, kv, story_script as script

OUT = os.path.join(ROOT, 'artifacts', 'sources', 'units.jsonl')
TAG = re.compile(r'<[@$/][^>]*>|</>|<[^>]{1,40}>')


# The game's numbering of Integrated Strategies runs is one ahead of the data's run ids: IS #1 was Ceobe's Fungimist
# (CN only, in roguelike_table.json, untranslated, never in roguelike_topic_table), so rogue_1 Phantom & Crimson
# Solitaire is IS #2 and rogue_5 Sui's Garden of Grotesqueries is IS #6 (Ian, 2026-09-30). IS_OFFSET=0 restores
# the old titles, which numbered them 1 to 5.
IS_OFFSET = int(os.environ.get('IS_OFFSET', '1'))


def is_no(n):
    return str(int(n) + IS_OFFSET) if str(n).isdigit() else n


def load(name):
    return json.load(open(os.path.join(GD, name)))


def clean(s):
    # Rich-text tags ("<@ba.kw>", "<$ba.camp>", "</>") and the player's nickname placeholder.
    s = TAG.sub('', s or '').replace('{@nickname}', 'Doctor').replace('\\n', '\n')
    return '\n'.join(' '.join(l.split()) for l in s.split('\n') if l.strip())


def words(s):
    return len((s or '').split())


def add_modules(add, cname):
    for eid, e in kv(load('uniequip_table.json')['EquipDict']).items():
        cid = e.get('CharId')
        if e.get('Type_') == 'INITIAL' or not cid:
            continue  # the ORIGINAL module of every operator carries one generic line
        op = cname.get(cid, cid)
        add(f'module_{eid}', 'module', f"Module story: {e.get('UniEquipName')} ({op}'s module, {e.get('TypeName1', '')}-{e.get('TypeName2', '')})",
            e.get('UniEquipDesc'), [op])


def add_voices(add, cname):
    lines = collections.defaultdict(list)
    for w in kv(load('charword_table.json')['CharWords']).values():
        # The default voice set only: a skin's alternate set repeats the same titles.
        if w.get('WordKey') == w.get('CharId') and w.get('VoiceText'):
            lines[w['CharId']].append((int(w.get('VoiceIndex') or 0), w.get('VoiceTitle'), w['VoiceText']))
    for cid, ls in lines.items():
        op = cname.get(cid, cid)
        add(f'voice_{cid}', 'voice', f'Voice lines: {op}', '\n'.join(f'{t}: {x}' for _, t, x in sorted(ls)), [op])


def add_is_runs(add, rl, topics):
    """Per Integrated Strategies run: its encounters, endings, relics with flavor text, stages and floors."""
    for d in rl['Details']:
        rid, v = d['key'], d['value']
        name = f"Integrated Strategies {is_no(rid.split('_')[-1])}, {topics.get(rid, rid)}"
        scenes = kv(v.get('ChoiceScenes', []))
        add(f'is_{rid}_encounters', 'is', f'{name}: encounters',
            '\n'.join(f"{s.get('Title', '')}: {s.get('Description', '')}" for s in scenes.values() if words(s.get('Description')) >= 6))
        add(f'is_{rid}_endings', 'is', f'{name}: endings',
            '\n'.join(f"{e.get('Name', '')}: {e.get('Desc', '')}" for e in kv(v.get('Endings', [])).values()))
        items = kv(v.get('Items', []))
        add(f'is_{rid}_relics', 'is', f'{name}: collectibles and their descriptions',
            '\n'.join(f"{i.get('Name', '')}: {i.get('Description', '')}" for i in items.values() if words(i.get('Description')) >= 8))
        add(f'is_{rid}_stages', 'is', f'{name}: operations',
            '\n'.join(f"{s.get('Name', '')} ({s.get('Code', '')}): {TAG.sub('', s.get('Description') or '').split(chr(10))[0]}"
                      for s in kv(v.get('Stages', [])).values() if words(s.get('Description')) >= 6))
        add(f'is_{rid}_zones', 'is', f'{name}: floors',
            '\n'.join(f"{z.get('Name', '')}: {z.get('Description', '')} {z.get('EndingDescription', '') or ''}"
                      for z in kv(v.get('Zones', [])).values() if words(z.get('Description')) >= 6))


def add_is_scripts(add, rl, topics):
    """The IS cutscenes themselves (ending scenes, monthly squad stories): story text the corpus build never fetched."""
    story = os.path.join(GD, '..', 'story', 'obt')
    for path in sorted(glob.glob(os.path.join(story, 'roguelike', 'ro*', 'level_rogue*_ending_*.txt.txt'))):
        m = re.search(r'rogue(\d+)_ending_(\d+)', path)
        run = f'rogue_{m.group(1)}'
        end = next((e for d in rl['Details'] if d['key'] == run for e in kv(d['value']['Endings']).values()
                    if e['Id'].endswith(f'ending_{m.group(2)}')), {})
        add(f'is_{run}_ending_{m.group(2)}_script', 'is',
            f"Integrated Strategies {is_no(m.group(1))}, {topics.get(run, run)}: ending scene '{end.get('Name', m.group(2))}'", script(path))
    for path in sorted(glob.glob(os.path.join(story, 'rogue', '**', '*.txt.txt'), recursive=True)):
        base = os.path.basename(path).replace('.txt.txt', '')
        m = re.search(r'rogue_(\d+)', base)
        run = f'rogue_{m.group(1)}' if m else 'rogue'
        add(f'is_{base}', 'is', f"Integrated Strategies {is_no(m.group(1)) if m else ''}, {topics.get(run, run)}: monthly squad story {base}", script(path))


def add_enemies(add):
    for eid, e in kv(load('enemy_handbook_table.json')['EnemyData']).items():
        if e.get('HideInHandbook') is True:
            continue
        add(f'enemy_{eid}', 'enemy', f"Enemy entry: {e.get('Name')}", e.get('Description'))


def add_outfits(add, cname):
    for sid, s in kv(load('skin_table.json')['CharSkins']).items():
        d = s.get('DisplaySkin') or {}
        body = '\n'.join(x for x in (d.get('Content'), d.get('Dialog'), d.get('Usage'), d.get('Description')) if x)
        op = cname.get(s.get('CharId'), s.get('CharId'))
        title = f"Outfit: {d.get('SkinName') or d.get('SkinGroupName') or sid} ({op})"
        if d.get('SkinGroupName') != 'Default Outfit':
            add(f'skin_{sid}', 'skin', title, body, [op] if op else [])


def add_items(add):
    for iid, i in kv(load('item_table.json')['Items']).items():
        if words(i.get('Description')) >= 8:
            add(f'item_{iid}', 'item', f"Item: {i.get('Name')}", f"{i.get('Description')}\n{i.get('Usage') or ''}")


def main():
    chars = kv(load('character_table.json')['Characters'])
    cname = {k: v.get('Name') for k, v in chars.items()}
    units = []

    def add(sid, group, title, body, speakers=()):
        body = clean(body)
        if words(body) >= 6:
            units.append({'storyId': sid, 'groupId': group, 'title': title, 'text': f'{title}\n{body}', 'speakers': list(speakers)})

    add_modules(add, cname)
    add_voices(add, cname)
    rl = load('roguelike_topic_table.json')
    topics = {t['key']: t['value'].get('Name') for t in rl['Topics']}
    add_is_runs(add, rl, topics)
    add_is_scripts(add, rl, topics)
    add_enemies(add)
    add_outfits(add, cname)
    add_items(add)

    # Stable ids: a source that appears twice (a skin in two groups) keeps its first unit.
    seen, out = set(), []
    for u in units:
        u['storyId'] = re.sub(r'[^A-Za-z0-9_#.-]', '_', u['storyId'])
        if u['storyId'] not in seen:
            seen.add(u['storyId']); out.append(u)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    common.write_jsonl(OUT, out)
    c = collections.Counter(u['groupId'] for u in out)
    w = collections.Counter()
    for u in out:
        w[u['groupId']] += words(u['text'])
    print(f"sources: {len(out)} units {dict(c)}; words {dict(w)}")


if __name__ == '__main__':
    main()
