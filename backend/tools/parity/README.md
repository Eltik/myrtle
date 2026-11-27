# myrtle-parity

Checks that the rewrite answers like the legacy backend (git branch `main`).
The `parity` binary builds a corpus of GET requests from main's OpenAPI
document and the game data, records one backend's answers, and compares them
with a second backend or recording: JSON structurally (key order ignored,
floats at 1e-9 relative), everything else by SHA-256, status and content type
first. `tests/` also holds the engine dump tests against main. Every command
below runs from `backend/`.

## Build

```sh
cargo build -p myrtle-parity
```

The binary is `target/debug/parity`. Exit codes: 0 parity holds, 1 a
difference (or an error or a 429 during a run, or a superset violation), 2 the
tool itself failed.

## The legacy contract

`tests/fixtures/openapi.json` is main's OpenAPI document at `176e8864d`: 276
paths, 303 operations. 213 are GET: 149 public, 23 that take an optional
token, 41 that need one. The rest are 53 POST, 19 PUT, 16 DELETE and 2 PATCH,
which the harness never sends. `tests/corpus_pipeline.rs` pins these numbers,
so a refreshed document has to update them on purpose.

## Record, replay, diff

Ids for the path parameters come from the excel tables, then the corpus:

```sh
mkdir -p target/parity
./target/debug/parity ids --excel ../assets/output/en/gamedata/excel --out target/parity/ids.json
./target/debug/parity corpus --openapi tools/parity/tests/fixtures/openapi.json \
  --ids target/parity/ids.json --sample 50 --seed 1 --role user \
  --plan target/parity/plan.json --out target/parity/corpus.jsonl
```

`ids` prints a count per kind. Kinds no table supplies (player uids, tier-list
and grid slugs, siren ids, story sprites, lookback snapshots, ...) stay empty,
and `parity corpus` skips every route that needs one, with the reason. Put
values for them in a second ids file (`{"kinds": {"uid": ["..."]}}`) and pass
it with `--merge`. A `bindings` list in the same file
(`{"route": ..., "param": ..., "kind": ...}`) overrides which kind a parameter
draws from.

`--sample` caps each route at N parameter combinations per role; each route's
sample is seeded from `--seed` and its own template, so adding a route leaves
the others alone. The same inputs and seed give a byte-identical file.
`--role` (repeatable) emits each token GET once per role; without it the 41
token-only routes are skipped. `plan.json` lists, per route, the full product
size, the requests emitted and any skip reason. `--nginx-log FILE` (repeatable,
`-` for stdin) adds real 2xx GET paths from an access log, with credential-like
query parameters removed and any line carrying a JWT dropped.

Tokens come only from the environment, as `PARITY_TOKEN_<ROLE>` (role
upper-cased, other characters as `_`). For `diff`, `PARITY_A_TOKEN_<ROLE>` and
`PARITY_B_TOKEN_<ROLE>` override it per side. A token is sent only in the
`Authorization` header and never written to a corpus, recording or summary.
A role without a token is skipped and reported, not failed.

Record the legacy backend (main listens on 3060 by default), then replay the
recording against the rewrite, diff two live backends, or compare two
recordings offline:

```sh
./target/debug/parity record --base http://localhost:3060 \
  --corpus target/parity/corpus.jsonl --out target/parity/legacy
./target/debug/parity replay --base <rewrite base URL> \
  --corpus target/parity/corpus.jsonl --against target/parity/legacy \
  --summary target/parity/summary.json
./target/debug/parity diff --a http://localhost:3060 --b <rewrite base URL> \
  --corpus target/parity/corpus.jsonl
./target/debug/parity compare --a target/parity/legacy --b <second recording> \
  --corpus target/parity/corpus.jsonl
```

A recording is `manifest.json`, `responses.jsonl` and `bodies/<id>.json` per
JSON body; other bodies keep only their hash unless `record --keep-binary`.
Through nginx the rate limiter answers 429 and fails the run; cap the rate
with `--rps`.

`--unordered` and `--ignore` take JSON-pointer globs (`*` one segment, `**`
any number). `--rules FILE` sets them per route as well; command-line
tolerances replace the file's, command-line globs add to them:

```json
{
  "rel_tol": 1e-9,
  "unordered": ["/**/tags"],
  "routes": {
    "/api/health": { "ignore": ["/uptime_seconds"] },
    "/api/music/siren/songs/{cid}/stream": { "skip": true }
  }
}
```

`openapi-superset` fails when a new document drops an operation, a schema
name, a response status or content type, or changes a response's `$ref`
names. The rewrite does not emit a document yet; once it does:

```sh
./target/debug/parity openapi-superset --old tools/parity/tests/fixtures/openapi.json --new <rewrite openapi.json>
```

## Harness tests

```sh
cargo test -p myrtle-parity --lib --bin parity --test compare --test corpus --test glob --test http --test ids --test nginx --test openapi --test report --test rng --test rules --test run --test corpus_pipeline --test http_diff
```

## Engine dump tests

Each does nothing unless its output variable is set. Roots are
`../assets/output` (holding `en/`, `cn/`, `jp/`, `kr/`) or one server under it.

| Test | Command |
|------|---------|
| `archives_dump` | `ARCHIVES_DUMP=<dir> ARCHIVES_ROOT=../assets/output cargo test -p myrtle-parity --test archives_dump` |
| `base_dump` | `BASE_DIFF=<players.json> BASE_OUT=<file> GAMEDATA_ROOT=../assets/output/en cargo test -p myrtle-parity --test base_dump` (`BASE_ROTATIONS=<n>` optional) |
| `dps_ext_dump` | `DPS_EXT_DUMP=<file> DPS_EXT_SCENARIOS=<scenarios.json> cargo test -p myrtle-parity --test dps_ext_dump` |
| `gamedata_dump` | `GAMEDATA_DUMP=<dir> GAMEDATA_ROOT=../assets/output/en cargo test -p myrtle-parity --test gamedata_dump` (`GAMEDATA_SERVER=cn` for CN) |
| `grade_dump` | `GRADE_DIFF=<players.json> GRADE_OUT=<file> GAMEDATA_ROOT=../assets/output/en cargo test -p myrtle-parity --test grade_dump` |
| `hps_dump` | `HPS_DUMP=<file> cargo test -p myrtle-parity --test hps_dump` (`HPS_DUMP_ALL=1` for every row) |
| `music_dump` | `MUSIC_DUMP=<dir> MUSIC_ROOT=../assets/output cargo test -p myrtle-parity --test music_dump` (`MUSIC_OWN_ART=1` optional) |
| `parity_dump` | `DPS_DUMP=<file> cargo test -p myrtle-parity --test parity_dump` (`DPS_DUMP_ALL=1` for every row) |
| `planner_dump` | `PLANNER_DUMP=<dir> PLANNER_ROOT=../assets/output cargo test -p myrtle-parity --test planner_dump` |
| `release_dump` | `RELEASE_DUMP=<dir> RELEASE_ROOT=../assets/output cargo test -p myrtle-parity --test release_dump` |
| `squad_dump` | `SQUAD_DUMP=<dir> GAMEDATA_ROOT=../assets/output/en cargo test -p myrtle-parity --test squad_dump` (`SQUAD_CN_ROOT=../assets/output/cn` adds CN) |
| `story_dump` | `STORY_DUMP=<dir> STORY_ROOT=../assets/output cargo test -p myrtle-parity --test story_dump` |

`hps_dump` and `parity_dump` compare against `tests/fixtures/expected_hps.json`
and `expected_dps.json`. They and `dps_ext_dump` load EN game data through
`myrtle-testkit` (`MYRTLE_TEST_ASSETS`, else the repo's `assets/output`).

Two suites assert main's numbers directly instead of dumping:

```sh
cargo test -p myrtle-parity --test dps_hand_main -- --ignored
GAMEDATA_ROOT=../assets/output/en cargo test -p myrtle-parity --test grade_operators_main
```

All 12 `dps_hand_main` tests are ignored by default: 11 need EN game data,
one needs CN (through `myrtle-testkit`, as above). `grade_operators_main`
skips without `GAMEDATA_ROOT`.
