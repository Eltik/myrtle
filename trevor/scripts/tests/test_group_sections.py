#!/usr/bin/env python3
"""Method D of the P2 group summaries and the name-coverage measure (2026-10-19, design/experiment-log.md "Group
summaries that stop after the first stories").

Checks the part split (sizes, order, the 8-story threshold), the part bodies' naming, the incremental key of D (its own
prompt sha, (groupId, 'D') like the others, A/B/C untouched), the served-summary choice (D over C,
SUMMARY_METHOD_D=0 reads C), and the coverage measure on a fixture. No model; nothing under artifacts/ is read or
written (the stage runs on a temporary output directory with an in-memory corpus).

  python3 -m unittest scripts/tests/test_group_sections.py      (from trevor/; stdlib only, under a second)
"""
import json, os, sys, tempfile, unittest
from unittest import mock

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, SCRIPTS)
import common  # noqa: E402

p2 = common.load_script('p2-summaries')
cov = common.load_script('summary_coverage')


class PartSplit(unittest.TestCase):
    def test_threshold(self):
        for n in range(0, 9):
            self.assertEqual(p2.split_parts(n, min_stories=8), [], n)
        self.assertEqual(p2.split_parts(9, min_stories=8), [(0, 3), (3, 6), (6, 9)])
        self.assertEqual(p2.split_parts(9, min_stories=9), [])

    def test_sizes_and_order(self):
        for n in range(9, 80):
            parts = p2.split_parts(n, min_stories=8)
            self.assertTrue(3 <= len(parts) <= 5, n)
            self.assertEqual(parts[0][0], 0)
            self.assertEqual(parts[-1][1], n)
            for (a, b), (c, _) in zip(parts, parts[1:]):
                self.assertEqual(b, c, n)  # consecutive, nothing skipped or repeated
            sizes = [b - a for a, b in parts]
            self.assertLessEqual(max(sizes) - min(sizes), 1, (n, sizes))
            self.assertTrue(all(s > 0 for s in sizes))

    def test_episode_15(self):
        # 25 stories: 4 parts, the second "stories 7 to 13".
        self.assertEqual(p2.split_parts(25, min_stories=8), [(0, 6), (6, 13), (13, 19), (19, 25)])
        self.assertEqual(len(p2.split_parts(40, min_stories=8)), 5)


def corpus(n_stories, gid='g1'):
    ids = [f'{gid}_s{i:02d}' for i in range(1, n_stories + 1)]
    facts = {s: {'header': f'Event One, {s}', 'synopsis': f'syn {s}', 'n': i} for i, s in enumerate(ids, 1)}
    return {gid: ids}, facts, {s: f'summary of {s}' for s in ids}


class PartBodies(unittest.TestCase):
    def test_bodies_name_their_part(self):
        groups, facts, stories = corpus(25)
        with mock.patch.object(p2, 'PARTS_MIN', 8):
            bodies = p2.part_bodies('g1', groups, facts, stories)
            self.assertEqual(len(bodies), 4)
            self.assertTrue(bodies[1].startswith('Part 2 of 4, stories 7 to 13 of 25 of Event One.'))
            self.assertIn('g1_s07', bodies[1]); self.assertIn('g1_s13', bodies[1]); self.assertNotIn('g1_s14', bodies[1])
            self.assertNotIn('g1_s06', bodies[1])
            m = p2.merge_body('g1', groups, facts, ['one', 'two', 'three', 'four'])
            self.assertIn('Part 4 of 4 (stories 20 to 25): four', m)
            short, f2, s2 = corpus(8)
            self.assertEqual(p2.part_bodies('g1', short, f2, s2), [])


class IncrementalKey(unittest.TestCase):
    """D has its own prompt sha and unit key; registering it changes nothing for A, B and C."""

    def stage(self, out, groups, facts, stories, **kw):
        by_story = {s: [] for s in facts}
        with mock.patch.object(p2, 'OUT', out), mock.patch.object(p2, 'load', lambda: (by_story, facts, groups)), \
                mock.patch.object(p2, 'PARTS_MIN', 8):
            with open(os.path.join(out, 'stories.jsonl'), 'w') as f:
                for s, t in stories.items():
                    f.write(json.dumps({'storyId': s, 'summary': t}) + '\n')
            return p2.group_stage(**kw)

    def test_d_key(self):
        groups, facts, stories = corpus(25)
        g2, f2, s2 = corpus(5, 'g2')
        groups.update(g2); facts.update(f2); stories.update(s2)
        with tempfile.TemporaryDirectory() as out:
            st, order = self.stage(out, groups, facts, stories)[:2]
            self.assertEqual([k for k in order if k[1] == 'D'], [])  # no D row, no D unit in a plain run
            for g, m in order:
                st.put({'groupId': g, 'method': m, 'summary': 'x', 'storyIds': groups[g], 'promptSha': st.psha})
            st, order = self.stage(out, groups, facts, stories, methods=['D'], select=['g1', 'g2'])[:2]
            self.assertEqual(order, [('g1', 'D')])  # g2 has 5 stories: not sectioned
            self.assertEqual(st.reason(('g1', 'D')), 'new')
            self.assertEqual(st.unit_psha(('g1', 'D')), p2.D_PSHA)
            self.assertNotEqual(p2.D_PSHA, st.psha)
            st.put({'groupId': 'g1', 'method': 'D', 'summary': 'd', 'parts': [], 'storyIds': groups['g1'],
                    'promptSha': p2.D_PSHA})
            st, order = self.stage(out, groups, facts, stories)[:2]
            self.assertIn(('g1', 'D'), order)  # a plain run keeps the D row up to date
            self.assertEqual(st.stale(), set())
            self.assertEqual(st.plan(order, show=False), [])
            rows = common.read_jsonl(os.path.join(out, 'groups.jsonl'))
            self.assertEqual(len(rows), 7)  # g1 C/B/A/D, g2 C/B/A: nothing dropped
            stories['g1_s20'] = 'a redone summary'
            st, order = self.stage(out, groups, facts, stories)[:2]
            self.assertEqual(st.stale(), {('g1', 'C'), ('g1', 'B'), ('g1', 'D')})


class Served(unittest.TestCase):
    rows = [{'groupId': 'a', 'method': 'C', 'summary': 'ca'}, {'groupId': 'a', 'method': 'B', 'summary': 'ba'},
            {'groupId': 'b', 'method': 'C', 'summary': 'cb'}, {'groupId': 'a', 'method': 'D', 'summary': 'da'}]

    def test_d_over_c(self):
        with mock.patch.dict(os.environ, {'SUMMARY_METHOD_D': '1'}):
            self.assertEqual([r['summary'] for r in common.served_group_rows(self.rows)], ['da', 'cb'])

    def test_kill_switch(self):
        with mock.patch.dict(os.environ, {'SUMMARY_METHOD_D': '0'}):
            self.assertEqual(common.served_group_rows(self.rows), [self.rows[0], self.rows[2]])


class Coverage(unittest.TestCase):
    stories = {
        's1': 'Amiya met Kal\'tsit in Londinium. Finally, the Doctor arrived.',
        's2': 'Meanwhile, Amiya argued with Kal\'tsit about the Doctor.',
        's3': 'In the tower, Kal\'tsit\'s plan failed and Theresis waited. Nearby, Amiya fled.',
        's4': 'Then Theresis spoke to Amiya, and Theresis left.',
        's5': 'At the end, Mon3tr woke beside Amiya and Theresis.',
        's6': 'Finally, Mon3tr asked for Kal\'tsit.',
    }

    def test_recurring_names(self):
        # Quarter of 6 is 2, so 3 stories are needed. Finally (3 stories) is only ever sentence-initial; Doctor is in 2.
        self.assertEqual(cov.recurring_names(list(self.stories.values())), ["Amiya", "Kal'tsit", "Theresis"])

    def test_measure(self):
        row = {'groupId': 'g', 'method': 'C', 'storyIds': list(self.stories),
               'summary': "Amiya and Kal'tsit's quarrel opened the chapter."}
        r = cov.measure([row], self.stories)['g']
        self.assertEqual(r['missing'], ['Theresis'])
        self.assertEqual(r['names'], 3)
        self.assertAlmostEqual(r['share'], 0.3333)

    def test_exit_code(self):
        with tempfile.TemporaryDirectory() as d:
            gp, sp, out = (os.path.join(d, x) for x in ('groups.jsonl', 'stories.jsonl', 'coverage.json'))
            common.write_jsonl(sp, [{'storyId': s, 'summary': t} for s, t in self.stories.items()])
            common.write_jsonl(gp, [{'groupId': 'g', 'method': 'C', 'storyIds': list(self.stories), 'summary': 'Amiya.'}])
            argv = ['--groups', gp, '--stories', sp, '--out', out]
            with mock.patch('sys.stdout'):
                self.assertEqual(cov.main(argv), 2)  # 2 of 3 missing, above 0.4
                self.assertEqual(cov.main(argv + ['--threshold', '0.7']), 0)
            self.assertEqual(common.load_json(out)['groups']['g']['missing'], ["Kal'tsit", 'Theresis'])


if __name__ == '__main__':
    unittest.main()
