#!/usr/bin/env python3
"""The nightly re-answer (design/trevor-self-contained-deploy.md section 4 priority 2, step 6; 2026-10-19).

Yesterday's live questions, which a reader got from the `cpu` rung (E2B at the `mid` profile) or as passages only,
are answered again at the `full` profile on the stage runner's model (the 12B) and join the bank, so the next reader
of the story gets the better answer as a panel suggestion. Three subcommands, run in this order:

  collect [--since HOURS] [--min-asks N] [--out FILE] [--server en]
      Reads the asks of the last HOURS (24) from Postgres through `psql` (stdlib only: no Postgres client library is
      available to the scripts, and the backend has no route that lists asks with their rung; `GET
      /api/trevor/worker/popular` has neither the rung nor a time window). An ask counts when its job (the deferred
      one of an overflowed ask, else its own) ran on the `cpu` or `passages` rung; a cache hit (no job) and a `cf`
      answer do not. The asks are de-duplicated by the backend's normalized question (`qkey`, the hash of normQ)
      across every era: one row per question, its most-asked wording as `question`, the other wordings as `aliases`,
      `asks` and `askers` (distinct accounts), the eras it was asked under (`eras`, with `spoilers` safe, all or
      mixed) and the rungs it got. Rows accumulate in artifacts/bank/reanswer/questions.jsonl by qid (`r` + the
      first 16 hex of the qkey): a rerun over an overlapping window counts only asks newer than a row's `lastAsked`,
      so `asks` never double counts (`askers` is the largest distinct count of one window, a lower bound: no account
      id is stored, because the bank travels in the corpus snapshot). FILE (artifacts/bank/reanswer/pending.jsonl)
      gets every row asked at least N times that has no answer yet, in the shape `ask --batch` and
      `answer_bank.py answer` read (qid, question, plus the fields above).
      DSN: REANSWER_DSN, else TREVOR_DATABASE_URL, else DATABASE_URL; its password goes to psql in PGPASSWORD, never
      on the command line. Read only; it writes nothing to the database.
  answer [FILE] [--profile full] [--model PATH] [-- ASK_FLAGS...]
      `target/release/ask --corpus CORPUS --no-spawn --server SERVER --profile full --batch FILE --out
      artifacts/bank/reanswer/answers.jsonl`, as answer_bank.py stage_answer runs it (same CORPUS and SERVER env),
      resumable on qid. TREVOR_LLM_LOCK becomes --lock, TREVOR_RETRIEVAL --retrieval, REANSWER_ASK_FLAGS and the
      words after `--` are appended. Afterwards every new answer gets its evidence row in
      artifacts/bank/reanswer/evidence.jsonl: the text sha of each cited chunk now (answer_bank.evidence_shas).
      PLAN=1 prints `plan N` (pending questions without an answer) and runs nothing, as the stage runner expects.
  merge
      answer_bank.py merge. Its merge folds these answers in on every run (answer_bank.stage_merge calls
      `bank_rows` below), so the nightly update's own merge keeps them as well; the keep rules are answer_bank's:
      the first sentence does not decline (answer_bank.DECLINE), it cites something, and every cited chunk's sha
      still matches its evidence row (an answer without one is dropped, not kept unverified). Each kept row is a
      bank `ask` entry with `source: "reanswer"`, `asks` and `askers`, so panels.py and publish.py carry it like any
      other bank row; nothing here writes a backend table.

The horizon rule. `ask --batch` takes no horizon, so every answer is made with no spoiler limit (the NO_LIMIT
bucket, eraPos 2147483647), whatever horizons the live asks carried. The bank entry's gate is what keeps it
spoiler-safe: `gateStories` covers every story the answer cited AND every story among the passages the model was
shown (answer_bank merges gate on the cited stories and the question's own story; a live question has no own story,
and a no-limit answer may draw on a passage it does not cite). panels.py shows an entry only to a reader who has
cleared every gate story, and an id it cannot date (a dossier or profile passage) gates it to never. On the 942
answers of artifacts/bank/answers.jsonl, 67 (7.1%) had such a passage against 10 among the cited ones alone.
answer_bank.within_gate (a citation at or before the question's own story) needs that own story and does not apply.

Open decision 4 of the design doc: whether only questions asked more than once are re-answered. The default is
`--min-asks 2` (REANSWER_MIN_ASKS overrides it, the flag overrides both) since 2026-10-19: a bank row becomes a
public panel suggestion, so one reader's wording must not become one, and it keeps the 12B's hours for questions
that recur. `--min-asks 1` is the behaviour before.

Kill switch: REANSWER=0 makes every subcommand print why and do nothing, and answer_bank.py merge leaves these rows
out (bank.jsonl is then byte-identical to the merge before this step, as it is when no re-answer has run). Not
wiring the step into the runner is the same.
Env: TREVOR_BANK_DIR (artifacts/bank, answer_bank's), CORPUS, SERVER, ASK_BIN (target/release/ask).
"""
import argparse, collections, datetime, json, os, re, subprocess, sys, urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and answer_bank.py sit beside the scripts
import common  # noqa: E402
from common import ROOT, read_jsonl  # noqa: E402

NO_LIMIT_ERA = 2**31 - 1  # backend services::trevor::NO_LIMIT_ERA: spoiler mode and internal jobs
OFF = os.environ.get('REANSWER') == '0'
RUNGS = ('cpu', 'passages')
MAX_ALIASES = 5


def paths(bank_dir):
    d = os.path.join(bank_dir, 'reanswer')
    return {'dir': d, 'store': os.path.join(d, 'questions.jsonl'), 'pending': os.path.join(d, 'pending.jsonl'),
            'answers': os.path.join(d, 'answers.jsonl'), 'ledger': os.path.join(d, 'evidence.jsonl'),
            'history': os.path.join(d, 'history.jsonl')}


def show(path):
    rel = os.path.relpath(path, ROOT)
    return path if rel.startswith('..') else rel


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')


# ---------------------------------------------------------------- collect

def collect_sql(server, hours):
    """The asks of the last `hours` on `server` answered on the cpu or passages rung, one JSON object per line."""
    if not re.fullmatch(r'[a-z]{2,8}', server):
        raise ValueError(f'bad server {server!r}')
    hours = int(hours)
    return f"""
SELECT row_to_json(t) FROM (
  SELECT k.qkey, k.normq AS "normQ", k.question, k.era_pos AS "eraPos", k.user_id::text AS "userId",
         to_char(k.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS "createdAt",
         j.rung, j.status
  FROM trevor_asks k
  JOIN trevor_jobs j ON j.id = COALESCE(k.deferred_job_id, k.job_id)
  WHERE k.server = '{server}' AND k.created_at > now() - make_interval(hours => {hours})
    AND j.rung IN ('cpu', 'passages')
  ORDER BY k.created_at, k.id
) t;"""


def dsn():
    for k in ('REANSWER_DSN', 'TREVOR_DATABASE_URL', 'DATABASE_URL'):
        if os.environ.get(k):
            return os.environ[k]
    sys.exit('reanswer collect: set REANSWER_DSN (or TREVOR_DATABASE_URL, DATABASE_URL) to the backend database')


def split_password(conn):
    """(the DSN without its password, the password or None): a password on psql's argv is visible to every local user
    in `ps`, so it travels in PGPASSWORD instead. Both DSN forms: a URI (postgres:// or postgresql://, the password
    percent-decoded) and keyword=value pairs (password=... or password='...')."""
    u = urllib.parse.urlsplit(conn)
    if u.scheme in ('postgres', 'postgresql'):
        password = None if u.password is None else urllib.parse.unquote(u.password)
        q = urllib.parse.parse_qsl(u.query, keep_blank_values=True)
        password = next((v for k, v in q if k == 'password'), password)  # ?password= wins, as in libpq
        if password is None:
            return conn, None
        userinfo, _, host = u.netloc.rpartition('@')
        user = userinfo.partition(':')[0]
        query = urllib.parse.urlencode([(k, v) for k, v in q if k != 'password'])
        return urllib.parse.urlunsplit(u._replace(netloc=f'{user}@{host}' if user else host, query=query)), password
    m = re.search(r"(?:^|\s)password\s*=\s*('(?:[^'\\]|\\.)*'|\S*)", conn)
    if not m:
        return conn, None
    v = m.group(1)
    if v.startswith("'"):
        v = re.sub(r'\\(.)', r'\1', v[1:-1])
    return (conn[:m.start()] + conn[m.end():]).strip(), v


def psql_command(conn):
    """psql's argv and environment for `conn`: the password, if any, in PGPASSWORD, never on the argv."""
    conn, password = split_password(conn)
    env = dict(os.environ)
    if password is not None:
        env['PGPASSWORD'] = password
    return ['psql', '-X', '-q', '-A', '-t', '-v', 'ON_ERROR_STOP=1', conn], env


def fetch_asks(server, hours):
    argv, env = psql_command(dsn())
    out = subprocess.run(argv, env=env, input=collect_sql(server, hours), capture_output=True, text=True, timeout=300)
    if out.returncode:
        sys.exit(f'reanswer collect: psql exited {out.returncode}: {out.stderr.strip()[-500:]}')
    return [json.loads(l) for l in out.stdout.splitlines() if l.strip()]


def spoiler_mode(eras):
    modes = {'all' if e == NO_LIMIT_ERA else 'safe' for e in eras}
    return modes.pop() if len(modes) == 1 else 'mixed'


def fold(asks, store):
    """Fold asks (rows of collect_sql) into the store rows, de-duplicated by qkey. A known question counts only the
    asks after its `lastAsked`. Returns (rows sorted by qid, counts)."""
    by = {r['qid']: dict(r) for r in store}
    groups = collections.defaultdict(list)
    for a in asks:
        groups[a['qkey']].append(a)
    n = collections.Counter()
    for qkey, g in groups.items():
        qid = 'r' + qkey[:16]
        old = by.get(qid)
        if old:
            g = [a for a in g if a['createdAt'] > old['lastAsked']]
            if not g:
                continue
        g.sort(key=lambda a: a['createdAt'])
        texts = collections.Counter(a['question'] for a in g)
        latest = {a['question']: i for i, a in enumerate(g)}
        ranked = sorted(texts, key=lambda t: (-texts[t], -latest[t]))  # most asked, then most recent
        eras = sorted({a['eraPos'] for a in g} | set(old['eras'] if old else ()))
        row = old or {'qid': qid, 'question': ranked[0], 'normQ': g[-1]['normQ'], 'qkey': qkey, 'aliases': [],
                      'asks': 0, 'askers': 0, 'firstAsked': g[0]['createdAt']}
        row['aliases'] = (row['aliases'] + [t for t in ranked if t != row['question'] and t not in row['aliases']]
                          )[:MAX_ALIASES]
        row['asks'] += len(g)
        row['askers'] = max(row['askers'], len({a['userId'] for a in g}))
        row['eras'] = eras
        row['spoilers'] = spoiler_mode(eras)
        row['rungs'] = sorted({a['rung'] for a in g} | set(row.get('rungs') or ()))
        row['lastAsked'] = g[-1]['createdAt']
        by[qid] = row
        n['updated' if old else 'new'] += 1
    return [by[k] for k in sorted(by)], n


def pending(store, answered, min_asks):
    """The store rows asked at least `min_asks` times that have no answer yet."""
    return [r for r in store if r['asks'] >= min_asks and r['qid'] not in answered]


def answered_qids(p):
    return {r['qid'] for r in read_jsonl(p['answers'])}


def collect(a, ab):
    p = paths(ab.OUT)
    asks = fetch_asks(a.server, a.since)
    os.makedirs(p['dir'], exist_ok=True)
    rows, n = fold(asks, read_jsonl(p['store']))
    common.replace_jsonl(p['store'], rows)
    todo = pending(rows, answered_qids(p), a.min_asks)
    out = a.out or p['pending']
    common.replace_jsonl(out, todo)
    res = {'at': stamp(), 'step': 'collect', 'since': a.since, 'minAsks': a.min_asks, 'asks': len(asks),
           'questions': len({x['qkey'] for x in asks}), 'new': n['new'], 'updated': n['updated'],
           'stored': len(rows), 'pending': len(todo)}
    common.append_jsonl(p['history'], res)
    print(f"reanswer collect: {len(asks)} asks in {a.since} h on the cpu or passages rung, {res['questions']} distinct "
          f"questions ({n['new']} new, {n['updated']} asked again); {len(rows)} stored, {len(todo)} to answer "
          f"(asked at least {a.min_asks} times) -> {show(out)}", flush=True)
    return res


# ---------------------------------------------------------------- answer

def ask_cmd(ab, batch, out, profile, model, extra, env=os.environ):
    cmd = [env.get('ASK_BIN') or os.path.join(ROOT, 'target', 'release', 'ask'), '--corpus', ab.CORPUS, '--no-spawn',
           '--server', ab.SERVER, '--profile', profile, '--batch', batch, '--out', out]
    if model:
        cmd += ['--model', model]
    if env.get('TREVOR_LLM_LOCK'):
        cmd += ['--lock', env['TREVOR_LLM_LOCK']]
    if env.get('TREVOR_RETRIEVAL'):
        cmd += ['--retrieval', env['TREVOR_RETRIEVAL']]
    return cmd + (env.get('REANSWER_ASK_FLAGS') or '').split() + list(extra)


def answer(a, ab):
    p = paths(ab.OUT)
    batch = a.file or p['pending']
    have = answered_qids(p)
    todo = [r for r in read_jsonl(batch) if r['qid'] not in have]
    if os.environ.get('PLAN'):
        print(f'reanswer: plan {len(todo)}', flush=True)
        return 0
    if not todo:
        print('reanswer answer: nothing to answer', flush=True)
        return 0
    os.makedirs(p['dir'], exist_ok=True)
    cmd = ask_cmd(ab, batch, p['answers'], a.profile, a.model, a.ask_flags)
    print('$ ' + ' '.join(cmd), flush=True)
    rc = subprocess.call(cmd, cwd=ROOT)
    made = record_evidence(ab, p, a.profile)
    common.append_jsonl(p['history'], {'at': stamp(), 'step': 'answer', 'todo': len(todo), 'answered': made,
                                       'profile': a.profile, 'rc': rc})
    print(f'reanswer answer: {made} of {len(todo)} answered at the {a.profile} profile; ask exited {rc}', flush=True)
    return rc


def record_evidence(ab, p, profile):
    """An evidence row (the sha of every cited chunk now) for each answer that has none yet."""
    ledger = {r['qid'] for r in read_jsonl(p['ledger'])}
    new = [r for r in read_jsonl(p['answers']) if r['qid'] not in ledger]
    if not new:
        return 0
    cur = ab.evidence_shas(os.path.join(ROOT, 'artifacts'))
    for r in new:
        common.append_jsonl(p['ledger'], {'qid': r['qid'], 'shas': {c: cur.get(c) for c in sorted(set(r['cited']))},
                                          'made': 'reanswer', 'profile': profile, 'at': stamp()})
    return len(new)


# ---------------------------------------------------------------- merge (called by answer_bank.stage_merge)

def first_sentence(text):
    return re.split(r'(?<=[.!?])\s+', text.strip(), maxsplit=1)[0]


def verdict(ans, q, ev, cur, decline):
    """None to keep the answer, else why it is dropped (answer_bank's keep rules, evidence required)."""
    if q is None:
        return 'question gone'
    if decline.search(first_sentence(ans['answer'])):
        return 'declined'
    if not ans['cited']:
        return 'uncited'
    if ev is None:
        return 'no evidence row'
    moved = [c for c, h in ev['shas'].items() if h is None or cur.get(c) != h]
    if moved:
        return 'evidence vanished' if any(c not in cur for c in moved) else 'evidence changed'
    return None


def bank_row(ans, q, gate, sp):
    """The bank entry: gated on every cited story and every story among the passages it was shown."""
    seen = sorted({c.split('#')[0] for c in list(ans['cited']) + list(ans['passages'])})
    chars = [s[len('archive_'):] for s in seen if s.startswith('archive_')]
    return {'id': f"ask:{ans['qid']}", 'kind': 'ask', 'question': q['question'], 'aliases': list(q.get('aliases') or []),
            'answer': ans['answer'], 'sources': ans['cited'], 'passages': ans['passages'], **gate(seen, chars, sp),
            'source': 'reanswer', 'asks': q['asks'], 'askers': q['askers']}


def bank_rows(bank_dir, sp, gate, decline, shas_fn):
    """(bank entries, counts) for the re-answered questions under `bank_dir`; ([], {}) with REANSWER=0."""
    if OFF:
        return [], {}
    p = paths(bank_dir)
    answers = {r['qid']: r for r in read_jsonl(p['answers'])}  # the newest answer of a qid wins
    if not answers:
        return [], {}
    Q = {r['qid']: r for r in read_jsonl(p['store'])}
    ledger = {r['qid']: r for r in read_jsonl(p['ledger'])}
    cur = shas_fn()
    rows, why = [], collections.Counter()
    for qid in sorted(answers):
        v = verdict(answers[qid], Q.get(qid), ledger.get(qid), cur, decline)
        if v:
            why[v] += 1
        else:
            rows.append(bank_row(answers[qid], Q[qid], gate, sp))
    why['kept'] = len(rows)
    return rows, dict(why)


def merge(ab):
    res = ab.stage_merge()
    common.append_jsonl(paths(ab.OUT)['history'], {'at': stamp(), 'step': 'merge', **res})
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = ap.add_subparsers(dest='cmd', required=True)
    c = sub.add_parser('collect')
    c.add_argument('--since', type=int, default=24, help='hours back (24)')
    c.add_argument('--min-asks', type=int, default=int(os.environ.get('REANSWER_MIN_ASKS') or 2),
                   help='asks a question needs (REANSWER_MIN_ASKS, else 2; open decision 4)')
    c.add_argument('--out', help='the questions file (artifacts/bank/reanswer/pending.jsonl)')
    c.add_argument('--server', default='en')
    w = sub.add_parser('answer')
    w.add_argument('file', nargs='?', help='questions (artifacts/bank/reanswer/pending.jsonl)')
    w.add_argument('--profile', default='full')
    w.add_argument('--model', help='passed to ask as --model')
    sub.add_parser('merge')
    argv = list(sys.argv[1:] if argv is None else argv)
    extra = argv[argv.index('--') + 1:] if '--' in argv else []  # `answer [FILE] -- ASK_FLAGS...`
    a = ap.parse_args(argv[:len(argv) - len(extra) - (1 if '--' in argv else 0)])
    a.ask_flags = extra
    if OFF:
        print(f'REANSWER=0: reanswer {a.cmd} does nothing', flush=True)
        return 0
    ab = common.load_script('answer_bank')
    if a.cmd == 'collect':
        collect(a, ab)
        return 0
    if a.cmd == 'answer':
        return answer(a, ab)
    return merge(ab)


if __name__ == '__main__':
    sys.exit(main())
