#!/usr/bin/env python3
"""Reading guide: the EN release order of the main story and events, what each event builds on, and where
each character first appears.

Ian (2026-09-28): reading-order questions are 5 to 9% of real questions ("where do I start", "when should I
read Babel") and `ask` answered 0.29 of them. Since 2026-10-04 (Ian: the wiki is a reference, never a source)
the EN release dates of the main episodes come from the game data alone:
  dates   (default source, no network) -> artifacts/chrono/main_release.json: an episode's zone open time
          (zone_table MainlineAdditionInfo, Episodes 10 to 14) or its home-screen theme start (activity_table
          ActThemes, MAINLINE main_9, and the ACTIVITY_COMP theme of the activity whose zone holds main_15 or
          main_16 stages) is its date; else, when its Adverse (hard_NN) stages carry a schedule (stage_table
          StageValidInfo), the date is their opening minus the offset measured on the episodes that have both
          (7.25 to 7.46 days on Episodes 10 to 14), labelled an estimate; an episode with neither, before the first
          dated one and without Adverse stages (older than the Adverse system), shipped with the game: on or
          before the game's first event (activity_table BasicInfo, the earliest StartTime); any other undated
          episode (Episode 5: Adverse stages without a schedule) is placed right after the episode before it,
          date unknown.
          MAIN_DATES=wiki copies the wiki reference (eval/reference/main_release.wiki.json) instead, the file
          before 2026-10-04, for measurement only.
  wiki    eval reference only (REFRESH_REFERENCE=1 or by hand; network): the wiki's Main Theme act pages ->
          eval/reference/main_release.wiki.json, never read by build unless MAIN_DATES=wiki
  score   no network: the game-data dates against the wiki reference (exact, within 1 day, order agreement)
  storylines  the game's own reading links (stage_table.json Storylines, the Terminal "Movements" view): 14
          storylines with their story sets in order, and every BEFORE/AFTER location turned into a "read A before
          B" edge -> artifacts/chrono/storylines.json; compared against the community's placement rules
          (data/reading_community.json), which become a check instead of the source
  build   -> artifacts/chrono/reading_guide.json (main episodes and events in EN release order; per event,
          the earlier groups it builds on, from its primer, and the later ones that build on it; an
          event's weight as a prerequisite) and artifacts/chrono/first_appearance.json (per speaker, the
          earliest-released group they speak in)
"""
import collections, datetime, json, os, re, shutil, statistics, sys, time, urllib.parse, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CH = os.path.join(ROOT, 'artifacts', 'chrono')
GD = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel')
WIKI_REF = os.path.join(ROOT, 'eval', 'reference', 'main_release.wiki.json')
API = 'https://arknights.wiki.gg/api.php'
ACTS = ['Operation/Main_Theme/Act_initium', 'Operation/Main_Theme/Act_I', 'Operation/Main_Theme/Act_II',
        'Operation/Main_Theme/Act_III']
EN_LAUNCH = '2020-01-16'  # Arknights global launch; the Main Theme page: "Prologue to Episode 04 were available upon the initial release"


def wikitext(page):
    q = urllib.parse.urlencode({'action': 'parse', 'page': page, 'prop': 'wikitext', 'format': 'json', 'redirects': 1})
    with urllib.request.urlopen(urllib.request.Request(f'{API}?{q}', headers={'User-Agent': 'trevor-research/0.1'}), timeout=60) as r:
        return json.load(r)['parse']['wikitext']['*']


def stage_wiki():
    eps = []
    for page in ACTS:
        w = wikitext(page)
        for cell in w.split('{{Main Theme operations cell')[1:]:
            # [ \t]* not \s*: an empty "|global =" line otherwise took the next line ("|tw = ...") as its value.
            f = dict(re.findall(r'^\|(\w+)[ \t]*=[ \t]*(.*)$', cell, re.M))
            if 'episode' not in f:
                continue
            g = f.get('global', '').strip().replace('/', '-')
            g = g if re.fullmatch(r'\d{4}-\d{2}-\d{2}', g) else ''
            eps.append({'episode': int(f['episode']), 'title': f.get('title', '').strip(),
                        'global': g or (EN_LAUNCH if int(f['episode']) <= 4 else None),
                        'basis': 'wiki global date' if g else ('EN launch' if int(f['episode']) <= 4 else 'unknown'),
                        'source': f'https://arknights.wiki.gg/wiki/{page}', 'fetched': time.strftime('%Y-%m-%d')})
        time.sleep(1)
    eps.sort(key=lambda e: e['episode'])
    os.makedirs(os.path.dirname(WIKI_REF), exist_ok=True)
    json.dump(eps, open(WIKI_REF, 'w'), ensure_ascii=False, indent=1)
    print(f"wiki: {len(eps)} episodes; dated {sum(bool(e['global']) for e in eps)} -> {WIKI_REF} (eval reference only)")


def kv(x):
    return {r['key']: r['value'] for r in x} if isinstance(x, list) else x


def stage_dates():
    """EN release dates of the main episodes from the game data alone (see the module docstring)."""
    if os.environ.get('MAIN_DATES') == 'wiki':
        shutil.copyfile(WIKI_REF, os.path.join(CH, 'main_release.json'))
        print(f'dates: MAIN_DATES=wiki, copied {WIKI_REF} (measurement only)')
        return
    zt = json.load(open(os.path.join(GD, 'zone_table.json')))
    st = json.load(open(os.path.join(GD, 'stage_table.json')))
    act = json.load(open(os.path.join(GD, 'activity_table.json')))
    zones, stages = kv(zt['Zones']), kv(st['Stages'])
    opened = {k: v.get('ZoneOpenTime', -1) for k, v in kv(zt['MainlineAdditionInfo']).items()}
    # Episode number per zone: main_N zones, and the zones holding main_N-NN stages (main_15 in act2mainss_zone1).
    zone_ep = {}
    for sid, sv in stages.items():
        m = re.match(r'main_(\d+)-\d+$', sid)
        if m and sv.get('ZoneId'):
            zone_ep.setdefault(sv['ZoneId'], int(m.group(1)))
    for z in zones:
        m = re.fullmatch(r'main_(\d+)', z)
        if m:
            zone_ep.setdefault(z, int(m.group(1)))
    themes = {t['FuncId']: t for t in act['ActThemes']}
    exact = {}
    for z, ep in zone_ep.items():
        if opened.get(z, -1) > 0:
            exact[ep] = (opened[z], f'game data: zone_table {z} ZoneOpenTime')
        elif z in themes and themes[z]['Type_'] == 'MAINLINE':
            exact[ep] = (themes[z]['StartTs'], f'game data: activity_table ActThemes {themes[z]["Id"]}')
        else:
            a = z.rsplit('_zone', 1)[0]
            if a in themes and themes[a]['Type_'].startswith('ACTIVITY'):
                exact.setdefault(ep, (themes[a]['StartTs'], f'game data: activity_table ActThemes {themes[a]["Id"]} ({a})'))
    has_hard = {int(m.group(1)) for m in (re.match(r'hard_(\d+)-\d+$', k) for k in stages) if m}
    hard = collections.defaultdict(list)
    for r in st['StageValidInfo']:
        m = re.match(r'hard_(\d+)-\d+$', r['key'])
        if m and r['value'].get('StartTs', -1) > 0:
            hard[int(m.group(1))].append(r['value']['StartTs'])
    hard = {ep: min(v) for ep, v in hard.items()}
    offs = [hard[ep] - exact[ep][0] for ep in exact if ep in hard and hard[ep] > exact[ep][0]]
    off = statistics.median(offs) if offs else None
    first_event = min((v['StartTime'] for v in kv(act['BasicInfo']).values() if v.get('StartTime', -1) > 0))
    first_id = next(v['Id'] for v in kv(act['BasicInfo']).values() if v.get('StartTime') == first_event)
    eps = []
    first_dated = min(list(exact) + [e for e in hard if off is not None and e not in exact])
    prev = None
    for ep in sorted(set(zone_ep.values())):
        z = next(z for z, e in zone_ep.items() if e == ep)
        title = (zones.get(z) or {}).get('ZoneNameSecond') or ''
        if ep in exact:
            t, basis = exact[ep]
            g = day(t); shown = g
        elif ep in hard and off is not None:
            g = day(hard[ep] - off); shown = f'about {g}'
            basis = (f'estimate: its Adverse stages opened {day(hard[ep])}; on {len(offs)} dated episodes they open '
                     f'{min(offs) / 86400:.2f} to {max(offs) / 86400:.2f} days after the episode')
        elif ep < first_dated and ep not in has_hard:
            g = day(first_event); shown = f'by {g}'
            basis = f'upper bound: no schedule in the game data, so it shipped with the game, before its first event ({first_id})'
        else:
            g = None; shown = f'not dated in the game data (after Episode {prev})'
            basis = 'unknown: no open time, theme or Adverse schedule in the game data; placed after the episode before it'
        eps.append({'episode': ep, 'title': title, 'global': g, 'shown': shown, 'basis': basis,
                    'source': 'EN game data (zone_table, activity_table, stage_table)'})
        prev = ep
    json.dump(eps, open(os.path.join(CH, 'main_release.json'), 'w'), ensure_ascii=False, indent=1)
    print(f"dates: {len(eps)} episodes; exact {len(exact)}, estimated {sum(e['shown'].startswith('about') for e in eps)}, "
          f"upper bound {sum(e['shown'].startswith('by ') for e in eps)}, unknown {sum(e['global'] is None for e in eps)}; "
          f"Adverse offset median {off / 86400 if off else float('nan'):.2f} days from {len(offs)} episodes")


def stage_score():
    """Eval only: the game-data dates against the wiki reference."""
    mine = {e['episode']: e for e in json.load(open(os.path.join(CH, 'main_release.json')))}
    ref = {e['episode']: e for e in json.load(open(WIKI_REF)) if e.get('global')}
    both = [ep for ep in ref if ep in mine]
    dd = lambda a, b: abs((datetime.date.fromisoformat(a) - datetime.date.fromisoformat(b)).days)
    ex = [ep for ep in both if mine[ep]['global'] == ref[ep]['global']]
    near = [ep for ep in both if mine[ep]['global'] and dd(mine[ep]['global'], ref[ep]['global']) <= 1]
    print(f"score: wiki dates {len(ref)} episodes; game data dated {sum(bool(mine[ep]['global']) for ep in both)}; "
          f"same date {len(ex)}, within 1 day {len(near)}")
    for ep in both:
        m = mine[ep]
        diff = dd(m['global'], ref[ep]['global']) if m['global'] else None
        print(f"  Episode {ep}: game data {m['shown']} | wiki {ref[ep]['global']} ({ref[ep]['basis']}) | days off {diff}")


def day(t):
    return datetime.datetime.utcfromtimestamp(t).strftime('%Y-%m-%d')


def stage_build():
    tl = {g['groupId']: g for g in json.load(open(os.path.join(CH, 'timeline_v1.json')))['groups']}
    eps = {e['episode']: e for e in json.load(open(os.path.join(CH, 'main_release.json')))}
    items = []
    sort_day = {}
    for g in tl.values():
        gid = g['groupId']
        if gid.startswith('story_'):
            continue  # operator records: optional side stories, listed separately
        if g.get('chapter') is not None:
            e = eps.get(g['chapter'])
            if not e or not (e['global'] or 'shown' in e):
                continue
            items.append({'groupId': gid, 'name': g['name'], 'kind': 'main', 'episode': g['chapter'],
                          'released': e.get('shown', e['global']), 'dateBasis': e['basis']})
            # An undated episode (game data, 2026-10-04) sorts right after the episode before it.
            sort_day[gid] = e['global'] or next((eps[k]['global'] for k in range(g['chapter'] - 1, -1, -1)
                                                 if k in eps and eps[k]['global']), '')
        elif g.get('releaseTime'):
            items.append({'groupId': gid, 'name': g['name'], 'kind': 'event', 'released': day(g['releaseTime']),
                          'dateBasis': 'game data'})
    # Same day: the main episode first (events of the same day are its side events).
    # Episodes 0 to 4 share the launch date: order them by episode.
    items.sort(key=lambda i: (sort_day.get(i['groupId'], i['released']), i['kind'] != 'main', i.get('episode') or 0, i['name']))
    prior = {p['groupId']: p['prior'] for p in map(json.loads, open(os.path.join(ROOT, 'artifacts', 'primers', 'primers.jsonl')))}
    later = collections.defaultdict(list)
    for g, ps in prior.items():
        for h in ps:
            later[h].append(g)
    pos = {i['groupId']: n for n, i in enumerate(items)}
    name = {i['groupId']: i['name'] for i in items}
    last_main = None
    for n, i in enumerate(items):
        i['position'] = n + 1
        if i['kind'] == 'main':
            last_main = i['episode']
        i['afterEpisode'] = last_main
        i['buildsOn'] = [name[h] for h in prior.get(i['groupId'], []) if h in name]
        i['builtOnBy'] = [name[g] for g in sorted(later.get(i['groupId'], []), key=lambda g: pos.get(g, 1e9)) if g in name]
        # Weight as a prerequisite: how many later main episodes and events list it among what they build on.
        i['prereqOf'] = len(later.get(i['groupId'], []))
    json.dump({'items': items, 'note': 'EN release order; buildsOn = primer prior context (in-world earlier, released '
               'no later, most shared recurring speakers)'}, open(os.path.join(CH, 'reading_guide.json'), 'w'),
              ensure_ascii=False, indent=1)
    # First appearance: the earliest-released main episode or event where a speaker has a line. Word counts per
    # group give a reading time (words of dialogue and narration in the script).
    first = {}
    words = collections.Counter()
    for l in open(os.path.join(ROOT, 'artifacts', 'chunks.jsonl')):
        r = json.loads(l)
        if r['groupId'] not in pos:
            continue
        words[r['groupId']] += len(r['text'].split())
        for sp in set(r['speakers']):
            cur = first.get(sp)
            if cur is None or (pos[r['groupId']], r['ordinal']) < (pos[cur['groupId']], cur['ordinal']):
                first[sp] = {'groupId': r['groupId'], 'group': name[r['groupId']], 'storyId': r['storyId'],
                             'ordinal': r['ordinal'], 'released': items[pos[r['groupId']]]['released']}
    json.dump(first, open(os.path.join(CH, 'first_appearance.json'), 'w'), ensure_ascii=False)
    for i in items:
        i['words'] = words[i['groupId']]
    json.dump({'items': items, 'note': 'EN release order; buildsOn = primer prior context (in-world earlier, released '
               'no later, most shared recurring speakers); words = script words'},
              open(os.path.join(CH, 'reading_guide.json'), 'w'), ensure_ascii=False, indent=1)
    top = sorted((i for i in items if i['kind'] == 'event'), key=lambda i: -i['prereqOf'])[:8]
    print(f"guide: {len(items)} entries ({sum(i['kind'] == 'main' for i in items)} main, {sum(i['kind'] == 'event' for i in items)} events); "
          f"first appearances for {len(first)} speakers; most built-on events: "
          + ', '.join(f"{i['name']} ({i['prereqOf']})" for i in top))


def stage_storylines():
    st = json.load(open(os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel', 'stage_table.json')))
    kv = lambda x: {r['key']: r['value'] for r in x} if isinstance(x, list) else x
    sets = kv(st['StorylineStorySets'])
    names = {g['groupId']: g['name'] for g in json.load(open(os.path.join(CH, 'timeline_v1.json')))['groups']}

    # Episodes 15 and 16 appear in the Storylines only as their activity ids (act2mainss, act3mainss); their stages
    # sit in zones act2mainss_zone1 and act3mainss_zone1 with ids main_15-NN and main_16-NN, so the zone names the
    # episode. STORYLINE_ZONES=0 keeps the raw activity ids (the file before 2026-09-30).
    zone_main = {}
    if os.environ.get('STORYLINE_ZONES', '1') != '0':
        for sid_, sv in kv(st['Stages']).items():
            m = re.match(r'main_(\d+)-\d+$', sid_)
            if m and sv.get('ZoneId'):
                zone_main.setdefault(sv['ZoneId'].rsplit('_zone', 1)[0], f'main_{int(m.group(1))}')

    def group(set_id):
        s_ = sets.get(set_id) or {}
        g = (s_.get('MainlineData') or {}).get('ZoneId') or s_.get('RelevantActivityId')
        return zone_main.get(g, g) if g and not g.startswith('main_') else g
    lines, edges = [], []
    for sid, line in kv(st['Storylines']).items():
        locs = sorted(kv(line['Locations']).values(), key=lambda l: l.get('SortId', 0))
        order = [group(l['RelevantStorySetId']) for l in locs if l.get('LocationType') == 'STORY_SET']
        lines.append({'storylineId': sid, 'name': line.get('StorylineName'), 'type': line.get('StorylineType'),
                      'groups': [g for g in order if g], 'names': [names.get(g, g) for g in order if g]})
        # A BEFORE entry sits just ahead of the story set it precedes; an AFTER entry just behind the one it follows.
        for i, l in enumerate(locs):
            t = l.get('LocationType')
            if t not in ('BEFORE', 'AFTER'):
                continue
            other = group(l.get('RelevantStorySetId'))
            near = (next((x for x in locs[i + 1:] if x.get('LocationType') == 'STORY_SET'), None) if t == 'BEFORE'
                    else next((x for x in reversed(locs[:i]) if x.get('LocationType') == 'STORY_SET'), None))
            here = group(near['RelevantStorySetId']) if near else None
            if other and here:
                first, then = (other, here) if t == 'BEFORE' else (here, other)
                edges.append({'first': first, 'then': then, 'firstName': names.get(first, first),
                              'thenName': names.get(then, then), 'storyline': sid, 'locationId': l.get('LocationId')})
    uniq = {(e['first'], e['then']): e for e in edges}
    json.dump({'source': 'stage_table.json Storylines (the game\'s Movements view), EN game data',
               'storylines': lines, 'edges': list(uniq.values())},
              open(os.path.join(CH, 'storylines.json'), 'w'), ensure_ascii=False, indent=1)
    print(f"storylines: {len(lines)}; read-before edges {len(uniq)}")
    for e in uniq.values():
        print(f"  {e['firstName']}  ->  {e['thenName']}   ({e['storyline']})")
    # The community's rules as a check: does a game edge involve the same story and a main episode it names?
    comm = json.load(open(os.path.join(ROOT, 'data', 'reading_community.json')))['rules']
    agree = 0
    for r in comm:
        mine = [e for e in uniq.values() if r['groupId'] in (e['first'], e['then'])]
        if mine:
            agree += 1
    print(f"community placement rules with a game edge on the same story: {agree} of {len(comm)}")


if __name__ == '__main__':
    {'wiki': stage_wiki, 'dates': stage_dates, 'score': stage_score, 'build': stage_build,
     'storylines': stage_storylines}[sys.argv[1]]()
