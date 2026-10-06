"""Helpers the stage scripts share: paths, JSONL files, hashing, the llama-server client, thread fan-out, BM25 and
citation markers.

Each helper replaces copies that the scripts carried inline until 2026-10-06, and keeps their behavior exactly: the
same request bodies (key order included), timeouts and retry schedule, the same file encodings and modes, the same
float order in BM25. A script keeps the server URL, timeout and limits it always had and passes them in, so what a
stage sends, writes and keys on is unchanged.

Scripts import it as `import common` or `from common import ...`: run as `python3 scripts/x.py`, the scripts
directory is on sys.path, and so it is for a script that another loads with load_script().
"""
import collections, hashlib, importlib.util, json, math, os, re, threading, time, urllib.parse, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GAMEDATA = os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata')
EXCEL = os.path.join(GAMEDATA, 'excel')
# The yes/no grammar every judge call uses.
G_BOOL = 'root ::= "true" | "false"\n'
# A citation marker such as " [3]" or " [1, 4]", with the whitespace before it.
CITE = re.compile(r'\s*\[[\d,;\s]+\]')
JSON_HEADERS = {'content-type': 'application/json'}


def sha16(s):
    """The first 16 hex digits of the sha256 of a string: every input, prompt and row key in the pipeline."""
    return hashlib.sha256(s.encode()).hexdigest()[:16]


# ---------------------------------------------------------------- files

def read_jsonl(p):
    """Rows of a JSONL file, or [] when it does not exist."""
    return [json.loads(l) for l in open(p)] if os.path.exists(p) else []


def write_jsonl(path, rows, ensure_ascii=False):
    """Rewrite a JSONL file in place, one row per line."""
    with open(path, 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=ensure_ascii) + '\n')


def replace_jsonl(path, rows):
    """Rewrite a JSONL file through `<path>.tmp` and a rename, so a reader never sees half a file."""
    tmp = path + '.tmp'
    write_jsonl(tmp, rows)
    os.replace(tmp, path)


def append_jsonl(path, row, ensure_ascii=False):
    """Append one row to a JSONL file."""
    with open(path, 'a') as f:
        f.write(json.dumps(row, ensure_ascii=ensure_ascii) + '\n')


def load_json(path):
    return json.load(open(path))


def kv(x):
    """A game table's [{key, value}] list as a dict; anything else unchanged."""
    return {r['key']: r['value'] for r in x} if isinstance(x, list) and x and isinstance(x[0], dict) and 'key' in x[0] else x


def story_script(path):
    """Narration and dialogue of a story script: '[name="X"]line' becomes 'X: line'; other bracketed commands are
    stage directions and are dropped."""
    out = []
    for line in open(path, encoding='utf-8'):
        line = line.strip()
        m = re.match(r'^\[name="([^"]*)"\](.*)$', line)
        if m:
            out.append(f'{m.group(1)}: {m.group(2).strip()}')
        elif line and not line.startswith('['):
            out.append(line)
    return '\n'.join(out)


def playable_operators():
    """(charId, name) of every obtainable operator, in character_table order: no TOKEN or TRAP, not IsNotObtainable."""
    ct = json.load(open(os.path.join(EXCEL, 'character_table.json')))['Characters']
    return [(c['key'], c['value']['Name']) for c in ct
            if c['value'].get('Profession') not in ('TOKEN', 'TRAP') and not c['value'].get('IsNotObtainable')]


def load_script(name):
    """Import scripts/<name>.py as a module (names with dashes too: 'answer-eval' becomes module answer_eval)."""
    spec = importlib.util.spec_from_file_location(name.replace('-', '_'), os.path.join(ROOT, 'scripts', name + '.py'))
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
    return m


# ---------------------------------------------------------------- llama-server

def chat_body(system, user, max_tokens, **extra):
    """The chat request every stage sends: a system and a user message, greedy, seed 1. `extra` keys (grammar,
    json_schema, cache_prompt, ...) follow in the order given; a None value is left out."""
    body = {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
            'temperature': 0, 'seed': 1, 'max_tokens': max_tokens}
    body.update((k, v) for k, v in extra.items() if v is not None)
    return body


def post(url, data, timeout):
    """One POST of JSON bytes; the decoded JSON answer. No retry."""
    req = urllib.request.Request(url, data, JSON_HEADERS)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.load(r)


def post_retry(url, data, timeout, parse=lambda d: d, tries=4):
    """post() up to `tries` times, sleeping 5, 10, 15, 20 s after each failure, then raising the last error. `parse`
    runs inside the retry, so an answer it cannot read is asked again (one failed decode on llama-server 500s every
    in-flight request)."""
    err = None
    for attempt in range(tries):
        try:
            return parse(post(url, data, timeout))
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def content(resp):
    """The message text of a chat completion."""
    return resp['choices'][0]['message']['content']


def chat_url(server):
    return f'{server}/v1/chat/completions'


def stripped(resp):
    return content(resp).strip()


def chat(server, system, user, max_tokens, timeout, parse=stripped, **extra):
    """A chat completion with retries: parse(answer), by default the stripped message text. `parse` runs inside the
    retry, so an answer it cannot read is asked again."""
    data = json.dumps(chat_body(system, user, max_tokens, **extra)).encode()
    return post_retry(chat_url(server), data, timeout, parse)


def chat_once(server, body, timeout):
    """A chat completion of `body` without retries; the decoded answer."""
    return post(chat_url(server), json.dumps(body).encode(), timeout)


def verdict(server, system, user, timeout, **extra):
    """A yes/no judge call (3 tokens under G_BOOL) with retries: True when the model answers true."""
    data = json.dumps(chat_body(system, user, 3, grammar=G_BOOL, **extra)).encode()
    return post_retry(chat_url(server), data, timeout, lambda d: content(d).strip() == 'true')


def add_tool_passages(chunks, topics_required=False, overview=False):
    """Add to `chunks` ({passage id: {'text': ...}}) the passages ask builds from Trevor's own tables, as ask showed
    them to its model, for the answer judges: each topic summary as 'topic:<name>' (summaryV2, --lore v2's text, else
    the summary; citations stripped), each dossier as 'dossier:<name>' (ask --lore v2, 2026-10-02) and, with
    `overview`, each summary-tree unit as 'overview:<id>' (2026-10-01). A missing topics file is skipped unless
    `topics_required`."""
    tp = os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl')
    for t in (map(json.loads, open(tp)) if topics_required or os.path.exists(tp) else []):
        chunks[f"topic:{t['topic']}"] = {'text': CITE.sub('', t.get('summaryV2') or t.get('summary') or '')}
    dp = os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl')
    for d in (map(json.loads, open(dp)) if os.path.exists(dp) else []):
        aka = [x for x in d.get('names') or [] if x != d['name']]
        head = f"Character profile: {d['name']}" + (f" (also known as {', '.join(aka)})" if aka else '')
        chunks[f"dossier:{d['name']}"] = {'text': f"{head}\n{d['dossier']}"}
    if overview:
        op = os.path.join(ROOT, 'artifacts', 'overview', 'overview.jsonl')
        for t in (map(json.loads, open(op)) if os.path.exists(op) else []):
            chunks[f"overview:{t['id']}"] = {'text': CITE.sub('', t.get('summary') or '')}
    return chunks


# ---------------------------------------------------------------- the wiki (eval references only)

WIKI_API = 'https://arknights.wiki.gg/api.php'
WIKI_UA = 'trevor-research/0.1'


def wiki_request(params, ua=WIKI_UA):
    """A MediaWiki API GET on arknights.wiki.gg. Only the eval-reference stages call it (the wiki is an answer key,
    never a source)."""
    return urllib.request.Request(f'{WIKI_API}?{urllib.parse.urlencode(params)}', headers={'User-Agent': ua})


def wiki_get(params, ua=WIKI_UA, timeout=60):
    """One API call, decoded; no retry."""
    with urllib.request.urlopen(wiki_request(params, ua), timeout=timeout) as r:
        return json.load(r)


def wiki_get_retry(params, page, ua=WIKI_UA, timeout=60, parse=lambda t: t):
    """parse(API answer) with up to four attempts, sleeping 5, 10, 15, 20 s after each failure; then RuntimeError."""
    req = wiki_request(params, ua)
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                t = json.load(r)
            return parse(t)
        except Exception:
            time.sleep(5 * (attempt + 1))
    raise RuntimeError(f'wiki unreachable for {page}')


# ---------------------------------------------------------------- threads

def par(items, fn, n=2, lock=None):
    """Call fn on every item from `n` threads that share one iterator (guarded by `lock`, a new one by default)."""
    it = iter(items); lock = lock or threading.Lock()

    def w():
        while True:
            with lock:
                x = next(it, None)
            if x is None:
                return
            fn(x)
    th = [threading.Thread(target=w) for _ in range(n)]
    [t.start() for t in th]; [t.join() for t in th]


# ---------------------------------------------------------------- text

def bm25_rank(query, docs):
    """(score, index) of every doc for the query, best first: Okapi BM25 (k1 1.2, b 0.75) over lower-case word tokens."""
    tok = lambda s: re.findall(r"[a-z0-9']+", s.lower())
    D = [tok(d) for d in docs]; N = len(D); avg = sum(map(len, D)) / max(N, 1)
    df = collections.Counter(w for d in D for w in set(d)); q = set(tok(query)); sc = []
    for i, d in enumerate(D):
        tf = collections.Counter(d); s = 0.0
        for w in q:
            if w in tf:
                s += math.log(1 + (N - df[w] + 0.5) / (df[w] + 0.5)) * tf[w] * 2.2 / (tf[w] + 1.2 * (0.25 + 0.75 * len(d) / avg))
        sc.append((s, i))
    return sorted(sc, reverse=True)


def bm25_top(query, docs, k):
    """Indexes of the k best docs for the query (scores of 0 included)."""
    return [i for _, i in bm25_rank(query, docs)[:k]]


def evidence_window(claim, pool, k=5):
    """Indexes into `pool` (chunks in story order) of the k best BM25 chunks for a claim and their neighbours in the
    same story, sorted: the evidence a judge reads for one sentence."""
    ev = set()
    for i in bm25_top(claim, [c['text'] for c in pool], k):
        for j in (i - 1, i, i + 1):
            if 0 <= j < len(pool) and pool[j]['storyId'] == pool[i]['storyId']:
                ev.add(j)
    return sorted(ev)


def forms_regex(forms):
    """One whole-word pattern for a topic's forms of 3 or more characters, longest first: case-sensitive for a
    capitalized form, case-insensitive otherwise."""
    forms = sorted((f for f in forms if len(f) >= 3), key=len, reverse=True)
    pats = [re.escape(f) if f[0].isupper() else '(?i:' + re.escape(f) + ')' for f in forms]
    return re.compile(r"(?<![\w'])(" + '|'.join(pats) + r")(?![\w'])")


def word_norm(s):
    """Lower-case words joined by single spaces, punctuation dropped: how a quote is matched against its passage."""
    return re.sub(r'\W+', ' ', s or '').strip().lower()


def strip_cites(t):
    return CITE.sub('', t)


def cited_numbers(x):
    """The passage numbers a sentence cites, in order: '[1, 3] ... [2]' -> [1, 3, 2]."""
    return [int(n) for g in re.findall(r'\[([\d,;\s]+)\]', x) for n in re.split(r'[,;\s]+', g) if n.strip().isdigit()]


def split_sentences(t, abbr, start='[A-Z"\']'):
    """Sentences of t over 20 characters, whitespace collapsed: split after . ! or ? before a capital or a quote
    (`start`), never after an abbreviation `abbr` (a run of negative lookbehinds)."""
    return [x.strip() for x in re.split(abbr + r'(?<=[.!?])\s+(?=' + start + ')', ' '.join(t.split())) if len(x.strip()) > 20]
