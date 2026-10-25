//! DPS + HPS operator implementations.
//!
//! Two layers. `generated*` holds the transpiled per-operator bodies and
//! `dispatch*` the `match` tables routing a `char_id` to them; all four come from
//! `cargo run --bin generate-dps` and are overwritten on every regeneration.
//! `hand/` holds hand-written formulas the generator never touches. The two
//! `dispatch` functions below consult `hand` FIRST, then the generated table.
//! This `mod.rs` and `hand/` are hand-written. `__init__` state (drone ATK by module
//! level, flag overrides, Wiš'adel's shadows) is transpiled into each generated
//! function by `generate-dps`; there is no hand-written init table any more.

mod dispatch;
mod dispatch_hps;
mod generated;
mod generated_hps;
pub mod hand;

use super::engine::HpsResult;
use super::operator_unit::{EnemyStats, OperatorUnit};

/// The operator's DPS formula: the hand layer's if it has one, else the
/// generated one. `None` when neither layer knows the operator.
pub fn dispatch(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match hand::dps(op_id) {
        Some(formula) => formula(unit, enemy),
        None => dispatch::dispatch(unit, enemy),
    }
}

/// The skill duration the operator's Python `__init__` sets (Kafka S2, Skadi S2, ...),
/// generated per class by `generate-dps`. `None` when the class sets none, or when
/// the hand layer owns the operator (a hand formula reads the game-data duration).
pub fn skill_duration_override(unit: &OperatorUnit) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    if hand::dps(op_id).is_some() {
        return None;
    }
    dispatch::dispatch_skill_duration(unit)
}

/// [`skill_duration_override`] for the HPS table.
pub fn skill_duration_override_hps(unit: &OperatorUnit) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    if hand::hps(op_id).is_some() {
        return None;
    }
    dispatch_hps::dispatch_skill_duration(unit)
}

/// The operator's HPS formula, hand layer first, like [`dispatch`].
pub fn dispatch_hps(unit: &OperatorUnit) -> Option<HpsResult> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match hand::hps(op_id) {
        // The generated HPS dispatch passes a zero target; so does the hand layer.
        Some(formula) => formula(
            unit,
            &EnemyStats {
                defense: 0.0,
                res: 0.0,
            },
        ),
        None => dispatch_hps::dispatch(unit),
    }
}
