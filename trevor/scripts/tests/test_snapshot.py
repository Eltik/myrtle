#!/usr/bin/env python3
"""scripts/snapshot.py install against crafted tarballs (design/trevor-self-contained-deploy.md step 3; 2026-10-19).

Install runs as root under deploy/bootstrap.sh, so a tampered snapshot must not plant a setuid or world-writable file
or a link. Each test writes a small tar.gz whose first member is a valid manifest, then runs `snapshot.py check` or
`install --into` a scratch directory: member modes are masked (0o755 at most, 0o644 when not executable, no setuid,
setgid or sticky bit), directory members are accepted, and a symlink, hard link, device or fifo member fails the
verification by name with nothing written. Nothing outside the scratch directory is read or written.

  python3 -m unittest scripts/tests/test_snapshot.py      (from trevor/; stdlib only, about 1 s)
"""
import hashlib, io, json, os, shutil, stat, subprocess, sys, tarfile, tempfile, unittest

SCRIPTS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SNAPSHOT = os.path.join(SCRIPTS, 'snapshot.py')


def manifest(files):
    return {'schema': 1, 'created': '2026-10-19T00:00:00Z', 'git': {'commit': 'test', 'dirty': False},
            'includesModels': False, 'fileCount': len(files), 'totalBytes': sum(len(d) for _, d, _ in files),
            'corpora': {}, 'models': [],
            'files': [{'path': n, 'size': len(d), 'sha256': hashlib.sha256(d).hexdigest()} for n, d, _ in files]}


class Install(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp(prefix='trevor-snapshot-')
        self.into = os.path.join(self.d, 'artifacts')

    def tearDown(self):
        shutil.rmtree(self.d)

    def tarball(self, files, extra=(), dirs=()):
        """files: (name, bytes, mode) in the manifest; extra: TarInfo members outside it; dirs: directory members."""
        path = os.path.join(self.d, 'snap.tar.gz')
        with tarfile.open(path, 'w:gz', format=tarfile.PAX_FORMAT) as tar:
            m = json.dumps(manifest(files)).encode()
            info = tarfile.TarInfo('manifest.json'); info.size = len(m)
            tar.addfile(info, io.BytesIO(m))
            for name in dirs:
                info = tarfile.TarInfo(name); info.type = tarfile.DIRTYPE; info.mode = 0o7777
                tar.addfile(info)
            for name, data, mode in files:
                info = tarfile.TarInfo(name); info.size, info.mode, info.mtime = len(data), mode, 1790000000
                tar.addfile(info, io.BytesIO(data))
            for info in extra:
                tar.addfile(info)
        return path

    def run_snapshot(self, *args):
        p = subprocess.run([sys.executable, SNAPSHOT, *args], capture_output=True, text=True, timeout=60,
                           env={**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'})
        return p.returncode, p.stdout + p.stderr

    def mode(self, rel):
        return stat.S_IMODE(os.stat(os.path.join(self.into, rel)).st_mode)

    def test_modes_are_masked(self):
        files = [('artifacts/x/suid.bin', b'a\n', 0o4777), ('artifacts/x/world.jsonl', b'b\n', 0o666),
                 ('artifacts/x/sgid.sh', b'c\n', 0o2775), ('artifacts/x/sticky.txt', b'd\n', 0o1640),
                 ('artifacts/x/private.txt', b'e\n', 0o600), ('artifacts/x/plain.txt', b'f\n', 0o644)]
        rc, out = self.run_snapshot('install', self.tarball(files, dirs=['artifacts', 'artifacts/x']), '--into', self.into)
        self.assertEqual(rc, 0, out)
        got = {n.split('/')[-1]: self.mode(n.split('/', 1)[1]) for n, _, _ in files}
        self.assertEqual(got, {'suid.bin': 0o755, 'world.jsonl': 0o644, 'sgid.sh': 0o755, 'sticky.txt': 0o644,
                               'private.txt': 0o644, 'plain.txt': 0o644})
        for n, data, _ in files:
            with open(os.path.join(self.d, n), 'rb') as f:
                self.assertEqual(f.read(), data)

    def refused(self, info, word):
        path = self.tarball([('artifacts/x/a.txt', b'a\n', 0o644)], extra=[info])
        rc, out = self.run_snapshot('check', path)
        self.assertEqual(rc, 1, out)
        self.assertIn(info.name, out)
        self.assertIn(word, out)
        rc, out = self.run_snapshot('install', path, '--into', self.into)
        self.assertEqual(rc, 1, out)
        self.assertIn(word, out)
        self.assertEqual(sorted(os.listdir(self.d)), ['snap.tar.gz'], 'install wrote something')

    def test_symlink_refused(self):
        info = tarfile.TarInfo('artifacts/x/passwd'); info.type = tarfile.SYMTYPE; info.linkname = '/etc/passwd'
        self.refused(info, 'symlink')

    def test_hard_link_refused(self):
        info = tarfile.TarInfo('artifacts/x/hard'); info.type = tarfile.LNKTYPE; info.linkname = 'artifacts/x/a.txt'
        self.refused(info, 'hard link')

    def test_device_refused(self):
        info = tarfile.TarInfo('artifacts/x/null'); info.type = tarfile.CHRTYPE; info.devmajor, info.devminor = 1, 3
        self.refused(info, 'character device')

    def test_fifo_refused(self):
        info = tarfile.TarInfo('artifacts/x/pipe'); info.type = tarfile.FIFOTYPE
        self.refused(info, 'fifo')

    def test_directory_outside_tree_refused(self):
        info = tarfile.TarInfo('etc/cron.d'); info.type = tarfile.DIRTYPE
        path = self.tarball([('artifacts/x/a.txt', b'a\n', 0o644)], extra=[info])
        rc, out = self.run_snapshot('check', path)
        self.assertEqual(rc, 1, out)
        self.assertIn('etc/cron.d', out)


if __name__ == '__main__':
    unittest.main()
