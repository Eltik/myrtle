use myrtle_dps_formulas::tables::supported_operators;
use myrtle_engine_dps::zones::BuffZones;
use myrtle_engine_dps::{DpsMetrics, SkillCategory, SpRecovery};
use serde::{Deserialize, Serialize};

use super::request::{
    CalculateRequest, RequestBuffs, RequestCycleModel, RequestSecondary, RequestShred,
};
use super::retention::RetentionStatus;
use super::{Data, advanced_modules, calculate_detailed_in, resolve_module};
use crate::AppError;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum RankPreset {
    Default,
    #[default]
    P1Max,
    P6Max,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RankMetric {
    #[default]
    SkillDps,
    TotalDamage,
    AverageDps,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DamageKind {
    Physical,
    Arts,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RankFilters {
    pub professions: Option<Vec<String>>,
    pub rarities: Option<Vec<i32>>,
    pub damage_types: Option<Vec<DamageKind>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankRequest {
    pub defense: Option<f64>,
    pub res: Option<f64>,
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    pub all_cond: Option<bool>,
    pub zones: Option<BuffZones>,
    pub secondary: Option<RequestSecondary>,
    pub cycle_model: Option<RequestCycleModel>,
    pub preset: Option<RankPreset>,
    pub filters: Option<RankFilters>,
    pub sort_by: Option<RankMetric>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RankContext<'a> {
    defense: Option<f64>,
    res: Option<f64>,
    buffs: &'a Option<RequestBuffs>,
    shred: &'a Option<RequestShred>,
    targets: Option<i32>,
    sp_boost: Option<f32>,
    all_cond: Option<bool>,
    zones: &'a Option<BuffZones>,
    secondary: Option<RequestSecondary>,
    cycle_model: Option<RequestCycleModel>,
    preset: RankPreset,
}

impl RankRequest {
    pub(super) fn context_json(&self) -> String {
        serde_json::to_string(&RankContext {
            defense: self.defense,
            res: self.res,
            buffs: &self.buffs,
            shred: &self.shred,
            targets: self.targets,
            sp_boost: self.sp_boost,
            all_cond: self.all_cond,
            zones: &self.zones,
            secondary: self.secondary,
            cycle_model: self.cycle_model,
            preset: self.preset.unwrap_or_default(),
        })
        .unwrap_or_default()
    }

    #[must_use]
    pub fn on_page_grid(&self) -> bool {
        let on_step = |v: Option<f64>, step: f64, max: f64| {
            v.is_none_or(|v| (0.0..=max).contains(&v) && (v / step).fract() == 0.0)
        };
        on_step(self.defense, 100.0, 5_000.0)
            && on_step(self.res, 5.0, 100.0)
            && self.targets.is_none_or(|t| t == 1)
            && self.buffs.is_none()
            && self.shred.is_none()
            && self.sp_boost.is_none()
            && self.all_cond.is_none()
            && self.zones.is_none()
            && self.secondary.is_none()
    }

    #[must_use]
    pub fn row_request(&self, operator_id: &str, skill: i32, module: i32) -> CalculateRequest {
        let preset = self.preset.unwrap_or_default();
        let maxed = preset != RankPreset::Default;
        CalculateRequest {
            operator_id: operator_id.to_owned(),
            promotion: maxed.then_some(2),
            level: maxed.then_some(90),
            potential: match preset {
                RankPreset::Default => None,
                RankPreset::P1Max => Some(1),
                RankPreset::P6Max => Some(6),
            },
            trust: maxed.then_some(100),
            skill_index: Some(skill),
            mastery_level: maxed.then_some(3),
            skill_level: None,
            module_index: Some(module),
            module_level: (maxed && module > 0).then_some(3),
            defense: self.defense,
            res: self.res,
            buffs: self.buffs.clone(),
            shred: self.shred.clone(),
            targets: self.targets,
            sp_boost: self.sp_boost,
            conditionals: None,
            all_cond: self.all_cond,
            zones: self.zones.clone(),
            secondary: self.secondary,
            cycle_model: self.cycle_model,
            frame_timing: None,
            clone_operator_id: None,
        }
    }

    pub fn validate(&self) -> Result<(), AppError> {
        self.row_request("char_rank_probe", 0, 0).validate()?;
        if self.limit.is_some_and(|l| l > 5000) {
            return Err(AppError::bad_request("limit must be at most 5000"));
        }
        if let Some(f) = &self.filters
            && (f.professions.as_ref().is_some_and(|v| v.len() > 16)
                || f.rarities.as_ref().is_some_and(|v| v.len() > 16)
                || f.damage_types.as_ref().is_some_and(|v| v.len() > 4))
        {
            return Err(AppError::bad_request("too many filter values"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankRow {
    pub operator_id: String,
    pub name: String,
    pub rarity: i32,
    pub profession: String,
    pub sub_profession_id: String,
    pub damage_type: DamageKind,
    pub skill_index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    pub module_index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<DpsMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<DpsMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<SkillCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sp_recovery: Option<SpRecovery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_time: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_time: Option<f64>,
    #[serde(default)]
    pub retention: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankComputed {
    pub rows: Vec<RankRow>,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankResponse {
    pub rows: Vec<RankRow>,
    pub matched: usize,
    pub computed: usize,
    pub skipped: usize,
    pub preset: RankPreset,
    pub sort_by: RankMetric,
    pub data: String,
    pub retention_status: RetentionStatus,
}

#[must_use]
pub fn rank_rows_in(data: &Data, req: &RankRequest) -> RankComputed {
    let formulas = supported_operators();
    let mut ids: Vec<&String> = formulas.keys().collect();
    ids.sort();
    let mut rows = Vec::new();
    let mut skipped = 0;
    for id in ids {
        let formula = &formulas[id];
        let Some(operator) = data.snapshot.model.operators.get(id) else {
            continue;
        };
        let Some(prepared) = data.operator(id) else {
            continue;
        };
        let sorted = advanced_modules(operator);
        let mut modules = vec![(0, None)];
        for (pos, &m) in formula.available_modules.iter().enumerate() {
            if let Some(module) = resolve_module(&sorted, pos, m) {
                modules.push((m, Some(module)));
            }
        }
        let skills = if formula.available_skills.is_empty() {
            vec![0]
        } else {
            formula.available_skills.clone()
        };
        let profession = serde_json::to_value(&operator.profession)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        for &skill in &skills {
            let skill_static = usize::try_from(skill - 1)
                .ok()
                .and_then(|i| operator.skills.get(i));
            for &(module, module_ref) in &modules {
                let Ok((r, duration)) =
                    calculate_detailed_in(data, req.row_request(id, skill, module))
                else {
                    skipped += 1;
                    continue;
                };
                rows.push(RankRow {
                    operator_id: id.clone(),
                    name: operator.name.clone(),
                    rarity: prepared.rarity,
                    profession: profession.clone(),
                    sub_profession_id: operator.sub_profession_id.clone(),
                    damage_type: if prepared.is_physical {
                        DamageKind::Physical
                    } else {
                        DamageKind::Arts
                    },
                    skill_index: skill,
                    skill_id: skill_static.map(|s| s.skill_id.clone()),
                    skill_name: skill_static
                        .and_then(|s| s.static_data.as_ref())
                        .and_then(|s| s.levels.last())
                        .map(|l| l.name.clone()),
                    module_index: module,
                    module_id: module_ref.map(|m| m.module.uni_equip_id.clone()),
                    module_name: module_ref.map(|m| m.module.uni_equip_name.clone()),
                    skill_dps: r.skill_dps,
                    total_damage: r.total_damage,
                    average_dps: r.average_dps,
                    primary: r.primary,
                    secondary: r.secondary,
                    category: r.category,
                    sp_recovery: r.sp_recovery,
                    startup_time: r.startup_time,
                    duration: (skill > 0 && duration > 0.0).then_some(duration),
                    cycle_time: r.cycle_time,
                    retention: None,
                });
            }
        }
    }
    RankComputed { rows, skipped }
}

fn passes<T>(filter: &Option<Vec<T>>, matches: impl Fn(&T) -> bool) -> bool {
    match filter {
        Some(values) if !values.is_empty() => values.iter().any(matches),
        _ => true,
    }
}

#[must_use]
pub fn finish_rank(computed: &RankComputed, req: &RankRequest, data: &str) -> RankResponse {
    let sort_by = req.sort_by.unwrap_or_default();
    let f = req.filters.clone().unwrap_or_default();
    let mut rows: Vec<RankRow> = computed
        .rows
        .iter()
        .filter(|r| {
            passes(&f.professions, |p| p.eq_ignore_ascii_case(&r.profession))
                && passes(&f.rarities, |v| *v == r.rarity)
                && passes(&f.damage_types, |v| *v == r.damage_type)
        })
        .cloned()
        .collect();
    let metric = |r: &RankRow| match sort_by {
        RankMetric::SkillDps => r.skill_dps,
        RankMetric::TotalDamage => r.total_damage,
        RankMetric::AverageDps => r.average_dps,
    };
    rows.sort_by(|a, b| {
        metric(b)
            .total_cmp(&metric(a))
            .then_with(|| a.operator_id.cmp(&b.operator_id))
            .then_with(|| a.skill_index.cmp(&b.skill_index))
            .then_with(|| a.module_index.cmp(&b.module_index))
    });
    let matched = rows.len();
    if let Some(limit) = req.limit {
        rows.truncate(limit as usize);
    }
    RankResponse {
        rows,
        matched,
        computed: computed.rows.len(),
        skipped: computed.skipped,
        preset: req.preset.unwrap_or_default(),
        sort_by,
        data: data.to_owned(),
        retention_status: RetentionStatus::Pending,
    }
}
