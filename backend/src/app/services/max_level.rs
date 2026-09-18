//! The account-wide "what does it take to max every operator" figure: the
//! EXP and LMD still needed to bring each owned operator that is not yet at
//! its final promotion and level cap up to it, against what the account
//! holds.
//!
//! Costs come from the same tables the operator planner uses for a single
//! plan (`calculate_leveling_costs`: the per-level EXP and LMD maps by
//! promotion), plus the promotion LMD from gamedata's `evolveGoldCost`
//! (rarity x promotion), which the operator table's `evolveCost` does not
//! carry as an item. EXP owned is the sum of the account's EXP cards; LMD
//! owned is the synced balance.

use serde::Serialize;
use ts_rs::TS;

use crate::app::error::ApiError;
use crate::app::services::planner::calculate_leveling_costs;
use crate::app::state::AppState;
use crate::database::queries::{
    items as items_queries, roster as roster_queries, users as users_queries,
};

/// One operator still short of its cap, with what closing the gap costs.
#[derive(Debug, Clone, Serialize, TS)]
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

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct MaxLevelCostResponse {
    /// Owned operators the game data knows.
    pub operators_total: usize,
    /// Owned operators not yet at their final promotion and level cap.
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
}

/// LMD item id in the planner's material map.
const LMD_ITEM: &str = "4001";
/// EXP pseudo-item id in the planner's material map.
const EXP_ITEM: &str = "5001";

pub async fn max_level_costs(
    state: &AppState,
    uid: &str,
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

    let mut operators_total = 0usize;
    let mut operators: Vec<MaxLevelOperatorDto> = Vec::new();
    for entry in &roster {
        let Some(operator) = gamedata.operators.get(&entry.operator_id) else {
            continue;
        };
        let Some(last) = operator.phases.last() else {
            continue;
        };
        operators_total += 1;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let target_elite = (operator.phases.len() - 1) as i16;
        #[allow(clippy::cast_possible_truncation)]
        let target_level = last.max_level as i16;
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
        lmd_missing: (lmd_needed - lmd_owned).max(0),
        operators,
    })
}
