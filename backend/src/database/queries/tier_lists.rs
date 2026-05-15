use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::tier_list::{
    Tier, TierList, TierListFlair, TierListPermission, TierListStats, TierListVersion,
    TierPlacement,
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
            "SELECT * FROM tier_lists WHERE is_active = true AND is_listed = true AND list_type = $1 ORDER BY updated_at DESC"
        )
        .bind(lt)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, TierList>(
            "SELECT * FROM tier_lists WHERE is_active = true AND is_listed = true ORDER BY updated_at DESC",
        )
        .fetch_all(pool)
        .await
    }
}

pub async fn set_visibility(
    pool: &PgPool,
    tier_list_id: Uuid,
    is_listed: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tier_lists SET is_listed = $2, updated_at = NOW() WHERE id = $1")
        .bind(tier_list_id)
        .bind(is_listed)
        .execute(pool)
        .await?;
    Ok(())
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
        "INSERT INTO tiers (tier_list_id, name, display_order, color, description)
          VALUES ($1, $2, $3, $4, $5)
          RETURNING *",
    )
    .bind(tier_list_id)
    .bind(name)
    .bind(display_order)
    .bind(color)
    .bind(description)
    .fetch_one(pool)
    .await
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
        "INSERT INTO tier_placements (tier_id, operator_id, sub_order, notes)
          VALUES ($1,$2,$3,$4)
          ON CONFLICT (tier_id, operator_id)
          DO UPDATE SET sub_order = EXCLUDED.sub_order,
                        notes = EXCLUDED.notes,
                        updated_at = NOW()
          RETURNING *",
    )
    .bind(tier_id)
    .bind(operator_id)
    .bind(sub_order)
    .bind(notes)
    .fetch_one(pool)
    .await
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

/// Get tier list stats
pub async fn get_stats(
    pool: &PgPool,
    tier_list_id: Uuid,
) -> Result<Option<TierListStats>, sqlx::Error> {
    sqlx::query_as::<_, TierListStats>("SELECT * FROM tier_list_stats WHERE tier_list_id = $1")
        .bind(tier_list_id)
        .fetch_optional(pool)
        .await
}

pub async fn ensure_stats_row(pool: &PgPool, tier_list_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tier_list_stats (tier_list_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(tier_list_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Record a view. Dedupe window is 30 minutes. Returns true if counter was bumped.
pub async fn record_view(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Option<Uuid>,
    session_hash: Option<&str>,
) -> Result<bool, sqlx::Error> {
    // Guard against lists without a stats row (defensive; backfill + create should handle it).
    ensure_stats_row(pool, tier_list_id).await?;

    // Dedupe check
    let dedupe_exists: bool = match (user_id, session_hash) {
        (Some(uid), _) => sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tier_list_view_events
            WHERE tier_list_id=$1 AND user_id=$2 AND viewed_at > NOW() - INTERVAL '30 minutes')"
        ).bind(tier_list_id).bind(uid).fetch_one(pool).await?,
        (None, Some(sh)) => sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM tier_list_view_events
            WHERE tier_list_id=$1 AND session_hash=$2 AND viewed_at > NOW() - INTERVAL '30 minutes')"
        ).bind(tier_list_id).bind(sh).fetch_one(pool).await?,
        (None, None) => false,
    };

    if dedupe_exists {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO tier_list_view_events (tier_list_id, user_id, session_hash) VALUES ($1,$2,$3)",
    )
    .bind(tier_list_id)
    .bind(user_id)
    .bind(session_hash)
    .execute(pool)
    .await?;

    sqlx::query(
        "UPDATE tier_list_stats
        SET view_count = view_count + 1,
            unique_view_count = unique_view_count + 1,
            last_viewed_at = NOW()
        WHERE tier_list_id = $1",
    )
    .bind(tier_list_id)
    .execute(pool)
    .await?;
    Ok(true)
}

pub async fn bump_share(pool: &PgPool, tier_list_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tier_list_stats SET share_count = share_count + 1 WHERE tier_list_id = $1")
        .bind(tier_list_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_favorite(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO tier_list_favorites (tier_list_id, user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING"
    ).bind(tier_list_id).bind(user_id).execute(pool).await?;

    if row.rows_affected() > 0 {
        sqlx::query("UPDATE tier_list_stats SET favorite_count = favorite_count + 1 WHERE tier_list_id = $1")
            .bind(tier_list_id).execute(pool).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn remove_favorite(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row =
        sqlx::query("DELETE FROM tier_list_favorites WHERE tier_list_id = $1 AND user_id = $2")
            .bind(tier_list_id)
            .bind(user_id)
            .execute(pool)
            .await?;

    if row.rows_affected() > 0 {
        sqlx::query("UPDATE tier_list_stats SET favorite_count = GREATEST(favorite_count - 1, 0) WHERE tier_list_id = $1")
            .bind(tier_list_id).execute(pool).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn is_favorited(
    pool: &PgPool,
    tier_list_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM tier_list_favorites WHERE tier_list_id=$1 AND user_id=$2)",
    )
    .bind(tier_list_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn list_flairs(
    pool: &PgPool,
    only_active: bool,
) -> Result<Vec<TierListFlair>, sqlx::Error> {
    let sql = if only_active {
        "SELECT * FROM tier_list_flairs WHERE is_active = true ORDER BY display_order, label"
    } else {
        "SELECT * FROM tier_list_flairs ORDER BY display_order, label"
    };
    sqlx::query_as::<_, TierListFlair>(sql)
        .fetch_all(pool)
        .await
}

pub async fn create_flair(
    pool: &PgPool,
    code: &str,
    label: &str,
    color: Option<&str>,
    display_order: i16,
) -> Result<TierListFlair, sqlx::Error> {
    sqlx::query_as::<_, TierListFlair>(
        "INSERT INTO tier_list_flairs (code, label, color, display_order) VALUES ($1,$2,$3,$4) RETURNING *"
    ).bind(code).bind(label).bind(color).bind(display_order).fetch_one(pool).await
}

pub async fn update_flair(
    pool: &PgPool,
    id: i16,
    label: &str,
    color: Option<&str>,
    display_order: i16,
    is_active: bool,
) -> Result<TierListFlair, sqlx::Error> {
    sqlx::query_as::<_, TierListFlair>(
        "UPDATE tier_list_flairs SET label=$2, color=$3, display_order=$4, is_active=$5, updated_at=NOW()
        WHERE id=$1 RETURNING *"
    ).bind(id).bind(label).bind(color).bind(display_order).bind(is_active).fetch_one(pool).await
}

pub async fn set_flair(
    pool: &PgPool,
    tier_list_id: Uuid,
    flair_id: Option<i16>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tier_lists SET flair_id = $2, updated_at = NOW() WHERE id = $1")
        .bind(tier_list_id)
        .bind(flair_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn recompute_trending(pool: &PgPool, top_n: i64) -> Result<(), sqlx::Error> {
    // One query: refresh windowed counts + score for all lists
    sqlx::query(r#"
        UPDATE tier_list_stats s SET
            views_last_24h = COALESCE(w.v24, 0),
            views_last_7d  = COALESCE(w.v7, 0),
            trending_score = COALESCE(w.v24, 0) * 1.0
                            + COALESCE(f.f24, 0) * 5.0
                            + COALESCE(w.v7, 0) * 0.15
                            - LEAST(EXTRACT(EPOCH FROM (NOW() - tl.created_at)) / 3600.0, 720) * 0.1,
            stats_updated_at = NOW()
        FROM tier_lists tl
        LEFT JOIN (
            SELECT tier_list_id,
                    COUNT(*) FILTER (WHERE viewed_at > NOW() - INTERVAL '24 hours') AS v24,
                    COUNT(*) FILTER (WHERE viewed_at > NOW() - INTERVAL '7 days')   AS v7
            FROM tier_list_view_events GROUP BY tier_list_id
        ) w ON w.tier_list_id = tl.id
        LEFT JOIN (
            SELECT tier_list_id, COUNT(*) AS f24
            FROM tier_list_favorites
            WHERE favorited_at > NOW() - INTERVAL '24 hours'
            GROUP BY tier_list_id
        ) f ON f.tier_list_id = tl.id
        WHERE s.tier_list_id = tl.id AND tl.is_active = true
    "#).execute(pool).await?;

    // Flip top-N active lists to is_trending = true, all others false.
    sqlx::query(
        r#"
        WITH ranked AS (
            SELECT s.tier_list_id,
                   RANK() OVER (ORDER BY s.trending_score DESC) AS rnk
            FROM tier_list_stats s
            JOIN tier_lists tl ON tl.id = s.tier_list_id
            WHERE tl.is_active = true
        )
        UPDATE tier_list_stats s
        SET is_trending = COALESCE(
            (SELECT r.rnk <= $1 AND s.trending_score > 0
             FROM ranked r WHERE r.tier_list_id = s.tier_list_id),
            false
        )
    "#,
    )
    .bind(top_n)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_flair_by_id(pool: &PgPool, id: i16) -> Result<Option<TierListFlair>, sqlx::Error> {
    sqlx::query_as::<_, TierListFlair>("SELECT * FROM tier_list_flairs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}
