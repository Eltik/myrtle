//! The account-wide "what does it take to max every operator" figure: the
//! EXP and LMD still needed to bring each owned operator that is not yet at
//! its target up to it, against what the account holds. The target is the
//! level cap by default, or the level the operator's modules unlock at
//! (`LevelTarget::Module`): the last thirty levels of a 6-star are 43.0% of
//! its LMD and 47.1% of its EXP for a few points of stat, and a player who
//! stops where the module opens wants that priced, not the cap.
//!
//! Costs come from the same tables the operator planner uses for a single
//! plan (`calculate_leveling_costs`: the per-level EXP and LMD maps by
//! promotion), plus the promotion LMD from gamedata's `evolveGoldCost`
//! (rarity x promotion), which the operator table's `evolveCost` does not
//! carry as an item. EXP owned is the sum of the account's EXP cards; LMD
//! owned is the synced balance.

use std::sync::Arc;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::services::planner::{calculate_leveling_costs, module_phase_to_int};
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::ModuleType;
use crate::core::gamedata::types::operator::Operator;
use crate::core::grade::base::assignment::compute_live_assignment;
use crate::core::grade::base::context::BaseContext;
use crate::core::grade::base::sustain_sim::synced_live_morale;
use crate::core::grade::base::types::UserBuilding;
use crate::core::grade::base::yield_model::BaseFlows;
use crate::database::models::roster::RosterEntry;
use crate::database::queries::building::get_building;
use crate::database::queries::{
    items as items_queries, roster as roster_queries, users as users_queries,
};

/// Where the walk stops for each operator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LevelTarget {
    /// The final promotion and its level cap.
    #[default]
    Max,
    /// The promotion and level the operator's modules unlock at. An operator
    /// without a module takes its rarity's module level (every module of a
    /// rarity unlocks at one point: 4-star E2 40, 5-star E2 50, 6-star
    /// E2 60, read from the module table, never assumed); a rarity with no
    /// modules at all (1 to 3 stars) keeps its cap.
    Module,
}

/// `(elite, level)` a module unlocks at, `None` for the initial badge every
/// operator holds from E0 1.
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

/// The earliest module unlock per rarity (star count), over every operator
/// the game data knows.
pub fn module_targets_by_rarity(gamedata: &GameData) -> HashMap<i16, (i16, i16)> {
    let mut by_rarity: HashMap<i16, (i16, i16)> = HashMap::new();
    for operator in gamedata.operators.values() {
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

/// `(elite, level)` the walk stops at for one operator: its own earliest
/// module, else its rarity's, else the cap. Never past the cap.
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

/// One operator still short of its target, with what closing the gap costs.
#[derive(Debug, Clone, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MaxLevelOperatorDto {
    pub operator_id: String,
    pub name: String,
    pub rarity: i16,
    pub elite: i16,
    pub level: i16,
    pub target_elite: i16,
    pub target_level: i16,
    /// EXP for the remaining levels, across every promotion on the way.
    #[ts(type = "number")]
    pub exp: i64,
    /// LMD for the remaining levels.
    #[ts(type = "number")]
    pub level_lmd: i64,
    /// LMD for the remaining promotions.
    #[ts(type = "number")]
    pub promotion_lmd: i64,
}

#[derive(Debug, Clone, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MaxLevelCostResponse {
    /// Owned operators the game data knows.
    pub operators_total: usize,
    /// Owned operators not yet at their target.
    pub operators_remaining: usize,
    #[ts(type = "number")]
    pub exp_needed: i64,
    #[ts(type = "number")]
    pub level_lmd_needed: i64,
    #[ts(type = "number")]
    pub promotion_lmd_needed: i64,
    /// `level_lmd_needed + promotion_lmd_needed`.
    #[ts(type = "number")]
    pub lmd_needed: i64,
    /// EXP held as cards (every EXP item times its EXP value).
    #[ts(type = "number")]
    pub exp_owned: i64,
    /// The synced LMD balance; 0 when the profile carries none.
    #[ts(type = "number")]
    pub lmd_owned: i64,
    /// Needed minus owned, floored at 0.
    #[ts(type = "number")]
    pub exp_missing: i64,
    #[ts(type = "number")]
    pub lmd_missing: i64,
    /// Every remaining operator, most expensive first.
    pub operators: Vec<MaxLevelOperatorDto>,
    /// What the account earns in a day and how long the LMD shortfall takes.
    pub income: LmdIncomeDto,
}

/// LMD the account earns in a day, and the days until `lmd_missing` is
/// covered - once from the base and mission chests alone, once with every
/// day's natural sanity spent on the LMD farming stage.
#[derive(Debug, Clone, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct LmdIncomeDto {
    /// The synced base's realized LMD per day as stationed right now (the
    /// Score tab's current figure). `None` without a synced base.
    pub base_per_day: Option<f64>,
    /// The live daily mission chests (weekday and weekend groups weighted by
    /// their days) plus the live weekly chests over seven days, LMD only,
    /// every chest claimed.
    pub dailies_per_day: f64,
    /// The stage the farming estimate runs (the LMD stage, CE-6).
    pub farming_stage: String,
    /// Runs of that stage a day's natural sanity regeneration buys.
    pub farming_runs_per_day: f64,
    /// LMD those runs pay.
    pub farming_per_day: f64,
    /// Days until the shortfall is earned from the base and dailies: 0 when
    /// nothing is missing, `None` when nothing is earned.
    #[ts(type = "number | null")]
    pub days_without_farming: Option<i64>,
    /// The same with the farming stage's LMD added.
    #[ts(type = "number | null")]
    pub days_with_farming: Option<i64>,
}

/// The LMD stage the farming estimate spends sanity on.
const FARMING_STAGE_CODE: &str = "CE-6";
/// Minutes in a day, for sanity regeneration.
const MINUTES_PER_DAY: f64 = 1440.0;

/// LMD item id in the planner's material map.
const LMD_ITEM: &str = "4001";
/// EXP pseudo-item id in the planner's material map.
const EXP_ITEM: &str = "5001";

pub async fn max_level_costs(
    state: &AppState,
    uid: &str,
    target: LevelTarget,
) -> Result<MaxLevelCostResponse, ApiError> {
    let user = users_queries::find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;
    let roster = roster_queries::get_roster(&state.db, user.id).await?;
    let inventory = items_queries::get_inventory(&state.db, user.id).await?;
    let lmd_owned = users_queries::find_by_id(&state.db, user.id)
        .await?
        .and_then(|p| p.lmd)
        .map_or(0, i64::from);
    let gamedata = state.default_game_data();

    let exp_owned: i64 = gamedata
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
        LevelTarget::Module => module_targets_by_rarity(&gamedata),
    };
    let mut operators_total = 0usize;
    let mut operators: Vec<MaxLevelOperatorDto> = Vec::new();
    for entry in &roster {
        let Some(operator) = gamedata.operators.get(&entry.operator_id) else {
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
        let mut materials = std::collections::HashMap::new();
        calculate_leveling_costs(
            operator,
            &gamedata,
            entry.elite,
            entry.level,
            target_elite,
            target_level,
            &mut materials,
        );
        let exp = i64::from(materials.get(EXP_ITEM).copied().unwrap_or(0));
        let level_lmd = i64::from(materials.get(LMD_ITEM).copied().unwrap_or(0));
        // `evolveGoldCost[rarity - 1].values[promotion - 1]`; -1 marks a
        // promotion the rarity does not have.
        let rarity = operator.rarity.to_star_int();
        let promotion_lmd: i64 = ((entry.elite + 1)..=target_elite)
            .filter_map(|elite| {
                gamedata
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
            exp,
            level_lmd,
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
    let lmd_missing = (lmd_needed - lmd_owned).max(0);
    let income = lmd_income(state, user.id, &roster, &gamedata, lmd_missing).await?;
    Ok(MaxLevelCostResponse {
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
    })
}

/// What the account earns in a day and how long `lmd_missing` takes.
async fn lmd_income(
    state: &AppState,
    user_id: uuid::Uuid,
    roster: &[RosterEntry],
    gamedata: &Arc<GameData>,
    lmd_missing: i64,
) -> Result<LmdIncomeDto, ApiError> {
    // The base as stationed now, priced the way the Score tab prices it:
    // a live assignment and the coupled gold-to-LMD flow. It is a real
    // computation (0.2 s), so it runs on the blocking pool like every other
    // base scorer.
    let base_per_day = match get_building(&state.db, user_id).await? {
        Some(json) => {
            let roster = roster.to_vec();
            let gd = Arc::clone(gamedata);
            cpu::offload("max_level_income", move || {
                current_base_lmd_per_day(&roster, &gd, &json)
            })
            .await?
        }
        None => None,
    };
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    )
    .unwrap_or(0);
    let dailies_per_day = gamedata.missions.daily_lmd_per_day(now).unwrap_or(0.0)
        + gamedata.missions.weekly_lmd_per_day(now).unwrap_or(0.0);
    let (farming_runs_per_day, farming_per_day) = farming_income(gamedata);
    let without = base_per_day.unwrap_or(0.0) + dailies_per_day;
    Ok(LmdIncomeDto {
        base_per_day,
        dailies_per_day,
        farming_stage: FARMING_STAGE_CODE.to_string(),
        farming_runs_per_day,
        farming_per_day,
        days_without_farming: days_to_earn(lmd_missing, without),
        days_with_farming: days_to_earn(lmd_missing, without + farming_per_day),
    })
}

/// The synced base's realized LMD per day as stationed now; `None` when the
/// sync carries no rooms.
fn current_base_lmd_per_day(
    roster: &[RosterEntry],
    gamedata: &GameData,
    building_json: &serde_json::Value,
) -> Option<f64> {
    let building = UserBuilding::from_json(building_json);
    if building.is_empty() {
        return None;
    }
    let BaseContext {
        profiles,
        registry,
        morale_drains,
    } = BaseContext::build(roster, gamedata, false);
    let live_morale = synced_live_morale(building_json);
    let current = compute_live_assignment(
        &profiles,
        &building,
        &gamedata.building,
        &registry,
        &morale_drains,
        None,
        &live_morale,
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
    Some(flows.realized_lmd())
}

/// `(runs per day, LMD per day)` from spending a day's natural sanity on
/// the farming stage: sanity a day is the minutes in a day over the game's
/// regeneration interval; a run costs the stage's sanity and pays its LMD.
/// Zero when the stage or the interval is missing from the game data.
fn farming_income(gamedata: &GameData) -> (f64, f64) {
    let regen = gamedata.consts.player_ap_regen_speed;
    let Some(stage) = gamedata
        .stages
        .values()
        .find(|s| s.code == FARMING_STAGE_CODE && !s.stage_id.contains('#'))
    else {
        return (0.0, 0.0);
    };
    if regen <= 0 || stage.ap_cost <= 0 {
        return (0.0, 0.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let sanity_per_day = MINUTES_PER_DAY / regen as f64;
    let runs = sanity_per_day / f64::from(stage.ap_cost);
    (runs, runs * f64::from(stage.gold_gain))
}

/// Whole days until `missing` LMD is earned at `per_day`: 0 when nothing is
/// missing, `None` when nothing is earned.
fn days_to_earn(missing: i64, per_day: f64) -> Option<i64> {
    if missing <= 0 {
        return Some(0);
    }
    if per_day <= 0.0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    Some((missing as f64 / per_day).ceil() as i64)
}

#[cfg(test)]
mod tests {
    use super::days_to_earn;

    #[test]
    fn days_round_up_and_handle_the_edges() {
        assert_eq!(days_to_earn(0, 100.0), Some(0));
        assert_eq!(days_to_earn(-5, 0.0), Some(0));
        assert_eq!(days_to_earn(250, 100.0), Some(3));
        assert_eq!(days_to_earn(300, 100.0), Some(3));
        assert_eq!(days_to_earn(1, 0.0), None);
    }
}
