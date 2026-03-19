use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::tier_list::{Tier, TierList, TierListVersion, TierPlacement};

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
