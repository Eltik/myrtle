use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::services::tier_entity::{self, EntitySummary};
use crate::app::state::AppState;
use crate::core::auth::permissions::{GlobalRole, Permission};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::database::models::tier_list::{
    EntityKind, Tier, TierList, TierListFlair, TierListStats, TierPlacement,
};
use crate::database::queries::tier_lists as queries;
use crate::database::queries::tier_lists::count_by_user;
use crate::database::queries::tier_lists::ensure_stats_row;
use crate::database::queries::tier_lists::find_all_active_limited;
use crate::database::queries::tier_lists::find_by_slug;
use crate::database::queries::tier_lists::get_flair_by_id;
use crate::database::queries::tier_lists::get_flairs_by_ids;
use crate::database::queries::tier_lists::get_placements_for_tiers;
use crate::database::queries::tier_lists::get_stats;
use crate::database::queries::tier_lists::get_stats_for_lists;
use crate::database::queries::tier_lists::get_tiers;
use crate::database::queries::tier_lists::get_tiers_for_lists;
use crate::database::queries::tier_lists::get_user_permission;
use crate::database::queries::tier_lists::update;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use ts_rs::TS;
use uuid::Uuid;

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
pub struct TierListDetail {
    #[serde(flatten)]
    pub list: TierList,
    pub tiers: Vec<TierDetail>,
    pub stats: Option<TierListStats>,
    pub flair: Option<TierListFlair>,
    pub author: Option<TierListAuthor>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListAuthor {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub avatar_id: Option<String>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize)]
pub struct TierDetail {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<PlacementDetail>,
}

/// A stored placement and what it points at in the served game data.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Clone, Serialize, Deserialize)]
pub struct PlacementDetail {
    #[serde(flatten)]
    pub placement: TierPlacement,
    /// `None` when the served game data has no such entity (a CN-only operator
    /// read on EN, an id from a removed kind). The placement is still served,
    /// so the client can show it rather than silently drop it.
    pub entity: Option<EntitySummary>,
}

impl PlacementDetail {
    const fn unresolved(placement: TierPlacement) -> Self {
        Self {
            placement,
            entity: None,
        }
    }
}

/// One tier as a published version stores it: the rows, never the resolved
/// entities, so a snapshot reads in whatever language and data it is served
/// with. Snapshots written before entity kinds existed carry `operator_id` and
/// no kind, which [`TierPlacement`]'s serde attributes read as operators.
#[derive(Serialize, Deserialize)]
pub struct TierSnapshot {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<TierPlacement>,
}

/// Parse a stored `tier_list_versions.snapshot`, old shape or new.
///
/// # Errors
/// When the JSON is not a list of tiers with placements.
pub fn read_snapshot(snapshot: &serde_json::Value) -> Result<Vec<TierSnapshot>, serde_json::Error> {
    Vec::<TierSnapshot>::deserialize(snapshot)
}

/// The snapshot a publish stores for `tiers`.
pub fn snapshot_of(tiers: &[TierDetail]) -> Vec<TierSnapshot> {
    tiers
        .iter()
        .map(|t| TierSnapshot {
            tier: t.tier.clone(),
            placements: t.placements.iter().map(|p| p.placement.clone()).collect(),
        })
        .collect()
}

/// A snapshot in the live detail's shape, resolved against `gd` and `assets`.
pub fn resolve_snapshot(
    snapshot: Vec<TierSnapshot>,
    gd: &GameData,
    assets: &AssetIndex,
) -> Vec<TierDetail> {
    snapshot
        .into_iter()
        .map(|t| TierDetail {
            tier: t.tier,
            placements: t
                .placements
                .into_iter()
                .map(|p| resolve_placement(p, gd, assets))
                .collect(),
        })
        .collect()
}

fn resolve_entity(
    placement: &TierPlacement,
    gd: &GameData,
    assets: &AssetIndex,
) -> Option<EntitySummary> {
    tier_entity::resolve(gd, assets, placement.entity_kind, &placement.entity_id)
}

fn resolve_placement(
    placement: TierPlacement,
    gd: &GameData,
    assets: &AssetIndex,
) -> PlacementDetail {
    let entity = resolve_entity(&placement, gd, assets);
    PlacementDetail { placement, entity }
}

/// Fill every placement's `entity` from `gd`, icons from `assets`.
fn resolve_detail(detail: &mut TierListDetail, gd: &GameData, assets: &AssetIndex) {
    for tier in &mut detail.tiers {
        for p in &mut tier.placements {
            p.entity = resolve_entity(&p.placement, gd, assets);
        }
    }
}

/// Reject a `(kind, id)` that no loaded server's game data (or, for a story
/// sprite, asset extract) knows.
///
/// Any server, not only the default: a CN-locale editor places operators EN has
/// not released, and the list must accept what that editor's pool offers.
///
/// # Errors
/// `400` naming the kind and id.
pub fn validate_entity(state: &AppState, kind: EntityKind, id: &str) -> Result<(), ApiError> {
    let loaded: Vec<_> = state
        .servers
        .values()
        .filter(|sd| sd.loaded.load(std::sync::atomic::Ordering::Acquire))
        .map(|sd| (sd.game_data.load_full(), sd.asset_index.load_full()))
        .collect();
    tier_entity::validate(
        loaded
            .iter()
            .map(|(gd, assets)| (gd.as_ref(), assets.as_ref())),
        kind,
        id,
    )
}

fn assemble_details(
    lists: Vec<TierList>,
    tiers: Vec<Tier>,
    placements: Vec<TierPlacement>,
    stats: Vec<TierListStats>,
    flairs: Vec<TierListFlair>,
    authors: Vec<TierListAuthor>,
    (gd, assets): (&GameData, &AssetIndex),
) -> Vec<TierListDetail> {
    let mut tiers_by_list: HashMap<Uuid, Vec<Tier>> = HashMap::new();
    for tier in tiers {
        tiers_by_list
            .entry(tier.tier_list_id)
            .or_default()
            .push(tier);
    }

    let mut placements_by_tier: HashMap<Uuid, Vec<TierPlacement>> = HashMap::new();
    for placement in placements {
        placements_by_tier
            .entry(placement.tier_id)
            .or_default()
            .push(placement);
    }

    let mut stats_by_list: HashMap<Uuid, TierListStats> =
        stats.into_iter().map(|s| (s.tier_list_id, s)).collect();
    let flairs_by_id: HashMap<i16, TierListFlair> = flairs.into_iter().map(|f| (f.id, f)).collect();
    let authors_by_id: HashMap<Uuid, TierListAuthor> =
        authors.into_iter().map(|a| (a.id, a)).collect();

    lists
        .into_iter()
        .map(|list| {
            let list_id = list.id;
            let flair_id = list.flair_id;
            let created_by = list.created_by;
            let tier_details = tiers_by_list
                .remove(&list_id)
                .unwrap_or_default()
                .into_iter()
                .map(|tier| {
                    let placements = placements_by_tier
                        .remove(&tier.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|p| resolve_placement(p, gd, assets))
                        .collect();
                    TierDetail { tier, placements }
                })
                .collect();

            TierListDetail {
                list,
                tiers: tier_details,
                stats: stats_by_list.remove(&list_id),
                flair: flair_id.and_then(|id| flairs_by_id.get(&id).cloned()),
                author: created_by.and_then(|id| authors_by_id.get(&id).cloned()),
            }
        })
        .collect()
}

/// Global role, then ownership, then the per-list grant: the first that covers `required` admits.
pub async fn check_permission(
    state: &AppState,
    tier_list: &TierList,
    user_id: Uuid,
    role: GlobalRole,
    required: Permission,
) -> Result<(), ApiError> {
    if let Some(global_perm) = role.global_tier_permission()
        && global_perm.grants(required)
    {
        return Ok(());
    }

    if tier_list.created_by == Some(user_id) {
        return Ok(());
    }

    let perm = get_user_permission(&state.db, tier_list.id, user_id).await?;
    match perm {
        Some(p) => {
            let level = Permission::from_str(&p.permission).map_err(|_| ApiError::Forbidden)?;
            if level.grants(required) {
                Ok(())
            } else {
                Err(ApiError::Forbidden)
            }
        }
        None => Err(ApiError::Forbidden),
    }
}

/// Load the tier list by `slug` (404 if absent) and confirm `user_id`/`role` may
/// perform `required` on it (403 otherwise), returning the authorized list.
pub async fn find_and_authorize(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
    required: Permission,
) -> Result<TierList, ApiError> {
    let list = find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    check_permission(state, &list, user_id, role, required).await?;
    Ok(list)
}

/// The list as `GET /tier-lists/{slug}` serves it, with every placement
/// resolved against `server`'s game data (the default server's, when `server`
/// is not loaded).
pub async fn get_by_slug(
    state: &AppState,
    slug: &str,
    server: Server,
) -> Result<TierListDetail, ApiError> {
    let mut detail = load_detail(state, slug).await?;
    resolve_detail(
        &mut detail,
        &state.game_data(server),
        &state.asset_index(server),
    );
    Ok(detail)
}

/// The stored list, cached, with no placement resolved. Resolution happens per
/// request so one cache entry serves every server and follows a game-data
/// reload.
pub async fn load_detail(state: &AppState, slug: &str) -> Result<TierListDetail, ApiError> {
    let key = CacheKey::TierList { slug };
    if let Some(cached) = state.cache.get::<TierListDetail>(&key).await {
        return Ok(cached);
    }

    let list = find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;

    let flair_id = list.flair_id;
    let created_by = list.created_by;
    let tiers_fut = get_tiers(&state.db, list.id);
    let stats_fut = get_stats(&state.db, list.id);
    let flair_fut = async {
        match flair_id {
            Some(id) => get_flair_by_id(&state.db, id).await,
            None => Ok(None),
        }
    };
    let author_fut = async {
        match created_by {
            Some(uid) => {
                sqlx::query_as::<_, TierListAuthor>(
                    "SELECT id, uid, nickname, avatar_id FROM users WHERE id = $1",
                )
                .bind(uid)
                .fetch_optional(&state.db)
                .await
            }
            None => Ok(None),
        }
    };

    let (tiers, stats, flair, author) =
        tokio::try_join!(tiers_fut, stats_fut, flair_fut, author_fut)?;
    let tier_ids: Vec<Uuid> = tiers.iter().map(|tier| tier.id).collect();
    let placements = get_placements_for_tiers(&state.db, &tier_ids).await?;
    let mut placement_map: HashMap<Uuid, Vec<TierPlacement>> = HashMap::new();
    for placement in placements {
        placement_map
            .entry(placement.tier_id)
            .or_default()
            .push(placement);
    }
    let tiers = tiers
        .into_iter()
        .map(|tier| {
            let placements = placement_map
                .remove(&tier.id)
                .unwrap_or_default()
                .into_iter()
                .map(PlacementDetail::unresolved)
                .collect();
            TierDetail { tier, placements }
        })
        .collect();

    let detail = TierListDetail {
        list,
        tiers,
        stats,
        flair,
        author,
    };
    state.cache.set(&key, &detail).await;
    Ok(detail)
}

pub async fn list_details(
    state: &AppState,
    limit: i64,
    server: Server,
) -> Result<Vec<TierListDetail>, ApiError> {
    let lists = find_all_active_limited(&state.db, limit).await?;
    let list_ids: Vec<Uuid> = lists.iter().map(|list| list.id).collect();
    let flair_ids: Vec<i16> = lists
        .iter()
        .filter_map(|list| list.flair_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let author_ids: Vec<Uuid> = lists
        .iter()
        .filter_map(|list| list.created_by)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let tiers_fut = get_tiers_for_lists(&state.db, &list_ids);
    let stats_fut = get_stats_for_lists(&state.db, &list_ids);
    let flairs_fut = get_flairs_by_ids(&state.db, &flair_ids);
    let authors_fut = async {
        if author_ids.is_empty() {
            Ok(Vec::new())
        } else {
            let authors = sqlx::query_as::<_, TierListAuthor>(
                "SELECT id, uid, nickname, avatar_id FROM users WHERE id = ANY($1)",
            )
            .bind(&author_ids)
            .fetch_all(&state.db)
            .await?;
            Ok(authors)
        }
    };

    let (tiers, stats, flairs, authors) =
        tokio::try_join!(tiers_fut, stats_fut, flairs_fut, authors_fut)?;
    let tier_ids: Vec<Uuid> = tiers.iter().map(|tier| tier.id).collect();
    let placements = get_placements_for_tiers(&state.db, &tier_ids).await?;

    Ok(assemble_details(
        lists,
        tiers,
        placements,
        stats,
        flairs,
        authors,
        (&state.game_data(server), &state.asset_index(server)),
    ))
}

/// How many active tier lists a regular user may own. Tier-list staff are
/// exempt: they curate lists for the site rather than for themselves.
const MAX_LISTS_PER_USER: i64 = 10;

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    role: GlobalRole,
    name: &str,
    description: Option<&str>,
    list_type: &str,
) -> Result<TierList, ApiError> {
    if list_type == "community" && !role.is_any_admin_role() {
        let count = count_by_user(&state.db, user_id).await?;
        if count >= MAX_LISTS_PER_USER {
            return Err(ApiError::Conflict(format!(
                "maximum {MAX_LISTS_PER_USER} tier lists per user"
            )));
        }
    }

    if list_type == "official" && !role.is_any_admin_role() {
        return Err(ApiError::Forbidden);
    }

    let slug = generate_slug(name);
    let list = queries::create(&state.db, name, &slug, description, list_type, user_id).await?;
    ensure_stats_row(&state.db, list.id).await?;
    Ok(list)
}

pub async fn update_list(
    state: &AppState,
    slug: &str,
    user_id: Uuid,
    role: GlobalRole,
    name: &str,
    description: Option<&str>,
    entity_kinds: Option<&[EntityKind]>,
) -> Result<TierList, ApiError> {
    let kinds = entity_kinds.map(offered_kinds).transpose()?;
    let list = find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    check_permission(state, &list, user_id, role, Permission::Edit).await?;
    let updated = update(&state.db, list.id, name, description, kinds.as_deref())
        .await
        .map_err(ApiError::from)?;
    Ok(updated)
}

/// A list's offered kinds as stored: first sighting kept, order kept.
///
/// # Errors
/// `400` when the set is empty: a list's editor offers at least one kind.
fn offered_kinds(kinds: &[EntityKind]) -> Result<Vec<String>, ApiError> {
    let mut out: Vec<String> = Vec::with_capacity(kinds.len());
    for kind in kinds {
        let s = kind.as_str().to_owned();
        if !out.contains(&s) {
            out.push(s);
        }
    }
    if out.is_empty() {
        return Err(ApiError::BadRequest(
            "entity_kinds: a list offers at least one kind".into(),
        ));
    }
    Ok(out)
}

pub async fn invalidate_detail(state: &AppState, slug: &str) {
    state.cache.invalidate(&CacheKey::TierList { slug }).await;
}

pub(crate) fn generate_slug(name: &str) -> String {
    let base: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let suffix: String = (0..6)
        .map(|_| {
            let n = rand::random::<u8>() % 36;
            if n < 10 {
                (b'0' + n) as char
            } else {
                (b'a' + n - 10) as char
            }
        })
        .collect();
    format!("{base}-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offered_kinds_dedupe_in_order_and_refuse_none() {
        let kinds = offered_kinds(&[EntityKind::Enemy, EntityKind::Operator, EntityKind::Enemy])
            .expect("non-empty");
        assert_eq!(kinds, ["enemy", "operator"]);
        assert!(matches!(offered_kinds(&[]), Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn an_old_snapshot_reads_as_operators() {
        let old = serde_json::json!([{
            "id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
            "tier_list_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02",
            "name": "S",
            "display_order": 0,
            "color": "#dc4d56",
            "description": null,
            "placements": [{
                "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
                "operator_id": "char_002_amiya",
                "sub_order": 3,
                "description": "why",
                "updated_at": "2026-01-02T03:04:05Z"
            }]
        }]);
        let tiers = read_snapshot(&old).expect("old snapshot parses");
        let p = &tiers[0].placements[0];
        assert_eq!(p.entity_kind, EntityKind::Operator);
        assert_eq!(p.entity_id, "char_002_amiya");
        assert_eq!(p.sub_order, 3);
        assert_eq!(p.description.as_deref(), Some("why"));
    }

    #[test]
    fn a_cache_entry_from_before_kinds_still_reads() {
        // `PlacementDetail` flattens `TierPlacement`, so the alias and default
        // have to survive serde's buffered flatten path, not only the direct one.
        let old = serde_json::json!({
            "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
            "operator_id": "char_002_amiya",
            "sub_order": 2,
            "description": null,
            "updated_at": "2026-01-02T03:04:05Z"
        });
        let p: PlacementDetail = serde_json::from_value(old).expect("old cache entry parses");
        assert_eq!(p.placement.entity_kind, EntityKind::Operator);
        assert_eq!(p.placement.entity_id, "char_002_amiya");
        assert_eq!(p.placement.sub_order, 2);
        assert!(p.entity.is_none());
    }

    #[test]
    fn a_new_snapshot_round_trips_without_resolved_entities() {
        let old = serde_json::json!([{
            "id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
            "tier_list_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02",
            "name": "S",
            "display_order": 0,
            "color": null,
            "description": null,
            "placements": [{
                "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
                "operator_id": "char_002_amiya",
                "sub_order": 0,
                "description": null,
                "updated_at": "2026-01-02T03:04:05Z"
            }]
        }]);
        let written = serde_json::to_value(read_snapshot(&old).expect("parses")).expect("writes");
        let placement = &written[0]["placements"][0];
        assert_eq!(placement["entity_kind"], "operator");
        assert_eq!(placement["entity_id"], "char_002_amiya");
        assert!(placement.get("operator_id").is_none());
        assert!(placement.get("entity").is_none());
        let again = read_snapshot(&written).expect("new shape parses");
        assert_eq!(again[0].placements[0].entity_id, "char_002_amiya");
    }
}
