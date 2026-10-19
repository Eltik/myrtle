#!/usr/bin/env python3
"""The paced stage runner, scripts/stages.py (design/trevor-self-contained-deploy.md step 5; 2026-10-19).

No model runs here. Each runner test builds a scratch tree with a stub stage (`stub_stage.py NAME SECONDS`: under
PLAN=1 it prints "NAME: plan N to do" from a counter file; a real run writes a start marker, sleeps, takes one off
the counter and writes a done marker) and a stub llama-server (`stub_llama.py`: answers /health on --port and records
its argv and pid), points the runner at them with --spec and STAGES_LLAMA_SERVER, and gives it its own lock file and
state dir. The one test on the real tree is `--plan`, which runs every stage's PLAN=1 dry run (read-only, no model) and
checks that no llama-server ran before or after.

  python3 -m unittest scripts/tests/test_stages.py      (from trevor/; stdlib only; about 3 minutes, most of it the
                                                         real tree's plan)
Scratch trees go under $TMPDIR.
"""
import json, os, re, shutil, signal, socket, subprocess, sys, tempfile, time, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TREVOR = os.path.dirname(SCRIPTS)
STAGES = os.path.join(SCRIPTS, 'stages.py')
sys.path.insert(0, SCRIPTS)
import stages  # noqa: E402

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

    def p(self, *parts):
        return os.path.join(self.root, *parts)

    def env(self, **kw):
        e = dict(os.environ)
        for k in list(e):
            if k.startswith('STAGES_') or k in ('PLAN', 'PORT', 'GEN', 'JUDGE', 'TREVOR_PUBLISH'):
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
            dead = subprocess.Popen(['true']); dead.wait()
            wr(p, f'{dead.pid} ask\n')
            self.assertIsNone(stages.lock_holder(p))
            self.assertFalse(os.path.exists(p), 'a dead holder\'s lock is removed')
            wr(p, f'{os.getpid()} wrapper\n')
            self.assertIsNone(stages.lock_holder(p), 'this process holds it: ours')
            self.assertTrue(os.path.exists(p))
            live = subprocess.Popen(['sleep', '30'])
            try:
                wr(p, f'{live.pid} worker\n')
                self.assertEqual(stages.lock_holder(p), (live.pid, 'worker'))
            finally:
                live.kill(); live.wait()
        finally:
            shutil.rmtree(d)

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
        self.assertIn(f'-m models/llm/stub.gguf --host 127.0.0.1 --port {t.port} -np 2 --kv-unified-per-slot 16384 '
                      '-fa on -cram 512 --no-webui -ngl all --reasoning off', srv)
        self.assertFalse(os.path.exists(t.lock), 'the lock is released')
        st = t.status()
        self.assertEqual(st['units']['a']['left'], 0)
        self.assertEqual(st['units']['a']['done'], 2)

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
        """While trevor-vps-update.sh holds update.lock (flock), no stage starts; the canary gate needs the model lock."""
        t = self.tree([stub('a', 0.5)], {'a': 1})
        holder = subprocess.Popen([sys.executable, '-c', 'import fcntl, sys, time; f = open(sys.argv[1], "a"); '
                                   'fcntl.flock(f, fcntl.LOCK_EX); print("held", flush=True); time.sleep(120)',
                                   t.p('update.lock')], stdout=subprocess.PIPE, text=True)
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
