//! Hand-written DPS / HPS formulas, consulted before the generated dispatch.
//!
//! One file per operator, one row in a registry below, one entry in `formulas.json`.
//! A hand id that upstream now covers fails `tables::tests::bundled_formula_tables_parse`
//! until it is deleted or listed in `REVIEWED_HAND_OVERRIDES`.

use super::super::HpsResult;
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
/// Skill duration in seconds; `None` keeps the game data's.
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

/// Hand HPS formulas by operator id, called with a zero-DEF / zero-RES enemy.
pub const HPS: &[(&str, HpsFn)] = &[("char_1052_kalts2", kaltsit_esperanta::hps)];

/// Durations the game data gets wrong (permanent skills stored as timed, ammo and
/// stone skills paced by their rate). The engine sets `unit.skill_duration` from
/// this before the formula runs; one table serves both engines.
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
