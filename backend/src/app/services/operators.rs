use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::operator::{
    Operator, OperatorPosition, OperatorProfession, OperatorRarity,
};
use crate::core::gamedata::types::skin::SkinData;
use crate::core::gamedata::types::voice::Voices;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::operator_ownership;

/// Compact operator record for client-side search palettes and autocompletes.
/// Order of fields matches the JSON contract - keep stable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorIndexEntry {
    pub id: String,
    pub name: String,
    pub appellation: String,
    /// 1-6. Converts from the game's `TIER_N` enum so the frontend doesn't have to.
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub sub_profession_id: String,
    pub position: OperatorPosition,
    pub tag_list: Vec<String>,
    pub nation_id: String,
    pub is_not_obtainable: bool,
}

fn to_index_entry(id: &str, op: &Operator) -> OperatorIndexEntry {
    OperatorIndexEntry {
        id: op.id.clone().unwrap_or_else(|| id.to_string()),
        name: op.name.clone(),
        appellation: op.appellation.clone(),
        rarity: rarity_to_stars(&op.rarity),
        profession: op.profession.clone(),
        sub_profession_id: op.sub_profession_id.clone(),
        position: op.position.clone(),
        tag_list: op.tag_list.clone(),
        nation_id: op.nation_id.clone(),
        is_not_obtainable: op.is_not_obtainable,
    }
}

pub async fn get_index(
    state: &AppState,
    server: Server,
) -> Result<Vec<OperatorIndexEntry>, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource: "operators_index",
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Vec<OperatorIndexEntry>>(&key).await {
        return Ok(cached);
    }

    let gd = server_data.game_data.load_full();
    let mut entries: Vec<OperatorIndexEntry> = gd
        .operators
        .iter()
        .map(|(id, op)| to_index_entry(id, op))
        .collect();
    entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then_with(|| a.name.cmp(&b.name)));

    state.cache.set(&key, &entries).await;
    Ok(entries)
}

/// Population-level ownership: how many sharing players own each operator, plus
/// the denominator. Only operators with at least one owner are listed; a missing
/// id implies zero owners. `totalUsers` is the eligible population on this
/// server (players who imported a roster and opted into stat sharing).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorOwnershipResponse {
    pub total_users: i64,
    pub counts: HashMap<String, i64>,
    pub computed_at: String,
}

/// Served from the precomputed aggregate via a short-lived cache, so a request
/// never scans the roster. Returns 404 for a server that is not loaded.
pub async fn get_ownership(
    state: &AppState,
    server: Server,
) -> Result<OperatorOwnershipResponse, ApiError> {
    state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let key = CacheKey::OperatorOwnership {
        server: server.as_str(),
    };
    if let Some(cached) = state.cache.get::<OperatorOwnershipResponse>(&key).await {
        return Ok(cached);
    }

    let server_id = server.index() as i16;
    let (total_users, rows) =
        operator_ownership::get_operator_ownership(&state.db, server_id).await?;
    let counts: HashMap<String, i64> = rows
        .into_iter()
        .map(|r| (r.operator_id, r.owners))
        .collect();

    let response = OperatorOwnershipResponse {
        total_users,
        counts,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(response)
}

/// Operators present on `source` but absent from the default (global/EN) server:
/// the "upcoming operators" preview. Returns 404 when `source` is not a loaded
/// server.
pub async fn get_upcoming(
    state: &AppState,
    source: Server,
) -> Result<Vec<OperatorIndexEntry>, ApiError> {
    let source_data = state.try_server_data(source).ok_or(ApiError::NotFound)?;
    let key = CacheKey::StaticData {
        resource: "upcoming",
        server: source.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Vec<OperatorIndexEntry>>(&key).await {
        return Ok(cached);
    }

    let base = state.default_game_data();
    let src = source_data.game_data.load_full();
    let base_ids: HashSet<&str> = base.operators.keys().map(String::as_str).collect();

    let mut entries: Vec<OperatorIndexEntry> = src
        .operators
        .iter()
        .filter(|(id, _)| !base_ids.contains(id.as_str()))
        .map(|(id, op)| to_index_entry(id, op))
        .collect();
    entries.sort_by(|a, b| b.rarity.cmp(&a.rarity).then_with(|| a.name.cmp(&b.name)));

    state.cache.set(&key, &entries).await;
    Ok(entries)
}

/// One fully-enriched operator by id. Lets the detail page fetch a single
/// operator instead of downloading the whole `/static/operators` table.
pub async fn get_operator(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<Operator, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    gd.operators.get(id).cloned().ok_or(ApiError::NotFound)
}

/// Find an operator across loaded servers, preferring `preferred` (the default).
/// Returns the operator and the server it was found on, so a single request
/// resolves both global and upcoming (CN-only) operators - no client-side retry.
pub fn resolve_operator(
    state: &AppState,
    preferred: Server,
    id: &str,
) -> Option<(Operator, Server)> {
    let mut order = vec![preferred];
    order.extend(
        state
            .config
            .servers
            .iter()
            .copied()
            .filter(|s| *s != preferred),
    );
    for srv in order {
        if let Some(sd) = state.try_server_data(srv)
            && let Some(op) = sd.game_data.load_full().operators.get(id)
        {
            return Some((op.clone(), srv));
        }
    }
    None
}

/// Just one operator's voice lines (+ its voice-lang entry), so the Audio tab
/// fetches KB instead of the whole `/static/voices` table.
pub async fn get_operator_voices(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<Voices, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    Ok(Voices {
        char_words: gd
            .voices
            .char_words
            .iter()
            .filter(|(_, v)| v.char_id.as_str() == id)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        voice_lang_dict: gd
            .voices
            .voice_lang_dict
            .iter()
            .filter(|(_, vl)| vl.char_id.as_str() == id)
            .map(|(k, vl)| (k.clone(), vl.clone()))
            .collect(),
        ..Default::default()
    })
}

/// Just one operator's skins, so the Skins tab fetches KB instead of the whole
/// `/static/skins` table. Matches by `tmplId` (alternate forms) else `charId`.
pub async fn get_operator_skins(
    state: &AppState,
    server: Server,
    id: &str,
) -> Result<SkinData, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    Ok(SkinData {
        char_skins: gd
            .skins
            .char_skins
            .iter()
            .filter(|(_, s)| match &s.tmpl_id {
                Some(t) => t.as_str() == id,
                None => s.char_id.as_str() == id,
            })
            .map(|(k, s)| (k.clone(), s.clone()))
            .collect(),
        ..Default::default()
    })
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
