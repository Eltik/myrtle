//! What CN's game data says about an operator that the trailing server's does
//! not, for operators both servers already have.
//!
//! The two servers' TEXT is in different languages, so nothing here reads a
//! name or a description. Each category is projected to its structure and
//! numbers (ids, unlock conditions, blackboards, stats) and the projections are
//! compared, numbers within [`EPSILON`]. Blackboards are compared as sorted
//! `(key, value)` sets, so a reordered blackboard is not a change.
//!
//! Left out because they cannot be compared honestly or are not gameplay:
//! module `Target` and `ResKey` (which text a module overrides: 2026-10-24
//! CN vs 2026-10-20 EN, `uniequip_002_slent2` flips `TRAIT` to
//! `TRAIT_DATA_ONLY` with every number equal), module and promotion costs, base
//! skills (their numbers live in the building table, not on the operator),
//! handbook, voices and skins.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ts_rs::TS;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::{ModuleBlackboard, ModuleType};
use crate::core::gamedata::types::operator::{
    AttributeKeyFrame, Blackboard, Operator, UnlockCondition,
};
use crate::core::gamedata::types::skill::{SkillBlackboard, SkillLevel};

/// Two numbers closer than this are equal. The tables carry `f32` values
/// widened to `f64`, so the same 0.1 can arrive with different low bits.
const EPSILON: f64 = 1e-6;

/// One way an operator's CN data differs from the trailing server's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum OperatorChange {
    /// A module CN has and the trailing server does not. The default
    /// (`ORIGINAL`) module is never reported: it appears with the first
    /// real one.
    ModuleAdded { module_id: String },
    /// A module both have whose stats, talent or trait numbers, or unlock
    /// requirement differ.
    ModuleChanged { module_id: String },
    /// A skill CN lists that the trailing server does not.
    SkillAdded { skill_id: String },
    /// A skill both have whose per-level blackboard, SP data, duration or
    /// range differs.
    SkillValuesChanged { skill_id: String },
    /// Talent count, unlock conditions, ranges or blackboard values.
    TalentsChanged,
    /// Trait unlock conditions, ranges or blackboard values.
    TraitChanged,
    /// Promotion count, level caps or attribute key frames.
    StatsChanged,
    /// The attack range of a promotion phase.
    RangeChanged,
    /// Potential rank types or their attribute bonuses.
    PotentialsChanged,
    /// The trust bonus key frames.
    TrustBonusChanged,
}

/// Every difference found on one operator, in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OperatorCnChanges {
    pub id: String,
    pub changes: Vec<OperatorChange>,
}

/// The operators on both servers whose CN data differs, sorted by id.
#[must_use]
pub fn cn_changes(base: &GameData, cn: &GameData) -> Vec<OperatorCnChanges> {
    let mut out: Vec<OperatorCnChanges> = base
        .operators
        .iter()
        .filter_map(|(id, op)| {
            let changes = operator_changes(op, cn.operators.get(id)?);
            (!changes.is_empty()).then(|| OperatorCnChanges {
                id: id.clone(),
                changes,
            })
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// What `cn` adds or changes over `base`, the same operator on the trailing
/// server.
#[must_use]
pub fn operator_changes(base: &Operator, cn: &Operator) -> Vec<OperatorChange> {
    let mut out = Vec::new();

    let base_modules: HashMap<&str, Value> = base
        .modules
        .iter()
        .map(|m| (m.module.uni_equip_id.as_str(), module_shape(m)))
        .collect();
    for m in &cn.modules {
        let id = m.module.uni_equip_id.as_str();
        match base_modules.get(id) {
            None if m.module.module_type != ModuleType::Initial => {
                out.push(OperatorChange::ModuleAdded {
                    module_id: id.to_string(),
                });
            }
            Some(shape) if !same(shape, &module_shape(m)) => {
                out.push(OperatorChange::ModuleChanged {
                    module_id: id.to_string(),
                });
            }
            _ => {}
        }
    }

    let base_skills: HashMap<&str, Value> = base
        .skills
        .iter()
        .map(|s| {
            (
                s.skill_id.as_str(),
                skill_shape(s.static_data.as_ref().map(|d| d.levels.as_slice())),
            )
        })
        .collect();
    for s in &cn.skills {
        let id = s.skill_id.as_str();
        if id.is_empty() {
            continue;
        }
        match base_skills.get(id) {
            None => out.push(OperatorChange::SkillAdded {
                skill_id: id.to_string(),
            }),
            Some(shape) => {
                let levels = s.static_data.as_ref().map(|d| d.levels.as_slice());
                if !same(shape, &skill_shape(levels)) {
                    out.push(OperatorChange::SkillValuesChanged {
                        skill_id: id.to_string(),
                    });
                }
            }
        }
    }

    let checks: [(fn(&Operator) -> Value, OperatorChange); 6] = [
        (talents_shape, OperatorChange::TalentsChanged),
        (trait_shape, OperatorChange::TraitChanged),
        (stats_shape, OperatorChange::StatsChanged),
        (range_shape, OperatorChange::RangeChanged),
        (potentials_shape, OperatorChange::PotentialsChanged),
        (trust_shape, OperatorChange::TrustBonusChanged),
    ];
    for (shape, change) in checks {
        if !same(&shape(base), &shape(cn)) {
            out.push(change);
        }
    }
    out
}

/// Structural equality with numbers compared within [`EPSILON`].
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= EPSILON * x.abs().max(y.abs()).max(1.0),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

/// A blackboard as `(key, value, valueStr)` rows sorted by key. Keys are
/// lowercased: the tables spell the same key in both cases.
fn blackboard(rows: impl Iterator<Item = (String, f64, Option<String>)>) -> Value {
    let mut rows: Vec<(String, f64, Option<String>)> =
        rows.map(|(k, v, s)| (k.to_lowercase(), v, s)).collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    Value::Array(rows.into_iter().map(|(k, v, s)| json!([k, v, s])).collect())
}

fn operator_blackboard(rows: &[Blackboard]) -> Value {
    blackboard(
        rows.iter()
            .map(|b| (b.key.clone(), b.value, b.value_str.clone())),
    )
}

fn skill_blackboard(rows: &[SkillBlackboard]) -> Value {
    blackboard(
        rows.iter()
            .map(|b| (b.key.clone(), b.value, b.value_str.clone())),
    )
}

fn module_blackboard(rows: &[ModuleBlackboard]) -> Value {
    blackboard(rows.iter().map(|b| (b.key.clone(), b.value, None)))
}

fn unlock(u: &UnlockCondition) -> Value {
    json!([u.phase, u.level])
}

fn module_shape(m: &crate::core::gamedata::types::operator::OperatorModule) -> Value {
    let phases: Vec<Value> = m
        .data
        .phases
        .iter()
        .map(|p| {
            let parts: Vec<Value> = p
                .parts
                .iter()
                .map(|part| {
                    let talents: Vec<Value> = part
                        .add_or_override_talent_data_bundle
                        .candidates
                        .iter()
                        .flatten()
                        .map(|c| {
                            json!([
                                c.talent_index,
                                c.unlock_condition.phase,
                                c.unlock_condition.level,
                                c.required_potential_rank,
                                c.range_id,
                                module_blackboard(&c.blackboard),
                            ])
                        })
                        .collect();
                    let traits: Vec<Value> = part
                        .override_trait_data_bundle
                        .candidates
                        .iter()
                        .flatten()
                        .map(|c| {
                            json!([
                                c.unlock_condition.phase,
                                c.unlock_condition.level,
                                c.required_potential_rank,
                                c.range_id,
                                module_blackboard(&c.blackboard),
                            ])
                        })
                        .collect();
                    json!([part.is_token, talents, traits])
                })
                .collect();
            json!([
                p.equip_level,
                module_blackboard(&p.attribute_blackboard),
                parts
            ])
        })
        .collect();
    json!([
        m.module.unlock_evolve_phase,
        m.module.unlock_level,
        m.module.unlock_favor_point,
        phases,
    ])
}

fn skill_shape(levels: Option<&[SkillLevel]>) -> Value {
    let levels: Vec<Value> = levels
        .unwrap_or_default()
        .iter()
        .map(|l| {
            let sp = &l.sp_data;
            json!([
                l.skill_type,
                l.duration_type,
                l.range_id,
                l.duration,
                [
                    sp.sp_type,
                    sp.max_charge_time,
                    sp.sp_cost,
                    sp.init_sp,
                    sp.increment
                ],
                skill_blackboard(&l.blackboard),
            ])
        })
        .collect();
    Value::Array(levels)
}

fn talents_shape(op: &Operator) -> Value {
    op.talents
        .iter()
        .map(|t| {
            t.candidates
                .iter()
                .map(|c| {
                    json!([
                        unlock(&c.unlock_condition),
                        c.required_potential_rank,
                        c.range_id,
                        operator_blackboard(&c.blackboard),
                    ])
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn trait_shape(op: &Operator) -> Value {
    op.trait_data
        .iter()
        .flat_map(|t| &t.candidates)
        .map(|c| {
            json!([
                unlock(&c.unlock_condition),
                c.required_potential_rank,
                c.range_id,
                operator_blackboard(&c.blackboard),
            ])
        })
        .collect()
}

fn key_frames(frames: &[AttributeKeyFrame]) -> Value {
    frames
        .iter()
        .map(|f| {
            json!([
                f.level,
                serde_json::to_value(&f.data).unwrap_or(Value::Null)
            ])
        })
        .collect()
}

fn stats_shape(op: &Operator) -> Value {
    op.phases
        .iter()
        .map(|p| json!([p.max_level, key_frames(&p.attributes_key_frames)]))
        .collect()
}

fn range_shape(op: &Operator) -> Value {
    op.phases.iter().map(|p| json!(p.range_id)).collect()
}

fn potentials_shape(op: &Operator) -> Value {
    op.potential_ranks
        .iter()
        .map(|r| {
            let modifiers: Vec<Value> = r
                .buff
                .iter()
                .filter_map(|b| b.attributes.attribute_modifiers.as_ref())
                .flatten()
                .map(|m| json!([m.attribute_type, m.formula_item, m.value]))
                .collect();
            json!([r.potential_type, modifiers])
        })
        .collect()
}

fn trust_shape(op: &Operator) -> Value {
    key_frames(&op.favor_key_frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::module::{Module, ModuleData, ModulePhase};
    use crate::core::gamedata::types::operator::{
        EnrichedSkill, OperatorModule, Phase, SkillStatic, Talent, TalentCandidate,
    };
    use crate::core::gamedata::types::skill::SkillSpData;

    fn module(id: &str, kind: ModuleType, atk: f64) -> OperatorModule {
        OperatorModule {
            module: Module {
                uni_equip_id: id.to_string(),
                module_type: kind,
                ..Module::default()
            },
            data: ModuleData {
                phases: vec![ModulePhase {
                    equip_level: 1,
                    attribute_blackboard: vec![ModuleBlackboard {
                        key: "atk".into(),
                        value: atk,
                    }],
                    ..ModulePhase::default()
                }],
            },
        }
    }

    fn skill(id: &str, values: &[(&str, f64)], name: &str) -> EnrichedSkill {
        EnrichedSkill {
            skill_id: id.to_string(),
            static_data: Some(SkillStatic {
                skill_id: id.to_string(),
                levels: vec![SkillLevel {
                    name: name.to_string(),
                    description: format!("{name} description"),
                    sp_data: SkillSpData {
                        sp_cost: 30,
                        ..SkillSpData::default()
                    },
                    blackboard: values
                        .iter()
                        .map(|(k, v)| SkillBlackboard {
                            key: (*k).to_string(),
                            value: *v,
                            value_str: None,
                        })
                        .collect(),
                    ..SkillLevel::default()
                }],
                ..SkillStatic::default()
            }),
            ..EnrichedSkill::default()
        }
    }

    fn base() -> Operator {
        Operator {
            name: "Shu".into(),
            modules: vec![
                module("uniequip_001_shu", ModuleType::Initial, 0.0),
                module("uniequip_002_shu", ModuleType::Advanced, 60.0),
            ],
            skills: vec![skill("skchr_shu_1", &[("atk", 0.3), ("heal", 0.2)], "Rain")],
            phases: vec![Phase {
                max_level: 50,
                ..Phase::default()
            }],
            talents: vec![Talent {
                candidates: vec![TalentCandidate {
                    name: Some("Harvest".into()),
                    blackboard: vec![Blackboard {
                        key: "hp_ratio".into(),
                        value: 0.1,
                        value_str: None,
                    }],
                    ..TalentCandidate::default()
                }],
            }],
            ..Operator::default()
        }
    }

    #[test]
    fn identical_operators_and_translated_text_report_nothing() {
        let en = base();
        let mut cn = base();
        cn.name = "黍".into();
        cn.description = "不同语言".into();
        cn.skills[0].static_data.as_mut().unwrap().levels[0].name = "雨".into();
        cn.talents[0].candidates[0].name = Some("丰收".into());
        assert_eq!(operator_changes(&en, &cn), vec![]);
    }

    #[test]
    fn a_reordered_blackboard_or_widened_float_is_not_a_change() {
        let en = base();
        let mut cn = base();
        cn.skills[0] = skill(
            "skchr_shu_1",
            &[("heal", 0.200_000_002_980_232_24), ("atk", 0.3)],
            "雨",
        );
        assert_eq!(operator_changes(&en, &cn), vec![]);
    }

    #[test]
    fn a_new_advanced_module_is_reported_and_the_default_one_is_not() {
        let mut en = base();
        en.modules.clear();
        let mut cn = base();
        cn.modules
            .push(module("uniequip_003_shu", ModuleType::Advanced, 80.0));
        assert_eq!(
            operator_changes(&en, &cn),
            vec![
                OperatorChange::ModuleAdded {
                    module_id: "uniequip_002_shu".into()
                },
                OperatorChange::ModuleAdded {
                    module_id: "uniequip_003_shu".into()
                },
            ]
        );
    }

    #[test]
    fn numeric_changes_are_reported_per_category() {
        let en = base();
        let mut cn = base();
        cn.modules[1] = module("uniequip_002_shu", ModuleType::Advanced, 65.0);
        cn.skills[0] = skill("skchr_shu_1", &[("atk", 0.35), ("heal", 0.2)], "雨");
        cn.skills.push(skill("skchr_shu_4", &[], "新"));
        cn.talents[0].candidates[0].blackboard[0].value = 0.12;
        cn.phases[0].max_level = 55;
        assert_eq!(
            operator_changes(&en, &cn),
            vec![
                OperatorChange::ModuleChanged {
                    module_id: "uniequip_002_shu".into()
                },
                OperatorChange::SkillValuesChanged {
                    skill_id: "skchr_shu_1".into()
                },
                OperatorChange::SkillAdded {
                    skill_id: "skchr_shu_4".into()
                },
                OperatorChange::TalentsChanged,
                OperatorChange::StatsChanged,
            ]
        );
    }

    #[test]
    fn the_wire_shape_is_kind_tagged_with_camel_case_fields() {
        let value = serde_json::to_value(OperatorChange::ModuleAdded {
            module_id: "uniequip_003_shu".into(),
        })
        .unwrap();
        assert_eq!(
            value,
            json!({ "kind": "module_added", "moduleId": "uniequip_003_shu" })
        );
        assert_eq!(
            serde_json::to_value(OperatorChange::StatsChanged).unwrap(),
            json!({ "kind": "stats_changed" })
        );
    }
}
