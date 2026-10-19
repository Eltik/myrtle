#!/usr/bin/env python3
"""The chunk-scoped renames of the "???" label (scripts/label_renames.py `chunk_renames`, 2026-10-19).

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


if __name__ == '__main__':
    unittest.main()
