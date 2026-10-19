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
                                file ask and trevor-vps-update.sh write: "<pid> <label>". `lock_holder` reads it three
                                ways: absent, ours (this process or an ancestor, ask's rule in
                                src/bin/ask/llama_server.rs), or held (a live pid, or a file with no pid yet). Held:
                                wait, never steal. A dead pid's file is removed (trevor-vps-update.sh take_model_lock);
                                an empty or unparseable file counts as held until it is LOCK_STALE_AGE (60) s old,
                                since the update's noclobber `echo` creates the file before it writes the pid. The
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
  the nightly update            while trevor-vps-update.sh runs (STAGES_UPDATE_LOCK, $TREVOR_HOME/state/update.lock,
                                holds the pid of the run that has the flock; the probe reads it and never takes the
                                flock, so it cannot make the nightly run's `flock -n` fail): its canary gate waits at
                                most an hour (gates.py --lock-wait) for the model lock, so an incremental stage gives
                                the lock up when the update starts
A pause between stages waits (STAGES_PAUSE_POLL, 60 s); a pause that starts during an incremental stage stops that
stage's process group and its server (the stage loses at most the unit in flight). Two bounds keep the canary's hour:
the wait for another llama-server to go, under the model lock, gives up after STAGES_SERVER_WAIT (300) s or as soon as
the update starts, releases the lock and records an error; and canon, which is never interrupted, does not START within
STAGES_CANON_GUARD (2) hours before the daily timer (STAGES_DAILY_AT, 19:30 UTC, trevor-update.timer's OnCalendar;
0 turns the guard off). Canon is not split at its sub-steps because a resumed full run would wipe its outputs again:
`canon_refs.py inputs` compares with a stamp written only at the end, so it says "full" until the last step, and the
runner's CANON_FULL deletes every output before the passes. A canon run longer than the guard still overlaps the window.

Alerts and blocks (state in status.json, kept across restarts with the owed rebuilds):
  A stage, plan or rebuild that fails STAGES_ALERT_AFTER (2) times in a row posts one Discord embed per stage per day
  (DISCORD_ALERT_WEBHOOK, else TREVOR_DISCORD_WEBHOOK; trevor-vps-update.sh post_embed's JSON shape; a failed post is
  logged and swallowed). A stage interrupted STAGES_MAX_INTERRUPTIONS (3) times with no progress in its plan is blocked
  until the next day, with an embed. A rebuild that fails, or finds the nightly update running (exit 4 of
  `trevor-vps-update.sh rebuild`), stays owed and runs again in the next pass.

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
  logs/<YYYYMMDD>/run.log, server.log   one directory per day (a new one at midnight, appended to by every process
                that day): the runner's lines and every child's output; canon-before/ for a full canon. On the VPS
                ($TREVOR_HOME/state/stages/logs) scripts/retention.py keeps the newest RETAIN_LOGS (30)

After a group made progress, its outputs are rebuilt the way update.sh rebuilds them:
  groups 1, 3, 4   update.sh step 4 (chronology, dates, identities, deaths, renames, attributes, typed sources, the
                   lores p3b/p4/p4x, reading guide, source dates, gate order, answer bank seed and merge)
  group 5          topics traitrule, thin and units; the lores again when the units changed (update.sh step 4b)
  groups 6, 7      nothing (deep entries are read at answer time; canon builds and stamps itself)
  --no-rebuild     kill switch: leave the stage outputs as they are (no rebuild at all)
Publishing: `--no-publish` is the default (the development-machine mode: the rebuilt artifacts stay in place, nothing
is gated or uploaded). `--publish` is the VPS mode and needs TREVOR_PUBLISH=1 and scripts/trevor-vps-update.sh: the
runner then works in the update's stage tree ($TREVOR_HOME/stage/trevor, never the live release), runs the topics'
units step itself, and in place of update.sh's local rebuild calls `trevor-vps-update.sh rebuild`, which rebuilds the
stage deterministically (`deterministic_build 0`), runs the panels and gates.py (`panels_and_gates`, builtFrom kept at
the active version's, since nothing was fetched), and publishes through publish.py with the update's rollback
(`publish_stage`): cmd_summaries' tail without its model part. Without --publish the runner refuses a tree whose
artifacts/ is a symlink (the VPS live layout).

The nightly window (design/trevor-memory-budget.md section 4; 2026-10-19): the 12B does not fit beside serving on the
11 GB box, so for a bounded window each night the runner has the serving server stopped, runs its stages on the 12B,
and has serving restored; the backend serves from cache and passages meanwhile (contract v1.1's passages rung). The
runner (user trevor) cannot stop a system unit, so it asks root through a marker file:
  marker        <state dir>/llama-window holds `stop` or `start` (one word and a newline, written to a temp file and
                renamed into place); deploy/systemd/trevor-llama-window.path watches it and its .service runs
                `systemctl stop|start trevor-llama.service` as root, re-reading the marker until it stops changing
  opening       before the first unit in the window that needs a model: `stop`, then wait until the serving check
                (STAGES_SERVING_CHECK, default `pgrep -x llama-server`; its stdout lists pids, this runner's own server
                left out) shows none, at most STAGES_WINDOW_WAIT (300) s; then the model lock as for any stage. A
                serving server still there after the wait gives up the night's window (error logged, `start` written)
  in the window GEN is STAGES_WINDOW_GEN (update.sh's 12B), JUDGE STAGES_WINDOW_JUDGE (default: the JUDGE outside),
                STAGES_WINDOW_SERVER_ARGS appended to the server flags (default none), and the duty is
                STAGES_WINDOW_DUTY (1: no idle between units while serving is down; the duty file's 0 still stops all)
  closing       `start` at the window's end (a running interruptible stage is stopped first), when a pass has nothing
                left, before a --publish rebuild (its canary asks through the serving server), on any pause (the
                nightly update's lock, the pause file, memory), on duty 0, and on every exit of the runner (SIGTERM
                included). The 12B failing to start, or a memory floor tripping during a window stage, gives up the
                window until the next night. A runner that starts and finds `stop` writes `start` (a predecessor died)
  bounds        STAGES_WINDOW_START (18) to STAGES_WINDOW_END (0), UTC, as H, H:M or H:M:S: 03:00 to 09:00 JST. The
                daily update (19:30 UTC) falls inside: its lock pauses the stages and closes the window like any pause,
                and canon does not start in the window within STAGES_CANON_GUARD hours of the window's end either
  alert         one Discord embed per window when serving is not back STAGES_WINDOW_GRACE (600) s after `start` (or
                after the window's end while a stage that cannot be interrupted still holds it)
  status.json   window: {state (closed, opening, open, restoring), since, model, end, reason, error}
  STAGES_WINDOW kill switch: 0 never writes the marker and never changes the model (status.json has no window field).
                Default 1 with --publish (the VPS service), 0 without (the development machine has no serving unit)
  STAGES_WINDOW_ONLY=1  no stages outside the window (default 0: outside it the runner behaves as before, E2B beside
                serving). incr.py's keys hold no model name, so a unit E2B did by day is not redone by the 12B at night
The stage order inside the window is the order above: summaries and the re-answer first.

Service: deploy/systemd/trevor-stages.service (Type=simple, User=trevor, EnvironmentFile=/etc/trevor/trevor.env,
Nice=10, Restart=on-failure); it runs the loop with --publish. TODO: a paragraph in deploy/README.md.

Testing hooks: --root DIR (the tree to run in), --spec FILE (a JSON stage table in place of the built-in one, same
shape as GROUPS below), --state-dir DIR. scripts/tests/test_stages.py drives the runner with a stub stage and a stub
llama-server; no model runs in the tests.
"""
import argparse, json, os, re, shlex, signal, socket, subprocess, sys, time, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
TREVOR = os.path.dirname(HERE)
sys.path.insert(0, HERE)
import common  # noqa: E402  (llm_lock_path)

GEN_DEFAULT = 'models/llm/gemma-4-12b-it-qat-q4_0.gguf'   # update.sh GEN
DAILY_AT_DEFAULT = '19:30'   # trevor-update.timer OnCalendar, UTC (a test pins the two together)
JUDGE_DEFAULT = 'models/llm/Qwen3.5-9B-Q4_K_M.gguf'        # update.sh JUDGE
WINDOW_START_DEFAULT, WINDOW_END_DEFAULT = '18', '0'      # UTC hours: 03:00 to 09:00 JST
SERVING_CHECK_DEFAULT = 'pgrep -x llama-server'
WINDOW_MARKER = 'llama-window'                            # in the state dir; trevor-llama-window.path watches it

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


LOCK_STALE_AGE = 60.0   # seconds an empty or unparseable model lock counts as held (being written)


def lock_holder(path, stale_age=LOCK_STALE_AGE):
    """The model lock's state as (state, pid, label), state one of:
      'absent'  no file, or a stale one this call removed (a dead pid, or no pid and older than `stale_age` s)
      'ours'    a live pid that is this process or an ancestor (ask's foreign_lock rule)
      'held'    a live pid of another job, or a file with no parseable pid yet that is younger than `stale_age` s
                (trevor-vps-update.sh's noclobber `echo` creates the file before it writes the pid); pid None then
    A file is removed only when it is still the one judged stale (same inode, mtime and text)."""
    try:
        st = os.stat(path)
        text = slurp(path).strip()
    except OSError:
        return 'absent', None, ''
    pid_s, _, label = text.partition(' ')
    try:
        pid = int(pid_s)
    except ValueError:
        pid = None
    if pid is not None and alive(pid):
        return ('ours' if pid in ancestors() else 'held'), pid, label
    if pid is None and time.time() - st.st_mtime < stale_age:
        return 'held', None, label or '(lock being written)'
    try:
        now = os.stat(path)
        if (now.st_ino, now.st_mtime_ns) == (st.st_ino, st.st_mtime_ns) and slurp(path).strip() == text:
            os.remove(path)
    except OSError:
        pass
    return 'absent', None, ''


def update_lock_holder(path):
    """The pid of the trevor-vps-update.sh run that holds update.lock, or None. The script writes its pid into the
    file after its flock succeeds; this reads it and never takes the flock (a probe holding it, however briefly, made
    the nightly run's `flock -n` fail and skip the night). A dead pid, or a live one that is not the script (a pid
    reused after a reboot), is not a holder."""
    try:
        pid = int(slurp(path).split()[0])
    except (OSError, IndexError, ValueError):
        return None
    if not alive(pid):
        return None
    try:
        cmd = subprocess.run(['ps', '-o', 'command=', '-p', str(pid)], capture_output=True, text=True,
                             timeout=10).stdout
    except (OSError, subprocess.TimeoutExpired):
        return pid
    return pid if 'trevor-vps-update' in cmd else None


def post_embed(title, desc, color, footer):
    """One Discord embed in trevor-vps-update.sh post_embed's shape ({"embeds": [{title, description, color,
    footer}]}) to DISCORD_ALERT_WEBHOOK (else TREVOR_DISCORD_WEBHOOK). Returns whether it was sent; never raises."""
    url = os.environ.get('DISCORD_ALERT_WEBHOOK') or os.environ.get('TREVOR_DISCORD_WEBHOOK')
    if not url:
        return False
    body = json.dumps({'embeds': [{'title': title[:250], 'description': desc[:3900], 'color': int(color),
                                   'footer': {'text': footer[:200]}}]}).encode()
    # Discord's edge refuses urllib's default User-Agent; curl's passes, so name the client.
    req = urllib.request.Request(url, data=body, headers={'content-type': 'application/json',
                                                          'user-agent': 'trevor-stages (myrtle.moe, 1)'})
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            return 200 <= r.status < 300
    except Exception as e:
        print(f'== discord alert failed to send (swallowed): {e}', file=sys.stderr)
        return False


def minutes_until(hhmm, now=None):
    """Minutes from `now` (a UTC struct_time, default the current time) to the next HH:MM UTC, in [0, 1440)."""
    now = now or time.gmtime()
    h, m = (int(x) for x in hhmm.split(':'))
    return ((h * 60 + m) - (now.tm_hour * 60 + now.tm_min)) % 1440


def parse_tod(s):
    """A UTC time of day given as H, H:M or H:M:S (24 allowed for midnight) as seconds after midnight."""
    parts = [int(x) for x in str(s).strip().split(':')]
    if not 1 <= len(parts) <= 3:
        raise ValueError(f'not H, H:M or H:M:S: {s!r}')
    h, m, sec = (parts + [0, 0])[:3]
    if not (0 <= h <= 24 and 0 <= m < 60 and 0 <= sec < 60) or (h == 24 and (m or sec)):
        raise ValueError(f'not a time of day: {s!r}')
    return (h * 3600 + m * 60 + sec) % 86400


def window_bounds(start, end, now):
    """(inside, opens, ends) for the daily window [start, end) given in UTC seconds after midnight, at epoch `now`: the
    occurrence that contains `now`, else the next one. A window with start == end is never open."""
    if start == end:
        return False, None, None
    day = now - now % 86400
    sod = now - day
    if start < end:
        s = day + start if sod < end else day + 86400 + start
        e = s - start + end
    elif sod < end:            # wraps midnight; `now` is in the part after midnight, or before today's opening
        s, e = day - 86400 + start, day + end
    else:
        s, e = day + start, day + 86400 + end
    return s <= now < e, s, e


def utc_iso(ts):
    return time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime(ts)) if ts is not None else None


def http_ok(url, timeout=3):
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r:
            return r.status == 200
    except Exception:
        return False


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
        self.server_wait = env_float('STAGES_SERVER_WAIT', 300)
        self.canon_guard = env_float('STAGES_CANON_GUARD', 2)
        self.daily_at = os.environ.get('STAGES_DAILY_AT') or DAILY_AT_DEFAULT
        self.alert_after = max(1, int(env_float('STAGES_ALERT_AFTER', 2)))
        self.max_interruptions = max(1, int(env_float('STAGES_MAX_INTERRUPTIONS', 3)))
        self.lock_path = common.llm_lock_path()
        self.state_dir = a.state_dir
        self.server = None       # Popen of the llama-server this runner started
        self.server_why = ''     # why the last start_server failed
        self.child = None        # Popen of the running stage
        self.lock_held = False
        self.log = None
        self.log_day = None
        self.run_dir = None
        self.status = {'pid': os.getpid(), 'root': self.root, 'started': self.now(), 'state': 'starting',
                       'units': {}, 'lastError': None, 'publish': bool(a.publish)}
        self.pending = []        # rebuilds owed, in order
        self.failed_this_pass = set()
        self.errors = 0
        self.restore()
        # The nightly window (module docstring). Outside it, and with STAGES_WINDOW=0, models and flags stay these.
        self.outside = (dict(self.models), list(self.server_args))
        self.window_on = os.environ.get('STAGES_WINDOW', '1' if a.publish else '0') != '0'
        self.win_state = 'closed'      # closed, opening, open, restoring
        self.win_key = None            # the opening time of the occurrence in use
        self.win_until = None          # its end (epoch)
        self.win_given_up = None       # the occurrence given up (12B load failure, memory, serving never stopped)
        self.win_restore_by = None     # `start` written: serving should be back by then
        self.win_alerted = 'none'      # the occurrence whose grace embed went out
        self.win_close_after = None    # (why, give up) to apply once the stage's server is stopped
        self.win_checked = 0.0
        if self.window_on:
            self.win_start = parse_tod(os.environ.get('STAGES_WINDOW_START') or WINDOW_START_DEFAULT)
            self.win_end = parse_tod(os.environ.get('STAGES_WINDOW_END') or WINDOW_END_DEFAULT)
            self.win_gen = os.environ.get('STAGES_WINDOW_GEN') or GEN_DEFAULT
            self.win_judge = os.environ.get('STAGES_WINDOW_JUDGE') or self.models['judge']
            self.win_args = shlex.split(os.environ.get('STAGES_WINDOW_SERVER_ARGS', ''))
            self.win_wait = env_float('STAGES_WINDOW_WAIT', 300)
            self.win_grace = env_float('STAGES_WINDOW_GRACE', 600)
            self.win_duty = max(0.01, min(1.0, env_float('STAGES_WINDOW_DUTY', 1)))
            self.win_only = os.environ.get('STAGES_WINDOW_ONLY') == '1'
            self.serving_check = os.environ.get('STAGES_SERVING_CHECK') or SERVING_CHECK_DEFAULT
            self.status['window'] = {'state': 'closed', 'since': self.now(), 'model': self.models['gen']}

    # Kept across restarts (the service restarts on failure and on deploys): owed rebuilds, failure counts, the day
    # each stage last alerted, interruption counts and blocks.
    KEPT = ('failures', 'alerted', 'interruptions', 'blockedDay')

    def restore(self):
        try:
            old = json.loads(slurp(os.path.join(self.state_dir, 'status.json')))
        except (OSError, ValueError, TypeError):
            old = {}
        same = old.get('root') == self.root
        for k in self.KEPT:
            self.status[k] = old.get(k) if same and isinstance(old.get(k), dict) else {}
        if same and isinstance(old.get('pendingRebuilds'), list):
            self.pending = [n for n in old['pendingRebuilds'] if isinstance(n, str)]

    @staticmethod
    def today():
        return time.strftime('%Y-%m-%d')

    # ------------------------------------------------------------ bookkeeping
    @staticmethod
    def now():
        return time.strftime('%Y-%m-%dT%H:%M:%S%z')

    def open_log(self):
        """logs/<YYYYMMDD>/run.log, reopened when the day changes (a child keeps the file it was started with)."""
        day = time.strftime('%Y%m%d')
        if day == self.log_day and self.log:
            return
        self.run_dir = os.path.join(self.state_dir, 'logs', day)
        os.makedirs(self.run_dir, exist_ok=True)
        old, self.log = self.log, open(os.path.join(self.run_dir, 'run.log'), 'a', buffering=1)
        if old:
            old.close()
        self.log_day = day
        self.status['log'] = os.path.join(self.run_dir, 'run.log')

    def say(self, msg):
        line = f'== {time.strftime("%H:%M:%S")} {msg}'
        print(line, flush=True)
        if self.log:
            self.open_log()
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

    def fail(self, key, msg, error=True):
        """A failure of stage or rebuild `key` (an error unless `error` is false: a run that made no progress); the
        STAGES_ALERT_AFTER-th in a row posts the day's one embed for it."""
        if error:
            self.error(msg)
        f = self.status['failures']
        f[key] = f.get(key, 0) + 1
        if f[key] >= self.alert_after:
            self.alert(key, f'Trevor stages: {key} failed', f'`{key}` failed {f[key]} times in a row: {msg}',
                       'stages_failed')
        self.save()

    def ok(self, key):
        if self.status['failures'].pop(key, None) is not None:
            self.save()

    def alert(self, key, title, msg, kind):
        """At most one embed per key per day; the day is recorded only when the post went out."""
        day = self.today()
        if self.status['alerted'].get(key) == day:
            return
        tail = ''
        try:
            tail = ''.join(slurp(self.status['log']).splitlines(True)[-int(env_float('ALERT_LOG_LINES', 20)):])
        except (OSError, KeyError):
            pass
        tail = tail.replace('```', '` ` `')
        desc = (f'{msg}\nThe runner keeps going; the stage is retried in the next pass (state '
                f'{os.path.join(self.state_dir, "status.json")}).\n```\n{tail or "(no log)"}\n```')
        if post_embed(title, desc, 13646394, f'{kind} · {socket.gethostname()} · log {self.status.get("log")}'):
            self.status['alerted'][key] = day
            self.say(f'alert posted: {title}')
        self.save()

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
            self.window_watch()
            if wake and wake():
                return

    def runnable(self):
        """Whether a unit may start now as far as the duty and STAGES_WINDOW_ONLY go (the idle loop's wake test)."""
        return self.duty() > 0 and not (self.window_on and self.win_only and not self.window_wanted())

    def unit_duty(self):
        """The duty for the sleep after a unit: STAGES_WINDOW_DUTY while the window is open, unless the duty is 0."""
        d = self.duty()
        return self.win_duty if d > 0 and self.win_state == 'open' else d

    # ------------------------------------------------------------ the nightly window
    def window_now(self):
        """(inside, opens, ends) of the window occurrence at the current time; (False, None, None) when it is off."""
        if not self.window_on:
            return False, None, None
        return window_bounds(self.win_start, self.win_end, time.time())

    def window_wanted(self):
        """Inside the window and the night's occurrence not given up."""
        inside, opens, _ = self.window_now()
        return inside and self.win_given_up != opens

    def marker_path(self):
        return os.path.join(self.state_dir, WINDOW_MARKER)

    def read_marker(self):
        try:
            return slurp(self.marker_path()).strip()
        except OSError:
            return None

    def write_marker(self, state):
        """`stop` or `start` for trevor-llama-window.path: a temp file renamed into place (one change event)."""
        p = self.marker_path()
        with open(p + '.tmp', 'w') as f:
            f.write(state + '\n')
        os.replace(p + '.tmp', p)
        self.say(f'window: wrote {state} to {p}')

    def serving_pids(self):
        """The serving check's pids without this runner's own server; None when the check could not run."""
        try:
            out = subprocess.run(['bash', '-c', self.serving_check], capture_output=True, text=True,
                                 timeout=30).stdout.split()
        except (OSError, subprocess.TimeoutExpired):
            return None
        own = str(self.server.pid) if self.server is not None else None
        return [p for p in out if p != own]

    def set_window(self, state, **kw):
        self.win_state = state
        self.status['window'] = {'state': state, 'since': self.now(), 'model': self.models['gen'],
                                 'end': utc_iso(self.win_until), **kw}
        self.save()

    def open_window(self):
        """Ask for the serving server to stop and wait for the serving check to clear; then GEN is the window's model.
        'open'; 'paused' (a pause began while waiting: the window is closed again and no unit runs); anything else
        means the unit runs as outside the window (the night given up, or the window ended meanwhile)."""
        _, self.win_key, self.win_until = self.window_now()
        self.say(f'window: opening until {utc_iso(self.win_until)}; asking for the serving server to stop')
        self.write_marker('stop')
        self.set_window('opening')
        end = time.monotonic() + self.win_wait
        while True:
            pids = self.serving_pids()
            if pids == []:
                break
            if self.win_state != 'opening':          # closed by window_watch: the window ended while waiting
                return 'closed'
            if (r := self.pause_reasons()):
                self.close_window('paused while opening: ' + '; '.join(r))
                return 'paused'
            if time.monotonic() >= end:
                self.close_window(f'the serving server still runs {self.win_wait:g} s after stop (serving check: '
                                  f'{pids}); is trevor-llama-window.path installed?', give_up=True, error=True)
                return 'given-up'
            self.save(state='window-opening')
            self.tick(min(self.lock_poll, max(0.1, end - time.monotonic())))
        self.models['gen'], self.models['judge'] = self.win_gen, self.win_judge
        self.server_args = self.outside[1] + self.win_args
        self.set_window('open')
        self.say(f'window: open until {utc_iso(self.win_until)}; GEN {self.models["gen"]}, JUDGE {self.models["judge"]}')
        return 'open'

    def close_window(self, why, give_up=False, error=False):
        """Back to the outside models and `start` for the serving server. Only after the window's own server stopped."""
        if give_up:
            self.win_given_up = self.win_key
        if self.win_state not in ('opening', 'open'):
            return
        self.models, self.server_args = dict(self.outside[0]), list(self.outside[1])
        self.write_marker('start')
        self.win_restore_by = time.time() + self.win_grace
        extra = {'error': why} if error else {}
        self.set_window('restoring', reason=why + ('; given up until the next window' if give_up else ''), **extra)
        self.say(f'window: closed ({why}){"; given up until the next window" if give_up else ""}; serving asked to '
                 'start')
        if error:
            self.error('window: ' + why)

    def window_watch(self, can_close=True):
        """Called while the runner waits: closes a window past its end when no stage holds it (`can_close`), sees the
        serving server back after `start`, and posts the grace embed when it is not."""
        if not self.window_on or self.win_state == 'closed':
            return
        now = time.time()
        if self.win_state in ('opening', 'open'):
            if now >= self.win_until and can_close and self.child is None and self.server is None:
                self.close_window('the window ended')
            elif now >= self.win_until + self.win_grace:
                self.window_alert(f'The window ended at {utc_iso(self.win_until)} and the serving server is still '
                                  f'stopped: `{self.status.get("stage")}` cannot be interrupted.')
            return
        if now - self.win_checked < min(5.0, self.lock_poll):
            return
        self.win_checked = now
        if self.serving_pids():
            kept = {k: v for k, v in self.status['window'].items() if k in ('error', 'alert')}
            self.set_window('closed', reason='serving is back', **kept)
            self.say('window: the serving server is back')
        elif now >= self.win_restore_by:
            self.window_alert(f'`start` was written {self.win_grace:g} s ago (STAGES_WINDOW_GRACE) and the serving '
                              f'check (`{self.serving_check}`) still shows no server.')

    def window_alert(self, msg):
        """One embed per window occurrence (recorded even when the post fails, so a dead webhook is not hit each second)."""
        if self.win_alerted == self.win_key:
            return
        self.win_alerted = self.win_key
        self.say('window ALERT: ' + msg)
        self.status['window']['alert'] = msg
        self.save()
        post_embed('Trevor stages: serving not restored after the window',
                   f'{msg}\nMarker {self.marker_path()}: {self.read_marker()}. Check trevor-llama-window.path and '
                   '`systemctl status trevor-llama`; the backend serves from cache and passages meanwhile.',
                   13646394, f'stages_window · {socket.gethostname()} · log {self.status.get("log")}')

    def wait_restored(self):
        """After `start`, before a --publish rebuild: wait (STAGES_WINDOW_WAIT) for the serving check and for
        TREVOR_LLM_URL/health, which the canary asks through. False when it is not back; the rebuild runs anyway."""
        url = (os.environ.get('TREVOR_LLM_URL') or 'http://127.0.0.1:8081').rstrip('/') + '/health'
        end = time.monotonic() + self.win_wait
        while time.monotonic() < end:
            if self.win_state == 'closed' and http_ok(url):
                return True
            self.tick(min(5.0, max(0.1, end - time.monotonic())))
        self.say(f'window: serving not back {self.win_wait:g} s after start ({url}); the rebuild runs anyway')
        return False

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
        """Whether trevor-vps-update.sh runs (the pid it writes into update.lock under its flock; `update_lock_holder`
        never takes the flock). Checked on every host where the file exists: its canary gate waits at most an hour for
        the model lock, so a stage must give the lock up while it runs."""
        p = os.environ.get('STAGES_UPDATE_LOCK') or os.path.join(self.a.trevor_home, 'state', 'update.lock')
        return update_lock_holder(p) is not None

    def wait_unpaused(self):
        told = None
        while (r := self.pause_reasons()):
            if r != told:
                self.say('paused: ' + '; '.join(r)); told = r
            self.close_window('paused: ' + '; '.join(r))
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
                state, pid, label = lock_holder(self.lock_path)
                if state == 'ours':      # an ancestor holds it for us; it releases it, not this process
                    self.lock_held = False
                    return
                if state == 'absent':    # gone, or stale and removed: try the exclusive create again
                    continue
                if not told:
                    self.say(f'the model is busy with {label or "?"} (pid {pid}); waiting for {self.lock_path}')
                    told = True
                self.save(state='waiting-lock', lockHolder={'pid': pid, 'label': label})
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
        # The stage flags of design/trevor-memory-budget.md section 4, fixed here so STAGES_SERVER_ARGS cannot drop
        # them: one slot, no prompt cache (8 GiB by default), no context checkpoints (32 per slot by default).
        # STAGES_SERVER_LEAN=0 restores update.sh's flags exactly (the profile's slots, -cram 512).
        if os.environ.get('STAGES_SERVER_LEAN', '1') == '0':
            mem = ['-np', str(slots), '--kv-unified-per-slot', kv, '-fa', 'on', '-cram', '512']
        else:
            mem = ['-np', '1', '--kv-unified-per-slot', kv, '-fa', 'on', '--cache-ram', '0', '--ctx-checkpoints', '0']
        return [self.llama, '-m', self.models[model], '--host', '127.0.0.1', '--port', str(self.port), *mem,
                '--no-webui', '-ngl', 'all', *extra, *self.server_args]

    def start_server(self, profile):
        """Start llama-server for `profile` under the lock; never beside another on the port (or, exclusive, at all).
        The wait for another server is bounded (STAGES_SERVER_WAIT) and ends when the nightly update starts, since the
        lock is held meanwhile; False then, with the reason in self.server_why (the caller releases the lock)."""
        told, self.server_why = False, ''
        end = time.monotonic() + self.server_wait
        while self.health() or (self.exclusive and self.other_llama()):
            if not told:
                self.say(f'another llama-server runs ({"port " + str(self.port) if self.health() else "STAGES_EXCLUSIVE=1"}); waiting')
                told = True
            if self.update_lock_busy():
                self.server_why = 'the nightly update started while waiting for another llama-server to go'
                self.say(self.server_why)
                return False
            if time.monotonic() >= end:
                self.server_why = f'another llama-server still runs after {self.server_wait:g} s (STAGES_SERVER_WAIT)'
                self.say(self.server_why)
                return False
            self.save(state='waiting-server')
            self.tick(min(self.lock_poll, max(0.1, end - time.monotonic())))
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
                self.server_why = f'llama-server exited ({self.server.returncode}) while loading; see {slog.name}'
                self.say(self.server_why)
                self.server = None
                return False
            time.sleep(1)
        self.server_why = f'llama-server not healthy in {self.health_timeout:g} s'
        self.say(self.server_why)
        self.stop_server()
        return False

    def stop_server(self):
        if self.server is not None:
            stop_proc(self.server)
            self.say(f'stopped llama-server (pid {self.server.pid})')
            self.server = None
        self.status.pop('serverPid', None)

    def cleanup(self, why='the runner stops'):
        if self.child is not None:
            stop_group(self.child)
            self.child = None
        self.stop_server()
        self.release_lock()
        self.close_window(why)

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
            if time.monotonic() - last >= self.poll:
                last = time.monotonic()
                self.window_watch(can_close=False)
                if not interruptible:
                    continue
                r = self.pause_reasons(running=True)
                if self.win_state == 'open' and time.time() >= self.win_until:
                    r.append('the window ended')
                if r and self.win_state == 'open':   # closed once the stage's server is stopped (run_unit)
                    mem = any(x.startswith(('memory pressure', 'MemAvailable')) for x in r)
                    self.win_close_after = ('; '.join(r), mem)
                if r:
                    self.say('pausing mid-stage: ' + '; '.join(r))
                    self.status['lastPause'] = '; '.join(r)
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
        if self.window_on:
            self.window_watch()               # a window past its end closes before the next unit
            if (kind == 'canon' or st.get('server')) and self.window_wanted() and self.win_state != 'open':
                if self.open_window() == 'paused':
                    return 'paused'
        self.take_lock('stages ' + st['id'])
        try:
            if kind == 'canon':
                return self.run_canon()
            if st.get('server') and not self.start_model(st['server']):
                return 'server'
            self.save(state='running')
            return self.run_child(st['cmd'], st.get('interruptible', True))
        finally:
            self.stop_server()
            self.release_lock()
            if self.win_close_after:
                why, give_up = self.win_close_after
                self.win_close_after = None
                self.close_window(why, give_up=give_up, error=give_up and why.startswith('the window model'))

    def start_model(self, profile):
        """start_server; in the window a failure gives up the night's window once the server is stopped."""
        if self.start_server(profile):
            return True
        if self.win_state == 'open':
            self.win_close_after = (f'the window model did not start: {self.server_why}', True)
        return False

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
            if not self.start_model(profile):
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
        """trevor-vps-update.sh rebuild: the stage rebuilt, gated, published (only with --publish). True, False, or
        'busy' when the nightly update runs (seen here, or the script's exit 4: its run lock was taken)."""
        if self.update_lock_busy():
            self.say('the nightly update holds update.lock; publish owed to the next pass')
            return 'busy'
        self.open_log()
        rc = subprocess.call(['bash', os.path.join(TREVOR, 'scripts', 'trevor-vps-update.sh'), 'rebuild'],
                             stdout=self.log, stderr=subprocess.STDOUT)
        self.say(f'trevor-vps-update.sh rebuild: exit {rc}')
        if rc == 4:
            self.say('the nightly update took the run lock first; publish owed to the next pass')
            return 'busy'
        return rc == 0

    def flush(self, names):
        """Run the owed rebuilds among `names`. One that fails or finds the update running goes back on the owed list
        and is not tried again in this pass (run_pass's final flush skips it)."""
        for n in names:
            if n in self.pending and n not in self.failed_this_pass:
                if self.a.publish and not self.a.no_rebuild and self.win_state in ('opening', 'open'):
                    self.close_window('a publish gates through the serving server (the canary)')
                    self.wait_restored()
                self.pending.remove(n)
                self.save(state='rebuilding', stage=n)
                res = self.rebuild(n)
                if res is True:
                    self.ok('rebuild ' + n)
                    continue
                self.pending.append(n)
                self.failed_this_pass.add(n)
                if res == 'busy':
                    self.save()
                else:
                    self.fail('rebuild ' + n, f'rebuild {n} failed; owed to the next pass')

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
        if self.window_on and self.win_only and not self.window_wanted():
            raise Idle('outside the window (STAGES_WINDOW_ONLY=1)')

    def canon_guarded(self):
        """Why canon may not start now, or None: the daily timer is less than STAGES_CANON_GUARD hours away (canon is
        never interrupted, and the update's canary waits at most an hour for the model lock)."""
        if self.canon_guard <= 0:
            return None
        try:
            left = minutes_until(self.daily_at)
        except ValueError:
            return None
        if left < self.canon_guard * 60:
            return (f'the daily update is {left} min away (STAGES_DAILY_AT {self.daily_at} UTC, guard '
                    f'{self.canon_guard:g} h, STAGES_CANON_GUARD)')
        if self.window_wanted():
            end = self.window_now()[2]
            if end - time.time() < self.canon_guard * 3600:
                return (f'the window ends in {int((end - time.time()) // 60)} min (guard {self.canon_guard:g} h, '
                        'STAGES_CANON_GUARD)')
        return None

    def interrupted(self, sid, t, before, after):
        """A stage stopped by a pause: progress in its plan resets the count; STAGES_MAX_INTERRUPTIONS in a row
        without any block it until the next day, with the day's embed."""
        if after is not None and after < before:
            self.status['interruptions'].pop(sid, None)
            self.say(f'{sid}: interrupted after {t:.1f} s ({before} before, {after} after); it resumes after the pause')
            self.save()
            return
        k = self.status['interruptions'][sid] = self.status['interruptions'].get(sid, 0) + 1
        if k < self.max_interruptions:
            self.say(f'{sid}: interrupted after {t:.1f} s with no progress ({k} of {self.max_interruptions}); it '
                     'resumes after the pause')
            self.save()
            return
        self.status['interruptions'].pop(sid, None)
        self.status['blockedDay'][sid] = self.today()
        msg = f'{sid}: interrupted {k} times in a row with no progress; blocked until tomorrow'
        self.say(msg)
        self.alert(sid, f'Trevor stages: {sid} blocked', msg + f' (last pause: {self.status.get("lastPause", "?")})',
                   'stages_blocked')
        self.save()

    def sweep(self, blocked):
        """Top to bottom; returns after the first unit that ran (True), or None at the end of the order."""
        for g in self.groups:
            if not self.enabled(g):
                continue
            for st in g['stages']:
                if st['id'] in blocked or not self.enabled(st):
                    continue
                if self.status['blockedDay'].get(st['id']) == self.today():
                    continue
                self.check_duty()
                self.wait_unpaused()
                for c in st.get('pre_plan', []):
                    if self.sh(c):
                        self.fail(st['id'], f'{st["id"]}: {c} failed'); blocked.add(st['id']); break
                if st['id'] in blocked:
                    break
                self.save(state='planning', group=g['id'], stage=st['id'])
                n = self.plan(st)
                u = self.status['units'].setdefault(st['id'], {})
                u['left'] = n
                u['plannedAt'] = self.now()
                if n is None:
                    self.fail(st['id'], f'{st["id"]}: the plan failed ({u.get("planError", "no plan line")}); the rest '
                              f'of {g["id"]} waits for the next pass')
                    blocked.update(s['id'] for s in g['stages'])
                    break
                if n == 0:
                    continue
                if st.get('kind') == 'canon' and (why := self.canon_guarded()):
                    self.say(f'{st["id"]}: {n} to do, not started: {why}')
                    u['guard'] = why
                    blocked.add(st['id'])
                    continue
                u.pop('guard', None)
                self.say(f'{g["id"]} / {st["id"]}: {n} to do')
                t0 = time.monotonic()
                rc = self.run_unit(st)
                t = time.monotonic() - t0
                if rc == 'paused':     # a pause began while the window opened: back to the top, which waits on it
                    return True
                after = self.plan(st) if rc in (0, 'interrupted') else n
                u['left'] = after
                if rc in (0, 'interrupted'):
                    u['done'] = u.get('done', 0) + max(0, n - (after if after is not None else n))
                last = {'stage': st['id'], 'seconds': round(t, 2), 'rc': rc, 'before': n, 'after': after,
                        'at': self.now()}
                self.save(lastUnit=last)
                if rc == 'interrupted':
                    self.interrupted(st['id'], t, n, after)
                elif rc != 0:
                    why = {3: ' (shrink guard: input empty or shrunk)', 'server': f' ({self.server_why})'}.get(rc, '')
                    self.fail(st['id'], f'{st["id"]}: exit {rc}{why}')
                    blocked.update(s['id'] for s in g['stages'])
                elif after is None or after >= n:
                    self.say(f'{st["id"]}: no progress ({n} before, {after} after); skipped until the next pass')
                    self.fail(st['id'], f'{st["id"]}: ran without progress ({n} before, {after} after)', error=False)
                    blocked.add(st['id'])
                else:
                    self.progress = True
                    self.ok(st['id'])
                    self.status['interruptions'].pop(st['id'], None)
                    for r in g.get('rebuild', []):
                        if r not in self.pending:
                            self.pending.append(r)
                d = self.unit_duty()
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
        self.failed_this_pass = set()
        today = self.today()
        self.status['blockedDay'] = {k: v for k, v in self.status['blockedDay'].items() if v == today}
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
                    self.close_window('nothing to do')
                    self.say(f'nothing to do; idle {self.idle:g} s')
                    self.save(state='idle', idleReason='nothing to do')
                    self.tick(self.idle)
            except Idle as e:
                self.cleanup(str(e))
                if self.a.once:
                    self.say(f'{e}: stopping')
                    self.save(state='stopped', idleReason=str(e))
                    return
                self.say(f'{e}: idle, re-checking every {self.idle:g} s')
                self.save(state='idle', idleReason=str(e))
                self.tick(self.idle, wake=self.runnable)


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
    if r.window_on:
        inside, opens, ends = r.window_now()
        print(f'# window {utc_iso(opens)} to {utc_iso(ends)} ({"now" if inside else "next"}): echo stop > '
              f'{r.marker_path()}; wait for `{r.serving_check}` to show none; GEN {r.win_gen}; echo start at the end')
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
                     'gates and publish of the stage without a model); this copy is older than 2026-10-19')
        a.root = a.root or os.path.join(a.trevor_home, 'stage', 'trevor')
        if not os.path.isdir(os.path.join(a.root, 'artifacts')):
            ap.error(f'no stage tree at {a.root}: run trevor-vps-update.sh daily once first')
    a.root = os.path.abspath(a.root or TREVOR)
    if not a.publish and os.path.islink(os.path.join(a.root, 'artifacts')) and not (a.plan or a.dry_run):
        ap.error(f'{a.root}/artifacts is a symlink (the VPS live release); the runner never writes it: use --publish '
                 '(the stage tree) or --root')
    a.state_dir = os.path.abspath(a.state_dir or os.environ.get('STAGES_STATE_DIR') or (
        os.path.join(a.trevor_home, 'state', 'stages') if a.publish else os.path.join(a.root, 'artifacts', 'stages')))
    if os.environ.get('STAGES_WINDOW', '1' if a.publish else '0') != '0':
        for k, d in (('STAGES_WINDOW_START', WINDOW_START_DEFAULT), ('STAGES_WINDOW_END', WINDOW_END_DEFAULT)):
            try:
                parse_tod(os.environ.get(k) or d)
            except ValueError as e:
                ap.error(f'{k}: {e}')
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
    if r.window_on:
        r.say(f'window {utc_iso(r.window_now()[1])} to {utc_iso(r.window_now()[2])}: GEN {r.win_gen}, serving check '
              f'`{r.serving_check}`, marker {r.marker_path()}{", no stages outside it" if r.win_only else ""}')
        if r.read_marker() == 'stop':   # a runner that died with the window open: serving first
            r.win_state = 'open'
            r.close_window('the marker said stop at startup (a previous runner did not restore serving)')
    elif r.read_marker() == 'stop':
        r.say(f'WARNING: {r.marker_path()} says stop and STAGES_WINDOW=0 never writes it: `echo start > '
              f'{r.marker_path()}` restores serving')
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
