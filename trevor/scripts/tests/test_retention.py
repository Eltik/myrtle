#!/usr/bin/env python3
"""Retention (scripts/retention.py; design/trevor-self-contained-deploy.md section 6 step 9; 2026-10-19).

Each test builds a scratch tree with fake logs, eval runs, spool files, releases and asset manifests whose ages are
set with os.utime (what `touch -t` does), runs retention.py on it and checks: each policy keeps what it must and lists
the rest; a dry run deletes nothing; --apply deletes exactly the listed entries; the eval index and the live symlink
target survive; the asset manifests need --apply-assets too; the exit codes are 0, 2 and 1 as documented. The last
test drives `trevor-vps-update.sh prune` and checks that the disk warning reaches a stand-in Discord webhook.
Class ReloadWait drives `daily`'s wait for the backend's deferred reload (backend 89f07593) through a harness with
stub build steps and a stand-in model lock naming a live `sleep`.

The scratch trees go under $RETENTION_SCRATCH when set (else the system temporary directory); nothing outside them is
read or written.

  python3 -m unittest scripts/tests/test_retention.py      (from trevor/; stdlib only, a few seconds)
"""
import http.server, json, os, shutil, stat, subprocess, sys, tempfile, threading, time, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RETENTION = os.path.join(SCRIPTS, 'retention.py')
UPDATE = os.path.join(SCRIPTS, 'trevor-vps-update.sh')
BASE = 1790000000          # the fake trees' clock: entry i is BASE + i hours
DAY = 86400


def stamp(i, fmt='%Y%m%dT%H%M%SZ'):
    return time.strftime(fmt, time.gmtime(BASE + i * 3600))


def snapshot(root):
    """Every path under ROOT (files, directories, symlinks), symlinks not followed."""
    out = set()
    for d, dirs, files in os.walk(root):
        for x in dirs + files:
            out.add(os.path.join(d, x))
    return out


class Tree:
    """trevor/ (design, eval, artifacts/update, artifacts/spool), home/ (logs, releases, current, state), assets/."""

    def __init__(self):
        self.root = tempfile.mkdtemp(prefix='trevor-retention-', dir=os.environ.get('RETENTION_SCRATCH') or None)
        self.trevor = self.p('trevor')
        self.home = self.p('home')
        self.assets = self.p('assets')
        self.expected = set()   # every entry the policy must list

    def p(self, *rel):
        return os.path.join(self.root, *rel)

    def write(self, path, text='x', age_days=None):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'w') as f:
            f.write(text)
        if age_days is not None:
            t = time.time() - age_days * DAY
            os.utime(path, (t, t))
        return path

    def full(self):
        """The tree every policy has something to say about."""
        # Logs of trevor-vps-update.sh: 35 runs; the newest 30 stay, and logs 0 and 1 made the current and the
        # previous release, so only 2, 3 and 4 are over the policy.
        for i in range(35):
            d = self.p('home', 'logs', f'{stamp(i)}-daily')
            self.write(os.path.join(d, 'run.log'))
            if i in (2, 3, 4):
                self.expected.add(d)
        # update.sh logs on a development tree: 33, the oldest 3 go.
        for i in range(33):
            d = self.p('trevor', 'artifacts', 'update', stamp(i, '%Y%m%d-%H%M%S'))
            self.write(os.path.join(d, 'steps.log'))
            if i < 3:
                self.expected.add(d)
        # Eval runs: 55; the newest 50 are 5..54. Run 1's label is named as a baseline in design/, run 2's id is named
        # in design/, run 3 is the newest of gold set B; 0 and 4 go.
        rows = []
        for i in range(55):
            rid = f'{BASE + i}-run{i}'
            self.write(self.p('trevor', 'eval', 'runs', rid + '.json'), json.dumps({'record': {'runId': rid}}))
            rows.append({'runId': rid, 'label': f'run{i}', 'goldsetSha': 'B' if i == 3 else 'A'})
            if i in (0, 4):
                self.expected.add(self.p('trevor', 'eval', 'runs', rid + '.json'))
        self.write(self.p('trevor', 'eval', 'runs.jsonl'), ''.join(json.dumps(r) + '\n' for r in rows))
        self.write(self.p('trevor', 'design', 'doc.md'),
                   '# A doc\n\nThe recall baseline is run1 (gold v2).\n\n'
                   f'An earlier run, `{BASE + 2}-run2`, is quoted here; run0 and run4 are only mentioned.\n')
        # Gate runs on the VPS: 52; the newest 50 stay, run 0 is named by a published version's gate report.
        grows = []
        for i in range(52):
            rid = f'{BASE + i}-gate-v{i}'
            self.write(self.p('home', 'state', 'gates', 'eval', 'runs', rid + '.json'), '{}')
            grows.append({'runId': rid, 'goldsetSha': 'G'})
            if i == 1:
                self.expected.add(self.p('home', 'state', 'gates', 'eval', 'runs', rid + '.json'))
        self.write(self.p('home', 'state', 'gates', 'eval', 'runs.jsonl'), ''.join(json.dumps(r) + '\n' for r in grows))
        self.write(self.p('home', 'state', 'versions', 'v0', 'activate.json'),
                   json.dumps({'gateReport': {'gates': {'recall': {'runId': f'{BASE}-gate-v0'}}}}))
        # The spool: one result 20 days old goes, 10 days and new stay.
        self.expected.add(self.write(self.p('trevor', 'artifacts', 'spool', 'job-a.json'), '{}', age_days=20))
        self.write(self.p('trevor', 'artifacts', 'spool', 'job-b.json'), '{}', age_days=10)
        self.write(self.p('trevor', 'artifacts', 'spool', 'job-c.json'), '{}')
        # Releases: 11; the newest 5 are 6..10; current -> r0 (a rollback), previous r1 (the version history.jsonl
        # names before r0's), stage base r2; 3, 4 and 5 go.
        names = [f'{stamp(i)}-v{i}' for i in range(11)]
        for i, n in enumerate(names):
            self.write(self.p('home', 'releases', n, 'artifacts', 'chunks.jsonl'), f'release {i}\n')
            if i in (3, 4, 5):
                self.expected.add(self.p('home', 'releases', n))
        os.symlink(self.p('home', 'releases', names[0], 'artifacts'), self.p('home', 'current'))
        order = names[2:] + [names[1], names[0]]
        self.write(self.p('home', 'state', 'releases.jsonl'),
                   ''.join(json.dumps({'version': n, 'release': self.p('home', 'releases', n)}) + '\n' for n in order))
        self.write(self.p('home', 'state', 'history.jsonl'),
                   ''.join(json.dumps({'version': n, 'how': 'daily'}) + '\n' for n in (names[5], names[1], names[0])))
        self.write(self.p('home', 'state', 'stage-base'), self.p('home', 'releases', names[2], 'artifacts') + '\n')
        # Asset manifests: four in en (4, 3, 2, 1 days old): the newest 2 stay. hot_update_list.idx and bundles are
        # never candidates; kr has one manifest.
        for k, age in enumerate((4, 3, 2, 1)):
            path = self.write(self.p('assets', 'ArkAssets', 'en', f'{k:032x}.idx'), age_days=age)
            if age > 2:
                self.expected.add(path)
        self.write(self.p('assets', 'ArkAssets', 'en', 'hot_update_list.idx'), age_days=10)
        self.write(self.p('assets', 'ArkAssets', 'en', 'anon', 'old.bin'), age_days=100)
        self.write(self.p('assets', 'ArkAssets', 'kr', 'f' * 32 + '.idx'), age_days=50)
        return self

    def run(self, *args, env=None):
        e = {k: v for k, v in os.environ.items()
             if not k.startswith(('RETAIN_', 'DISK_', 'TREVOR_', 'PRUNE', 'ASSETS_DIR'))}
        e['DISK_MIN_FREE_GB'] = '0'
        e.update(env or {})
        p = subprocess.run([sys.executable, RETENTION, '--trevor-dir', self.trevor, '--home', self.home,
                            '--assets-dir', self.assets, *args], capture_output=True, text=True, env=e)
        lines = p.stdout.strip().splitlines()
        summary = json.loads(lines[-1]) if lines and lines[-1].startswith('{') else None
        return p.returncode, summary, p.stdout + p.stderr

    def listed(self, summary):
        return {x for s in summary['policies'].values() for x in s['listed']}


def read(path):
    with open(path) as f:
        return f.read()


class Retention(unittest.TestCase):
    def setUp(self):
        self.t = Tree()

    def tearDown(self):
        for d, dirs, _ in os.walk(self.t.root):
            for x in dirs:
                os.chmod(os.path.join(d, x), 0o755)
        shutil.rmtree(self.t.root, ignore_errors=True)

    def test_dry_run_lists_the_policy_and_deletes_nothing(self):
        self.t.full()
        before = snapshot(self.t.root)
        code, s, out = self.t.run()
        self.assertEqual(code, 2, out)
        self.assertEqual(self.t.listed(s), self.t.expected, out)
        self.assertEqual(snapshot(self.t.root), before)
        per = {k: v['over'] for k, v in s['policies'].items()}
        self.assertEqual(per, {'logs': 6, 'runs': 3, 'spool': 1, 'releases': 3, 'assets': 2})
        self.assertEqual(s['deleted'], 0)
        for why in ('log of the current release', 'log of the previous release', 'named in design/',
                    'its label is named as a baseline in design/', 'the newest run of its gold set',
                    "named by a published version's gate report", 'the current release (the live symlink target)',
                    'the previous release', "the stage's base"):
            self.assertIn(why, out)

    def test_apply_deletes_exactly_the_listed_entries(self):
        self.t.full()
        _, dry, _ = self.t.run()
        before = snapshot(self.t.root)
        code, s, out = self.t.run('--apply', '--apply-assets')
        self.assertEqual(code, 2, out)
        self.assertEqual(self.t.listed(s), self.t.expected)
        gone = before - snapshot(self.t.root)
        top = {g for g in gone if os.path.dirname(g) not in gone}
        self.assertEqual(top, self.t.listed(dry))
        self.assertEqual(s['deleted'], len(self.t.expected))
        # The index and the live release survive; nothing is over the policy afterwards.
        self.assertTrue(os.path.isfile(self.t.p('trevor', 'eval', 'runs.jsonl')))
        self.assertTrue(os.path.isfile(self.t.p('home', 'state', 'gates', 'eval', 'runs.jsonl')))
        self.assertTrue(os.path.isfile(os.path.join(self.t.p('home', 'current'), 'chunks.jsonl')))
        self.assertTrue(os.path.isfile(self.t.p('assets', 'ArkAssets', 'en', 'anon', 'old.bin')))
        self.assertTrue(os.path.isfile(self.t.p('assets', 'ArkAssets', 'en', 'hot_update_list.idx')))
        code, s, out = self.t.run()
        self.assertEqual((code, s['overPolicy']), (0, 0), out)

    def test_asset_manifests_need_their_own_flag(self):
        self.t.full()
        code, s, out = self.t.run('--apply')
        self.assertEqual(code, 2)
        assets = {x for x in self.t.expected if x.endswith('.idx')}
        self.assertEqual(len(assets), 2)
        for x in assets:
            self.assertTrue(os.path.exists(x))
        for x in self.t.expected - assets:
            self.assertFalse(os.path.lexists(x), x)
        self.assertIn('needs --apply --apply-assets', out)
        code, s, _ = self.t.run('--apply')
        self.assertEqual((code, s['overPolicy'], s['deleted']), (2, 2, 0))

    def test_a_tree_within_the_policy_exits_0(self):
        self.t.write(self.t.p('home', 'logs', f'{stamp(0)}-daily', 'run.log'))
        self.t.write(self.t.p('trevor', 'eval', 'runs', f'{BASE}-a.json'))
        self.t.write(self.t.p('trevor', 'artifacts', 'spool', 'new.json'))
        os.makedirs(self.t.p('home', 'releases', f'{stamp(0)}-v0', 'artifacts'))
        code, s, out = self.t.run()
        self.assertEqual((code, s['overPolicy']), (0, 0), out)
        code, s, out = self.t.run('--apply', '--apply-assets')
        self.assertEqual((code, s['deleted']), (0, 0), out)

    def test_the_symlink_target_is_never_deleted(self):
        self.t.full()
        # The live release is also the oldest name and outside the newest N; a symlinked log entry points outside.
        outside = self.t.write(self.t.p('elsewhere', 'keep.txt'))
        link = self.t.p('home', 'logs', '00000000T000000Z-daily')
        os.symlink(os.path.dirname(outside), link)
        code, s, out = self.t.run('--apply', '--apply-assets', env={'RETAIN_RELEASES': '1', 'RETAIN_LOGS': '1'})
        self.assertEqual(code, 2, out)
        self.assertFalse(os.path.lexists(link))
        self.assertTrue(os.path.isfile(outside))
        self.assertTrue(os.path.isfile(os.path.join(self.t.p('home', 'current'), 'chunks.jsonl')))
        left = sorted(os.listdir(self.t.p('home', 'releases')))
        self.assertEqual(left, [f'{stamp(i)}-v{i}' for i in (0, 1, 2, 10)])
        self.assertEqual(len(os.listdir(self.t.p('home', 'logs'))), 3)   # newest, current's, previous's

    def test_a_log_root_inside_a_release_is_left_to_the_release_policy(self):
        rel = self.t.p('home', 'releases', f'{stamp(0)}-v0', 'artifacts')
        for i in range(40):
            self.t.write(os.path.join(rel, 'update', stamp(i, '%Y%m%d-%H%M%S'), 'steps.log'))
        os.symlink(rel, self.t.p('home', 'current'))
        os.makedirs(self.t.trevor)
        os.symlink(self.t.p('home', 'current'), self.t.p('trevor', 'artifacts'))
        code, s, out = self.t.run('--only', 'logs,releases')
        self.assertEqual((code, s['overPolicy']), (0, 0), out)
        self.assertIn('inside a release directory', out)

    def test_errors_exit_1(self):
        self.t.full()
        code, _, out = self.t.run(env={'RETAIN_LOGS': 'many'})
        self.assertEqual(code, 1, out)
        self.assertIn('RETAIN_LOGS', out)
        code, _, out = self.t.run('--only', 'logs,everything')
        self.assertEqual(code, 1, out)
        # A listed entry that cannot be deleted: the rest still go, and the run exits 1.
        spool = self.t.p('trevor', 'artifacts', 'spool')
        os.chmod(spool, stat.S_IRUSR | stat.S_IXUSR)
        code, s, out = self.t.run('--apply')
        os.chmod(spool, 0o755)
        self.assertEqual(code, 1, out)
        self.assertEqual(s['errors'], 1)
        self.assertTrue(os.path.exists(os.path.join(spool, 'job-a.json')))
        self.assertFalse(os.path.lexists(self.t.p('home', 'releases', f'{stamp(3)}-v3')))

    def test_disk_warning(self):
        os.makedirs(self.t.home)
        code, s, out = self.t.run('--only', 'disk', env={'DISK_MIN_FREE_GB': '1e9', 'DISK_MIN_FREE_PCT': '100'})
        self.assertEqual(code, 0, out)
        self.assertEqual(len(s['diskLow']), 1)
        self.assertRegex(out, r'== \d\d:\d\d:\d\d WARNING: disk low: \S+ has [\d.]+ GB free of [\d.]+ GB')
        code, s, out = self.t.run('--only', 'disk', env={'DISK_MIN_FREE_GB': '1e9', 'DISK_MIN_FREE_PCT': '0'})
        self.assertEqual((code, s['diskLow']), (0, []), out)   # the smaller threshold wins

    def test_update_script_prune_and_disk_alert(self):
        """`trevor-vps-update.sh prune`: a dry run unless PRUNE_APPLY=1, and the disk warning posted once."""
        self.t.full()
        posts = []

        class Hook(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                posts.append(json.loads(self.rfile.read(int(self.headers['content-length']))))
                self.send_response(204)
                self.end_headers()

            def log_message(self, *a):
                pass

        srv = http.server.HTTPServer(('127.0.0.1', 0), Hook)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        os.symlink(SCRIPTS, self.t.p('trevor', 'scripts'))
        env = {k: v for k, v in os.environ.items() if not k.startswith(('RETAIN_', 'DISK_', 'TREVOR_', 'PRUNE'))}
        env.update({'TREVOR_DIR': self.t.trevor, 'TREVOR_HOME': self.t.home, 'ASSETS_DIR': self.t.assets,
                    'TREVOR_NICED': '1', 'DISCORD_ALERT_WEBHOOK': f'http://127.0.0.1:{srv.server_port}/',
                    'DISK_MIN_FREE_GB': '1e9', 'DISK_MIN_FREE_PCT': '100', 'TMPDIR': self.t.root})
        try:
            before = snapshot(self.t.root)
            p = subprocess.run(['bash', UPDATE, 'prune'], capture_output=True, text=True, env=env)
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertIn('prune: retention.py --only logs,runs,spool,releases,assets', p.stdout)
            self.assertIn('dry run: nothing deleted', p.stdout)
            self.assertEqual(before - snapshot(self.t.root), set())   # nothing deleted (the run lock file is new)
            self.assertEqual(len(posts), 1)
            embed = posts[0]['embeds'][0]
            self.assertEqual(embed['title'], 'Trevor: disk space is low')
            self.assertIn('WARNING: disk low', embed['description'])
            self.assertTrue(embed['footer']['text'].startswith('disk_low'))
            p = subprocess.run(['bash', UPDATE, 'prune'], capture_output=True, text=True,
                               env={**env, 'PRUNE_APPLY': '1', 'DISK_CHECK': '0'})
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            assets = {x for x in self.t.expected if x.endswith('.idx')}
            for x in self.t.expected - assets:
                self.assertFalse(os.path.lexists(x), x)
            for x in assets:
                self.assertTrue(os.path.exists(x), x)
            self.assertEqual(len(posts), 1)   # DISK_CHECK=0: no second post
            # Kill switches: PRUNE=0 and DISK_CHECK=0 leave only the lines the script printed before 2026-10-19.
            before = snapshot(self.t.root)
            p = subprocess.run(['bash', UPDATE, 'prune'], capture_output=True, text=True,
                               env={**env, 'PRUNE': '0', 'DISK_CHECK': '0', 'PRUNE_APPLY': '1'})
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertNotIn('retention', p.stdout)
            self.assertNotIn('disk', p.stdout)
            self.assertEqual(before - snapshot(self.t.root), set())
            self.assertEqual(len(posts), 1)
            # `daily` checks the disk on every run, before its "up to date" exit (PLAN=1 posts nothing).
            os.makedirs(self.t.p('trevor', 'artifacts'), exist_ok=True)
            shutil.rmtree(self.t.p('trevor', 'artifacts'))
            os.symlink(self.t.p('home', 'current'), self.t.p('trevor', 'artifacts'))
            self.t.write(self.t.p('assets', 'ArkAssets', 'en', '.version'), 'res-1\n')
            self.t.write(self.t.p('home', 'state', 'active.json'), json.dumps({'version': 'v0', 'builtFrom': 'res-1'}))
            p = subprocess.run(['bash', UPDATE, 'daily'], capture_output=True, text=True, env={**env, 'PLAN': '1'})
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertIn('WARNING: disk low', p.stdout)
            self.assertIn('up to date', p.stdout)
            self.assertLess(p.stdout.index('WARNING: disk low'), p.stdout.index('up to date'))
            self.assertNotIn('prune:', p.stdout)
            self.assertEqual(len(posts), 1)
            # A disk with room posts nothing: under bash 3.2 grep's empty match ran the ERR trap inside disk_check's
            # substitution, which posted "update failed: start" and then "disk space is low" (chaos run 2026-10-19).
            p = subprocess.run(['bash', UPDATE, 'daily'], capture_output=True, text=True,
                               env={**env, 'DISK_MIN_FREE_GB': '0', 'DISK_MIN_FREE_PCT': '0'})
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertIn('up to date', p.stdout)
            self.assertNotIn('update_failed', p.stdout + p.stderr)
            self.assertEqual(len(posts), 1)
        finally:
            srv.shutdown()
            srv.server_close()

    # ------------------------------------------------------------ the publish path (chaos run 2026-10-19, 5, 6, 8)
    def release_tree(self):
        """home/releases/<initial> live through home/current and trevor/artifacts, the real scripts as
        trevor/scripts, a stage whose scripts/publish.py is a stub (it logs its argv; `--activate V` fails under
        STUB_FAIL=1, else appends {version, how} to history.jsonl and writes active.json, as publish.py's record
        does), and a harness that sources trevor-vps-update.sh up to its command dispatch and runs publish_stage."""
        t = self.t
        init = t.p('home', 'releases', f'{stamp(0)}-initial')
        t.write(os.path.join(init, 'artifacts', 'chunks.jsonl'), 'initial\n')
        os.symlink(os.path.join(init, 'artifacts'), t.p('home', 'current'))
        os.makedirs(t.trevor)
        os.symlink(SCRIPTS, t.p('trevor', 'scripts'))
        os.symlink(t.p('home', 'current'), t.p('trevor', 'artifacts'))
        os.makedirs(t.p('home', 'state'))
        t.write(t.p('home', 'stage', 'trevor', 'artifacts', 'chunks.jsonl'), 'stage\n')
        t.write(t.p('home', 'stage', 'trevor', 'scripts', 'publish.py'), """import json, os, sys
a = sys.argv[1:]
open(os.environ['STUB_LOG'], 'a').write(json.dumps(a) + '\\n')
if '--activate' in a:
    if os.environ.get('STUB_FAIL') == '1':
        sys.exit(1)
    st = os.environ['TREVOR_STATE_DIR']
    v, how = a[a.index('--activate') + 1], (a[a.index('--how') + 1] if '--how' in a else None)
    open(os.path.join(st, 'history.jsonl'), 'a').write(json.dumps({'version': v, 'how': how}) + '\\n')
    json.dump({'version': v}, open(os.path.join(st, 'active.json'), 'w'))
""")
        t.write(t.p('harness.sh'), """set -u
sed '/^case "$CMD" in/,$d' "$1" > "$HARNESS_LIB"
U="$1"; shift
source "$HARNESS_LIB" "$@"
STAMP="$H_STAMP"; WORK="$STAGE/trevor"
publish_stage
log "harness: publish_stage returned"
""")
        self.stub_log = t.p('stub.log')
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('RETAIN_', 'DISK_', 'TREVOR_', 'PRUNE', 'KEEP_', 'DISCORD_', 'STUB_'))}
        self.env.update({'TREVOR_DIR': t.trevor, 'TREVOR_HOME': t.home, 'ASSETS_DIR': t.assets, 'TREVOR_NICED': '1',
                         'TREVOR_WORKER_KEY': 'k', 'WORKER_SETTLE': '0', 'TREVOR_WARM_TOP': '0', 'PRUNE': '0',
                         'TMPDIR': t.root, 'STUB_LOG': self.stub_log, 'HARNESS_LIB': t.p('lib.sh')})
        return init

    def publish(self, i, version, fail=False, cmd='daily'):
        """publish_stage of VERSION under release stamp I; the activate fails under FAIL."""
        self.t.write(self.t.p('home', 'stage', 'trevor', 'artifacts', 'panels', 'manifest.json'),
                     json.dumps({'version': version, 'builtFrom': f'res-{version}'}))
        p = subprocess.run(['bash', self.t.p('harness.sh'), UPDATE, cmd], capture_output=True, text=True,
                           env={**self.env, 'H_STAMP': stamp(i), 'STUB_FAIL': '1' if fail else '0'})
        return p.returncode, p.stdout + p.stderr

    def rel(self, i, version):
        return self.t.p('home', 'releases', f'{stamp(i)}-{version}')

    def history(self):
        with open(self.t.p('home', 'state', 'history.jsonl')) as f:
            return [json.loads(l) for l in f if l.strip()]

    def test_never_live_releases_never_prune_the_previous_live_one(self):
        """One good publish, three builds whose activate failed (promoted, flipped back, never live), one more good
        publish: KEEP_RELEASES=3 keeps the live release, the previous live one (v0's) and the newest other; before
        the fix the three never-live releases pushed v0's out and `rollback` found no release for v0."""
        init = self.release_tree()
        self.assertEqual(self.publish(1, 'v0')[0], 0)
        for i, v in ((2, 'v1'), (3, 'v2'), (4, 'v3')):
            code, out = self.publish(i, v, fail=True)
            self.assertEqual(code, 1, out)
            self.assertIn('activate failed; the worker is back on', out)
        code, out = self.publish(5, 'v4')
        self.assertEqual(code, 0, out)
        self.assertIn('keeping release ' + self.rel(1, 'v0') + ' (the previous live release)', out)
        self.assertEqual(os.path.realpath(self.t.p('home', 'current')), os.path.realpath(self.rel(5, 'v4') + '/artifacts'))
        kept = sorted(os.listdir(self.t.p('home', 'releases')))
        self.assertEqual(kept, sorted(os.path.basename(x) for x in (self.rel(1, 'v0'), self.rel(4, 'v3'), self.rel(5, 'v4'))))
        self.assertFalse(os.path.exists(init))
        # history.jsonl names the command: a daily publish records "daily" (it read "rollback" for every row).
        self.assertEqual([(r['version'], r['how']) for r in self.history()], [('v0', 'daily'), ('v4', 'daily')])
        with open(self.stub_log) as f:
            calls = [json.loads(l) for l in f]
        self.assertIn(['--activate', 'v4', '--how', 'daily'], calls)
        # `rollback` (no VERSION) finds v0's release.
        p = subprocess.run(['bash', UPDATE, 'rollback'], capture_output=True, text=True, env={**self.env, 'PLAN': '1'})
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        self.assertIn(f"rollback to v0 (release {self.rel(1, 'v0')})", p.stdout)
        # The real rollback against a stand-in backend: the worker goes to v0's release and publish.py records
        # "rollback".
        posts = []

        class Backend(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                posts.append(self.rfile.read(int(self.headers['content-length'])))
                body = json.dumps({'version': 'v0', 'server': 'en', 'panels': 0, 'active': True}).encode()
                self.send_response(200)
                self.send_header('content-length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *a):
                pass

        srv = http.server.HTTPServer(('127.0.0.1', 0), Backend)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        try:
            for v in ('v0', 'v4'):
                self.t.write(self.t.p('home', 'state', 'versions', v, 'activate.json'),
                             json.dumps({'version': v, 'activate': True, 'panels': {}, 'manifest': {'builtFrom': 'r'}}))
            env = {**self.env, 'TREVOR_BACKEND': f'http://127.0.0.1:{srv.server_port}',
                   'TREVOR_STATE_DIR': self.t.p('home', 'state')}
            p = subprocess.run(['bash', UPDATE, 'rollback'], capture_output=True, text=True, env=env)
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertEqual(os.path.realpath(self.t.p('home', 'current')),
                             os.path.realpath(self.rel(1, 'v0') + '/artifacts'))
            self.assertEqual(self.history()[-1]['how'], 'rollback')
            # publish.py itself: --how daily records "daily"; --activate without --how stays "rollback".
            p = subprocess.run([sys.executable, os.path.join(SCRIPTS, 'publish.py'), '--activate', 'v4', '--how', 'daily'],
                               capture_output=True, text=True, env=env)
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            p = subprocess.run([sys.executable, os.path.join(SCRIPTS, 'publish.py'), '--activate', 'v0'],
                               capture_output=True, text=True, env=env)
            self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
            self.assertEqual([(r['version'], r['how']) for r in self.history()[-3:]],
                             [('v0', 'rollback'), ('v4', 'daily'), ('v0', 'rollback')])
            self.assertEqual(len(posts), 3)
        finally:
            srv.shutdown()
            srv.server_close()

    def test_retention_previous_release_is_the_previous_live_one(self):
        """retention.py's release policy: the previous release is v0's (the version history.jsonl names before the
        current one), not the newest releases.jsonl row (a never-live release)."""
        t = self.t
        rows, names = [], []
        for i, v in enumerate(('v0', 'v1', 'v2', 'v3', 'v4')):
            n = f'{stamp(i + 1)}-{v}'
            names.append(n)
            t.write(t.p('home', 'releases', n, 'artifacts', 'chunks.jsonl'), v + '\n')
            rows.append({'version': v, 'release': t.p('home', 'releases', n)})
        t.write(t.p('home', 'releases', f'{stamp(0)}-initial', 'artifacts', 'chunks.jsonl'), 'initial\n')
        os.symlink(t.p('home', 'releases', names[4], 'artifacts'), t.p('home', 'current'))
        t.write(t.p('home', 'state', 'releases.jsonl'), ''.join(json.dumps(r) + '\n' for r in rows))
        t.write(t.p('home', 'state', 'history.jsonl'),
                ''.join(json.dumps({'version': v, 'how': 'daily'}) + '\n' for v in ('v0', 'v4')))
        os.makedirs(t.trevor)
        code, s, out = t.run('--only', 'releases', env={'RETAIN_RELEASES': '2'})
        self.assertEqual(code, 2, out)
        self.assertEqual(set(s['policies']['releases']['listed']),
                         {t.p('home', 'releases', x) for x in (f'{stamp(0)}-initial', names[1], names[2])})
        self.assertIn(f"keep  {t.p('home', 'releases', names[0])}  (the previous release)", out)
        p = subprocess.run([sys.executable, RETENTION, '--trevor-dir', t.trevor, '--home', t.home, '--previous-release'],
                           capture_output=True, text=True)
        self.assertEqual((p.returncode, p.stdout.strip()), (0, os.path.realpath(t.p('home', 'releases', names[0]))))
        # No history row before the current one: no previous release (a never-live release is never it).
        t.write(t.p('home', 'state', 'history.jsonl'), json.dumps({'version': 'v4'}) + '\n')
        p = subprocess.run([sys.executable, RETENTION, '--trevor-dir', t.trevor, '--home', t.home, '--previous-release'],
                           capture_output=True, text=True)
        self.assertEqual((p.returncode, p.stdout.strip()), (0, ''))

    def test_a_stale_restart_tmp_never_flips(self):
        """A worker-restart.tmp left by a failed flip: the next run removes it (logged), and a run that then fails
        at the flip leaves `current` untouched (the same symlink inode: it never flipped and flipped back)."""
        self.release_tree()
        self.assertEqual(self.publish(1, 'v0')[0], 0)
        cur, state = self.t.p('home', 'current'), self.t.p('home', 'state')
        tmp, marker = os.path.join(state, 'worker-restart.tmp'), os.path.join(state, 'worker-restart')
        before = (os.lstat(cur).st_ino, os.readlink(cur), read(marker))
        try:
            # 1. home read-only: the symlink flip itself fails.
            self.t.write(tmp, 'stale\n')
            os.chmod(self.t.home, 0o555)
            code, out = self.publish(2, 'v1')
            os.chmod(self.t.home, 0o755)
            self.assertNotEqual(code, 0, out)
            self.assertIn('removing a stale ' + tmp, out)
            self.assertNotIn('is back on', out)
            self.assertNotIn('harness: publish_stage returned', out)
            self.assertEqual((os.lstat(cur).st_ino, os.readlink(cur), read(marker)), before)
            self.assertFalse(os.path.lexists(tmp), 'the failed flip removes its temp file')
            # 2. state read-only with a writable leftover .tmp (the chaos run's `home/state` case): before the fix the
            # open succeeded, `current` flipped and flipped back; now the flip fails before anything moves.
            self.t.write(tmp, 'stale\n')
            os.chmod(state, 0o555)
            code, out = self.publish(3, 'v2')
            os.chmod(state, 0o755)
            self.assertNotEqual(code, 0, out)
            self.assertIn('could not remove ' + tmp, out)
            self.assertNotIn('is back on', out)
            self.assertNotIn('harness: publish_stage returned', out)
            self.assertEqual((os.lstat(cur).st_ino, os.readlink(cur), read(marker)), before)
        finally:
            os.chmod(self.t.home, 0o755)
            os.chmod(state, 0o755)
        # The next run with room publishes normally.
        os.remove(tmp)
        self.assertEqual(self.publish(4, 'v3')[0], 0)
        self.assertEqual(os.path.realpath(cur), os.path.realpath(self.rel(4, 'v3') + '/artifacts'))


class ReloadWait(unittest.TestCase):
    """`daily` waits for the backend's game-data reload before any fetch (backend 89f07593 defers a reload while the
    Trevor model lock names a live pid). Exact when /api/health has gameData.loaded.en.version; without it the script
    reads the guard's own inputs. Here the model lock names a live `sleep` (the stand-in for the stage runner's 12B), a
    stub /api/health answers `self.health(n)` for its n-th request (by default a backend before the field), and the
    harness sources the script up to its command dispatch, replaces the four build steps with stubs that log, and runs
    take_run_lock then cmd_daily. RELOAD_RETRY_SECS=1 and RELOAD_BUILD_SECS=0 stand in for the backend's 300 s retry."""

    def setUp(self):
        self.t = Tree()
        t = self.t
        init = t.p('home', 'releases', f'{stamp(0)}-initial')
        t.write(os.path.join(init, 'artifacts', 'chunks.jsonl'), 'initial\n')
        os.symlink(os.path.join(init, 'artifacts'), t.p('home', 'current'))
        os.makedirs(t.trevor)
        os.symlink(SCRIPTS, t.p('trevor', 'scripts'))
        os.symlink(t.p('home', 'current'), t.p('trevor', 'artifacts'))
        t.write(t.p('assets', 'ArkAssets', 'en', '.version'), 'res-2\n', age_days=1)
        t.write(t.p('home', 'state', 'active.json'), json.dumps({'version': 'v1', 'builtFrom': 'res-1'}))
        self.lock = t.p('llm.lock')
        self.state = t.p('home', 'state', 'reload-wait.json')
        t.write(t.p('harness.sh'), """set -u
sed '/^case "$CMD" in/,$d' "$1" > "$HARNESS_LIB"
shift
source "$HARNESS_LIB" "$@"
prepare_stage() { log "STUB prepare_stage"; }
deterministic_build() { log "STUB deterministic_build $*"; }
panels_and_gates() { log "STUB panels_and_gates $*"; }
publish_stage() { log "STUB publish_stage"; }
take_run_lock
cmd_daily
log "harness: cmd_daily returned"
""")
        self.posts = []
        posts = self.posts
        self.health_gets = 0
        self.health = lambda n: {'status': 'ok', 'gameData': {'degraded': False, 'tables': {}, 'stale': {}}}
        case = self

        class Hook(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                posts.append(json.loads(self.rfile.read(int(self.headers['content-length']))))
                self.send_response(204)
                self.end_headers()

            def do_GET(self):
                if self.path != '/api/health':
                    self.send_response(404)
                    self.end_headers()
                    return
                case.health_gets += 1
                body = json.dumps(case.health(case.health_gets)).encode()
                self.send_response(200)
                self.send_header('content-type', 'application/json')
                self.send_header('content-length', str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *a):
                pass

        self.srv = http.server.HTTPServer(('127.0.0.1', 0), Hook)
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()
        self.holder = subprocess.Popen(['sleep', '300'])
        self.env = {k: v for k, v in os.environ.items()
                    if not k.startswith(('RETAIN_', 'DISK_', 'TREVOR_', 'PRUNE', 'KEEP_', 'DISCORD_', 'BACKEND_',
                                         'RELOAD_', 'PLAN', 'FORCE', 'SKIP_FETCH'))}
        self.env.update({'TREVOR_DIR': t.trevor, 'TREVOR_HOME': t.home, 'ASSETS_DIR': t.assets, 'TREVOR_NICED': '1',
                         'TMPDIR': t.root, 'HARNESS_LIB': t.p('lib.sh'), 'TREVOR_LLM_LOCK': self.lock,
                         'DISCORD_ALERT_WEBHOOK': f'http://127.0.0.1:{self.srv.server_port}/',
                         'TREVOR_BACKEND': f'http://127.0.0.1:{self.srv.server_port}',
                         'DISK_CHECK': '0', 'STALE_CHECK': '0', 'WATCHER_MAX_AGE_DAYS': '0',
                         'RELOAD_RETRY_SECS': '1', 'RELOAD_BUILD_SECS': '0', 'BACKEND_RELOAD_POLL_SECS': '1'})

    def tearDown(self):
        self.holder.kill()
        self.holder.wait()
        self.srv.shutdown()
        self.srv.server_close()
        shutil.rmtree(self.t.root, ignore_errors=True)

    def daily(self, script=UPDATE, **env):
        p = subprocess.run(['bash', self.t.p('harness.sh'), script, 'daily'], capture_output=True, text=True,
                           env={**self.env, **env}, timeout=120)
        return p.returncode, p.stdout + p.stderr

    def hold(self):
        self.t.write(self.lock, f'{self.holder.pid} stages\n')

    @staticmethod
    def serving(version):
        """An /api/health body from a backend with gameData.loaded, serving `version` for EN."""
        return {'status': 'ok', 'gameData': {'degraded': False, 'tables': {}, 'stale': {}, 'loaded': {
            'en': {'version': version, 'loadedAt': '2026-10-19T08:30:00.000Z', 'lazy': False},
            'jp': {'version': None, 'loadedAt': None, 'lazy': True}}}}

    def test_exact_version_proceeds_on_the_first_match(self):
        """/api/health serves res-1 for 3 requests, then res-2: the wait passes on the 4th, with the lock held the
        whole time (the inference would never pass), and the log names the exact mode."""
        self.hold()
        self.health = lambda n: self.serving('res-1' if n <= 3 else 'res-2')
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='1')
        self.assertEqual(code, 0, out)
        self.assertIn('reload wait mode exact (/api/health gameData.loaded.en.version): proceeding when it reports '
                      'res-2', out)
        self.assertIn('/api/health reports EN res-1', out)
        self.assertIn('the backend serves res-2 (gameData.loaded.en.version); proceeding', out)
        self.assertNotIn('inference', out)
        self.assertNotIn('live pid', out)
        self.assertEqual(self.health_gets, 4)
        self.assertLess(out.index('the backend serves res-2'), out.index('STUB prepare_stage'))
        self.assertIn('STUB publish_stage', out)
        self.assertFalse(os.path.exists(self.state))
        # PLAN=1 prints the exact check and builds.
        code, out = self.daily(PLAN='1')
        self.assertIn("PLAN: wait up to 20 min for the backend's EN reload of res-2 (exact: /api/health reports EN "
                      "res-2, a match)", out)
        self.assertIn('STUB deterministic_build 1', out)

    def test_exact_version_never_matching_skips_the_night(self):
        """/api/health keeps serving res-1 with the lock FREE (the inference would pass in 1 s): no fallback, the skip
        line names what the backend reports, no build, exit 0."""
        self.health = lambda n: self.serving('res-1')
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')
        self.assertEqual(code, 0, out)
        self.assertIn('backend has not loaded res-2 yet (serves an older version: /api/health reports EN res-1); '
                      'skipping tonight, the next run retries', out)
        self.assertNotIn('STUB', out)
        self.assertNotIn('inference', out)
        self.assertGreaterEqual(self.health_gets, 3)
        self.assertEqual(json.loads(read(self.state))['nights'], 1)

    def test_null_version_or_no_field_falls_back_to_the_inference(self):
        """A backend that has not reloaded since its boot (version null) and one before the field both take the
        guard's inputs, and the log says which: the lock frees after 3 s and the run proceeds."""
        for body, mode in ((self.serving(None), 'reports no EN version'),
                           (self.health(1), 'has no gameData.loaded.en.version')):
            with self.subTest(mode=mode):
                self.hold()
                self.health = lambda n, body=body: body
                threading.Timer(3, os.remove, [self.lock]).start()
                code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='1')
                self.assertEqual(code, 0, out)
                self.assertIn(f'reload wait mode inference (/api/health {mode}', out)
                self.assertIn(f'the model lock names live pid {self.holder.pid} (stages)', out)
                self.assertIn('taking res-2 as loaded (inference)', out)
                self.assertIn('STUB publish_stage', out)

    def test_the_wait_ends_when_the_lock_is_freed_and_the_run_proceeds(self):
        """The lock names a live pid for about 3 polls, then the holder frees it: the wait ends and the build runs."""
        self.hold()
        self.t.write(self.state, json.dumps({'res': 'res-2', 'nights': 1, 'lastNight': '2026-01-01'}))
        threading.Timer(3, os.remove, [self.lock]).start()
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='1')
        self.assertEqual(code, 0, out)
        self.assertIn('waiting for the backend to load res-2', out)
        self.assertIn(f'the model lock names live pid {self.holder.pid} (stages)', out)
        self.assertIn('taking res-2 as loaded', out)
        self.assertIn('STUB deterministic_build 1', out)
        self.assertIn('STUB publish_stage', out)
        self.assertLess(out.index('taking res-2 as loaded'), out.index('STUB prepare_stage'))
        self.assertFalse(os.path.exists(self.state), 'a wait that ends clears the consecutive-night count')
        self.assertEqual(self.posts, [])
        # A lock naming a dead pid is free, as the backend reads it (a job that died without its cleanup).
        dead = subprocess.Popen(['true'])
        dead.wait()
        self.t.write(self.lock, f'{dead.pid} stages\n')
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='1')
        self.assertEqual(code, 0, out)
        self.assertNotIn('live pid', out)
        self.assertIn('STUB publish_stage', out)

    def test_never_loaded_skips_and_alerts_on_the_second_night_only(self):
        """The lock stays held: exit 0, no build, the skip line; the embed only on the second consecutive night."""
        self.hold()
        skip = (f'backend has not loaded res-2 yet (serves an older version: the model lock names live pid '
                f'{self.holder.pid} (stages)); skipping tonight, the next run retries')
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')
        self.assertEqual(code, 0, out)
        self.assertIn(skip, out)
        self.assertNotIn('STUB', out)
        self.assertIn('harness: cmd_daily returned', out)
        self.assertNotIn('update_failed', out)
        self.assertEqual(json.loads(read(self.state))['nights'], 1)
        self.assertEqual(self.posts, [], 'the first night posts nothing')
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')   # the same night again: still one night
        self.assertIn(skip, out)
        self.assertEqual((json.loads(read(self.state))['nights'], self.posts), (1, []))
        # Yesterday skipped too: tonight is the second consecutive night, one embed.
        yesterday = time.strftime('%Y-%m-%d', time.gmtime(time.time() - DAY))
        self.t.write(self.state, json.dumps({'res': 'res-2', 'nights': 1, 'lastNight': yesterday}))
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')
        self.assertEqual(code, 0, out)
        self.assertNotIn('STUB', out)
        self.assertEqual(len(self.posts), 1, out)
        embed = self.posts[0]['embeds'][0]
        self.assertEqual(embed['title'], 'Trevor: the backend has not loaded the new game data')
        self.assertIn('res_version res-2', embed['description'])
        self.assertIn(f'live pid {self.holder.pid}', embed['description'])
        self.assertTrue(embed['footer']['text'].startswith('reload_wait'))
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')   # rerun the same night: no second embed
        self.assertEqual((json.loads(read(self.state))['nights'], len(self.posts)), (2, 1))
        # A gap night restarts the count: no embed.
        old = time.strftime('%Y-%m-%d', time.gmtime(time.time() - 3 * DAY))
        self.t.write(self.state, json.dumps({'res': 'res-2', 'nights': 2, 'lastNight': old}))
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0.05')
        self.assertEqual((json.loads(read(self.state))['nights'], len(self.posts)), (1, 1))

    def test_plan_prints_the_check_and_continues(self):
        self.hold()
        code, out = self.daily(PLAN='1')
        self.assertEqual(code, 0, out)
        self.assertIn(f'PLAN: wait up to 20 min for the backend\'s EN reload of res-2 (the reload guard refuses now: '
                      f'the model lock names live pid {self.holder.pid} (stages)', out)
        self.assertIn('STUB deterministic_build 1', out)
        self.assertNotIn('still waiting', out)
        self.assertFalse(os.path.exists(self.state))

    def test_wait_disabled_is_the_script_before(self):
        """BACKEND_RELOAD_WAIT_MIN=0 with the lock held: no wait, and the same lines as the script before the wait
        (HEAD's copy when it predates the wait), timestamps aside."""
        self.hold()
        code, out = self.daily(BACKEND_RELOAD_WAIT_MIN='0')
        self.assertEqual(code, 0, out)
        self.assertNotIn('wait', out)
        self.assertIn('STUB publish_stage', out)
        before = self.t.p('trevor-vps-update.before.sh')
        p = subprocess.run(['git', 'show', 'HEAD:trevor/scripts/trevor-vps-update.sh'], capture_output=True,
                           text=True, cwd=SCRIPTS)
        if p.returncode != 0 or 'BACKEND_RELOAD_WAIT_MIN' in p.stdout:
            self.skipTest('no HEAD copy that predates the wait')
        self.t.write(before, p.stdout)
        strip = lambda o: [l.split(' ', 2)[-1] if l.startswith('== ') else l for l in o.splitlines()]
        code0, out0 = self.daily(script=before)
        self.assertEqual((code0, strip(out0)), (0, strip(out)))
        # A SKIP_FETCH run reads nothing from the backend and does not wait either.
        code, out = self.daily(SKIP_FETCH='1', BACKEND_RELOAD_WAIT_MIN='0.05')
        self.assertEqual((code, 'wait' in out, 'STUB publish_stage' in out), (0, False, True), out)


if __name__ == '__main__':
    unittest.main()
