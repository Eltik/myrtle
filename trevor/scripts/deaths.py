#!/usr/bin/env python3
"""Death events: who dies in which story, extracted from the script, with a verbatim quote.

Ian (2026-09-28): "What non-playable characters died in x story", and Arknights is live service, so a
character's status can change with each new story. Deaths are therefore stored as events tied to the
story where they happen, not as a per-character status: status as of a reader's progress is the
latest event in the stories they have cleared, and every event carries its own spoiler gate. The
summary-built operator table (operator_status.py) was refuted (19 flagged, most wrong), so this reads
the script passages themselves, and a quote must be found verbatim in its passage or the mention is
dropped. Every stage is keyed per chunk (text sha + prompt sha), so an asset update re-extracts only
changed or new passages.
  wiki     eval reference only (network): every wiki page whose infobox status is Deceased ->
           eval/reference/wiki_deceased.jsonl (83 named characters plus 2 list pages on 2026-09-28); since
           2026-10-04 nothing that builds deaths.jsonl reads it (Ian: the wiki is a reference, never a source)
  extract  generator model on $SERVER, chunks with a death word (4,700 of 13,837), DEATH_GROUPS=a,b limits
           to groups (pilot) -> entities/death_mentions.jsonl (resumable, 2 threads)
  check    generator model: a strict second question per named mention ("does this passage show NAME
           actually dying?"); the pilot's playable events were 4 of 4 wrong on the first pass
           (Amiya "I'm already dead", Crownslayer "we lost contact") -> entities/death_checks.jsonl
  build    quotes verified, names resolved (identity links, character_table), named vs unnamed
           ("my son", "the others"), check verdicts applied (DEATH_CHECK=0 ignores them), one event per
           (story, character, kind) at its first confirmed passage -> entities/deaths.jsonl. A death is
           `corroborated` from the game text alone (2026-10-04): it is reported in 2 or more story groups and
           the character speaks in no story released after the first report (W and Ines, reported dead twice
           each, speak in 7 and 5 later groups, which is what refuted the bare 2-story rule); release order is
           timeline_v1's releaseTime and the game-data main episode dates (reading_guide.py dates). Routes show
           the rest apart. DEATH_CONFIRM=wiki restores the wiki confirmation (eval/reference/wiki_deceased.jsonl
           and wiki_status.jsonl, and the wiki titles as known names) for measurement only
  eval     the game-text confirmation against the wiki reference: precision and recall over named characters,
           and over playable operators
PLAN=1 makes extract and check print what they would do and stop, calling no model.
"""
import collections, json, os, re, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT, read_jsonl, sha16

ENT = os.path.join(ROOT, 'artifacts', 'entities')
REF = os.path.join(ROOT, 'eval', 'reference')
LEGACY = os.environ.get('DEATH_CONFIRM') == 'wiki'
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
GD = common.EXCEL
DEATH = re.compile(r"\b(die[sd]?|dying|death|dead|killed|kill(?:ed)?|perish\w*|slain|passed away|corpse|lifeless|funeral|"
                   r"sacrific\w+|last breath|stopped breathing|murder\w*|executed|execution)\b", re.I)
SYS = ("You read a passage of an Arknights story script, written as 'Speaker: line'. List every character whose death this "
       "passage shows or reports. For each give: name, as the passage calls them; kind, \"dies\" if they die in this scene "
       "or are reported to have just died in the story's present, \"dead\" if the passage mentions them as having died "
       "earlier, before this story; quote, one line from the passage copied exactly, that shows or states the death. "
       "Leave out threats, fears, wounds, near-death, deaths in dreams or visions, figurative uses of 'dead', and unnamed "
       "crowds ('many soldiers'). Output a JSON list, [] if none.")
GRAMMAR = r'''root ::= "[" ( item ( ", " item )* )? "]"
item ::= "{\"name\": " str ", \"kind\": " ( "\"dies\"" | "\"dead\"" ) ", \"quote\": " qstr "}"
str ::= "\"" [^"\n]{1,60} "\""
qstr ::= "\"" [^"\n]{1,300} "\""
'''
PSHA = sha16(SYS + GRAMMAR)


# Speaker labels that name a role, not a person ("Injured Soldier", "Mysterious Mercenary"): kept, marked
# generic, listed after named characters.
GENERIC = re.compile(r"^(?:Infected|Sarkaz|Guard|Soldier|Mercenary|Citizen|Villager|Inquisitor|Knight|Civilian|Worker|Officer|"
                     r"Vanguard|Cultist|Banshee|Seaborn|Messenger|Hunter|Priest|Nun|Monk)s?$|"
                     r"^(?:Mysterious|Injured|Dying|Unknown|Wounded|Young|Old|Masked|Hooded)\b|\s(?:Member|Operator|Soldier|Vanguard|"
                     r"Guard|Mercenary|Citizen|Officer|Worker|Villager|Infected|Man|Woman|Girl|Boy|Child|Agent|Scout|Civilian|"
                     r"Resident|Thug|Bandit|Trooper|Messenger|Warrior|Knight|Teacher|Fighter|Assassin)s?$")


def norm(s):
    return re.sub(r'\W+', ' ', s).strip().lower()


def chat(user):
    """The death mentions the model lists for one passage (its JSON answer, parsed inside the retry)."""
    return common.chat(SERVER, SYS, user, 500, 600, lambda d: json.loads(common.content(d)), grammar=GRAMMAR)


def stage_wiki():
    titles, off = [], 0
    get = common.wiki_get
    while True:
        t = get({'action': 'query', 'list': 'search', 'srsearch': 'insource:/status *= *Deceased/', 'srlimit': 500,
                 'sroffset': off, 'format': 'json', 'srnamespace': 0})
        titles += [h['title'] for h in t['query']['search']]
        if 'continue' not in t:
            break
        off = t['continue']['sroffset']; time.sleep(1)
    os.makedirs(REF, exist_ok=True)
    with open(os.path.join(REF, 'wiki_deceased.jsonl'), 'w') as f:
        for title in titles:
            t = get({'action': 'parse', 'page': title, 'prop': 'wikitext', 'format': 'json'})
            w = t['parse']['wikitext']['*'] if 'parse' in t else ''
            m = re.search(r'^\|\s*status\s*=\s*(.*)$', w, re.M)
            f.write(json.dumps({'title': title, 'statusRaw': m.group(1).strip() if m else None,
                                'fetched': time.strftime('%Y-%m-%d')}, ensure_ascii=False) + '\n')
            time.sleep(1)
    print(f'wiki: {len(titles)} pages with status Deceased')


def stage_extract():
    rows = read_jsonl(os.path.join(ROOT, 'artifacts', 'chunks.jsonl'))
    groups = set(filter(None, os.environ.get('DEATH_GROUPS', '').split(',')))
    todo_all = [r for r in rows if DEATH.search(r['text']) and (not groups or r['groupId'] in groups)]
    path = os.path.join(ENT, 'death_mentions.jsonl')
    done = {(r['chunkId'], r['textSha'], r['promptSha']) for r in read_jsonl(path)}
    todo = [r for r in todo_all if (r['chunkId'], sha16(r['text']), PSHA) not in done]
    print(f'extract: {len(todo)} passages to do of {len(todo_all)}, prompt {PSHA}', flush=True)
    if os.environ.get('PLAN'):
        return
    lock = threading.Lock(); n = [0]; t0 = time.time()

    def one(r):
        ms = chat(r['text'])
        text = norm(r['text'])
        for m in ms:
            m['quoteFound'] = bool(norm(m['quote'])) and norm(m['quote']) in text
        with lock:
            common.append_jsonl(path, {'chunkId': r['chunkId'], 'storyId': r['storyId'], 'groupId': r['groupId'],
                                       'textSha': sha16(r['text']), 'promptSha': PSHA, 'mentions': ms})
            n[0] += 1
            if n[0] % 100 == 0:
                print(f'{time.strftime("%H:%M:%S")} extract {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.2f} s each', flush=True)
    common.par(todo, one, 2, lock)
    print('extract done', flush=True)


CHECK_SYS = ("You check one claim against a passage of an Arknights story script. Answer true only if the passage shows "
             "or plainly states that this character dies or is dead. Answer false if the character only loses contact, is "
             "missing, wounded, threatened, speaks figuratively ('I'm already dead'), appears in a dream or vision, or if the "
             "death is someone else's. Answer true or false only.")
CSHA = sha16(CHECK_SYS)


def check(text, name, quote):
    return common.verdict(SERVER, CHECK_SYS, f'PASSAGE:\n{text}\n\nCHARACTER: {name}\nQUOTE: {quote}', 600)


def mkey(r, m):
    return sha16(f"{r['chunkId']}|{r['textSha']}|{m['name']}|{m['kind']}|{m['quote']}|{CSHA}")


JUNK_NAMES = {'he', 'she', 'they', 'him', 'her', 'them', 'dad', 'mom', 'mum', 'papa', 'mama', 'father', 'mother',
              'brother', 'sister', 'one', 'dead', 'king', 'queen', 'someone', 'everyone', 'nobody', 'anyone', 'boss',
              'sir', 'madam', 'captain', 'doctor', 'kid', 'child'}


def namer():
    """Named = a known speaker, identity name or operator (plus the wiki deceased titles under
    DEATH_CONFIRM=wiki), or a proper name in capitals; everything else ("my son", "the four soldiers",
    "others") is an unnamed death, counted but not listed."""
    known = set()
    for l in open(os.path.join(ROOT, 'artifacts', 'chunks.jsonl')):
        known.update(x.lower() for x in json.loads(l)['speakers'])
    for i in json.load(open(os.path.join(ENT, 'identities.json'))):
        known.update(n.lower() for n in i['names'])
    for w in (read_jsonl(os.path.join(REF, 'wiki_deceased.jsonl')) if LEGACY else []):
        known.add(w['title'].lower())
    ct = json.load(open(os.path.join(GD, 'character_table.json')))['Characters']
    known.update(c['value']['Name'].lower() for c in ct if c['value'].get('Name'))
    lead = re.compile(r"^(?:my|his|her|their|our|your|the|a|an|all|another|some|those|these)\b", re.I)
    proper = lambda n: all(w[:1].isupper() or w in ('of', 'the', 'de', 'von', 'van', 'la', 'le') for w in n.split())

    def named(name, resolved):
        n = name.strip()
        # Pronouns and kin words passed as capitalized names ("He", "They", "Dad", "Mama") and so did "Dead" and
        # "One"; a real-question sweep matched "Is she dead?" to a character called "Dead" (2026-09-28).
        if n.lower() in JUNK_NAMES:
            return False
        # Known names, or a proper name in capitals that is not "Her Majesty" or "My Captain" (Yliš, the King
        # of Sarkaz in 898, is in no speaker list).
        return bool(n[:1].isupper() and not lead.match(n) and
                    (n.lower() in known or resolved.lower() in known or proper(n)))
    return named


def stage_check():
    chunks = {r['chunkId']: r for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'chunks.jsonl'))}
    named = namer(); resolve = resolver()
    path = os.path.join(ENT, 'death_checks.jsonl')
    done = {r['key'] for r in read_jsonl(path)}
    todo = []
    for r in read_jsonl(os.path.join(ENT, 'death_mentions.jsonl')):
        if r['promptSha'] != PSHA or r['chunkId'] not in chunks:
            continue
        for m in r['mentions']:
            if m['quoteFound'] and named(m['name'], resolve(m['name'])[0]) and mkey(r, m) not in done:
                todo.append((r, m))
    print(f'check: {len(todo)} named mentions to check', flush=True)
    if os.environ.get('PLAN'):
        return
    lock = threading.Lock()

    def one(x):
        r, m = x
        ok = check(chunks[r['chunkId']]['text'], m['name'], m['quote'])
        with lock:
            common.append_jsonl(path, {'key': mkey(r, m), 'ok': ok}, ensure_ascii=True)
    common.par(todo, one, 2, lock)
    C = read_jsonl(path)
    print(f"check: {sum(c['ok'] for c in C)}/{len(C)} confirmed", flush=True)


def resolver():
    ops = {name: cid for cid, name in common.playable_operators()}
    alias = {}
    for i in json.load(open(os.path.join(ENT, 'identities.json'))):
        for n in i['names']:
            # A title passes between people ("King of Sarkaz": Yliš, Theresa, Amiya), so a death under a title
            # never resolves to whoever holds it now; the pilot gave Yliš's death in 898 to Amiya.
            rel = {e['relation'] for e in i['evidence'].get(n, [])}
            if rel and rel <= {'title'}:
                continue
            alias.setdefault(n.lower(), i['label'])
    title = re.compile(r"^(?:(?:Captain|Lord|Lady|Duke|Duchess|Sir|Miss|Mr\.|Mrs\.|Ms\.|Dr\.|Master|King|Queen|Saint|General|"
                       r"Commander|Professor|Grand Knight|High Inquisitor|Inquisitor|Old|Young|the|The)\s+)+")

    def untitle(n):
        # "Captain Ulpianus" -> "Ulpianus", but "The King of Sarkaz" stays whole: a title is dropped only
        # in front of a name, never in front of "of".
        t = title.sub('', n)
        return t if t and t[:1].isupper() else n

    def resolve(name):
        n = name.strip().strip('"\'')
        for cand in (n, untitle(n)):
            if cand.lower() in alias:
                label = alias[cand.lower()]
                return label, ops.get(label) or next((ops[x] for x in (cand, n) if x in ops), None)
        base = untitle(n)
        return base, ops.get(base) or ops.get(n)
    return resolve


def release_order():
    """groupId -> a sortable EN release key from the game data: timeline_v1's releaseTime for events, the
    main_release.json date (reading_guide.py dates) for main episodes; an undated episode sits after the one before."""
    import datetime
    order = {}
    for g in json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))['groups']:
        if g.get('releaseTime'):
            order[g['groupId']] = float(g['releaseTime'])
    last = 0.0
    for e in sorted(json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'main_release.json'))), key=lambda e: e['episode']):
        if e.get('global'):
            last = datetime.datetime.strptime(e['global'], '%Y-%m-%d').replace(tzinfo=datetime.timezone.utc).timestamp()
        order[f"main_{e['episode']}"] = last + e['episode'] * 1e-3
    return order


def speakers_by_group():
    sp = collections.defaultdict(set)
    for l in open(os.path.join(ROOT, 'artifacts', 'chunks.jsonl')):
        r = json.loads(l)
        for x in r['speakers']:
            sp[norm(x)].add(r['groupId'])
    return sp


def stage_build():
    chunks = {r['chunkId']: r for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'chunks.jsonl'))}
    rel = {g['groupId']: g.get('releaseTime') for g in json.load(open(os.path.join(ROOT, 'artifacts', 'chrono', 'timeline_v1.json')))['groups']}
    resolve = resolver(); named = namer()
    use_check = os.environ.get('DEATH_CHECK') != '0'
    checks = {c['key']: c['ok'] for c in read_jsonl(os.path.join(ENT, 'death_checks.jsonl'))}
    if LEGACY:
        wiki_dead = {norm(w['title']) for w in read_jsonl(os.path.join(REF, 'wiki_deceased.jsonl'))}
        sys.path.insert(0, os.path.join(ROOT, 'scripts'))
        from wiki_status import wiki_class
        wiki_op = {r['charId']: wiki_class(r['statusRaw']) for r in read_jsonl(os.path.join(REF, 'wiki_status.jsonl'))}
    M = [r for r in read_jsonl(os.path.join(ENT, 'death_mentions.jsonl')) if r['promptSha'] == PSHA
         and r['chunkId'] in chunks and r['textSha'] == sha16(chunks[r['chunkId']]['text'])]
    events = {}; unnamed = collections.Counter(); dropped = collections.Counter()
    for r in sorted(M, key=lambda r: (r['storyId'], chunks[r['chunkId']]['ordinal'])):
        for m in r['mentions']:
            if not m['quoteFound']:
                dropped['quote not in passage'] += 1
                continue
            if len(norm(m['quote']).split()) < 3:
                # "Kreide: ......" matched verbatim and showed nothing.
                dropped['quote under 3 words'] += 1
                continue
            who, cid = resolve(m['name'])
            if not named(m['name'], who):
                unnamed[(r['storyId'], m['kind'])] += 1
                continue
            if use_check and not checks.get(mkey(r, m), False):
                dropped['check false or missing'] += 1
                continue
            key = (r['storyId'], who, m['kind'])
            if key in events:
                continue
            events[key] = {'character': who, 'nameInText': m['name'], 'playable': cid is not None, 'charId': cid,
                           'generic': cid is None and bool(GENERIC.search(who)), 'kind': m['kind'], 'storyId': r['storyId'], 'groupId': r['groupId'],
                           'releaseTime': rel.get(r['groupId']), 'chunkId': r['chunkId'], 'quote': m['quote']}
    stories_of = collections.defaultdict(set)
    for e in events.values():
        stories_of[e['character']].add(e['storyId'])
    groups_of = collections.defaultdict(set)
    for e in events.values():
        groups_of[e['character']].add(e['groupId'])
    order, speaks = release_order(), speakers_by_group()
    for e in events.values():
        if e['playable'] and LEGACY:
            # The wiki confirmation before 2026-10-04 (measurement only). "Reported in 2+ stories" alone was refuted
            # on the full run: W and Ines, both alive, are reported dead in two stories each (in-story rumors), and
            # Ebenholz inherited the previous Graf Urtica's death through the title. DEATH_CORROB_STORIES=1 restores it.
            e['corroborated'] = (norm(e['character']) in wiki_dead or wiki_op.get(e['charId']) in ('deceased', 'mixed')
                                 or (os.environ.get('DEATH_CORROB_STORIES') == '1' and len(stories_of[e['character']]) >= 2))
        elif e['playable'] or not LEGACY:
            # From the game text (2026-10-04): reported in 2+ story groups, and no line in any later-released group.
            c = e['character']
            first = min((order[g] for g in groups_of[c] if g in order), default=None)
            later = sorted(g for g in speaks.get(norm(c), set()) | speaks.get(norm(e['nameInText']), set())
                           if first is not None and order.get(g, -1) > first)
            e['corroborated'] = len(groups_of[c]) >= 2 and first is not None and not later
            e['confirm'] = {'storyGroups': len(groups_of[c]), 'laterGroupsSpeaking': len(later), 'firstLaterGroup':
                            later[0] if later else None}
    common.write_jsonl(os.path.join(ENT, 'deaths.jsonl'), events.values())
    with open(os.path.join(ENT, 'deaths_unnamed.json'), 'w') as f:
        json.dump([{'storyId': s, 'kind': k, 'count': n} for (s, k), n in sorted(unnamed.items())], f, indent=0)
    c = collections.Counter(e['kind'] for e in events.values())
    P = [e for e in events.values() if e['playable']]
    print(f"build: {len(M)} passages, {sum(len(r['mentions']) for r in M)} mentions, unnamed {sum(unnamed.values())}, "
          f"dropped {dict(dropped)}; {len(events)} named events {dict(c)}; playable {len(P)} "
          f"(corroborated {sum(e['corroborated'] for e in P)})")


def stage_eval():
    """Eval only: the game-text confirmation against the wiki reference (eval/reference)."""
    E = read_jsonl(os.path.join(ENT, 'deaths.jsonl'))
    W = read_jsonl(os.path.join(REF, 'wiki_deceased.jsonl'))
    groups = set(filter(None, os.environ.get('DEATH_GROUPS', '').split(',')))
    found = {norm(e['character']) for e in E} | {norm(e['nameInText']) for e in E}
    ref = [w['title'] for w in W if '/' not in w['title'] and '(NPC)' not in w['title']]
    hit = [t for t in ref if norm(t) in found or norm(t.split()[0]) in found]
    print(f'eval: wiki reference {len(ref)} named deceased; found in the events {len(hit)}')
    if groups:
        print('  (pilot: only groups ' + ','.join(sorted(groups)) + ' were extracted, so recall is over those alone)')
    refn = {norm(t) for t in ref} | {norm(t.split()[0]) for t in ref}
    named = collections.defaultdict(list)
    for e in E:
        if not e['generic']:
            named[e['character']].append(e)
    inref = lambda c, es: norm(c) in refn or any(norm(e['nameInText']) in refn for e in es)
    conf = {c for c, es in named.items() if any(e.get('corroborated') for e in es)}
    pos = {c for c, es in named.items() if inref(c, es)}
    tp = conf & pos
    print(f'  named characters with events {len(named)}, in the wiki reference {len(pos)}; confirmed from the game text '
          f'{len(conf)}: precision {len(tp)}/{len(conf)} = {len(tp) / max(len(conf), 1):.3f}, recall {len(tp)}/{len(pos)} = '
          f'{len(tp) / max(len(pos), 1):.3f}')
    print('  confirmed and in the reference:', ', '.join(sorted(tp)))
    print('  confirmed, not in the reference:', ', '.join(sorted(conf - pos)))
    ops = {c for c, es in named.items() if es[0]['playable']}
    wst = {r['charId']: r.get('status') for r in read_jsonl(os.path.join(REF, 'wiki_status.jsonl'))}
    op_ref = {c for c in ops if inref(c, named[c]) or wst.get(named[c][0]['charId']) in ('deceased', 'mixed')}
    print(f'  playable operators with events {len(ops)}: confirmed {len(conf & ops)} ({", ".join(sorted(conf & ops))}), '
          f'wiki marks dead {len(op_ref)}; agree on {sum((c in conf) == (c in op_ref) for c in ops)} of {len(ops)}')
    unl = sorted({e['character'] for e in E if norm(e['character']) not in refn})
    print(f'  events for {len(unl)} named characters the wiki does not mark deceased:', ', '.join(unl[:80]))


if __name__ == '__main__':
    {'wiki': stage_wiki, 'extract': stage_extract, 'check': stage_check, 'build': stage_build, 'eval': stage_eval}[sys.argv[1]]()
