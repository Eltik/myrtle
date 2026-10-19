#!/usr/bin/env python3
"""The chunk-scoped renames of the "???" label (scripts/label_renames.py, 2026-10-19 and 2026-10-20).

A "???" line followed within 3 lines by a label's first line of the chunk that introduces itself by that label names
the chunk's "???" lines, unless a "???" line comes after the introduction (another unknown speaker, which also rules
out two names in one chunk). No model; nothing under artifacts/ is read or written.

  python3 -m unittest scripts/tests/test_label_renames.py      (from trevor/; stdlib only, under a second)
"""
import os, sys, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, SCRIPTS)
import label_renames as lr  # noqa: E402


def chunk(cid, *lines):
    return {'chunkId': cid, 'storyId': cid.split('#')[0], 'text': '\n'.join(lines)}


WAKE = chunk('main_15_level_main_15-18_end#0007',
             'Amiya: Storing data?',
             "Kal'tsit's life. The myriad branches once again merge together.",
             "???: ...Kal'tsit? The rain caresses her palms.",
             'Mon3tr: My name is Mon3tr. You gave me this name.',
             'Amiya: ......')


class ChunkRenames(unittest.TestCase):
    def test_introduction_names_the_unknown_label(self):
        self.assertEqual(lr.chunk_renames([WAKE]), {'main_15_level_main_15-18_end#0007': {'???': 'Mon3tr'}})

    def test_only_the_unknown_label(self):
        c = chunk('s#0', 'Hooded Man: Who goes there?', "Red: I'm Red.")
        self.assertEqual(lr.chunk_renames([c]), {})

    def test_introduction_must_be_the_speakers_first_line_in_the_chunk(self):
        c = chunk('s#0', 'Mon3tr: ...', "???: Kal'tsit?", 'Mon3tr: My name is Mon3tr.')
        self.assertEqual(lr.chunk_renames([c]), {})

    def test_more_than_three_lines_later_is_no_introduction(self):
        c = chunk('s#0', '???: Hello.', 'a', 'b', 'c', 'Ben: Call me Ben.')
        self.assertEqual(lr.chunk_renames([c]), {})

    def test_an_unknown_line_after_the_introduction_drops_the_chunk(self):
        c = chunk('s#0', '???: Hello.', 'Ben: Call me Ben.', '???: And who are you?')
        self.assertEqual(lr.chunk_renames([c]), {})

    def test_a_second_introduction_drops_the_chunk(self):
        c = chunk('s#0', '???: Hello.', 'Ben: Call me Ben.', 'x', 'y', 'z', 'w', '???: Hi.', 'Ann: Call me Ann.')
        self.assertEqual(lr.chunk_renames([c]), {})

    def test_the_name_must_be_the_label(self):
        c = chunk('s#0', '???: Hello.', 'Ben: My name is Benthonexigios.')
        self.assertEqual(lr.chunk_renames([c]), {})


TONYA = chunk('main_17_level_main_17-04_beg#0006', 'Guard: Tonya Alliluyeva.', '???: ......', 'Guard: Tonya Alliluyeva!',
              '???: Oh, yes.', "Tonya: I'm here...", 'Guard: Who do you think you are putting on a show for?')
RED = chunk('main_12_level_main_12-05_end#0000', 'Starving Thug: And who are you? Here to rob him, too?',
            '???: Hello. My name is Red.', "Red: Back in my Ursus days, they all called me 'Redblade.'",
            'Misery: Pick up the pace.')


class SelfIntroRule(unittest.TestCase):
    def test_the_unknown_line_names_itself_and_the_label_follows(self):
        self.assertEqual(lr.self_intro_renames([RED]), {RED['chunkId']: {'???': 'Red'}})

    def test_the_label_must_be_the_name(self):
        c = chunk('s#0', "???: I'm Red.", 'Misery: Pick up the pace.')
        self.assertEqual(lr.self_intro_renames([c]), {})

    def test_an_unknown_line_after_the_label_drops_the_chunk(self):
        c = chunk('s#0', "???: I'm Red.", 'Red: Hello.', '???: And I am someone else.')
        self.assertEqual(lr.self_intro_renames([c]), {})


class AnsweredCallRule(unittest.TestCase):
    def test_a_called_name_answered_by_the_unknown_label(self):
        self.assertEqual(lr.answered_call_renames([TONYA]), {TONYA['chunkId']: {'???': 'Tonya'}})

    def test_the_label_must_be_new_to_the_story(self):
        earlier = chunk('main_17_level_main_17-04_beg#0005', 'Tonya: Morning.')
        earlier['ordinal'], TONYA['ordinal'] = 5, 6
        self.assertEqual(lr.answered_call_renames([earlier, TONYA]), {})

    def test_no_call_no_name(self):
        c = chunk('s#0', 'Guard: Stand up!', '???: Oh, yes.', "Tonya: I'm here...")
        self.assertEqual(lr.answered_call_renames([c]), {})


LAVINIA = chunk('act38side_level_act38side_09_beg#0007', 'Texas: I am with you.', 'Lavinia: Good.',
                '???: Stay here, Lavinia.', 'Lavinia: Eirene?', 'Eirene: It is all wise guys.')


class AddressedRule(unittest.TestCase):
    def test_the_called_name_speaks_next(self):
        self.assertEqual(lr.addressed_renames([LAVINIA]), {LAVINIA['chunkId']: {'???': 'Eirene'}})

    def test_a_label_that_spoke_while_the_unknown_one_did_is_someone_else(self):
        c = chunk('s#0', '???: You, cannot.', 'Rosmontis: How?', '???: ......', "Kal'tsit: Rosmontis, fall back.",
                  'Rosmontis: But I...')
        self.assertEqual(lr.addressed_renames([c]), {})

    def test_a_long_silence_between_unknown_lines_drops_the_chunk(self):
        c = chunk('s#0', '???: Mum calls me Amiya.', 'a', 'b', 'c', 'd', '???: You are finally here.',
                  'Theresa: Oracle?', 'Oracle: Yes.')
        self.assertEqual(lr.addressed_renames([c]), {})

    def test_the_call_must_open_the_line(self):
        c = chunk('s#0', '???: Stay here.', 'Lavinia: Is that you, Eirene?', 'Eirene: It is.')
        self.assertEqual(lr.addressed_renames([c]), {})


class RevealRule(unittest.TestCase):
    def test_the_first_line_of_a_new_label_after_the_unknown_one(self):
        c = chunk('s#0', 'Matterhorn: What is he doing?', '???: Cut him a break.', 'Vulcan: He worked hard on it.',
                  'Matterhorn: Vulcan.')
        self.assertEqual(lr.reveal_renames([c]), {'s#0': {'???': 'Vulcan'}})

    def test_who_are_you_is_the_interlocutor(self):
        c = chunk('s#0', 'Capone: Too bad.', '???: Hey, excuse me.', "Mafioso: Who're you?!", 'Capone: Hm.')
        self.assertEqual(lr.reveal_renames([c]), {})

    def test_a_second_new_label_right_after_drops_the_chunk(self):
        c = chunk('s#0', 'Child: Hm?', '???: Compass?', "Guard Operator: I'm here, Dr. Warfarin.", 'Warfarin: Good.')
        self.assertEqual(lr.reveal_renames([c]), {})


class RuleOrder(unittest.TestCase):
    def test_the_kill_switch_keeps_the_introduction_rule_alone(self):
        old = os.environ.get('LABEL_RENAMES_CHUNK_RULES')
        os.environ['LABEL_RENAMES_CHUNK_RULES'] = 'intro'
        try:
            out, counts = lr.all_chunk_renames([WAKE, RED, TONYA, LAVINIA])
        finally:
            if old is None:
                del os.environ['LABEL_RENAMES_CHUNK_RULES']
            else:
                os.environ['LABEL_RENAMES_CHUNK_RULES'] = old
        self.assertEqual(out, lr.chunk_renames([WAKE, RED, TONYA, LAVINIA]))
        self.assertEqual(counts, {'intro': 1})

    def test_the_default_runs_every_rule_but_reveal(self):
        out, counts = lr.all_chunk_renames([WAKE, RED, TONYA, LAVINIA], list(lr.DEFAULT_RULES))
        self.assertEqual(sorted(out), sorted(c['chunkId'] for c in (WAKE, RED, TONYA, LAVINIA)))
        self.assertNotIn('reveal', counts)

    def test_a_chunk_keeps_the_first_rules_answer(self):
        out, counts = lr.all_chunk_renames([WAKE], ['intro', 'reveal'])
        self.assertEqual(out, {WAKE['chunkId']: {'???': 'Mon3tr'}})
        self.assertEqual(counts, {'intro': 1, 'reveal': 0})


if __name__ == '__main__':
    unittest.main()
