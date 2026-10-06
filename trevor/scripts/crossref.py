#!/usr/bin/env python3
"""Cast-wide cross-references from game data, for `ask`'s cross_ref tool (2026-10-03, item 4 of Ian's list).

First table, "bosses_playable": every boss enemy of the enemy handbook (enemy_handbook_table EnemyLevel BOSS) whose
character is also a playable operator. A boss's character name is its handbook name without quotes, without a title
after the first comma ("Talulah, the Fighter" -> Talulah) and without a leading "The"; it matches an operator when it
equals the operator's codename (character_table, obtainable, not a token or trap), the operator's real name
(artifacts/entities/real_names.jsonl) or a name the identity links put with the operator (artifacts/entities/
identities.json; "W" ~ "Wiš'adel"). No per-name rules: every match comes from those three tables, and the output says
which one made it. No model.

  python3 scripts/crossref.py            -> artifacts/crossref/bosses_playable.json
"""
import json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
from common import EXCEL, ROOT

OUT = os.path.join(ROOT, 'artifacts', 'crossref')


def norm(s):
    s = s.replace('’', "'").replace('‘', "'").lower()
    return ' '.join(re.sub(r"[^a-z0-9' À-ɏ-]", ' ', s).split())


def core_name(enemy_name):
    n = enemy_name.replace('‘', "'").replace('’', "'")
    n = n.split(',')[0].strip().strip("'").strip()
    if n.lower().startswith('the '):
        n = n[4:]
    return n


def load(name):
    return json.load(open(os.path.join(EXCEL, name)))


def main():
    enemies = {x['key']: x['value'] for x in load('enemy_handbook_table.json')['EnemyData']}
    chars = {x['key']: x['value'] for x in load('character_table.json')['Characters']}
    ops = {k: v['Name'] for k, v in chars.items()
           if k.startswith('char_') and v.get('Profession') not in ('TOKEN', 'TRAP') and not v.get('IsNotObtainable')}
    by_name = {}
    for k, n in ops.items():
        by_name.setdefault(norm(n), []).append((k, n, 'codename'))
    rn_path = os.path.join(ROOT, 'artifacts', 'entities', 'real_names.jsonl')
    for line in open(rn_path):
        r = json.loads(line)
        if r.get('realName') and r.get('charId') in ops:
            by_name.setdefault(norm(r['realName']), []).append((r['charId'], ops[r['charId']], f"real name ({r['realName']}, {r.get('chunkId')})"))
    op_by_norm = {norm(n): k for k, n in ops.items()}
    # CROSSREF_IDENTITIES=v2 reads identities.v2.json (entities.py extract2 + ENTITY_V2=1 build, 2026-10-03); default v1.
    if os.environ.get('CROSSREF_IDENTITIES') == 'v2':
        idents = json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.v2.json')))['identities']
    else:
        idents = json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.json')))
    for ident in idents:
        linked = [op_by_norm[norm(n)] for n in ident['names'] if norm(n) in op_by_norm]
        for n in ident['names']:
            if norm(n) in op_by_norm:
                continue
            for k in linked:
                ev = next((e for e in ident['evidence'].get(n, []) if norm(e['other']) == norm(ops[k])), None) \
                    or (ident['evidence'].get(n) or [{}])[0]
                by_name.setdefault(norm(n), []).append((k, ops[k], f"identity link ({n} ~ {ops[k]}: \"{ev.get('quote', '')}\", {ev.get('where', '')})"))
    rows = {}
    for eid, e in enemies.items():
        if e.get('EnemyLevel') != 'BOSS':
            continue
        c = norm(core_name(e['Name']))
        for k, opname, how in by_name.get(c, []):
            key = (k, e['Name'])
            if key in rows:
                rows[key]['enemyIds'].append(eid)
                continue
            rows[key] = {'operator': opname, 'charId': k, 'boss': e['Name'], 'enemyIds': [eid], 'matchedBy': how,
                         'bossDescription': (e.get('Description') or '').strip()}
    out = sorted(rows.values(), key=lambda r: (r['operator'].lower(), r['boss']))
    os.makedirs(OUT, exist_ok=True)
    json.dump({'table': 'bosses_playable', 'bosses': sum(1 for e in enemies.values() if e.get('EnemyLevel') == 'BOSS'),
               'operators': len(ops), 'rows': out}, open(os.path.join(OUT, 'bosses_playable.json'), 'w'), ensure_ascii=False, indent=1)
    print(f"bosses_playable: {len(out)} rows, {len({r['charId'] for r in out})} operators, from "
          f"{sum(1 for e in enemies.values() if e.get('EnemyLevel') == 'BOSS')} boss entries and {len(ops)} operators")
    for r in out:
        print(f"  {r['operator']} <- {r['boss']} [{r['matchedBy'][:90]}]")


if __name__ == '__main__':
    main()
