//! Per-operator `__init__` state the generator cannot hoist.
//!
//! `generate-dps` transpiles each Python class's `skill_dps` body and hoists
//! single-line `self.<field> = ...` assignments out of `__init__`. Nested
//! control flow in `__init__` is beyond that mini-transpiler, and running it
//! line by line would emit every branch unconditionally, so those classes are
//! mirrored here by hand and applied once the unit's stats are resolved. Keep
//! each arm in step with its Python source, cited inline.

use super::super::operator_unit::OperatorUnit;

/// Apply hand-mirrored `__init__` state for `unit`'s operator. A no-op for
/// every operator not listed.
// A one-arm dispatch table that will grow; keeps the shape of dispatch.rs.
#[allow(clippy::single_match)]
pub fn apply(unit: &mut OperatorUnit) {
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match op_id {
        // damage_formulas.py `Walter.__init__`: the shadows summoned by talent 2
        // each add fmax(drone_atk*(1-res/100), drone_atk*0.05)/4.25 to every
        // skill's DPS. Without this the calculator understated Wiš'adel by
        // n*drone_atk/4.25 (777/4.25 = 182.8 per shadow at E2 max), found by
        // the first CI run of the parity suite, 2026-09-14.
        //
        //   self.shadows = 0
        //   if self.elite == 2:
        //       if self.skill in [0,3]:
        //           if self.talent2_dmg: self.shadows = 3
        //           else: self.shadows = min(self.skill + 1, 2)
        //           if self.skill_params[1] == 1 and self.skill == 3: self.shadows -= 1
        //       else:
        //           self.shadows = 1 if self.talent2_dmg else 0
        "char_1035_wisdel" => {
            let skill = unit.skill_index;
            unit.shadows = if unit.elite != 2 {
                0.0
            } else if skill == 0 || skill == 3 {
                let mut shadows = if unit.talent2_damage {
                    3.0
                } else {
                    f64::from((skill + 1).min(2))
                };
                if skill == 3 && unit.skill_parameters.get(1).copied() == Some(1.0) {
                    shadows -= 1.0;
                }
                shadows
            } else if unit.talent2_damage {
                1.0
            } else {
                0.0
            };
        }
        _ => {}
    }
}
