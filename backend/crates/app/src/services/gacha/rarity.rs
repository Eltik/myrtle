use std::collections::{HashMap, HashSet};

use myrtle_db::repos::gacha::{self, CharRarityRow};
use myrtle_gamedata_raw::tables::operator::Operator;

use super::stats::flush_shared_stats;
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RaritySync {
    pub distinct_pairs: usize,
    pub fixed_char_ids: usize,
    pub rows_updated: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RarityFix {
    pub char_id: String,
    pub from: i16,
    pub to: i16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RarityPlan {
    pub distinct_pairs: usize,
    pub fixes: Vec<RarityFix>,
}

#[must_use]
pub fn rarity_fixes(
    pairs: &[CharRarityRow],
    operators: &HashMap<String, Operator>,
) -> Vec<RarityFix> {
    let mut seen = HashSet::new();
    let mut fixes = Vec::new();
    for pair in pairs {
        let Some(canonical) = operators
            .get(&pair.char_id)
            .map(|op| op.rarity.to_star_int())
        else {
            continue;
        };
        if canonical != pair.rarity && seen.insert(pair.char_id.as_str()) {
            fixes.push(RarityFix {
                char_id: pair.char_id.clone(),
                from: pair.rarity,
                to: canonical,
            });
        }
    }
    fixes
}

pub async fn plan_against(
    ctx: &AppContext,
    operators: &HashMap<String, Operator>,
) -> Result<RarityPlan, AppError> {
    let pairs = gacha::distinct_char_rarities(&mut *ctx.db.conn().await?).await?;
    Ok(RarityPlan {
        distinct_pairs: pairs.len(),
        fixes: rarity_fixes(&pairs, operators),
    })
}

pub async fn reconcile_rarities(ctx: &AppContext, generation: i64) -> Result<RaritySync, AppError> {
    let default = ctx.snapshots.default_server();
    if ctx.snapshots.get(default).is_none() {
        return Err(AppError::Unavailable);
    }
    let snapshot = ctx.snapshot_at(default, generation).await?;
    reconcile_against(ctx, &snapshot.model.operators).await
}

pub async fn reconcile_against(
    ctx: &AppContext,
    operators: &HashMap<String, Operator>,
) -> Result<RaritySync, AppError> {
    let plan = plan_against(ctx, operators).await?;
    let (char_ids, rarities): (Vec<String>, Vec<i16>) = plan
        .fixes
        .iter()
        .map(|fix| (fix.char_id.clone(), fix.to))
        .unzip();
    let rows_updated = if char_ids.is_empty() {
        0
    } else {
        gacha::correct_rarities(&mut *ctx.db.conn().await?, &char_ids, &rarities).await?
    };
    if rows_updated > 0 {
        tracing::info!(
            rows = rows_updated,
            char_ids = char_ids.len(),
            "gacha rarities reconciled"
        );
        flush_shared_stats(ctx).await;
    }
    Ok(RaritySync {
        distinct_pairs: plan.distinct_pairs,
        fixed_char_ids: char_ids.len(),
        rows_updated,
    })
}
