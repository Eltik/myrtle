# Re-embedding Trevor's corpora with the fp32 export, safely

2026-10-16. Owner of this doc: the embed step (`src/corpus/embed.rs`, `src/bin/embed-corpus.rs`,
`src/bin/embed-check.rs`, `scripts/fetch-model.sh`). Ian's ask: "figure out a safe way to re-embed without causing any
hardware failures", so that the VPS can add vectors for new story chunks daily while the Mac stays a development
machine.

## 1. Why every vector moves to fp32

The INT8 export's vectors depend on the ONNX Runtime build and the CPU, not just on the text. Measured:

| comparison (same text, gte-modernbert-base) | n | min | median | mean |
|---|---|---|---|---|
| Mac fp32 (Rust, ORT 1.28) vs stored Mac INT8 (Rust, ORT 1.28), P3b sample | 1,000 | 0.992290 | 0.996808 | 0.996679 |
| Mac INT8 (Python, ORT 1.19.2, arm64) vs stored Mac INT8 (Rust, ORT 1.28) | 100 | 0.907510 | 0.946003 | 0.945908 |
| Rosetta x86_64 INT8 (Python, ORT 1.19.2) vs stored Mac INT8 | 100 | 0.906126 | 0.945915 | 0.945318 |
| Xeon AVX-512 VNNI INT8 vs fp32 (trevor-p0-status.md, ORT 1.25 and 1.28) | 40 | 0.9174 | | 0.9410 |
| Mac fp32 (Python, ORT 1.19.2, arm64) vs stored Mac fp32 (Rust, ORT 1.28) | 100 | 0.999999 | 1.000000 | 1.000000 |
| Rosetta x86_64 fp32 (Python, ORT 1.19.2) vs stored Mac fp32 (Rust, ORT 1.28) | 100 | 0.999999 | 1.000000 | 1.000000 |

The second row is new and corrects what we believed: "the INT8 export is accurate on Apple Silicon" is true only of the
ORT 1.25/1.28 build the Rust crate downloads. The same file on the same Mac under ORT 1.19.2 lands at 0.946 of the
stored vectors, the same drift the x86 Xeon showed. So INT8 vectors are tied to one runtime build on one CPU family, and
an ORT upgrade alone could silently move every query away from the corpus. The fp32 export agrees with itself to
0.999999 across two ORT versions (1.19.2 and 1.28) and two instruction sets (NEON and Rosetta's x86_64 AVX2
translation). The Python pipeline is the control: its fp32 vectors match the Rust ones at 0.999999, so the INT8 gap is
the runtime, not the harness. (Rosetta translates no AVX-512, so the VPS's Zen 4 path is not covered by the proxy: the
VPS check in section 5 is the real gate.)

fp32 costs disk and memory, not speed, on the Mac: `model.onnx` is 596,392,315 bytes (INT8: 150,218,016), the embed
process peaks at 939 MB to 1,417 MB RSS (INT8 run 650 MB per HANDOFF), and fp32 is only a little slower: 7.42 chunks/s
at 2 threads and 9.16 at 4 (first 500 P3b rows) against INT8's 8.47 at 2, 9.57 at 4 and 7.22 at ORT's default (first
200 rows), all under other agents' builds, so the sets and loads differ.

## 2. Flags

`embed-corpus` (every new behavior has an off value; all four off values together reproduce the old run, verified
below):

| flag | default | off value | what it does |
|---|---|---|---|
| `--onnx-file` / `--model-file` | `model_int8.onnx` | | the export; the swap uses `model.onnx`. Unchanged default until the serving swap is decided |
| `--threads N` | 2 | `0` = ORT's own default (the old run) | ONNX Runtime intra-op threads |
| `--no-guard` | guard on | `--no-guard` | before an embedding call, at most every `--guard-every` s (10), pause while a reason holds, re-probing every `--guard-pause` s (30), logging each reason. macOS: `pmset -g therm` shows a recorded thermal or performance warning (or a `*_Limit` below 100), `sysctl kern.memorystatus_vm_pressure_level` >= 2, or `pmset -g batt` says Battery Power (`--guard-allow-battery`). Linux: `MemAvailable` < `--guard-min-mem-gib` (1.5), `/proc/pressure/cpu` some avg10 > `--guard-max-cpu-pressure` (40), or the 1-minute load average > the machine's online CPUs (`/proc/stat`, not the affinity mask; `--guard-no-load`). A probe that cannot run is skipped, never a pause. The checkpoint is fsynced before each pause |
| `--duty D` | 0.6 | `1` | after an embedding call of `w` seconds, sleep `w (1 - D) / D`: inference takes at most `D` of wall time, bounding sustained heat |
| `--max-minutes M` | none | omit | stop cleanly after M minutes of wall time (pauses count), checkpoint synced, exit code 3; rerun the same command to resume |
| `--checkpoint-every N` | 100 | `0` (no checkpoint written or read, old meta removed first) | every embedded row is appended to `vectors.ckpt` (16-byte key, u32 dim, the f32 row) and flushed to the OS at once; fsynced every N rows. `vectors.ckpt.json` names the model and tokenizer; a checkpoint of another model is discarded. A rerun reads the valid prefix (a torn, zeroed, non-hex or non-unit record ends it, and the file is cut back there) and embeds only what is missing. With a checkpoint the old `vectors.meta.json` stays until the new `vectors.bin` is renamed in, so the old vectors keep serving and a resumed in-place run still reuses them. The checkpoint is read even under `--no-reuse` (its rows were embedded for this run) and deleted when the run completes |

None of these changes a vector. They decide when a row is embedded, never how: rows stay one text per call, so a
vector depends on its text and model alone.

`embed-check` (new, read-only): re-embeds a seeded sample (`--n 200 --seed 20261016`) or `--ids a,b,c` of a corpus on
this machine with `--onnx-file` (default `model.onnx`, threads 2) and prints min, p01, p05, median, mean and max cosine
against the stored vectors, the 5 worst chunk ids and the rate. Exit 1 (FAIL) when the minimum is below `--min-cos`
(0.999) or the fresh model's sha is not the vectors' `modelSha` (the comparison is still printed; that is how the
INT8-vs-fp32 row above was measured). `--out FILE` writes one `{chunkId, cos}` line per chunk.

Query side (`corpus::embed`): `vectors.meta.json` already records `modelFile` and `modelSha`.
`check_query_model(vectors_model_file, vectors_model_sha, &query_embedder, allow)` refuses a query embedder whose sha
differs; `TREVOR_ALLOW_QUERY_MODEL_MISMATCH=1` (or `allow`) turns the refusal into a warning, the old behavior.
`resolve_query_onnx("auto", modelFile)` picks the vectors' own file. The loader lives in `src/search/runtime.rs`,
which this change does not own, so the check is NOT wired yet; the three-line patch is in section 7.

`scripts/fetch-model.sh`: `WITH_FP32=1` fetches `onnx/model.onnx` from Alibaba-NLP/gte-modernbert-base at revision
e7f32e3c00f91d699e8c43b53106206bcc72bb22 (the INT8 file's repo and revision), sha256
947f31df7effaeec4edb57c50e4ed7e0f2034d9336063f92615b92e3e0d24d78, 596,392,315 bytes. Kept behind the flag (4x the
INT8 file); fetched and verified on the Mac 2026-10-16.

## 3. Verification

- **Kill switch.** `--threads 0 --no-guard --duty 1 --checkpoint-every 0` against the binary built before the change
  (INT8, 200 P3b chunks, `--no-reuse`): `vectors.bin` byte-identical, `vectors.meta.json` identical in every field and
  key order except `builtAtUnix` and `seconds`.
- **Threads do not move vectors.** INT8 at threads 0, 2 and 4: byte-identical (200 chunks), and byte-identical to the
  stored P3b rows. fp32 at threads 2 and 4: byte-identical (500 chunks). So reuse and resume stay exact across thread
  settings.
- **Resume.** A run killed by SIGTERM (exit 143) after 61 rows left 61 whole records (188,612 bytes, 0 torn); the rerun
  read "61 rows from an interrupted run", embedded 139, wrote `resumed: 61`, and its `vectors.bin` is byte-identical to
  an uninterrupted run. A `--max-minutes 0.25` run that spent its time paused stopped with exit 3 and a 0-row
  checkpoint; the rerun completed, byte-identical. Unit test `checkpoint_keeps_the_valid_prefix` covers the torn,
  zeroed, non-unit, non-hex and changed-dim tails.
- **Guard.** Fired for real: memory pressure level 2 (other agents compiling Rust) paused the full run and both resume
  tests for 30 to 90 s at a time and resumed on its own; no thermal or performance warning was recorded at any point
  (PENDING totals).
- `cargo clippy --release --all-targets --features embed`: 0 warnings. `cargo test --release --features embed --lib
  embed`: 3 passed.

## 4. Mac measurements

fp32, 500 P3b chunks (the first 500 rows), guard on, duty 1, nice 10, other agents compiling (load average 3.0 to 4.4):

| threads | embedding time | chunks/s | process CPU | RSS | battery temp (ioreg, C) | pmset therm |
|---|---|---|---|---|---|---|
| 2 | 67.4 s | 7.42 | 188 to 192 % | 583 to 939 MB | 30.76 -> 30.75 | nothing recorded |
| 4 | 54.6 s | 9.16 | 337 to 346 % | 852 to 1,417 MB | 30.75 -> 30.75 | nothing recorded |

4 threads buy 1.23x the rate for 1.8x the CPU, so 2 threads is the default. The only temperature the Mac publishes
without sudo is the battery's (`ioreg -rn AppleSmartBattery` "Temperature"), a chassis proxy, not the package; it did not
move at either setting, and `pmset -g therm` recorded nothing. With `--duty 0.6` the process averages about 115 % CPU.

PENDING: the full run.

## 5. Procedure

### Mac: full re-embed (once, then only when the model changes)

Check first: `memory_pressure | tail -1` (free >= 25 %), `pmset -g therm` (nothing recorded), on AC power. Then, into
copies, never over the serving directories:

```sh
cd trevor
WITH_FP32=1 ./scripts/fetch-model.sh
for c in p3b p4x p4-art; do mkdir -p artifacts/$c-fp32 && cp -R artifacts/$c/{bm25,chunks.jsonl,manifest.p0.json,spoiler.jsonl,unresolved.jsonl} artifacts/$c-fp32/; done
nice -n 10 target/release/embed-corpus --model-dir models/gte-modernbert-base --model-file model.onnx \
  --corpus artifacts/p3b-fp32 --threads 2 --duty 0.6
nice -n 10 target/release/embed-corpus --model-dir models/gte-modernbert-base --model-file model.onnx \
  --corpus artifacts/p4x-fp32 --reuse-from artifacts/p3b-fp32 --threads 2 --duty 0.6
nice -n 10 target/release/embed-corpus --model-dir models/gte-modernbert-base --model-file model.onnx \
  --corpus artifacts/p4-art-fp32 --reuse-from artifacts/p4x-fp32 --threads 2 --duty 0.6
```

Interrupt it any time (Ctrl-C, closing the lid, `kill PID`): the same command resumes. Add `--max-minutes 120` to cap a
session. Keys: P3b has 16,044 distinct texts, P4x adds 8,317, P4-art 848; P3a and P4 add 0 (their rows are all in P3b
or P4x), so their fp32 vectors cost seconds with `--reuse-from`.

### VPS: daily incremental

The VPS embeds only new or changed chunks, reusing the fp32 vectors copied from the Mac (same `modelSha`, so reuse
applies across machines). Before the first incremental run, gate the host (read-only, about a minute):

```sh
sudo systemd-run --uid="$(id -u)" --gid="$(id -g)" --working-directory="$PWD" -p Nice=19 -p CPUWeight=10 -p MemoryMax=2G -p AllowedCPUs=1 --pty --wait --collect target/release/embed-check --model-dir models/gte-modernbert-base --onnx-file model.onnx --corpus artifacts/p3b-fp32 --n 200 --threads 1
```

PASS (min cosine >= 0.999) means the VPS's fp32 vectors are interchangeable with the Mac's. Optional second number
for the record: the same with `--onnx-file model_int8.onnx --corpus artifacts/p3b --min-cos 0` measures this CPU's INT8
drift against the Mac's INT8 vectors.

The incremental embed, under the same limits (one CPU, so one thread):

```sh
sudo systemd-run --uid="$(id -u)" --gid="$(id -g)" --working-directory="$PWD" -p Nice=19 -p CPUWeight=10 -p MemoryMax=2G -p AllowedCPUs=1 -p IOWeight=10 --wait --collect target/release/embed-corpus --model-dir models/gte-modernbert-base --model-file model.onnx --corpus artifacts/p4x --reuse-from artifacts/p3b --threads 1 --duty 0.6 --max-minutes 120
```

`MemoryMax=2G` is above the fp32 process's measured peak (1,417 MB at 4 threads on the Mac; 1 thread needs less).
`AllowedCPUs=1` pins it to one of the three vCPUs; the guard's load test still compares the system load average with
all 3 online CPUs. A stopped or killed run (exit 3, OOM, deploy) resumes on the next timer from its checkpoint.

## 6. Quality

PENDING.

## 7. Not done here, and what the swap needs

- **Wire the query check** in `src/search/runtime.rs` (another agent's file). In `Runtime::load_sharing`, resolve the
  file and check after the embedder is built or shared:
  `let onnx = embed::resolve_query_onnx(&a.query_onnx, &d.meta.model_file).to_owned();`, use `onnx` where
  `a.query_onnx` is used (the load and `SharedEmbedder::fits`), then
  `embed::check_query_model(&d.meta.model_file, &d.meta.model_sha, &e.inner.lock()..., a.allow_query_model_mismatch)?`,
  with `--query-onnx` defaulting to `auto` and a new `--allow-query-model-mismatch` flag. With INT8 corpora, `auto`
  loads `model_int8.onnx`, the old default, so nothing changes until the swap.
- **Swap**: point ask at the `-fp32` directories (or move them into place) together with `--query-onnx model.onnx` (or
  `auto`), and make `scripts/update.sh` and `scripts/trevor-vps-update.sh` pass `--model-file model.onnx`. `trevor-vps-update.sh`
  passes no `--threads`, so it gets 2 threads on its one allowed CPU; `--threads 1` fits its units better.
- `update.sh` now runs embed-corpus with the new defaults: 2 threads, the guard (it pauses on battery), duty 0.6
  (about 1.7x the wall time of an incremental embed, seconds for a monthly event).
