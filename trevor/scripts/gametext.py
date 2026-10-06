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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import GAMEDATA, ROOT, kv, story_script as script
from sources import clean, load, words

STORY = os.path.join(GAMEDATA, 'story')
OUT = os.path.join(ROOT, 'artifacts', 'gametext')
# The unit text is cut into pieces of at most this many words, so one unit stays one or two chunks.
PIECE = int(os.environ.get('GAMETEXT_PIECE', '350'))


def read(p):
    return open(p, encoding='utf-8').read()


class Units:
    """The gametext units in the order added; a body is cut into pieces of at most PIECE words, one unit each, and a
    unit id seen before is skipped."""

    def __init__(self):
        self.rows, self.seen = [], set()

    def add(self, sid, title, body, kind):
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
            if uid in self.seen:
                continue
            self.seen.add(uid)
            head = f'Game text ({kind}): {title}' + (f' (part {i + 1} of {len(parts)})' if len(parts) > 1 else '')
            self.rows.append({'storyId': uid, 'groupId': 'gametext', 'title': head, 'text': head + '\n' + '\n'.join(p), 'speakers': []})


def add_intros(u, reviews, names):
    """Story intros: per event, from the review table's own story list (name, tag, info path); then the intro files no
    review entry names (operator records, roguelike, guides), one unit per folder."""
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
            u.add(f'gametext_intro_{gid}', f'story intros of {names[gid]}', '\n'.join(lines), 'intro')
    rest = collections.defaultdict(list)
    for p in sorted(glob.glob(os.path.join(STORY, '[[]uc[]]info', '**', '*.txt'), recursive=True)):
        if os.path.normpath(p) in info_done:
            continue
        rest[os.path.dirname(os.path.relpath(p, os.path.join(STORY, '[uc]info')))].append(p)
    for folder, ps in rest.items():
        lines = [' '.join(read(p).split()) for p in ps if read(p).strip()]
        u.add('gametext_intro_' + folder.replace('/', '_'), f'story intros, {folder}', '\n'.join(lines), 'intro')


def add_unfetched_stories(u, reviews, names):
    """Story scripts the corpus never fetched (not training or tutorial, not [uc]info)."""
    corpus_ids = {json.loads(l)['storyId'] for l in open(os.path.join(ROOT, 'artifacts', 'p4', 'chunks.jsonl'))}
    txt2id = {s.get('StoryTxt'): s.get('StoryId') for v in reviews.values() for s in v.get('InfoUnlockDatas') or []}
    corpus_tails = {i.split('_level_', 1)[1] and 'level_' + i.split('_level_', 1)[1] for i in corpus_ids if '_level_' in i}
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
        u.add('gametext_story_' + '_'.join(parts), f'a story script of {grp}', body, 'story')


def add_record_notes(u):
    for p in sorted(glob.glob(os.path.join(STORY, 'obt', 'record', '*', '*.txt'))):
        base = os.path.basename(p).split('.')[0]
        m = re.match(r'text_main_(\d+)_note_(\d+)', base)
        title = f'Episode {int(m.group(1))} record note {m.group(2)}' if m else base
        u.add('gametext_record_' + base, title, read(p), 'record note')


def add_event_archives(u, names):
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
            u.add(f'gametext_archive_{act}_{label}', f'event archive of {names.get(act, act)}, {label}s', '\n'.join(texts), 'event archive')


def add_mail(u):
    for k, v in kv(load('display_meta_table.json')['MailArchiveData']['MailArchiveInfoDict']).items():
        u.add(f'gametext_mail_{k}', f"mail \"{v.get('Title', '')}\"", v.get('Content', ''), 'mail')


def add_record_intros(u):
    """Operator record intros."""
    chars = kv(load('character_table.json')['Characters'])
    for e in load('handbook_info_table.json')['HandbookDict']:
        for s in e['value'].get('HandbookAvgList') or []:
            op = chars.get(s.get('CharId'), {}).get('Name', s.get('CharId'))
            intro = ' '.join(x.get('StoryIntro', '') for x in s.get('AvgList') or [])
            u.add(f"gametext_recordintro_{s.get('StorySetId')}", f"operator record \"{s.get('StorySetName', '')}\" ({op})", intro, 'operator record intro')


def add_event_text(u, names):
    """Activity event text: event descriptions and news."""
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
                u.add(f'gametext_event_{aid}', f'event text of {names.get(aid, aid)}', '\n'.join(texts), 'event text')


def main():
    u = Units()
    reviews = kv(load('story_review_table.json')['Story_reviews'])
    names = {k: v.get('Name') or k for k, v in reviews.items()}
    add_intros(u, reviews, names)
    add_unfetched_stories(u, reviews, names)
    add_record_notes(u)
    add_event_archives(u, names)
    add_mail(u)
    add_record_intros(u)
    u.add('gametext_worldtips', 'world-view loading tips', '\n'.join(t.get('Description', '') for t in load('tip_table.json')['WorldViewTips']), 'world tip')
    add_event_text(u, names)
    units = u.rows

    os.makedirs(OUT, exist_ok=True)
    common.write_jsonl(os.path.join(OUT, 'units.jsonl'), units)
    base = open(os.path.join(ROOT, 'artifacts', 'sources', 'all_units.jsonl'), encoding='utf-8').read()
    with open(os.path.join(OUT, 'p4x_units.jsonl'), 'w') as f:
        f.write(base)
        for row in units:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
    kinds = collections.Counter(row['title'].split(')')[0][len('Game text ('):] for row in units)
    print(f"gametext: {len(units)} units, {sum(words(row['text']) for row in units)} words; by kind {dict(kinds)}")


if __name__ == '__main__':
    main()
