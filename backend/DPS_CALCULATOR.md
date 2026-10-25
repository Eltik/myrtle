# DPS Calculator Setup

The engine model (buff zones, primary/secondary targets, cycle v2, batch/rank/supporters
endpoints, retention score) and every measured result are in
[`docs/dps-refactor.md`](../docs/dps-refactor.md). Read it before changing `src/dps/`.

## Two formula layers

| Layer | Files | Written by |
|---|---|---|
| Generated | `src/dps/custom/generated.rs`, `dispatch.rs`, `generated_hps.rs`, `dispatch_hps.rs`, `src/dps/config/operator_formulas.json`, `heal_formulas.json` | `generate-dps`, overwritten on every run |
| Hand | `src/dps/custom/hand/` (one file per operator, `mod.rs` registry, `formulas.json` metadata) | people; the generator refuses to write there |

`custom/mod.rs` dispatches to the hand layer FIRST and falls back to the generated table;
`engine::load_formulas` / `load_heal_formulas` merge `hand/formulas.json` over the
generated JSON the same way. Never edit a generated file by hand: the next regeneration
deletes the edit (that is how Angelina the Mellow Wish was lost before the hand layer).
`custom/mod.rs` is hand-written too. There is no hand-written `__init__` table: the
generator transpiles each class's `__init__` state into its function (see below).

### What the generator reproduces, and the gate that keeps it honest

- `__init__` state. Every statement after `super().__init__` that writes `self.<field>`
  is kept with its full if / elif / else guard chain and any `__init__` locals it reads,
  and prepended to the transpiled body. Fields the unit carries (`drone_atk`, `atk`,
  `attack_speed`, `ammo`, `skill_duration`, `target_hp`, `shadows`, the five
  conditionals) become shadow locals; any other field the formula reads becomes an
  `st_<field>` local. `except:` branches count (the engine passes no keyword
  arguments, so the reference always takes them); `try:` bodies do not.
- Local / `self.` name splits. `self.atk_interval` (and a hoisted shadow) transpiles to
  a plain Rust local; when the body also assigns a Python local of that name, the local
  is renamed `_<name>_w` so `self.<name>` keeps the instance value.
- `cargo run -j 3 --bin generate-dps -- --audit` prints both detectors' full output.
  `cargo test -j 3 --bin generate-dps` FAILS on any state write the formula reads that is
  neither reproduced nor listed in `INIT_STATE_ELSEWHERE` (with its reason), on a name
  collision it cannot split, and on a stale `INIT_STATE_ELSEWHERE` entry. Run it after
  every pull.
- Skill duration. When a class's `__init__` sets `self.skill_duration` (Kafka S2, Skadi
  S2, Viviana S3, ...), the generator also emits `<fn>_skill_duration` and a
  `dispatch_skill_duration` arm. The engine applies it to `unit.skill_duration` before
  the formula runs, so the formula, `total_damage`, `average_dps` and cycle v2 share one
  duration. Hand-layer operators use the game-data duration.
- Loops and helpers: `for x in range(n)` / `range(a, b)` (nested), `math.comb` (exact,
  emitted as `py_comb` when used), `sum(list)`.

### Reference corrections (when the Python is wrong against game data)

`src/dps/custom/hand/reference_corrections.json` declares class-scoped textual fixes to
the upstream source, each with a `reason` and the game-data `evidence`. The generator
applies them before it parses, transpiles AND runs the reference for the expected
fixtures, so the engine and `expected_dps.json` both carry the corrected value; the
parity test reads the same file to report each correction and fails on one that matches
no case. A correction whose text no longer occurs upstream makes the generator (and
`cargo test --bin generate-dps`) panic, so an upstream change to that code is reviewed,
not silently dropped. Prefer this over a hand formula when the fix is a few tokens: the
rest of the class keeps tracking upstream.

## Prerequisites

- Game data unpacked at `../assets/output/` (or set `GAME_DATA_DIR` and `ASSETS_DIR` in `.env`)
- ArknightsDpsCompare at `external/ArknightsDpsCompare`, a git SUBMODULE (or pass `--repo <path>`)
- For `--expected` / `--healing`: Python 3.12 with the reference's imports. The repo's own
  `pyproject.toml` pulls manim; the expected step only needs this:

```bash
uv venv /tmp/adc-py --python 3.12
uv pip install --python /tmp/adc-py/bin/python numpy dill discord matplotlib nltk pillow requests
export PYTHON_BIN=/tmp/adc-py/bin/python
```

## Pull upstream and regenerate

```bash
git -C external/ArknightsDpsCompare fetch origin
git -C external/ArknightsDpsCompare checkout --detach origin/main   # moves the submodule pin

# Formulas, transpile and expected fixtures for DPS and HPS (seconds, debug build)
cargo run -j 3 --bin generate-dps -- --formulas --transpile --expected --healing
rustfmt --edition 2024 src/dps/custom/{generated,generated_hps,dispatch,dispatch_hps}.rs

# Conditional polarity follows the formula tables (exits 1 when stale)
python3 scripts/dps_conditional_polarity.py && python3 scripts/dps_conditional_polarity.py --check
```

The JSON tables are written with sorted keys, so `git diff` shows only real changes. The
generator prints `HAND OVERRIDE: <id>` for every operator that is in both layers: the hand
formula still wins, so decide whether to delete the hand file now that upstream covers it.

Before trusting a regeneration, dump the engine before and after and `cmp` the dumps (the
house rule is bit-identical for everything upstream did not touch):

```bash
DPS_DUMP=/tmp/dps.tsv cargo test -j 3 --test dps_engine_test -- --nocapture
HPS_DUMP=/tmp/hps.tsv cargo test -j 3 --test hps_engine_test -- --nocapture
# every formula operator, every skill and module, 3 enemies (covers hand operators too)
DPS_TABLE_DUMP=/tmp/table.tsv cargo test -j 3 --test dps_table_dump_test -- --ignored
```

The parity dumps hold only cases present in `tests/fixtures/expected_*.json`, so compare
them on the shared keys when the fixture grew.

## Test

```bash
cargo test -j 3 --test dps_engine_test -- --nocapture   # Python parity, 15% bar
cargo test -j 3 --test hps_engine_test -- --nocapture
cargo test -j 3 --test dps_hand_test -- --ignored        # hand-layer snapshots (CN data)
```

## Add an operator by hand

Use this when upstream has no class for the operator yet, or its transpiled formula is
wrong and the fix cannot go into the transpiler.

1. Create `src/dps/custom/hand/<operator>.rs` with `pub fn dps(unit: &OperatorUnit, enemy:
   &EnemyStats) -> Option<f64>` (and / or `pub fn hps(...) -> Option<HpsResult>`). Read
   the same `OperatorUnit` fields a generated function reads: `atk`, `buff_atk`,
   `buff_atk_flat`, `attack_speed`, `attack_interval`, `targets`, `skill_index`,
   `skill_parameters` (raw blackboard order), `talent1_parameters`, the `*_damage`
   conditionals. Apply external buffs the way generated functions do
   (`atk * (1 + skill_buff + unit.buff_atk) + unit.buff_atk_flat`); the engine applies
   fragile, zones, secondary targets and the cycle model around the call. Document the
   blackboard indices you read at the top of the file.
2. Register it in `hand/mod.rs`: `mod <operator>;` and a row in `DPS` / `HPS`.
3. Add its metadata to `hand/formulas.json` (`dps` / `hps` section, same shape as an
   `operator_formulas.json` entry: `available_skills`, `available_modules`,
   `default_skill`, `conditionals`, `skills`). `engine::tests::hand_registry_matches_hand_metadata`
   fails if the registry and the metadata disagree.
4. If it has conditionals, give each a polarity in `HAND` in
   `scripts/dps_conditional_polarity.py` and rerun the script.
5. Test it. There is no Python reference, so the parity suite cannot see it. Add a
   snapshot test to `tests/dps_hand_test.rs`: pin the engine's exact output for a few
   cases (`assert_eq!` on the full `f64`), AND derive the same cases by hand from the
   game's own text (skill and talent descriptions, base stats) and assert they agree to
   1e-6 relative. Write the derivation and the text it came from in the test comment, with
   the date checked, so a reviewer can redo it. The Angelina the Mellow Wish test is the
   template.
6. Run `cargo test -j 3 --test dps_formula_audit_test -- --ignored --nocapture`: the
   operator should leave the missing list.

## Missing-formula audit

```bash
cargo test -j 3 --test dps_formula_audit_test -- --ignored --nocapture
```

Lists every playable operator (a `char_` id, profession not TOKEN or TRAP, not
`IsNotObtainable`, loaded through the backend's game-data loader, which merges the two
Amiya forms from `char_patch_table`) with no DPS and no HPS formula, grouped by rarity and
class, and every medic with no HPS formula. Exemptions (operator id + reason, reviewed)
live in `tests/fixtures/dps_formula_exemptions.json`; it starts empty, and the test fails
on an exemption that names a non-playable operator or one that now has a formula.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `GAME_DATA_DIR` | `../assets/output/gamedata/excel` | Path to game data JSON files |
| `ASSETS_DIR` | `../assets/output` | Path to unpacked assets root |
| `PYTHON_BIN` | first of python3.13 .. python3 with 3.10+ | Python for `--expected` |

## Output Files

| File | Generated by |
|---|---|
| `src/dps/config/operator_formulas.json` | `--formulas` |
| `src/dps/custom/generated.rs`, `dispatch.rs` | `--transpile` |
| `tests/fixtures/expected_dps.json` | `--expected` |
| `src/dps/config/heal_formulas.json`, `custom/generated_hps.rs`, `dispatch_hps.rs`, `tests/fixtures/expected_hps.json` | `--healing` |
| `src/dps/config/conditional_polarity.json` | `scripts/dps_conditional_polarity.py` |

**Note**: Ebenholz fails, but that is a Python bug. Rust is correct.

## DPS Watcher (report-only)

`src/core/dps_watcher.rs` polls the upstream repository and LOGS a new commit. It never
fetches, regenerates, rebuilds or restarts, so it cannot touch either formula layer; a
pull is the reviewed procedure above, done in a normal change.

| Variable | Default | Description |
|---|---|---|
| `DPS_POLL_INTERVAL` | _(unset = disabled)_ | Seconds between GitHub checks |
| `DPS_UPSTREAM_REPO` | `WhoAteMyCQQkie/ArknightsDpsCompare` | GitHub repo to watch |
| `DPS_UPSTREAM_BRANCH` | `main` | Branch to track |
| `DPS_STATE_FILE` | `.dps-updater-state.json` | Persisted state file |
| `GITHUB_TOKEN` | _(optional)_ | For higher rate limits (not required) |
