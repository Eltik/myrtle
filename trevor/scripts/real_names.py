#!/usr/bin/env python3
"""Real names of playable operators, each with a verbatim quote, for whole-roster questions.

Ian (2026-09-28): "List all the known real names of every operator" was answered from 17 retrieved
passages: 7 names, two of them codenames (Rose Salt, Titi), one not an operator. A whole-roster
question needs a table. Per operator (407, character_table minus tokens, traps, unobtainable), the
generator reads the operator's archive plus up to 4 story passages where the codename appears near a
naming cue ("real name", "full name", "née", ...) and gives the real name with a quote. Kept only when
the quote is verbatim in those passages, contains the name, the name differs from the codename, and a
second strict question confirms that the quote states it is this operator's name. Keyed per operator
on (charId, input sha, prompt sha) via incr.Stage, so an update redoes only operators whose passages
changed.
  gen    generator model on $SERVER -> artifacts/entities/real_names.jsonl (PLAN=1: dry run)
  show   print the table
"""
import collections, json, os, re, sys, threading, time, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import incr

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'artifacts', 'entities', 'real_names.jsonl')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3a'))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
GD = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata', 'excel')
CUE = re.compile(r"real name|true name|full name|birth name|given name|family name|surname|n[ée]e\b|born as|"
                 r"my name is|name's|named\b|codename|code name", re.I)
SYS = ("You read passages about one Arknights operator, known by a codename. Give the operator's real name: their own "
       "personal name, as opposed to the codename, as the passages state it (a full name such as 'Margaret Nearl' "
       "counts). Answer null if the passages do not state it, if the codename is their only name, or if the name "
       "belongs to someone else. quote: one line copied exactly from the passages that contains the name.")
GRAMMAR = r'''root ::= "{\"realName\": " ( "null" | str ) ", \"quote\": " ( "null" | qstr ) "}"
str ::= "\"" [^"\n]{1,60} "\""
qstr ::= "\"" [^"\n]{1,300} "\""
'''
# The first check prompt ("states or plainly shows") rejected Gummy = Lada, whom a friend addresses by it,
# and Ch'en = Ch'en Hui-chieh ("or my name isn't Ch'en Hui-chieh"), both 2 of 8 correct names in the pilot.
CHECK_SYS = ("You check one claim about an Arknights operator against a quote from the game text. Answer true if the "
             "quote shows the given name is this operator's own personal name: stated as their real or full name, used "
             "by the operator for themselves, or used by someone addressing them by name. Answer false if it is another "
             "person's name, a nickname or title, or only a guess. Answer true or false only.")
PSHA = incr.sha16(SYS + GRAMMAR + CHECK_SYS)


def base(name):
    # An alter operator's codename carries a title ("Hoshiguma the Breacher"); its base is the codename.
    return re.split(r'\s+the\s+', name, maxsplit=1)[0]


def norm(s):
    return re.sub(r'\W+', ' ', s or '').strip().lower()


def post(system, user, max_tokens, grammar):
    body = json.dumps({'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
                       'temperature': 0, 'seed': 1, 'max_tokens': max_tokens, 'grammar': grammar}).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=900) as r:
                return json.load(r)['choices'][0]['message']['content']
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def operators():
    ct = json.load(open(os.path.join(GD, 'character_table.json')))['Characters']
    return [(c['key'], c['value']['Name']) for c in ct
            if c['value'].get('Profession') not in ('TOKEN', 'TRAP') and not c['value'].get('IsNotObtainable')]


def build_stage():
    rows = incr.read_jsonl(os.path.join(CORPUS, 'chunks.jsonl'))
    arch = collections.defaultdict(list); story = []
    for r in rows:
        if r['storyId'].startswith('archive_'):
            arch[r['storyId'][len('archive_'):]].append(r)
        elif r['groupId'] not in ('profile', 'summary') and CUE.search(r['text']):
            story.append(r)
    by_story = collections.defaultdict(list)
    for r in rows:
        by_story[r['storyId']].append(r)
    link_quotes = collections.defaultdict(list)
    for i in json.load(open(os.path.join(ROOT, 'artifacts', 'entities', 'identities.json'))):
        for n in i['names']:
            for e in i['evidence'].get(n, []):
                if e['relation'] != 'title':
                    link_quotes[i['label']].append((e['quote'], e['where']))
    st = incr.Stage('real names', OUT, lambda r: r['charId'], PSHA)
    inputs = {}
    only = set(filter(None, os.environ.get('ONLY_NAMES', '').split(',')))  # a pilot: these codenames only
    for cid, name in operators():
        if only and name not in only:
            continue
        pat = re.compile(r"(?<![\w'])" + re.escape(name) + r"(?![\w'])")
        near = []
        for r in story:
            if pat.search(r['text']):
                # Rank by naming cues within 200 characters of the codename.
                hits = sum(1 for m in pat.finditer(r['text']) if CUE.search(r['text'][max(0, m.start() - 200):m.end() + 200]))
                if hits:
                    near.append((-hits, r['chunkId'], r))
        near = [r for _, _, r in sorted(near)[:4]]
        # Passages behind this operator's identity links (entities.py), where a story reveals a name.
        for q, where in link_quotes.get(name, []):
            for r in by_story.get(where, []):
                if norm(q) in norm(r['text']) and r not in near:
                    near.append(r)
                    break
        passages = sorted(arch.get(cid, []), key=lambda r: r['ordinal']) + near
        if not passages:
            continue
        user = f"Operator codename: {name}\n\n" + '\n---\n'.join(f"[{r['chunkId']}]\n{r['text']}" for r in passages)
        inputs[cid] = (name, user, passages)
        st.unit(cid, user)
    return st, inputs


def stage_gen():
    st, inputs = build_stage()
    todo = st.plan(list(inputs))
    if todo is None:
        return
    print(f'real names: {len(todo)} operators to do, prompt {PSHA}', flush=True)
    it = iter(todo); lock = threading.Lock(); n = [0]; t0 = time.time()

    def work():
        while True:
            with lock:
                cid = next(it, None)
            if cid is None:
                return
            name, user, passages = inputs[cid]
            out = json.loads(post(SYS, user, 200, GRAMMAR))
            # The grammar allows the string "null" as well as null; both mean no name.
            real, quote = [None if (v or '').strip().lower() in ('', 'null', 'none', 'unknown') else v.strip()
                           for v in (out.get('realName'), out.get('quote'))]
            src = next((r for r in passages if quote and norm(quote) and norm(quote) in norm(r['text'])), None)
            why = None
            if not real:
                why = 'none stated'
            elif not src:
                why = 'quote not in passages'
            elif norm(real) not in norm(quote):
                why = 'name not in quote'
            elif norm(real) in (norm(name), norm(base(name))):
                why = 'same as codename'
            else:
                ok = post(CHECK_SYS, f'OPERATOR (codename): {name}\nNAME: {real}\nQUOTE: {quote}', 3,
                          'root ::= "true" | "false"\n').strip() == 'true'
                why = None if ok else 'check false'
            row = {'name': name, 'charId': cid, 'realName': real if why is None else None, 'candidate': real,
                   'quote': quote, 'chunkId': src['chunkId'] if src else None, 'storyId': src['storyId'] if src else None,
                   'rejected': why, 'promptSha': PSHA}
            st.put(row)
            with lock:
                n[0] += 1
                if n[0] % 50 == 0:
                    print(f'{time.strftime("%H:%M:%S")} real names {n[0]}/{len(todo)}, {(time.time() - t0) / n[0]:.1f} s each', flush=True)
    th = [threading.Thread(target=work) for _ in range(2)]
    [t.start() for t in th]; [t.join() for t in th]
    R = incr.read_jsonl(OUT)
    print(f"real names: {len(R)} operators, {sum(bool(r['realName']) for r in R)} with a real name; "
          f"rejected {dict(collections.Counter(r['rejected'] for r in R if r['rejected']))}", flush=True)


def stage_show():
    for r in incr.read_jsonl(OUT):
        if r['realName']:
            print(f"{r['name']}: {r['realName']} | {r['quote'][:100]} ({r['storyId']})")


if __name__ == '__main__':
    {'gen': stage_gen, 'show': stage_show}[sys.argv[1]]()
