#!/usr/bin/env python3
"""Corpus snapshot: pack the derived artifacts into one checksummed tarball, and install one.

Why: a fresh deployment must start from the tables the build machine already derived (chunks, vectors, indexes,
summaries, dossiers, topics, entities, chronology, appearances, bank) and only ever run incremental updates; building
them from nothing on a CPU takes weeks (design/trevor-self-contained-deploy.md, section 6 step 3).

  snapshot.py pack [--out FILE] [--include-models]
      Writes FILE (default trevor-snapshot-<UTC stamp>.tar.zst, or .tar.gz when zstd is not on PATH; zstd runs with one
      thread and a 512 MB long-distance window, about 0.6 GB of memory to write or read). The first member is
      manifest.json: schema, created (UTC, RFC 3339), the git commit of the tree (and whether trevor/ had uncommitted
      changes), every file with its size and sha256, per corpus the chunks.jsonl sha
      (sha16 and sha256) with whether its BM25 index and its vectors were built from that file, and the embedding
      model files the vectors were made with (vectors.meta.json modelFile, modelSha, tokenizerSha). Every file is
      hashed twice, once for the manifest and again while it is written: a file that changed in between (an update
      running) aborts the pack. A size table per top-level entry is printed before writing.
      --include-models also packs models/gte-modernbert-base and models/ettin-reranker-17m-v1 (the embedder and the
      reranker; scripts/fetch-model.sh fetches the same pinned files, and the GGUF models are never packed).
  snapshot.py install FILE [--into DIR] [--force]
      Verifies every member against the manifest (size and sha256, no member missing, extra or repeated) before
      writing anything, then extracts into DIR.tmp and renames it to DIR (default trevor/artifacts), so a reader sees
      the old tree or the whole new one. An existing DIR is refused unless --force; with --force the old tree is
      renamed to DIR.old-<stamp> (kept, never deleted). A symlinked DIR (the VPS release link) is always refused:
      install into a plain directory and adopt it with `trevor-vps-update.sh init`. Packed models go to the models
      directory beside DIR, file by file; a differing existing model file is refused unless --force. The manifest is
      written to DIR/snapshot.json, then the summary and each corpus's chunks sha are printed.
  snapshot.py check FILE
      The verification of install, writing nothing.

What is packed (PACK below, each with its reason): the P0 corpus and the corpora ask and the update read with their
vectors and BM25 indexes (p3a, p3b, p4, p4x, p4-art), the fp32 vector copies (p3b-fp32, p4x-fp32, p4-art-fp32: the
portable vectors a non-Mac host reuses, trevor-reembed.md section 6), and every stage output and resume file
(p2, dossiers, entities, topics, canon, chrono, bank, ...): model stages key their units on these files (incr.py), so
without them an update redoes every unit.

What is left out (LEAVE below): raw game data (../assets, never under artifacts), eval runs (eval/runs, outside
artifacts) and eval outputs (answers, trivia, goldgen-arch, goldgen-v2), update logs (update), the worker spool
(spool), development experiments (p1a-split, p3b-lore, p4-lore, embedft, primers.v1), the query-embedding cache
(router), instance state (publish: another instance's active version and gate baselines; canary), and these, which
the nightly update regenerates in seconds from the rest (2026-10-19: build-prefixes 0.5 s, panels.py 16 s):
  p1a                      build-prefixes (chunks.jsonl plus the game data); its vectors are unused and stale
  panels                   panels.py, run by every `trevor-vps-update.sh daily` before the gates
  sources/all_units.jsonl  the concatenation of sources/units.jsonl and topics/units.v1.jsonl (update.sh build_lores)
  chunks.prev.jsonl        build-corpus --refresh renames chunks.jsonl to it on every refresh
  changes.json             rewritten by every build-corpus --refresh; it describes the build machine's last refresh
Inside packed directories, scratch files are skipped: names ending in .tmp, .log, .pid or .lock, and dotfiles other
than tantivy's .managed.json (mkstemp temporaries start with a dot). A top-level entry in neither list is reported
and left out, so a new artifact is noticed instead of silently shipped or silently dropped.

Stdlib only; run with nice. Exit 0 on success, 1 on a verification failure or a refused install, 2 on usage errors.
"""
import argparse, datetime, hashlib, io, json, os, shutil, subprocess, sys, tarfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ARTIFACTS = os.path.join(ROOT, 'artifacts')
SCHEMA = 1
BLOCK = 1 << 20

# Top-level entries of artifacts/ that a serving instance or an incremental update reads.
PACK = {
    'chunks.jsonl': 'P0 corpus: every update diffs the refetched stories against it',
    'vectors.bin': 'P0 vectors, the reuse source for p3a',
    'vectors.meta.json': 'P0 vectors: model, chunks sha, content shas (reuse keys)',
    'manifest.p0.json': 'P0 build manifest, read by embed-corpus and the derived builders',
    'spoiler.jsonl': 'story spoiler order (gate order input)',
    'unresolved.jsonl': 'P0 unresolved lines, read by the derived builders',
    'bm25': 'P0 BM25 index',
    'p3a': 'stories plus operator archives: search, eval, reuse source for p3b',
    'p3b': "ask's main corpus",
    'p4': 'typed sources, reuse source for p4x (ask --no-game-text reads it)',
    'p4x': 'typed sources plus game text, read by ask for source routes',
    'p4-art': "art units, read by ask's typed fallback",
    'p3b-fp32': 'portable fp32 vectors for p3b (reuse source on a non-Mac host)',
    'p4x-fp32': 'portable fp32 vectors for p4x',
    'p4-art-fp32': 'portable fp32 vectors for p4-art',
    'p2': 'story and group summaries (model stage)',
    'dossiers': 'character dossiers (model stage)',
    'primers': 'primers (model stage), read by the bank seed',
    'entities': 'identities, deaths, real names, attributes, label renames, extract resume files',
    'topics': 'topics, deep topics, candidates and classifications (model stages)',
    'canon': 'IS ending canon and its resume files (model stages)',
    'chrono': 'chronology, reading guide, dates, gate order, storylines',
    'appearances': 'appearance index',
    'bank': 'answer bank, its generated answers and evidence ledger',
    'overview': 'overview entries, read as tool passages',
    'crossref': 'cross-reference tables read by the tools',
    'gametext': 'game text units; carried over when a game table is missing',
    'sources': 'typed source units (p4 input)',
    'art': 'art captions, OCR and units (model stage)',
    'goldgen': 'names.json (read by ask and appearances.py) and the gold-set pipeline state',
}

# Top-level entries deliberately left out, and why.
LEAVE = {
    'p1a': 'regenerable in seconds: build-prefixes',
    'panels': 'regenerable in seconds: panels.py (every daily update)',
    'chunks.prev.jsonl': 'rewritten by every build-corpus --refresh',
    'changes.json': 'rewritten by every build-corpus --refresh',
    'update': 'update logs',
    'spool': 'worker spool',
    'publish': "instance state: another instance's active version and gate baselines",
    'canary': 'instance state: the cf canary file',
    'answers': 'eval outputs',
    'answers.jsonl': 'ask --batch default output',
    'trivia': 'eval set generation',
    'goldgen-arch': 'eval: older gold-set generation',
    'goldgen-v2': 'eval: gold-set generation v2',
    'p1a-split': 'experiment',
    'p3b-lore': 'experiment: the lore v2 dossiers built into P3b (2026-10-01), not served',
    'p4-lore': 'experiment: the lore v2 topics built into P4 (2026-10-01), not served',
    'embedft': 'embedder fine-tune pairs (experiment)',
    'primers.v1': 'old primers copy, read by nothing',
    'router': 'query-embedding cache, rebuilt on demand',
    'snapshot.json': 'the installed snapshot manifest, rewritten by each install',
}
SKIP_FILES = {'sources/all_units.jsonl'}  # regenerable: cat of sources/units.jsonl and topics/units.v1.jsonl
SCRATCH_SUFFIXES = ('.tmp', '.log', '.pid', '.lock')
KEEP_DOTFILES = {'.managed.json'}  # tantivy's list of the files it manages
MODEL_DIRS = ('gte-modernbert-base', 'ettin-reranker-17m-v1')
# zstd's long-distance window, 2^29 = 512 MB: the corpora repeat each other's chunks and reused vectors byte for byte,
# so the window, not the level, sets the size (2026-10-19: 766 MB at the default window, 216 MB at 2^29, 206 MB at
# 2^30 for twice the memory). Reading passes --long=31, which allows any window up to 2 GB.
ZSTD_WINDOW_LOG = 29


def die(msg, code=1):
    print(f'snapshot: {msg}', file=sys.stderr)
    sys.exit(code)


def human(n):
    for unit in ('B', 'KB', 'MB', 'GB'):
        if n < 1024 or unit == 'GB':
            return f'{n:.0f} {unit}' if unit == 'B' else f'{n:.1f} {unit}'
        n /= 1024


def utc_now():
    """The current UTC time in RFC 3339, to the second: 2026-10-19T02:50:41Z."""
    return datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace('+00:00', 'Z')


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for block in iter(lambda: f.read(BLOCK), b''):
            h.update(block)
    return h.hexdigest()


# ---------------------------------------------------------------- choosing the files

def is_scratch(name):
    if name.startswith('.'):
        return name not in KEEP_DOTFILES
    return name.endswith(SCRATCH_SUFFIXES)


def walk_files(top, prefix):
    """(member name, path) of every regular non-scratch file under top, sorted, member names under prefix."""
    if os.path.isfile(top):
        return [(prefix, top)]
    out = []
    for d, dirs, files in os.walk(top):
        dirs.sort()
        for name in sorted(files):
            path = os.path.join(d, name)
            if is_scratch(name) or not os.path.isfile(path) or os.path.islink(path):
                continue
            out.append((prefix + '/' + os.path.relpath(path, top), path))
    return out


def choose_files(include_models):
    """The files to pack as (member name, path), plus the top-level entries in neither list."""
    present = sorted(os.listdir(ARTIFACTS))
    unknown = [e for e in present if e not in PACK and e not in LEAVE and not is_scratch(e)]
    files = []
    for entry in sorted(PACK):
        path = os.path.join(ARTIFACTS, entry)
        if os.path.exists(path):
            files += walk_files(path, 'artifacts/' + entry)
    files = [(m, p) for m, p in files if m[len('artifacts/'):] not in SKIP_FILES]
    if include_models:
        for d in MODEL_DIRS:
            path = os.path.join(ROOT, 'models', d)
            if not os.path.isdir(path):
                die(f'--include-models: {path} is missing (scripts/fetch-model.sh fetches it)')
            files += walk_files(path, 'models/' + d)
    return files, unknown


# ---------------------------------------------------------------- the manifest

def git_state():
    def git(*args):
        r = subprocess.run(['git', '-C', ROOT] + list(args), capture_output=True, text=True)
        return r.stdout.strip() if r.returncode == 0 else None
    commit = git('rev-parse', 'HEAD')
    status = git('status', '--porcelain', '--', '.')
    return {'commit': commit, 'dirty': bool(status) if status is not None else None}


def read_json(path):
    try:
        with open(path) as f:
            return json.load(f)
    except (OSError, ValueError):
        return None


def corpus_dirs(entries):
    """Member names of the corpus directories (those holding chunks.jsonl): 'artifacts' for P0, 'artifacts/<dir>'."""
    dirs = []
    for member in entries:
        parts = member.split('/')
        if parts[0] == 'artifacts' and parts[-1] == 'chunks.jsonl' and len(parts) <= 3:
            dirs.append('/'.join(parts[:-1]))
    return sorted(dirs)


def corpus_record(member_dir, entries):
    """The chunks sha of one corpus and whether its BM25 index and vectors were built from that chunks.jsonl."""
    sha = entries[member_dir + '/chunks.jsonl']['sha256']
    rec = {'chunksSha16': sha[:16], 'chunksSha256': sha}
    path = os.path.join(ROOT, member_dir)
    bm25 = read_json(os.path.join(path, 'bm25', 'trevor-bm25.json'))
    rec['bm25Current'] = None if bm25 is None else bm25.get('chunksSha') == sha
    meta = read_json(os.path.join(path, 'vectors.meta.json'))
    if meta is not None:
        rec['vectors'] = {'modelFile': meta.get('modelFile'), 'modelSha': meta.get('modelSha'),
                          'tokenizerSha': meta.get('tokenizerSha'), 'count': meta.get('count'),
                          'current': meta.get('chunksSha') == sha}
    return rec


def model_records(corpora):
    """One entry per embedding model file the vectors were made with, and which corpora used it."""
    models = {}
    for name, rec in corpora.items():
        v = rec.get('vectors')
        if v:
            key = (v['modelFile'], v['modelSha'], v['tokenizerSha'])
            models.setdefault(key, []).append(name)
    return [{'modelFile': f, 'modelSha': s, 'tokenizerSha': t, 'corpora': c} for (f, s, t), c in sorted(models.items())]


def build_manifest(files, include_models):
    entries = {}
    for member, path in files:
        st = os.stat(path)
        entries[member] = {'path': member, 'size': st.st_size, 'sha256': sha256_file(path)}
    corpora = {d: corpus_record(d, entries) for d in corpus_dirs(entries)}
    for name, rec in corpora.items():  # an fp32 copy built from older chunks is a reuse source, not a drop-in
        base = corpora.get(name[:-len('-fp32')]) if name.endswith('-fp32') else None
        if base:
            rec['fp32CopyOf'] = name[:-len('-fp32')]
            rec['sameChunksAsBase'] = rec['chunksSha256'] == base['chunksSha256']
    return {
        'schema': SCHEMA,
        'created': utc_now(),
        'git': git_state(),
        'includesModels': include_models,
        'fileCount': len(entries),
        'totalBytes': sum(e['size'] for e in entries.values()),
        'corpora': corpora,
        'models': model_records(corpora),
        'files': [entries[m] for m, _ in files],
    }


# ---------------------------------------------------------------- pack

def size_table(files):
    rows = {}
    for member, path in files:
        parts = member.split('/')
        top = '/'.join(parts[:2])
        n, size = rows.get(top, (0, 0))
        rows[top] = (n + 1, size + os.path.getsize(path))
    print(f'{"entry":<34}{"files":>7}{"bytes":>16}{"size":>11}')
    for top, (n, size) in sorted(rows.items(), key=lambda kv: -kv[1][1]):
        print(f'{top:<34}{n:>7}{size:>16,}{human(size):>11}')
    total = sum(s for _, s in rows.values())
    print(f'{"total":<34}{len(files):>7}{total:>16,}{human(total):>11}')


class HashingReader:
    """Reads a file for tarfile while hashing it, so the bytes written can be checked against the manifest."""

    def __init__(self, f):
        self.f, self.h, self.n = f, hashlib.sha256(), 0

    def read(self, size=-1):
        b = self.f.read(size)
        self.h.update(b)
        self.n += len(b)
        return b


def open_writer(tmp):
    """A streaming tar writer into tmp, through `zstd -T1` when it is on PATH, else gzip; returns (tar, proc)."""
    if tmp.endswith('.zst'):
        cmd = ['zstd', '-q', '-T1', '-3', f'--long={ZSTD_WINDOW_LOG}', '-f', '-o', tmp]
        proc = subprocess.Popen(cmd, stdin=subprocess.PIPE)
        return tarfile.open(fileobj=proc.stdin, mode='w|', format=tarfile.PAX_FORMAT), proc
    return tarfile.open(tmp, mode='w:gz', compresslevel=6, format=tarfile.PAX_FORMAT), None


def add_bytes(tar, name, data):
    info = tarfile.TarInfo(name)
    info.size, info.mtime, info.mode = len(data), int(time.time()), 0o644
    tar.addfile(info, io.BytesIO(data))


def add_file(tar, member, path, expect):
    st = os.stat(path)
    info = tarfile.TarInfo(member)
    info.size, info.mtime, info.mode = expect['size'], int(st.st_mtime), st.st_mode & 0o777
    with open(path, 'rb') as f:
        reader = HashingReader(f)
        tar.addfile(info, reader)
    if st.st_size != expect['size'] or reader.n != expect['size'] or reader.h.hexdigest() != expect['sha256']:
        raise RuntimeError(f'{member} changed while packing (an update running?); nothing was written, rerun')


def cmd_pack(a):
    t0 = time.time()
    files, unknown = choose_files(a.include_models)
    for e in unknown:
        print(f'snapshot: WARNING: artifacts/{e} is in neither PACK nor LEAVE; left out (add it to one)',
              file=sys.stderr)
    size_table(files)
    manifest = build_manifest(files, a.include_models)
    ext = '.tar.zst' if shutil.which('zstd') else '.tar.gz'
    out = a.out or os.path.abspath(f'trevor-snapshot-{time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())}{ext}')
    if not out.endswith(ext):
        die(f'--out must end in {ext} (zstd {"is" if ext == ".tar.zst" else "is not"} on PATH)', 2)
    tmp = out[:-len(ext)] + '.tmp' + ext
    tar, proc = open_writer(tmp)
    try:
        add_bytes(tar, 'manifest.json', json.dumps(manifest, indent=1, ensure_ascii=False).encode() + b'\n')
        for (member, path), expect in zip(files, manifest['files']):
            add_file(tar, member, path, expect)
        tar.close()
        if proc is not None:
            proc.stdin.close()
            if proc.wait() != 0:
                raise RuntimeError(f'zstd exited {proc.returncode}')
    except (Exception, KeyboardInterrupt) as e:
        if proc is not None:
            proc.kill()
            proc.wait()
        if os.path.exists(tmp):
            os.remove(tmp)
        die(f'pack failed ({type(e).__name__}: {e}); removed {tmp}')
    os.replace(tmp, out)
    print_summary(manifest)
    print(f'wrote {out}: {os.path.getsize(out):,} bytes ({human(os.path.getsize(out))}) from '
          f'{manifest["totalBytes"]:,} bytes in {time.time() - t0:.1f} s')


# ---------------------------------------------------------------- read, verify, install

def open_reader(path):
    """A streaming tar reader over path (zstd -d for .zst, else gzip); returns (tar, proc)."""
    if path.endswith('.zst'):
        if not shutil.which('zstd'):
            die('zstd is not on PATH; it is needed to read a .tar.zst snapshot')
        proc = subprocess.Popen(['zstd', '-q', '-d', '--long=31', '-c', path], stdout=subprocess.PIPE)
        return tarfile.open(fileobj=proc.stdout, mode='r|'), proc
    return tarfile.open(path, mode='r|gz'), None


def close_reader(tar, proc):
    tar.close()
    if proc is not None:
        proc.stdout.close()
        if proc.wait() != 0:
            die(f'zstd exited {proc.returncode} while reading the snapshot')


def rest(tar):
    """The members after the ones already read (iterating the TarFile itself would yield manifest.json again)."""
    while True:
        info = tar.next()
        if info is None:
            return
        yield info


def safe_member(name):
    parts = name.split('/')
    return (not name.startswith('/') and '..' not in parts and '' not in parts
            and parts[0] in ('artifacts', 'models') and len(parts) >= 2)


def copy_member(tar, info, sinks):
    """Stream one member's bytes into every sink (file objects); returns (size, sha256)."""
    h, n = hashlib.sha256(), 0
    f = tar.extractfile(info)
    for block in iter(lambda: f.read(BLOCK), b''):
        h.update(block)
        n += len(block)
        for s in sinks:
            s.write(block)
    return n, h.hexdigest()


def read_manifest(tar):
    first = tar.next()
    if first is None or first.name != 'manifest.json':
        die('the first member is not manifest.json: not a Trevor snapshot')
    manifest = json.loads(tar.extractfile(first).read())
    if manifest.get('schema') != SCHEMA:
        die(f'manifest schema {manifest.get("schema")} is not {SCHEMA}; use the snapshot.py that packed it')
    return manifest


def verify(path):
    """Every member against the manifest, writing nothing. Returns the manifest; exits 1 listing every problem."""
    tar, proc = open_reader(path)
    manifest = read_manifest(tar)
    want = {e['path']: e for e in manifest['files']}
    seen, errors = set(), []
    for info in rest(tar):
        name = info.name
        if not info.isfile() or not safe_member(name):
            errors.append(f'{name}: not a regular file under artifacts/ or models/')
        elif name in seen:
            errors.append(f'{name}: appears twice')
        elif name not in want:
            errors.append(f'{name}: not in the manifest')
        else:
            seen.add(name)
            size, sha = copy_member(tar, info, [])
            if size != want[name]['size'] or sha != want[name]['sha256']:
                errors.append(f'{name}: size {size} sha256 {sha[:16]}, manifest {want[name]["size"]} '
                              f'{want[name]["sha256"][:16]}')
    close_reader(tar, proc)
    errors += [f'{m}: in the manifest, missing from the archive' for m in sorted(set(want) - seen)]
    if errors:
        for e in errors[:50]:
            print(f'FAIL {e}', file=sys.stderr)
        die(f'{len(errors)} problem(s); nothing written')
    print(f'verified {len(seen)} files, {manifest["totalBytes"]:,} bytes, every sha256 matches the manifest')
    return manifest


def print_summary(manifest):
    git = manifest.get('git') or {}
    print(f'snapshot created {manifest["created"]}, commit {git.get("commit")}'
          f'{" (trevor/ had uncommitted changes)" if git.get("dirty") else ""}, {manifest["fileCount"]} files, '
          f'{manifest["totalBytes"]:,} bytes, models {"included" if manifest["includesModels"] else "not included"}')
    print(f'{"corpus":<24}{"chunks sha16":<18}{"bm25":<10}vectors')
    for name, rec in manifest['corpora'].items():
        bm25 = {True: 'current', False: 'STALE', None: 'none'}[rec.get('bm25Current')]
        v = rec.get('vectors')
        vec = 'none'
        if v:
            vec = f'{v["modelFile"]} x {v["count"]}, {"current" if v["current"] else "STALE (older chunks)"}'
        behind = '' if rec.get('sameChunksAsBase', True) else f'; built from older chunks than {rec["fp32CopyOf"]}'
        print(f'{name:<24}{rec["chunksSha16"]:<18}{bm25:<10}{vec}{behind}')
    for m in manifest['models']:
        print(f'embedding model {m["modelFile"]} sha {m["modelSha"][:16]} tokenizer {m["tokenizerSha"][:16]}: '
              f'{", ".join(m["corpora"])}')


def model_conflicts(manifest, models_dir, force):
    """Packed model files that exist with other content; refused before anything is written unless --force."""
    bad = []
    for e in manifest['files']:
        if e['path'].startswith('models/'):
            dest = os.path.join(models_dir, e['path'][len('models/'):])
            if os.path.exists(dest) and (os.path.getsize(dest) != e['size'] or sha256_file(dest) != e['sha256']):
                bad.append(dest)
    if bad and not force:
        die('these model files exist with other content (--force replaces them): ' + ', '.join(bad))


def write_model(tar, info, dest, expect):
    if os.path.exists(dest) and os.path.getsize(dest) == expect['size'] and sha256_file(dest) == expect['sha256']:
        copy_member(tar, info, [])
        return False
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    tmp = dest + '.tmp'
    with open(tmp, 'wb') as f:
        size, sha = copy_member(tar, info, [f])
    if size != expect['size'] or sha != expect['sha256']:
        os.remove(tmp)
        die(f'{info.name} changed between verification and install (the snapshot file was replaced?)')
    os.chmod(tmp, info.mode)
    os.replace(tmp, dest)
    return True


def extract(path, manifest, tmp_dir, models_dir):
    """Second pass: write every artifacts member under tmp_dir and every model file into models_dir."""
    want = {e['path']: e for e in manifest['files']}
    tar, proc = open_reader(path)
    read_manifest(tar)
    models_written = 0
    for info in rest(tar):
        expect = want.get(info.name)
        if expect is None:
            die(f'{info.name} appeared after verification (the snapshot file was replaced?); {tmp_dir} is left')
        rel = info.name.split('/', 1)[1]
        if info.name.startswith('models/'):
            models_written += write_model(tar, info, os.path.join(models_dir, rel), expect)
            continue
        dest = os.path.join(tmp_dir, rel)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        with open(dest, 'wb') as f:
            size, sha = copy_member(tar, info, [f])
        if size != expect['size'] or sha != expect['sha256']:
            die(f'{info.name} changed between verification and install (the snapshot file was replaced?); '
                f'{tmp_dir} is left for inspection')
        os.chmod(dest, info.mode)
        os.utime(dest, (info.mtime, info.mtime))
    close_reader(tar, proc)
    return models_written


def cmd_install(a):
    t0 = time.time()
    into = os.path.abspath(a.into or ARTIFACTS)
    tmp_dir = into + '.tmp'
    models_dir = os.path.join(os.path.dirname(into), 'models')
    if os.path.islink(into):
        die(f'{into} is a symlink (the VPS release link); install into a plain directory and adopt it with '
            '`trevor-vps-update.sh init`')
    if os.path.exists(into) and not a.force:
        die(f'{into} exists; --force installs over it (the old tree is kept as {os.path.basename(into)}.old-<stamp>)')
    if os.path.exists(tmp_dir):
        die(f'{tmp_dir} exists (an interrupted install?); remove it and rerun')
    manifest = verify(a.file)
    model_conflicts(manifest, models_dir, a.force)
    os.makedirs(os.path.dirname(into), exist_ok=True)
    free = shutil.disk_usage(os.path.dirname(into)).free
    if free < manifest['totalBytes'] * 1.1:
        die(f'{human(free)} free beside {into}, the snapshot needs {human(manifest["totalBytes"])}')
    os.makedirs(tmp_dir)
    models_written = extract(a.file, manifest, tmp_dir, models_dir)
    record = {k: v for k, v in manifest.items() if k != 'files'}
    record.update(installedAt=utc_now(), installedFrom=os.path.abspath(a.file))
    with open(os.path.join(tmp_dir, 'snapshot.json'), 'w') as f:
        json.dump(record, f, indent=1, ensure_ascii=False)
        f.write('\n')
    if os.path.exists(into):
        old = f'{into}.old-{time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())}'
        os.rename(into, old)
        print(f'the previous tree is kept at {old}')
    os.rename(tmp_dir, into)
    print_summary(manifest)
    if manifest['includesModels']:
        print(f'models: {models_written} file(s) written into {models_dir}, the rest already identical')
    print(f'installed into {into} in {time.time() - t0:.1f} s; chunks shas installed for: '
          + ', '.join(f'{n} {r["chunksSha16"]}' for n, r in manifest['corpora'].items()))


def cmd_check(a):
    print_summary(verify(a.file))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('pack', help='write a snapshot of the derived artifacts')
    p.add_argument('--out', help='the snapshot file (.tar.zst with zstd on PATH, else .tar.gz)')
    p.add_argument('--include-models', action='store_true', help='also pack the embedder and the reranker')
    p.set_defaults(fn=cmd_pack)
    p = sub.add_parser('install', help='verify a snapshot, then install it')
    p.add_argument('file')
    p.add_argument('--into', help='the artifacts directory to create (default trevor/artifacts)')
    p.add_argument('--force', action='store_true', help='replace an existing tree (kept as <dir>.old-<stamp>)')
    p.set_defaults(fn=cmd_install)
    p = sub.add_parser('check', help='verify a snapshot, writing nothing')
    p.add_argument('file')
    p.set_defaults(fn=cmd_check)
    a = ap.parse_args()
    if a.cmd == 'pack' and a.out:
        a.out = os.path.abspath(a.out)
    try:
        a.fn(a)
    except (tarfile.TarError, EOFError) as e:
        die(f'the snapshot is unreadable ({type(e).__name__}: {e}): truncated or corrupt')


if __name__ == '__main__':
    main()
