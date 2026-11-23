use std::collections::HashMap;

use myrtle_db::repos::items::ItemRow;
use myrtle_db::repos::{items, progress, roster, users};
use myrtle_engine_base::BaseContext;
use myrtle_engine_base::assignment::compute_live_assignment;
use myrtle_engine_base::rotation::sustain_sim::synced_live_morale;
use myrtle_engine_base::scoring::yield_model::BaseFlows;
use myrtle_engine_base::types::UserBuilding;
use myrtle_engine_planner::{EXP_ITEM, LMD_ITEM, calculate_leveling_costs, module_phase_to_int};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::ModuleType;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::roster::RosterEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::policy::Subject;
use crate::privacy::{
    self,
    layout::{ProfileHeaderField, ProfileTabId},
};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const FARMING_STAGE_CODE: &str = "CE-6";
const MINUTES_PER_DAY: f64 = 1440.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LevelTarget {
    #[default]
    Max,
    Module,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaxLevelOperatorDto {
    pub operator_id: String,
    pub name: String,
    pub rarity: i16,
    pub elite: i16,
    pub level: i16,
    pub target_elite: i16,
    pub target_level: i16,
    pub exp: i64,
    pub level_lmd: i64,
    pub promotion_lmd: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaxLevelCostResponse {
    pub operators_total: usize,
    pub operators_remaining: usize,
    pub exp_needed: i64,
    pub level_lmd_needed: i64,
    pub promotion_lmd_needed: i64,
    pub lmd_needed: i64,
    pub exp_owned: i64,
    pub lmd_owned: Option<i64>,
    pub exp_missing: i64,
    pub lmd_missing: Option<i64>,
    pub operators: Vec<MaxLevelOperatorDto>,
    pub income: LmdIncomeDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct LmdIncomeDto {
    pub base_per_day: Option<f64>,
    pub dailies_per_day: f64,
    pub farming_stage: String,
    pub farming_runs_per_day: f64,
    pub farming_per_day: f64,
    pub days_without_farming: Option<i64>,
    pub days_with_farming: Option<i64>,
}

fn module_unlock(operator: &Operator) -> impl Iterator<Item = (i16, i16)> + '_ {
    operator.modules.iter().filter_map(|m| {
        if m.module.module_type == ModuleType::Initial {
            return None;
        }
        Some((
            module_phase_to_int(&m.module.unlock_evolve_phase),
            i16::try_from(m.module.unlock_level).ok()?,
        ))
    })
}

#[must_use]
pub fn module_targets_by_rarity(model: &GameModel) -> HashMap<i16, (i16, i16)> {
    let mut by_rarity: HashMap<i16, (i16, i16)> = HashMap::new();
    for operator in model.operators.values() {
        let Some(earliest) = module_unlock(operator).min() else {
            continue;
        };
        by_rarity
            .entry(operator.rarity.to_star_int())
            .and_modify(|current| *current = (*current).min(earliest))
            .or_insert(earliest);
    }
    by_rarity
}

#[must_use]
pub fn operator_target(
    operator: &Operator,
    target: LevelTarget,
    by_rarity: &HashMap<i16, (i16, i16)>,
) -> Option<(i16, i16)> {
    let last = operator.phases.last()?;
    let cap = (
        i16::try_from(operator.phases.len()).ok()? - 1,
        i16::try_from(last.max_level).ok()?,
    );
    if target == LevelTarget::Max {
        return Some(cap);
    }
    let module = module_unlock(operator)
        .min()
        .or_else(|| by_rarity.get(&operator.rarity.to_star_int()).copied());
    Some(module.map_or(cap, |m| m.min(cap)))
}

pub async fn max_level_cost(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    target: LevelTarget,
) -> Result<MaxLevelCostResponse, AppError> {
    let (user, visitor_layout) =
        privacy::resolve_user_id_as_viewer(ctx, subject, uid, ProfileTabId::Optimizer).await?;
    let hide_lmd = visitor_layout
        .as_ref()
        .is_some_and(|layout| !layout.header_shows(ProfileHeaderField::Lmd));
    let mut conn = ctx.db.conn().await?;
    let profile = users::profile(&mut conn, user)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let roster = roster::roster(&mut conn, user).await?;
    let inventory = items::inventory(&mut conn, user).await?;
    let building = progress::building(&mut conn, user).await?;
    drop(conn);

    let snapshot = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    let lmd_owned = (!hide_lmd).then(|| profile.lmd.map_or(0, i64::from));
    let now = chrono::Utc::now().timestamp();
    ctx.cpu
        .offload("max_level_income", move || {
            costs(
                &snapshot,
                &roster,
                &inventory,
                building.as_ref(),
                lmd_owned,
                target,
                now,
            )
        })
        .await
}

#[must_use]
pub fn costs(
    snapshot: &Snapshot,
    roster: &[RosterEntry],
    inventory: &[ItemRow],
    building: Option<&Value>,
    lmd_owned: Option<i64>,
    target: LevelTarget,
    now: i64,
) -> MaxLevelCostResponse {
    let model = &snapshot.model;
    let exp_owned: i64 = model
        .materials
        .exp_items
        .values()
        .map(|exp_item| {
            let qty = inventory
                .iter()
                .find(|i| i.item_id == exp_item.id)
                .map_or(0, |i| i.quantity);
            i64::from(qty) * i64::from(exp_item.gain_exp)
        })
        .sum();

    let by_rarity = match target {
        LevelTarget::Max => HashMap::new(),
        LevelTarget::Module => module_targets_by_rarity(model),
    };
    let mut operators_total = 0usize;
    let mut operators: Vec<MaxLevelOperatorDto> = Vec::new();
    for entry in roster {
        let Some(operator) = model.operators.get(&entry.operator_id) else {
            continue;
        };
        let Some((target_elite, target_level)) = operator_target(operator, target, &by_rarity)
        else {
            continue;
        };
        operators_total += 1;
        if entry.elite >= target_elite && entry.level >= target_level {
            continue;
        }
        let mut materials = HashMap::new();
        calculate_leveling_costs(
            operator,
            model,
            entry.elite,
            entry.level,
            target_elite,
            target_level,
            &mut materials,
        );
        let rarity = operator.rarity.to_star_int();
        let promotion_lmd: i64 = ((entry.elite + 1)..=target_elite) // evolveGoldCost[rarity - 1].values[promotion - 1]; -1 marks a missing one
            .filter_map(|elite| {
                model
                    .consts
                    .evolve_gold_cost
                    .get(usize::try_from(rarity - 1).ok()?)?
                    .values
                    .get(usize::try_from(elite - 1).ok()?)
                    .copied()
                    .filter(|cost| *cost > 0)
            })
            .map(i64::from)
            .sum();
        operators.push(MaxLevelOperatorDto {
            operator_id: entry.operator_id.clone(),
            name: operator.name.clone(),
            rarity,
            elite: entry.elite,
            level: entry.level,
            target_elite,
            target_level,
            exp: i64::from(materials.get(EXP_ITEM).copied().unwrap_or(0)),
            level_lmd: i64::from(materials.get(LMD_ITEM).copied().unwrap_or(0)),
            promotion_lmd,
        });
    }
    operators.sort_by(|a, b| {
        (b.exp + b.level_lmd + b.promotion_lmd)
            .cmp(&(a.exp + a.level_lmd + a.promotion_lmd))
            .then_with(|| a.name.cmp(&b.name))
    });

    let exp_needed: i64 = operators.iter().map(|o| o.exp).sum();
    let level_lmd_needed: i64 = operators.iter().map(|o| o.level_lmd).sum();
    let promotion_lmd_needed: i64 = operators.iter().map(|o| o.promotion_lmd).sum();
    let lmd_needed = level_lmd_needed + promotion_lmd_needed;
    let lmd_missing = lmd_owned.map(|owned| (lmd_needed - owned).max(0));
    let income = lmd_income(snapshot, roster, building, lmd_missing.unwrap_or(0), now);
    MaxLevelCostResponse {
        operators_total,
        operators_remaining: operators.len(),
        exp_needed,
        level_lmd_needed,
        promotion_lmd_needed,
        lmd_needed,
        exp_owned,
        lmd_owned,
        exp_missing: (exp_needed - exp_owned).max(0),
        lmd_missing,
        operators,
        income,
    }
}

fn lmd_income(
    snapshot: &Snapshot,
    roster: &[RosterEntry],
    building: Option<&Value>,
    lmd_missing: i64,
    now: i64,
) -> LmdIncomeDto {
    let model = &snapshot.model;
    let base_per_day = building.and_then(|json| current_base_lmd_per_day(snapshot, roster, json));
    let dailies_per_day = model.missions.daily_lmd_per_day(now).unwrap_or(0.0)
        + model.missions.weekly_lmd_per_day(now).unwrap_or(0.0);
    let (farming_runs_per_day, farming_per_day) = farming_income(model);
    let without = base_per_day.unwrap_or(0.0) + dailies_per_day;
    LmdIncomeDto {
        base_per_day,
        dailies_per_day,
        farming_stage: FARMING_STAGE_CODE.to_owned(),
        farming_runs_per_day,
        farming_per_day,
        days_without_farming: days_to_earn(lmd_missing, without),
        days_with_farming: days_to_earn(lmd_missing, without + farming_per_day),
    }
}

fn current_base_lmd_per_day(
    snapshot: &Snapshot,
    roster: &[RosterEntry],
    building_json: &Value,
) -> Option<f64> {
    let building = UserBuilding::from_json(building_json);
    if building.is_empty() {
        return None;
    }
    let BaseContext {
        profiles,
        registry,
        morale_drains,
    } = BaseContext::build(roster, &snapshot.base, false);
    let current = compute_live_assignment(
        &profiles,
        &building,
        snapshot.base.building(),
        &registry,
        &morale_drains,
        None,
        &synced_live_morale(building_json),
    );
    let mut flows = BaseFlows::default();
    for r in &current.rooms {
        flows.add_room(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            r.total_efficiency,
            r.order_gold,
            r.order_value,
            r.operators.len(),
            r.order_limit,
        );
    }
    Some(flows.lmd_per_day())
}

fn farming_income(model: &GameModel) -> (f64, f64) {
    let regen = model.consts.player_ap_regen_speed;
    let Some(stage) = model
        .stages
        .values()
        .find(|s| s.code == FARMING_STAGE_CODE && !s.stage_id.contains('#'))
    else {
        return (0.0, 0.0);
    };
    if regen <= 0 || stage.ap_cost <= 0 {
        return (0.0, 0.0);
    }
    let sanity_per_day = MINUTES_PER_DAY / regen as f64;
    let runs = sanity_per_day / f64::from(stage.ap_cost);
    (runs, runs * f64::from(stage.gold_gain))
}

#[must_use]
pub fn days_to_earn(missing: i64, per_day: f64) -> Option<i64> {
    if missing <= 0 {
        return Some(0);
    }
    if per_day <= 0.0 {
        return None;
    }
    Some((missing as f64 / per_day).ceil() as i64)
}
