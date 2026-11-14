use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::{ModuleBlackboard, ModuleType};
use myrtle_gamedata_raw::tables::operator::{
    AttributeKeyFrame, Blackboard, Operator, OperatorModule, UnlockCondition,
};
use myrtle_gamedata_raw::tables::skill::{SkillBlackboard, SkillLevel};

const EPSILON: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum OperatorChange {
    ModuleAdded {
        module_id: String,
        translations: Vec<CnTranslation>,
    },
    /// A module that had stats/trait/etc. changed
    ModuleChanged { module_id: String },
    /// Skill that was added
    SkillAdded { skill_id: String },
    /// Skill that had blackboard/values changed
    SkillValuesChanged { skill_id: String },
    /// Talent count, unlock conditions, ranges, etc. that changed
    TalentsChanged,
    /// Trait unlock conditions, ranges, etc. that changed
    TraitChanged,
    /// Promotion count, level caps, etc. that changed
    StatsChanged,
    /// Attack range that changed
    RangeChanged,
    /// Potential/attributes that changed
    PotentialsChanged,
    /// Trust bonus key frames that changed
    TrustBonusChanged,
}

impl OperatorChange {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CnTranslation {
    pub cn: String,
    pub en: String,
}

/// Every difference found on one operator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorCnChanges {
    pub id: String,
    pub changes: Vec<OperatorChange>,
}

#[must_use]
pub fn cn_changes(base: &GameModel, cn: &GameModel) -> Vec<OperatorCnChanges> {
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
    out
}

#[must_use]
pub fn shared_operator_count(base: &GameModel, cn: &GameModel) -> usize {
    base.operators
        .keys()
        .filter(|id| cn.operators.contains_key(*id))
        .count()
}

#[must_use]
pub fn data_wide_kinds(changes: &[OperatorCnChanges], shared: usize) -> Vec<(&'static str, usize)> {
    if shared == 0 {
        return Vec::new();
    }
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for op in changes {
        for change in &op.changes {
            *counts.entry(change.kind()).or_default() += 1;
        }
    }
    let mut wide: Vec<_> = counts
        .into_iter()
        .filter(|&(_, n)| n * 2 > shared)
        .collect();
    wide.sort_unstable();
    wide
}

#[derive(Debug, Default)]
pub struct SharedTexts {
    pub texts: HashMap<String, String>,
    pub ambiguous: usize,
}

impl SharedTexts {
    #[must_use]
    pub fn build(base: &GameModel, cn: &GameModel) -> Self {
        let mut pairs: Vec<(&str, &str)> = Vec::new();
        for (id, op) in &base.operators {
            if let Some(cn_op) = cn.operators.get(id) {
                module_text_pairs(op, cn_op, &mut pairs);
            }
        }
        Self::from_pairs(pairs)
    }

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
/// module id and the same phase, part and candidate
pub fn module_text_pairs<'a>(
    base: &'a Operator,
    cn: &'a Operator,
    out: &mut Vec<(&'a str, &'a str)>,
) {
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

/// What `cn` adds or changes over `base`
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

type ShapeCheck = (fn(&Operator) -> Value, OperatorChange);

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

/// A blackboard as `(key, value, valueStr)` rows sorted by key
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
