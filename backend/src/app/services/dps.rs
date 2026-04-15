use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::dps::engine::{self, DpsResult};
use crate::dps::operator_unit::{
    EnemyStats, OperatorBuffs, OperatorConditionals, OperatorParams, OperatorShred,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInfo {
    pub conditional_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorListEntry {
    pub id: String,
    pub name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    pub default_skill: i32,
    pub default_module: i32,
    pub conditionals: Vec<ConditionalInfo>,
}

pub fn list_operators() -> Vec<OperatorListEntry> {
    engine::supported_operators()
        .iter()
        .map(|(id, formula)| OperatorListEntry {
            id: id.clone(),
            name: formula.name.clone(),
            available_skills: formula.available_skills.clone(),
            available_modules: formula.available_modules.clone(),
            default_skill: formula.default_skill,
            default_module: formula.default_module,
            conditionals: formula
                .conditionals
                .iter()
                .map(|c| ConditionalInfo {
                    conditional_type: c.cond_type.clone(),
                    name: c.name.clone(),
                    default: c.default,
                    skills: c.skills.clone(),
                    modules: c.modules.clone(),
                })
                .collect(),
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestConditionals {
    pub trait_damage: Option<bool>,
    pub talent_damage: Option<bool>,
    pub talent2_damage: Option<bool>,
    pub skill_damage: Option<bool>,
    pub module_damage: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateRequest {
    pub operator_id: String,
    // Operator config
    pub promotion: Option<i32>,
    pub level: Option<i32>,
    pub potential: Option<i32>,
    pub trust: Option<i32>,
    pub skill_index: Option<i32>,
    pub mastery_level: Option<i32>,
    pub module_index: Option<i32>,
    pub module_level: Option<i32>,
    // Enemy
    pub defense: Option<f64>,
    pub res: Option<f64>,
    // Buffs
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    // Targets
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    // Conditionals
    pub conditionals: Option<RequestConditionals>,
    pub all_cond: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
    pub aspd: Option<i32>,
    pub fragile: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestShred {
    pub def: Option<i32>,
    pub def_flat: Option<i32>,
    pub res: Option<i32>,
    pub res_flat: Option<i32>,
}

pub fn calculate(state: &AppState, req: CalculateRequest) -> Result<DpsResult, ApiError> {
    let gd = state.game_data.load();
    let operator = gd
        .operators
        .get(&req.operator_id)
        .ok_or(ApiError::NotFound)?;

    let params = OperatorParams {
        promotion: req.promotion,
        level: req.level,
        potential: req.potential,
        trust: req.trust.or(Some(100)),
        skill_index: req.skill_index,
        mastery_level: req.mastery_level,
        module_index: req.module_index,
        module_level: req.module_level,
        buffs: req
            .buffs
            .map(|b| OperatorBuffs {
                atk: b.atk,
                flat_atk: b.flat_atk,
                aspd: b.aspd,
                fragile: b.fragile,
            })
            .unwrap_or_default(),
        sp_boost: req.sp_boost,
        targets: req.targets,
        shred: req.shred.map(|s| OperatorShred {
            def: s.def,
            def_flat: s.def_flat,
            res: s.res,
            res_flat: s.res_flat,
        }),
        conditionals: req.conditionals.map(|c| OperatorConditionals {
            trait_damage: c.trait_damage,
            talent_damage: c.talent_damage,
            talent2_damage: c.talent2_damage,
            skill_damage: c.skill_damage,
            module_damage: c.module_damage,
        }),
        all_cond: req.all_cond,
        ..Default::default()
    };

    let enemy = EnemyStats {
        defense: req.defense.unwrap_or(0.0),
        res: req.res.unwrap_or(0.0),
    };

    engine::calculate_dps(operator, params, &enemy).ok_or(ApiError::BadRequest(
        "DPS calculation failed for this operator/config".into(),
    ))
}
