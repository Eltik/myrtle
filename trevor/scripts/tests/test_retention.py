#!/usr/bin/env python3
"""Retention (scripts/retention.py; design/trevor-self-contained-deploy.md section 6 step 9; 2026-10-19).

Each test builds a scratch tree with fake logs, eval runs, spool files, releases and asset manifests whose ages are
set with os.utime (what `touch -t` does), runs retention.py on it and checks: each policy keeps what it must and lists
the rest; a dry run deletes nothing; --apply deletes exactly the listed entries; the eval index and the live symlink
target survive; the asset manifests need --apply-assets too; the exit codes are 0, 2 and 1 as documented. The last
test drives `trevor-vps-update.sh prune` and checks that the disk warning reaches a stand-in Discord webhook.

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
        # Releases: 11; the newest 5 are 6..10; current -> r0 (a rollback), previous r1 (releases.jsonl), stage base
        # r2; 3, 4 and 5 go.
        names = [f'{stamp(i)}-v{i}' for i in range(11)]
        for i, n in enumerate(names):
            self.write(self.p('home', 'releases', n, 'artifacts', 'chunks.jsonl'), f'release {i}\n')
            if i in (3, 4, 5):
                self.expected.add(self.p('home', 'releases', n))
        os.symlink(self.p('home', 'releases', names[0], 'artifacts'), self.p('home', 'current'))
        order = names[2:] + [names[1], names[0]]
        self.write(self.p('home', 'state', 'releases.jsonl'),
                   ''.join(json.dumps({'version': n, 'release': self.p('home', 'releases', n)}) + '\n' for n in order))
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


if __name__ == '__main__':
    unittest.main()
