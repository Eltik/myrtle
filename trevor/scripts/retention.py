#!/usr/bin/env python3
"""Retention: what the box keeps of its logs, eval runs, spool, releases and asset manifests, and a disk check
(design/trevor-self-contained-deploy.md section 6 step 9; design/trevor-vps-runbook.md section 12).

A dry run by default: every entry over the policy is listed and nothing is deleted. --apply deletes the listed
entries (the list is printed in full first); the asset manifests need --apply-assets as well, because that tree is
the asset watcher's, not Trevor's.

  retention.py [--apply] [--apply-assets] [--only logs,runs,spool,releases,assets,disk]
               [--trevor-dir DIR] [--home DIR] [--assets-dir DIR] [--spool-dir DIR]

The policy (every number is an environment variable; the defaults in brackets):

  logs      <home>/logs/<stamp>-<cmd>/ (trevor-vps-update.sh) and <trevor>/artifacts/update/<stamp>/ (update.sh):
            keep the newest RETAIN_LOGS [30] by name, and the log of the run that made the current and the previous
            release (same <stamp> as the release directory). A log root inside a release directory is left to the
            release policy. trevor-vps-update.sh still deletes its logs older than 30 days on every run (as before).
  runs      <trevor>/eval/runs/*.json and <home>/state/gates/eval/runs/*.json (the gate's recall runs): keep the
            newest RETAIN_RUNS [50] by run time, the newest run of every gold set in runs.jsonl (what `eval run
            --baseline last` loads), every run whose id appears in design/ or whose label appears on a design/ line
            that says "baseline", and every run a published version's gate report names
            (<home>/state/versions/*/activate.json). runs.jsonl, the index, is never deleted.
  spool     TREVOR_SPOOL_DIR (else <trevor>/artifacts/spool, the worker's default): results the worker could not
            deliver, resent when it starts; files older than RETAIN_SPOOL_DAYS [14] days go.
  releases  <home>/releases/<stamp>-<version>/: keep the current release (the target of <home>/current and of
            <trevor>/artifacts), the previous live one (the release of the last version state/history.jsonl names
            before the current one, as `rollback` picks it; a release that never went live is never it), the
            stage's base (state/stage-base) and the newest RETAIN_RELEASES [5] by name. trevor-vps-update.sh's
            prune_releases (KEEP_RELEASES, 3: the live release, the previous live one, then the newest others) runs
            first after every publish and is stricter, so this catches what it never sees (a release left by a
            failed activate is pruned by the next successful publish; one left before a box stops publishing is not).
            --previous-release prints the previous live release (empty when there is none) and does nothing else.
  assets    <assets>/ArkAssets/<server>/<32 hex>.idx: the resource manifest of one res_version each; keep the newest
            2 by mtime (the current and the previous version). The download is one tree per server updated in place,
            and assets/run.mjs pruneOrphans already deletes the bundles and manifests the current hot-update list does
            not name; which old bundle belongs to which version needs that list (the network), so bundles are never
            touched here. Deleted only with --apply --apply-assets.
  disk      the filesystems of <home>, <trevor> and <assets>: one "WARNING: disk low" line each when the free space
            is below the smaller of DISK_MIN_FREE_GB [10] GB and DISK_MIN_FREE_PCT [10] percent of the filesystem.
            trevor-vps-update.sh posts those lines to Discord; this script never posts anything.

Exit: 0 = nothing is over the policy; 2 = something is (listed in a dry run, deleted with --apply); 1 = an error (a
bad argument or variable, a deletion that failed). The disk warning does not change the exit code. The last line is
a JSON summary. Nothing is followed through a symlink: a symlinked entry is removed as a link, and every deletion is
checked to lie inside its policy's root.
"""
import argparse, json, os, re, shutil, sys, time

POLICIES = ('logs', 'runs', 'spool', 'releases', 'assets', 'disk')
IDX = re.compile(r'^[0-9a-f]{32}\.idx$')
RUN_PREFIX = re.compile(r'^(\d+)-')
RUN_ID_IN_JSON = re.compile(r'"runId"\s*:\s*"([^"]+)"')
STAMP = re.compile(r'^(\d{8}T\d{6}Z)-')


class Fail(Exception):
    pass


def say(msg):
    print(f"== {time.strftime('%H:%M:%S', time.gmtime())} {msg}", flush=True)


def env_num(name, default, kind=int):
    raw = os.environ.get(name, '').strip()
    if not raw:
        return default
    try:
        v = kind(raw)
    except ValueError:
        raise Fail(f"{name}={raw!r} is not a number")
    if v < 0:
        raise Fail(f"{name}={raw!r} is negative")
    return v


def real(p):
    return os.path.realpath(p)


def inside(path, root):
    """True when PATH (not resolved: a symlink entry counts as itself) lies strictly inside the resolved ROOT."""
    parent = real(os.path.dirname(os.path.abspath(path)))
    r = real(root)
    return parent == r or parent.startswith(r + os.sep)


def freed_bytes(path):
    """Bytes a deletion frees: regular files with no other hard link (a release shares most of its files)."""
    if os.path.islink(path) or not os.path.isdir(path):
        try:
            st = os.lstat(path)
            return st.st_size if st.st_nlink == 1 and not os.path.islink(path) else 0
        except OSError:
            return 0
    total = 0
    for d, _, files in os.walk(path):
        for f in files:
            try:
                st = os.lstat(os.path.join(d, f))
            except OSError:
                continue
            if st.st_nlink == 1 and not os.path.islink(os.path.join(d, f)):
                total += st.st_size
    return total


def human(n):
    for unit in ('B', 'KB', 'MB', 'GB'):
        if n < 1000 or unit == 'GB':
            return f"{n:.0f} {unit}" if unit == 'B' else f"{n:.1f} {unit}"
        n /= 1000


class Plan:
    """One policy over one root: what it keeps (with the reason for anything beyond the newest N) and what goes."""

    def __init__(self, policy, root, flag='--apply'):
        self.policy, self.root, self.flag = policy, root, flag
        self.total = 0
        self.kept = []      # (path, reason) for entries kept by a rule other than "newest N"
        self.newest = 0
        self.prune = []
        self.notes = []

    def head(self):
        extra = f", {len(self.kept)} named" if self.kept else ''
        return (f"{self.policy} {self.root}: {self.total} entries, keep {self.total - len(self.prune)} "
                f"(newest {self.newest}{extra}), over the policy {len(self.prune)}")


def keep_newest(plan, entries, n, protected):
    """ENTRIES sorted newest first; keep the first N and every protected one (path -> reason)."""
    plan.total = len(entries)
    for i, e in enumerate(entries):
        if i < n:
            plan.newest += 1
        elif e in protected:
            plan.kept.append((e, protected[e]))
        else:
            plan.prune.append(e)


def subdirs(root):
    return sorted((os.path.join(root, x) for x in os.listdir(root)
                   if os.path.isdir(os.path.join(root, x)) or os.path.islink(os.path.join(root, x))),
                  reverse=True)


# ---------------------------------------------------------------- the release state, shared by logs and releases
def release_of(path):
    """The release directory a live symlink resolves to (<releases>/<name>/artifacts -> <releases>/<name>)."""
    if not os.path.lexists(path):
        return None
    t = real(path)
    return os.path.dirname(t) if os.path.basename(t) == 'artifacts' else t


def read_jsonl(path):
    try:
        return [r for r in (json.loads(l) for l in open(path) if l.strip()) if isinstance(r, dict)]
    except (OSError, ValueError):
        return []


def previous_live_release(home, current=()):
    """The release that was live before the current one, or None. The previous version is the last version
    state/history.jsonl names before its last row's (the rule `trevor-vps-update.sh rollback` uses); its release is
    the newest existing state/releases.jsonl row for that version that is not a CURRENT release (rollback picks the
    same row). A release that never went live (a failed activate, a failed flip) has no history row, so it is never
    the previous one (chaos run 2026-10-19, mismatch 6)."""
    hist = [r.get('version') for r in read_jsonl(os.path.join(home, 'state', 'history.jsonl')) if r.get('version')]
    if not hist:
        return None
    prev = [v for v in hist if v != hist[-1]]
    if not prev:
        return None
    cur = {real(c) for c in current}
    for row in reversed(read_jsonl(os.path.join(home, 'state', 'releases.jsonl'))):
        r = row.get('release')
        if row.get('version') == prev[-1] and r and os.path.isdir(r) and real(r) not in cur:
            return real(r)
    return None


def release_state(a):
    releases = os.path.join(a.home, 'releases')
    current = {r for r in (release_of(os.path.join(a.home, 'current')), release_of(os.path.join(a.trevor_dir, 'artifacts')))
               if r and real(os.path.dirname(r)) == real(releases)}
    previous = previous_live_release(a.home, current)
    base = None
    try:
        base = release_of(open(os.path.join(a.home, 'state', 'stage-base')).read().strip())
    except OSError:
        pass
    return releases, {real(c) for c in current}, previous, base


# ---------------------------------------------------------------- policies
def plan_logs(a):
    n = env_num('RETAIN_LOGS', 30)
    releases, current, previous, _ = release_state(a)
    stamps = {}
    for r, why in [(c, 'the current release') for c in current] + ([(previous, 'the previous release')] if previous else []):
        m = STAMP.match(os.path.basename(r))
        if m:
            stamps[m.group(1)] = f"log of {why}"
    plans = []
    for root in (os.path.join(a.home, 'logs'), os.path.join(a.trevor_dir, 'artifacts', 'update'),
                 os.path.join(a.home, 'state', 'stages', 'logs')):
        if not os.path.isdir(root):
            continue
        plan = Plan('logs', root)
        if real(root).startswith(real(releases) + os.sep):
            plan.notes.append('inside a release directory: left to the release policy')
            plans.append(plan)
            continue
        entries = subdirs(root)
        protected = {}
        for e in entries:
            for s, why in stamps.items():
                if os.path.basename(e).startswith(s + '-'):
                    protected[e] = why
        keep_newest(plan, entries, max(n, 1), protected)
        plans.append(plan)
    return plans


def design_refs(a):
    """(text of every design/ doc, the lines of them that say "baseline")."""
    texts, lines = [], []
    root = os.path.join(a.trevor_dir, 'design')
    for d, _, files in os.walk(root):
        for f in files:
            if f.endswith('.md'):
                try:
                    t = open(os.path.join(d, f), encoding='utf-8', errors='replace').read()
                except OSError:
                    continue
                texts.append(t)
                lines += [l for l in t.splitlines() if 'baseline' in l.lower()]
    return '\n'.join(texts), lines


def gate_report_runs(a):
    ids = set()
    vdir = os.path.join(a.home, 'state', 'versions')
    if os.path.isdir(vdir):
        for v in os.listdir(vdir):
            try:
                ids.update(RUN_ID_IN_JSON.findall(open(os.path.join(vdir, v, 'activate.json'), errors='replace').read()))
            except OSError:
                pass
    return ids


def plan_runs(a):
    n = env_num('RETAIN_RUNS', 50)
    design_text, baseline_lines = design_refs(a)
    gate_ids = gate_report_runs(a)
    plans = []
    for base in (os.path.join(a.trevor_dir, 'eval'), os.path.join(a.home, 'state', 'gates', 'eval')):
        runs = os.path.join(base, 'runs')
        if not os.path.isdir(runs):
            continue
        plan = Plan('runs', runs)
        newest_per_set = {}
        try:
            with open(os.path.join(base, 'runs.jsonl'), errors='replace') as f:
                for i, line in enumerate(f, 1):
                    if not line.strip():
                        continue
                    try:
                        r = json.loads(line)
                        newest_per_set[r.get('goldsetSha')] = r['runId']
                    except (ValueError, KeyError, TypeError, AttributeError):
                        plan.notes.append(f"runs.jsonl line {i} does not parse; skipped")
        except OSError:
            plan.notes.append('no runs.jsonl beside it')
        files = [os.path.join(runs, x) for x in os.listdir(runs)
                 if x.endswith('.json') and (os.path.isfile(os.path.join(runs, x)) or os.path.islink(os.path.join(runs, x)))]

        def when(p):
            m = RUN_PREFIX.match(os.path.basename(p))
            return (int(m.group(1)) if m else int(os.lstat(p).st_mtime), os.path.basename(p))
        files.sort(key=when, reverse=True)
        last_ids = set(newest_per_set.values())
        protected = {}
        for p in files:
            rid = os.path.basename(p)[:-len('.json')]
            label = RUN_PREFIX.sub('', rid, count=1)
            if rid in last_ids:
                protected[p] = 'the newest run of its gold set (eval --baseline last)'
            elif rid in gate_ids:
                protected[p] = "named by a published version's gate report"
            elif rid in design_text:
                protected[p] = 'named in design/'
            elif len(label) >= 4 and any(re.search(r'(?<![\w.-])' + re.escape(label) + r'(?![\w.-])', l)
                                         for l in baseline_lines):
                protected[p] = 'its label is named as a baseline in design/'
        keep_newest(plan, files, n, protected)
        plans.append(plan)
    return plans


def plan_spool(a):
    days = env_num('RETAIN_SPOOL_DAYS', 14, float)
    root = a.spool_dir
    if not os.path.isdir(root):
        return []
    plan = Plan('spool', root)
    cut = time.time() - days * 86400
    files = sorted(os.path.join(root, x) for x in os.listdir(root)
                   if os.path.isfile(os.path.join(root, x)) or os.path.islink(os.path.join(root, x)))
    plan.total = len(files)
    for p in files:
        if os.lstat(p).st_mtime < cut:
            plan.prune.append(p)
    plan.newest = plan.total - len(plan.prune)
    plan.notes.append(f"files older than {days:g} days go")
    return [plan]


def plan_releases(a):
    n = env_num('RETAIN_RELEASES', 5)
    releases, current, previous, base = release_state(a)
    if not os.path.isdir(releases):
        return []
    plan = Plan('releases', releases)
    entries = subdirs(releases)
    protected = {}
    for e in entries:
        r = real(e)
        if r in current:
            protected[e] = 'the current release (the live symlink target)'
        elif previous and r == previous:
            protected[e] = 'the previous release'
        elif base and r == real(base):
            protected[e] = "the stage's base"
    if not current:
        plan.notes.append('no live symlink resolves into this directory')
    keep_newest(plan, entries, n, protected)
    return [plan]


def plan_assets(a):
    root = os.path.join(a.assets_dir, 'ArkAssets')
    if not os.path.isdir(root):
        return []
    plans = []
    for server in sorted(os.listdir(root)):
        sdir = os.path.join(root, server)
        if not os.path.isdir(sdir) or os.path.islink(sdir):
            continue
        plan = Plan('assets', sdir, flag='--apply --apply-assets')
        idx = [os.path.join(sdir, x) for x in os.listdir(sdir) if IDX.match(x) and os.path.isfile(os.path.join(sdir, x))]
        idx.sort(key=lambda p: (os.lstat(p).st_mtime, p), reverse=True)
        keep_newest(plan, idx, 2, {})
        plans.append(plan)
    return plans


def disk_check(a):
    gb = env_num('DISK_MIN_FREE_GB', 10, float)
    pct = env_num('DISK_MIN_FREE_PCT', 10, float)
    seen, low = set(), []
    for p in (a.home, a.trevor_dir, a.assets_dir):
        if not os.path.exists(p):
            continue
        dev = os.stat(p).st_dev
        if dev in seen:
            continue
        seen.add(dev)
        u = shutil.disk_usage(p)
        limit = min(gb * 1e9, u.total * pct / 100)
        share = 100 * u.free / u.total if u.total else 0
        if u.free < limit:
            line = (f"WARNING: disk low: {p} has {u.free / 1e9:.1f} GB free of {u.total / 1e9:.1f} GB ({share:.1f}%), "
                    f"under {limit / 1e9:.1f} GB (the smaller of DISK_MIN_FREE_GB {gb:g} and DISK_MIN_FREE_PCT {pct:g}%)")
            say(line)
            low.append(line)
        else:
            say(f"disk: {p} has {u.free / 1e9:.1f} GB free of {u.total / 1e9:.1f} GB ({share:.1f}%)")
    return low


# ---------------------------------------------------------------- deletion
def delete(path, root):
    if not inside(path, root):
        raise Fail(f"refusing {path}: not inside {root}")
    if os.path.islink(path) or not os.path.isdir(path):
        os.remove(path)
    else:
        shutil.rmtree(path)


class Parser(argparse.ArgumentParser):
    def error(self, message):  # exit 1, not argparse's 2 (2 means "something was over the policy")
        self.print_usage(sys.stderr)
        print(f"{self.prog}: error: {message}", file=sys.stderr)
        sys.exit(1)


def main(argv=None):
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    ap = Parser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--apply', action='store_true', help='delete what is over the policy (default: list it)')
    ap.add_argument('--apply-assets', action='store_true', help='with --apply, also delete old asset manifests')
    ap.add_argument('--only', default=','.join(POLICIES), help='comma-separated policies (default: all)')
    ap.add_argument('--trevor-dir', default=os.environ.get('TREVOR_DIR') or here)
    ap.add_argument('--home', default=os.environ.get('TREVOR_HOME') or '/var/lib/trevor')
    ap.add_argument('--assets-dir', default=os.environ.get('ASSETS_DIR'))
    ap.add_argument('--spool-dir', default=os.environ.get('TREVOR_SPOOL_DIR'))
    ap.add_argument('--previous-release', action='store_true',
                    help="print the previous live release (trevor-vps-update.sh's prune_releases keeps it) and exit 0")
    a = ap.parse_args(argv)
    a.trevor_dir = os.path.abspath(a.trevor_dir)
    a.home = os.path.abspath(a.home)
    if a.previous_release:
        print(release_state(a)[2] or '')
        return 0
    a.assets_dir = os.path.abspath(a.assets_dir or os.path.join(a.trevor_dir, '..', 'assets'))
    a.spool_dir = os.path.abspath(a.spool_dir or os.path.join(a.trevor_dir, 'artifacts', 'spool'))
    only = [p.strip() for p in a.only.split(',') if p.strip()]
    bad = [p for p in only if p not in POLICIES]
    if bad:
        ap.error(f"unknown policy {', '.join(bad)} (one of {', '.join(POLICIES)})")

    summary = {'applied': a.apply, 'overPolicy': 0, 'deleted': 0, 'freedBytes': 0, 'kept': 0, 'errors': 0,
               'diskLow': [], 'policies': {}}
    try:
        mode = 'apply' if a.apply else 'dry run; --apply deletes'
        if a.apply and not a.apply_assets and 'assets' in only:
            mode += ', asset manifests only with --apply-assets'
        say(f"retention ({mode})")
        plans = []
        builders = {'logs': plan_logs, 'runs': plan_runs, 'spool': plan_spool, 'releases': plan_releases,
                    'assets': plan_assets}
        for p in only:
            if p in builders:
                plans += builders[p](a)
        # The whole list first, then the deletions.
        todo = []
        for plan in plans:
            say(plan.head())
            for note in plan.notes:
                print(f"  note: {note}")
            for path, why in plan.kept:
                print(f"  keep  {path}  ({why})")
            allowed = a.apply and (plan.policy != 'assets' or a.apply_assets)
            for path in plan.prune:
                size = freed_bytes(path)
                summary['freedBytes'] += size
                print(f"  {'prune' if allowed else 'over '} {path}  ({human(size)})")
                if allowed:
                    todo.append((path, plan.root))
            if plan.prune and not allowed:
                print(f"  ({len(plan.prune)} listed, not deleted: needs {plan.flag})")
            summary['overPolicy'] += len(plan.prune)
            summary['kept'] += plan.total - len(plan.prune)
            s = summary['policies'].setdefault(plan.policy, {'entries': 0, 'over': 0, 'listed': []})
            s['entries'] += plan.total
            s['over'] += len(plan.prune)
            s['listed'] += plan.prune
        for path, root in todo:
            try:
                delete(path, root)
                summary['deleted'] += 1
            except (OSError, Fail) as e:
                summary['errors'] += 1
                print(f"ERROR: could not delete {path}: {e}", file=sys.stderr)
        verb = 'deleted' if a.apply else 'would free'
        say(f"retention: {summary['overPolicy']} over the policy, {summary['deleted']} deleted, "
            f"{verb} about {human(summary['freedBytes'])}"
            + ('' if a.apply or not summary['overPolicy'] else ' (dry run: nothing deleted)'))
        if 'disk' in only:
            summary['diskLow'] = disk_check(a)
    except Fail as e:
        print(f"ERROR: {e}", file=sys.stderr)
        summary['errors'] += 1
    print(json.dumps(summary))
    if summary['errors']:
        return 1
    return 2 if summary['overPolicy'] else 0


if __name__ == '__main__':
    sys.exit(main())
