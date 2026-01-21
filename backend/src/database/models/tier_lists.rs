use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// ============================================================================
// Core Models (Database Tables)
// ============================================================================

/// Main tier list configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierList {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default = "default_tier_list_type")]
    pub tier_list_type: String,
    #[serde(default)]
    pub is_deleted: bool,
    pub deleted_by: Option<Uuid>,
    pub deleted_reason: Option<String>,
}

fn default_tier_list_type() -> String {
    "official".to_string()
}

/// Tier definitions within a list (e.g., S+, S, A, B, C)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tier {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
}

/// Operator placements within a tier
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierPlacement {
    pub id: Uuid,
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Version history for changelogs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierListVersion {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version: i32,
    pub snapshot: serde_json::Value,
    pub changelog: String,
    pub change_summary: Option<String>,
    pub published_at: DateTime<Utc>,
    pub published_by: Option<Uuid>,
}

/// Granular change audit log
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierChangeLog {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version_id: Option<Uuid>,
    pub change_type: String,
    pub operator_id: Option<String>,
    pub old_tier_id: Option<Uuid>,
    pub new_tier_id: Option<Uuid>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub reason: Option<String>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
}

/// Admin permissions for tier lists
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierListPermission {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

// ============================================================================
// Snapshot Types (for versioning, not stored directly)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierListSnapshot {
    pub tiers: Vec<TierSnapshot>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierSnapshot {
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
    pub operators: Vec<OperatorPlacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorPlacement {
    pub operator_id: String,
    pub sub_order: i32,
    pub notes: Option<String>,
}

// ============================================================================
// DTOs (Data Transfer Objects)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTierList {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(default = "default_tier_list_type")]
    pub tier_list_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTierList {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTier {
    pub name: String,
    pub display_order: i32,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlacement {
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MovePlacement {
    pub new_tier_id: Uuid,
    pub new_sub_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PublishVersion {
    pub changelog: String,
    pub change_summary: Option<String>,
}

// ============================================================================
// TierList Implementation
// ============================================================================

impl TierList {
    pub async fn create(
        pool: &PgPool,
        input: CreateTierList,
        created_by: Option<Uuid>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_lists (name, slug, description, created_by, tier_list_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .bind(&input.slug)
        .bind(&input.description)
        .bind(created_by)
        .bind(&input.tier_list_type)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM tier_lists WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM tier_lists WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all_active(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_lists WHERE is_active = true ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_all(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_lists ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        input: UpdateTierList,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_lists
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tier_lists WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Count community tier lists by owner (for limit enforcement)
    pub async fn count_community_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM tier_lists
            WHERE created_by = $1 AND tier_list_type = 'community' AND is_deleted = false
            "#,
        )
        .bind(owner_id)
        .fetch_one(pool)
        .await
    }

    /// Find community tier lists by owner
    pub async fn find_community_by_owner(
        pool: &PgPool,
        owner_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tier_lists
            WHERE created_by = $1 AND tier_list_type = 'community'
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    /// Find all active tier lists with optional type filter
    pub async fn find_all_active_by_type(
        pool: &PgPool,
        tier_list_type: Option<&str>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        match tier_list_type {
            Some(t) => {
                sqlx::query_as::<_, Self>(
                    r#"
                    SELECT * FROM tier_lists
                    WHERE is_active = true AND is_deleted = false AND tier_list_type = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(t)
                .fetch_all(pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Self>(
                    r#"
                    SELECT * FROM tier_lists
                    WHERE is_active = true AND is_deleted = false
                    ORDER BY created_at DESC
                    "#,
                )
                .fetch_all(pool)
                .await
            }
        }
    }

    /// Soft delete a tier list (for moderation)
    pub async fn soft_delete(
        pool: &PgPool,
        id: Uuid,
        deleted_by: Uuid,
        reason: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE tier_lists
            SET is_deleted = true, deleted_by = $1, deleted_reason = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(deleted_by)
        .bind(reason)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Check if the tier list is a community tier list
    pub fn is_community(&self) -> bool {
        self.tier_list_type == "community"
    }

    /// Check if the user is the owner of this tier list
    pub fn is_owner(&self, user_id: Uuid) -> bool {
        self.created_by == Some(user_id)
    }
}

// ============================================================================
// Tier Implementation
// ============================================================================

impl Tier {
    pub async fn create(
        pool: &PgPool,
        tier_list_id: Uuid,
        input: CreateTier,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tiers (tier_list_id, name, display_order, color, description)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(tier_list_id)
        .bind(&input.name)
        .bind(input.display_order)
        .bind(&input.color)
        .bind(&input.description)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tiers WHERE tier_list_id = $1 ORDER BY display_order ASC",
        )
        .bind(tier_list_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM tiers WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<String>,
        display_order: Option<i32>,
        color: Option<String>,
        description: Option<String>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tiers
            SET name = COALESCE($1, name),
                display_order = COALESCE($2, display_order),
                color = COALESCE($3, color),
                description = COALESCE($4, description)
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(display_order)
        .bind(color)
        .bind(description)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tiers WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn reorder(
        pool: &PgPool,
        tier_list_id: Uuid,
        tier_orders: Vec<(Uuid, i32)>,
    ) -> Result<(), sqlx::Error> {
        // First, set all display_orders to temporary negative values to avoid unique constraint violations
        // when tiers swap positions
        for (i, (tier_id, _)) in tier_orders.iter().enumerate() {
            sqlx::query("UPDATE tiers SET display_order = $1 WHERE id = $2 AND tier_list_id = $3")
                .bind(-(i as i32) - 1000) // Use negative values like -1000, -1001, etc.
                .bind(tier_id)
                .bind(tier_list_id)
                .execute(pool)
                .await?;
        }

        // Now set the final display_order values
        for (tier_id, order) in tier_orders {
            sqlx::query("UPDATE tiers SET display_order = $1 WHERE id = $2 AND tier_list_id = $3")
                .bind(order)
                .bind(tier_id)
                .bind(tier_list_id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }
}

// ============================================================================
// TierPlacement Implementation
// ============================================================================

impl TierPlacement {
    pub async fn create(pool: &PgPool, input: CreatePlacement) -> Result<Self, sqlx::Error> {
        let sub_order = input.sub_order.unwrap_or(0);
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_placements (tier_id, operator_id, sub_order, notes)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(input.tier_id)
        .bind(&input.operator_id)
        .bind(sub_order)
        .bind(&input.notes)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_tier(pool: &PgPool, tier_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_placements WHERE tier_id = $1 ORDER BY sub_order ASC",
        )
        .bind(tier_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT tp.* FROM tier_placements tp
            JOIN tiers t ON tp.tier_id = t.id
            WHERE t.tier_list_id = $1
            ORDER BY t.display_order ASC, tp.sub_order ASC
            "#,
        )
        .bind(tier_list_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_operator(
        pool: &PgPool,
        tier_list_id: Uuid,
        operator_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT tp.* FROM tier_placements tp
            JOIN tiers t ON tp.tier_id = t.id
            WHERE t.tier_list_id = $1 AND tp.operator_id = $2
            "#,
        )
        .bind(tier_list_id)
        .bind(operator_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn move_to_tier(
        pool: &PgPool,
        id: Uuid,
        new_tier_id: Uuid,
        new_sub_order: Option<i32>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_placements
            SET tier_id = $1, sub_order = COALESCE($2, sub_order), updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(new_tier_id)
        .bind(new_sub_order)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_notes(
        pool: &PgPool,
        id: Uuid,
        notes: Option<String>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_placements SET notes = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(notes)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update_sub_order(
        pool: &PgPool,
        id: Uuid,
        sub_order: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_placements SET sub_order = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(sub_order)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        sub_order: Option<i32>,
        notes: Option<Option<String>>,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_placements
            SET sub_order = COALESCE($1, sub_order),
                notes = COALESCE($2, notes),
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(sub_order)
        .bind(notes)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tier_placements WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_by_operator(
        pool: &PgPool,
        tier_list_id: Uuid,
        operator_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM tier_placements tp
            USING tiers t
            WHERE tp.tier_id = t.id AND t.tier_list_id = $1 AND tp.operator_id = $2
            "#,
        )
        .bind(tier_list_id)
        .bind(operator_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// TierListVersion Implementation
// ============================================================================

impl TierListVersion {
    pub async fn create(
        pool: &PgPool,
        tier_list_id: Uuid,
        snapshot: serde_json::Value,
        input: PublishVersion,
        published_by: Option<Uuid>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_list_versions (tier_list_id, version, snapshot, changelog, change_summary, published_by)
            VALUES ($1, COALESCE((SELECT MAX(version) FROM tier_list_versions WHERE tier_list_id = $1), 0) + 1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(tier_list_id)
        .bind(snapshot)
        .bind(&input.changelog)
        .bind(&input.change_summary)
        .bind(published_by)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_versions WHERE tier_list_id = $1 ORDER BY version DESC LIMIT $2",
        )
        .bind(tier_list_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_version(
        pool: &PgPool,
        tier_list_id: Uuid,
        version: i32,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_versions WHERE tier_list_id = $1 AND version = $2",
        )
        .bind(tier_list_id)
        .bind(version)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_latest(
        pool: &PgPool,
        tier_list_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_versions WHERE tier_list_id = $1 ORDER BY version DESC LIMIT 1",
        )
        .bind(tier_list_id)
        .fetch_optional(pool)
        .await
    }
}

// ============================================================================
// TierChangeLog Implementation
// ============================================================================

impl TierChangeLog {
    #![allow(clippy::all)]
    pub async fn create(
        pool: &PgPool,
        tier_list_id: Uuid,
        change_type: &str,
        operator_id: Option<&str>,
        old_tier_id: Option<Uuid>,
        new_tier_id: Option<Uuid>,
        reason: Option<&str>,
        changed_by: Option<Uuid>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_change_log (tier_list_id, change_type, operator_id, old_tier_id, new_tier_id, reason, changed_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(tier_list_id)
        .bind(change_type)
        .bind(operator_id)
        .bind(old_tier_id)
        .bind(new_tier_id)
        .bind(reason)
        .bind(changed_by)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_change_log WHERE tier_list_id = $1 ORDER BY changed_at DESC LIMIT $2",
        )
        .bind(tier_list_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_operator(
        pool: &PgPool,
        tier_list_id: Uuid,
        operator_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_change_log WHERE tier_list_id = $1 AND operator_id = $2 ORDER BY changed_at DESC",
        )
        .bind(tier_list_id)
        .bind(operator_id)
        .fetch_all(pool)
        .await
    }

    pub async fn link_to_version(
        pool: &PgPool,
        tier_list_id: Uuid,
        version_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE tier_change_log
            SET version_id = $1
            WHERE tier_list_id = $2 AND version_id IS NULL AND changed_at >= $3
            "#,
        )
        .bind(version_id)
        .bind(tier_list_id)
        .bind(since)
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

// ============================================================================
// TierListPermission Implementation
// ============================================================================

impl TierListPermission {
    pub async fn grant(
        pool: &PgPool,
        tier_list_id: Uuid,
        user_id: Uuid,
        permission: &str,
        granted_by: Option<Uuid>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (tier_list_id, user_id, permission) DO UPDATE SET granted_at = NOW()
            RETURNING *
            "#,
        )
        .bind(tier_list_id)
        .bind(user_id)
        .bind(permission)
        .bind(granted_by)
        .fetch_one(pool)
        .await
    }

    pub async fn revoke(
        pool: &PgPool,
        tier_list_id: Uuid,
        user_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2 AND permission = $3",
        )
        .bind(tier_list_id)
        .bind(user_id)
        .bind(permission)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn find_user_permissions(
        pool: &PgPool,
        tier_list_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2",
        )
        .bind(tier_list_id)
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    pub async fn has_permission(
        pool: &PgPool,
        tier_list_id: Uuid,
        user_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2 AND permission = $3",
        )
        .bind(tier_list_id)
        .bind(user_id)
        .bind(permission)
        .fetch_one(pool)
        .await?;
        Ok(result > 0)
    }

    pub async fn find_all_for_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_permissions WHERE tier_list_id = $1 ORDER BY granted_at DESC",
        )
        .bind(tier_list_id)
        .fetch_all(pool)
        .await
    }
}

// ============================================================================
// Snapshot Builder (for versioning)
// ============================================================================

impl TierListSnapshot {
    pub async fn build(pool: &PgPool, tier_list_id: Uuid) -> Result<Self, sqlx::Error> {
        let tiers = Tier::find_by_tier_list(pool, tier_list_id).await?;

        let mut tier_snapshots = Vec::new();
        for tier in tiers {
            let placements = TierPlacement::find_by_tier(pool, tier.id).await?;
            let operators = placements
                .into_iter()
                .map(|p| OperatorPlacement {
                    operator_id: p.operator_id,
                    sub_order: p.sub_order,
                    notes: p.notes,
                })
                .collect();

            tier_snapshots.push(TierSnapshot {
                name: tier.name,
                display_order: tier.display_order,
                color: tier.color,
                description: tier.description,
                operators,
            });
        }

        Ok(Self {
            tiers: tier_snapshots,
            metadata: serde_json::json!({}),
        })
    }
}

// ============================================================================
// Tier List Reports (for community tier list moderation)
// ============================================================================

/// Report for inappropriate community tier lists
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TierListReport {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub reporter_id: Uuid,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub action_taken: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub reason: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewReport {
    pub status: String,
    pub action_taken: Option<String>,
}

impl TierListReport {
    /// Create a new report
    pub async fn create(
        pool: &PgPool,
        tier_list_id: Uuid,
        reporter_id: Uuid,
        input: CreateReport,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tier_list_reports (tier_list_id, reporter_id, reason, description)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(tier_list_id)
        .bind(reporter_id)
        .bind(&input.reason)
        .bind(&input.description)
        .fetch_one(pool)
        .await
    }

    /// Find report by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM tier_list_reports WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Find all reports for a tier list
    pub async fn find_by_tier_list(
        pool: &PgPool,
        tier_list_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tier_list_reports WHERE tier_list_id = $1 ORDER BY created_at DESC",
        )
        .bind(tier_list_id)
        .fetch_all(pool)
        .await
    }

    /// Find all pending reports (for admin moderation)
    pub async fn find_pending(pool: &PgPool, limit: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM tier_list_reports
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// Check if user has already reported a tier list
    pub async fn has_reported(
        pool: &PgPool,
        tier_list_id: Uuid,
        reporter_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tier_list_reports WHERE tier_list_id = $1 AND reporter_id = $2",
        )
        .bind(tier_list_id)
        .bind(reporter_id)
        .fetch_one(pool)
        .await?;
        Ok(result > 0)
    }

    /// Review a report (admin action)
    pub async fn review(
        pool: &PgPool,
        id: Uuid,
        reviewed_by: Uuid,
        input: ReviewReport,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tier_list_reports
            SET status = $1, reviewed_by = $2, reviewed_at = NOW(), action_taken = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&input.status)
        .bind(reviewed_by)
        .bind(&input.action_taken)
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    /// Count pending reports (for admin dashboard)
    pub async fn count_pending(pool: &PgPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM tier_list_reports WHERE status = 'pending'")
            .fetch_one(pool)
            .await
    }
}
