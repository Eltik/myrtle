use myrtle_dps_formulas::operator_unit::{
    OperatorBuffs, OperatorConditionals, OperatorParams, OperatorShred,
};
use myrtle_engine_dps::zones::{BuffZones, MAX_SOURCES, StatReduction};
use myrtle_engine_dps::{CycleModel, DpsRequest, DpsResult, HpsResult, SecondaryTarget};
use serde::{Deserialize, Serialize};

use crate::AppError;

pub const MAX_BATCH: usize = 64;
pub const MAX_SWEEP_STEPS: u32 = 101;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestConditionals {
    pub trait_damage: Option<bool>,
    pub talent_damage: Option<bool>,
    pub talent2_damage: Option<bool>,
    pub skill_damage: Option<bool>,
    pub module_damage: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateRequest {
    pub operator_id: String,
    pub promotion: Option<i32>,
    pub level: Option<i32>,
    pub potential: Option<i32>,
    pub trust: Option<i32>,
    pub skill_index: Option<i32>,
    pub mastery_level: Option<i32>,
    pub skill_level: Option<i32>,
    pub module_index: Option<i32>,
    pub module_level: Option<i32>,
    pub defense: Option<f64>,
    pub res: Option<f64>,
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    pub conditionals: Option<RequestConditionals>,
    pub all_cond: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zones: Option<BuffZones>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<RequestSecondary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_model: Option<RequestCycleModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_timing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_operator_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct RequestSecondary {
    pub def: f64,
    pub res: f64,
    pub count: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequestCycleModel {
    Legacy,
    V2,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
    pub aspd: Option<i32>,
    pub fragile: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestShred {
    pub def: Option<i32>,
    pub def_flat: Option<i32>,
    pub res: Option<i32>,
    pub res_flat: Option<i32>,
}

impl CalculateRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        int_range("promotion", self.promotion, 0, 2)?;
        int_range("level", self.level, 1, 90)?;
        int_range("potential", self.potential, 1, 6)?;
        int_range("trust", self.trust, 0, 200)?;
        int_range("skillIndex", self.skill_index, 0, 3)?;
        int_range("masteryLevel", self.mastery_level, 0, 3)?;
        int_range("skillLevel", self.skill_level, 1, 7)?;
        int_range("moduleIndex", self.module_index, -1, 5)?;
        int_range("moduleLevel", self.module_level, 0, 3)?;
        int_range("targets", self.targets, 1, 50)?;

        float_range("defense", self.defense, 0.0, 100_000.0)?;
        float_range("res", self.res, 0.0, 1_000.0)?;
        float_range("spBoost", self.sp_boost.map(f64::from), -10.0, 100.0)?;

        if let Some(buffs) = &self.buffs {
            int_range("buffs.aspd", buffs.aspd, -500, 1_000)?;
            int_range("buffs.flatAtk", buffs.flat_atk, -100_000, 100_000)?;
            float_range("buffs.atk", buffs.atk.map(f64::from), -10.0, 100.0)?;
            float_range("buffs.fragile", buffs.fragile.map(f64::from), -10.0, 100.0)?;
        }

        if let Some(shred) = &self.shred {
            int_range("shred.def", shred.def, -100, 100)?;
            int_range("shred.res", shred.res, -100, 100)?;
            int_range("shred.defFlat", shred.def_flat, -100_000, 100_000)?;
            int_range("shred.resFlat", shred.res_flat, -10_000, 10_000)?;
        }

        if let Some(zones) = &self.zones {
            validate_zones(zones)?;
        }

        if let Some(secondary) = &self.secondary {
            float_range("secondary.def", Some(secondary.def), 0.0, 100_000.0)?;
            float_range("secondary.res", Some(secondary.res), 0.0, 1_000.0)?;
            int_range("secondary.count", Some(secondary.count), 1, 49)?;
            if self.targets.is_some_and(|t| t > 1) {
                return Err(AppError::bad_request(
                    "targets and secondary are exclusive: secondary.count sets the extra targets",
                ));
            }
        }

        if self.frame_timing == Some(true) {
            return Err(AppError::bad_request(
                "frameTiming is not available: see docs/dps-refactor.md",
            ));
        }

        if let Some(id) = &self.clone_operator_id
            && (id.len() > 64 || !id.starts_with("char_"))
        {
            return Err(AppError::bad_request(
                "cloneOperatorId must be an operator id (char_...)",
            ));
        }

        Ok(())
    }

    pub(super) fn params(self) -> DpsRequest {
        DpsRequest {
            params: OperatorParams {
                promotion: self.promotion,
                level: self.level,
                potential: self.potential,
                trust: self.trust.or(Some(100)),
                skill_index: self.skill_index,
                mastery_level: self.mastery_level,
                skill_level: self.skill_level,
                module_index: self.module_index,
                module_level: self.module_level,
                buffs: self
                    .buffs
                    .map(|b| OperatorBuffs {
                        atk: b.atk,
                        flat_atk: b.flat_atk,
                        aspd: b.aspd,
                        fragile: b.fragile,
                    })
                    .unwrap_or_default(),
                sp_boost: self.sp_boost,
                targets: self.targets,
                shred: self.shred.map(|s| OperatorShred {
                    def: s.def,
                    def_flat: s.def_flat,
                    res: s.res,
                    res_flat: s.res_flat,
                }),
                conditionals: self.conditionals.map(|c| OperatorConditionals {
                    trait_damage: c.trait_damage,
                    talent_damage: c.talent_damage,
                    talent2_damage: c.talent2_damage,
                    skill_damage: c.skill_damage,
                    module_damage: c.module_damage,
                }),
                all_cond: self.all_cond,
                ..OperatorParams::default()
            },
            zones: self.zones,
            secondary: self.secondary.map(|s| SecondaryTarget {
                defense: s.def,
                res: s.res,
                count: s.count,
            }),
            cycle_model: match self.cycle_model {
                Some(RequestCycleModel::V2) => CycleModel::V2,
                Some(RequestCycleModel::Legacy) | None => CycleModel::Legacy,
            },
            clone_operator: None,
        }
    }
}

fn validate_zones(z: &BuffZones) -> Result<(), AppError> {
    float_range("zones.atkFlat", z.atk_flat, -100_000.0, 100_000.0)?;
    float_range("zones.atkPercent", z.atk_percent, -10.0, 100.0)?;
    float_range("zones.aspd", z.aspd, -500.0, 1_000.0)?;
    float_range("zones.spRecovery", z.sp_recovery, -10.0, 100.0)?;
    for (field, list) in z.lists() {
        if list.len() > MAX_SOURCES {
            return Err(AppError::BadRequest(format!(
                "{field} takes at most {MAX_SOURCES} sources"
            )));
        }
        let (min, max) = match field {
            "zones.inspire" => (-100_000.0, 100_000.0),
            "zones.def.steal" => (0.0, 100_000.0),
            "zones.atkMultiplier" => (0.0, 100.0),
            "zones.damageReduction" => (0.0, 1.0),
            _ => (-10.0, 100.0),
        };
        for v in list {
            float_range(field, Some(*v), min, max)?;
            if field == "zones.atkMultiplier" && *v <= 0.0 {
                return Err(AppError::BadRequest(format!(
                    "{field} must be greater than 0 (got {v})"
                )));
            }
        }
    }
    if z.res.steal.len() > MAX_SOURCES {
        return Err(AppError::BadRequest(format!(
            "zones.res.steal takes at most {MAX_SOURCES} sources"
        )));
    }
    for v in &z.res.steal {
        float_range("zones.res.steal", Some(*v), 0.0, 10_000.0)?;
    }
    validate_reduction("zones.def", &z.def, 100_000.0)?;
    validate_reduction("zones.res", &z.res, 10_000.0)
}

fn validate_reduction(prefix: &str, r: &StatReduction, flat_max: f64) -> Result<(), AppError> {
    float_range(&format!("{prefix}.flat"), r.flat, -flat_max, flat_max)?;
    float_range(&format!("{prefix}.percent"), r.percent, -1.0, 1.0)?;
    float_range(
        &format!("{prefix}.ignoreFlat"),
        r.ignore_flat,
        0.0,
        flat_max,
    )?;
    float_range(
        &format!("{prefix}.ignorePercent"),
        r.ignore_percent,
        0.0,
        1.0,
    )
}

fn int_range(field: &str, value: Option<i32>, min: i32, max: i32) -> Result<(), AppError> {
    match value {
        Some(v) if v < min || v > max => Err(AppError::BadRequest(format!(
            "{field} must be between {min} and {max} (got {v})"
        ))),
        _ => Ok(()),
    }
}

fn float_range(field: &str, value: Option<f64>, min: f64, max: f64) -> Result<(), AppError> {
    match value {
        Some(v) if !v.is_finite() => Err(AppError::BadRequest(format!(
            "{field} must be a finite number"
        ))),
        Some(v) if v < min || v > max => Err(AppError::BadRequest(format!(
            "{field} must be between {min} and {max} (got {v})"
        ))),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SweepAxis {
    Def,
    Res,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sweep {
    pub axis: SweepAxis,
    pub min: f64,
    pub max: f64,
    pub steps: u32,
}

impl Sweep {
    #[must_use]
    pub fn points(&self) -> Vec<f64> {
        let n = self.steps.max(1);
        (0..n)
            .map(|i| {
                if n == 1 {
                    self.min
                } else if i == n - 1 {
                    self.max
                } else {
                    self.min + (self.max - self.min) * f64::from(i) / f64::from(n - 1)
                }
            })
            .collect()
    }

    fn validate(&self) -> Result<(), AppError> {
        let max = match self.axis {
            SweepAxis::Def => 100_000.0,
            SweepAxis::Res => 1_000.0,
        };
        float_range("sweep.min", Some(self.min), 0.0, max)?;
        float_range("sweep.max", Some(self.max), 0.0, max)?;
        if self.min > self.max {
            return Err(AppError::bad_request("sweep.min must not exceed sweep.max"));
        }
        if self.steps < 1 || self.steps > MAX_SWEEP_STEPS {
            return Err(AppError::BadRequest(format!(
                "sweep.steps must be between 1 and {MAX_SWEEP_STEPS} (got {})",
                self.steps
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    pub requests: Vec<CalculateRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sweep: Option<Sweep>,
}

impl BatchRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.requests.is_empty() || self.requests.len() > MAX_BATCH {
            return Err(AppError::BadRequest(format!(
                "requests must hold between 1 and {MAX_BATCH} calculations (got {})",
                self.requests.len()
            )));
        }
        for (i, r) in self.requests.iter().enumerate() {
            r.validate().map_err(|e| match e {
                AppError::BadRequest(m) => AppError::BadRequest(format!("requests[{i}]: {m}")),
                other => other,
            })?;
        }
        if let Some(sweep) = &self.sweep {
            sweep.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepPoint {
    pub x: f64,
    #[serde(flatten)]
    pub result: DpsResult,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<DpsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<SweepPoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResponse {
    pub results: Vec<BatchItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HpsBatchItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<HpsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HpsBatchResponse {
    pub results: Vec<HpsBatchItem>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportersQuery {
    pub promotion: Option<i32>,
    pub skill_level: Option<i32>,
    pub potential: Option<i32>,
    pub module_level: Option<i32>,
}
