//! Reconcile stored `gacha_records.rarity` against the latest game data.
//!
//! When a gacha record is fetched from Yostar's API, the canonical rarity is
//! read from `character_table` (via `rarity_from_gamedata`). If game data is
//! missing the `char_id` at fetch time — typically because the operator is
//! brand-new and the asset pipeline hasn't caught up yet — the code falls
//! back to the API's `star` field, which has historically been wrong/stale.
//!
//! Once game data catches up (post hot-reload), those fallback rows can carry
//! the wrong rarity. This module rewrites them. It's cheap: distinct
//! `(char_id, rarity)` pairs are bounded by the operator count (few hundred),
//! and the UPDATE is a single unnest-driven batch.

use std::collections::HashSet;

use sqlx::PgPool;

use crate::core::gamedata::types::GameData;

#[derive(Debug, Clone, Copy, Default)]
pub struct ResyncStats {
    /// Distinct (`char_id`, rarity) pairs observed in `gacha_records`.
    pub distinct_pairs: usize,
    /// `char_ids` whose canonical rarity disagreed with at least one stored row.
    pub fixed_char_ids: usize,
    /// Total `gacha_records` rows mutated by the UPDATE.
    pub rows_updated: u64,
}

pub async fn reconcile_rarities(db: &PgPool, gd: &GameData) -> Result<ResyncStats, sqlx::Error> {
    let pairs: Vec<(String, i16)> =
        sqlx::query_as("SELECT DISTINCT char_id, rarity FROM gacha_records")
            .fetch_all(db)
            .await?;

    let distinct_pairs = pairs.len();

    // One canonical rarity per char_id. Multiple stored rarities for the same
    // char only need a single fix entry — the UPDATE rewrites every row whose
    // rarity disagrees.
    let mut seen: HashSet<String> = HashSet::new();
    let mut char_ids: Vec<String> = Vec::new();
    let mut rarities: Vec<i16> = Vec::new();
    for (char_id, stored) in pairs {
        let Some(canonical) = gd.operators.get(&char_id).map(|op| op.rarity.to_star_int()) else {
            continue;
        };
        if canonical != stored && seen.insert(char_id.clone()) {
            char_ids.push(char_id);
            rarities.push(canonical);
        }
    }

    let rows_updated = if char_ids.is_empty() {
        0
    } else {
        sqlx::query(
            "UPDATE gacha_records gr \
             SET rarity = u.rarity \
             FROM unnest($1::TEXT[], $2::SMALLINT[]) AS u(char_id, rarity) \
             WHERE gr.char_id = u.char_id AND gr.rarity <> u.rarity",
        )
        .bind(&char_ids)
        .bind(&rarities)
        .execute(db)
        .await?
        .rows_affected()
    };

    Ok(ResyncStats {
        distinct_pairs,
        fixed_char_ids: char_ids.len(),
        rows_updated,
    })
}
