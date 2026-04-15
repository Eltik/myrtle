use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::tier_list::{
    Tier, TierList, TierListPermission, TierListVersion, TierPlacement,
};

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<TierList>, sqlx::Error> {
    sqlx::query_as::<_, TierList>("SELECT * FROM tier_lists WHERE slug = $1 AND is_active = true")
        .bind(slug)
        .fetch_optional(pool)
        .await
}

pub async fn find_all_active(
    pool: &PgPool,
    list_type: Option<&str>,
) -> Result<Vec<TierList>, sqlx::Error> {
    if let Some(lt) = list_type {
        sqlx::query_as::<_, TierList>(
            "SELECT * FROM tier_lists WHERE is_active = true AND list_type = $1 ORDER BY updated_at DESC"
        )
        .bind(lt)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, TierList>(
            "SELECT * FROM tier_lists WHERE is_active = true ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await
    }
}

pub async fn create(
    pool: &PgPool,
    name: &str,
    slug: &str,
    description: Option<&str>,
    list_type: &str,
    created_by: Uuid,
) -> Result<TierList, sqlx::Error> {
    sqlx::query_as::<_, TierList>(
        "INSERT INTO tier_lists (name, slug, description, list_type, created_by) VALUES ($1,$2,$3,$4,$5) RETURNING *"
    )
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(list_type)
    .bind(created_by)
    .fetch_one(pool)
    .await
}

pub async fn get_tiers(pool: &PgPool, tier_list_id: Uuid) -> Result<Vec<Tier>, sqlx::Error> {
    sqlx::query_as::<_, Tier>("SELECT * FROM tiers WHERE tier_list_id = $1 ORDER BY display_order")
        .bind(tier_list_id)
        .fetch_all(pool)
        .await
}

pub async fn get_placements(
    pool: &PgPool,
    tier_id: Uuid,
) -> Result<Vec<TierPlacement>, sqlx::Error> {
    sqlx::query_as::<_, TierPlacement>(
        "SELECT * FROM tier_placements WHERE tier_id = $1 ORDER BY sub_order",
    )
    .bind(tier_id)
    .fetch_all(pool)
    .await
}

pub async fn create_version(
    pool: &PgPool,
    tier_list_id: Uuid,
    version: i32,
    snapshot: &serde_json::Value,
    changelog: Option<&str>,
    published_by: Uuid,
) -> Result<TierListVersion, sqlx::Error> {
    sqlx::query_as::<_, TierListVersion>(
        "INSERT INTO tier_list_versions (tier_list_id, version, snapshot, changelog, published_by) VALUES ($1,$2,$3,$4,$5) RETURNING *"
    )
    .bind(tier_list_id)
    .bind(version)
    .bind(snapshot)
    .bind(changelog)
    .bind(published_by)
    .fetch_one(pool)
    .await
}

/// Soft delete a tier list
pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tier_lists SET is_active = false WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update tier list metadata
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<TierList, sqlx::Error> {
    sqlx::query_as::<_, TierList>(
        "UPDATE tier_lists SET name = $2, description = $3, updated_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(id).bind(name).bind(description)
    .fetch_one(pool).await
}

/// List tier lists created by a specific user
pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<TierList>, sqlx::Error> {
    sqlx::query_as::<_, TierList>(
        "SELECT * FROM tier_lists WHERE created_by = $1 AND is_active = true ORDER BY updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool).await
}

/// Count tier lists owned by a user (for enforcing limits)
pub async fn count_by_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM tier_lists WHERE created_by = $1 AND is_active = true")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

pub async fn create_tier(
    pool: &PgPool,
    tier_list_id: Uuid,
    name: &str,
    display_order: i16,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<Tier, sqlx::Error> {
    sqlx::query_as::<_, Tier>(
        "INSERT INTO tiers (tier_list_id, name, display_order, color, description) VALUES ($1,$2,$3,$4,$5) RETURNING *"
    )
    .bind(tier_list_id).bind(name).bind(display_order).bind(color).bind(description)
    .fetch_one(pool).await
}

pub async fn update_tier(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    display_order: i16,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<Tier, sqlx::Error> {
    sqlx::query_as::<_, Tier>(
        "UPDATE tiers SET name = $2, display_order = $3, color = $4, description = $5 WHERE id = $1 RETURNING *"
    )
    .bind(id).bind(name).bind(display_order).bind(color).bind(description)
    .fetch_one(pool).await
}

pub async fn delete_tier(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tiers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_list(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    // FK cascade handles tiers, placements, versions, permissions.
    sqlx::query("DELETE FROM tier_lists WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_placement(
    pool: &PgPool,
    tier_id: Uuid,
    operator_id: &str,
    sub_order: i16,
    notes: Option<&str>,
) -> Result<TierPlacement, sqlx::Error> {
    sqlx::query_as::<_, TierPlacement>(
        "INSERT INTO tier_placements (tier_id, operator_id, sub_order, notes) VALUES ($1,$2,$3,$4) RETURNING *"
    )
    .bind(tier_id).bind(operator_id).bind(sub_order).bind(notes)
    .fetch_one(pool).await
}

pub async fn remove_placement(
    pool: &PgPool,
    tier_id: Uuid,
    operator_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tier_placements WHERE tier_id = $1 AND operator_id = $2")
        .bind(tier_id)
        .bind(operator_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn move_placement(
    pool: &PgPool,
    old_tier_id: Uuid,
    new_tier_id: Uuid,
    operator_id: &str,
    sub_order: i16,
) -> Result<TierPlacement, sqlx::Error> {
    sqlx::query("DELETE FROM tier_placements WHERE tier_id = $1 AND operator_id = $2")
        .bind(old_tier_id)
        .bind(operator_id)
        .execute(pool)
        .await?;
    add_placement(pool, new_tier_id, operator_id, sub_order, None).await
}

pub async fn get_permissions(
    pool: &PgPool,
    tier_list_id: Uuid,
) -> Result<Vec<TierListPermission>, sqlx::Error> {
    sqlx::query_as::<_, TierListPermission>(
        "SELECT * FROM tier_list_permissions WHERE tier_list_id = $1",
    )
    .bind(tier_list_id)
    .fetch_all(pool)
    .await
}

pub async fn get_user_permission(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
) -> Result<Option<TierListPermission>, sqlx::Error> {
    sqlx::query_as::<_, TierListPermission>(
        "SELECT * FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2",
    )
    .bind(tier_list_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn grant_permission(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
    permission: &str,
    granted_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by) VALUES ($1,$2,$3,$4) ON CONFLICT (tier_list_id, user_id,
permission) DO NOTHING"
    )
    .bind(tier_list_id).bind(user_id).bind(permission).bind(granted_by)
    .execute(pool).await?;
    Ok(())
}

pub async fn revoke_permission(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2")
        .bind(tier_list_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_versions(
    pool: &PgPool,
    tier_list_id: Uuid,
) -> Result<Vec<TierListVersion>, sqlx::Error> {
    sqlx::query_as::<_, TierListVersion>(
        "SELECT * FROM tier_list_versions WHERE tier_list_id = $1 ORDER BY version DESC",
    )
    .bind(tier_list_id)
    .fetch_all(pool)
    .await
}

pub async fn latest_version(pool: &PgPool, tier_list_id: Uuid) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar("SELECT MAX(version) FROM tier_list_versions WHERE tier_list_id = $1")
        .bind(tier_list_id)
        .fetch_one(pool)
        .await
}
