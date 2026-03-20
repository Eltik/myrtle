use super::operator_unit::{EnemyStats, OperatorUnit};

pub fn dispatch(unit: &OperatorUnit, enemy: &EnemyStats) -> f64 {
    let _op_id = unit.data.data.id.as_deref().unwrap_or("");
    unit.normal_attack(enemy, None, None, None)
}
