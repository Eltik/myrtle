#!/usr/bin/env python3
"""scripts/reanswer.py, the nightly re-answer (design/trevor-self-contained-deploy.md step 6; 2026-10-19).

The pure parts on fixtures: the de-duplication by normQ (qkey) and its accumulation across windows, the pending list
and --min-asks, the keep rules, the gate over every passage, the bank row's marking; then answer_bank.py merge in a
scratch tree (scripts symlinked, small fixture artifacts): the re-answered rows join bank.jsonl marked
`source: reanswer`, a declined one does not, and REANSWER=0 writes the same bytes as a merge with no re-answer at all.
No database, no model, nothing under the real artifacts/.

  python3 -m unittest scripts/tests/test_reanswer.py      (from trevor/; stdlib only, about 1 s)
"""
import json, os, shutil, subprocess, sys, tempfile, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, SCRIPTS)
import answer_bank, common, reanswer  # noqa: E402

NL = reanswer.NO_LIMIT_ERA
K1, K2 = 'a' * 32, 'b' * 32


def ask(qkey, question, era, user, at, rung='cpu'):
    return {'qkey': qkey, 'normQ': question.lower().rstrip('?'), 'question': question, 'eraPos': era, 'userId': user,
            'createdAt': f'2026-10-18T{at}:00.000000Z', 'rung': rung, 'status': 'done'}


ASKS = [ask(K1, 'Who is Ines?', 120, 'u1', '01:00'), ask(K1, 'who is ines', 120, 'u2', '02:00'),
        ask(K1, 'Who is Ines?', NL, 'u3', '03:00', 'passages'), ask(K2, 'Why did W leave?', 40, 'u1', '04:00')]
SP = {'s1': {'storyId': 's1', 'requiredStages': ['st1']}, 's2': {'storyId': 's2', 'requiredStages': ['st2']},
      's9': {'storyId': 's9', 'requiredStages': ['st9']}}


class Fold(unittest.TestCase):
    def test_dedup_by_qkey(self):
        rows, n = reanswer.fold(ASKS, [])
        self.assertEqual([r['qid'] for r in rows], ['r' + K1[:16], 'r' + K2[:16]])
        self.assertEqual(dict(n), {'new': 2})
        r = rows[0]
        self.assertEqual((r['question'], r['aliases'], r['asks'], r['askers']), ('Who is Ines?', ['who is ines'], 3, 3))
        self.assertEqual((r['eras'], r['spoilers'], r['rungs']), ([120, NL], 'mixed', ['cpu', 'passages']))
        self.assertEqual((rows[1]['spoilers'], rows[1]['asks']), ('safe', 1))

    def test_rerun_does_not_double_count(self):
        rows, _ = reanswer.fold(ASKS, [])
        again, n = reanswer.fold(ASKS, rows)
        self.assertEqual(again, rows)
        self.assertEqual(dict(n), {})
        later, n = reanswer.fold(ASKS + [ask(K2, 'Why did W leave', NL, 'u4', '23:00')], rows)
        self.assertEqual(dict(n), {'updated': 1})
        r = later[1]
        self.assertEqual((r['asks'], r['question'], r['aliases'], r['spoilers']),
                         (2, 'Why did W leave?', ['Why did W leave'], 'mixed'))
        self.assertEqual(r['firstAsked'], rows[1]['firstAsked'])

    def test_pending_min_asks_and_answered(self):
        rows, _ = reanswer.fold(ASKS, [])
        self.assertEqual([r['qid'] for r in reanswer.pending(rows, set(), 1)], ['r' + K1[:16], 'r' + K2[:16]])
        self.assertEqual([r['qid'] for r in reanswer.pending(rows, set(), 2)], ['r' + K1[:16]])
        self.assertEqual(reanswer.pending(rows, {'r' + K1[:16]}, 2), [])

    def test_sql_filters_rung_and_window(self):
        q = reanswer.collect_sql('en', 24)
        self.assertIn("j.rung IN ('cpu', 'passages')", q)
        self.assertIn('make_interval(hours => 24)', q)
        self.assertIn('COALESCE(k.deferred_job_id, k.job_id)', q)
        with self.assertRaises(ValueError):
            reanswer.collect_sql("en'; drop table x; --", 24)


def ans(qid, text, cited, passages):
    return {'qid': qid, 'question': '?', 'answer': text, 'cited': cited, 'passages': passages}


class KeepAndGate(unittest.TestCase):
    Q = {'qid': 'r1', 'question': 'Who is Ines?', 'aliases': ['who is ines'], 'asks': 3, 'askers': 2}
    CUR = {'s1#0001': 'h1', 's2#0004': 'h2'}

    def v(self, a, q=Q, ev=None, cur=CUR):
        ev = ev if ev is not None else {'shas': {c: cur.get(c) for c in a['cited']}}
        return reanswer.verdict(a, q, ev, cur, answer_bank.DECLINE)

    def test_keep_rules(self):
        self.assertIsNone(self.v(ans('r1', 'Ines is a mercenary [1].', ['s1#0001'], ['s1#0001'])))
        self.assertEqual(self.v(ans('r1', 'I could not find that in the passages [1].', ['s1#0001'], [])), 'declined')
        self.assertEqual(self.v(ans('r1', 'Ines is a mercenary.', [], [])), 'uncited')
        self.assertEqual(self.v(ans('r1', 'Ines [1].', ['s1#0001'], []), q=None), 'question gone')
        a = ans('r1', 'Ines [1].', ['s1#0001'], [])
        self.assertEqual(reanswer.verdict(a, self.Q, None, self.CUR, answer_bank.DECLINE), 'no evidence row')
        self.assertEqual(self.v(a, ev={'shas': {'s1#0001': 'old'}}), 'evidence changed')
        self.assertEqual(self.v(a, ev={'shas': {'s1#0001': 'h1'}}, cur={}), 'evidence vanished')

    def test_gate_covers_every_passage_and_marks_the_row(self):
        a = ans('r1', 'Ines [1].', ['s1#0001'], ['s1#0001', 's9#0002', 'archive_char_1#0000', 'dossier:Ines'])
        row = reanswer.bank_row(a, self.Q, answer_bank.gate, SP)
        # s9 was only shown, never cited: it still gates the entry; an undatable passage is recorded as unknown.
        self.assertEqual(row['gateStories'], ['dossier:Ines', 's1', 's9'])
        self.assertEqual(row['requiredStages'], ['st1', 'st9'])
        self.assertEqual(row['gateChars'], ['char_1'])
        self.assertEqual(row['gateUnknown'], ['dossier:Ines'])
        self.assertEqual((row['id'], row['kind'], row['source'], row['asks'], row['askers'], row['aliases']),
                         ('ask:r1', 'ask', 'reanswer', 3, 2, ['who is ines']))
        self.assertEqual(row['sources'], ['s1#0001'])


class MergeInTree(unittest.TestCase):
    """answer_bank.py merge and reanswer.py merge in a scratch tree."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix='trevor-reanswer-')
        os.symlink(SCRIPTS, os.path.join(self.root, 'scripts'))
        w = self.write
        w('artifacts/p3b/spoiler.jsonl', list(SP.values()))
        w('artifacts/p3b/chunks.jsonl', [{'chunkId': 's1#0001', 'text': 'one'}, {'chunkId': 's2#0004', 'text': 'two'}])
        w('artifacts/p4x/chunks.jsonl', [{'chunkId': 's1#0001', 'text': 'one'}])
        w('artifacts/bank/seed.jsonl', [{'id': f'who:N{i}', 'kind': 'who', 'question': f'Who is N{i}?', 'aliases': [],
                                         'answer': 'x', 'sources': ['s1'], 'gateStories': ['s1'],
                                         'requiredStages': ['st1'], 'gateChars': []} for i in range(10)])
        w('artifacts/bank/questions.jsonl', [])
        w('artifacts/bank/answers.jsonl', [])

    def tearDown(self):
        shutil.rmtree(self.root)

    def write(self, rel, rows):
        p = os.path.join(self.root, rel)
        os.makedirs(os.path.dirname(p), exist_ok=True)
        with open(p, 'w') as f:
            f.writelines(json.dumps(r) + '\n' for r in rows)

    def run_py(self, *args, **env):
        e = {k: v for k, v in os.environ.items() if k not in ('REANSWER', 'TREVOR_BANK_DIR', 'KEEP_LAST')}
        r = subprocess.run([sys.executable, os.path.join(self.root, 'scripts', args[0]), *args[1:]], cwd=self.root,
                           env={**e, **env}, capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        return r.stdout

    def bank(self):
        with open(os.path.join(self.root, 'artifacts/bank/bank.jsonl'), 'rb') as f:
            return f.read()

    def test_merge_folds_reanswers_and_kill_switch_is_byte_identical(self):
        self.run_py('answer_bank.py', 'merge')
        before = self.bank()
        sh = common.sha16
        self.write('artifacts/bank/reanswer/questions.jsonl', [
            {'qid': 'rkeep', 'question': 'Who is Ines?', 'aliases': [], 'asks': 3, 'askers': 2},
            {'qid': 'rdecl', 'question': 'Who is Nobody?', 'aliases': [], 'asks': 1, 'askers': 1}])
        self.write('artifacts/bank/reanswer/answers.jsonl', [
            ans('rkeep', 'Ines is a mercenary [1].', ['s1#0001'], ['s1#0001', 's2#0004']),
            ans('rdecl', 'I could not find that in the passages [1].', ['s2#0004'], ['s2#0004'])])
        self.write('artifacts/bank/reanswer/evidence.jsonl', [
            {'qid': 'rkeep', 'shas': {'s1#0001': sh('one')}, 'made': 'reanswer'},
            {'qid': 'rdecl', 'shas': {'s2#0004': sh('two')}, 'made': 'reanswer'}])
        self.run_py('reanswer.py', 'merge', REANSWER='0')
        self.assertEqual(self.bank(), before)
        out = self.run_py('reanswer.py', 'merge')
        self.assertIn("reanswer {'declined': 1, 'kept': 1}", out)
        rows = [json.loads(l) for l in self.bank().decode().splitlines()]
        re_rows = [r for r in rows if r.get('source') == 'reanswer']
        self.assertEqual([r['id'] for r in re_rows], ['ask:rkeep'])
        self.assertEqual((re_rows[0]['gateStories'], re_rows[0]['asks']), (['s1', 's2'], 3))
        self.assertEqual(rows[:10], [json.loads(l) for l in before.decode().splitlines()])

    def test_bank_dir_override(self):
        shutil.copytree(os.path.join(self.root, 'artifacts/bank'), os.path.join(self.root, 'scratch/bank'))
        self.run_py('answer_bank.py', 'merge', TREVOR_BANK_DIR='scratch/bank')
        self.assertTrue(os.path.exists(os.path.join(self.root, 'scratch/bank/bank.jsonl')))
        self.assertFalse(os.path.exists(os.path.join(self.root, 'artifacts/bank/bank.jsonl')))


if __name__ == '__main__':
    unittest.main()
