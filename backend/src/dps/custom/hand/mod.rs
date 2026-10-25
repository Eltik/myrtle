//! Hand-written DPS / HPS formulas, consulted BEFORE the generated dispatch.
//!
//! The generated layer (`generated*.rs`, `dispatch*.rs`, `config/operator_formulas.json`,
//! `config/heal_formulas.json`) is rewritten wholesale by `generate-dps` from the
//! upstream Python repo, which updates rarely. An operator it does not cover, or
//! covers wrongly, lives here instead:
//!
//! - one file per operator, exposing `dps` and / or `hps` with the generated
//!   signature (`fn(&OperatorUnit, &EnemyStats) -> Option<f64>` /
//!   `Option<HpsResult>`), so the engine runs it through the same path: zones,
//!   secondary targets, cycle v2 and conditionals apply unchanged;
//! - one row in `DPS` / `HPS` below (the registry), and one in `SKILL_DURATION` when a
//!   skill's real duration is not the game data's;
//! - shared helpers (module blackboard lookups, the unlimited-duration value) in
//!   `util.rs`;
//! - one entry in `formulas.json` (`dps` / `hps` section): the same metadata shape as
//!   `operator_formulas.json`, merged over the generated table by
//!   `engine::load_formulas` / `load_heal_formulas`.
//!
//! An id registered here wins over a generated formula for the same id. The
//! generator never writes under `custom/hand/` (it refuses to, see
//! `generate_dps.rs`), and it prints every id that is in both layers so a hand
//! formula that upstream now covers gets reviewed rather than silently shadowing
//! the transpiled one. How to add an operator: `backend/DPS_CALCULATOR.md`.

use super::super::engine::HpsResult;
use super::super::operator_unit::{EnemyStats, OperatorUnit};

mod angelina_alter;
mod astgenne_lightchaser;
mod bellone;
mod chen_dawnstreak;
mod closure;
mod kaltsit_esperanta;
mod mantra;
mod nasti;
mod silverash_alter;
mod util;
mod viy;
mod wang;
mod zima_alter;

pub type DpsFn = fn(&OperatorUnit, &EnemyStats) -> Option<f64>;
pub type HpsFn = fn(&OperatorUnit, &EnemyStats) -> Option<HpsResult>;
/// The operator's real skill duration in seconds when it differs from the game data's
/// (`None`: keep the game-data duration). See [`SKILL_DURATION`].
pub type DurationFn = fn(&OperatorUnit) -> Option<f64>;

/// Metadata for the hand formulas, `{"dps": {id: OperatorFormula}, "hps": {...}}`.
pub const FORMULAS_JSON: &str = include_str!("formulas.json");

/// Hand DPS formulas by operator id.
pub const DPS: &[(&str, DpsFn)] = &[
    ("char_1015_aglna2", angelina_alter::dps),
    ("char_4037_demetr", bellone::dps),
    ("char_1051_headb2", zima_alter::dps),
    ("char_1050_chen3", chen_dawnstreak::dps),
    ("char_2027_wang", wang::dps),
    ("char_1047_halo2", astgenne_lightchaser::dps),
    ("char_4212_nasti", nasti::dps),
    ("char_1052_kalts2", kaltsit_esperanta::dps),
    ("char_4226_veen", viy::dps),
    ("char_4204_mantra", mantra::dps),
    ("char_1045_svash2", silverash_alter::dps),
    ("char_4228_closur", closure::dps),
];

/// Hand HPS formulas by operator id. Called with a zero-DEF / zero-RES enemy, the
/// way the generated HPS dispatch calls its functions.
pub const HPS: &[(&str, HpsFn)] = &[("char_1052_kalts2", kaltsit_esperanta::hps)];

/// Hand skill-duration functions by operator id, the hand layer's
/// `dispatch_skill_duration`. For an operator the hand layer owns (in `DPS` for the
/// DPS engine, in `HPS` for the HPS engine) the engine consults this INSTEAD of the
/// generated table and sets `unit.skill_duration` to the result after the zones and
/// before the formula, so the formula, `total_damage`, `average_dps` and cycle v2 all
/// read one duration. One table serves both engines: a duration belongs to the skill.
/// Use it when the game-data duration is not the real one: a permanent skill whose
/// game data says otherwise (`util::UNLIMITED_DURATION`), an ammo skill whose length
/// follows from its attack rate, a stone skill whose length follows from its placement
/// rate (Wang S3). Only Zima S3 still returns a TOTAL as its skill DPS (an instant
/// skill with no duration, Degenbrecher's "totalDMG" convention).
pub const SKILL_DURATION: &[(&str, DurationFn)] = &[
    ("char_1051_headb2", zima_alter::skill_duration),
    ("char_1052_kalts2", kaltsit_esperanta::skill_duration),
    ("char_4226_veen", viy::skill_duration),
    ("char_2027_wang", wang::skill_duration),
];

pub fn dps(op_id: &str) -> Option<DpsFn> {
    DPS.iter().find(|(id, _)| *id == op_id).map(|(_, f)| *f)
}

pub fn hps(op_id: &str) -> Option<HpsFn> {
    HPS.iter().find(|(id, _)| *id == op_id).map(|(_, f)| *f)
}

pub fn skill_duration(op_id: &str) -> Option<DurationFn> {
    SKILL_DURATION
        .iter()
        .find(|(id, _)| *id == op_id)
        .map(|(_, f)| *f)
}
