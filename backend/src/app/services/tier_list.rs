use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::permissions::{GlobalRole, Permission};
use crate::database::models::tier_list::{
    Tier, TierList, TierListFlair, TierListStats, TierPlacement,
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
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct TierListDetail {
    #[serde(flatten)]
    pub list: TierList,
    pub tiers: Vec<TierDetail>,
    pub stats: Option<TierListStats>,
    pub flair: Option<TierListFlair>,
    pub author: Option<TierListAuthor>,
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListAuthor {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub avatar_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TierDetail {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<TierPlacement>,
}

fn assemble_details(
    lists: Vec<TierList>,
    tiers: Vec<Tier>,
    placements: Vec<TierPlacement>,
    stats: Vec<TierListStats>,
    flairs: Vec<TierListFlair>,
    authors: Vec<TierListAuthor>,
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
                    let placements = placements_by_tier.remove(&tier.id).unwrap_or_default();
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

/// Check if a user can perform `required` action on a tier list.
/// Returns Ok(()) or Err(Forbidden).
pub async fn check_permission(
    state: &AppState,
    tier_list: &TierList,
    user_id: Uuid,
    role: GlobalRole,
    required: Permission,
) -> Result<(), ApiError> {
    // Global admins can do anything
    if let Some(global_perm) = role.global_tier_permission()
        && global_perm.grants(required)
    {
        return Ok(());
    }

    // Owner has Admin permission
    if tier_list.created_by == Some(user_id) {
        return Ok(());
    }

    // Check per-list permission
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

pub async fn get_by_slug(state: &AppState, slug: &str) -> Result<TierListDetail, ApiError> {
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
            let placements = placement_map.remove(&tier.id).unwrap_or_default();
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

pub async fn list_details(state: &AppState, limit: i64) -> Result<Vec<TierListDetail>, ApiError> {
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
        lists, tiers, placements, stats, flairs, authors,
    ))
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    role: GlobalRole,
    name: &str,
    description: Option<&str>,
    list_type: &str,
) -> Result<TierList, ApiError> {
    // Community lists: 10-per-user limit
    if list_type == "community" {
        let count = count_by_user(&state.db, user_id).await?;
        if count >= 10 {
            return Err(ApiError::Conflict("maximum 10 tier lists per user".into()));
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
) -> Result<TierList, ApiError> {
    let list = find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    check_permission(state, &list, user_id, role, Permission::Edit).await?;
    let updated = update(&state.db, list.id, name, description)
        .await
        .map_err(ApiError::from)?;
    Ok(updated)
}

pub async fn invalidate_detail(state: &AppState, slug: &str) {
    state.cache.invalidate(&CacheKey::TierList { slug }).await;
}

fn generate_slug(name: &str) -> String {
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
