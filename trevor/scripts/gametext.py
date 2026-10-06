#!/usr/bin/env python3
"""Lore-bearing game text Trevor never indexed, as one labelled unit kind "gametext" (2026-10-03, item 6).

The coverage audit of 2026-10-03 (design/trevor-questions.md section 12) sampled every excel field and story file under
../assets/output/en against P4 and the story corpus. The lore-bearing gaps this script ingests, each unit headed
"Game text (<kind>): <title>" so an answer can say where a claim comes from:
  intro      the game's one- or two-sentence intro of every story ("[uc]info" files, 1,860 of 1,881 missing), one unit
             per event or chapter, each line "<story name> (<tag>): <intro>", names from story_review_table
  story      story scripts never fetched: event entry, hidden and ending scripts (act12d6, act17d7, act13side
             hidden_st01..03, act1lock), act29side chats and marks, the sandbox battle scenes, the IS and guide scenes,
             minigame scripts; one unit per file (training and tutorial scripts are left out: battle instructions)
  record     the main-story record notes (obt/record/main_10..13, 35 files)
  archive    event archives (story_review_meta_table ActArchiveResData): logs, diaries and files, landmarks, news, picture
             descriptions; one unit per event and kind
  mail       mail archive letters (display_meta_table MailArchiveInfoDict), one unit per letter
  record intro  operator record intros (handbook_info_table HandbookAvgList StoryIntro), one unit per record set
  world tip  the loading-screen world-view tips (tip_table WorldViewTips), one unit
  event text activity event descriptions and news (activity_table EventDataMap EventDesList, NewsInfoList), one unit per
             activity
Left out as not lore or too noisy for retrieval: stage descriptions (stage_table, retro_table), furniture and medal
flavor text, skills, talents, missions. Deterministic, seconds, no model.

  python3 scripts/gametext.py   -> artifacts/gametext/units.jsonl (the new units) and
                                   artifacts/gametext/p4x_units.jsonl (P4's units, artifacts/sources/all_units.jsonl,
                                   byte for byte, then the new ones)
then  target/release/build-units --corpus artifacts/p3b --units artifacts/gametext/p4x_units.jsonl --out artifacts/p4x
      target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p4x --reuse-from artifacts/p4
      target/release/build-index --corpus artifacts/p4x
Read only by `ask --game-text` (P4 routes read artifacts/p4x), so P4 and every default answer stay as they were.
"""
import collections, glob, json, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sources import clean, kv, load, script, words  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STORY = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'story')
OUT = os.path.join(ROOT, 'artifacts', 'gametext')
# The unit text is cut into pieces of at most this many words, so one unit stays one or two chunks.
PIECE = int(os.environ.get('GAMETEXT_PIECE', '350'))


def read(p):
    return open(p, encoding='utf-8').read()


def main():
    units = []
    seen_ids = set()

    def add(sid, title, body, kind):
        body = clean(body)
        if words(body) < 6:
            return
        ws, parts, cur = body.split('\n'), [], []
        for line in ws:
            if cur and words('\n'.join(cur + [line])) > PIECE:
                parts.append(cur); cur = []
            cur.append(line)
        parts.append(cur)
        for i, p in enumerate(parts):
            uid = sid if len(parts) == 1 else f'{sid}_{i + 1}'
            if uid in seen_ids:
                continue
            seen_ids.add(uid)
            head = f'Game text ({kind}): {title}' + (f' (part {i + 1} of {len(parts)})' if len(parts) > 1 else '')
            units.append({'storyId': uid, 'groupId': 'gametext', 'title': head, 'text': head + '\n' + '\n'.join(p), 'speakers': []})

    corpus_ids = {json.loads(l)['storyId'] for l in open(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    reviews = kv(load('story_review_table.json')['Story_reviews'])
    txt2id = {s.get('StoryTxt'): s.get('StoryId') for v in reviews.values() for s in v.get('InfoUnlockDatas') or []}
    corpus_tails = {i.split('_level_', 1)[1] and 'level_' + i.split('_level_', 1)[1] for i in corpus_ids if '_level_' in i}
    names = {k: v.get('Name') or k for k, v in reviews.items()}

    # intro: per event, from the review table's own story list (name, tag, info path)
    info_done = set()
    for gid, v in reviews.items():
        lines = []
        for s in v.get('InfoUnlockDatas') or []:
            info = s.get('StoryInfo') or ''
            p = os.path.join(STORY, '[uc]' + info + '.txt.txt')
            if info and os.path.exists(p):
                info_done.add(os.path.normpath(p))
                t = ' '.join(read(p).split())
                if t:
                    lines.append(f"{s.get('StoryName', '')} ({s.get('AvgTag', '')}, {s.get('StoryCode', '')}): {t}")
        if lines:
            add(f'gametext_intro_{gid}', f'story intros of {names[gid]}', '\n'.join(lines), 'intro')
    # intro files no review entry names (operator records, roguelike, guides): one unit per folder
    rest = collections.defaultdict(list)
    for p in sorted(glob.glob(os.path.join(STORY, '[[]uc[]]info', '**', '*.txt'), recursive=True)):
        if os.path.normpath(p) in info_done:
            continue
        rest[os.path.dirname(os.path.relpath(p, os.path.join(STORY, '[uc]info')))].append(p)
    for folder, ps in rest.items():
        lines = [' '.join(read(p).split()) for p in ps if read(p).strip()]
        add('gametext_intro_' + folder.replace('/', '_'), f'story intros, {folder}', '\n'.join(lines), 'intro')

    # story scripts the corpus never fetched (not training or tutorial, not [uc]info)
    for p in sorted(glob.glob(os.path.join(STORY, '**', '*.txt'), recursive=True)):
        rel = os.path.relpath(p, STORY)
        # obt/rogue and obt/roguelike scenes are sources.py's "is" units already; guides and training levels are
        # battle instructions.
        if rel.startswith(('[uc]info', 'obt/record/', 'obt/rogue/', 'obt/roguelike/', 'obt/guide/')) or '/training/' in rel \
                or 'tutorial' in rel or 'traininglevel' in rel:
            continue
        parts = rel.split('.')[0].split('/')
        # In the corpus a story is the review table's StoryId of its script path; a script no review lists counts as
        # indexed when a corpus story id ends with its file name.
        sid = txt2id.get('/'.join(parts))
        if (sid and sid in corpus_ids) or parts[-1] in corpus_tails:
            continue
        body = script(p)
        grp = names.get(parts[1], parts[1]) if parts[0] == 'activities' and len(parts) > 1 else '/'.join(parts[:-1])
        # The heading leaves out the file name: its pieces ("dialog_sandbox_1_main1_3_ed") became BM25 words and put a
        # sandbox scene second for "Who is Ed?" (2026-10-03). The unit id keeps the path.
        add('gametext_story_' + '_'.join(parts), f'a story script of {grp}', body, 'story')

    # record notes
    for p in sorted(glob.glob(os.path.join(STORY, 'obt', 'record', '*', '*.txt'))):
        base = os.path.basename(p).split('.')[0]
        m = re.match(r'text_main_(\d+)_note_(\d+)', base)
        title = f'Episode {int(m.group(1))} record note {m.group(2)}' if m else base
        add('gametext_record_' + base, title, read(p), 'record note')

    # event archives
    res = load('story_review_meta_table.json')['ActArchiveResData']
    by = collections.defaultdict(lambda: collections.defaultdict(list))
    for kind, field, label in [('Logs', 'LogDesc', 'log'), ('Stories', 'Text', 'file'), ('Landmarks', 'LandmarkDesc', 'landmark'),
                               ('Pics', 'PicDescription', 'picture'), ('News', 'NewsText', 'news')]:
        for k, v in kv(res.get(kind) or []).items():
            text = v.get(field) or ''
            if kind == 'News' and v.get('NewsLines'):
                text = text + '\n' + '\n'.join(x.get('Content', '') for x in v['NewsLines'])
            if words(text) < 6:
                continue
            act = k.split('_')[0]
            head = v.get('Desc') or v.get('Date') or k
            by[act][label].append(f'{head}: {text}' if head else text)
    for act, kinds in by.items():
        for label, texts in kinds.items():
            add(f'gametext_archive_{act}_{label}', f'event archive of {names.get(act, act)}, {label}s', '\n'.join(texts), 'event archive')

    # mail
    for k, v in kv(load('display_meta_table.json')['MailArchiveData']['MailArchiveInfoDict']).items():
        add(f'gametext_mail_{k}', f"mail \"{v.get('Title', '')}\"", v.get('Content', ''), 'mail')

    # operator record intros
    chars = kv(load('character_table.json')['Characters'])
    for e in load('handbook_info_table.json')['HandbookDict']:
        for s in e['value'].get('HandbookAvgList') or []:
            op = chars.get(s.get('CharId'), {}).get('Name', s.get('CharId'))
            intro = ' '.join(x.get('StoryIntro', '') for x in s.get('AvgList') or [])
            add(f"gametext_recordintro_{s.get('StorySetId')}", f"operator record \"{s.get('StorySetName', '')}\" ({op})", intro, 'operator record intro')

    # world-view tips
    add('gametext_worldtips', 'world-view loading tips', '\n'.join(t.get('Description', '') for t in load('tip_table.json')['WorldViewTips']), 'world tip')

    # activity event text
    act = load('activity_table.json')
    for gid, groups in (act.get('Activity') or {}).items():
        for a in groups if isinstance(groups, list) else []:
            v = a.get('value') if isinstance(a, dict) else None
            if not isinstance(v, dict):
                continue
            texts = []
            for ev in kv(v.get('EventDataMap') or []).values() if isinstance(v.get('EventDataMap'), list) else []:
                texts.extend(x for x in ev.get('EventDesList') or [] if isinstance(x, str))
            for n in kv(v.get('NewsInfoList') or []).values() if isinstance(v.get('NewsInfoList'), list) else []:
                texts.append(n.get('NewsText') or '')
                texts.extend(x.get('Content', '') for x in n.get('NewsLines') or [])
            if texts:
                aid = a.get('key', gid)
                add(f'gametext_event_{aid}', f'event text of {names.get(aid, aid)}', '\n'.join(texts), 'event text')

    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, 'units.jsonl'), 'w') as f:
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + '\n')
    base = open(os.path.join(ROOT, 'artifacts', 'sources', 'all_units.jsonl'), encoding='utf-8').read()
    with open(os.path.join(OUT, 'p4x_units.jsonl'), 'w') as f:
        f.write(base)
        for u in units:
            f.write(json.dumps(u, ensure_ascii=False) + '\n')
    kinds = collections.Counter(u['title'].split(')')[0][len('Game text ('):] for u in units)
    print(f"gametext: {len(units)} units, {sum(words(u['text']) for u in units)} words; by kind {dict(kinds)}")


if __name__ == '__main__':
    main()
