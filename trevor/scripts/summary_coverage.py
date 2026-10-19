#!/usr/bin/env python3
"""Name coverage of the P2 group summaries: does a group summary reach every part of its group? No model.

For each multi-story group, the names that recur in its story summaries (artifacts/p2/stories.jsonl) are the
capitalized words found in at least a quarter of them, and in at least 3; the measure is the share of those names the
served group summary (artifacts/p2/groups.jsonl, method D where present, else C: common.served_group_rows) leaves out.
A word counts as a name only when the group's story summaries use it at least once inside a sentence (so "Finally"
and "Meanwhile" at a sentence start do not) and it is not one of NOISE. A possessive counts as its name ("Kal'tsit's").
Why: "Give a summary of chapter 15" returned a method-C summary of Episode 15 that covered only its first four
stories (design/experiment-log.md, 2026-10-19, "Group summaries that stop after the first stories").

  python3 scripts/summary_coverage.py [--threshold 0.4] [--min-stories 6] [--method served|C|D] [--out FILE] [--top N]
Prints the groups with --min-stories or more stories, worst first, and writes --out (artifacts/p2/coverage.json: per
group its story count, method, recurring names, missing names and share). Exit 2 when any of those groups misses more
than --threshold of its names, else 0; update.sh runs it after the group summaries as a check.
"""
import argparse, collections, json, math, os, re, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py sits beside the scripts
import common
from common import ROOT, read_derived

P2 = os.path.join(ROOT, 'artifacts', 'p2')
# Capitalized words that are not names: sentence openers, pronouns, determiners and connectives the 12B starts
# sentences with, and titles. Most are already out by the inside-a-sentence rule; this list catches the rest.
NOISE = frozenset('''
A About Above Across After Afterward Afterwards Again Against Ahead Along Already Also Although Amid Amidst Among An And
Another Any Around As At Back Because Before Behind Being Below Beneath Beside Besides Between Beyond Both But By
Consequently Despite Determined During Each Earlier Either Elsewhere Even Eventually Every Feeling Finally First
Following For Fearing From Furthermore Having He Her Here Hers Herself Him Himself His However I If In Inside Instead
Into It Its Itself Just Knowing Later Like Many Meanwhile Moreover Most Much My Near Nearby Neither Never Nevertheless
No Nonetheless Nor Not Now Of Off On Once One Only Or Other Others Our Out Outside Over Overall Realizing Recognizing
Seeing Seeking Several She Since So Some Soon Still Subsequently Such Suddenly That The Their Theirs Them Themselves
Then There Therefore These They This Those Though Through Throughout Thus To Together Too Toward Towards Two Three
Ultimately Under Unable Unaware Unlike Until Upon Us Using We What When Whereas Whether Which While Who Whose Why With
Within Without Yet You Your Mr Mrs Ms Dr Sir Lady Lord
'''.split())
WORD = re.compile(r"[A-Za-z][A-Za-z0-9]*(?:['’-][A-Za-z0-9]+)*")
SENTENCE_END = re.compile(r'[.!?]["\'’”)\]]*\s*$')


def words(text):
    """(word, inside a sentence) for every word of a text, a possessive cut to its name."""
    for m in WORD.finditer(text):
        w = re.sub(r"['’]s$", '', m.group())
        before = text[:m.start()].rstrip(' \t"\'“‘(')
        yield w, bool(before) and not before.endswith('\n') and not SENTENCE_END.search(before)


def names_in(text):
    """Capitalized words of a text (any position), the set a group summary is checked against."""
    return {w for w, _ in words(text) if w[0].isupper()}


def recurring_names(summaries):
    """The names found in at least a quarter (and 3) of these story summaries, used inside a sentence at least once."""
    per, inside = collections.Counter(), set()
    for t in summaries:
        seen = set()
        for w, mid in words(t):
            if w[0].isupper() and w not in NOISE:
                seen.add(w)
                if mid:
                    inside.add(w)
        per.update(seen)
    need = max(3, math.ceil(len(summaries) / 4))
    return sorted(w for w, n in per.items() if n >= need and w in inside)


def measure(group_rows, story_summaries):
    """{groupId: {stories, method, names, missing, share}} for each group row against its stories' summaries."""
    out = {}
    for g in group_rows:
        summ = [story_summaries[s] for s in g['storyIds'] if s in story_summaries]
        rec = recurring_names(summ)
        have = names_in(g['summary'])
        missing = [w for w in rec if w not in have]
        out[g['groupId']] = {'stories': len(g['storyIds']), 'method': g['method'], 'names': len(rec),
                             'missing': missing, 'share': round(len(missing) / len(rec), 4) if rec else 0.0}
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--threshold', type=float, default=0.4, help='largest missing share that passes (0.4)')
    ap.add_argument('--min-stories', type=int, default=6, help='groups this long or longer are ranked and checked (6)')
    ap.add_argument('--method', choices=('served', 'C', 'D'), default='served',
                    help='served: D where present, else C (SUMMARY_METHOD_D=0: C); or one method only')
    ap.add_argument('--groups', default=os.path.join(P2, 'groups.jsonl'))
    ap.add_argument('--stories', default=os.path.join(P2, 'stories.jsonl'))
    ap.add_argument('--out', default=os.path.join(P2, 'coverage.json'))
    ap.add_argument('--top', type=int, default=0, help='print only the N worst (0: every ranked group)')
    a = ap.parse_args(argv)
    rows = read_derived(a.groups)
    rows = common.served_group_rows(rows) if a.method == 'served' else [r for r in rows if r['method'] == a.method]
    stories = {r['storyId']: r['summary'] for r in read_derived(a.stories)}
    res = measure(rows, stories)
    ranked = sorted((g for g, r in res.items() if r['stories'] >= a.min_stories), key=lambda g: (-res[g]['share'], g))
    flagged = [g for g in ranked if res[g]['share'] > a.threshold]
    print(f'coverage: {len(ranked)} groups of {a.min_stories}+ stories ({a.method}), {len(flagged)} above {a.threshold}')
    print(f"{'group':<14} {'stories':>7} {'method':>6} {'names':>5} {'missing':>7} {'share':>6}  missing names")
    for g in ranked[:a.top or None]:
        r = res[g]
        print(f"{g:<14} {r['stories']:>7} {r['method']:>6} {r['names']:>5} {len(r['missing']):>7} {r['share']:>6.2f}  "
              + ', '.join(r['missing'][:8]) + (' ...' if len(r['missing']) > 8 else ''))
    common.write_json_atomic(a.out, {'threshold': a.threshold, 'minStories': a.min_stories, 'method': a.method,
                                     'flagged': flagged, 'groups': res}, indent=1)
    return 2 if flagged else 0


if __name__ == '__main__':
    sys.exit(main())
