//! DPS + HPS operator implementations.

mod dispatch;
mod dispatch_hps;
mod generated;
mod generated_hps;
pub mod hand;

use super::HpsResult;
use super::operator_unit::{EnemyStats, OperatorUnit};

#[must_use]
pub fn dispatch(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match hand::dps(op_id) {
        Some(formula) => formula(unit, enemy),
        None => dispatch::dispatch(unit, enemy),
    }
}

#[must_use]
pub fn skill_duration_override(unit: &OperatorUnit) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    if hand::dps(op_id).is_some() {
        return hand::skill_duration(op_id).and_then(|duration| duration(unit));
    }
    dispatch::dispatch_skill_duration(unit)
}

#[must_use]
pub fn skill_duration_override_hps(unit: &OperatorUnit) -> Option<f64> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    if hand::hps(op_id).is_some() {
        return hand::skill_duration(op_id).and_then(|duration| duration(unit));
    }
    dispatch_hps::dispatch_skill_duration(unit)
}

#[must_use]
pub fn dispatch_hps(unit: &OperatorUnit) -> Option<HpsResult> {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match hand::hps(op_id) {
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
