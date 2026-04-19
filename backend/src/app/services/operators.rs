use serde::{Deserialize, Serialize};

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::operator::{OperatorPosition, OperatorProfession, OperatorRarity};

/// Compact operator record for client-side search palettes and autocompletes.
/// Order of fields matches the JSON contract — keep stable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexEntry {
    pub id: String,
    pub name: String,
    pub appellation: String,
    /// 1–6. Converts from the game's TIER_N enum so the frontend doesn't have to.
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub sub_profession_id: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub nation_id: String,
    pub is_not_obtainable: bool,
}

pub async fn get_index(state: &AppState) -> Result<Vec<OperatorIndexEntry>, ApiError> {
    let key = CacheKey::StaticData {
        resource: "operators_index",
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Vec<OperatorIndexEntry>>(&key).await {
        return Ok(cached);
    }

    let gd = state.game_data.load();
    let mut entries: Vec<OperatorIndexEntry> = gd
        .operators
        .iter()
        .filter_map(|(id, op)| {
            let operator_id = op.id.clone().unwrap_or_else(|| id.clone());
            Some(OperatorIndexEntry {
                id: operator_id,
                name: op.name.clone(),
                appellation: op.appellation.clone(),
                rarity: rarity_to_stars(&op.rarity),
                profession: op.profession.clone(),
                sub_profession_id: op.sub_profession_id.clone(),
                position: op.position.clone(),
                tag_list: op.tag_list.clone(),
                nation_id: op.nation_id.clone(),
                is_not_obtainable: op.is_not_obtainable,
            })
        })
        .collect();
    entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then_with(|| a.name.cmp(&b.name)));

    state.cache.set(&key, &entries).await;
    Ok(entries)
}

const fn rarity_to_stars(rarity: &OperatorRarity) -> u8 {
    match rarity {
        OperatorRarity::SixStar => 6,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::FourStar => 4,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::OneStar => 1,
    }
}
