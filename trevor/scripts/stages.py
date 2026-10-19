#!/usr/bin/env python3
"""The paced stage runner, `trevor-stages` (design/trevor-self-contained-deploy.md section 4, step 5; 2026-10-19).

A long-running, resumable runner for the model stages of scripts/update.sh: it plans every stage with the stage's own
PLAN=1 dry run (update.sh's `planned()`), runs the stages that have work in a fixed priority order, one model at a
time under the model lock, and paces itself with a duty cycle. The stage scripts resume themselves (scripts/incr.py
keys every unit by id, input sha and prompt sha), so the runner keeps no queue: a restart replans from the top and
skips every stage whose plan is 0.

Priority order (each group's commands are update.sh's own, verbatim; scripts/tests/test_stages.py checks that):
  1 summaries   p2-summaries.py story, then group
  2 reanswer    yesterday's live questions answered again (step 6, scripts/reanswer.py): `collect` (psql, no model)
                before the plan, `answer` (`ask --batch --profile full`, resumes on qid) on the GEN server, then
                `merge` as the group's rebuild. Off with REANSWER=0 or when no DSN is set (REANSWER_DSN,
                TREVOR_DATABASE_URL, DATABASE_URL); reanswer.py's own commands, not update.sh's
  3 bank        answer_bank.py questions, then answer_bank.py answer (`ask --batch`, resumes on qid)
  4 derived     chrono extract and events (cues first), identity extract (cues first), death extract and check,
                real names, dossiers, primers
  5 topics      topics classify (after `topics mine`), gen (after `topics plan`), judge (JUDGE), retrait
  6 deep        topic_deep gen --top DEEP_TOP, judge (JUDGE)
  7 canon       canon_refs.py: inputs, then endings or prune, the GEN, JUDGE and reasoning-on passes, build, stamp
Where this differs from the doc's list: the bank's questions run before its answers (the answers need them), and the
chronology extract and events sit with group 4 (update.sh runs them in its model section; the doc does not name them).
After every unit that made progress the runner goes back to the top, so a new story's summary is never queued behind
a topic run. A stage whose plan does not shrink after a run, or that fails, is skipped until the next pass.

One unit is one stage call: the stage scripts take no unit cap, so a stage redoes its whole plan in one call (a game
update's summaries are hours on a CPU). The duty cycle is applied between calls.

  python3 scripts/stages.py            the loop (the service): passes until stopped, idles STAGES_IDLE s between
  python3 scripts/stages.py --once     one pass through the order, then exit (1 when a stage or rebuild failed)
  python3 scripts/stages.py --plan     every stage and its unit count from PLAN=1; no server, no lock, writes nothing
  python3 scripts/stages.py --dry-run  the commands a pass would run, printed; runs none (not even the plans)

Pacing and pauses:
  --duty D (STAGES_DUTY, 0.5)   after a unit of T seconds, sleep T*(1-D)/D. D=0 is the kill switch: stop after the
                                current unit, then idle and re-check every STAGES_IDLE (900) s. A file `duty` in the
                                state dir (one number) overrides it while running: `echo 0 > artifacts/stages/duty`.
  model lock                    common.llm_lock_path() (TREVOR_LLM_LOCK, else ${TMPDIR:-/tmp}/trevor-llm.lock), the
                                file ask and trevor-vps-update.sh write: "<pid> <label>". Held by a live process that
                                is not this one or an ancestor (ask's rule, src/bin/ask/llama_server.rs): wait, never
                                steal; a dead holder's file is removed (trevor-vps-update.sh take_model_lock). The
                                runner writes "<pid> stages <stage>" only while a stage runs, and starts llama-server
                                only under it; the server is stopped by PID after every stage and on any exit.
  never a second server         nothing may answer on the port; with STAGES_EXCLUSIVE=1 (default, update.sh's
                                await_no_llama) no llama-server process may run at all. The VPS sets 0, since
                                trevor-llama.service serves E2B on :8081 for good, and gives the runner its own port.
  memory                        macOS: `memory_pressure` free percentage below STAGES_MIN_FREE_PCT (25, the house
                                rule); Linux: MemAvailable below STAGES_MIN_MEM_GIB (2) GiB. During a stage, whose
                                own server holds the model, the floors are STAGES_MIN_FREE_PCT_RUNNING (10) and
                                STAGES_MIN_MEM_GIB_RUNNING (0.5)
  load                          Linux: the 1-minute load average above STAGES_MAX_LOAD (the CPU count), checked
                                before a stage only (the stage's own threads raise it)
  pause file                    `pause` in the state dir (operator pause: `touch artifacts/stages/pause`)
  the nightly update            while trevor-vps-update.sh holds STAGES_UPDATE_LOCK ($TREVOR_HOME/state/update.lock,
                                when it exists): its canary gate waits at most an hour (gates.py --lock-wait) for
                                the model lock, so an incremental stage gives the lock up when the update starts
A pause between stages waits (STAGES_PAUSE_POLL, 60 s); a pause that starts during an incremental stage stops that
stage's process group and its server (the stage loses at most the unit in flight); canon is never interrupted.

Models and servers (update.sh's own command lines; env names as there):
  STAGES_GEN [GEN, else models/llm/gemma-4-12b-it-qat-q4_0.gguf]   STAGES_JUDGE [JUDGE, else Qwen3.5-9B-Q4_K_M.gguf]
  STAGES_PORT [PORT, else 8081]   KV [16384]   STAGES_LLAMA_SERVER [LLAMA_SERVER, else llama-server]
  STAGES_SERVER_ARGS  extra llama-server flags appended to update.sh's (e.g. "-ngl 0 -t 2 --poll 0" on a CPU box)
  The 11 GB VPS cannot hold the 12B beside serving (section 5): there the documented configuration is
  STAGES_GEN=models/llm/gemma-4-e2b-it-q4_k_m.gguf (scripts/fetch-model.sh WITH_E2B=1 installs it under that name),
  and STAGES_JUDGE the same file or Qwen when memory allows. No code change.
  update.sh's knobs apply: SKIP_BANK_ANSWERS=1, SKIP_P4_MODELS=1 (groups 5 to 7), DEEP_TOP (40), and the rebuild's
  PREFIXES-style switches (IDENTITIES_V2, LABEL_RENAMES, GAMETEXT, SOURCE_DATES, GATE_ORDER).

State (STAGES_STATE_DIR, default <root>/artifacts/stages; with --publish $TREVOR_HOME/state/stages):
  status.json   the state (running, sleeping, paused, waiting-lock, idle), the stage, every stage's units left (from
                its plan) and done (by this process), the last unit, the last error, timestamps
  logs/<stamp>/run.log   one log per run: the runner's lines and every child's output; canon-before/ for a full canon

After a group made progress, its outputs are rebuilt the way update.sh rebuilds them:
  groups 1, 3, 4   update.sh step 4 (chronology, dates, identities, deaths, renames, attributes, typed sources, the
                   lores p3b/p4/p4x, reading guide, source dates, gate order, answer bank seed and merge)
  group 5          topics traitrule, thin and units; the lores again when the units changed (update.sh step 4b)
  groups 6, 7      nothing (deep entries are read at answer time; canon builds and stamps itself)
  --no-rebuild     kill switch: leave the stage outputs as they are (no rebuild at all)
Publishing: `--no-publish` is the default (the development-machine mode: the rebuilt artifacts stay in place, nothing
is gated or uploaded). `--publish` is the VPS mode and needs TREVOR_PUBLISH=1 and scripts/trevor-vps-update.sh: the
runner then works in the update's stage tree ($TREVOR_HOME/stage/trevor, never the live release), runs the topics'
units step itself, and in place of update.sh's local rebuild calls `trevor-vps-update.sh rebuild`, which is to rebuild
the stage deterministically (`deterministic_build 0`), run the panels and gates.py (`panels_and_gates`), and publish
through publish.py with the update's rollback (`publish_stage`): cmd_summaries' tail without its model part. That
command is NOT in trevor-vps-update.sh yet (2026-10-19: the script was being edited by another change), so --publish
refuses to start until it exists. Without --publish the runner refuses a tree whose artifacts/ is a symlink (the VPS
live layout).

Service: deploy/systemd/trevor-stages.service (Type=simple, User=trevor, EnvironmentFile=/etc/trevor/trevor.env,
Nice=10, Restart=on-failure); it runs the loop. TODO: a paragraph in deploy/README.md.

Testing hooks: --root DIR (the tree to run in), --spec FILE (a JSON stage table in place of the built-in one, same
shape as GROUPS below), --state-dir DIR. scripts/tests/test_stages.py drives the runner with a stub stage and a stub
llama-server; no model runs in the tests.
"""
import argparse, fcntl, json, os, re, shlex, signal, subprocess, sys, time, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
TREVOR = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import common  # noqa: E402  (llm_lock_path)

GEN_DEFAULT = 'models/llm/gemma-4-12b-it-qat-q4_0.gguf'   # update.sh GEN
JUDGE_DEFAULT = 'models/llm/Qwen3.5-9B-Q4_K_M.gguf'        # update.sh JUDGE

# Server profiles, as update.sh starts them: (model, slots, KV or None for ${KV:-16384}, extra args).
SERVERS = {
    'gen2': ('gen', 2, '16384', ['--reasoning', 'off']),     # run_models: start_llama "$GEN" ... 2 16384 --reasoning off
    'gen': ('gen', 1, None, ['--reasoning', 'off']),         # 4b: serve "$GEN" --reasoning off
    'judge': ('judge', 1, None, ['--reasoning', 'off']),     # 4b: serve "$JUDGE" --reasoning off
    'gen_on': ('gen', 1, '24576', ['--reasoning', 'on', '--reasoning-budget', '3000']),  # KV=24576 serve "$GEN" ...
}


def stage(sid, cmd, server='gen2', plan='planned', pre=(), pre_plan=(), interruptible=True, skip_env=None):
    return {'id': sid, 'cmd': cmd, 'server': server, 'plan': plan, 'pre': list(pre), 'pre_plan': list(pre_plan),
            'interruptible': interruptible, 'skip_env': skip_env}


DEEP = 'python3 scripts/topic_deep.py gen --top "${DEEP_TOP:-40}"'
# update.sh's canon block, in its order.
CANON_GEN = ['summarize', 'judge', 'wide', 'undated', 'discriminate', 'story gemma']
CANON_JUDGE = ['second', 'story qwen', 'verdict qwen-off']
CANON_FULL = [
    'mkdir -p "$LOG/canon-before"; find artifacts/canon -maxdepth 1 -name \'*.json*\' ! -name inputs.json -exec cp -p {} "$LOG/canon-before/" \\;',
    'python3 scripts/canon_refs.py endings',
    "find artifacts/canon -maxdepth 1 -name '*.json*' ! -name inputs.json ! -name endings.jsonl ! -name is_endings.json -exec rm {} \\;",
]

GROUPS = [
    {'id': 'summaries', 'title': 'story and group summaries', 'rebuild': ['step4'], 'stages': [
        stage('p2-story', 'python3 scripts/p2-summaries.py story'),
        stage('p2-group', 'python3 scripts/p2-summaries.py group')]},
    {'id': 'reanswer', 'title': "yesterday's live questions answered again at the full profile (step 6)",
     'rebuild': ['reanswer_merge'], 'off_env': ['REANSWER', '0'],
     'needs_env': ['REANSWER_DSN', 'TREVOR_DATABASE_URL', 'DATABASE_URL'], 'stages': [
        stage('reanswer', 'python3 scripts/reanswer.py answer', server='gen',
              pre_plan=['python3 scripts/reanswer.py collect'])]},
    {'id': 'bank', 'title': 'bank questions and answers', 'rebuild': ['step4'], 'stages': [
        stage('bank-questions', 'python3 scripts/answer_bank.py questions'),
        stage('bank-answers', 'python3 scripts/answer_bank.py answer', plan='bank_answers', skip_env='SKIP_BANK_ANSWERS')]},
    {'id': 'derived', 'title': 'chronology, identities, deaths, real names, dossiers, primers', 'rebuild': ['step4'],
     'stages': [
        stage('chrono-extract', 'python3 scripts/chrono.py extract', pre=['python3 scripts/chrono.py cues']),
        stage('chrono-events', 'python3 scripts/chrono.py events'),
        stage('entities-extract', 'python3 scripts/entities.py extract', pre=['python3 scripts/entities.py cues']),
        stage('deaths-extract', 'python3 scripts/deaths.py extract', plan={'regex': r'extract: (\d+) passages to do'}),
        stage('deaths-check', 'python3 scripts/deaths.py check', plan={'regex': r'check: (\d+) named mentions'}),
        stage('real-names', 'python3 scripts/real_names.py gen'),
        stage('dossiers', 'python3 scripts/dossiers.py gen'),
        stage('primers', 'python3 scripts/primers.py gen')]},
    {'id': 'topics', 'title': 'topics', 'rebuild': ['topic_units'], 'skip_env': 'SKIP_P4_MODELS', 'stages': [
        stage('topics-classify', 'python3 scripts/topics.py classify', server='gen',
              pre_plan=['python3 scripts/topics.py mine']),
        stage('topics-gen', 'python3 scripts/topics.py gen', server='gen', pre_plan=['python3 scripts/topics.py plan']),
        stage('topics-judge', 'python3 scripts/topics.py judge', server='judge'),
        stage('topics-retrait', 'python3 scripts/topics.py retrait', server='gen')]},
    {'id': 'deep', 'title': 'deep topics', 'rebuild': [], 'skip_env': 'SKIP_P4_MODELS', 'stages': [
        stage('deep-gen', DEEP, server='gen'),
        stage('deep-judge', 'python3 scripts/topic_deep.py judge', server='judge')]},
    {'id': 'canon', 'title': 'canon and its judge passes', 'rebuild': [], 'skip_env': 'SKIP_P4_MODELS', 'stages': [
        {'id': 'canon', 'kind': 'canon', 'cmd': 'python3 scripts/canon_refs.py inputs', 'plan': 'canon',
         'interruptible': False}]},
]

# update.sh's build_lores, verbatim; (command, env switch whose "0" skips it).
BUILD_LORES = [
    ('python3 scripts/dossiers.py sets', None),
    ('target/release/build-profiles --dossiers artifacts/dossiers/dossiers.v1.jsonl', None),
    ('target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p3b --reuse-from artifacts/p3a', None),
    ('target/release/build-index --corpus artifacts/p3b', None),
    ('cat artifacts/sources/units.jsonl artifacts/topics/units.v1.jsonl > artifacts/sources/all_units.jsonl', None),
    ('target/release/build-units --units artifacts/sources/all_units.jsonl', None),
    ('target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p4 --reuse-from artifacts/p3b', None),
    ('target/release/build-index --corpus artifacts/p4', None),
    ('python3 scripts/gametext.py', 'GAMETEXT'),
    ('target/release/build-units --corpus artifacts/p3b --units artifacts/gametext/p4x_units.jsonl --out artifacts/p4x', 'GAMETEXT'),
    ('target/release/embed-corpus --model-dir models/gte-modernbert-base --corpus artifacts/p4x --reuse-from artifacts/p4', 'GAMETEXT'),
    ('target/release/build-index --corpus artifacts/p4x', 'GAMETEXT'),
]
# update.sh step 4, verbatim (the wiki references are left out: REFRESH_REFERENCE is off by default and eval only).
STEP4 = [
    ('python3 scripts/chrono.py derive', None), ('python3 scripts/chrono.py flashbacks', None),
    ('python3 scripts/chrono.py build', None), ('python3 scripts/chrono.py history', None),
    ('python3 scripts/reading_guide.py dates', None),
    ('python3 scripts/entities.py build', None), ('python3 scripts/deaths.py build', None),
    ('ENTITY_V2=1 python3 scripts/entities.py build', 'IDENTITIES_V2'),
    ('python3 scripts/label_renames.py', 'LABEL_RENAMES'),
    ('python3 scripts/attributes.py', None),
    ('python3 scripts/sources.py', None),
    *BUILD_LORES,
    ('python3 scripts/reading_guide.py build', None),
    ('python3 scripts/source_dates.py ${TREVOR_GAMEDATA:+--gamedata "$TREVOR_GAMEDATA/excel"}', 'SOURCE_DATES'),
    ('target/release/ask --gate-order', 'GATE_ORDER'),
    ('PRIMERS=1 python3 scripts/answer_bank.py seed', None), ('python3 scripts/answer_bank.py merge', None),
]
# update.sh step 4b after the topic stages; the lores are rebuilt only when a units file changed.
TOPIC_POST = ['python3 scripts/topics.py traitrule', 'python3 scripts/topics.py thin', 'python3 scripts/topics.py units']
TOPIC_UNITS = ['artifacts/topics/units.jsonl', 'artifacts/topics/units.v1.jsonl']
# Step 6's merge: answer_bank.py merge with the re-answered rows folded in (scripts/reanswer.py).
REANSWER_MERGE = 'python3 scripts/reanswer.py merge'


class Stop(Exception):
    """SIGTERM, SIGINT or SIGHUP: clean up (stage group, server, lock) and exit."""


def slurp(path, mode='r'):
    with open(path, mode) as f:
        return f.read()


def env_float(name, default):
    try:
        return float(os.environ.get(name) or default)
    except ValueError:
        return float(default)


def duty_sleep(t, duty):
    """Seconds to idle after a unit of `t` seconds at duty `duty` (0 < duty <= 1): t*(1-duty)/duty."""
    return t * (1 - duty) / duty if duty > 0 else 0.0


def parse_memory_pressure(text):
    """The free percentage from macOS `memory_pressure` ("System-wide memory free percentage: 73%"), or None."""
    m = re.search(r'free percentage:\s*(\d+)%', text)
    return int(m.group(1)) if m else None


def parse_meminfo(text):
    """MemAvailable in GiB from /proc/meminfo, or None."""
    m = re.search(r'^MemAvailable:\s+(\d+)\s*kB', text, re.M)
    return int(m.group(1)) / 1048576 if m else None


def alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def ancestors():
    """This process and its ancestors' pids (ask's foreign_lock walks the same chain)."""
    out, p = [os.getpid()], os.getpid()
    for _ in range(64):
        try:
            pp = int(subprocess.run(['ps', '-o', 'ppid=', '-p', str(p)], capture_output=True, text=True).stdout.strip())
        except ValueError:
            break
        if pp <= 1 or pp == p:
            break
        out.append(pp); p = pp
    return out


def lock_holder(path):
    """(pid, label) of the model lock's holder when it is another live job; None when the lock is free, stale or ours.
    A stale lock (no pid, or a dead one) is removed, as trevor-vps-update.sh take_model_lock does."""
    try:
        text = slurp(path).strip()
    except OSError:
        return None
    pid_s, _, label = text.partition(' ')
    try:
        pid = int(pid_s)
    except ValueError:
        pid = None
    if pid is None or not alive(pid):
        try:
            os.remove(path)
        except OSError:
            pass
        return None
    if pid in ancestors():
        return None
    return pid, label


class Runner:
    def __init__(self, a):
        self.a = a
        self.root = os.path.abspath(a.root)
        self.groups = json.loads(slurp(a.spec))['groups'] if a.spec else GROUPS
        self.port = os.environ.get('STAGES_PORT') or os.environ.get('PORT') or '8081'
        self.models = {'gen': os.environ.get('STAGES_GEN') or os.environ.get('GEN') or GEN_DEFAULT,
                       'judge': os.environ.get('STAGES_JUDGE') or os.environ.get('JUDGE') or JUDGE_DEFAULT}
        self.llama = os.environ.get('STAGES_LLAMA_SERVER') or os.environ.get('LLAMA_SERVER') or 'llama-server'
        self.server_args = shlex.split(os.environ.get('STAGES_SERVER_ARGS', ''))
        self.exclusive = os.environ.get('STAGES_EXCLUSIVE', '1') != '0'
        self.idle = env_float('STAGES_IDLE', 900)
        self.pause_poll = env_float('STAGES_PAUSE_POLL', 60)
        self.lock_poll = env_float('STAGES_LOCK_POLL', 10)
        self.poll = env_float('STAGES_POLL', 15)
        self.health_timeout = env_float('STAGES_HEALTH_TIMEOUT', 180)
        self.lock_path = common.llm_lock_path()
        self.state_dir = a.state_dir
        self.server = None       # Popen of the llama-server this runner started
        self.child = None        # Popen of the running stage
        self.lock_held = False
        self.log = None
        self.run_dir = None
        self.status = {'pid': os.getpid(), 'root': self.root, 'started': self.now(), 'state': 'starting',
                       'units': {}, 'lastError': None, 'publish': bool(a.publish)}
        self.pending = []        # rebuilds owed, in order
        self.errors = 0

    # ------------------------------------------------------------ bookkeeping
    @staticmethod
    def now():
        return time.strftime('%Y-%m-%dT%H:%M:%S%z')

    def open_log(self):
        stamp = time.strftime('%Y%m%d-%H%M%S')
        self.run_dir = os.path.join(self.state_dir, 'logs', stamp)
        os.makedirs(self.run_dir, exist_ok=True)
        self.log = open(os.path.join(self.run_dir, 'run.log'), 'a', buffering=1)
        self.status['log'] = os.path.join(self.run_dir, 'run.log')

    def say(self, msg):
        line = f'== {time.strftime("%H:%M:%S")} {msg}'
        print(line, flush=True)
        if self.log:
            self.log.write(line + '\n')

    def save(self, **kw):
        self.status.update(kw)
        self.status['updated'] = self.now()
        self.status['duty'] = self.duty()
        self.status['pendingRebuilds'] = list(self.pending)
        p = os.path.join(self.state_dir, 'status.json')
        tmp = p + '.tmp'
        with open(tmp, 'w') as f:
            json.dump(self.status, f, indent=1, sort_keys=True)
        os.replace(tmp, p)

    def error(self, msg):
        self.errors += 1
        self.say('ERROR: ' + msg)
        self.save(lastError=msg, lastErrorAt=self.now())

    def duty(self):
        try:
            return max(0.0, min(1.0, float(slurp(os.path.join(self.state_dir, 'duty')).strip())))
        except (OSError, ValueError):
            return self.a.duty

    def tick(self, seconds, wake=None):
        """Sleep `seconds` in short steps (signals stay prompt); `wake()` true ends it early."""
        end = time.monotonic() + seconds
        while (left := end - time.monotonic()) > 0:
            time.sleep(min(1.0, left))
            if wake and wake():
                return

    # ------------------------------------------------------------ children
    def env(self, plan=False):
        e = dict(os.environ)
        e.pop('PLAN', None)
        if plan:
            e['PLAN'] = '1'
        e['SERVER'] = f'http://127.0.0.1:{self.port}'
        if self.run_dir:
            e['LOG'] = self.run_dir
        return e

    def sh(self, cmd, skip_env=None):
        """Run one update.sh command line in the root (bash, set -euo pipefail), its output into the run log."""
        if skip_env and os.environ.get(skip_env, '1') == '0':
            self.say(f'{skip_env}=0: skipping {cmd}')
            return 0
        self.say(f'$ {cmd}')
        rc = subprocess.call(['bash', '-c', 'set -euo pipefail; ' + cmd], cwd=self.root, env=self.env(),
                             stdout=self.log, stderr=subprocess.STDOUT)
        if rc:
            self.say(f'exit {rc}: {cmd}')
        return rc

    def plan(self, st):
        """The stage's unit count from its PLAN=1 run (update.sh planned()), or None when the plan failed."""
        kind = st.get('kind')
        if kind == 'hook':
            return 0
        p = st.get('plan', 'planned')
        if p == 'bank_answers':
            return self.bank_answers_left()
        cmd = st['cmd']
        out = subprocess.run(['bash', '-c', 'set -euo pipefail; ' + cmd], cwd=self.root, env=self.env(plan=True),
                             capture_output=True, text=True)
        if out.returncode:
            tail = (out.stderr or out.stdout).strip().splitlines()[-3:]
            self.status['units'].setdefault(st['id'], {})['planError'] = ' | '.join(tail)
            return None
        if p == 'canon':
            last = (out.stdout.strip().splitlines() or ['none'])[-1].split()[0]
            self.status['units'].setdefault(st['id'], {})['canon'] = last
            return 0 if last == 'none' else 1
        rx = p['regex'] if isinstance(p, dict) else r'plan (\d+)'
        m = re.search(rx, out.stdout)
        return int(m.group(1)) if m else None

    def bank_answers_left(self):
        """Questions in artifacts/bank/questions.jsonl with no row in answers.jsonl (ask --batch resumes on qid)."""
        def qids(p):
            try:
                return {json.loads(l)['qid'] for l in slurp(os.path.join(self.root, p)).splitlines() if l.strip()}
            except OSError:
                return set()
        return len(qids('artifacts/bank/questions.jsonl') - qids('artifacts/bank/answers.jsonl'))

    # ------------------------------------------------------------ pauses
    def pause_reasons(self, running=False):
        """Why the stages should not run now. `running`: checked during a stage, whose own server uses the memory and
        the cores, so the floors are the lower *_RUNNING ones and the load is not checked."""
        r = []
        if os.path.exists(os.path.join(self.state_dir, 'pause')):
            r.append('pause file')
        if sys.platform == 'darwin':
            floor = env_float('STAGES_MIN_FREE_PCT_RUNNING', 10) if running else env_float('STAGES_MIN_FREE_PCT', 25)
            if floor > 0:
                try:
                    free = parse_memory_pressure(subprocess.run(['memory_pressure'], capture_output=True, text=True,
                                                                timeout=30).stdout)
                except (OSError, subprocess.TimeoutExpired):
                    free = None
                if free is not None and free < floor:
                    r.append(f'memory pressure: {free}% free < {floor:g}%')
        elif os.path.exists('/proc/meminfo'):
            floor = env_float('STAGES_MIN_MEM_GIB_RUNNING', 0.5) if running else env_float('STAGES_MIN_MEM_GIB', 2)
            avail = parse_meminfo(slurp('/proc/meminfo'))
            if avail is not None and avail < floor:
                r.append(f'MemAvailable {avail:.1f} GiB < {floor:g} GiB')
            cap = env_float('STAGES_MAX_LOAD', os.cpu_count() or 1)
            load = os.getloadavg()[0]
            if load > cap and not running:
                r.append(f'load {load:.1f} > {cap:g}')
        if self.update_lock_busy():
            r.append('the nightly update holds update.lock')
        return r

    def update_lock_busy(self):
        """Whether trevor-vps-update.sh runs (its flock on update.lock). Checked on every host where the file exists:
        its canary gate waits at most an hour for the model lock, so a stage must give the lock up while it runs."""
        p = os.environ.get('STAGES_UPDATE_LOCK') or os.path.join(self.a.trevor_home, 'state', 'update.lock')
        if not os.path.exists(p):
            return False
        with open(p, 'a') as f:
            try:
                fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                return True
            fcntl.flock(f, fcntl.LOCK_UN)
        return False

    def wait_unpaused(self):
        told = None
        while (r := self.pause_reasons()):
            if r != told:
                self.say('paused: ' + '; '.join(r)); told = r
            self.save(state='paused', pauseReason='; '.join(r))
            self.tick(self.pause_poll)
        if told:
            self.say('pause over')
        self.status.pop('pauseReason', None)

    # ------------------------------------------------------------ the lock and the server
    def take_lock(self, label):
        told = False
        os.makedirs(os.path.dirname(self.lock_path) or '.', exist_ok=True)
        while True:
            try:
                fd = os.open(self.lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o644)
            except FileExistsError:
                h = lock_holder(self.lock_path)
                if h is None:
                    if os.path.exists(self.lock_path):   # ours: an ancestor holds it for us
                        self.lock_held = False
                        return
                    continue
                if not told:
                    self.say(f'the model is busy with {h[1] or "?"} (pid {h[0]}); waiting for {self.lock_path}')
                    told = True
                self.save(state='waiting-lock', lockHolder={'pid': h[0], 'label': h[1]})
                self.tick(self.lock_poll)
                continue
            with os.fdopen(fd, 'w') as f:
                f.write(f'{os.getpid()} {label}\n')
            self.lock_held = True
            self.status.pop('lockHolder', None)
            return

    def release_lock(self):
        if self.lock_held:
            try:
                if slurp(self.lock_path).split()[0] == str(os.getpid()):
                    os.remove(self.lock_path)
            except (OSError, IndexError):
                pass
        self.lock_held = False

    def health(self):
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{self.port}/health', timeout=3) as r:
                return r.status == 200
        except Exception:
            return False

    def other_llama(self):
        try:
            return subprocess.run(['pgrep', '-x', 'llama-server'], capture_output=True, text=True).stdout.split()
        except OSError:
            return []

    def server_cmd(self, profile):
        model, slots, kv, extra = SERVERS[profile]
        kv = kv or os.environ.get('KV') or '16384'
        return [self.llama, '-m', self.models[model], '--host', '127.0.0.1', '--port', str(self.port), '-np', str(slots),
                '--kv-unified-per-slot', kv, '-fa', 'on', '-cram', '512', '--no-webui', '-ngl', 'all', *extra,
                *self.server_args]

    def start_server(self, profile):
        """Start llama-server for `profile` under the lock; never beside another on the port (or, exclusive, at all)."""
        told = False
        while self.health() or (self.exclusive and self.other_llama()):
            if not told:
                self.say(f'another llama-server runs ({"port " + str(self.port) if self.health() else "STAGES_EXCLUSIVE=1"}); waiting')
                told = True
            self.save(state='waiting-server')
            self.tick(self.lock_poll)
        cmd = self.server_cmd(profile)
        self.say('$ ' + ' '.join(shlex.quote(c) for c in cmd))
        slog = open(os.path.join(self.run_dir, 'server.log'), 'a')
        self.server = subprocess.Popen(cmd, cwd=self.root, stdout=slog, stderr=subprocess.STDOUT,
                                       stdin=subprocess.DEVNULL, start_new_session=True)
        slog.close()
        end = time.monotonic() + self.health_timeout
        while time.monotonic() < end:
            if self.health():
                self.status['serverPid'] = self.server.pid
                return True
            if self.server.poll() is not None:
                self.say(f'llama-server exited ({self.server.returncode}) while loading; see {slog.name}')
                self.server = None
                return False
            time.sleep(1)
        self.say(f'llama-server not healthy in {self.health_timeout:g} s')
        self.stop_server()
        return False

    def stop_server(self):
        if self.server is not None:
            stop_proc(self.server)
            self.say(f'stopped llama-server (pid {self.server.pid})')
            self.server = None
        self.status.pop('serverPid', None)

    def cleanup(self):
        if self.child is not None:
            stop_group(self.child)
            self.child = None
        self.stop_server()
        self.release_lock()

    # ------------------------------------------------------------ one unit
    def run_child(self, cmd, interruptible):
        """Run a stage command; poll the pauses while it runs. Returns its exit code, or 'interrupted'."""
        self.say(f'$ {cmd}')
        self.child = subprocess.Popen(['bash', '-c', 'set -euo pipefail; ' + cmd], cwd=self.root, env=self.env(),
                                      stdout=self.log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
                                      start_new_session=True)
        last = time.monotonic()
        while (rc := self.child.poll()) is None:
            time.sleep(0.2)
            if interruptible and time.monotonic() - last >= self.poll:
                last = time.monotonic()
                r = self.pause_reasons(running=True)
                if r:
                    self.say('pausing mid-stage: ' + '; '.join(r))
                    stop_group(self.child)
                    self.child = None
                    return 'interrupted'
        self.child = None
        return rc

    def run_unit(self, st):
        """Pre steps, the lock, the server, the stage, the server stopped, the lock released. Returns the exit code
        (0 for success) or 'interrupted'."""
        kind = st.get('kind')
        if kind == 'hook':
            self.say(f'{st["id"]}: hook only (step 6 not built); nothing run')
            return 0
        for c in st.get('pre', []):
            if (rc := self.sh(c)):
                return rc
        self.take_lock('stages ' + st['id'])
        try:
            if kind == 'canon':
                return self.run_canon()
            if st.get('server') and not self.start_server(st['server']):
                return 'server'
            self.save(state='running')
            return self.run_child(st['cmd'], st.get('interruptible', True))
        finally:
            self.stop_server()
            self.release_lock()

    def run_canon(self):
        """update.sh's canon block, under one lock, three servers in turn."""
        out = subprocess.run(['bash', '-c', 'python3 scripts/canon_refs.py inputs'], cwd=self.root, env=self.env(),
                             capture_output=True, text=True)
        self.log.write(out.stdout + out.stderr)
        if out.returncode:
            return out.returncode
        canon = (out.stdout.strip().splitlines() or ['none'])[-1].strip()
        if canon == 'none':
            return 0
        self.say(f'canon: {canon} run')
        for c in (CANON_FULL if canon == 'full' else ['python3 scripts/canon_refs.py prune']):
            if (rc := self.sh(c)):
                return rc
        for profile, steps in (('gen', CANON_GEN), ('judge', CANON_JUDGE), ('gen_on', ['verdict gemma-on'])):
            if not self.start_server(profile):
                return 'server'
            self.save(state='running')
            for s in steps:
                if (rc := self.run_child(f'python3 scripts/canon_refs.py {s}', False)):
                    return rc
            self.stop_server()
        for c in ('python3 scripts/canon_refs.py build', 'python3 scripts/canon_refs.py stamp'):
            if (rc := self.sh(c)):
                return rc
        return 0

    # ------------------------------------------------------------ rebuilds and publish
    def rebuild(self, name):
        if self.a.no_rebuild:
            self.say(f'--no-rebuild: {name} left owed')
            return True
        if name == 'step4':
            if self.a.publish:
                return self.publish()
            return all(self.sh(c, g) == 0 for c, g in STEP4)
        if name == 'reanswer_merge':
            if self.sh(REANSWER_MERGE):
                return False
            return self.publish() if self.a.publish else True
        if name == 'topic_units':
            before = {p: self.read(p) for p in TOPIC_UNITS}
            for c in TOPIC_POST:
                if self.sh(c):
                    return False
            if all(self.read(p) == b for p, b in before.items()):
                self.say('topic units unchanged: no lores rebuild')
                return True
            self.say('topic units changed: rebuild p3b, p4')
            if self.a.publish:
                return self.publish()
            return all(self.sh(c, g) == 0 for c, g in BUILD_LORES)
        self.say(f'unknown rebuild {name}')
        return False

    def read(self, rel):
        try:
            return slurp(os.path.join(self.root, rel), 'rb')
        except OSError:
            return None

    def publish(self):
        """trevor-vps-update.sh rebuild: the stage rebuilt, gated, published (only with --publish)."""
        if self.update_lock_busy():
            self.say('the nightly update holds update.lock; publish owed to the next pass')
            return False
        rc = subprocess.call(['bash', os.path.join(TREVOR, 'scripts', 'trevor-vps-update.sh'), 'rebuild'],
                             stdout=self.log, stderr=subprocess.STDOUT)
        self.say(f'trevor-vps-update.sh rebuild: exit {rc}')
        return rc == 0

    def flush(self, names):
        for n in names:
            if n in self.pending:
                self.pending.remove(n)
                self.save(state='rebuilding', stage=n)
                if not self.rebuild(n):
                    self.error(f'rebuild {n} failed')

    # ------------------------------------------------------------ the order
    def off_reason(self, x):
        """Why a group or stage is off by the environment, or None: SKIP_* set (update.sh's `-z` test), an off value
        (REANSWER=0), or none of the variables it needs set (a database DSN)."""
        if x.get('skip_env') and os.environ.get(x['skip_env']):
            return f'{x["skip_env"]} set'
        if x.get('off_env') and os.environ.get(x['off_env'][0]) == x['off_env'][1]:
            return f'{x["off_env"][0]}={x["off_env"][1]}'
        if x.get('needs_env') and not any(os.environ.get(k) for k in x['needs_env']):
            return 'none of ' + ', '.join(x['needs_env']) + ' set'
        return None

    def enabled(self, x):
        return self.off_reason(x) is None

    def check_duty(self):
        if self.duty() <= 0:
            raise Idle('duty 0')

    def sweep(self, blocked):
        """Top to bottom; returns after the first unit that ran (True), or None at the end of the order."""
        for g in self.groups:
            if not self.enabled(g):
                continue
            for st in g['stages']:
                if st['id'] in blocked or not self.enabled(st):
                    continue
                self.check_duty()
                self.wait_unpaused()
                for c in st.get('pre_plan', []):
                    if self.sh(c):
                        self.error(f'{st["id"]}: {c} failed'); blocked.add(st['id']); break
                if st['id'] in blocked:
                    break
                self.save(state='planning', group=g['id'], stage=st['id'])
                n = self.plan(st)
                u = self.status['units'].setdefault(st['id'], {})
                u['left'] = n
                u['plannedAt'] = self.now()
                if n is None:
                    self.error(f'{st["id"]}: the plan failed ({u.get("planError", "no plan line")}); the rest of '
                               f'{g["id"]} waits for the next pass')
                    blocked.update(s['id'] for s in g['stages'])
                    break
                if n == 0:
                    continue
                self.say(f'{g["id"]} / {st["id"]}: {n} to do')
                t0 = time.monotonic()
                rc = self.run_unit(st)
                t = time.monotonic() - t0
                after = self.plan(st) if rc == 0 else n
                u['left'] = after
                u['done'] = u.get('done', 0) + max(0, n - (after or 0)) if rc == 0 else u.get('done', 0)
                last = {'stage': st['id'], 'seconds': round(t, 2), 'rc': rc, 'before': n, 'after': after,
                        'at': self.now()}
                self.save(lastUnit=last)
                if rc == 'interrupted':
                    self.say(f'{st["id"]}: interrupted after {t:.1f} s; it resumes after the pause')
                elif rc != 0:
                    self.error(f'{st["id"]}: exit {rc}' + (' (shrink guard: input empty or shrunk)' if rc == 3 else ''))
                    blocked.update(s['id'] for s in g['stages'])
                elif after is None or after >= n:
                    self.say(f'{st["id"]}: no progress ({n} before, {after} after); skipped until the next pass')
                    blocked.add(st['id'])
                else:
                    self.progress = True
                    for r in g.get('rebuild', []):
                        if r not in self.pending:
                            self.pending.append(r)
                d = self.duty()
                if rc != 'interrupted' and st.get('kind') != 'hook':
                    s = duty_sleep(t, d)
                    self.say(f'unit {st["id"]}: {t:.2f} s at duty {d:g}; sleeping {s:.2f} s')
                    if d > 0 and s > 0:
                        self.save(state='sleeping', sleepUntil=time.strftime('%H:%M:%S', time.localtime(time.time() + s)))
                        self.tick(s, wake=lambda: self.duty() <= 0)
                return True
            self.flush(g.get('rebuild', []))
        return None

    def run_pass(self):
        blocked, self.progress = set(), False
        if not os.path.isdir(os.path.join(self.root, 'artifacts')):
            self.error(f'no artifacts in {self.root} (on the VPS the stage appears after the first daily update)')
            return False
        self.status['pass'] = self.status.get('pass', 0) + 1
        self.status['passStarted'] = self.now()
        while self.sweep(blocked):
            pass
        self.flush(list(self.pending))
        self.save(state='pass done', stage=None, passDone=self.now())
        return self.progress

    def loop(self):
        while True:
            try:
                progress = self.run_pass()
                if self.a.once:
                    return
                if not progress:
                    self.say(f'nothing to do; idle {self.idle:g} s')
                    self.save(state='idle', idleReason='nothing to do')
                    self.tick(self.idle)
            except Idle as e:
                self.cleanup()
                if self.a.once:
                    self.say(f'{e}: stopping')
                    self.save(state='stopped', idleReason=str(e))
                    return
                self.say(f'{e}: idle, re-checking every {self.idle:g} s')
                self.save(state='idle', idleReason=str(e))
                self.tick(self.idle, wake=lambda: self.duty() > 0)


class Idle(Exception):
    """Duty 0: stop after the current unit."""


def stop_proc(p, grace=30):
    try:
        p.terminate()
    except ProcessLookupError:
        pass
    try:
        p.wait(grace)
    except subprocess.TimeoutExpired:
        p.kill(); p.wait()


def stop_group(p, grace=30):
    try:
        os.killpg(p.pid, signal.SIGTERM)
    except (ProcessLookupError, PermissionError):
        pass
    try:
        p.wait(grace)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass
        p.wait()


# ---------------------------------------------------------------- the modes without a runner loop

def show_plan(r):
    print(f'root {r.root}; GEN {r.models["gen"]}; JUDGE {r.models["judge"]}')
    total = 0
    for i, g in enumerate(r.groups, 1):
        off = '' if r.enabled(g) else f' (off: {r.off_reason(g)})'
        print(f'{i} {g["id"]}: {g.get("title", "")}{off}')
        for st in g['stages']:
            if not r.enabled(g) or not r.enabled(st):
                print(f'    {st["id"]}: off')
                continue
            n = r.plan(st)
            extra = r.status['units'].get(st['id'], {})
            note = f' ({extra["canon"]} run)' if 'canon' in extra else ''
            if st.get('kind') == 'hook':
                note = ' (hook: step 6 not built)'
            if n is None:
                note = f' (plan failed: {extra.get("planError", "no plan line")})'
            print(f'    {st["id"]}: {"?" if n is None else n}{note}')
            total += n or 0
    print(f'units to do: {total}')


def dry_run(r):
    d = r.a.duty
    print(f'dry run in {r.root}: nothing is run; lock {r.lock_path}; duty {d:g}')
    for i, g in enumerate(r.groups, 1):
        if not r.enabled(g):
            print(f'# {i} {g["id"]}: off ({r.off_reason(g)}); when on:')
        print(f'# {i} {g["id"]}: {g.get("title", "")}')
        for st in g['stages']:
            if not r.enabled(st):
                print(f'#   {st["id"]}: off ({r.off_reason(st)}); when on:')
            if st.get('kind') == 'hook':
                print(f'#   {st["id"]}: hook only, runs nothing')
                continue
            for c in st.get('pre_plan', []):
                print(f'{c}')
            if st.get('plan') == 'bank_answers':
                print('# plan: questions.jsonl qids without a row in answers.jsonl')
            else:
                print(f'PLAN=1 {st["cmd"]}   # the plan; the rest runs only when it is above 0')
            for c in st.get('pre', []):
                print(c)
            print(f'# take the model lock {r.lock_path} as "<pid> stages {st["id"]}"')
            if st.get('kind') == 'canon':
                print('python3 scripts/canon_refs.py inputs   # none: stop; full:')
                for c in CANON_FULL:
                    print('  ' + c)
                print('  # incremental: python3 scripts/canon_refs.py prune')
                for profile, steps in (('gen', CANON_GEN), ('judge', CANON_JUDGE), ('gen_on', ['verdict gemma-on'])):
                    print('  ' + ' '.join(shlex.quote(c) for c in r.server_cmd(profile)))
                    for s in steps:
                        print(f'  python3 scripts/canon_refs.py {s}')
                    print('  # stop llama-server by pid')
                print('  python3 scripts/canon_refs.py build\n  python3 scripts/canon_refs.py stamp')
            else:
                if st.get('server'):
                    print(' '.join(shlex.quote(c) for c in r.server_cmd(st['server'])))
                print(f'SERVER=http://127.0.0.1:{r.port} {st["cmd"]}')
                if st.get('server'):
                    print('# stop llama-server by pid')
            print(f'# release the lock; sleep T*(1-{d:g})/{d:g} = {duty_sleep(1, d) if d else 0:g} x the unit\'s T')
        for name in g.get('rebuild', []):
            if name == 'reanswer_merge' and not r.a.no_rebuild:
                print(f'{REANSWER_MERGE}   # after a unit made progress'
                      + ('; then bash scripts/trevor-vps-update.sh rebuild' if r.a.publish else ''))
            elif r.a.no_rebuild:
                print(f'# rebuild {name}: off (--no-rebuild)')
            elif r.a.publish and name == 'step4':
                print('bash scripts/trevor-vps-update.sh rebuild   # after a unit made progress')
            elif name == 'step4':
                print('# rebuild after a unit made progress (update.sh step 4):')
                for c, gate in STEP4:
                    print(f'  {c}' + (f'   # unless {gate}=0' if gate else ''))
            elif name == 'topic_units':
                print('# rebuild after a unit made progress (update.sh step 4b):')
                for c in TOPIC_POST:
                    print('  ' + c)
                if r.a.publish:
                    print('  bash scripts/trevor-vps-update.sh rebuild   # when a units file changed')
                else:
                    print('  # when a units file changed: build_lores')
                    for c, gate in BUILD_LORES:
                        print(f'    {c}' + (f'   # unless {gate}=0' if gate else ''))


def main(argv=None):
    ap = argparse.ArgumentParser(description='The paced stage runner (trevor-stages); see the module docstring.')
    ap.add_argument('--once', action='store_true', help='one pass through the order, then exit')
    m = ap.add_mutually_exclusive_group()
    m.add_argument('--plan', action='store_true', help='print every stage with its unit count and exit')
    m.add_argument('--dry-run', action='store_true', help='print the commands a pass would run; run none')
    ap.add_argument('--duty', type=float, default=env_float('STAGES_DUTY', 0.5),
                    help='0.0 to 1.0 (STAGES_DUTY, 0.5); 0 stops after the current unit')
    p = ap.add_mutually_exclusive_group()
    p.add_argument('--publish', action='store_true', help='VPS mode: needs TREVOR_PUBLISH=1 (module docstring)')
    p.add_argument('--no-publish', dest='publish', action='store_false', help='the default: no gates, no upload')
    ap.add_argument('--no-rebuild', action='store_true', help='kill switch: no deterministic rebuild after a stage')
    ap.add_argument('--root', help='the trevor tree to run in (default: this checkout; with --publish the stage)')
    ap.add_argument('--state-dir', help='status and logs (STAGES_STATE_DIR)')
    ap.add_argument('--spec', default=os.environ.get('STAGES_SPEC'), help='a JSON stage table (tests)')
    a = ap.parse_args(argv)
    if not 0 <= a.duty <= 1:
        ap.error('--duty must be within 0.0 and 1.0')
    a.trevor_home = os.environ.get('TREVOR_HOME', '/var/lib/trevor')
    if a.publish:
        if os.environ.get('TREVOR_PUBLISH') != '1':
            ap.error('--publish needs TREVOR_PUBLISH=1 in the environment (the host opts in)')
        vps = os.path.join(TREVOR, 'scripts', 'trevor-vps-update.sh')
        if not os.path.exists(vps):
            ap.error('--publish needs scripts/trevor-vps-update.sh')
        if 'cmd_rebuild' not in slurp(vps):
            ap.error('--publish needs a `rebuild` command in scripts/trevor-vps-update.sh (the deterministic build, '
                     'gates and publish of the stage without a model); it is not there yet')
        a.root = a.root or os.path.join(a.trevor_home, 'stage', 'trevor')
        if not os.path.isdir(os.path.join(a.root, 'artifacts')):
            ap.error(f'no stage tree at {a.root}: run trevor-vps-update.sh daily once first')
    a.root = os.path.abspath(a.root or TREVOR)
    if not a.publish and os.path.islink(os.path.join(a.root, 'artifacts')) and not (a.plan or a.dry_run):
        ap.error(f'{a.root}/artifacts is a symlink (the VPS live release); the runner never writes it: use --publish '
                 '(the stage tree) or --root')
    a.state_dir = os.path.abspath(a.state_dir or os.environ.get('STAGES_STATE_DIR') or (
        os.path.join(a.trevor_home, 'state', 'stages') if a.publish else os.path.join(a.root, 'artifacts', 'stages')))
    r = Runner(a)
    if a.plan:
        show_plan(r)
        return 0
    if a.dry_run:
        dry_run(r)
        return 0

    def on_signal(sig, _frame):
        raise Stop(signal.Signals(sig).name)
    for s in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(s, on_signal)
    os.makedirs(a.state_dir, exist_ok=True)
    r.open_log()
    r.say(f'trevor-stages in {r.root}: duty {r.duty():g}, GEN {r.models["gen"]}, JUDGE {r.models["judge"]}, port '
          f'{r.port}, lock {r.lock_path}, {"publish" if a.publish else "no publish"}, state {a.state_dir}')
    rc = 0
    try:
        r.loop()
        rc = 1 if r.errors else 0
    except Stop as e:
        r.say(f'{e}: stopping')
        rc = 143
    finally:
        r.cleanup()
        r.save(state='stopped' if rc != 143 else 'stopped by signal', stage=None, stoppedAt=r.now())
    return rc


if __name__ == '__main__':
    sys.exit(main())
