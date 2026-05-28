use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::auth::permissions::{GlobalRole, Permission};
use crate::database::models::tier_list::{
    Tier, TierList, TierListFlair, TierListStats, TierPlacement,
};
use crate::database::queries::tier_lists as queries;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize)]
pub struct TierListDetail {
    #[serde(flatten)]
    pub list: TierList,
    pub tiers: Vec<TierDetail>,
    pub stats: Option<TierListStats>,
    pub flair: Option<TierListFlair>,
    pub author: Option<TierListAuthor>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TierListAuthor {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub avatar_id: Option<String>,
}

#[derive(Serialize)]
pub struct TierDetail {
    #[serde(flatten)]
    pub tier: Tier,
    pub placements: Vec<TierPlacement>,
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
    let perm = queries::get_user_permission(&state.db, tier_list.id, user_id).await?;
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

pub async fn get_by_slug(state: &AppState, slug: &str) -> Result<TierListDetail, ApiError> {
    let list = queries::find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    let tiers = queries::get_tiers(&state.db, list.id).await?;

    // For each tier, load placements
    let mut tier_details = Vec::new();
    for tier in tiers {
        let placements = queries::get_placements(&state.db, tier.id).await?;
        tier_details.push(TierDetail { tier, placements });
    }

    let stats = queries::get_stats(&state.db, list.id).await?;
    let flair = match list.flair_id {
        Some(id) => queries::get_flair_by_id(&state.db, id).await?,
        None => None,
    };
    let author = match list.created_by {
        Some(uid) => {
            sqlx::query_as::<_, TierListAuthor>(
                "SELECT id, uid, nickname, avatar_id FROM users WHERE id = $1",
            )
            .bind(uid)
            .fetch_optional(&state.db)
            .await?
        }
        None => None,
    };

    Ok(TierListDetail {
        list,
        tiers: tier_details,
        stats,
        flair,
        author,
    })
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
        let count = queries::count_by_user(&state.db, user_id).await?;
        if count >= 10 {
            return Err(ApiError::Conflict("maximum 10 tier lists per user".into()));
        }
    }

    if list_type == "official" && !role.is_any_admin_role() {
        return Err(ApiError::Forbidden);
    }

    let slug = generate_slug(name);
    let list = queries::create(&state.db, name, &slug, description, list_type, user_id).await?;
    queries::ensure_stats_row(&state.db, list.id).await?;
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
    let list = queries::find_by_slug(&state.db, slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    check_permission(state, &list, user_id, role, Permission::Edit).await?;
    queries::update(&state.db, list.id, name, description)
        .await
        .map_err(std::convert::Into::into)
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
