# Installing Trevor on the VPS

The why behind each step, the failure table and rollback are in `../design/trevor-vps-runbook.md`.

## The one command: `bootstrap.sh`

A fresh Ubuntu 24.04 x86_64 server becomes a serving Trevor instance with no model vendor (the cache and `cpu` rungs,
passages as the floor; `../design/trevor-self-contained-deploy.md`) from the clone and one command, run as root:

```bash
git clone https://github.com/Eltik/myrtle.git /var/www/myrtle.moe && cd /var/www/myrtle.moe && git checkout <sha on main>
sudo trevor/deploy/bootstrap.sh --dry-run --snapshot https://.../trevor-snapshot-<stamp>.tar.zst   # the plan, nothing changed
sudo trevor/deploy/bootstrap.sh --snapshot https://.../trevor-snapshot-<stamp>.tar.zst --snapshot-sha256 <hex>
```

It runs ten steps in order, each a function that checks its own done-state, so a rerun after a failure continues where
it stopped; it stops at the first error and names the step. `--only STEP` and `--skip STEP` (comma lists, repeatable)
pick steps; `--dry-run` prints every command and changes nothing (the read-only checks still run).

| step | does | done when |
|---|---|---|
| `preflight` | Ubuntu 24.04 x86_64 (other Linux warns, other arch stops); installs missing apt packages (curl, tar, zstd, python3, rsync, util-linux, openssl, git); memory: stops under 8 GB, skips the 12B under 16 GB, warns under 32 GB (design section 5); warns under 8 cores; disk: stops under 15 GB free, warns under 40 GB; the `trevor` user, `/var/lib/trevor/{state,spool,logs,snapshot}`, `/etc/trevor` (700), tmpfiles for `/run/trevor` | always runs (idempotent) |
| `release` | `scripts/fetch-release.sh --components trevor,assets <sha>` (plus `backend` when `backend/target/release/backend` is missing). `--sha` defaults to the checked-out HEAD. The bundle exists only for commits built on main by the Release Bundle workflow (the newest 20 are kept): exit 3 stops with that message, and nothing is ever compiled on the box | `target/release/.release-installed` names the sha |
| `llama` | llama.cpp `b11146` (`pins.env`: tag, asset, sha256), the build every CPU number in the design doc was measured with; downloaded, sha256-checked, unpacked to `/opt/llama.cpp/b11146`, `current` symlink, `/usr/local/bin/llama-server` and `llama-bench` | `.verified` in the tag dir and `llama-server --version` runs |
| `models` | `WITH_FP32=1 WITH_E2B=1 scripts/fetch-model.sh trevor/models`, plus `WITH_LLM=1` (Gemma 4 12B and Qwen3.5 9B, 12.7 GB) at 16 GB of memory or more (`WITH_LLM=0/1` overrides); links the E2B file under the upper-case name the units use | a marker records fetch-model.sh's sha and the flags |
| `snapshot` | `scripts/snapshot.py install <file> --into trevor/artifacts` from `--snapshot URL|FILE` (`--snapshot-sha256` checks the file first). Refuses to go on without one; `--no-snapshot` starts from an empty tree, and the first update then builds every table from nothing, which is weeks on a CPU | `artifacts/snapshot.json`, or `artifacts` is already the release link |
| `env` | `/etc/trevor/trevor.env` from `trevor.env.example`: the paths, `TREVOR_RETRIEVAL=bm25` and `EMBED=0` (step 9 below for hybrid), a fresh `TREVOR_WORKER_KEY` (kept on reruns), `--webhook` as `DISCORD_ALERT_WEBHOOK`, every `TREVOR_CF_*` commented out (cf is not used here); the same key in `backend/.env`. Without `backend/.env` it stops, unless `--database-url` is given: then it creates one from `backend/.env.example` with fresh `JWT_SECRET`, `SERVICE_KEY` and `GAME_CREDENTIAL_KEY` | both files hold the same key |
| `database` | the backend applies its migrations on startup (`backend/README.md`, Database; there is no separate command), so this restarts it under pm2 (`pm2 restart myrtle-backend --update-env`, or `pm2 start backend/ecosystem.config.cjs`), waits up to 15 minutes for `/api/health` and checks that the worker key is accepted | the worker key is accepted (HTTP 200 or 404 on `worker/popular`) |
| `units` | installs `trevor-llama`, `trevor-worker`, `trevor-worker-restart.{path,service}`, `trevor-update.{service,timer}` (paths rewritten when the clone is not at `/var/www/myrtle.moe`), daemon-reload, enables and starts llama (waits for `/health`), the worker and the update timer. Not `trevor-summaries.timer`: it is the Cloudflare summaries path, unused in this deployment | unchanged unit files are left alone |
| `first-update` | `trevor-vps-update.sh init` (adopts `trevor/artifacts` as the first release), `chown -R trevor /var/lib/trevor`, then `daily` once in the foreground with `FORCE=1` under `systemd-run` as `trevor`; the log is `/var/lib/trevor/logs/<stamp>-daily/run.log` | `state/active.json` exists |
| `verify` | `/api/trevor/status` until a worker is online with the `cpu` rung (3 minutes; warns if `cf` shows), `ask --route-only` of one question, a one-screen summary | always runs |

Prerequisites the bootstrap does not create: Postgres reachable at the backend's `DATABASE_URL`, node and pm2 for the
backend, and the asset watcher's EN extract (`assets/ecosystem.config.cjs`; the backend needs the game data to start and
the first update needs `assets/ArkAssets/en/.version`). Tests on a scratch tree: `DESTDIR=/tmp/x
trevor/deploy/bootstrap.sh --only env --database-url postgresql://...` writes both env files under `/tmp/x`.

Unverified until a fresh Ubuntu box runs it: the dry run and the env step were run on the development machine
(2026-10-19), nothing else.

## What bootstrap does, by hand

The sections below are the same work as hand steps: read them to understand a step, or to repair one the bootstrap
stopped at. The VPS of the original plan: Ubuntu 24.04, 3 vCPU AMD Ryzen 9 7900, 11 GB.

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

The bootstrap installs the release pinned in `pins.env` (b11146, `llama-b11146-bin-ubuntu-x64.tar.gz`, sha256-checked)
into `/opt/llama.cpp/b11146` and links `/usr/local/bin/llama-server` to it. By hand: use a pinned llama.cpp release built for Ubuntu x64 (the release archive loads the best CPU variant, AVX-512
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
sha256sum /var/www/myrtle.moe/trevor/models/llm/gemma-4-e2b-it-q4_k_m.gguf
# gte-modernbert-base: tokenizer.json is needed by every build step; model.onnx (fp32) only when EMBED=1.
# Copy models/gte-modernbert-base from the build machine (scripts/fetch-model.sh pins and checks it).
```

## 4. Benchmark the CPU rung (one minute, read-only)

```bash
taskset -c 2 nice -n 19 llama-bench -m /var/www/myrtle.moe/trevor/models/llm/gemma-4-e2b-it-q4_k_m.gguf \
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
sudo systemctl enable --now trevor-update.timer trevor-summaries.timer   # bootstrap leaves the summaries timer off (Cloudflare path)
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
