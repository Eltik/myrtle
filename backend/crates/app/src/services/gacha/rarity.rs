use std::collections::{HashMap, HashSet};

use myrtle_db::repos::gacha;
use myrtle_gamedata_raw::tables::operator::Operator;

use super::stats::flush_shared_stats;
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RaritySync {
    pub distinct_pairs: usize,
    pub fixed_char_ids: usize,
    pub rows_updated: u64,
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
    let mut conn = ctx.db.conn().await?;
    let pairs = gacha::distinct_char_rarities(&mut conn).await?;
    let mut seen = HashSet::new();
    let (mut char_ids, mut rarities) = (Vec::new(), Vec::new());
    for pair in &pairs {
        let Some(canonical) = operators
            .get(&pair.char_id)
            .map(|op| op.rarity.to_star_int())
        else {
            continue;
        };
        if canonical != pair.rarity && seen.insert(pair.char_id.as_str()) {
            char_ids.push(pair.char_id.clone());
            rarities.push(canonical);
        }
    }
    let rows_updated = if char_ids.is_empty() {
        0
    } else {
        gacha::correct_rarities(&mut conn, &char_ids, &rarities).await?
    };
    drop(conn);
    if rows_updated > 0 {
        tracing::info!(
            rows = rows_updated,
            char_ids = char_ids.len(),
            "gacha rarities reconciled"
        );
        flush_shared_stats(ctx).await;
    }
    Ok(RaritySync {
        distinct_pairs: pairs.len(),
        fixed_char_ids: char_ids.len(),
        rows_updated,
    })
}
