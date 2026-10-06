#!/usr/bin/env python3
"""Operator attributes from the game data, for whole-roster questions ("Which operators are Sarkaz?").

No model: character_table (rarity, class, branch, nation, group, team) and each operator's handbook
"Basic Info" (gender, place and date of birth, race, height, infection status, combat experience).
Ian (2026-09-28): "List all the operators from Kazimierz" got 4 names from 19 retrieved passages; this
table answers such questions completely. "From X" is ambiguous in the data: `nation` is the game's
affiliation field (character_table NationId, 383 of 407 set), `birthplace` is the handbook's Place of
Birth; `ask` reports both. Values are the game's own text; "Undisclosed" and "Unknown" stay as written.
Writes artifacts/entities/operator_attributes.jsonl. Deterministic, seconds; rerun after any update.
"""
import collections, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import EXCEL as GD, ROOT

OUT = os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl')
CLASS = {'PIONEER': 'Vanguard', 'WARRIOR': 'Guard', 'TANK': 'Defender', 'SNIPER': 'Sniper', 'CASTER': 'Caster',
         'MEDIC': 'Medic', 'SUPPORT': 'Supporter', 'SPECIAL': 'Specialist'}


def load(name):
    return json.load(open(os.path.join(GD, name)))


def kv(rows):
    return {r['key']: r['value'] for r in rows} if isinstance(rows, list) else rows


def infected(text):
    t = text.lower()
    # "Medical tests have confirmed that no infection is present" vs "... confirmed as infected".
    if re.search(r"\bno (?:infection|signs? of infection)|not infected|uninfected|non-infected|no originium", t):
        return False
    if re.search(r"\binfected\b|infection (?:is |was )?(?:confirmed|present)|confirmed (?:as )?infect|oripathy|"
                 r"crystals? (?:have |has )?(?:appeared|visible|formed)", t):
        return True
    return None


def main():
    chars = kv(load('character_table.json')['Characters'])
    teams = {k: v['PowerName'] for k, v in kv(load('handbook_team_table.json')['Handbook_teams']).items()}
    branches = {k: v['SubProfessionName'] for k, v in kv(load('uniequip_table.json')['SubProfDict']).items()}
    hb = {x['value']['CharID']: x['value'] for x in load('handbook_info_table.json')['HandbookDict']}
    rows = []
    for cid, c in chars.items():
        if c.get('Profession') in ('TOKEN', 'TRAP') or c.get('IsNotObtainable'):
            continue
        info = {}
        e = hb.get(cid)
        if e and e['StoryTextAudio']:
            text = e['StoryTextAudio'][0]['Stories'][0]['StoryText']
            # "[Field] value" per line; a field whose value is on the following lines (Infection Status) takes them.
            for m in re.finditer(r'^\[([^\]]+)\]\s*(.*?)(?=^\[|\Z)', text, re.M | re.S):
                info[m.group(1).strip()] = ' '.join(m.group(2).split())
        height = re.search(r'(\d{2,3})\s*cm', info.get('Height', ''))
        inf = info.get('Infection Status', '')
        rows.append({
            'name': c['Name'], 'charId': cid,
            'rarity': int(re.sub(r'\D', '', c.get('Rarity') or '0') or 0),
            'class': CLASS.get(c.get('Profession'), c.get('Profession')),
            'branch': branches.get(c.get('SubProfessionId')),
            'nation': teams.get(c.get('NationId')) if c.get('NationId') else None,
            'group': teams.get(c.get('GroupId')) if c.get('GroupId') else None,
            'team': teams.get(c.get('TeamId')) if c.get('TeamId') else None,
            'gender': (info.get('Gender') or '').strip(' ]') or None,
            'birthplace': info.get('Place of Birth') or None,
            'birthday': info.get('Date of Birth') or None,
            'race': info.get('Race') or None,
            'heightCm': int(height.group(1)) if height else None,
            'infected': infected(inf) if inf else None,
            'infectionText': inf or None,
            'combatExperience': info.get('Combat Experience') or None,
        })
    rows.sort(key=lambda r: r['name'])
    common.write_jsonl(OUT, rows)
    cov = {k: sum(r[k] is not None for r in rows) for k in ('nation', 'group', 'team', 'gender', 'birthplace', 'race',
                                                              'heightCm', 'infected', 'birthday')}
    print(f"attributes: {len(rows)} operators; set per field {cov}; "
          f"infected {collections.Counter(r['infected'] for r in rows)}")


if __name__ == '__main__':
    main()
