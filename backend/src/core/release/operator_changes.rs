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
    AttributeKeyFrame, Blackboard, Operator, OperatorModule, UnlockCondition,
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
    /// real one. `translations` carries the trailing server's text for each
    /// of the module's CN texts that a module both servers have also uses.
    ModuleAdded {
        module_id: String,
        translations: Vec<CnTranslation>,
    },
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

impl OperatorChange {
    /// The wire `kind` tag.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ModuleAdded { .. } => "module_added",
            Self::ModuleChanged { .. } => "module_changed",
            Self::SkillAdded { .. } => "skill_added",
            Self::SkillValuesChanged { .. } => "skill_values_changed",
            Self::TalentsChanged => "talents_changed",
            Self::TraitChanged => "trait_changed",
            Self::StatsChanged => "stats_changed",
            Self::RangeChanged => "range_changed",
            Self::PotentialsChanged => "potentials_changed",
            Self::TrustBonusChanged => "trust_bonus_changed",
        }
    }
}

/// A CN text and the trailing server's text at the same place, both raw
/// templates (`{key}` placeholders and rich-text tags left in).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CnTranslation {
    pub cn: String,
    pub en: String,
}

/// Every difference found on one operator, in a fixed order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OperatorCnChanges {
    pub id: String,
    pub changes: Vec<OperatorChange>,
}

/// The operators on both servers whose CN data differs, sorted by id. Each
/// added module carries the translations [`SharedTexts`] has for its text.
#[must_use]
pub fn cn_changes(base: &GameData, cn: &GameData) -> Vec<OperatorCnChanges> {
    let shared = SharedTexts::build(base, cn);
    let mut out: Vec<OperatorCnChanges> = base
        .operators
        .iter()
        .filter_map(|(id, op)| {
            let cn_op = cn.operators.get(id)?;
            let mut changes = operator_changes(op, cn_op);
            for change in &mut changes {
                if let OperatorChange::ModuleAdded {
                    module_id,
                    translations,
                } = change
                    && let Some(m) = cn_op
                        .modules
                        .iter()
                        .find(|m| &m.module.uni_equip_id == module_id)
                {
                    *translations = shared.translations(m);
                }
            }
            (!changes.is_empty()).then(|| OperatorCnChanges {
                id: id.clone(),
                changes,
            })
        })
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    warn_on_data_wide_kinds(&out, shared_operator_count(base, cn));
    out
}

fn shared_operator_count(base: &GameData, cn: &GameData) -> usize {
    base.operators
        .keys()
        .filter(|id| cn.operators.contains_key(*id))
        .count()
}

/// A kind of change on most shared operators is a decode or schema difference
/// between the two extracts, not a balance change, so it is logged loudly.
fn warn_on_data_wide_kinds(changes: &[OperatorCnChanges], shared: usize) {
    if shared == 0 {
        return;
    }
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for op in changes {
        for change in &op.changes {
            *counts.entry(change.kind()).or_default() += 1;
        }
    }
    for (kind, n) in counts {
        if n * 2 > shared {
            tracing::warn!(
                kind,
                operators = n,
                shared,
                "cn changes: one kind flags most shared operators; likely an extract difference, not an update"
            );
        }
    }
}

/// CN text -> the trailing server's text, read off modules both servers have:
/// the same module, phase, part and candidate on each side. A CN text paired
/// with two different texts is dropped as ambiguous.
#[derive(Debug, Default)]
pub struct SharedTexts {
    pub texts: HashMap<String, String>,
    /// CN texts dropped for pairing with more than one text.
    pub ambiguous: usize,
}

impl SharedTexts {
    #[must_use]
    pub fn build(base: &GameData, cn: &GameData) -> Self {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        for (id, op) in &base.operators {
            if let Some(cn_op) = cn.operators.get(id) {
                module_text_pairs(op, cn_op, &mut pairs);
            }
        }
        Self::from_pairs(pairs)
    }

    /// Keeps each CN text that pairs with exactly one other text.
    #[must_use]
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut seen: HashMap<&str, Option<&str>> = HashMap::new();
        for (cn, en) in pairs {
            if cn.is_empty() || en.is_empty() || cn == en {
                continue;
            }
            seen.entry(cn)
                .and_modify(|prior| {
                    if prior.is_some_and(|p| p != en) {
                        *prior = None;
                    }
                })
                .or_insert(Some(en));
        }
        let ambiguous = seen.values().filter(|en| en.is_none()).count();
        let texts = seen
            .into_iter()
            .filter_map(|(cn, en)| Some((cn.to_string(), en?.to_string())))
            .collect();
        Self { texts, ambiguous }
    }

    /// The pairs for the texts `module` uses, in the order it uses them,
    /// each once.
    #[must_use]
    pub fn translations(&self, module: &OperatorModule) -> Vec<CnTranslation> {
        let mut out: Vec<CnTranslation> = Vec::new();
        for text in module_texts(module) {
            if let Some(en) = self.texts.get(text)
                && !out.iter().any(|t| t.cn == text)
            {
                out.push(CnTranslation {
                    cn: text.to_string(),
                    en: en.clone(),
                });
            }
        }
        out
    }
}

/// Every text a module's parts carry, by part position: talent names and
/// descriptions, then trait additions and overrides, per candidate.
fn module_texts(m: &OperatorModule) -> impl Iterator<Item = &str> {
    m.data
        .phases
        .iter()
        .flat_map(|p| &p.parts)
        .flat_map(|part| {
            let talents = part
                .add_or_override_talent_data_bundle
                .candidates
                .iter()
                .flatten()
                .flat_map(|c| {
                    [
                        Some(c.name.as_str()),
                        Some(c.upgrade_description.as_str()),
                        c.description.as_deref(),
                    ]
                });
            let traits = part
                .override_trait_data_bundle
                .candidates
                .iter()
                .flatten()
                .flat_map(|c| {
                    [
                        Some(c.additional_description.as_str()),
                        c.override_description.as_deref(),
                    ]
                });
            talents.chain(traits).flatten()
        })
}

/// `(cn, base)` text pairs from the modules both operators have, matched by
/// module id and the same phase, part and candidate.
fn module_text_pairs<'a>(base: &'a Operator, cn: &'a Operator, out: &mut Vec<(&'a str, &'a str)>) {
    for m in &base.modules {
        let Some(cn_m) = cn
            .modules
            .iter()
            .find(|c| c.module.uni_equip_id == m.module.uni_equip_id)
        else {
            continue;
        };
        for (p, cn_p) in m.data.phases.iter().zip(&cn_m.data.phases) {
            for (part, cn_part) in p.parts.iter().zip(&cn_p.parts) {
                let talents = part
                    .add_or_override_talent_data_bundle
                    .candidates
                    .iter()
                    .flatten()
                    .zip(
                        cn_part
                            .add_or_override_talent_data_bundle
                            .candidates
                            .iter()
                            .flatten(),
                    );
                for (c, cn_c) in talents {
                    out.push((&cn_c.name, &c.name));
                    out.push((&cn_c.upgrade_description, &c.upgrade_description));
                    if let (Some(x), Some(y)) = (&cn_c.description, &c.description) {
                        out.push((x, y));
                    }
                }
                let traits = part
                    .override_trait_data_bundle
                    .candidates
                    .iter()
                    .flatten()
                    .zip(
                        cn_part
                            .override_trait_data_bundle
                            .candidates
                            .iter()
                            .flatten(),
                    );
                for (c, cn_c) in traits {
                    out.push((&cn_c.additional_description, &c.additional_description));
                    if let (Some(x), Some(y)) =
                        (&cn_c.override_description, &c.override_description)
                    {
                        out.push((x, y));
                    }
                }
            }
        }
    }
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
                    translations: Vec::new(),
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

    let checks: [ShapeCheck; 6] = [
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

/// A category's projection and the change reported when it differs.
type ShapeCheck = (fn(&Operator) -> Value, OperatorChange);

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

fn module_shape(m: &OperatorModule) -> Value {
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

/// Attribute key frames, numeric attributes only. The immunity flags are left
/// out: no operator on EN sets one, and the CN extract of 2026-11-01 decodes
/// `StunImmune` as true on all 467 of its operators, which flagged every shared
/// operator's stats and trust as changed.
fn key_frames(frames: &[AttributeKeyFrame]) -> Value {
    frames
        .iter()
        .map(|f| {
            let mut data = serde_json::to_value(&f.data).unwrap_or(Value::Null);
            if let Value::Object(fields) = &mut data {
                fields.retain(|_, v| v.is_number());
            }
            json!([f.level, data])
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
    use crate::core::gamedata::types::module::{
        AddModuleCandidates, AddOrOverrideTalentDataBundle, Module, ModuleCandidates, ModuleData,
        ModulePart, ModulePhase, OverrideTraitDataBundle,
    };
    use crate::core::gamedata::types::operator::AttributeData;
    use crate::core::gamedata::types::operator::{
        EnrichedSkill, Phase, SkillStatic, Talent, TalentCandidate,
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
                    module_id: "uniequip_002_shu".into(),
                    translations: vec![],
                },
                OperatorChange::ModuleAdded {
                    module_id: "uniequip_003_shu".into(),
                    translations: vec![],
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
    fn an_immunity_flag_is_not_a_stat_or_trust_change() {
        let frame = AttributeKeyFrame {
            level: 1,
            data: AttributeData {
                atk: 500,
                ..AttributeData::default()
            },
        };
        let mut en = base();
        en.phases[0].attributes_key_frames = vec![frame.clone()];
        en.favor_key_frames = vec![frame];
        let mut cn = en.clone();
        cn.phases[0].attributes_key_frames[0].data.stun_immune = true;
        cn.favor_key_frames[0].data.stun_immune = true;
        assert_eq!(operator_changes(&en, &cn), vec![]);

        cn.phases[0].attributes_key_frames[0].data.atk = 520;
        assert_eq!(
            operator_changes(&en, &cn),
            vec![OperatorChange::StatsChanged]
        );
    }

    #[test]
    fn every_kind_names_its_wire_tag() {
        for change in [
            OperatorChange::StatsChanged,
            OperatorChange::TrustBonusChanged,
            OperatorChange::SkillAdded {
                skill_id: "s".into(),
            },
        ] {
            let wire = serde_json::to_value(&change).unwrap();
            assert_eq!(wire["kind"], change.kind());
        }
    }

    #[test]
    fn the_wire_shape_is_kind_tagged_with_camel_case_fields() {
        let value = serde_json::to_value(OperatorChange::ModuleAdded {
            module_id: "uniequip_003_shu".into(),
            translations: vec![CnTranslation {
                cn: "受到的伤害减少15%".into(),
                en: "Reduces damage taken by 15%".into(),
            }],
        })
        .unwrap();
        assert_eq!(
            value,
            json!({
                "kind": "module_added",
                "moduleId": "uniequip_003_shu",
                "translations": [{ "cn": "受到的伤害减少15%", "en": "Reduces damage taken by 15%" }],
            })
        );
        assert_eq!(
            serde_json::to_value(OperatorChange::StatsChanged).unwrap(),
            json!({ "kind": "stats_changed" })
        );
    }

    fn texted(id: &str, trait_add: &str, talent: (&str, &str)) -> OperatorModule {
        let mut m = module(id, ModuleType::Advanced, 60.0);
        m.data.phases[0].parts = vec![
            ModulePart {
                override_trait_data_bundle: OverrideTraitDataBundle {
                    candidates: Some(vec![ModuleCandidates {
                        additional_description: trait_add.into(),
                        ..ModuleCandidates::default()
                    }]),
                },
                ..ModulePart::default()
            },
            ModulePart {
                add_or_override_talent_data_bundle: AddOrOverrideTalentDataBundle {
                    candidates: Some(vec![AddModuleCandidates {
                        name: talent.0.into(),
                        upgrade_description: talent.1.into(),
                        ..AddModuleCandidates::default()
                    }]),
                },
                ..ModulePart::default()
            },
        ];
        m
    }

    fn pair(cn: &str, en: &str) -> CnTranslation {
        CnTranslation {
            cn: cn.into(),
            en: en.into(),
        }
    }

    #[test]
    fn shared_texts_keep_exact_pairs_and_drop_ambiguous_ones() {
        let shared = SharedTexts::from_pairs([
            ("受到的伤害减少15%", "Reduces damage taken by 15%"),
            ("受到的伤害减少15%", "Reduces damage taken by 15%"),
            ("攻击力+{atk}", "ATK +{atk}"),
            ("攻击力+{atk}", "Attack +{atk}"),
            ("", "Empty"),
            ("+15%", "+15%"),
        ]);
        assert_eq!(shared.ambiguous, 1);
        assert_eq!(
            shared.texts,
            HashMap::from([(
                "受到的伤害减少15%".to_string(),
                "Reduces damage taken by 15%".to_string()
            )])
        );
    }

    #[test]
    fn shared_texts_pair_the_same_part_position_only() {
        let mut en_op = base();
        en_op.modules = vec![texted(
            "uniequip_002_yato",
            "Reduces damage taken by 15%",
            ("Edge", "ATK +{atk}"),
        )];
        let mut cn_op = base();
        cn_op.modules = vec![texted(
            "uniequip_002_yato",
            "受到的伤害减少15%",
            ("锋刃", "攻击力+{atk}"),
        )];
        // A module only one side has pairs nothing.
        cn_op
            .modules
            .push(texted("uniequip_003_yato", "仅CN", ("新", "新的")));
        let mut pairs = Vec::new();
        module_text_pairs(&en_op, &cn_op, &mut pairs);
        assert_eq!(
            pairs,
            vec![
                ("受到的伤害减少15%", "Reduces damage taken by 15%"),
                ("锋刃", "Edge"),
                ("攻击力+{atk}", "ATK +{atk}"),
            ]
        );
    }

    #[test]
    fn an_added_module_carries_the_translations_its_text_has() {
        let mut en = GameData::default();
        let mut cn = GameData::default();
        let mut yato_en = base();
        yato_en.modules = vec![texted(
            "uniequip_002_yato",
            "Reduces damage taken by 15%",
            ("Edge", "ATK +{atk}"),
        )];
        let mut yato_cn = base();
        yato_cn.modules = vec![texted(
            "uniequip_002_yato",
            "受到的伤害减少15%",
            ("锋刃", "攻击力+{atk}"),
        )];
        let mut shu_en = base();
        shu_en.modules.truncate(1);
        let mut shu_cn = base();
        shu_cn.modules = vec![
            shu_cn.modules[0].clone(),
            texted(
                "uniequip_002_shu",
                "受到的伤害减少15%",
                ("丰", "攻击力+{atk}"),
            ),
        ];
        en.operators.insert("char_yato".into(), yato_en);
        en.operators.insert("char_shu".into(), shu_en);
        cn.operators.insert("char_yato".into(), yato_cn);
        cn.operators.insert("char_shu".into(), shu_cn);
        let changes = cn_changes(&en, &cn);
        let shu = changes.iter().find(|o| o.id == "char_shu").unwrap();
        assert_eq!(
            shu.changes,
            vec![OperatorChange::ModuleAdded {
                module_id: "uniequip_002_shu".into(),
                translations: vec![
                    pair("受到的伤害减少15%", "Reduces damage taken by 15%"),
                    pair("攻击力+{atk}", "ATK +{atk}"),
                ],
            }]
        );
    }
}
