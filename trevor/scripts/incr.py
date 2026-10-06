#!/usr/bin/env python3
"""Incremental model stages: redo a unit only when its inputs or its prompt changed.

A stage skips a unit only when its key matches (unit id, sha of the unit's inputs, sha of the prompt
and grammar), the rule scripts/deaths.py uses for death mentions (design/trevor-updates.md). The input
sha is taken over the exact text the model is sent, so a changed story changes its summary, which
changes its group's input, and so on down the chain, with no dependency graph to maintain.
Output files stay JSONL with one row per unit: a redone unit replaces its row in place (atomic
rewrite), a new unit is appended, and at the start of a real run rows of units that no longer exist
and duplicate rows are dropped, so no reader ever sees two rows for one unit.
  PLAN=1          print how many units each stage would redo and why (new, input changed, prompt
                  changed, upstream pending), write nothing, call no model server
  KEY_IDS_ONLY=1  kill switch: skip on the unit id alone and append, exactly the behavior before
                  2026-09-28 (rows still carry inputSha; nothing is replaced or dropped)
  CHUNKS=path     read this chunks.jsonl instead of artifacts/chunks.jsonl (simulating an update)
  python3 scripts/incr.py backfill   one-time migration, see backfill()
"""
import collections, json, os, sys, tempfile, threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
from common import ROOT, load_script, read_jsonl, sha16

PLAN = os.environ.get('PLAN') == '1'
IDS_ONLY = os.environ.get('KEY_IDS_ONLY') == '1'
# Listed at most this many unit ids per reason in a plan, enough to read a simulated update.
SHOW = int(os.environ.get('PLAN_SHOW', '12'))


def chunks_path():
    return os.environ.get('CHUNKS') or os.path.join(ROOT, 'artifacts', 'chunks.jsonl')


def input_sha(x):
    return sha16(x if isinstance(x, str) else json.dumps(x, ensure_ascii=False, sort_keys=True))


def write_jsonl(path, rows):
    # Temp file in the same directory, then rename: a reader sees the old file or the new one, never half. (A
    # mkstemp file, mode 0600, unlike common.replace_jsonl's: kept so a stage's output file mode is unchanged.)
    fd, tmp = tempfile.mkstemp(dir=os.path.dirname(path), prefix='.' + os.path.basename(path) + '.')
    with os.fdopen(fd, 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    os.replace(tmp, path)


class Stage:
    """One model stage's output file and its units. `key` maps a row to its unit id; unit() registers
    every unit that exists in the current corpus with its exact model input."""

    def __init__(self, name, path, key, psha):
        self.name, self.path, self.key, self.psha = name, path, key, psha
        self.rows = read_jsonl(path)
        self.have = {}
        for r in self.rows:
            self.have[key(r)] = r  # newest row wins, as appended
        self.units = {}
        self.lock = threading.Lock()
        self.row_input = None  # backfill: rebuild an old row's input from the row itself, when it records it
        self.no_backfill = None  # backfill: why this stage's old rows cannot be vouched for

    def unit(self, k, model_input, upstream=False):
        """`upstream`: the unit reads a model output that its own stage will redo (PLAN only), so this
        unit will be redone too once that has run."""
        if k not in self.units:  # a unit id seen twice keeps its first input, as the first run did
            self.units[k] = (input_sha(model_input), upstream)

    def reason(self, k):
        r = self.have.get(k)
        if r is None:
            return 'new'
        if IDS_ONLY:
            return None
        if r.get('promptSha') != self.psha:
            return 'prompt changed'
        if r.get('inputSha') != self.units[k][0]:
            return 'input changed'
        if self.units[k][1]:
            return 'upstream pending'
        return None

    def stale(self):
        """Unit ids a run would redo, for downstream stages' plans."""
        return {k for k in self.units if self.reason(k)}

    def plan(self, order, live=None, show=True):
        """`order`: unit ids in run order (a subset under CHRONO_GROUPS and the like); `live`: ids whose
        rows are kept, default every registered unit. Returns the ids to do, or None under PLAN."""
        if IDS_ONLY:
            # Exactly the old skip: every listed id absent from the file, duplicates included.
            todo = [k for k in order if k not in self.have]
            why = collections.Counter({'new': len(todo)}) if todo else collections.Counter()
            ex = {'new': todo[:SHOW]}
        else:
            todo, seen, why, ex = [], set(), collections.Counter(), collections.defaultdict(list)
            for k in order:
                if k in seen:
                    continue
                seen.add(k)
                w = self.reason(k)
                if w:
                    todo.append(k); why[w] += 1
                    if len(ex[w]) < SHOW:
                        ex[w].append(k)
        live = set(self.units) if live is None else live
        gone = 0 if IDS_ONLY else sum(1 for k in self.have if k not in live)
        dup = 0 if IDS_ONLY else len(self.rows) - len(self.have)
        if show or PLAN:
            parts = ', '.join(f'{w} {n}' for w, n in sorted(why.items())) or 'nothing changed'
            print(f'{self.name}: plan {len(todo)} to do of {len(order) if IDS_ONLY else len(set(order))} ({parts}); rows {len(self.rows)}, '
                  f'to drop: gone {gone}, duplicate {dup}' + (' [KEY_IDS_ONLY]' if IDS_ONLY else ''), flush=True)
            if PLAN:
                for w, ks in ex.items():
                    if ks:
                        print(f'  {w}: ' + ', '.join(map(str, ks)) + (' ...' if why[w] > len(ks) else ''), flush=True)
        if PLAN:
            return None
        if gone or dup:
            self.rows = [r for k, r in self.have.items() if k in live]
            self.have = {self.key(r): r for r in self.rows}
            write_jsonl(self.path, self.rows)
        return todo

    def put(self, row):
        """Write one finished unit: append a new one, replace the row of a redone one."""
        k = self.key(row)
        if k in self.units:
            row['inputSha'] = self.units[k][0]
        with self.lock:
            if k in self.have and not IDS_ONLY:
                self.rows = [row if self.key(r) == k else r for r in self.rows]
                write_jsonl(self.path, self.rows)
            else:
                self.rows.append(row)
                with open(self.path, 'a') as f:
                    f.write(json.dumps(row, ensure_ascii=False) + '\n')
            self.have[k] = row


module = load_script


def p2_pending():
    """(story ids, group ids) whose P2 summaries a run would redo; a group counts when any of its method
    rows would be redone or any member story is pending. Downstream plans mark their readers of these."""
    p2 = module('p2-summaries')
    st = p2.story_stage()[0]
    stories = st.stale()
    gst = p2.group_stage(pending_stories=stories)[0]
    groups = {g for g, _ in gst.stale()}
    by_story = {k: r['groupId'] for k, r in st.have.items()}
    # A one-story group has no group summary; readers use its story summary in its place.
    groups |= {by_story.get(s) for s in stories}
    return stories, groups


# Every stage converted to the incremental key: (script, builder). A builder returns the Stage first.
STAGES = [('p2-summaries', 'story_stage'), ('p2-summaries', 'group_stage'), ('chrono', 'extract_stage'),
          ('chrono', 'place_stage'), ('chrono', 'events_stage'), ('chrono', 'exceptions_stage'),
          ('entities', 'extract_stage'), ('dossiers', 'gen_stage'), ('primers', 'gen_stage'),
          ('answer_bank', 'questions_stage')]


def backfill():
    """One-time migration: rows written before 2026-09-28 carry no inputSha, so every unit would count as
    changed. This writes the sha of each unit's CURRENT input into its rows. That is valid only because
    the corpus and every upstream output are unchanged since those rows were built (no asset update
    between the model runs and this migration); run it once, before any update. A stage whose old rows
    record their own inputs rebuilds them from the row (dossiers), and a stage whose inputs demonstrably
    changed after its run is left out (chrono place). Rows that already carry
    an inputSha are never touched, so running it again cannot hide a real change. Bank questions also
    lacked promptSha; the current Q_SYS is written under the same assumption."""
    mods = {}
    for script, builder in STAGES:
        m = mods.setdefault(script, module(script))
        st = getattr(m, builder)()[0]
        if st.no_backfill:
            print(f'{st.name}: not backfilled, {st.no_backfill}', flush=True)
            continue
        n = 0
        for r in st.rows:
            k = st.key(r)
            if 'inputSha' not in r and k in st.units:
                r['inputSha'] = input_sha(st.row_input(r)) if st.row_input else st.units[k][0]; n += 1
            if 'promptSha' not in r:
                r['promptSha'] = st.psha
        before = sum(1 for _ in open(st.path))
        write_jsonl(st.path, st.rows)
        after = sum(1 for _ in open(st.path))
        print(f'{st.name}: {st.path[len(ROOT) + 1:]} rows {before} -> {after}, inputSha written {n}, '
              f'rows of units not in the corpus {sum(1 for r in st.rows if st.key(r) not in st.units)}', flush=True)
        assert before == after


def plan_all():
    """PLAN=1 over every converted stage."""
    assert PLAN, 'run as PLAN=1 python3 scripts/incr.py plan'
    mods = {}
    for script, builder in STAGES:
        m = mods.setdefault(script, module(script))
        st, order, live = getattr(m, builder)()[:3]
        st.plan(order, live)


if __name__ == '__main__':
    {'backfill': backfill, 'plan': plan_all}[sys.argv[1]]()
