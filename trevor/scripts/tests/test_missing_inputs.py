#!/usr/bin/env python3
"""The missing-input sweep (design/trevor-self-contained-deploy.md section 9, step 7; 2026-10-19).

Every reader of a game table or a derived file that the sweep changed is pointed at a scratch tree with that input
removed or empty, and the test asserts two things: the previous output file is byte-identical afterwards, and the
process exits with the documented code (0 when a missing game table keeps the last output and logs it, 1 when a
missing derived input raises common.MissingInput, 3 when incr.py's shrink guard refuses or when the seed, the bank or
topics' units come out empty with every input present: a refused write of a computed output is suspicious). The kill switches are checked too: each one
reproduces the old behaviour (INCR_ALLOW_SHRINK=1 drops the rows, KEEP_LAST=0 writes the empty output,
TABLES_REQUIRED=1 fails on a missing table).

Each scratch tree is a temporary directory with `scripts` symlinked to this checkout's scripts (common.ROOT is taken
from the unresolved path, so the stages read and write the scratch tree), an `artifacts` tree of small fixtures and a
game-data `excel` directory named by TREVOR_GAMEDATA. Nothing under the real artifacts/ is read or written.

  python3 -m unittest scripts/tests/test_missing_inputs.py      (from trevor/; stdlib only, about 10 s)
"""
import contextlib, io, json, os, shutil, subprocess, sys, tempfile, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, SCRIPTS)
import common, incr  # noqa: E402

OLD = '{"kept": "the previous good output"}\n'


class Tree:
    """A scratch trevor tree: scripts symlinked, artifacts and game data as fixtures."""

    def __init__(self):
        self.root = tempfile.mkdtemp(prefix='trevor-missing-')
        os.symlink(SCRIPTS, os.path.join(self.root, 'scripts'))
        self.excel = os.path.join(self.root, 'gamedata', 'excel')
        os.makedirs(self.excel)

    def path(self, rel):
        return os.path.join(self.root, rel)

    def write(self, rel, text):
        p = self.path(rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, 'w') as f:
            f.write(text)
        return p

    def jsonl(self, rel, rows):
        return self.write(rel, ''.join(json.dumps(r) + '\n' for r in rows))

    def json(self, rel, obj):
        return self.write(rel, json.dumps(obj))

    def table(self, name, obj):
        return self.json(os.path.join('gamedata', 'excel', name), obj)

    def bytes(self, rel):
        with open(self.path(rel), 'rb') as f:
            return f.read()

    def run(self, *args, env=None, code=None):
        """Run `python3 scripts/<args>` (or `-c` code) in the tree; (exit code, stderr)."""
        e = {k: v for k, v in os.environ.items() if k not in ('KEEP_LAST', 'INCR_ALLOW_SHRINK', 'INCR_MAX_DROP',
                                                               'TABLES_REQUIRED', 'CORPUS', 'CHUNKS', 'PLAN',
                                                               'DERIVED_REQUIRED', 'TREVOR_BANK_DIR', 'REANSWER')}
        e.update({'TREVOR_GAMEDATA': os.path.dirname(self.excel), 'PYTHONDONTWRITEBYTECODE': '1'}, **(env or {}))
        cmd = [sys.executable, '-c', code] if code else [sys.executable, os.path.join('scripts', args[0]), *args[1:]]
        p = subprocess.run(cmd, cwd=self.root, env=e, capture_output=True, text=True, timeout=120)
        return p.returncode, p.stderr

    def close(self):
        shutil.rmtree(self.root)


class TreeCase(unittest.TestCase):
    def setUp(self):
        self.t = Tree()

    def tearDown(self):
        self.t.close()

    def assertKept(self, rel, before, rc, want_rc, err=''):
        self.assertEqual(rc, want_rc, err[-2000:])
        self.assertEqual(self.t.bytes(rel), before, f'{rel} changed')


# ---------------------------------------------------------------- common.py and incr.py, in process

class CommonHelpers(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp(prefix='trevor-common-')
        self.env = dict(os.environ)
        for k in ('KEEP_LAST', 'INCR_MAX_DROP', 'TABLES_REQUIRED', 'INCR_ALLOW_SHRINK', 'DERIVED_REQUIRED'):
            os.environ.pop(k, None)

    def tearDown(self):
        shutil.rmtree(self.d)
        os.environ.clear(); os.environ.update(self.env)

    def file(self, name, rows):
        p = os.path.join(self.d, name)
        common.write_jsonl(p, rows)
        return p

    def quiet(self, fn, *a, **kw):
        with contextlib.redirect_stderr(io.StringIO()) as err:
            return fn(*a, **kw), err.getvalue()

    def test_read_jsonl_default_and_required(self):
        p = os.path.join(self.d, 'absent.jsonl')
        self.assertEqual(common.read_jsonl(p), [])
        with self.assertRaises(common.MissingInput):
            common.read_jsonl(p, required=True)
        with self.assertRaises(common.MissingInput):
            common.read_derived(p)
        with self.assertRaises(FileNotFoundError):  # the subclass a caller of open() already caught
            common.load_derived(p)

    def test_write_keep_last_refuses_empty(self):
        p = self.file('out.jsonl', [{'a': i} for i in range(10)])
        before = open(p, 'rb').read()
        ok, err = self.quiet(common.write_keep_last, p, [])
        self.assertFalse(ok)
        self.assertIn('keeping last', err)
        self.assertEqual(open(p, 'rb').read(), before)

    def test_write_keep_last_refuses_shrink_over_threshold(self):
        p = self.file('out.jsonl', [{'a': i} for i in range(10)])
        before = open(p, 'rb').read()
        ok, _ = self.quiet(common.write_keep_last, p, [{'a': i} for i in range(7)])  # 30% dropped
        self.assertFalse(ok)
        self.assertEqual(open(p, 'rb').read(), before)
        os.environ['INCR_MAX_DROP'] = '50'
        ok, _ = self.quiet(common.write_keep_last, p, [{'a': i} for i in range(7)])
        self.assertTrue(ok)
        self.assertEqual(len(common.read_jsonl(p)), 7)

    def test_write_keep_last_writes_same_bytes_as_plain_write(self):
        rows = [{'a': i, 'n': 'Ŝkadi'} for i in range(10)]
        p, q = os.path.join(self.d, 'a.jsonl'), os.path.join(self.d, 'b.jsonl')
        common.write_jsonl(p, rows)
        self.assertTrue(common.write_keep_last(q, rows[:9]))  # 10% drop on a new file: written
        self.assertTrue(common.write_keep_last(q, rows))
        self.assertEqual(open(p, 'rb').read(), open(q, 'rb').read())
        obj = {'items': rows, 'note': 'x'}
        r, s = os.path.join(self.d, 'a.json'), os.path.join(self.d, 'b.json')
        json.dump(obj, open(r, 'w'), ensure_ascii=False, indent=1)
        self.assertTrue(common.write_keep_last(s, obj, 'json', count=lambda d: len(d['items']), ensure_ascii=False, indent=1))
        self.assertEqual(open(r, 'rb').read(), open(s, 'rb').read())
        json.dump(obj, open(r, 'w'), indent=0)  # json.dump's own ensure_ascii when none is given
        self.assertTrue(common.write_keep_last(s, obj, 'json', count=lambda d: len(d['items']), indent=0))
        self.assertEqual(open(r, 'rb').read(), open(s, 'rb').read())

    def test_write_keep_last_json_count(self):
        p = os.path.join(self.d, 'g.json')
        common.write_json_atomic(p, {'items': [1, 2, 3, 4, 5], 'note': 'x'})
        before = open(p, 'rb').read()
        ok, _ = self.quiet(common.write_keep_last, p, {'items': [], 'note': 'x'}, 'json', count=lambda d: len(d['items']))
        self.assertFalse(ok)
        self.assertEqual(open(p, 'rb').read(), before)

    def test_write_keep_last_shrink_none_refuses_only_empty(self):
        p = self.file('out.jsonl', [{'a': i} for i in range(10)])
        before = open(p, 'rb').read()
        ok, err = self.quiet(common.write_keep_last, p, [], shrink=None)
        self.assertFalse(ok)
        self.assertIn('output empty', err)
        self.assertEqual(open(p, 'rb').read(), before)
        self.assertTrue(common.write_keep_last(p, [{'a': 0}], shrink=None))  # 90% dropped: written
        self.assertEqual(common.read_jsonl(p), [{'a': 0}])
        self.assertTrue(common.write_keep_last(p, [{'a': 0}, {'a': 1}], shrink=None))
        os.environ['KEEP_LAST'] = '0'
        self.assertTrue(common.write_keep_last(p, [], shrink=None))
        self.assertEqual(open(p, 'rb').read(), b'')

    def test_derived_required_0_reads_missing_as_empty(self):
        p = os.path.join(self.d, 'absent.jsonl')
        os.environ['DERIVED_REQUIRED'] = '0'
        self.assertEqual(common.read_derived(p), [])
        with self.assertRaises(common.MissingInput):  # load_derived's call sites always raised
            common.load_derived(p)

    def test_keep_last_0_writes_the_empty_output(self):
        p = self.file('out.jsonl', [{'a': i} for i in range(10)])
        os.environ['KEEP_LAST'] = '0'
        self.assertTrue(common.write_keep_last(p, []))
        self.assertEqual(open(p, 'rb').read(), b'')

    def test_excel_table_missing_and_stale(self):
        excel = os.path.join(self.d, 'gamedata', 'excel')
        os.makedirs(excel)
        with self.assertRaises(common.MissingTable):
            common.excel_table('zone_table.json', excel)
        json.dump({'Zones': []}, open(os.path.join(excel, 'zone_table.json'), 'w'))
        json.dump({'tables': [{'table': 'zone_table', 'since': '2026-10-14T05:00:00Z', 'previousOutput': True}]},
                  open(os.path.join(self.d, 'gamedata', '.skipped.json'), 'w'))
        common._STALE_SAID.discard('zone_table')
        t, err = self.quiet(common.excel_table, 'zone_table.json', excel)
        self.assertEqual(t, {'Zones': []})
        self.assertIn('stale zone_table kept since 2026-10-14T05:00:00Z', err)
        _, err = self.quiet(common.excel_table, 'zone_table.json', excel)
        self.assertEqual(err, '')  # once per run

    def test_keep_last_on_missing_table_and_tables_required(self):
        def boom():
            raise common.MissingTable('game table x.json is missing')
        r, err = self.quiet(common.keep_last_on_missing_table, boom, os.path.join(self.d, 'out.jsonl'))
        self.assertIsNone(r)
        self.assertIn('keeping last', err)
        os.environ['TABLES_REQUIRED'] = '1'
        with self.assertRaises(common.MissingTable):
            common.keep_last_on_missing_table(boom, os.path.join(self.d, 'out.jsonl'))


class IncrShrinkGuard(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp(prefix='trevor-incr-')
        self.p = os.path.join(self.d, 'stage.jsonl')
        common.write_jsonl(self.p, [{'id': f'u{i}', 'inputSha': incr.input_sha('x'), 'promptSha': 'p'} for i in range(10)])
        self.before = open(self.p, 'rb').read()
        self.env = dict(os.environ)
        for k in ('INCR_ALLOW_SHRINK', 'INCR_MAX_DROP', 'PLAN'):
            os.environ.pop(k, None)

    def tearDown(self):
        shutil.rmtree(self.d)
        os.environ.clear(); os.environ.update(self.env)

    def stage(self, n_live):
        st = incr.Stage('test', self.p, lambda r: r['id'], 'p')
        for i in range(n_live):
            st.unit(f'u{i}', 'x')
        return st

    def plan(self, st):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()) as err:
            try:
                return st.plan(sorted(st.units)), None
            except SystemExit as e:
                return e.code, err.getvalue()

    def test_empty_input_refused(self):
        code, err = self.plan(self.stage(0))
        self.assertEqual(code, 3)
        self.assertIn('10 of 10 rows would drop', err)
        self.assertEqual(open(self.p, 'rb').read(), self.before)

    def test_shrink_over_threshold_refused(self):
        code, _ = self.plan(self.stage(7))
        self.assertEqual(code, 3)
        self.assertEqual(open(self.p, 'rb').read(), self.before)

    def test_small_drop_allowed(self):
        todo, _ = self.plan(self.stage(9))
        self.assertEqual(todo, [])
        self.assertEqual(len(common.read_jsonl(self.p)), 9)

    def test_allow_shrink_restores_old_behaviour(self):
        os.environ['INCR_ALLOW_SHRINK'] = '1'
        todo, _ = self.plan(self.stage(0))
        self.assertEqual(todo, [])
        self.assertEqual(open(self.p, 'rb').read(), b'')


# ---------------------------------------------------------------- the stage scripts, as subprocesses

class LabelRenames(TreeCase):
    OUT = 'artifacts/entities/label_renames.json'

    def setUp(self):
        super().setUp()
        self.before = self.t.write(self.OUT, '{\n "s1": {\n  "Gap-Toothed Girl": "Idida"\n }\n}') and self.t.bytes(self.OUT)

    def test_missing_corpus_raises(self):
        rc, err = self.t.run('label_renames.py')
        self.assertKept(self.OUT, self.before, rc, 1, err)
        self.assertIn('MissingInput', err)

    def test_empty_corpus_keeps_last(self):
        self.t.write('artifacts/p3b/chunks.jsonl', '')
        rc, err = self.t.run('label_renames.py')
        self.assertKept(self.OUT, self.before, rc, 0, err)
        self.assertIn('keeping last', err)

    def test_keep_last_0_writes_empty(self):
        self.t.write('artifacts/p3b/chunks.jsonl', '')
        rc, err = self.t.run('label_renames.py', env={'KEEP_LAST': '0'})
        self.assertEqual(rc, 0, err)
        self.assertEqual(self.t.bytes(self.OUT), b'{}')


class EntitiesBuild(TreeCase):
    def setUp(self):
        super().setUp()
        self.t.jsonl('artifacts/p3a/chunks.jsonl', [{'chunkId': 'a#0', 'storyId': 'a', 'groupId': 'g', 'ordinal': 0,
                                                     'text': 'Amiya: hello', 'speakers': ['Amiya']}])
        self.ids = self.t.write('artifacts/entities/identities.json', '[{"label": "Amiya"}]') and self.t.bytes('artifacts/entities/identities.json')
        self.v2 = self.t.write('artifacts/entities/identities.v2.json', '{"identities": [{"label": "Amiya"}]}') and self.t.bytes('artifacts/entities/identities.v2.json')

    def test_missing_pairs_raises(self):
        rc, err = self.t.run('entities.py', 'build')
        self.assertKept('artifacts/entities/identities.json', self.ids, rc, 1, err)
        self.assertIn('pairs.jsonl', err)

    def test_missing_pairs_v2_raises(self):
        self.t.write('artifacts/entities/pairs.jsonl', '')
        rc, err = self.t.run('entities.py', 'build', env={'ENTITY_V2': '1'})
        self.assertKept('artifacts/entities/identities.v2.json', self.v2, rc, 1, err)

    def test_empty_pairs_keeps_last(self):
        self.t.write('artifacts/entities/pairs.jsonl', '')
        rc, err = self.t.run('entities.py', 'build')
        self.assertKept('artifacts/entities/identities.json', self.ids, rc, 0, err)
        self.t.write('artifacts/entities/pairs.v2.jsonl', '')
        rc, err = self.t.run('entities.py', 'build', env={'ENTITY_V2': '1'})
        self.assertKept('artifacts/entities/identities.v2.json', self.v2, rc, 0, err)

    def test_keep_last_0_writes_empty(self):
        self.t.write('artifacts/entities/pairs.jsonl', '')
        rc, err = self.t.run('entities.py', 'build', env={'KEEP_LAST': '0'})
        self.assertEqual(rc, 0, err)
        self.assertEqual(self.t.bytes('artifacts/entities/identities.json'), b'[]')


class DeathsBuild(TreeCase):
    OUT = 'artifacts/entities/deaths.jsonl'

    def setUp(self):
        super().setUp()
        self.t.write('artifacts/chunks.jsonl', '')
        self.t.json('artifacts/chrono/timeline_v1.json', {'groups': []})
        self.t.json('artifacts/chrono/main_release.json', [])
        self.t.json('artifacts/entities/identities.json', [])
        self.t.table('character_table.json', {'Characters': []})
        self.before = self.t.write(self.OUT, OLD) and self.t.bytes(self.OUT)

    def test_missing_mentions_raises(self):
        self.t.write('artifacts/entities/death_checks.jsonl', '')
        rc, err = self.t.run('deaths.py', 'build')
        self.assertKept(self.OUT, self.before, rc, 1, err)
        self.assertIn('death_mentions.jsonl', err)

    def test_missing_checks_raises(self):
        self.t.write('artifacts/entities/death_mentions.jsonl', '')
        rc, err = self.t.run('deaths.py', 'build')
        self.assertKept(self.OUT, self.before, rc, 1, err)

    def test_empty_mentions_keep_last(self):
        self.t.write('artifacts/entities/death_checks.jsonl', '')
        self.t.write('artifacts/entities/death_mentions.jsonl', '')
        rc, err = self.t.run('deaths.py', 'build')
        self.assertKept(self.OUT, self.before, rc, 0, err)


class Chrono(TreeCase):
    def setUp(self):
        super().setUp()
        self.t.write('artifacts/chunks.jsonl', '')
        self.t.write('artifacts/p1a/chunks.jsonl', '')
        self.t.json('artifacts/chrono/timeline_v1.json', {'groups': [{'groupId': 'g', 'storylineYear': 1097}]})
        self.files = {}
        for f in ('derived.jsonl', 'flashbacks.jsonl'):
            self.t.write(f'artifacts/chrono/{f}', OLD)
        self.snap = {f: self.t.bytes(f'artifacts/chrono/{f}') for f in ('derived.jsonl', 'flashbacks.jsonl', 'timeline_v1.json')}

    def kept(self, rc, err, want=1):
        self.assertEqual(rc, want, err[-2000:])
        for f, b in self.snap.items():
            self.assertEqual(self.t.bytes(f'artifacts/chrono/{f}'), b, f)

    def test_derive_missing_events(self):
        self.t.write('artifacts/chrono/facts.jsonl', '')
        self.kept(*self.t.run('chrono.py', 'derive'))

    def test_derive_missing_facts(self):
        self.t.write('artifacts/chrono/events.jsonl', '')
        self.kept(*self.t.run('chrono.py', 'derive'))

    def test_flashbacks_missing_facts(self):
        self.t.write('artifacts/chrono/events.jsonl', '')
        self.kept(*self.t.run('chrono.py', 'flashbacks'))

    def test_build_missing_events(self):
        self.kept(*self.t.run('chrono.py', 'build'))

    def test_derive_empty_inputs_keep_last(self):
        self.t.write('artifacts/chrono/events.jsonl', '')
        self.t.write('artifacts/chrono/facts.jsonl', '')
        rc, err = self.t.run('chrono.py', 'derive')
        self.kept(rc, err, 0)
        self.assertIn('keeping last', err)


class AnswerBank(TreeCase):
    def setUp(self):
        super().setUp()
        self.t.jsonl('artifacts/p3b/spoiler.jsonl', [{'storyId': 's1'}])
        self.t.json('artifacts/chrono/timeline_v1.json', {'groups': []})
        self.t.jsonl('artifacts/p2/stories.jsonl', [])
        self.t.json('artifacts/entities/identities.json', [])
        self.seed = self.t.write('artifacts/bank/seed.jsonl', OLD) and self.t.bytes('artifacts/bank/seed.jsonl')
        self.bank = self.t.write('artifacts/bank/bank.jsonl', OLD) and self.t.bytes('artifacts/bank/bank.jsonl')

    def test_seed_missing_dossiers(self):
        rc, err = self.t.run('answer_bank.py', 'seed')
        self.assertKept('artifacts/bank/seed.jsonl', self.seed, rc, 1, err)
        self.assertIn('dossiers.jsonl', err)

    def test_seed_missing_spoiler(self):
        os.remove(self.t.path('artifacts/p3b/spoiler.jsonl'))
        rc, err = self.t.run('answer_bank.py', 'seed')
        self.assertKept('artifacts/bank/seed.jsonl', self.seed, rc, 1, err)

    def test_seed_missing_primer_support(self):
        self.t.write('artifacts/dossiers/dossiers.jsonl', '')
        self.t.write('artifacts/p2/groups.jsonl', '')
        rc, err = self.t.run('answer_bank.py', 'seed', env={'PRIMERS': '1'})
        self.assertKept('artifacts/bank/seed.jsonl', self.seed, rc, 1, err)

    def test_seed_empty_inputs_keep_last(self):
        # Every input present but the seed came out empty: a refused write of a computed output exits 3 (2026-10-19).
        self.t.write('artifacts/dossiers/dossiers.jsonl', '')
        self.t.write('artifacts/p2/groups.jsonl', '')
        rc, err = self.t.run('answer_bank.py', 'seed')
        self.assertKept('artifacts/bank/seed.jsonl', self.seed, rc, 3, err)
        self.assertIn('keeping last', err)

    def test_seed_large_drop_is_written(self):
        # No percentage rule on the seed: 10 entries down to 1 is written, exit 0, so the file cannot freeze.
        self.t.jsonl('artifacts/bank/seed.jsonl', [{'id': f'old{i}'} for i in range(10)])
        self.t.write('artifacts/dossiers/dossiers.jsonl', '')
        self.t.jsonl('artifacts/p2/groups.jsonl', [{'groupId': 'g1', 'method': 'C', 'name': 'Event', 'summary': 'S.',
                                                     'storyIds': ['s1']}])
        rc, err = self.t.run('answer_bank.py', 'seed')
        self.assertEqual(rc, 0, err[-2000:])
        self.assertEqual(len(common.read_jsonl(self.t.path('artifacts/bank/seed.jsonl'))), 1)

    def test_seed_keep_last_0_writes_empty(self):
        self.t.write('artifacts/dossiers/dossiers.jsonl', '')
        self.t.write('artifacts/p2/groups.jsonl', '')
        rc, err = self.t.run('answer_bank.py', 'seed', env={'KEEP_LAST': '0'})
        self.assertEqual(rc, 0, err[-2000:])
        self.assertEqual(self.t.bytes('artifacts/bank/seed.jsonl'), b'')

    def merge_inputs(self):
        self.t.jsonl('artifacts/bank/questions.jsonl', [{'qid': 'b00001', 'question': 'q', 'storyId': 's1'}])
        self.t.jsonl('artifacts/bank/evidence.jsonl', [{'qid': 'b00001', 'shas': {'s1#0': 'x'}, 'made': 'regen'}])
        self.t.jsonl('artifacts/bank/answers.jsonl', [{'qid': 'b00001', 'answer': 'A.', 'cited': ['s1#0'], 'passages': []}])

    def test_merge_missing_p4x_chunks(self):
        self.merge_inputs()
        rc, err = self.t.run('answer_bank.py', 'merge')
        self.assertKept('artifacts/bank/bank.jsonl', self.bank, rc, 1, err)
        self.assertIn('p4x', err)

    def test_merge_missing_gate_order(self):
        self.merge_inputs()
        self.t.write('artifacts/p4x/chunks.jsonl', '')
        self.t.write('artifacts/p3b/chunks.jsonl', '')
        rc, err = self.t.run('answer_bank.py', 'merge')
        self.assertKept('artifacts/bank/bank.jsonl', self.bank, rc, 1, err)
        self.assertIn('gate_order.json', err)

    def test_merge_missing_answers(self):
        self.merge_inputs()
        os.remove(self.t.path('artifacts/bank/answers.jsonl'))
        rc, err = self.t.run('answer_bank.py', 'merge')
        self.assertKept('artifacts/bank/bank.jsonl', self.bank, rc, 1, err)

    def merge_tree(self, seed, answers):
        """Every merge input present; `answers` answer the question b00000..; their evidence matches the corpus."""
        self.t.jsonl('artifacts/bank/seed.jsonl', seed)
        self.t.jsonl('artifacts/p3b/chunks.jsonl', [{'chunkId': 's1#0', 'storyId': 's1', 'text': 'x'}])
        self.t.write('artifacts/p4x/chunks.jsonl', '')
        self.t.json('artifacts/chrono/gate_order.json', {'storyOrder': ['s1']})
        self.t.jsonl('artifacts/bank/questions.jsonl', [{'qid': f'b{i:05}', 'question': 'q', 'storyId': 's1'}
                                                        for i in range(answers)])
        self.t.write('artifacts/bank/evidence.jsonl', '')
        self.t.jsonl('artifacts/bank/answers.jsonl', [{'qid': f'b{i:05}', 'answer': 'A.', 'cited': ['s1#0'],
                                                       'passages': []} for i in range(answers)])

    def test_merge_large_drop_is_written(self):
        # A re-chunk dropped most answers: 10 bank entries down to 2 is written (exit 0), not kept and frozen.
        self.t.jsonl('artifacts/bank/bank.jsonl', [{'id': f'old{i}'} for i in range(10)])
        self.merge_tree([], 2)
        rc, err = self.t.run('answer_bank.py', 'merge', env={'REANSWER': '0'})
        self.assertEqual(rc, 0, err[-2000:])
        self.assertEqual(len(common.read_jsonl(self.t.path('artifacts/bank/bank.jsonl'))), 2)
        # and the next merge against it is not a "shrink" either
        self.merge_tree([], 2)
        rc, err = self.t.run('answer_bank.py', 'merge', env={'REANSWER': '0'})
        self.assertEqual(rc, 0, err[-2000:])

    def test_merge_empty_bank_with_inputs_exits_3(self):
        self.merge_tree([], 0)
        rc, err = self.t.run('answer_bank.py', 'merge', env={'REANSWER': '0'})
        self.assertKept('artifacts/bank/bank.jsonl', self.bank, rc, 3, err)
        self.assertIn('keeping last', err)
        rc, err = self.t.run('answer_bank.py', 'merge', env={'REANSWER': '0', 'KEEP_LAST': '0'})
        self.assertEqual(rc, 0, err[-2000:])
        self.assertEqual(self.t.bytes('artifacts/bank/bank.jsonl'), b'')


class Topics(TreeCase):
    def test_units_missing_topics(self):
        before = self.t.write('artifacts/topics/units.jsonl', OLD) and self.t.bytes('artifacts/topics/units.jsonl')
        rc, err = self.t.run('topics.py', 'units')
        self.assertKept('artifacts/topics/units.jsonl', before, rc, 1, err)

    def test_units_empty_topics_keep_last(self):
        # topics.jsonl present but nothing served: the refused write exits 3 so the update stops (2026-10-19).
        before = self.t.write('artifacts/topics/units.jsonl', OLD) and self.t.bytes('artifacts/topics/units.jsonl')
        self.t.write('artifacts/topics/topics.jsonl', '')
        rc, err = self.t.run('topics.py', 'units')
        self.assertKept('artifacts/topics/units.jsonl', before, rc, 3, err)

    def test_units_large_drop_is_written(self):
        self.t.jsonl('artifacts/topics/units.jsonl', [{'storyId': f'old{i}'} for i in range(10)])
        self.t.jsonl('artifacts/topics/units.v1.jsonl', [{'storyId': f'old{i}'} for i in range(10)])
        self.t.jsonl('artifacts/topics/topics.jsonl', [{'topic': 'Rhodes Island', 'summary': 'A company [1].'}])
        rc, err = self.t.run('topics.py', 'units')
        self.assertEqual(rc, 0, err[-2000:])
        for name in ('units.jsonl', 'units.v1.jsonl'):
            self.assertEqual(len(common.read_jsonl(self.t.path(f'artifacts/topics/{name}'))), 1, name)

    def mine_tree(self):
        self.t.jsonl('artifacts/p3b/chunks.jsonl', [{'chunkId': 'a#0', 'storyId': 'a', 'groupId': 'g', 'text': 'x: y'}])
        self.t.write('artifacts/entities/operator_attributes.jsonl', '')
        self.t.json('artifacts/entities/identities.json', [])
        return self.t.write('artifacts/topics/candidates.jsonl', OLD) and self.t.bytes('artifacts/topics/candidates.jsonl')

    def test_mine_missing_table_keeps_last(self):
        before = self.mine_tree()
        rc, err = self.t.run('topics.py', 'mine')
        self.assertKept('artifacts/topics/candidates.jsonl', before, rc, 0, err)
        self.assertIn('character_table.json is missing', err)

    def test_mine_tables_required_fails(self):
        before = self.mine_tree()
        rc, err = self.t.run('topics.py', 'mine', env={'TABLES_REQUIRED': '1'})
        self.assertKept('artifacts/topics/candidates.jsonl', before, rc, 1, err)

    def test_mine_missing_corpus_raises(self):
        before = self.t.write('artifacts/topics/candidates.jsonl', OLD) and self.t.bytes('artifacts/topics/candidates.jsonl')
        rc, err = self.t.run('topics.py', 'mine')
        self.assertKept('artifacts/topics/candidates.jsonl', before, rc, 1, err)


class TableReaders(TreeCase):
    """Non-core stages: a missing game table keeps the last output, exit 0; TABLES_REQUIRED=1 fails."""

    def check(self, script, out, *args, env=None):
        before = self.t.write(out, OLD) and self.t.bytes(out)
        rc, err = self.t.run(script, *args, env=env)
        self.assertKept(out, before, rc, 0, err)
        self.assertIn('keeping last', err)
        rc, err = self.t.run(script, *args, env={**(env or {}), 'TABLES_REQUIRED': '1'})
        self.assertKept(out, before, rc, 1, err)
        self.assertIn('MissingTable', err)

    def test_sources(self):
        self.check('sources.py', 'artifacts/sources/units.jsonl')

    def test_attributes(self):
        self.check('attributes.py', 'artifacts/entities/operator_attributes.jsonl')

    def test_gametext(self):
        self.t.write('artifacts/sources/all_units.jsonl', '')
        self.check('gametext.py', 'artifacts/gametext/units.jsonl')

    def test_gametext_require_tables_keeps_its_meaning(self):
        before = self.t.write('artifacts/gametext/units.jsonl', OLD) and self.t.bytes('artifacts/gametext/units.jsonl')
        rc, err = self.t.run('gametext.py', env={'GAMETEXT_REQUIRE_TABLES': '1'})
        self.assertKept('artifacts/gametext/units.jsonl', before, rc, 1, err)

    def test_source_dates(self):
        self.check('source_dates.py', 'artifacts/chrono/source_dates.json', '--gamedata', self.t.excel)

    def test_source_dates_missing_guide(self):
        for n, obj in (('handbook_info_table.json', {'HandbookDict': []}), ('uniequip_table.json', {'EquipDict': []}),
                       ('skin_table.json', {'CharSkins': []})):
            self.t.table(n, obj)
        before = self.t.write('artifacts/chrono/source_dates.json', OLD) and self.t.bytes('artifacts/chrono/source_dates.json')
        rc, err = self.t.run('source_dates.py', '--gamedata', self.t.excel)
        self.assertKept('artifacts/chrono/source_dates.json', before, rc, 1, err)
        self.assertIn('reading_guide.json', err)

    def test_reading_guide_dates(self):
        self.t.table('activity_table.json', {})
        self.check('reading_guide.py', 'artifacts/chrono/main_release.json', 'dates')

    def test_reading_guide_dates_require_tables(self):
        self.t.table('activity_table.json', {})
        before = self.t.write('artifacts/chrono/main_release.json', OLD) and self.t.bytes('artifacts/chrono/main_release.json')
        rc, err = self.t.run('reading_guide.py', 'dates', env={'DATES_REQUIRE_TABLES': '1'})
        self.assertKept('artifacts/chrono/main_release.json', before, rc, 1, err)

    def test_dates_without_activity_and_no_file(self):
        rc, err = self.t.run('reading_guide.py', 'dates')
        self.assertEqual(rc, 0, err)
        self.assertFalse(os.path.exists(self.t.path('artifacts/chrono/main_release.json')))
        self.assertIn('no artifacts/chrono/main_release.json to keep', err)


class ReadingGuideBuild(TreeCase):
    OUT = 'artifacts/chrono/reading_guide.json'

    def setUp(self):
        super().setUp()
        self.t.json('artifacts/chrono/timeline_v1.json', {'groups': [{'groupId': 'act1', 'name': 'A', 'releaseTime': 1600000000}]})
        self.t.json('artifacts/chrono/main_release.json', [])
        self.t.write('artifacts/primers/primers.jsonl', '')
        self.before = self.t.write(self.OUT, OLD) and self.t.bytes(self.OUT)

    def test_missing_chunks_writes_nothing(self):
        # Before 2026-10-19 the guide was written once without words before chunks.jsonl was read.
        rc, err = self.t.run('reading_guide.py', 'build')
        self.assertKept(self.OUT, self.before, rc, 1, err)

    def test_missing_primers(self):
        os.remove(self.t.path('artifacts/primers/primers.jsonl'))
        self.t.write('artifacts/chunks.jsonl', '')
        rc, err = self.t.run('reading_guide.py', 'build')
        self.assertKept(self.OUT, self.before, rc, 1, err)


class ModelStageInputs(TreeCase):
    """Derived inputs of the model stages raise instead of reading as empty."""

    def call(self, code):
        return self.t.run(code=f"import sys; sys.path.insert(0, 'scripts'); {code}")

    def test_primers_plan_missing_groups(self):
        self.t.json('artifacts/chrono/timeline_v1.json', {'groups': []})
        rc, err = self.call('import primers; primers.plan()')
        self.assertEqual(rc, 1, err)
        self.assertIn('groups.jsonl', err)

    def test_topic_deep_missing_p4x(self):
        rc, err = self.call('import topic_deep; topic_deep.Data()')
        self.assertEqual(rc, 1, err)
        self.assertIn('p4x', err)

    def test_canon_inputs_missing_table(self):
        before = self.t.write('artifacts/canon/inputs.json', OLD) and self.t.bytes('artifacts/canon/inputs.json')
        rc, err = self.t.run('canon_refs.py', 'inputs')
        self.assertKept('artifacts/canon/inputs.json', before, rc, 1, err)
        self.assertIn('roguelike_topic_table.json', err)


if __name__ == '__main__':
    unittest.main()
