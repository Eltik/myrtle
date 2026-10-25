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
  duration. A hand-layer operator uses the game-data duration unless its file exports
  a `skill_duration` registered in `hand::SKILL_DURATION` (see "Add an operator by
  hand"); the engine applies that one through the same path.
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
cargo test -j 3 --test dps_hand_test -- --include-ignored  # hand-layer snapshots (EN + CN data)
```

## Add an operator by hand

Use this when upstream has no class for the operator yet, or its transpiled formula is
wrong and the fix cannot go into the transpiler.

### The formula file

Create `src/dps/custom/hand/<operator>.rs` with `pub fn dps(unit: &OperatorUnit, enemy:
&EnemyStats) -> Option<f64>` (and / or `pub fn hps(...) -> Option<HpsResult>`). Read the
same `OperatorUnit` fields a generated function reads: `atk`, `buff_atk`, `buff_atk_flat`,
`attack_speed`, `attack_interval`, `targets`, `skill_index`, `skill_parameters` (raw
blackboard order), `talent1_parameters`, `talent2_parameters`, the `*_damage`
conditionals. Apply external buffs the way generated functions do (`atk * (1 +
skill_buff + unit.buff_atk) + unit.buff_atk_flat`); the engine applies fragile, zones,
secondary targets and the cycle model around the call. The file's header comment names
the game-data key behind every blackboard index and constant it reads, and lists the
modelling assumptions.

Read numbers from game data, not from the description text. Two shared helpers in
`hand/util.rs` cover the module blackboards `OperatorUnit` does not unpack:

- `module_trait_value(unit, key)`: a key from the equipped module's trait override
  (`OverrideTraitDataBundle`) at its current level (Bellone's and Ch'en's trait ASPD,
  Zima's `atk_scale_e`, Mantra's `damage_scale`).
- `module_talent_value(unit, prefab_key, key)`: a key from one of the module's talent
  parts (`AddOrOverrideTalentDataBundle`), picked by `PrefabKey` at the unit's
  potential. `OperatorData` files talent parts under talent 1 or 2 by index or name, so
  an unnamed hidden part (`TalentIndex` -1, no `Name`) never reaches
  `talent1_parameters`; this reads it from the unit's module data (Viy's `10_root`
  ASPD). A named hidden part that `OperatorData` does file is in
  `unit.data.talent1_module_extra` / `talent2_module_extra` (Astgenne's).

A helper two hand files need goes into `util.rs`, not into a second private copy, and
must not change a result bit for the files that move onto it.

### The skill duration

The engine reads `unit.skill_duration` (game data, seconds; -1 for an unlimited or an
ammo skill) for `total_damage`, the cycle averages and the formula itself. When the
real duration is not the game data's, export it:

```rust
/// `None` keeps the game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64>
```

and add a row to `hand::SKILL_DURATION`. For an operator the hand layer owns (DPS
engine: in `DPS`; HPS engine: in `HPS`) the engine calls it INSTEAD of the generated
`dispatch_skill_duration`, after the zones and before the formula, so the formula,
`total_damage`, `average_dps` and cycle v2 share the value (and an HPS formula reads
the same `unit.skill_duration`). One function serves both engines. Current users:

- Zima the Raging Tide S2 from the second activation: unlimited
  (`util::UNLIMITED_DURATION`, -1; the engine then reports `total_damage` and
  `average_dps` equal to `skill_dps`).
- Viy S3 with stored charges: 27 ammo, one charge per attack interval.
- Kal'tsit Esperanta S2: 10 ammo, one per attack.
- Wang S3: 20 ammo, 3 per manual stone placement, one placement per stone redeploy
  time (2 s at E2).

A skill whose formula returns the skill's TOTAL damage ("totalDMG", Degenbrecher's
convention: Zima S3, an instant skill) has no duration and needs no entry.

### Registry and metadata

1. Register it in `hand/mod.rs`: `mod <operator>;` (rustfmt keeps the list sorted) and a
   row in `DPS` / `HPS` (and `SKILL_DURATION`).
2. Add its metadata to `hand/formulas.json` (`dps` / `hps` section, same shape as an
   `operator_formulas.json` entry: `available_skills`, `available_modules`,
   `default_skill`, `conditionals`, `skills`). `engine::tests::hand_registry_matches_hand_metadata`
   fails if the registry and the metadata disagree, or a `SKILL_DURATION` row names an
   operator the hand layer does not own. `engine::tests::bundled_formula_tables_parse`
   fails if a hand id also has a generated formula, until the overlap is reviewed (delete
   the hand file, or list the id in `REVIEWED_HAND_OVERRIDES`). No test pins the table
   sizes: they follow from the generated tables and the hand registry.
3. Conditionals: a hand operator has no Python `__init__` to derive a polarity from.
   Give each DPS conditional an entry in `HAND` in `scripts/dps_conditional_polarity.py`,
   `"<id>|<type>|<name>": <inverted>` (`False`: the label describes the flag ON; `True`:
   it describes the flag OFF, as Mantra's `vsBoss`), with a comment saying what the flag
   does in the hand file. `HAND` is read for the `dps` section only; an HPS conditional
   stays unresolved. Rerun `python3 scripts/dps_conditional_polarity.py`, then `--check`.
   `dps_phase2_test::conditional_polarity_table` fails while
   `config/conditional_polarity.json` and the merged tables disagree.

### Tests

There is no Python reference, so the parity suite cannot see a hand operator.
`tests/dps_hand_test.rs` holds its snapshot:

- Layout: the shared helpers sit at the top (`en()` loads EN game data, `params(skill,
  module, targets)`, `with(conditionals, params)`, `dps(...)` returning `(skill_dps,
  total_damage, average_dps)`, `hps(...)` returning `(skill_hps, base_hps, avg_hps)`,
  `assert_hand_check`). Each batch of operators gets one `mod` named after the id
  suffixes (`mod demetr_headb2_chen3_wang`), importing what it uses from `super`, with
  one `#[ignore = "needs EN game data"]` test `<operator>_snapshot`. Add a helper to the
  top, not to a module.
- Pin the engine's exact output for a few cases (`assert_eq!` on the full `f64` triple),
  AND derive the same cases by hand from the game's own text (skill and talent
  descriptions, base stats) and assert they agree to 1e-6 relative with
  `assert_hand_check`. Write the derivation and the text it came from in the test's doc
  comment, with the date checked, so a reviewer can redo it. Cover the duration too when
  the file exports one (`total_damage` = skill DPS x the derived duration).

### Checklist

- [ ] `hand/<operator>.rs`: `dps` / `hps`, header comment with every game-data key read
      and every assumption; module blackboards through `util.rs`.
- [ ] `skill_duration` + `SKILL_DURATION` row, if the real duration is not the game
      data's.
- [ ] `hand/mod.rs` rows, `hand/formulas.json` entry.
- [ ] Polarity: `HAND` entries, then `python3 scripts/dps_conditional_polarity.py &&
      python3 scripts/dps_conditional_polarity.py --check`.
- [ ] Snapshot test in `tests/dps_hand_test.rs`; `cargo test -j 3 --test dps_hand_test
      -- --include-ignored`.
- [ ] `cargo test -j 3 --lib dps::engine::tests`, `cargo test -j 3 --test
      dps_phase1_test`, `--test dps_phase2_test`.
- [ ] `cargo test -j 3 --test dps_formula_audit_test -- --ignored --nocapture`: the
      operator leaves the missing list.
- [ ] `cargo clippy -j 3 --all-targets` and `cargo fmt --check`.

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
