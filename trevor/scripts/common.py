"""Helpers the stage scripts share: paths, JSONL files, hashing, the llama-server client, thread fan-out, BM25 and
citation markers.

Each helper replaces copies that the scripts carried inline until 2026-10-06, and keeps their behavior exactly: the
same request bodies (key order included), timeouts and retry schedule, the same file encodings and modes, the same
float order in BM25. A script keeps the server URL, timeout and limits it always had and passes them in, so what a
stage sends, writes and keys on is unchanged.

Scripts import it as `import common` or `from common import ...`: run as `python3 scripts/x.py`, the scripts
directory is on sys.path, and so it is for a script that another loads with load_script().
"""
import collections, hashlib, importlib.util, json, math, os, re, sys, threading, time, urllib.parse, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# TREVOR_GAMEDATA points at another checkout's game data (a git worktree has no assets/output of its own).
GAMEDATA = os.environ.get('TREVOR_GAMEDATA') or os.path.join(ROOT, '..', 'assets', 'output', 'en', 'gamedata')
EXCEL = os.path.join(GAMEDATA, 'excel')
# The yes/no grammar every judge call uses.
G_BOOL = 'root ::= "true" | "false"\n'
# A citation marker such as " [3]" or " [1, 4]", with the whitespace before it.
CITE = re.compile(r'\s*\[[\d,;\s]+\]')
JSON_HEADERS = {'content-type': 'application/json'}


def llm_lock_path():
    """The shared model lock, resolved as `ask --lock` resolves its default (src/bin/ask/cli.rs, lock_path_from):
    $TREVOR_LLM_LOCK when set and non-empty, else trevor-llm.lock in $TMPDIR, or in /tmp when that is unset or empty."""
    return os.environ.get('TREVOR_LLM_LOCK') or os.path.join(os.environ.get('TMPDIR') or '/tmp', 'trevor-llm.lock')


def sha16(s):
    """The first 16 hex digits of the sha256 of a string: every input, prompt and row key in the pipeline."""
    return hashlib.sha256(s.encode()).hexdigest()[:16]


# ---------------------------------------------------------------- files

class MissingInput(FileNotFoundError):
    """A stage input that must exist does not: a derived file an earlier stage writes, or a game table. A subclass of
    FileNotFoundError, so a caller that caught the open() error before still catches it."""


class MissingTable(MissingInput):
    """A game data table is not in excel/ (an asset update can leave one out, backend core/gamedata/stale.rs)."""


def read_jsonl(p, required=False):
    """Rows of a JSONL file, or [] when it does not exist. `required`: raise MissingInput instead (2026-10-19): an
    absent derived input read as [] made a stage overwrite its good output with an empty one."""
    if os.path.exists(p):
        return [json.loads(l) for l in open(p)]
    if required:
        raise MissingInput(f'missing input {os.path.relpath(p, ROOT)}: the stage that writes it has not run or failed; '
                           'nothing was written')
    return []


def read_derived(p):
    """Rows of a JSONL file an earlier stage writes; MissingInput when it is absent. DERIVED_REQUIRED=0 is the kill
    switch: [] again, as every call site's read_jsonl read it before 2026-10-19 (load_derived is not affected: its call
    sites raised on a missing file before too)."""
    return read_jsonl(p, required=os.environ.get('DERIVED_REQUIRED') != '0')


def load_derived(p):
    """A JSON file an earlier stage writes, decoded; MissingInput when it is absent."""
    if not os.path.exists(p):
        raise MissingInput(f'missing input {os.path.relpath(p, ROOT)}: the stage that writes it has not run or failed; '
                           'nothing was written')
    return json.load(open(p))


# ---------------------------------------------------------------- keeping the last good output (2026-10-19)
#
# Two different events keep an output as it was, and they end a stage differently:
#  - a MISSING INPUT: a game table is absent (keep_last_on_missing_table) or a derived file is (read_derived raises
#    MissingInput before anything is computed). The stage could not compute its output, so it keeps the last one, says
#    "keeping last" and, for a missing game table, exits 0: one table left out of an asset update is expected and must
#    not stop the update.
#  - a REFUSED WRITE: every input was present, the output was computed, and write_keep_last refused it because it came
#    out empty or shrank past its policy. That is not expected, so it is suspicious: write_keep_last returns False and
#    a caller that must not publish a frozen file exits 3 (answer_bank seed and merge, topics units), as incr.py does,
#    so update.sh (set -e) stops and alerts instead of publishing the old file night after night.
# KEEP_LAST=0 governs only the writes. It does not restore the reads before 2026-10-19: DERIVED_REQUIRED=0 does that
# for read_derived, TABLES_REQUIRED=1 makes a missing table fail, INCR_ALLOW_SHRINK=1 lifts incr.py's guard.

def keep_last_on():
    """KEEP_LAST=0 is the kill switch: every guarded write is a plain write again, whatever it replaces."""
    return os.environ.get('KEEP_LAST') != '0'


def max_drop():
    """The share of rows a rewrite may lose before it is refused: INCR_MAX_DROP percent, 20 by default."""
    return float(os.environ.get('INCR_MAX_DROP', '20')) / 100


def shrinks(old, new, policy='pct'):
    """Whether replacing `old` rows (or entries) with `new` loses all of them or, under policy 'pct', more than
    max_drop() of them. Policy None refuses only the loss of all of them."""
    return old > 0 and (new == 0 or (policy == 'pct' and old - new > max_drop() * old))


def _count_file(path, kind, count):
    """Rows of a JSONL file, or count(decoded JSON); None when there is no readable file to keep."""
    try:
        if kind == 'jsonl':
            with open(path, 'rb') as f:
                return sum(1 for l in f if l.strip())
        with open(path) as f:
            return count(json.load(f))
    except (OSError, ValueError):
        return None


def write_keep_last(path, data, kind='jsonl', count=len, ensure_ascii=None, newline=False, encoding=None,
                    shrink='pct', **dump_kw):
    """Write a served or derived output atomically, unless it would replace a non-empty file with an empty one or,
    with `shrink` 'pct' (the default), one that lost more than max_drop() of its rows: then the file is kept,
    "keeping last <path>" goes to stderr and the result is False (a refused write, see the note above: the caller
    decides whether to exit 3). `shrink` None refuses only the empty replacement, for an output that may lose many rows
    legitimately (the bank, when a re-chunk changes many answers' evidence; seed.jsonl; topics' units): a percentage
    rule would keep the old file and then refuse every later merge against it, freezing it.
    `kind` 'jsonl': `data` is the rows, written as write_jsonl writes them (ensure_ascii False unless given); 'json':
    `data` is one object, json.dump(data, **dump_kw) as write_json_atomic writes it (json.dump's own ensure_ascii
    unless given), and `count(obj)` is its size (len by default).
    The bytes written are the same as the plain write's. KEEP_LAST=0 writes whatever it is given."""
    if kind == 'jsonl':
        data = list(data)
    n = len(data) if kind == 'jsonl' else count(data)
    if keep_last_on():
        old = _count_file(path, kind, count)
        if old is not None and shrinks(old, n, shrink):
            rule = 'output empty' if shrink is None else 'input empty or shrunk'
            print(f'keeping last {os.path.relpath(path, ROOT)}: {n} rows now against {old} before '
                  f'({rule}; KEEP_LAST=0 writes it anyway)', file=sys.stderr, flush=True)
            return False
    d = os.path.dirname(path)
    if d:
        os.makedirs(d, exist_ok=True)
    if kind == 'jsonl':
        tmp = path + '.tmp'
        write_jsonl(tmp, data, ensure_ascii=bool(ensure_ascii))
        os.replace(tmp, path)
    else:
        if ensure_ascii is not None:
            dump_kw['ensure_ascii'] = ensure_ascii
        write_json_atomic(path, data, newline=newline, encoding=encoding, **dump_kw)
    return True


def keep_last_on_missing_table(fn, *outputs, required_by=()):
    """Run a stage that reads game tables. A missing table (MissingTable) leaves every output as it was and logs which
    ones were kept, and the run goes on (exit 0), so one table left out of an asset update does not stop the update.
    TABLES_REQUIRED=1, or any of the stage's own switches in `required_by` set to 1 (GAMETEXT_REQUIRE_TABLES,
    DATES_REQUIRE_TABLES), raises instead, the behaviour before 2026-10-19."""
    try:
        return fn()
    except MissingTable as e:
        if any(os.environ.get(k) == '1' for k in ('TABLES_REQUIRED', *required_by)):
            raise
        for p in outputs:
            print(f"keeping last {os.path.relpath(p, ROOT)}{'' if os.path.exists(p) else ' (none yet)'}: {e}",
                  file=sys.stderr, flush=True)
        return None


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


def write_json_atomic(path, obj, newline=False, encoding=None, **dump_kw):
    """json.dump(obj, **dump_kw) into `<path>.tmp` (plus a final newline when asked), then a rename over `path`, so a
    reader never sees half a file. `encoding` None is the platform default, as open() without one."""
    tmp = path + '.tmp'
    with open(tmp, 'w', encoding=encoding) as f:
        json.dump(obj, f, **dump_kw)
        if newline:
            f.write('\n')
    os.replace(tmp, path)


def sha_file(path):
    """The sha256 hex digest of a file's bytes, read in 1 MiB blocks."""
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for b in iter(lambda: f.read(1 << 20), b''):
            h.update(b)
    return h.hexdigest()


def load_json(path):
    return json.load(open(path))


_STALE_SAID = set()


def stale_tables(excel=None):
    """{table name: since} of the tables the unpacker skipped and kept from an earlier extract, from
    `<gamedata>/.skipped.json` ({"tables": [{"table", "since", "resVersion", "previousOutput"}]}, backend
    core/gamedata/stale.rs). {} without the file or when it cannot be read."""
    p = os.path.join(os.path.dirname(os.path.abspath(excel or EXCEL)), '.skipped.json')
    try:
        return {t['table']: t.get('since') for t in json.load(open(p)).get('tables') or [] if t.get('table')}
    except (OSError, ValueError, AttributeError, TypeError):
        return {}


def excel_table(name, excel=None):
    """A game data table from excel/ (TREVOR_GAMEDATA's when set, or `excel`), decoded. A table the unpacker kept from
    an earlier extract is logged once per run ("stale <table> kept since <date>"); a missing one raises MissingTable."""
    path = os.path.join(excel or EXCEL, name)
    base = name[:-5] if name.endswith('.json') else name
    since = stale_tables(excel).get(base)
    if not os.path.exists(path):
        raise MissingTable(f'game table {name} is missing from {excel or EXCEL}'
                           + (f' (skipped by the unpacker since {since})' if since else ''))
    if since and base not in _STALE_SAID:
        _STALE_SAID.add(base)
        print(f'stale {base} kept since {since}', file=sys.stderr, flush=True)
    return load_json(path)


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
    ct = excel_table('character_table.json')['Characters']
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


def model_id(server, timeout):
    """The model the llama-server at `server` has loaded: the file name of the first id /v1/models lists."""
    with urllib.request.urlopen(f'{server}/v1/models', timeout=timeout) as r:
        return os.path.basename(json.load(r)['data'][0]['id'])


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
