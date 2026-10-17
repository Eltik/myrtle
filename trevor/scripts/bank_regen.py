#!/usr/bin/env python3
"""Pre-generated answers that follow game updates (contract v1.1 section 3a and 3c; Ian 2026-10-17: "generate new
answers when there is a new update").

The bank's generated entries (`kind: ask`, scripts/answer_bank.py) are a question drawn from one chunk plus Trevor's
answer to it. After a game update this script finds the chunks that are new or whose text changed, asks a question
only of those, answers the questions with the `mid` profile and lets answer_bank.py merge keep every old entry whose
evidence is unchanged and drop every entry whose evidence changed or vanished. Nothing else is redone.

  detect --prev DIR   compare the chunk text shas of this tree's corpus (artifacts/p3b over artifacts/p4x) with DIR
                      (the previous release's artifacts, trevor-vps-update.sh's live release) and add to the pending
                      list (artifacts/bank/regen/pending.json): every question-eligible chunk that is new or changed
                      (answer_bank.eligible_chunk), and every existing question whose own chunk is unchanged but whose
                      answer cited a changed or vanished chunk (re-answered under its old qid). The first detect also
                      writes the evidence ledger (artifacts/bank/evidence.jsonl) of every answer made before it, from
                      DIR's shas, the corpus those answers were published with. No model.
  plan                what `run` would do (also PLAN=1 run): pending units, by reason and story group, the model calls
                      they cost (1 question + 1 answer each under mid), the entries the merge would drop.
  run                 regenerate pending units, at most REGEN_MAX_UNITS (150) a run, in slices of REGEN_SLICE (10):
                      questions (answer_bank.stage_questions on the slice's chunks), then `ask --batch` with the mid
                      profile, the answers folded into answers.jsonl by qid and their evidence shas into the ledger,
                      the units taken off the pending list; then answer_bank.py merge.
                      --backend auto: Cloudflare inside TREVOR_CF_WINDOW (summaries_cf.py's client, env and window;
                      its spend counted against TREVOR_CF_BANK_MAX_NEURONS, default 9000 minus --cf-spent, what the
                      summaries spent first), else the local llama-server at TREVOR_SUMMARY_LLM_URL until --until
                      (UTC). A unit left over stays pending for the next night.
  warm --version V    contract v1.1 section 3c: GET {backend}/api/trevor/worker/popular?version=V&limit=N (worker key),
                      then POST {backend}/api/trevor/worker/warm with the items as deferred internal jobs.
                      --dry-run DIR writes the POST body to DIR instead of sending it.
  count               pending units, one integer (for the shell script)
  --self-test         the warm-up client against a stand-in backend (request shapes, key, TREVOR_WARM_TOP=0, dry
                      run); no model. detect, run and merge were checked on a simulated update (runbook section 5a)

Kill switches: REGEN_ANSWERS=0 makes every subcommand a no-op that prints why (and answer_bank.py merge the merge
before 2026-10-17, byte for byte); TREVOR_WARM_TOP=0 skips the warm-up.
Env: CORPUS (artifacts/p3b), ASK_BIN (target/release/ask), TREVOR_RETRIEVAL (passed as --retrieval when set),
TREVOR_LLM_LOCK (passed to ask as --lock), REGEN_ASK_FLAGS (extra ask flags), REGEN_RATE (1.0: the share of new
chunks WITHOUT an earlier question that get one, drawn by a hash of the chunk id; changed chunks that had a question
always get one).
"""
import argparse, collections, datetime, json, math, os, re, subprocess, sys, time, urllib.error, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common  # noqa: E402
from common import ROOT, read_jsonl, sha16  # noqa: E402

OUT = os.path.join(ROOT, 'artifacts', 'bank')
REGEN = os.path.join(OUT, 'regen')
PENDING = os.path.join(REGEN, 'pending.json')
LEDGER = os.path.join(OUT, 'evidence.jsonl')
OFF = os.environ.get('REGEN_ANSWERS') == '0'
MAX_UNITS = int(os.environ.get('REGEN_MAX_UNITS', '150'))
SLICE = int(os.environ.get('REGEN_SLICE', '10'))
RATE = float(os.environ.get('REGEN_RATE', '1.0'))
# contract v1.1 section 4: mid = the keyword router, k 8, a 3,600-token passage budget, 250 output tokens, one model
# call. These flags are the eval's (artifacts/answers/eval3way/queue_mid.sh) for an `ask` without --profile mid.
MID_FLAGS = ['--router', 'keywords', '--k', '8', '--max-context-tokens', '3600', '--n-predict', '250',
             '--no-typed-fallback', '--no-answer-extend', '--no-answer-check', '--no-operator-list-check']
ANSWER_OUT_TOKENS = 250


def now_utc():
    return datetime.datetime.now(datetime.timezone.utc)


def stamp():
    return now_utc().strftime('%Y-%m-%dT%H:%M:%SZ')


def ab():
    return common.load_script('answer_bank')


def corpus_shas(art, corpus_rel):
    """chunk id -> (sha16 of text, row) for one tree: P4x first, the question corpus over it."""
    out = {}
    for d in ('p4x', corpus_rel):
        p = os.path.join(art, d, 'chunks.jsonl')
        if os.path.exists(p):
            for r in read_jsonl(p):
                out[r['chunkId']] = (sha16(r['text']), r)
    return out


def load_pending():
    if os.path.exists(PENDING):
        return json.load(open(PENDING))
    return {'chunks': {}, 'reanswer': {}}


def save_pending(p):
    os.makedirs(REGEN, exist_ok=True)
    common.write_json_atomic(PENDING, p, indent=1, sort_keys=True)


def corpus_rel():
    return os.path.relpath(ab().CORPUS, os.path.join(ROOT, 'artifacts'))


# ---------------------------------------------------------------- detect
def detect(prev_art, cur_art=None, verbose=True):
    """Add the changes between prev_art and this tree to the pending list. Returns the counts."""
    a = ab()
    rel = corpus_rel()
    cur_art = cur_art or os.path.join(ROOT, 'artifacts')
    t0 = time.time()
    prev = corpus_shas(prev_art, rel)
    cur = corpus_shas(cur_art, rel)
    if not prev:
        raise SystemExit(f'detect: no corpus under {prev_art} ({rel}/chunks.jsonl, p4x/chunks.jsonl)')
    new = {c for c in cur if c not in prev}
    changed = {c for c in cur if c in prev and prev[c][0] != cur[c][0]}
    vanished = {c for c in prev if c not in cur}
    moved = new | changed | vanished
    Q = {r['qid']: r for r in read_jsonl(os.path.join(OUT, 'questions.jsonl'))}
    q_by_chunk = {r['chunkId']: r for r in Q.values()}
    answers = read_jsonl(os.path.join(OUT, 'answers.jsonl'))
    # The evidence ledger: answers made before the first detect get their shas from prev_art, the corpus they were
    # published with. Rows already there are never rewritten.
    ledger = {r['qid']: r for r in read_jsonl(LEDGER)}
    back = []
    for ans in answers:
        q = Q.get(ans['qid'])
        if q is None or ans['qid'] in ledger:
            continue
        ids = sorted({q['chunkId'], *ans['cited']})
        back.append({'qid': ans['qid'], 'shas': {c: prev[c][0] if c in prev else None for c in ids},
                     'made': 'backfill', 'at': stamp()})
    for r in back:
        common.append_jsonl(LEDGER, r)
        ledger[r['qid']] = r
    gold = a.gold_stories()
    pend = load_pending()
    added = collections.Counter()
    for c in sorted(new | changed):
        sha, row = cur[c]
        if rel != 'p4x' and c not in corpus_ids(cur_art, rel):
            continue  # a P4x-only unit: bank questions come from the question corpus alone
        if not a.eligible_chunk(row, gold):
            continue
        had = c in q_by_chunk
        if not had and RATE < 1.0 and int(sha16(c), 16) % 10_000 >= RATE * 10_000:
            continue
        why = 'new' if c in new else ('changed, had a question' if had else 'changed')
        old = pend['chunks'].get(c)
        pend['chunks'][c] = {'sha': sha, 'why': 'new' if old and old['why'] == 'new' else why,
                             'storyId': row['storyId'], 'groupId': row['groupId'],
                             'since': old['since'] if old else stamp()}
        added[why] += 1
    # Re-answer: the question's own chunk is unchanged, but its answer rests on a chunk that moved.
    for ans in answers:
        q = Q.get(ans['qid'])
        if q is None or q['chunkId'] in moved:
            continue
        ev = ledger.get(ans['qid'])
        if ev and any(c in moved for c in ev['shas']) and ans['qid'] not in pend['reanswer']:
            pend['reanswer'][ans['qid']] = {'chunkId': q['chunkId'], 'storyId': q['storyId'], 'groupId': q['groupId'],
                                             'since': stamp()}
            added['reanswer'] += 1
    gone = [c for c in pend['chunks'] if c not in cur]
    for c in gone:
        del pend['chunks'][c]
    gone_q = [k for k, v in pend['reanswer'].items() if k not in Q or v['chunkId'] not in cur]
    for k in gone_q:
        del pend['reanswer'][k]
    save_pending(pend)
    res = {'at': stamp(), 'prev': prev_art, 'chunks': {'new': len(new), 'changed': len(changed), 'vanished': len(vanished)},
           'pendingAdded': dict(added), 'pendingDropped': len(gone) + len(gone_q), 'ledgerBackfilled': len(back),
           'pending': {'chunks': len(pend['chunks']), 'reanswer': len(pend['reanswer'])},
           'seconds': round(time.time() - t0, 1)}
    common.append_jsonl(os.path.join(REGEN, 'history.jsonl'), {'kind': 'detect', **res})
    if verbose:
        print(json.dumps(res), flush=True)
    return res


_IDS = {}


def corpus_ids(art, rel):
    k = (art, rel)
    if k not in _IDS:
        p = os.path.join(art, rel, 'chunks.jsonl')
        _IDS[k] = {r['chunkId'] for r in read_jsonl(p)} if os.path.exists(p) else set()
    return _IDS[k]


# ---------------------------------------------------------------- plan
def units_in_order(pend):
    """Run order: re-answers (cheapest: no question), then changed chunks that had a question (their old entry is
    already dropped), then the rest, oldest pending first, then story order."""
    pos = ab().gate_positions()
    rank = {'reanswer': 0, 'changed, had a question': 1, 'changed': 2, 'new': 3}
    us = [('reanswer', k, v) for k, v in pend['reanswer'].items()]
    us += [(v['why'], k, v) for k, v in pend['chunks'].items()]
    us.sort(key=lambda u: (rank.get(u[0], 9), u[2].get('since', ''), pos.get(u[2].get('storyId'), 10**9), u[1]))
    return us


def plan(verbose=True):
    pend = load_pending()
    us = units_in_order(pend)
    why = collections.Counter(u[0] for u in us)
    groups = collections.Counter(u[2].get('groupId') or u[2].get('storyId') for u in us)
    run_now = us[:MAX_UNITS]
    calls = sum(1 if u[0] == 'reanswer' else 2 for u in run_now)
    res = {'pending': len(us), 'byReason': dict(why), 'thisRun': len(run_now), 'cap': MAX_UNITS,
           'modelCalls': calls, 'topGroups': groups.most_common(8)}
    if verbose:
        print(f"bank regen plan: {len(us)} pending units ({', '.join(f'{k} {v}' for k, v in sorted(why.items())) or 'none'}); "
              f'this run {len(run_now)} (cap REGEN_MAX_UNITS {MAX_UNITS}), {calls} model calls '
              '(1 question + 1 mid answer a chunk, 1 answer a re-answer)', flush=True)
        for g, n in groups.most_common(8):
            print(f'  {g}: {n}', flush=True)
        for u in run_now[:int(os.environ.get('PLAN_SHOW', '12'))]:
            print(f'  {u[0]}: {u[1]}', flush=True)
    return res


# ---------------------------------------------------------------- run
def ask_bin():
    return os.environ.get('ASK_BIN') or os.path.join(ROOT, 'target', 'release', 'ask')


def mid_flags():
    """`--profile mid` when this ask has it, else the eval's equivalent flags."""
    try:
        h = subprocess.run([ask_bin(), '--help'], capture_output=True, text=True, timeout=60).stdout
    except (OSError, subprocess.TimeoutExpired):
        h = ''
    m = re.search(r'--profile <PROFILE>.*?\[possible values: ([^\]]*)\]', h, re.S)
    if m and re.search(r'\bmid\b', m.group(1)):
        return ['--profile', 'mid'], 'mid'
    # TODO(trevor): drop this branch once every deployed ask has --profile mid (the worker agent, 2026-10-17).
    return ['--profile', 'full'] + MID_FLAGS, 'mid-flags'


class Runner:
    def __init__(self, a, env=os.environ):
        cf_mod = common.load_script('summaries_cf')
        self.cf_mod = cf_mod
        self.backends = []
        if a.backend in ('auto', 'cf'):
            e = dict(env)
            cap = int(env.get('TREVOR_CF_BANK_MAX_NEURONS') or 9000) - int(a.cf_spent or 0)
            e['TREVOR_CF_SUMMARY_MAX_NEURONS'] = str(max(cap, 0))
            self.backends.append(cf_mod.CF(e))
        if a.backend in ('auto', 'local'):
            self.local_url = env.get('TREVOR_SUMMARY_LLM_URL') or 'http://127.0.0.1:8082'
            self.backends.append(cf_mod.Local(self.local_url, cf_mod.next_at(now_utc(), a.until)))
        self.env = env
        self.counts = collections.Counter()
        self.neurons = 0

    def pick(self):
        for b in self.backends:
            if b.usable():
                return b
        return None

    def chat_fn(self, b):
        def f(system, user, max_tokens):
            text, ti, to, model, n = b.chat(system, user, max_tokens)
            self.counts['questionCalls'] += 1
            if n:
                self.neurons += n
            return text
        return f

    def ask_cmd(self, b, batch, out, flags):
        cmd = [ask_bin(), '--corpus', ab().CORPUS, '--batch', batch, '--out', out] + flags
        if self.env.get('TREVOR_RETRIEVAL'):
            cmd += ['--retrieval', self.env['TREVOR_RETRIEVAL']]
        if self.env.get('TREVOR_LLM_LOCK'):
            cmd += ['--lock', self.env['TREVOR_LLM_LOCK']]
        if b.name == 'cf':
            cmd += ['--backend', 'openai']
        else:
            cmd += ['--backend', 'llama', '--server', b.url, '--no-spawn']
        cmd += (self.env.get('REGEN_ASK_FLAGS') or '').split()
        return cmd


def fold_answers(rows):
    """Replace answers.jsonl rows by qid with `rows` (append the new qids)."""
    path = os.path.join(OUT, 'answers.jsonl')
    by = {r['qid']: r for r in rows}
    old = read_jsonl(path)
    out = [by.pop(r['qid'], r) for r in old]
    out += [r for r in rows if r['qid'] in by]
    common.write_jsonl(path, out)


def run(a):
    if OFF:
        print('REGEN_ANSWERS=0: no regeneration', flush=True)
        return {'skipped': 'REGEN_ANSWERS=0'}
    p = plan(verbose=True)
    if os.environ.get('PLAN') == '1' or a.plan:
        return p
    t_start = time.time()
    r = Runner(a)
    flags, profile = mid_flags()
    print(f'bank regen: ask profile {profile}: {" ".join(flags)}', flush=True)
    rel = corpus_rel()
    status = {'at': stamp(), 'profile': profile, 'units': 0, 'skipped': 0, 'questions': 0, 'answered': 0,
              'byBackend': {}, 'stopped': None}
    pend = load_pending()
    todo = units_in_order(pend)[:MAX_UNITS]
    os.makedirs(REGEN, exist_ok=True)
    qpath, apath = os.path.join(OUT, 'questions.jsonl'), os.path.join(OUT, 'answers.jsonl')
    timings, i, tries = [], 0, 0
    while i < len(todo):
        sl = todo[i:i + SLICE]
        b = r.pick()
        if b is None:
            status['stopped'] = '; '.join(f'{x.name}: {x.stopped}' for x in r.backends)
            break
        t0 = time.time()
        chunk_ids = {u[1] for u in sl if u[0] != 'reanswer'}
        q_rows = []
        if chunk_ids:
            try:
                q_rows = ab().stage_questions(only=chunk_ids, chat_fn=r.chat_fn(b), show=False)
            except r.cf_mod.Quota:
                continue  # the cf backend is done for the run: the same slice again on the next backend
        status['questions'] += len(q_rows)
        Q = {q['qid']: q for q in read_jsonl(qpath)}
        q_by_chunk = {q['chunkId']: q for q in Q.values()}
        have = {x['qid'] for x in read_jsonl(apath)}
        unit_q = {}  # unit id -> the question that answers it
        for u in sl:
            if u[0] == 'reanswer':
                if u[1] in Q:
                    unit_q[u[1]] = Q[u[1]]
            else:
                q = q_by_chunk.get(u[1])
                if q and q.get('inputSha') == u[2]['sha']:
                    unit_q[u[1]] = q
        batch_rows = [q for k, q in unit_q.items() if q['qid'] not in have or pend['reanswer'].get(k)]
        out = os.path.join(REGEN, 'answers.slice.jsonl')
        if os.path.exists(out):
            os.remove(out)
        answered, rc = [], 0
        if batch_rows:
            batch = os.path.join(REGEN, 'questions.slice.jsonl')
            common.write_jsonl(batch, batch_rows)
            with open(os.path.join(REGEN, 'ask.log'), 'a') as log:
                rc = subprocess.call(r.ask_cmd(b, batch, out, flags), cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            answered = read_jsonl(out) if os.path.exists(out) else []
            if rc != 0:
                print(f'bank regen: ask exited {rc} after {len(answered)} of {len(batch_rows)} on {b.name}; '
                      f'see {os.path.join(REGEN, "ask.log")}', flush=True)
                b.stopped = b.stopped or f'ask exited {rc}'
        cur = corpus_shas(os.path.join(ROOT, 'artifacts'), rel)
        for ans in answered:
            q = Q[ans['qid']]
            ids = sorted({q['chunkId'], *ans['cited']})
            common.append_jsonl(LEDGER, {'qid': ans['qid'], 'shas': {c: cur[c][0] if c in cur else None for c in ids},
                                         'made': 'regen', 'profile': profile, 'backend': b.name, 'at': stamp()})
            if b.name == 'cf':
                # Cloudflare's usage is not in ask's batch row: its prompt tokens plus the whole output budget.
                n = math.ceil(ans.get('promptTokens', 0) * 9091 / 1e6 + ANSWER_OUT_TOKENS * 27273 / 1e6)
                r.neurons += n
                b.spent += n
        fold_answers(answered)
        r.counts['answerCalls'] += len(answered)
        have |= {x['qid'] for x in answered}
        done_now = {x['qid'] for x in answered}
        for u in sl:
            q = unit_q.get(u[1])
            if u[0] == 'reanswer':
                if q is None or q['qid'] in done_now:
                    pend['reanswer'].pop(u[1], None)
                    status['units' if q else 'skipped'] += 1
            elif q is None and u[1] not in chunk_ids:
                pass
            elif q is None:
                # Not eligible in this corpus after all (or the model wrote nothing): off the list.
                pend['chunks'].pop(u[1], None)
                status['skipped'] += 1
            elif q['qid'] in have:
                pend['chunks'].pop(u[1], None)
                status['units'] += 1
        save_pending(pend)
        status['answered'] += len(answered)
        status['byBackend'][b.name] = status['byBackend'].get(b.name, 0) + len(answered)
        dt = time.time() - t0
        timings.append({'units': len(sl), 'questions': len(q_rows), 'answers': len(answered), 'seconds': round(dt, 1),
                        'backend': b.name})
        print(f'{time.strftime("%H:%M:%S")} bank regen slice {len(timings)}: {len(q_rows)} questions, '
              f'{len(answered)} answers on {b.name} in {dt:.0f} s', flush=True)
        if rc != 0:
            tries += 1
            if tries < 3:
                continue  # the same slice on the next usable backend
        i += SLICE
        tries = 0
    status['merge'] = ab().stage_merge()
    status.update(questionCalls=r.counts['questionCalls'], answerCalls=r.counts['answerCalls'], neurons=r.neurons,
                  seconds=round(time.time() - t_start, 1), slices=timings,
                  pendingLeft=len(pend['chunks']) + len(pend['reanswer']))
    if status['units']:
        status['secondsPerUnit'] = round(sum(t['seconds'] for t in timings) / status['units'], 1)
        status['modelCallsPerUnit'] = round((status['questionCalls'] + status['answerCalls']) / status['units'], 2)
    common.append_jsonl(os.path.join(REGEN, 'history.jsonl'), {'kind': 'run', **{k: v for k, v in status.items() if k != 'slices'}})
    print(json.dumps({k: v for k, v in status.items() if k != 'slices'}), flush=True)
    if a.status:
        common.write_json_atomic(a.status, status, indent=1)
    return status


# ---------------------------------------------------------------- warm-up (contract v1.1 section 3c)
def http(method, url, key, body=None, timeout=60):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(url, data, {'content-type': 'application/json', 'x-trevor-worker-key': key},
                                 method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            raw = r.read()
            return r.status, (json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:
        return e.code, {'error': e.read()[:300].decode('utf-8', 'replace')}
    except (urllib.error.URLError, OSError, ValueError) as e:
        return 0, {'error': f'{type(e).__name__}: {e}'}


def warm(a, env=os.environ):
    limit = int(a.limit if a.limit is not None else env.get('TREVOR_WARM_TOP') or 100)
    if limit <= 0:
        print('warm-up: TREVOR_WARM_TOP=0, skipped', flush=True)
        return 0
    key = env.get('TREVOR_WORKER_KEY') or ''
    base = (a.backend_url or env.get('TREVOR_BACKEND') or 'http://127.0.0.1:3060').rstrip('/')
    if not key and not a.dry_run:
        print('warm-up: TREVOR_WORKER_KEY is not set', file=sys.stderr)
        return 1
    url = f'{base}/api/trevor/worker/popular?version={urllib.request.quote(a.version)}&limit={limit}'
    code, res = http('GET', url, key)
    if code != 200 or not isinstance(res, dict):
        print(f'warm-up: GET popular -> {code} {res}', file=sys.stderr)
        return 1
    items = [{'question': i['question'], 'eraPos': i.get('eraPos', 0)} for i in res.get('items') or []
             if isinstance(i, dict) and i.get('question')][:limit]
    body = {'version': a.version, 'items': items}
    if a.dry_run:
        os.makedirs(a.dry_run, exist_ok=True)
        with open(os.path.join(a.dry_run, 'warm.json'), 'w') as f:
            json.dump(body, f, indent=1)
        print(f'warm-up (dry run): {len(items)} questions -> {a.dry_run}/warm.json', flush=True)
        return 0
    if not items:
        print('warm-up: nothing to warm (every popular question was carried over)', flush=True)
        return 0
    code, res = http('POST', f'{base}/api/trevor/worker/warm', key, body)
    if code == 409:
        print(f'warm-up: {a.version} is no longer the active version; skipped', flush=True)
        return 0
    if code != 200:
        print(f'warm-up: POST warm -> {code} {res}', file=sys.stderr)
        return 1
    print(f'warm-up: {len(items)} questions sent: {json.dumps(res)}', flush=True)
    return 0


# ---------------------------------------------------------------- self-test
def self_test():
    """warm() against a stand-in backend."""
    import http.server, shutil, tempfile, threading
    tmp = tempfile.mkdtemp(prefix='bank-regen-')
    try:
        # The stand-in backend.
        seen = []

        class H(http.server.BaseHTTPRequestHandler):
            def _send(self, code, obj):
                b = json.dumps(obj).encode()
                self.send_response(code); self.send_header('content-length', str(len(b))); self.end_headers()
                self.wfile.write(b)

            def do_GET(self):
                seen.append(('GET', self.path, self.headers.get('x-trevor-worker-key')))
                if self.headers.get('x-trevor-worker-key') != 'k1':
                    return self._send(401, {'error': 'key'})
                self._send(200, {'version': 'v1', 'items': [{'question': 'Who is Amiya?', 'normQ': 'who is amiya',
                                                             'eraPos': 46, 'hits': 9}, {'question': 'Q2', 'eraPos': 0, 'hits': 2}]})

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers['content-length'])))
                seen.append(('POST', self.path, body))
                self._send(200, {'queued': len(body['items']), 'cached': 0, 'joined': 0})

            def log_message(self, *x):
                pass
        srv = http.server.HTTPServer(('127.0.0.1', 0), H)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        base = f'http://127.0.0.1:{srv.server_address[1]}'
        ns = argparse.Namespace(version='v1', limit=None, backend_url=base, dry_run=None)
        assert warm(ns, {'TREVOR_WORKER_KEY': 'k1', 'TREVOR_WARM_TOP': '5'}) == 0
        assert seen[0] == ('GET', '/api/trevor/worker/popular?version=v1&limit=5', 'k1'), seen[0]
        assert seen[1][1] == '/api/trevor/worker/warm' and seen[1][2] == {
            'version': 'v1', 'items': [{'question': 'Who is Amiya?', 'eraPos': 46}, {'question': 'Q2', 'eraPos': 0}]}
        assert warm(ns, {'TREVOR_WORKER_KEY': 'k1', 'TREVOR_WARM_TOP': '0'}) == 0 and len(seen) == 2
        assert warm(ns, {'TREVOR_WORKER_KEY': 'bad'}) == 1
        ns.dry_run = os.path.join(tmp, 'dry')
        assert warm(ns, {'TREVOR_WORKER_KEY': 'k1'}) == 0 and os.path.exists(os.path.join(tmp, 'dry', 'warm.json'))
        assert [s[0] for s in seen].count('POST') == 1
        srv.shutdown()
        print('bank_regen.py self-test: ok (warm-up GET/POST shape, key, TREVOR_WARM_TOP=0, dry run)')
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('cmd', nargs='?', choices=('detect', 'plan', 'run', 'warm', 'count'))
    ap.add_argument('--prev', help='detect: the previous release artifacts directory')
    ap.add_argument('--backend', choices=('auto', 'cf', 'local'), default='auto')
    ap.add_argument('--until', default='07:00', help='run: UTC time the local backend stops starting slices')
    ap.add_argument('--cf-spent', type=int, default=0, help='run: Neurons the summaries spent before this run')
    ap.add_argument('--plan', action='store_true')
    ap.add_argument('--status', default=None)
    ap.add_argument('--version', help='warm: the version just activated')
    ap.add_argument('--limit', type=int, default=None, help='warm: default $TREVOR_WARM_TOP or 100')
    ap.add_argument('--backend-url', default=None, help='warm: default $TREVOR_BACKEND')
    ap.add_argument('--dry-run', default=None, metavar='DIR', help='warm: write the POST body here, send nothing')
    ap.add_argument('--self-test', action='store_true')
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    if a.cmd is None:
        ap.error('a subcommand is required')
    if a.cmd == 'warm':
        if not a.version:
            ap.error('warm needs --version')
        return warm(a)
    if OFF:
        print(f'REGEN_ANSWERS=0: bank_regen.py {a.cmd} does nothing', flush=True)
        if a.cmd == 'count':
            print(0)
        return 0
    if a.cmd == 'detect':
        if not a.prev:
            ap.error('detect needs --prev')
        detect(os.path.abspath(a.prev))
    elif a.cmd == 'plan':
        plan()
    elif a.cmd == 'count':
        pend = load_pending()
        print(len(pend['chunks']) + len(pend['reanswer']))
    elif a.cmd == 'run':
        run(a)
    return 0


if __name__ == '__main__':
    sys.exit(main())
