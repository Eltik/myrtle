#!/usr/bin/env python3
"""The paced stage runner, scripts/stages.py (design/trevor-self-contained-deploy.md step 5; 2026-10-19).

No model runs here. Each runner test builds a scratch tree with a stub stage (`stub_stage.py NAME SECONDS`: under
PLAN=1 it prints "NAME: plan N to do" from a counter file; a real run writes a start marker, sleeps, takes one off
the counter and writes a done marker) and a stub llama-server (`stub_llama.py`: answers /health on --port and records
its argv and pid), points the runner at them with --spec and STAGES_LLAMA_SERVER, and gives it its own lock file and
state dir. The window tests (class Window) add a stand-in for trevor-llama-window.path: a thread that runs the unit's
own ExecStart line on every marker change with a stub `systemctl` first in PATH, which records its calls and toggles
a `serving` file that STAGES_SERVING_CHECK reads (no process is ever named llama-server). The one test on the real
tree is `--plan`, which runs every stage's PLAN=1 dry run (read-only, no model) and
checks that no llama-server ran before or after.

  python3 -m unittest scripts/tests/test_stages.py      (from trevor/; stdlib only; about 3 minutes, most of it the
                                                         real tree's plan)
Scratch trees go under $TMPDIR.
"""
import argparse, fcntl, http.server, json, os, re, shutil, signal, socket, subprocess, sys, tempfile, threading, time, unittest
from unittest import mock

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TREVOR = os.path.dirname(SCRIPTS)
STAGES = os.path.join(SCRIPTS, 'stages.py')
sys.path.insert(0, SCRIPTS)
import stages  # noqa: E402

for _k in ('DISCORD_ALERT_WEBHOOK', 'TREVOR_DISCORD_WEBHOOK'):   # no test posts to a real webhook
    os.environ.pop(_k, None)

STUB_STAGE = r'''#!/usr/bin/env python3
import os, sys, time
name, secs = sys.argv[1], float(sys.argv[2])
cnt = os.path.join('counters', name)
n = int(open(cnt).read()) if os.path.exists(cnt) else 0
if os.environ.get('PLAN') == '1':
    print(f'{name}: plan {n} to do of 9')
    sys.exit(0)
os.makedirs('markers', exist_ok=True)
with open(os.path.join('markers', name), 'a') as f:
    f.write(f'start {time.time()} {os.environ.get("SERVER")}\n')
time.sleep(secs)
open(cnt, 'w').write(str(max(0, n - 1)))
with open(os.path.join('markers', name), 'a') as f:
    f.write(f'done {time.time()}\n')
'''

STUB_LLAMA = r'''#!/usr/bin/env python3
import http.server, os, sys
port = int(sys.argv[sys.argv.index('--port') + 1])
root = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(root, 'markers', 'server'), 'a') as f:
    f.write(f'{os.getpid()} ' + ' '.join(sys.argv[1:]) + '\n')

class H(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200 if self.path == '/health' else 404)
        self.end_headers()
        self.wfile.write(b'{"status":"ok"}')

    def log_message(self, *a):
        pass

http.server.HTTPServer(('127.0.0.1', port), H).serve_forever()
'''


def wr(path, text):
    with open(path, 'w') as f:
        f.write(text)


def rd(path):
    with open(path) as f:
        return f.read()


def free_port():
    s = socket.socket()
    s.bind(('127.0.0.1', 0))
    p = s.getsockname()[1]
    s.close()
    return p


def alive(pid):
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False


def llama_pids():
    return subprocess.run(['pgrep', '-x', 'llama-server'], capture_output=True, text=True).stdout.split()


class Tree:
    """A scratch tree with the stubs, a spec, a lock file and a state dir of its own."""

    def __init__(self, stages_spec, counters):
        self.root = tempfile.mkdtemp(prefix='trevor-stages-')
        os.makedirs(self.p('markers')); os.makedirs(self.p('counters')); os.makedirs(self.p('artifacts'))
        for name, text in (('stub_stage.py', STUB_STAGE), ('stub_llama.py', STUB_LLAMA)):
            with open(self.p(name), 'w') as f:
                f.write(text)
            os.chmod(self.p(name), 0o755)
        for k, v in counters.items():
            wr(self.p('counters', k), str(v))
        spec = {'groups': [{'id': 'g1', 'title': 'stub group', 'rebuild': [], 'stages': stages_spec}]}
        wr(self.p('spec.json'), json.dumps(spec))
        self.state = self.p('state')
        self.lock = self.p('llm.lock')
        self.port = free_port()
        self.proc = None
        self.path_unit = None

    def p(self, *parts):
        return os.path.join(self.root, *parts)

    def env(self, **kw):
        e = dict(os.environ)
        for k in list(e):
            if k.startswith('STAGES_') or k in ('PLAN', 'PORT', 'GEN', 'JUDGE', 'TREVOR_PUBLISH', 'DISCORD_ALERT_WEBHOOK',
                                                 'TREVOR_DISCORD_WEBHOOK'):
                del e[k]
        e.update(TREVOR_LLM_LOCK=self.lock, STAGES_LLAMA_SERVER=self.p('stub_llama.py'), STAGES_PORT=str(self.port),
                 STAGES_EXCLUSIVE='0', STAGES_MIN_FREE_PCT='0', STAGES_MIN_MEM_GIB='0', STAGES_MAX_LOAD='100000',
                 STAGES_MIN_FREE_PCT_RUNNING='0', STAGES_MIN_MEM_GIB_RUNNING='0',
                 STAGES_IDLE='1', STAGES_PAUSE_POLL='0.3', STAGES_LOCK_POLL='0.3', STAGES_POLL='0.5',
                 STAGES_HEALTH_TIMEOUT='20', STAGES_GEN='models/llm/stub.gguf',
                 STAGES_UPDATE_LOCK=self.p('update.lock'))
        e.update(kw)
        return e

    def args(self, *extra):
        return ['nice', '-n', '19', sys.executable, STAGES, '--root', self.root, '--spec', self.p('spec.json'),
                '--state-dir', self.state, *extra]

    def start(self, *extra, **env):
        self.proc = subprocess.Popen(self.args(*extra), env=self.env(**env), stdout=subprocess.DEVNULL,
                                     stderr=subprocess.DEVNULL)
        return self.proc

    def run(self, *extra, timeout=60, **env):
        return subprocess.run(self.args(*extra), env=self.env(**env), capture_output=True, text=True, timeout=timeout)

    def status(self):
        try:
            return json.loads(rd(os.path.join(self.state, 'status.json')))
        except (OSError, ValueError):
            return {}

    def markers(self, name):
        try:
            return [l.split() for l in rd(self.p('markers', name)).splitlines()]
        except OSError:
            return []

    def server_pids(self):
        return [int(l[0]) for l in self.markers('server')]

    def wait(self, cond, timeout=30):
        end = time.time() + timeout
        while time.time() < end:
            if cond():
                return True
            time.sleep(0.1)
        return False

    def runlog(self):
        logs = os.path.join(self.state, 'logs')
        return ''.join(rd(os.path.join(logs, d, 'run.log')) for d in sorted(os.listdir(logs)))

    def close(self):
        if self.path_unit:
            self.path_unit.stop()
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(30)
            except subprocess.TimeoutExpired:
                self.proc.kill()
        for pid in self.server_pids():
            if alive(pid):
                os.kill(pid, signal.SIGKILL)
        shutil.rmtree(self.root, ignore_errors=True)


def stub(name, secs, server='gen2', interruptible=True):
    return {'id': name, 'cmd': f'python3 stub_stage.py {name} {secs}', 'server': server, 'plan': 'planned',
            'pre': [], 'pre_plan': [], 'interruptible': interruptible}


def make_runner(state_dir, **env):
    """A Runner over an empty tree with `state_dir`, for the methods that need no child process."""
    a = argparse.Namespace(root=state_dir, spec=None, state_dir=state_dir, publish=False, duty=1.0, no_rebuild=False,
                           trevor_home=state_dir, once=True, plan=False, dry_run=False)
    clean = {k: v for k, v in os.environ.items() if not k.startswith('STAGES_')}
    with mock.patch.dict(os.environ, {**clean, **env}, clear=True):
        return stages.Runner(a)


class Webhook:
    """A stub Discord webhook: records every POSTed JSON body."""

    def __init__(self):
        self.posts = []
        hook = self

        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                body = self.rfile.read(int(self.headers['content-length']))
                hook.posts.append({'body': json.loads(body), 'ua': self.headers.get('user-agent'),
                                   'type': self.headers.get('content-type')})
                self.send_response(204)
                self.end_headers()

            def log_message(self, *a):
                pass

        self.srv = http.server.HTTPServer(('127.0.0.1', 0), H)
        self.url = f'http://127.0.0.1:{self.srv.server_address[1]}/hook'
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()

    def close(self):
        self.srv.shutdown()
        self.srv.server_close()


FAKE_SYSTEMCTL = r'''#!/bin/sh
echo "$*" >> "$FAKE_DIR/calls"
case "$1" in
  stop) sleep "${FAKE_STOP_DELAY:-0}"; : > "$FAKE_DIR/serving" ;;
  start) [ -e "$FAKE_DIR/start-fails" ] && exit 1; echo 4242 > "$FAKE_DIR/serving" ;;
esac
'''

BAD_LLAMA = r'''#!/usr/bin/env python3
import os, sys
if 'models/llm/window.gguf' in sys.argv:
    sys.exit(3)
here = os.path.dirname(os.path.abspath(__file__))
os.execv(sys.executable, [sys.executable, os.path.join(here, 'stub_llama.py')] + sys.argv[1:])
'''

UNIT_MARKER = '/var/lib/trevor/state/stages/llama-window'


class PathUnit:
    """Stands in for trevor-llama-window.path: polls the marker and, on every change, runs the ExecStart line of
    deploy/systemd/trevor-llama-window.service (systemd's $$ unescaped, the marker path swapped for the tree's) with a
    stub systemctl first in PATH. The serving server starts as running (`serving` holds a pid)."""

    def __init__(self, tree, stop_delay=0.0):
        unit = rd(os.path.join(TREVOR, 'deploy', 'systemd', 'trevor-llama-window.service'))
        line = [l for l in unit.splitlines() if l.startswith('ExecStart=')]
        m = re.fullmatch(r"ExecStart=/bin/sh -c '(.*)'", line[0] if len(line) == 1 else '')
        assert m and UNIT_MARKER in m.group(1), line
        self.marker = os.path.join(tree.state, 'llama-window')
        self.script = m.group(1).replace('$$', '$').replace(UNIT_MARKER, self.marker)
        fakebin = tree.p('fakebin')
        os.makedirs(fakebin)
        wr(os.path.join(fakebin, 'systemctl'), FAKE_SYSTEMCTL)
        os.chmod(os.path.join(fakebin, 'systemctl'), 0o755)
        wr(tree.p('serving'), '4242\n')
        self.tree = tree
        self.env = dict(os.environ, PATH=fakebin + os.pathsep + os.environ['PATH'], FAKE_DIR=tree.root,
                        FAKE_STOP_DELAY=str(stop_delay))
        self.events = []        # (time the unit finished, the marker it applied)
        self.stopped = False
        self.thread = threading.Thread(target=self.watch, daemon=True)
        self.thread.start()
        tree.path_unit = self

    def watch(self):
        last = None
        while not self.stopped:
            try:
                st = os.stat(self.marker)
                key = (st.st_ino, st.st_mtime_ns, rd(self.marker))
            except OSError:
                key = None
            if key is not None and key != last:
                last = key
                subprocess.run(['sh', '-c', self.script], env=self.env, capture_output=True, timeout=60)
                self.events.append((time.time(), key[2].strip()))
            time.sleep(0.05)

    def calls(self):
        try:
            return rd(self.tree.p('calls')).splitlines()
        except OSError:
            return []

    def stop(self):
        self.stopped = True
        self.thread.join(5)


def window_env(tree, end_in=3600.0, **kw):
    """A window that opened a minute ago and ends in `end_in` seconds (UTC H:M:S), the window model, the stub check."""
    now = time.time()
    hms = lambda t: time.strftime('%H:%M:%S', time.gmtime(t))  # noqa: E731
    return {'STAGES_WINDOW': '1', 'STAGES_WINDOW_START': hms(now - 60), 'STAGES_WINDOW_END': hms(now + end_in),
            'STAGES_WINDOW_GEN': 'models/llm/window.gguf', 'STAGES_SERVING_CHECK': f'cat {tree.p("serving")}',
            'STAGES_WINDOW_WAIT': '20', **kw}


STOP, START = 'stop trevor-llama.service', 'start trevor-llama.service'


class Units(unittest.TestCase):
    def test_duty_math(self):
        self.assertEqual(stages.duty_sleep(10, 0.5), 10)
        self.assertAlmostEqual(stages.duty_sleep(10, 0.25), 30)
        self.assertEqual(stages.duty_sleep(10, 1.0), 0)
        self.assertEqual(stages.duty_sleep(10, 0), 0)

    def test_memory_parsers(self):
        self.assertEqual(stages.parse_memory_pressure('...\nSystem-wide memory free percentage: 73%\n'), 73)
        self.assertIsNone(stages.parse_memory_pressure('nothing'))
        self.assertAlmostEqual(stages.parse_meminfo('MemTotal: 1 kB\nMemAvailable:    2097152 kB\n'), 2.0)

    def test_lock_holder(self):
        d = tempfile.mkdtemp(prefix='trevor-stages-lock-')
        try:
            p = os.path.join(d, 'llm.lock')
            self.assertEqual(stages.lock_holder(p), ('absent', None, ''))
            dead = subprocess.Popen(['true']); dead.wait()
            wr(p, f'{dead.pid} ask\n')
            self.assertEqual(stages.lock_holder(p)[0], 'absent')
            self.assertFalse(os.path.exists(p), 'a dead holder\'s lock is removed')
            wr(p, f'{os.getpid()} wrapper\n')
            self.assertEqual(stages.lock_holder(p), ('ours', os.getpid(), 'wrapper'), 'this process holds it: ours')
            self.assertTrue(os.path.exists(p))
            live = subprocess.Popen(['sleep', '30'])
            try:
                wr(p, f'{live.pid} worker\n')
                self.assertEqual(stages.lock_holder(p), ('held', live.pid, 'worker'))
            finally:
                live.kill(); live.wait()
            # The update's noclobber echo creates the file before it writes the pid: an empty or unparseable file
            # is held while young and never removed then; past LOCK_STALE_AGE it is stale.
            for text in ('', 'garbage\n'):
                wr(p, text)
                self.assertEqual(stages.lock_holder(p)[:2], ('held', None), repr(text))
                self.assertTrue(os.path.exists(p), 'a young lock is never removed')
                old = time.time() - stages.LOCK_STALE_AGE - 5
                os.utime(p, (old, old))
                self.assertEqual(stages.lock_holder(p)[0], 'absent', repr(text))
                self.assertFalse(os.path.exists(p))
        finally:
            shutil.rmtree(d)

    def test_update_lock_holder_reads_the_pid(self):
        """update.lock is read, never flocked: the pid of a live trevor-vps-update.sh run is a holder; a dead pid, a
        live pid of another program (reused after a reboot) or an empty file is not."""
        d = tempfile.mkdtemp(prefix='trevor-stages-upd-')
        try:
            p = os.path.join(d, 'update.lock')
            self.assertIsNone(stages.update_lock_holder(p))
            wr(p, '')
            self.assertIsNone(stages.update_lock_holder(p))
            dead = subprocess.Popen(['true']); dead.wait()
            wr(p, f'{dead.pid} daily\n')
            self.assertIsNone(stages.update_lock_holder(p))
            other = subprocess.Popen(['sleep', '30'])
            fake = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)', 'trevor-vps-update.sh', 'daily'])
            try:
                wr(p, f'{other.pid} daily\n')
                self.assertIsNone(stages.update_lock_holder(p), 'a reused pid is not the update')
                wr(p, f'{fake.pid} daily\n')
                self.assertEqual(stages.update_lock_holder(p), fake.pid)
                with open(p, 'a') as f:   # a flock probe would block this exclusive non-blocking take; reading does not
                    fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self.assertEqual(stages.update_lock_holder(p), fake.pid)
            finally:
                for x in (other, fake):
                    x.kill(); x.wait()
        finally:
            shutil.rmtree(d)

    def test_daily_at_matches_the_timer(self):
        timer = rd(os.path.join(TREVOR, 'deploy', 'systemd', 'trevor-update.timer'))
        self.assertIn(f'OnCalendar=*-*-* {stages.DAILY_AT_DEFAULT}:00 UTC', timer)
        t = time.struct_time((2026, 10, 19, 18, 0, 0, 0, 292, 0))
        self.assertEqual(stages.minutes_until('19:30', t), 90)
        self.assertEqual(stages.minutes_until('17:30', t), 1410)
        self.assertEqual(stages.minutes_until('18:00', t), 0)

    def test_window_bounds(self):
        day = 1_760_000_000 - 1_760_000_000 % 86400      # a UTC midnight
        s, e = stages.parse_tod('18'), stages.parse_tod('0')
        self.assertEqual((s, e), (64800, 0))
        self.assertEqual(stages.window_bounds(s, e, day + 19 * 3600), (True, day + 64800, day + 86400))
        self.assertEqual(stages.window_bounds(s, e, day + 9 * 3600), (False, day + 64800, day + 86400))
        self.assertEqual(stages.window_bounds(s, e, day + 86400)[0], False, 'the end is outside')
        w = (stages.parse_tod('22:00'), stages.parse_tod('4:30'))
        self.assertEqual(stages.window_bounds(*w, day + 2 * 3600), (True, day - 7200, day + 16200))
        self.assertEqual(stages.window_bounds(*w, day + 23 * 3600), (True, day + 79200, day + 86400 + 16200))
        self.assertEqual(stages.window_bounds(*w, day + 12 * 3600), (False, day + 79200, day + 86400 + 16200))
        self.assertEqual(stages.window_bounds(3600, 7200, day + 3 * 3600), (False, day + 86400 + 3600, day + 86400 + 7200))
        self.assertEqual(stages.window_bounds(5, 5, day), (False, None, None))
        self.assertEqual(stages.parse_tod('24'), 0)
        self.assertEqual(stages.parse_tod('3:04:05'), 11045)
        for bad in ('25', '3:60', 'x', '1:2:3:4', '24:01'):
            with self.assertRaises(ValueError, msg=bad):
                stages.parse_tod(bad)

    def test_canon_guard(self):
        d = tempfile.mkdtemp(prefix='trevor-stages-guard-')
        try:
            soon = time.strftime('%H:%M', time.gmtime(time.time() + 1800))
            later = time.strftime('%H:%M', time.gmtime(time.time() + 3 * 3600 + 120))
            self.assertIn('min away', make_runner(d, STAGES_DAILY_AT=soon).canon_guarded() or '')
            self.assertIsNone(make_runner(d, STAGES_DAILY_AT=later).canon_guarded())
            self.assertIsNone(make_runner(d, STAGES_DAILY_AT=soon, STAGES_CANON_GUARD='0').canon_guarded())
        finally:
            shutil.rmtree(d)

    def test_flush_keeps_a_failed_rebuild_owed(self):
        """A rebuild that fails, or meets the nightly update ('busy', exit 4 of trevor-vps-update.sh rebuild), goes
        back on the owed list, is not retried in the same pass, and survives a restart; success clears it."""
        d = tempfile.mkdtemp(prefix='trevor-stages-flush-')
        try:
            r = make_runner(d)
            results = [False, 'busy', True]
            calls = []
            r.rebuild = lambda n: (calls.append(n), results.pop(0))[1]
            r.pending = ['step4']
            r.flush(['step4'])
            self.assertEqual(r.pending, ['step4'])
            self.assertEqual(r.status['failures'], {'rebuild step4': 1})
            r.flush(list(r.pending))   # run_pass's final flush, same pass
            self.assertEqual(calls, ['step4'], 'not retried in the pass that failed')
            r2 = make_runner(d)        # a restart: the owed rebuild comes back from status.json
            self.assertEqual(r2.pending, ['step4'])
            self.assertEqual(r2.status['failures'], {'rebuild step4': 1})
            r2.rebuild = r.rebuild
            r2.flush(['step4'])
            self.assertEqual(r2.pending, ['step4'], 'busy: still owed')
            self.assertEqual(r2.status['failures'], {'rebuild step4': 1}, 'busy is not a failure')
            r2.failed_this_pass = set()
            r2.flush(['step4'])
            self.assertEqual(r2.pending, [])
            self.assertEqual(r2.status['failures'], {})
            self.assertEqual(json.loads(rd(os.path.join(d, 'status.json')))['pendingRebuilds'], [])
        finally:
            shutil.rmtree(d)

    def test_vps_rebuild_exits_4_on_a_busy_run_lock(self):
        """trevor-vps-update.sh rebuild exits 4 when another update holds update.lock (the runner keeps its publish
        owed); the holder's line is not truncated; a run that takes the lock writes "<pid> <command>" into it."""
        if not shutil.which('flock'):
            self.skipTest('no flock(1) here')
        home = tempfile.mkdtemp(prefix='trevor-stages-vps-')
        try:
            os.makedirs(os.path.join(home, 'state'))
            lock = os.path.join(home, 'state', 'update.lock')
            wr(lock, '12345 daily\n')
            env = dict(os.environ, TREVOR_HOME=home, TREVOR_NICED='1', DISCORD_ALERT_WEBHOOK='', TREVOR_DISCORD_WEBHOOK='')
            vps = os.path.join(SCRIPTS, 'trevor-vps-update.sh')
            with open(lock, 'a') as f:
                fcntl.flock(f, fcntl.LOCK_EX)
                r = subprocess.run(['bash', vps, 'rebuild'], env=env, capture_output=True, text=True, timeout=60)
            self.assertEqual(r.returncode, 4, r.stdout + r.stderr)
            self.assertEqual(rd(lock), '12345 daily\n')
            r = subprocess.run(['bash', vps, 'rebuild'], env=env, capture_output=True, text=True, timeout=60)
            self.assertEqual(r.returncode, 1, 'no live release here: require_live_release stops it')
            self.assertRegex(rd(lock), r'^\d+ rebuild\n$')
        finally:
            shutil.rmtree(home)

    def test_commands_are_update_sh_verbatim(self):
        """Every command the runner runs is update.sh's own text, so the two cannot drift apart silently."""
        u = rd(os.path.join(SCRIPTS, 'update.sh'))
        cmds = [c for c, _ in stages.STEP4] + stages.TOPIC_POST + stages.CANON_FULL
        for g in stages.GROUPS:
            if g['id'] == 'reanswer':  # step 6: reanswer.py's commands, not update.sh's
                continue
            for st in g['stages']:
                cmds += [st[k] for k in ('cmd',) if st.get('cmd')] + st.get('pre', []) + st.get('pre_plan', [])
        for c in cmds:
            self.assertIn(c, u, c)
        for c in ('python3 scripts/canon_refs.py prune', 'python3 scripts/canon_refs.py build',
                  'python3 scripts/canon_refs.py stamp', 'python3 scripts/canon_refs.py verdict gemma-on'):
            self.assertIn(c, u)
        q = lambda xs: ' '.join(f'"{x}"' if ' ' in x else x for x in xs)  # noqa: E731
        self.assertIn(f'for s in {q(stages.CANON_GEN)}; do python3 scripts/canon_refs.py $s; done', u)
        self.assertIn(f'for s in {q(stages.CANON_JUDGE)}; do python3 scripts/canon_refs.py $s; done', u)
        self.assertIn(f'GEN={stages.GEN_DEFAULT}\n', u)
        self.assertIn(f'JUDGE={stages.JUDGE_DEFAULT}\n', u)
        self.assertIn('start_llama "$GEN" "$LOG/server.log" 2 16384 --reasoning off', u)
        self.assertIn('KV=24576 serve "$GEN" --reasoning on --reasoning-budget 3000', u)
        self.assertIn('start_llama "$1" "$LOG/server.log" 1 "${KV:-16384}" "${@:2}"', u)
        sh = rd(os.path.join(SCRIPTS, 'llama.sh'))
        self.assertIn('-np "$3" --kv-unified-per-slot "$4" \\\n    -fa on -cram 512 --no-webui -ngl all "${@:5}"', sh)


class Runner(unittest.TestCase):
    def setUp(self):
        self.trees = []

    def tearDown(self):
        for t in self.trees:
            t.close()

    def tree(self, spec, counters):
        t = Tree(spec, counters)
        self.trees.append(t)
        return t

    def test_a_duty_sleep_math(self):
        """Two one-second units at duty 0.25: the runner sleeps 3 T after each, T being the unit's own seconds."""
        t = self.tree([stub('a', 1.0)], {'a': 2})
        r = t.run('--once', '--duty', '0.25', timeout=90)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        log = t.runlog()
        units = [(float(a), float(b)) for a, b in re.findall(r'unit a: ([\d.]+) s at duty 0.25; sleeping ([\d.]+) s', log)]
        self.assertEqual(len(units), 2, log)
        for T, S in units:
            self.assertGreaterEqual(T, 1.0)
            self.assertAlmostEqual(S, 3 * T, delta=0.02)
        m = t.markers('a')
        self.assertEqual([x[0] for x in m], ['start', 'done', 'start', 'done'])
        gap = float(m[2][1]) - float(m[1][1])
        self.assertGreaterEqual(gap, units[0][1])
        self.assertLess(gap, units[0][1] + 5)
        self.assertEqual(m[0][2], f'http://127.0.0.1:{t.port}', 'the stage gets SERVER')
        self.assertEqual(len(t.server_pids()), 2, 'one server per unit')
        self.assertFalse(any(alive(p) for p in t.server_pids()), 'every server stopped')
        srv = rd(t.p('markers', 'server')).splitlines()[0]
        self.assertIn(f'-m models/llm/stub.gguf --host 127.0.0.1 --port {t.port} -np 1 --kv-unified-per-slot 16384 '
                      '-fa on --cache-ram 0 --ctx-checkpoints 0 --no-webui -ngl all --reasoning off', srv)
        self.assertFalse(os.path.exists(t.lock), 'the lock is released')
        st = t.status()
        self.assertEqual(st['units']['a']['left'], 0)
        self.assertEqual(st['units']['a']['done'], 2)
        # STAGES_SERVER_LEAN=0 reproduces update.sh's server flags (the profile's slots, -cram 512).
        t2 = self.tree([stub('a', 0.1)], {'a': 1})
        r = t2.run('--once', '--duty', '1', STAGES_SERVER_LEAN='0')
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn(f'-m models/llm/stub.gguf --host 127.0.0.1 --port {t2.port} -np 2 --kv-unified-per-slot 16384 '
                      '-fa on -cram 512 --no-webui -ngl all --reasoning off', rd(t2.p('markers', 'server')))

    def test_a_duty_zero_stops_after_the_unit(self):
        """duty 0 written while a unit runs: that unit finishes, no second starts, the runner idles; --duty 0 runs none."""
        t = self.tree([stub('a', 2.0)], {'a': 3})
        p = t.start('--duty', '0.5')
        self.assertTrue(t.wait(lambda: t.markers('a')), 'the first unit starts')
        wr(os.path.join(t.state, 'duty'), '0\n')
        self.assertTrue(t.wait(lambda: t.status().get('state') == 'idle', 30), t.status())
        self.assertEqual(t.status().get('idleReason'), 'duty 0')
        time.sleep(2)
        self.assertEqual([x[0] for x in t.markers('a')], ['start', 'done'])
        self.assertFalse(any(alive(x) for x in t.server_pids()))
        p.send_signal(signal.SIGTERM)
        self.assertEqual(p.wait(30), 143)
        self.assertEqual(t.status().get('state'), 'stopped by signal')
        t2 = self.tree([stub('a', 0.1)], {'a': 3})
        r = t2.run('--once', '--duty', '0')
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertEqual(t2.markers('a'), [])
        self.assertEqual(t2.status().get('idleReason'), 'duty 0')

    def test_b_held_lock_waits_and_starts_no_server(self):
        holder = subprocess.Popen(['sleep', '120'])
        try:
            t = self.tree([stub('a', 0.5)], {'a': 1})
            wr(t.lock, f'{holder.pid} worker\n')
            p = t.start('--once', '--duty', '1')
            self.assertTrue(t.wait(lambda: t.status().get('state') == 'waiting-lock', 20), t.status())
            time.sleep(2)
            self.assertEqual(t.status()['lockHolder'], {'pid': holder.pid, 'label': 'worker'})
            self.assertEqual(t.server_pids(), [], 'no server while the lock is held')
            self.assertEqual(t.markers('a'), [])
            self.assertEqual(rd(t.lock), f'{holder.pid} worker\n', 'the lock is not stolen')
            holder.kill(); holder.wait()
            self.assertEqual(p.wait(60), 0)
            self.assertEqual([x[0] for x in t.markers('a')], ['start', 'done'])
            self.assertEqual(len(t.server_pids()), 1)
            self.assertFalse(os.path.exists(t.lock))
        finally:
            if holder.poll() is None:
                holder.kill(); holder.wait()

    def test_c_pause_file_idles(self):
        t = self.tree([stub('a', 0.5)], {'a': 1})
        os.makedirs(t.state)
        wr(os.path.join(t.state, 'pause'), '')
        p = t.start('--once', '--duty', '1')
        self.assertTrue(t.wait(lambda: t.status().get('state') == 'paused', 20), t.status())
        time.sleep(2)
        self.assertEqual(t.status().get('pauseReason'), 'pause file')
        self.assertEqual(t.markers('a'), [])
        self.assertEqual(t.server_pids(), [])
        os.remove(os.path.join(t.state, 'pause'))
        self.assertEqual(p.wait(60), 0)
        self.assertEqual([x[0] for x in t.markers('a')], ['start', 'done'])

    def test_c_pause_during_a_stage_stops_it_and_resumes(self):
        t = self.tree([stub('a', 4.0)], {'a': 1})
        p = t.start('--once', '--duty', '1')
        self.assertTrue(t.wait(lambda: t.markers('a')))
        wr(os.path.join(t.state, 'pause'), '')
        self.assertTrue(t.wait(lambda: t.status().get('state') == 'paused', 20), t.status())
        self.assertEqual([x[0] for x in t.markers('a')], ['start'], 'the stage was stopped before its unit finished')
        self.assertFalse(any(alive(x) for x in t.server_pids()), 'its server too')
        self.assertFalse(os.path.exists(t.lock), 'and the lock released')
        os.remove(os.path.join(t.state, 'pause'))
        self.assertEqual(p.wait(60), 0)
        self.assertEqual([x[0] for x in t.markers('a')], ['start', 'start', 'done'])

    def test_c_nightly_update_lock_pauses(self):
        """While trevor-vps-update.sh runs (its pid in update.lock, written under its flock), no stage starts; the
        canary gate needs the model lock. The probe reads the pid: it never takes the flock."""
        t = self.tree([stub('a', 0.5)], {'a': 1})
        holder = subprocess.Popen([sys.executable, '-c', 'import fcntl, os, sys, time; f = open(sys.argv[1], "a"); '
                                   'fcntl.flock(f, fcntl.LOCK_EX); open(sys.argv[1], "w").write(f"{os.getpid()} daily\\n"); '
                                   'print("held", flush=True); time.sleep(120)',
                                   t.p('update.lock'), 'trevor-vps-update.sh'], stdout=subprocess.PIPE, text=True)
        try:
            self.assertEqual(holder.stdout.readline().strip(), 'held')
            p = t.start('--once', '--duty', '1')
            self.assertTrue(t.wait(lambda: t.status().get('state') == 'paused', 20), t.status())
            self.assertEqual(t.status().get('pauseReason'), 'the nightly update holds update.lock')
            self.assertEqual(t.markers('a'), [])
            holder.kill(); holder.wait()
            self.assertEqual(p.wait(60), 0)
            self.assertEqual([x[0] for x in t.markers('a')], ['start', 'done'])
        finally:
            if holder.poll() is None:
                holder.kill(); holder.wait()
            holder.stdout.close()

    def test_b_empty_lock_waits_until_stale(self):
        """An empty lock file (the update between its noclobber create and its pid) is held: the runner waits and
        starts no server; once the file is older than LOCK_STALE_AGE it is stale and taken."""
        t = self.tree([stub('a', 0.2)], {'a': 1})
        wr(t.lock, '')
        p = t.start('--once', '--duty', '1')
        self.assertTrue(t.wait(lambda: t.status().get('state') == 'waiting-lock', 20), t.status())
        time.sleep(1.5)
        self.assertEqual(t.status()['lockHolder']['pid'], None)
        self.assertEqual(t.server_pids(), [], 'no server while an empty young lock exists')
        self.assertEqual(rd(t.lock), '', 'the young lock is not stolen')
        old = time.time() - stages.LOCK_STALE_AGE - 5
        os.utime(t.lock, (old, old))
        self.assertEqual(p.wait(60), 0)
        self.assertEqual([x[0] for x in t.markers('a')], ['start', 'done'])
        self.assertFalse(os.path.exists(t.lock))

    def test_e_server_wait_is_bounded(self):
        """Another server answering on the port: the wait under the model lock ends after STAGES_SERVER_WAIT, the lock
        is released and the stage records an error naming the reason."""
        t = self.tree([stub('a', 0.2)], {'a': 1})
        other = subprocess.Popen([t.p('stub_llama.py'), '--port', str(t.port)])
        try:
            self.assertTrue(t.wait(lambda: t.markers('server')))
            time.sleep(0.5)
            r = t.run('--once', '--duty', '1', STAGES_SERVER_WAIT='1.5')
            self.assertEqual(r.returncode, 1, r.stdout)
            self.assertEqual(t.markers('a'), [])
            self.assertFalse(os.path.exists(t.lock), 'the lock is released')
            self.assertIn('STAGES_SERVER_WAIT', t.status()['lastError'])
            self.assertEqual(len(t.server_pids()), 1, 'the runner started none')
        finally:
            other.kill(); other.wait()

    def test_f_failure_alerts_once_a_day(self):
        """A stage that fails STAGES_ALERT_AFTER times in a row posts one embed in post_embed's shape; a restart the
        same day posts none (the day is kept in status.json); logs go to a directory per day."""
        hook = Webhook()
        try:
            t = self.tree([{'id': 'bad', 'cmd': 'python3 stub_stage.py bad 0; [ "${PLAN:-}" = 1 ] || exit 7', 'server': None,
                            'plan': 'planned', 'pre': [], 'pre_plan': [], 'interruptible': True}], {'bad': 9})
            for _ in range(3):
                r = t.run('--once', '--duty', '1', DISCORD_ALERT_WEBHOOK=hook.url)
                self.assertEqual(r.returncode, 1, r.stdout)
            self.assertEqual(len(hook.posts), 1, hook.posts)
            post = hook.posts[0]
            self.assertEqual(post['type'], 'application/json')
            self.assertNotIn('Python-urllib', post['ua'] or '')
            e = post['body']['embeds']
            self.assertEqual(len(e), 1)
            self.assertEqual(sorted(e[0]), ['color', 'description', 'footer', 'title'])
            self.assertEqual(e[0]['title'], 'Trevor stages: bad failed')
            self.assertEqual(e[0]['color'], 13646394)
            self.assertIn('exit 7', e[0]['description'])
            self.assertTrue(e[0]['footer']['text'].startswith('stages_failed'))
            st = t.status()
            self.assertEqual(st['alerted'], {'bad': time.strftime('%Y-%m-%d')})
            self.assertEqual(st['failures'], {'bad': 3})
            self.assertEqual(os.listdir(os.path.join(t.state, 'logs')), [time.strftime('%Y%m%d')])
        finally:
            hook.close()

    def test_f_interruptions_without_progress_block_the_stage(self):
        """A stage paused mid-unit STAGES_MAX_INTERRUPTIONS times with no progress is blocked until tomorrow, with
        one embed; the runner then goes on (here: nothing else to do, so the pass ends)."""
        hook = Webhook()
        try:
            t = self.tree([stub('a', 6.0)], {'a': 1})
            p = t.start('--once', '--duty', '1', DISCORD_ALERT_WEBHOOK=hook.url, STAGES_MAX_INTERRUPTIONS='2')
            pause = os.path.join(t.state, 'pause')
            self.assertTrue(t.wait(lambda: len(t.markers('a')) == 1, 30), t.markers('a'))
            wr(pause, '')
            self.assertTrue(t.wait(lambda: t.status().get('state') == 'paused', 20), t.status())
            self.assertEqual(t.status()['interruptions'], {'a': 1})
            os.remove(pause)
            self.assertTrue(t.wait(lambda: len(t.markers('a')) == 2, 30), t.markers('a'))
            wr(pause, '')   # the second interruption blocks the stage, so the pass ends without waiting on the pause
            self.assertEqual(p.wait(60), 0)
            os.remove(pause)
            self.assertEqual([x[0] for x in t.markers('a')], ['start', 'start'], 'never ran to the end')
            st = t.status()
            self.assertEqual(st['blockedDay'], {'a': time.strftime('%Y-%m-%d')})
            self.assertEqual(len(hook.posts), 1)
            self.assertEqual(hook.posts[0]['body']['embeds'][0]['title'], 'Trevor stages: a blocked')
            r = t.run('--once', '--duty', '1', DISCORD_ALERT_WEBHOOK=hook.url)   # same day: still blocked
            self.assertEqual(r.returncode, 0)
            self.assertEqual(len(t.markers('a')), 2)
            self.assertEqual(len(hook.posts), 1)
        finally:
            hook.close()

    def test_reanswer_wiring(self):
        """Step 6 in the built-in table: collect before the plan, answer under the lock on its own server, then merge;
        off with REANSWER=0 and without a DSN. A stub reanswer.py; every other stage's script is absent (plan fails)."""
        t = self.tree([], {'reanswer': 1})
        os.makedirs(t.p('scripts'))
        wr(t.p('scripts', 'reanswer.py'), '''import os, sys
cmd = sys.argv[1]
cnt = 'counters/reanswer'
n = int(open(cnt).read())
if cmd == 'answer' and os.environ.get('PLAN') == '1':
    print(f'reanswer: plan {n}')
    sys.exit(0)
lock = open(os.environ['TREVOR_LLM_LOCK']).read().split() if os.path.exists(os.environ['TREVOR_LLM_LOCK']) else []
with open('markers/reanswer', 'a') as f:
    f.write(cmd + ' ' + (os.environ.get('SERVER', '') if cmd == 'answer' else '') + ' ' + ' '.join(lock[1:]) + '\\n')
if cmd == 'answer':
    open(cnt, 'w').write('0')
''')
        args = ['nice', '-n', '19', sys.executable, STAGES, '--root', t.root, '--state-dir', t.state, '--once',
                '--duty', '1']
        r = subprocess.run(args, env=t.env(REANSWER_DSN='postgres://stub'), capture_output=True, text=True, timeout=120)
        self.assertEqual(r.returncode, 1, 'the other groups\' scripts are absent, so their plans fail')
        lines = [l.split() for l in rd(t.p('markers', 'reanswer')).splitlines()]
        self.assertEqual([l[0] for l in lines[:3]], ['collect', 'answer', 'collect'])
        self.assertEqual(lines[1][1:], [f'http://127.0.0.1:{t.port}', 'stages', 'reanswer'],
                         'answer runs with SERVER under the runner\'s lock')
        self.assertEqual(lines[-1][0], 'merge')
        self.assertEqual(len(t.server_pids()), 1)
        self.assertIn('--reasoning off', rd(t.p('markers', 'server')))
        os.remove(t.p('markers', 'reanswer'))
        for env in ({'REANSWER_DSN': 'postgres://stub', 'REANSWER': '0'}, {}):
            wr(t.p('counters', 'reanswer'), '1')
            subprocess.run(args, env=t.env(**env), capture_output=True, text=True, timeout=120)
            self.assertFalse(os.path.exists(t.p('markers', 'reanswer')), env)

    def test_e_dry_run_runs_nothing(self):
        t = self.tree([stub('a', 0.1), {'id': 'hook', 'kind': 'hook'}], {'a': 1})
        r = t.run('--dry-run', '--once')
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn('PLAN=1 python3 stub_stage.py a 0.1', r.stdout)
        self.assertIn(f'SERVER=http://127.0.0.1:{t.port} python3 stub_stage.py a 0.1', r.stdout)
        self.assertIn(f'{t.p("stub_llama.py")} -m models/llm/stub.gguf --host 127.0.0.1 --port {t.port}', r.stdout)
        self.assertIn('hook: hook only, runs nothing', r.stdout)
        self.assertEqual(os.listdir(t.p('markers')), [])
        self.assertEqual(os.listdir(t.p('artifacts')), [])
        self.assertFalse(os.path.exists(t.state), 'no state written')
        self.assertFalse(os.path.exists(t.lock))
        # The built-in table on an empty tree: update.sh's commands printed, none run.
        empty = tempfile.mkdtemp(prefix='trevor-stages-empty-')
        try:
            r = subprocess.run(['nice', '-n', '19', sys.executable, STAGES, '--dry-run', '--once', '--root', empty],
                               env=t.env(), capture_output=True, text=True, timeout=60)
            self.assertEqual(r.returncode, 0, r.stderr)
            for c in ('PLAN=1 python3 scripts/p2-summaries.py story', 'python3 scripts/answer_bank.py answer',
                      'python3 scripts/topics.py mine', 'python3 scripts/canon_refs.py verdict gemma-on',
                      'PRIMERS=1 python3 scripts/answer_bank.py seed', 'python3 scripts/reanswer.py collect',
                      'SERVER=http://127.0.0.1:{} python3 scripts/reanswer.py answer'.format(t.port),
                      'python3 scripts/reanswer.py merge'):
                self.assertIn(c, r.stdout)
            self.assertEqual(os.listdir(empty), [])
        finally:
            shutil.rmtree(empty)

    def test_refuses_the_live_symlink_without_publish(self):
        t = self.tree([stub('a', 0.1)], {'a': 1})
        os.makedirs(t.p('release'))
        os.rmdir(t.p('artifacts'))
        os.symlink(t.p('release'), t.p('artifacts'))
        r = t.run('--once')
        self.assertEqual(r.returncode, 2)
        self.assertIn('symlink', r.stderr)
        self.assertEqual(t.markers('a'), [])
        r = t.run('--once', '--publish')
        self.assertEqual(r.returncode, 2)
        self.assertIn('TREVOR_PUBLISH=1', r.stderr)


class Window(unittest.TestCase):
    """The nightly window (design/trevor-memory-budget.md section 4) with the stand-in path unit."""

    def setUp(self):
        self.trees = []

    def tearDown(self):
        for t in self.trees:
            t.close()

    def tree(self, spec, counters, stop_delay=0.0):
        t = Tree(spec, counters)
        self.trees.append(t)
        os.makedirs(t.state)
        return t, PathUnit(t, stop_delay)

    def servers(self, t):
        return [l for l in rd(t.p('markers', 'server')).splitlines()] if t.markers('server') else []

    def test_w1_opens_waits_uses_the_window_model_and_restores_at_its_end(self):
        """In the window: `stop`, the unit waits for the serving check to clear (the stub takes 1 s), runs on the
        window model at the window duty (1: no sleep); the window's end stops the unit and writes `start`; the rest
        runs outside on STAGES_GEN at --duty 0.5."""
        t, pu = self.tree([stub('a', 1.5)], {'a': 3}, stop_delay=1.0)
        r = t.run('--once', '--duty', '0.5', timeout=120, **window_env(t, end_in=5))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr + t.runlog())
        self.assertEqual(pu.calls(), [STOP, START], t.runlog())
        self.assertEqual(rd(pu.marker), 'start\n')
        self.assertEqual(pu.events[0][1], 'stop')
        self.assertGreaterEqual(float(t.markers('a')[0][1]), pu.events[0][0], 'no unit before serving cleared')
        srv = self.servers(t)
        self.assertIn('-m models/llm/window.gguf ', srv[0])
        self.assertIn('-m models/llm/stub.gguf ', srv[-1])
        log = t.runlog()
        self.assertRegex(log, r'unit a: [\d.]+ s at duty 1; sleeping 0.00 s')
        self.assertRegex(log, r'unit a: [\d.]+ s at duty 0.5; sleeping [1-9]')
        self.assertIn('window: closed (the window ended)', log)
        st = t.status()
        self.assertEqual(st['units']['a']['left'], 0)
        self.assertIn(st['window']['state'], ('restoring', 'closed'))
        self.assertEqual(st['window']['model'], 'models/llm/stub.gguf')
        self.assertFalse(any(alive(p) for p in t.server_pids()))
        self.assertFalse(os.path.exists(t.lock))

    def test_w2_sigterm_mid_window_writes_start(self):
        t, pu = self.tree([stub('a', 30.0)], {'a': 1})
        p = t.start('--duty', '1', **window_env(t))
        self.assertTrue(t.wait(lambda: t.markers('a'), 30), t.status())
        self.assertEqual(pu.calls(), [STOP])
        self.assertEqual(rd(pu.marker), 'stop\n')
        self.assertEqual(t.status()['window']['state'], 'open')
        self.assertEqual(t.status()['window']['model'], 'models/llm/window.gguf')
        p.send_signal(signal.SIGTERM)
        self.assertEqual(p.wait(60), 143)
        self.assertEqual(rd(pu.marker), 'start\n')
        self.assertTrue(t.wait(lambda: pu.calls() == [STOP, START], 10), pu.calls())
        self.assertFalse(any(alive(x) for x in t.server_pids()), 'the 12B stopped before serving was asked back')
        st = t.status()
        self.assertEqual(st['state'], 'stopped by signal')
        self.assertEqual(st['window']['state'], 'restoring')
        self.assertEqual(st['window']['reason'], 'the runner stops')

    def test_w3_window_model_that_fails_to_start_restores_and_records(self):
        """The 12B exits while loading: `start`, the error in status.json, the night given up (no second `stop`); the
        next pass runs the stage outside the window on STAGES_GEN."""
        t, pu = self.tree([stub('a', 0.2)], {'a': 1})
        wr(t.p('bad_llama.py'), BAD_LLAMA)
        os.chmod(t.p('bad_llama.py'), 0o755)
        p = t.start('--duty', '1', STAGES_LLAMA_SERVER=t.p('bad_llama.py'), **window_env(t))
        self.assertTrue(t.wait(lambda: [x[0] for x in t.markers('a')] == ['start', 'done'], 40), t.runlog())
        p.send_signal(signal.SIGTERM)
        self.assertEqual(p.wait(60), 143)
        self.assertEqual(pu.calls(), [STOP, START], t.runlog())
        self.assertEqual(rd(pu.marker), 'start\n')
        st = t.status()
        self.assertIn('the window model did not start: llama-server exited (3)', st['window']['error'])
        self.assertIn('exit server', st['lastError'])
        srv = self.servers(t)
        self.assertEqual(len(srv), 1, 'only the outside server answered')
        self.assertIn('-m models/llm/stub.gguf ', srv[0])

    def test_w4_kill_switch_never_writes_the_marker(self):
        """STAGES_WINDOW=0, and the default without --publish: inside the window bounds nothing changes."""
        for off in ('0', None):
            t, pu = self.tree([stub('a', 0.2)], {'a': 1})
            env = window_env(t)
            if off is None:
                del env['STAGES_WINDOW']
            else:
                env['STAGES_WINDOW'] = off
            r = t.run('--once', '--duty', '1', **env)
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            time.sleep(0.3)
            self.assertFalse(os.path.exists(pu.marker), off)
            self.assertEqual(pu.calls(), [])
            self.assertIn('-m models/llm/stub.gguf ', self.servers(t)[0])
            self.assertNotIn('window', t.status())
            self.assertNotIn('window', t.runlog())

    def test_w5_window_pauses_for_the_nightly_update(self):
        """The update's lock mid-window: the unit stops, `start` (its canary asks through serving), the runner waits;
        the lock gone, `stop` again and the unit resumes on the window model; `start` at the end."""
        t, pu = self.tree([stub('a', 3.0)], {'a': 1})
        p = t.start('--once', '--duty', '1', **window_env(t))
        self.assertTrue(t.wait(lambda: t.markers('a'), 30), t.runlog())
        holder = subprocess.Popen([sys.executable, '-c', 'import fcntl, os, sys, time; f = open(sys.argv[1], "a"); '
                                   'fcntl.flock(f, fcntl.LOCK_EX); open(sys.argv[1], "w").write(f"{os.getpid()} daily\\n"); '
                                   'print("held", flush=True); time.sleep(120)',
                                   t.p('update.lock'), 'trevor-vps-update.sh'], stdout=subprocess.PIPE, text=True)
        try:
            self.assertEqual(holder.stdout.readline().strip(), 'held')
            self.assertTrue(t.wait(lambda: t.status().get('state') == 'paused', 20), t.status())
            self.assertTrue(t.wait(lambda: pu.calls() == [STOP, START], 10), pu.calls())
            self.assertEqual(t.status().get('pauseReason'), 'the nightly update holds update.lock')
            self.assertIn(t.status()['window']['state'], ('restoring', 'closed'))
            self.assertFalse(any(alive(x) for x in t.server_pids()))
            holder.kill(); holder.wait()
            self.assertEqual(p.wait(60), 0, t.runlog())
        finally:
            if holder.poll() is None:
                holder.kill(); holder.wait()
            holder.stdout.close()
        self.assertTrue(t.wait(lambda: len(pu.calls()) == 4, 10), pu.calls())
        self.assertEqual(pu.calls(), [STOP, START, STOP, START])
        self.assertEqual([x[0] for x in t.markers('a')], ['start', 'start', 'done'])
        srv = self.servers(t)
        self.assertEqual(len(srv), 2)
        self.assertTrue(all('-m models/llm/window.gguf ' in x for x in srv), srv)

    def test_w6_serving_not_back_posts_one_embed(self):
        """`start` written and the serving check still empty after STAGES_WINDOW_GRACE: one embed, not one a tick."""
        hook = Webhook()
        try:
            t, pu = self.tree([stub('a', 0.2)], {'a': 1})
            wr(t.p('start-fails'), '')
            p = t.start('--duty', '1', DISCORD_ALERT_WEBHOOK=hook.url, **window_env(t, STAGES_WINDOW_GRACE='1'))
            self.assertTrue(t.wait(lambda: hook.posts, 30), t.runlog())
            time.sleep(2.5)
            p.send_signal(signal.SIGTERM)
            self.assertEqual(p.wait(60), 143)
            self.assertEqual(len(hook.posts), 1, hook.posts)
            e = hook.posts[0]['body']['embeds'][0]
            self.assertEqual(e['title'], 'Trevor stages: serving not restored after the window')
            self.assertIn('STAGES_WINDOW_GRACE', e['description'])
            self.assertTrue(e['footer']['text'].startswith('stages_window'))
            self.assertIn('alert', t.status()['window'])
            self.assertEqual(pu.calls()[:2], [STOP, START])
        finally:
            hook.close()

    def test_w7_startup_restores_a_stop_left_behind(self):
        """A runner that finds `stop` (its predecessor died) writes `start` before anything else."""
        t, pu = self.tree([stub('a', 0.2)], {'a': 0})
        wr(pu.marker, 'stop\n')
        self.assertTrue(t.wait(lambda: pu.calls() == [STOP], 10))
        r = t.run('--once', '--duty', '1', **window_env(t))
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(rd(pu.marker), 'start\n')
        self.assertTrue(t.wait(lambda: pu.calls() == [STOP, START], 10), pu.calls())
        self.assertIn('a previous runner did not restore serving', t.runlog())


class RealTreePlan(unittest.TestCase):
    def test_d_plan_on_the_real_tree(self):
        """--plan over the real artifacts: every stage with a count, read-only, and no llama-server before or after."""
        if not os.path.exists(os.path.join(TREVOR, 'artifacts', 'p2')):
            self.skipTest('no real artifacts in this checkout')
        if llama_pids():
            self.skipTest('a llama-server runs already; the before/after check would mean nothing')
        state = os.path.join(TREVOR, 'artifacts', 'stages')
        existed = os.path.exists(state)
        env = {k: v for k, v in os.environ.items() if not k.startswith('STAGES_') and k != 'PLAN'}
        r = subprocess.run(['nice', '-n', '19', sys.executable, STAGES, '--plan'], cwd=TREVOR, env=env,
                           capture_output=True, text=True, timeout=900)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(llama_pids(), [], 'no llama-server after the plan')
        for g in stages.GROUPS:
            for st in g['stages']:
                self.assertRegex(r.stdout, rf'\n    {re.escape(st["id"])}: (\d+|off)', st['id'])
        self.assertRegex(r.stdout, r'units to do: \d+')
        self.assertEqual(os.path.exists(state), existed, '--plan writes no state')
        print('\n' + r.stdout, file=sys.stderr)


if __name__ == '__main__':
    unittest.main()
