#!/usr/bin/env python3
"""Operator design inspirations inferred from the game data (2026-10-04, Ian: "use the Wiki as a *reference*").

The wiki's trivia pages say what an operator's design draws on ("Lucilla is based on the giant phantom
jelly"); Trevor must deduce that himself, so nothing here reads the wiki. Per operator the evidence is the
game's own text, numbered E1..En: the codename, class and branch; race, nation, birthplace and faction
(artifacts/entities/operator_attributes.jsonl); the recruitment blurb and trait (character_table); talent,
skill and module names with their descriptions (character_table, skill_table, uniequip_table); the operator
file (handbook_info_table: profile, archive files, promotion record); and the model-written descriptions of
the E2 and outfit art (artifacts/art/captions.jsonl, labelled as model-written). Gemma names at most three
real-world subjects (an animal, plant, myth, work of fiction, person, object) the design most likely draws
on, the most specific one the evidence supports, each citing evidence ids and a confidence; Qwen then checks
each subject against the evidence items it cites and keeps it only when they support it.

  gen     Gemma on $SERVER -> artifacts/entities/design_infer.raw.jsonl (keyed on evidence + prompt; resumable)
  check   Qwen on $SERVER  -> artifacts/entities/design_infer.check.jsonl (keyed on subject + cited evidence)
  build   no model -> artifacts/entities/design_infer.jsonl {operator, charId, alters, subjects: [{subject,
          category, why, confidence, evidence: [{id, source, text}]}]}, checked subjects only (DESIGN_CHECK=0
          keeps unchecked ones); read by ask's design_basis tool
  score   eval only: Qwen on $SERVER compares each operator's served subjects with the wiki reference
          (eval/reference/design_basis.jsonl, written by scripts/design_basis.py) -> eval/reference/
          design_infer.score.jsonl and prints the exact/close match rate over the operators the wiki covers
  report  no model: the score summary again, plus per-category misses
  animal  opt-in (DESIGN_ANIMAL=1, refuted on the dev half 2026-10-04): a second Gemma call asking only for the animal
          -> artifacts/entities/design_infer.animal.jsonl; with DESIGN_ANIMAL=1, check and build add its subject
Options: --first N (operators in charId order), --ids a,b (charIds or names), --minutes M. PLAN=1 prints counts.
Environment: DESIGN_VISUAL=1 (opt-in, 2026-10-04 night: the visual design reading of artifacts/art/design.jsonl as one more
evidence item, see below), DESIGN_MODEL=tag (gen's raw rows for another generator model, e.g. 26b), DESIGN_PROMPT=v2 (opt-in, refuted: the v2 prompt and evidence), DESIGN_ANIMAL=1 (opt-in, refuted),
DESIGN_CHECK=0 (unchecked subjects), DESIGN_FILTER=high|animal|animal_high (measurement), DESIGN_OUT=path (build to,
and score or report from, another table), SPLIT=split.json SPLIT_PART=heldout|dev (score and report one part of a split
fixed before any result was read). Measured 2026-10-04 evening against the wiki answer key (design/trevor-questions.md
section 12): the served-table bar (held-out precision 0.600) was not met by any version, so build's default output is
not kept in artifacts/entities (the design tool fires on nothing without it).
"""
import argparse, collections, json, os, re, sys, threading, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT, read_jsonl, sha16

GD = common.EXCEL
ENT = os.path.join(ROOT, 'artifacts', 'entities')
# DESIGN_MODEL=tag (2026-10-04 night, the Gemma 4 26B-A4B pilot): gen writes and reads design_infer.raw.<tag>.jsonl, so
# a run on another generator model (served on $SERVER as usual) never mixes with the 12B rows; unset = the 12B file as
# before. check and score are keyed on their own input text, so their files are shared across generators.
MODEL_TAG = os.environ.get('DESIGN_MODEL', '')
RAW = os.path.join(ENT, f'design_infer.raw.{MODEL_TAG}.jsonl' if MODEL_TAG else 'design_infer.raw.jsonl')
CHK = os.path.join(ENT, 'design_infer.check.jsonl')
OUT = os.environ.get('DESIGN_OUT') or os.path.join(ENT, 'design_infer.jsonl')
REF = os.path.join(ROOT, 'eval', 'reference')
SCORE = os.path.join(REF, 'design_infer.score.jsonl')
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
kv = lambda x: {r['key']: r['value'] for r in x} if isinstance(x, list) else x

GEN_SYS = (
    "You study an Arknights operator's character design using only the game's own text below, numbered E1, E2, ... "
    "Arknights designs draw on real-world subjects. First the animal: most races are animal-like, the race names only "
    "the family, and body parts, colors, patterns, abilities, habits, food, habitat and descriptions in the evidence "
    "point to the exact species; weigh every such clue, list the clues, then name the real animal by its common English name (the exact species "
    "when several clues agree, else the family), or null when the character shows no animal. Then at most two other "
    "real-world references: a myth or folk figure, a historical person or place, or a work of literature, film or "
    "music that the codename, skill or talent names, outfits or story refer to. Never name a role, profession, hobby, "
    "weapon, tool or element the character merely has (not \"drummer\", \"ninja\", \"theatre\", \"fire\"), and "
    "never a nation, race or faction of Terra (Sami, Sarkaz, Kazimierz are the game's own). Think about what the "
    "codename and skill names mean. Each subject cites the evidence ids that point to it and says in one sentence "
    "which clues do. Do not name an animal that only the race name states (a Feline is a cat; name the kind of cat "
    "only when other clues say so). confidence: high when several independent clues point to it, medium when one "
    "strong clue does, low when it is a guess. Output JSON only.")
# v2 (2026-10-04 evening): the prompt v1 ignored three general evidence patterns, found by reading the dev half's
# misses against the wiki's sentences (never a per-operator answer): names are often a word for the subject in another
# language or a scientific name (a genus, the species epithet in a real name, the Chinese codename), the race gives the
# animal family (sea races are sea creatures, read by body clues), and Gemma returned the codename itself or the string
# "null" as a subject. v2 adds the Chinese codename (the CN character_table) and the race's served summary (the first two
# sentences of topics.jsonl summaryV2r) as evidence. OPT-IN (DESIGN_PROMPT=v2), refuted on the held-out half of the
# 175 wiki-covered operators (sha256(charId) even, 91, fixed before any score): exact-or-close 17 -> 16 of 91, precision
# over operators with a served subject 17/35 = 0.486 -> 16/41 = 0.390 (unchecked: 30/91 -> 26/87). The v1 prompt and
# evidence are unchanged without the variable (gen plans 0 operators, the v1 build is byte-identical).
GEN_SYS_V2 = (
    "You study an Arknights operator's character design using only the game's own text below, numbered E1, E2, ... "
    "Arknights designs draw on real-world subjects, most often one real animal species, and the names are the "
    "strongest clue. Step 1, names: translate every name in the evidence, the codename, the Chinese codename, and any "
    "real name, surname or alias in the files. A name is often the subject's word in another language (Latin, Greek, "
    "Chinese, Japanese, Russian, German, Italian, French, Hebrew), its scientific genus or species name (the species "
    "epithet of a Latin name can hide in a real name or surname), an old or folk name of an animal, plant or mineral, "
    "or the name of a myth, person or work. When a name denotes a real animal, plant or mineral, that thing is very "
    "likely the design subject: name it by its common English name, never by the codename itself. Step 2, the animal: "
    "the race names only the family (the race summary, when given, says what the race is like; sea races are sea "
    "creatures, whose body clues such as tentacles, an umbrella or bell shape, fins, shells, bubbles, glowing or "
    "transparent parts tell which sea animal), and body parts, colors, patterns, abilities, habits, food and habitat "
    "in the files, skills and art descriptions point to the exact species; weigh every clue, list them, then name the "
    "real animal (the exact species when several clues agree, else the family), or null when the character shows no "
    "animal. Step 3: at most two other real-world references: a myth or folk figure, a historical person or place, or "
    "a work of literature, film, music or a game that a name, skill, talent, module or outfit refers to. Never name a "
    "role, profession, hobby, weapon, tool or element the character merely has (not \"drummer\", \"ninja\", "
    "\"theatre\", \"fire\"), never a nation, race, faction, place or person of Terra (Sami, Sarkaz, Kazimierz, other "
    "operators are the game's own), and never the operator's own codename unless it is also the name of a real-world "
    "thing, in which case name that thing. Each subject cites the evidence ids that point to it and says in one "
    "sentence which clues do. Do not name an animal that only the race name states (a Feline is a cat; name the kind "
    "of cat only when other clues say so). confidence: high when several independent clues point to it, medium when "
    "one strong clue does, low when it is a guess. Output JSON only.")
GEN_GRAMMAR = r'''root ::= "{\"clues\": " clues ", \"animal\": " ( subj | "null" ) ", \"references\": [" ( subj ( ", " subj )? )? "]}"
subj ::= "{\"subject\": " str ", \"category\": " cat ", \"why\": " why ", \"evidence\": [" eid ( ", " eid ){0,5} "], \"confidence\": " conf "}"
cat ::= "\"animal\"" | "\"plant\"" | "\"mythology\"" | "\"literature or film\"" | "\"historical\"" | "\"object or concept\"" | "\"other\""
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
eid ::= "\"E" [1-9] [0-9]? "\""
str ::= "\"" [^"\n]{1,60} "\""
why ::= "\"" [^"\n]{1,300} "\""
clues ::= "\"" [^"\n]{1,900} "\""
'''
CHECK_SYS = (
    "You check one claim about an Arknights character's design against the evidence it cites, quoted from the game. "
    "The claim says the design draws on a real-world subject because of certain clues. Decide whether the cited "
    "evidence really contains those clues and whether they point to this subject more than to a generic one (a "
    "tail alone does not show a fox; a tail with a fox's ears, colors or name does). The subject must be a real-world "
    "thing outside the game: a book, place, person or song invented by the game is not supported. Answer JSON only: "
    "{\"supported\": true or false, \"reason\": \"one sentence\"}.")
CHECK_GRAMMAR = r'''root ::= "{\"supported\": " ( "true" | "false" ) ", \"reason\": \"" [^"\n]{1,240} "\"}"
'''
SCORE_SYS = (
    "You compare two lists of what an Arknights operator's design is based on. REFERENCE comes from fan wiki trivia "
    "sentences; INFERRED comes from a model that read only the game text. Answer \"exact\" when an inferred subject "
    "is the same thing as a reference design subject (same species, same figure, same work, allowing synonyms and "
    "spelling), \"close\" when an inferred subject is the same kind but broader or a near relative (\"owl\" "
    "for \"barn owl\", \"dog\" for \"wolfhound\"), else \"none\". "
    "Reference sentences about names, voice actors, real people the name comes from or in-jokes count as subjects "
    "too. Answer JSON only: {\"match\": \"exact\" | \"close\" | \"none\", \"reason\": \"one sentence\"}.")
SCORE_GRAMMAR = r'''root ::= "{\"match\": " ( "\"exact\"" | "\"close\"" | "\"none\"" ) ", \"reason\": \"" [^"\n]{1,240} "\"}"
'''
V1 = os.environ.get('DESIGN_PROMPT', 'v1') == 'v1'  # v1 stays the default: v2 lost on the held-out half (below)
if not V1:
    GEN_SYS = GEN_SYS_V2
# The visual design reading (2026-10-04 night, opt-in DESIGN_VISUAL=1): scripts/art_captions.py design asks the vision
# model what real-world animal, creature, plant or object each operator's E2 art (E0 when there is no E2 or the E2
# reading is unclear) is modelled on, given only the race; its reading is added as one more evidence item, labelled as
# a model's reading of the art, and the prompt gets one sentence on how to weigh it. Unset, the evidence and the prompt
# are the v1 ones (gen plans 0 operators, the build is byte-identical).
VISUAL = os.environ.get('DESIGN_VISUAL') == '1'
VISUAL_SYS = (
    " An item labelled \"visual design reading\" is a vision model's reading of the operator's art, made from the image "
    "with only the race as context: the real thing the design looks modelled on and the visible features that show it. "
    "It is a strong clue for the animal when its features are specific (it may name the exact species where the text "
    "names only the family); weigh it with the names and the text, and cite it when you use it.")
if VISUAL:
    GEN_SYS = GEN_SYS + VISUAL_SYS
PSHA = sha16(GEN_SYS + GEN_GRAMMAR)
# The animal pass (2026-10-04 evening, opt-in DESIGN_ANIMAL=1): v1 names an animal for few operators (5 of its 150
# subjects on the held-out half carry the category "animal"), while the animal is the commonest kind of wiki subject;
# v2 asked for it inside the one prompt and lost references. This is a second, separate call that asks only for the
# animal, with the v2 evidence (the Chinese codename and the race summary); its subject is added to the v1 subjects and
# checked like them. Its rows live in design_infer.animal.jsonl, keyed on evidence + prompt.
ANIMAL = os.path.join(ENT, 'design_infer.animal.jsonl')
ANIMAL_SYS = (
    "You name the one real animal species an Arknights operator's character design is based on, using only the game's "
    "own text below, numbered E1, E2, ... Most operators are animal-like people, and the race names only the animal "
    "family. Use three kinds of clue. Names: translate the codename, the Chinese codename and any real name, surname "
    "or alias in the files; a name is often the animal's word in another language, an old or folk name for it, or its "
    "scientific (Latin) genus or species name. The race: the race summary says what the race is like; sea races are "
    "sea creatures. The body and life: body parts, colors, patterns, abilities, habits, food and habitat in the files, "
    "skills and art descriptions. List the clues, then name the animal by its common English name, the exact species "
    "when the clues agree, else the family, or null when nothing beyond the race name points to an animal. Never give "
    "the operator's codename itself unless it is the animal's real name. Output JSON only.")
ANIMAL_GRAMMAR = r'''root ::= "{\"clues\": " clues ", \"animal\": " ( subj | "null" ) "}"
subj ::= "{\"subject\": " str ", \"category\": \"animal\", \"why\": " why ", \"evidence\": [" eid ( ", " eid ){0,5} "], \"confidence\": " conf "}"
conf ::= "\"high\"" | "\"medium\"" | "\"low\""
eid ::= "\"E" [1-9] [0-9]? "\""
str ::= "\"" [^"\n]{1,60} "\""
why ::= "\"" [^"\n]{1,300} "\""
clues ::= "\"" [^"\n]{1,900} "\""
'''
ASHA = sha16(ANIMAL_SYS + ANIMAL_GRAMMAR)
USE_ANIMAL = os.environ.get('DESIGN_ANIMAL') == '1'

# The fair answer key (2026-10-05, Ian: score only what a design can show). Of the wiki's 306 trivia rows many are not a
# design basis at all (a voice actor, a gun model, a meme, an event teaser, an outfit's art homage), and no deduction
# from the game's art and text can match them. `label` asks Qwen (temperature 0, a grammar) which kind each row is ->
# eval/reference/design_kinds.jsonl, keyed on the row's sentence + subjects + this prompt; DESIGN_KEY=design scores
# only the design-basis kinds (animal, creature, plant, object, figure), DESIGN_KEY=animal only animal and creature
# rows; operators left with no row drop out of the key. Unset, the key is every row, as before.
KIND_SYS = (
    "You sort one trivia sentence about an Arknights operator, from a fan wiki, by what it says the character design "
    "draws on. Kinds: \"animal\" (a real animal species or group the character is based on), \"creature\" (a mythical "
    "or legendary being, a cryptid, a monster or a god as a creature), \"plant\" (a plant, flower, fungus or tree), "
    "\"object\" (an object, material, vehicle, food, drink, mineral or visual motif the design shows), \"figure\" (a "
    "historical person, a mythological or literary character, or a work of literature, film or music the character is "
    "modelled on), \"other\" (anything that is not a basis of the character's own design: a voice actor or casting, a "
    "real weapon or gun model, a meme or in-joke, an event teaser, an outfit's homage to an artwork, a place, a name's "
    "meaning or etymology alone, a coincidence or similarity with another work, game mechanics). Use the subjects in "
    "brackets as a hint only. Answer JSON only: {\"kind\": ..., \"reason\": \"a few words\"}.")
KIND_GRAMMAR = r'''root ::= "{\"kind\": " ( "\"animal\"" | "\"creature\"" | "\"plant\"" | "\"object\"" | "\"figure\"" | "\"other\"" ) ", \"reason\": \"" [^"\n]{1,160} "\"}"
'''
KSHA = sha16(KIND_SYS + KIND_GRAMMAR)
KINDS = os.path.join(REF, 'design_kinds.jsonl')
DESIGN_KINDS = {'design': {'animal', 'creature', 'plant', 'object', 'figure'}, 'animal': {'animal', 'creature'}}

CSHA = sha16(CHECK_SYS + CHECK_GRAMMAR)
SSHA = sha16(SCORE_SYS + SCORE_GRAMMAR)
TAG = re.compile(r'<[@$/][^>]*>|</>')


def clean(t, n=None):
    t = TAG.sub('', t or '').replace('\\n', '\n')
    t = re.sub(r'\{[-@:.\w%]+\}', 'X', t)
    t = re.sub(r'[ \t]+', ' ', t).strip()
    return t if n is None or len(t) <= n else t[:n].rsplit(' ', 1)[0] + ' ...'


def post(sys_prompt, user, grammar, max_tokens):
    """The parsed JSON answer of a grammar-bound call. Only the request is retried (one failed decode 500s every
    in-flight request); an answer that does not parse raises."""
    content = common.chat(SERVER, sys_prompt, user, max_tokens, 900, common.content, grammar=grammar, cache_prompt=False)
    try:
        return json.loads(content)
    except json.JSONDecodeError:
        # The grammar emits no escapes, so every backslash is literal text ("\\" before a closing quote broke one
        # output); double them all. Anything else (a truncated output) is an error.
        return json.loads(content.replace('\\', '\\\\'))


def evidence(extra=None):
    """charId -> (operator name, [(id, source, text)], alters)."""
    attrs = {r['charId']: r for r in read_jsonl(os.path.join(ENT, 'operator_attributes.jsonl'))}
    ct = kv(json.load(open(os.path.join(GD, 'character_table.json')))['Characters'])
    skills = kv(json.load(open(os.path.join(GD, 'skill_table.json')))['Skills'])
    eq = kv(json.load(open(os.path.join(GD, 'uniequip_table.json')))['EquipDict'])
    hb = kv(json.load(open(os.path.join(GD, 'handbook_info_table.json')))['HandbookDict'])
    meta = json.load(open(os.path.join(GD, 'char_meta_table.json')))
    groups = kv(meta.get('SpCharGroups', []))
    alter_of = {}
    for _, ids in groups.items():
        for i in ids:
            alter_of[i] = [x for x in ids if x != i]
    caps = collections.defaultdict(list)
    for c in read_jsonl(os.path.join(ROOT, 'artifacts', 'art', 'captions.jsonl')):
        if c.get('caption'):
            caps[c['char_id']].append(c)
    visual = visual_readings() if VISUAL else {}
    mods = collections.defaultdict(list)
    for e in eq.values():
        if e.get('Type_') != 'INITIAL':
            mods[e['CharId']].append(e)
    cn_names, race_sum = {}, {}
    if (not V1) if extra is None else extra:
        cn = os.path.join(ROOT, '..', 'assets', 'output', 'cn', 'gamedata', 'excel', 'character_table.json')
        if os.path.exists(cn):
            cct = json.load(open(cn))
            cct = kv(cct.get('Characters', cct))
            cn_names = {k: v.get('Name') for k, v in cct.items() if isinstance(v, dict) and v.get('Name')}
        for t in read_jsonl(os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl')):
            if t.get('kind') == 'race' and (t.get('summaryV2r') or t.get('summary')):
                txt = re.sub(r'\s*\[[\d, ]+\]', '', t.get('summaryV2r') or t['summary'])
                sents = re.split(r'(?<=[.!?])\s+', txt.strip())
                for name in [t['topic']] + list(t.get('aliases') or []):
                    race_sum[name] = ' '.join(sents[:2])
    out = {}
    for cid, a in attrs.items():
        c = ct.get(cid) or {}
        ev = []
        add = lambda src, text: ev.append((f'E{len(ev) + 1}', src, text)) if text and text.strip() else None
        add('codename', f"Codename: {a['name']}. Class: {a.get('class')}, branch {a.get('branch')}.")
        bits = [f"{k}: {a[k]}" for k in ('race', 'nation', 'birthplace', 'group', 'team', 'gender') if a.get(k)]
        add('operator file, basic info', '; '.join(bits) + '.')
        if (not V1) if extra is None else extra:
            if cn_names.get(cid) and cn_names[cid] != a['name']:
                add('Chinese codename (the game\'s CN text)', f"Chinese codename: {cn_names[cid]}")
            if a.get('race') and race_sum.get(a['race']):
                add('race summary (Trevor\'s summary of the story text about the race)', f"{a['race']}: {race_sum[a['race']]}")
        add('recruitment text', clean(f"{c.get('ItemUsage', '')} {c.get('ItemDesc', '')}"))
        add('trait', clean(c.get('Description', '')))
        for t in c.get('Talents') or []:
            cand = (t.get('Candidates') or [{}])[-1]
            if cand.get('Name'):
                add('talent', f"Talent \"{cand['Name']}\": {clean(cand.get('Description', ''), 220)}")
        for s in c.get('Skills') or []:
            lv = ((skills.get(s.get('SkillId')) or {}).get('Levels') or [{}])[-1]
            if lv.get('Name'):
                add('skill', f"Skill \"{lv['Name']}\": {clean(lv.get('Description', ''), 220)}")
        for m in mods[cid][:3]:
            add('module', f"Module \"{m.get('UniEquipName', '')}\": {clean(m.get('UniEquipDesc', ''), 300)}")
        for st in (hb.get(cid) or {}).get('StoryTextAudio') or []:
            title = st.get('StoryTitle', '')
            if title in ('Basic Info', 'Physical Exam') or not st.get('Stories'):
                continue
            text = st['Stories'][0].get('StoryText', '')
            if title == 'Clinical Analysis':
                # The medical boilerplate first; what is particular to the operator follows the density line.
                text = re.split(r'\[Blood Originium-Crystal Density\][^\n]*\n', text)[-1]
            add(f'operator file, {title}', clean(text, 900))
        cs = sorted(caps[cid], key=lambda x: (x['kind'] != 'e2', x['id']))
        for x in cs[:4]:
            what = 'E2 art' if x['kind'] == 'e2' else f"outfit art \"{x.get('skin_name') or ''}\""
            add(f'{what}, model-written description', clean(x['caption'], 700))
        if VISUAL and visual.get(cid):
            v = visual[cid]
            what = 'E2 (Elite 2) art' if v['art'] == 'e2' else 'E0 (base) art'
            add(f'visual design reading of the operator\'s {what} (a vision model\'s reading of the image, not game text)',
                f"The design looks modelled on: {v['subject']} (group: {v.get('broader') or 'not given'}; kind: {v.get('kind')}; "
                f"confidence {v.get('confidence')}). The character's own features: "
                f"{'; '.join(v.get('body_features') or v.get('features') or []) or 'none listed'}. "
                f"Other motifs: {'; '.join(v.get('motifs') or []) or 'none listed'}.")
        out[cid] = (a['name'], ev, [attrs[i]['name'] for i in alter_of.get(cid, []) if i in attrs])
    return out


def visual_readings():
    """charId -> the visual design reading to use: the E2 reading unless it is unclear (no subject, low confidence or
    unparsable) and an E0 reading names something with at least its confidence; readings that name nothing are dropped."""
    conf = {'high': 2, 'medium': 1, 'low': 0}
    by = collections.defaultdict(dict)
    for r in read_jsonl(os.environ.get('DESIGN_VISUAL_FILE') or os.path.join(ROOT, 'artifacts', 'art', 'design.jsonl')):
        if not r.get('parse_error'):
            by[r['char_id']][r['art']] = r  # the last row per art wins (a rerun with a new prompt appends)
    named = lambda r: r and (r.get('subject') or 'none').strip().lower() not in ('none', 'null', '')
    out = {}
    for cid, d in by.items():
        e2, e0 = d.get('e2'), d.get('e0')
        best = e2 if named(e2) else None
        if named(e0) and (best is None or (best.get('confidence') == 'low' and conf[e0['confidence']] >= conf[best['confidence']])):
            best = e0
        if best:
            out[cid] = best
    return out


def user_text(name, ev):
    return f"Operator: {name}\n\n" + '\n'.join(f"{i} ({src}): {t}" for i, src, t in ev)


def select(E, args):
    ids = sorted(E)
    if args.ids:
        want = {x.strip().lower() for x in args.ids.split(',')}
        ids = [i for i in ids if i.lower() in want or E[i][0].lower() in want]
    if args.first:
        ids = ids[:args.first]
    return ids


def run_pool(todo, fn, threads, minutes):
    lock = threading.Lock(); it = iter(todo); t0 = time.time()

    def work():
        while True:
            if minutes and time.time() - t0 > minutes * 60:
                return
            with lock:
                x = next(it, None)
            if x is None:
                return
            fn(x, lock)
    th = [threading.Thread(target=work) for _ in range(threads)]
    [t.start() for t in th]; [t.join() for t in th]


def stage_gen(args):
    E = evidence()
    done = {r['key'] for r in read_jsonl(RAW)}
    todo = []
    for cid in select(E, args):
        name, ev, _ = E[cid]
        u = user_text(name, ev)
        key = sha16(u + PSHA)
        if key not in done:
            todo.append((cid, name, u, key))
    print(f'gen: {len(todo)} operators to infer', flush=True)
    if os.environ.get('PLAN'):
        print(f'plan {len(todo)}')
        return

    def fn(x, lock):
        cid, name, u, key = x
        t = time.time()
        try:
            r = post(GEN_SYS, u, GEN_GRAMMAR, 800)
        except json.JSONDecodeError:
            try:
                r = post(GEN_SYS, u, GEN_GRAMMAR, 1400)  # truncated at 800 tokens: the same greedy output, longer
            except json.JSONDecodeError as e:
                print(f'{name}: unparsable output ({e}); skipped', flush=True)
                return
        r['subjects'] = ([r['animal']] if r.get('animal') else []) + r.get('references', [])
        if not V1:  # v2: the string "null" and the bare codename are not subjects
            r['subjects'] = [x for x in r['subjects'] if x['subject'].strip().lower() not in ('null', 'none', name.lower())]
        with lock:
            common.append_jsonl(RAW, {'key': key, 'charId': cid, 'operator': name, 'subjects': r['subjects'], 'clues': r.get('clues'),
                                      'promptSha': PSHA, 'seconds': round(time.time() - t, 1)})
            print(f"{name}: {[s['subject'] + '/' + s['confidence'] for s in r['subjects']]} ({time.time() - t:.0f}s)", flush=True)
    run_pool(todo, fn, args.threads, args.minutes)


def stage_animal(args):
    E = evidence(True)
    done = {r['key'] for r in read_jsonl(ANIMAL)}
    todo = []
    for cid in select(E, args):
        name, ev, _ = E[cid]
        u = user_text(name, ev)
        key = sha16(u + ASHA)
        if key not in done:
            todo.append((cid, name, u, key))
    print(f'animal: {len(todo)} operators', flush=True)
    if os.environ.get('PLAN'):
        print(f'plan {len(todo)}')
        return

    def fn(x, lock):
        cid, name, u, key = x
        t = time.time()
        try:
            r = post(ANIMAL_SYS, u, ANIMAL_GRAMMAR, 900)
        except json.JSONDecodeError as e:
            print(f'{name}: unparsable output ({e}); skipped', flush=True)
            return
        a = r.get('animal')
        if a and a['subject'].strip().lower() in ('null', 'none', name.lower()):
            a = None
        with lock:
            common.append_jsonl(ANIMAL, {'key': key, 'charId': cid, 'operator': name, 'animal': a, 'clues': r.get('clues'),
                                         'promptSha': ASHA, 'seconds': round(time.time() - t, 1)})
            print(f"{name}: {a and a['subject'] + '/' + a['confidence']} ({time.time() - t:.0f}s)", flush=True)
    run_pool(todo, fn, args.threads, args.minutes)


def latest_raw(E):
    """charId -> the raw row whose key matches the current evidence and prompt."""
    want = {cid: sha16(user_text(n, ev) + PSHA) for cid, (n, ev, _) in E.items()}
    raw = {r['charId']: r for r in read_jsonl(RAW) if want.get(r['charId']) == r['key']}
    if USE_ANIMAL:
        EA = evidence(True)
        wa = {cid: sha16(user_text(n, ev) + ASHA) for cid, (n, ev, _) in EA.items()}
        for r in read_jsonl(ANIMAL):
            if wa.get(r['charId']) == r['key'] and r['charId'] in raw and r.get('animal'):
                a = dict(r['animal'])
                # The animal pass numbers its evidence with two more items (Chinese codename, race summary) than v1, so
                # its cited items are carried with their own ids and text for the check and the table.
                a['evidence_items'] = [(i, src, t) for i, src, t in EA[r['charId']][1] if i in a['evidence']]
                old = {x['subject'].strip().lower() for x in raw[r['charId']]['subjects']}
                if a['subject'].strip().lower() not in old:
                    raw[r['charId']] = {**raw[r['charId']], 'subjects': raw[r['charId']]['subjects'] + [a]}
    return raw


def check_items(E, raw):
    for cid, r in raw.items():
        name, ev, _ = E[cid]
        byid = {i: (src, t) for i, src, t in ev}
        for s in r['subjects']:
            cited = (s['evidence_items'] if 'evidence_items' in s
                     else [(i, *byid[i]) for i in dict.fromkeys(s['evidence']) if i in byid])
            text = (f"Operator: {name}\nClaim: the design draws on {s['subject']} ({s['category']}), because {s['why']}\n\n"
                    "Cited evidence:\n" + ('\n'.join(f"{i} ({src}): {t}" for i, src, t in cited) or '(none)'))
            yield cid, s, cited, text, sha16(text + CSHA)


def stage_check(args):
    E = evidence(); raw = latest_raw(E)
    sel = set(select(E, args))
    done = {r['key'] for r in read_jsonl(CHK)}
    todo = [x for x in check_items(E, raw) if x[0] in sel and x[4] not in done]
    print(f'check: {len(todo)} subjects to check', flush=True)
    if os.environ.get('PLAN'):
        print(f'plan {len(todo)}')
        return

    def fn(x, lock):
        cid, s, cited, text, key = x
        v = post(CHECK_SYS, text, CHECK_GRAMMAR, 200) if cited else {'supported': False, 'reason': 'no valid evidence id'}
        with lock:
            common.append_jsonl(CHK, {'key': key, 'charId': cid, 'subject': s['subject'], **v})
            print(f"{E[cid][0]} / {s['subject']}: {v['supported']}", flush=True)
    run_pool(todo, fn, args.threads, args.minutes)


GENERIC = {'horn', 'feather', 'tail', 'wing', 'fur', 'scale', 'fin', 'ear', 'eye', 'teeth', 'claw', 'animal', 'creature',
           'water', 'fire', 'crystal', 'cat ear', 'tentacle', 'shell', 'paw', 'fang', 'beast', 'monster', 'spirit'}


def words(t):
    return {w[:-1] if len(w) > 4 and w.endswith('s') and not w.endswith('ss') else w
            for w in re.findall(r"[a-z]+", (t or '').lower()) if len(w) >= 3} - {'the', 'and', 'like', 'large', 'small'}


def visual_seen():
    """charId -> the words of what the art readings name: tiled body/companion subjects, the whole-image subject and group."""
    seen = collections.defaultdict(set)
    for cid, v in visual_readings().items():
        seen[cid] |= words(v.get('subject')) | words(v.get('broader'))
    for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'art', 'design_tiles.jsonl')):
        for t in r.get('subjects') or []:
            if {'body', 'companion'} & set(t.get('where') or []) and t['name'] not in GENERIC:
                seen[r['char_id']] |= words(t['name'])
    return seen


def tiles_rows(E):
    """DESIGN_FILTER=tiles (2026-10-05, the table slice): per operator the tiled reading's top animal or mythical-creature
    subject seen in 2 or more tiles on the body or as a companion (a body part alone, "horns", is not a subject)."""
    tl = {}
    for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'art', 'design_tiles.jsonl')):
        tl[r['char_id']] = r
    rows = []
    for cid in sorted(tl):
        if cid not in E:
            continue
        name, _, alters = E[cid]
        subs = [t for t in tl[cid]['subjects'] if t['kind'] in ('animal', 'mythical creature') and t['tiles'] >= 2
                and {'body', 'companion'} & set(t['where']) and not (words(t['name']) <= GENERIC)]
        rows.append({'operator': name, 'charId': cid, 'alters': alters, 'subjects': [
            {'subject': t['name'], 'category': 'animal' if t['kind'] == 'animal' else 'mythology', 'confidence': 'medium',
             'why': f"tiled art reading: {', '.join(t['features'][:3])} ({t['tiles']} of 4 tiles)", 'checked': False,
             'evidence': []} for t in subs[:1]]})
    return rows


def stage_build(_args):
    E = evidence()
    if os.environ.get('DESIGN_FILTER') == 'tiles':
        rows = tiles_rows(E)
        common.write_jsonl(OUT, rows)
        print(f"build: tiles slice, {len(rows)} operators read, {sum(bool(r['subjects']) for r in rows)} with a subject -> {OUT}")
        return
    raw = latest_raw(E)
    seen = visual_seen() if os.environ.get('DESIGN_FILTER') == 'visual_agree' else {}
    chk = {r['key']: r for r in read_jsonl(CHK)}
    use_check = os.environ.get('DESIGN_CHECK') != '0'
    rows, n_sub, n_kept, unchecked = [], 0, 0, 0
    by = collections.defaultdict(list)
    for cid, s, cited, _text, key in check_items(E, raw):
        n_sub += 1
        c = chk.get(key)
        if c is None:
            unchecked += 1
        if use_check and not (c and c['supported']):
            continue
        # DESIGN_FILTER (measurement, 2026-10-04 evening): high = high-confidence subjects only, animal = the animal
        # only, animal_high = both; unset keeps every subject the check (or DESIGN_CHECK=0) keeps, as before.
        filt = os.environ.get('DESIGN_FILTER', '')
        if ('high' in filt and s['confidence'] != 'high') or ('animal' in filt and s['category'] != 'animal'):
            continue
        # visual_agree (2026-10-05, the table slice): an animal subject only when the art readings name the same animal
        # (a shared word with the tiled readings' body or companion subjects or the whole-image reading's subject or group)
        if filt == 'visual_agree' and (s['category'] != 'animal' or not (words(s['subject']) & seen.get(cid, set()))):
            continue
        n_kept += 1
        by[cid].append({'subject': s['subject'], 'category': s['category'], 'why': s['why'], 'confidence': s['confidence'],
                        'checked': bool(c and c['supported']),
                        'evidence': [{'id': i, 'source': src, 'text': clean(t, 300)} for i, src, t in cited]})
    for cid in sorted(raw):
        name, _, alters = E[cid]
        rows.append({'operator': name, 'charId': cid, 'alters': alters, 'subjects': by.get(cid, [])})
    common.write_jsonl(OUT, rows)
    print(f"build: {len(rows)} operators inferred of {len(E)}; subjects {n_sub}, kept {n_kept} "
          f"(unchecked {unchecked}); operators with a kept subject {sum(bool(r['subjects']) for r in rows)} -> {OUT}")


def kind_text(r):
    return f"Operator: {r['operator']}\nSection: {r.get('section', '')}\nSentence: {r['sentence']}\n[subjects: {', '.join(r['basis'])}]"


def kind_key(r):
    return sha16(kind_text(r) + KSHA)


def reference():
    keep = DESIGN_KINDS.get(os.environ.get('DESIGN_KEY', ''))
    kinds = {k['key']: k['kind'] for k in read_jsonl(KINDS)} if keep else {}
    ref = collections.defaultdict(list)
    for r in read_jsonl(os.path.join(REF, 'design_basis.jsonl')):
        # an outfit's trivia is never the character's own design basis (the 30-row spot check of 2026-10-05: 3 outfit rows
        # Qwen labelled figure or object, film and mural homages, all other by the task's definition)
        if keep and (kinds.get(kind_key(r)) not in keep or r.get('section', '').lower().startswith('outfit')):
            continue
        ref[r['charId']].append(r)
    return ref


# The deduction evidence (2026-10-05, read by ask --design-deduce): per operator one compact row of what the game shows
# about the design, from the game data alone: the race, the recruitment text, the operator-file sentences that name a
# body feature or a creature, the model-written E2 art description, the whole-image visual design reading
# (artifacts/art/design.jsonl) and the tiled readings (artifacts/art/design_tiles.jsonl: subjects with their tile
# counts, where they are and their features). No model, seconds; rebuilt whole (rows are small) -> artifacts/entities/
# design_evidence.jsonl. ask shortlists from it by data (race family, feature words) before any model call.
EVID = os.path.join(ENT, 'design_evidence.jsonl')
BODY = re.compile(r'\b(ears?|tails?|horns?|antlers?|wings?|feathers?|scales?|fins?|tentacles?|shells?|fur|claws?|fangs?|'
                  r'gills?|beak|hooves|paws?|spots?|stripes?|jelly|jellyfish|octopus|squid|shark|whale|fish|bird|cat|dog|'
                  r'wolf|fox|bear|snake|dragon|lizard|crab|turtle|rabbit|deer|goat|sheep|horse|lion|tiger|leopard)\b', re.I)


def stage_evidence(_args):
    E = evidence(extra=False)
    attrs = {r['charId']: r for r in read_jsonl(os.path.join(ENT, 'operator_attributes.jsonl'))}
    vis = visual_readings()
    tiles = {}
    for r in read_jsonl(os.path.join(ROOT, 'artifacts', 'art', 'design_tiles.jsonl')):
        tiles[r['char_id']] = r  # the last row per operator wins (a rerun with a new prompt appends)
    n_tiles = 0
    with open(EVID, 'w') as f:
        for cid in sorted(E):
            name, ev, alters = E[cid]
            a = attrs.get(cid, {})
            parts = [f"Race: {a.get('race') or 'not stated'}."]
            for _i, src, t in ev:
                if src == 'recruitment text':
                    parts.append(f"Recruitment text: {clean(t, 200)}")
            body = []
            for _i, src, t in ev:
                if src.startswith('operator file') and src != 'operator file, basic info':
                    body += [x.strip() for x in re.split(r'(?<=[.!?])\s+', t) if BODY.search(x)]
            if body:
                parts.append('Operator file: ' + clean(' '.join(body[:3]), 400))
            cap = next((t for _i, src, t in ev if src.startswith('E2 art')), None) or \
                next((t for _i, src, t in ev if src.endswith('model-written description')), None)
            if cap:
                parts.append(f"Art (model-written description): {clean(cap, 600)}")
            v = vis.get(cid)
            if v:
                parts.append(f"Whole-image reading: looks modelled on {v['subject']} ({v.get('confidence')}); features: "
                             f"{'; '.join((v.get('body_features') or v.get('features') or [])[:5]) or 'none'}")
            tr = tiles.get(cid)
            if tr and tr.get('subjects'):
                n_tiles += 1
                parts.append('Tiled reading (2x2 crops): ' + '; '.join(
                    f"{s['name']} ({'/'.join(w for w in s['where'] if w)}, {s['tiles']} of 4 tiles: {', '.join(s['features'][:3])})"
                    for s in tr['subjects'][:6]))
            f.write(json.dumps({'charId': cid, 'operator': name, 'race': a.get('race') or '', 'alters': alters,
                                'text': '\n'.join(parts)}, ensure_ascii=False) + '\n')
    print(f"evidence: {len(E)} operators, {len(vis)} with a whole-image reading, {n_tiles} with a tiled reading -> {EVID}")


def stage_label(args):
    rows = read_jsonl(os.path.join(REF, 'design_basis.jsonl'))
    done = {k['key'] for k in read_jsonl(KINDS)}
    todo = [r for r in rows if kind_key(r) not in done]
    print(f'label: {len(todo)} of {len(rows)} wiki rows to sort', flush=True)
    if os.environ.get('PLAN'):
        return

    def fn(r, lock):
        v = post(KIND_SYS, kind_text(r), KIND_GRAMMAR, 120)
        with lock:
            common.append_jsonl(KINDS, {'key': kind_key(r), 'charId': r['charId'], 'operator': r['operator'],
                                        'basis': r['basis'][:3], **v})
    run_pool(todo, fn, args.threads, args.minutes)
    c = collections.Counter(k['kind'] for k in read_jsonl(KINDS))
    print('label: kinds', dict(c))


def score_text(r, rs):
    refl = '\n'.join(f"- {', '.join(x['basis'])}: \"{x['sentence']}\"" for x in rs)
    inf = '\n'.join(f"- {s['subject']} ({s['confidence']})" for s in r['subjects']) or '(nothing inferred)'
    text = f"Operator: {r['operator']}\nREFERENCE:\n{refl}\nINFERRED:\n{inf}"
    return text, sha16(text + SSHA)


def split_ids():
    """SPLIT=file.json SPLIT_PART=heldout|dev: score only that part of a split fixed before any result was read."""
    if not os.environ.get('SPLIT'):
        return None
    return set(json.load(open(os.environ['SPLIT']))[os.environ.get('SPLIT_PART', 'heldout')])


def stage_score(args):
    rows = {r['charId']: r for r in read_jsonl(OUT)}
    ref = reference()
    part = split_ids()
    if part is not None:
        ref = {c: v for c, v in ref.items() if c in part}
    done = {r['key'] for r in read_jsonl(SCORE)}
    todo = []
    for cid, rs in ref.items():
        r = rows.get(cid)
        if r is None:
            continue
        text, key = score_text(r, rs)
        if key not in done:
            todo.append((cid, r, text, key))
    print(f'score: {len(todo)} operators to compare', flush=True)
    if os.environ.get('PLAN'):
        return

    def fn(x, lock):
        cid, r, text, key = x
        v = post(SCORE_SYS, text, SCORE_GRAMMAR, 200) if r['subjects'] else {'match': 'none', 'reason': 'nothing inferred'}
        with lock:
            common.append_jsonl(SCORE, {'key': key, 'charId': cid, 'operator': r['operator'], **v})
    run_pool(todo, fn, args.threads, args.minutes)
    stage_report(args)


def stage_report(_args):
    rows = {r['charId']: r for r in read_jsonl(OUT)}
    ref = reference()
    part = split_ids()
    if part is not None:
        ref = {c: v for c, v in ref.items() if c in part}
    judged = {r['key']: r for r in read_jsonl(SCORE)}
    # The judgment of the current table's comparison text (the score file holds every version's judgments).
    sc = {c: judged[score_text(rows[c], ref[c])[1]] for c in ref if c in rows and score_text(rows[c], ref[c])[1] in judged}
    cov = [c for c in ref if c in rows]
    m = collections.Counter(sc[c]['match'] for c in cov if c in sc)
    n = len(cov)
    print(f"score: wiki covers {len(ref)} operators; inferred {n}; judged {sum(m.values())}: exact {m['exact']}, close "
          f"{m['close']}, none {m['none']}; exact rate {m['exact'] / max(n, 1):.3f}, exact or close {(m['exact'] + m['close']) / max(n, 1):.3f}")
    empty = sum(1 for c in cov if not rows[c]['subjects'])
    print(f"  of the {n}: {empty} with no checked subject")
    extra = [c for c in rows if c not in ref and rows[c]['subjects']]
    print(f"  operators outside the wiki's 175 with a checked subject: {len(extra)}")
    # Precision: of the operators with a served subject, how many match the reference (exact or close).
    served = [c for c in cov if rows[c]['subjects'] and c in sc]
    hit = sum(sc[c]['match'] in ('exact', 'close') for c in served)
    print(f"  precision over operators with a served subject: {hit}/{len(served)} = {hit / max(len(served), 1):.3f}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('stage', choices=['gen', 'animal', 'check', 'build', 'score', 'report', 'label', 'evidence'])
    ap.add_argument('--first', type=int, default=0)
    ap.add_argument('--ids', default='')
    ap.add_argument('--minutes', type=float, default=0)
    ap.add_argument('--threads', type=int, default=2)
    args = ap.parse_args()
    {'gen': stage_gen, 'animal': stage_animal, 'check': stage_check, 'build': stage_build, 'score': stage_score, 'report': stage_report,
     'label': stage_label, 'evidence': stage_evidence}[args.stage](args)


if __name__ == '__main__':
    main()
