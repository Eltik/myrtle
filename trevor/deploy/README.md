# Installing Trevor on the VPS

Step by step, for the operator of the VPS (Ubuntu 24.04, 3 vCPU AMD Ryzen 9 7900, 11 GB). Nothing here has been run
yet. The why behind each step, the failure table and rollback are in `../design/trevor-vps-runbook.md`.

Paths assumed below (change them in every unit and in `/etc/trevor/trevor.env` together if the box differs):

| what | where |
|---|---|
| repo (branch with Trevor) | `/var/www/myrtle.moe`, so the code is `/var/www/myrtle.moe/trevor` |
| asset watcher tree | `/var/www/myrtle.moe/assets` (EN version stamp: `assets/ArkAssets/en/.version`) |
| Trevor's state | `/var/lib/trevor` (`releases/`, `current` symlink, `stage/`, `state/`, `logs/`, `spool/`) |
| secrets | `/etc/trevor/trevor.env`, root-owned, mode 600 |
| model lock | `/run/trevor/llm.lock` |
| ports | backend 3060 (exists), llama-server 8081 (serving), 8082 (overnight summaries only) |

## 0. Read-only checks first

None of these change anything. Send the output back before going further.

```bash
lscpu | grep -E 'Model name|^CPU\(s\)|Flags' | cut -c1-200
grep -c avx512f /proc/cpuinfo           # > 0: AVX-512 present (ZC expects it on Zen 4)
free -m; df -h / /var
stat -fc %T /sys/fs/cgroup              # cgroup2fs = cgroup v2 (AllowedCPUs needs it)
cat /sys/fs/cgroup/cgroup.controllers   # must list cpuset and cpu
systemd --version | head -1             # >= 244 for AllowedCPUs
python3 --version; rsync --version | head -1; flock --version; curl --version | head -1
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3060/      # the backend answers
cat /var/www/myrtle.moe/assets/ArkAssets/en/.version                # the EN res_version the watcher holds
systemctl list-timers --all | head -20  # what else runs daily (pick quiet hours; the host backup window is unknown)
```

## 1. User and directories

```bash
sudo useradd --system --home-dir /var/lib/trevor --shell /usr/sbin/nologin trevor
sudo install -d -o trevor -g trevor -m 755 /var/lib/trevor /var/lib/trevor/state /var/lib/trevor/spool
sudo install -d -o root -g root -m 700 /etc/trevor
sudo cp /var/www/myrtle.moe/trevor/deploy/systemd/trevor.tmpfiles.conf /etc/tmpfiles.d/trevor.conf
sudo systemd-tmpfiles --create /etc/tmpfiles.d/trevor.conf     # creates /run/trevor
```

## 2. llama.cpp

Use a pinned llama.cpp release built for Ubuntu x64 (the release archive loads the best CPU variant, AVX-512
included, at run time), or build it from source with `cmake -B build -DGGML_NATIVE=ON && cmake --build build -j 2`
under `nice -n 19`. Either way, record the tag and the sha256 of what you installed in the runbook. The flags the
units use (`--no-repack`, `--poll`, `--cache-ram`, `--no-webui`) need a 2026 build; the zero-cost benchmark used one
built 2026-10-15.

```bash
sudo install -m 755 build/bin/llama-server build/bin/llama-bench /usr/local/bin/
llama-server --version
```

## 3. Gemma 4 E2B and the gte embedder

```bash
sudo install -d -o trevor -g trevor /var/www/myrtle.moe/trevor/models/llm
# Gemma 4 E2B instruct, Q4_K_M GGUF (about 3 GB). Pin the repo revision and check the sha256 after download.
sha256sum /var/www/myrtle.moe/trevor/models/llm/gemma-4-E2B-it-Q4_K_M.gguf
# gte-modernbert-base: tokenizer.json is needed by every build step; model.onnx (fp32) only when EMBED=1.
# Copy models/gte-modernbert-base from the build machine (scripts/fetch-model.sh pins and checks it).
```

## 4. Benchmark the CPU rung (one minute, read-only)

```bash
taskset -c 2 nice -n 19 llama-bench -m /var/www/myrtle.moe/trevor/models/llm/gemma-4-E2B-it-Q4_K_M.gguf \
  -t 1,2 -p 512 -n 128 -r 3 --no-repack 1 2>&1 | tee /tmp/e2b-bench.txt
```

`-t 2` under `taskset -c 2` shares one core between two threads; run `taskset -c 1-2` for the `-t 2` row if you want
the two-core number. Compare with the zero-cost benchmark (2.1 GHz Xeon, Q4_K_M, `--no-repack`): prompt 49.5 / 94.6
tok/s and generation 4.7 / 8.5 tok/s at 1 / 2 threads. A slim answer is about 2,000 prompt tokens and 250 generated,
so at 1 thread: 2,000 / pp + 250 / tg seconds. If one thread gives under 4 tok/s generation, consider
`AllowedCPUs=1-2` and `-t 2` in `trevor-llama.service`.

## 5. Trevor binaries and code

The VPS never builds Rust. The Release Bundle workflow (`.github/workflows/release.yml`, every push to main) builds
`cargo build --release --locked --features embed --bins` on Ubuntu 24.04 x86_64 and packs all 12 bins into the
commit's bundle; install them with:

```bash
cd /var/www/myrtle.moe
bash scripts/fetch-release.sh --components trevor <full sha>   # TREVOR_DIR=... if Trevor lives elsewhere
```

That puts `ask build-archives build-corpus build-index build-prefixes build-profiles build-units embed-check
embed-corpus eval goldset search` in `/var/www/myrtle.moe/trevor/target/release/` (the units' path), swapped by rename,
sha256-checked, and runs `ask --help` on the box before and after the swap. Exit 3 means no bundle for that commit
(it is not on main, or CI has not finished: `--wait 1500`); a bundle published before Trevor joined the workflow has
no `trevor/` and exits 1. ONNX Runtime 1.28 is linked into the bins statically, so nothing else ships with them;
they need the glibc and libstdc++ of Ubuntu 24.04. `trevor` is never in the default component set, so a plain
`fetch-release.sh <sha>` for the site leaves these files alone. The repo checkout supplies `scripts/ prompts/ data/
eval/ deploy/`.

## 6. Artifacts from the build machine

Everything Trevor serves or rebuilds from comes from the build machine's `trevor/artifacts` (the development
machine where `scripts/update.sh` runs). These directories are
development-only and can stay behind (about 0.4 GB): `answers/ p1a-split/ p3b-lore/ p4-lore/ goldgen-arch/
goldgen-v2/ embedft/ update/ router/`. Keep `goldgen/` (`goldgen/names.json` is read by `ask` and appearances.py).

```bash
# on the build machine
rsync -a --exclude answers/ --exclude p1a-split/ --exclude p3b-lore/ --exclude p4-lore/ --exclude goldgen-arch/ \
  --exclude goldgen-v2/ --exclude embedft/ --exclude update/ --exclude router/ --exclude panels/ --exclude publish/ \
  trevor/artifacts/ vps:/var/www/myrtle.moe/trevor/artifacts/
# on the VPS: adopt them as the first release (artifacts becomes a symlink to /var/lib/trevor/current)
sudo env TREVOR_HOME=/var/lib/trevor /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh init
sudo chown -R trevor:trevor /var/lib/trevor
```

## 7. Secrets

```bash
sudo cp /var/www/myrtle.moe/trevor/deploy/trevor.env.example /etc/trevor/trevor.env
sudo chmod 600 /etc/trevor/trevor.env
sudoedit /etc/trevor/trevor.env      # TREVOR_WORKER_KEY (same value as the backend's), TREVOR_CF_*, webhook
```

## 8. Units

```bash
cd /var/www/myrtle.moe/trevor/deploy/systemd
sudo cp trevor-llama.service trevor-worker.service trevor-worker-restart.path trevor-worker-restart.service \
  trevor-update.service trevor-update.timer trevor-summaries.service trevor-summaries.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now trevor-llama.service
curl -s http://127.0.0.1:8081/health                  # {"status":"ok"} after the model loads
sudo systemctl enable --now trevor-worker-restart.path trevor-worker.service
sudo systemctl enable --now trevor-update.timer trevor-summaries.timer
systemctl list-timers 'trevor-*'
```

## 9. Embeddings: pick one

- **BM25-only (start here).** `TREVOR_RETRIEVAL=bm25` and `EMBED=0` in the environment file. No vectors are made on
  the VPS and the build machine's INT8 vectors are never read.
- **Hybrid.** Vectors made with the fp32 export (`model.onnx`), the same on every CPU. Either copy fp32 vectors made
  on the build machine, or re-embed once on the VPS (about 1 to 2 h at nice 19):

  ```bash
  sudo systemd-run --unit=trevor-reembed --uid=trevor --gid=trevor -p EnvironmentFile=/etc/trevor/trevor.env \
    -p Nice=19 -p IOSchedulingClass=idle -p MemoryMax=2G --setenv=EMBED=1 --setenv=EMBED_FULL=1 --setenv=FORCE=1 \
    /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh daily
  journalctl -fu trevor-reembed
  ```

  Then `EMBED=1`, `TREVOR_RETRIEVAL=hybrid`. The daily job refuses to add fp32 vectors to a directory whose vectors
  came from another export, so the two runtimes never mix.

## 10. First publish and checks

```bash
sudo systemctl start trevor-update.service    # no version is active yet, so it builds and publishes now
journalctl -u trevor-update -n 100 --no-pager
sudo -u trevor env TREVOR_HOME=/var/lib/trevor /var/www/myrtle.moe/trevor/scripts/trevor-vps-update.sh status
curl -s http://127.0.0.1:3060/api/trevor/status
systemctl status trevor-llama trevor-worker --no-pager
```

Before the first backend deploy that carries migration v039: an off-box `pg_dump` and a tested restore (plan v3,
phase D step 4).
