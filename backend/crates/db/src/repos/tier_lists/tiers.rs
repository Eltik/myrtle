use chrono::{DateTime, Utc};
use myrtle_types::{EntityKind, TierListId};
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct TierRow {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlacementRow {
    pub tier_id: Uuid,
    pub entity_kind: EntityKind,
    pub entity_id: String,
    pub sub_order: i16,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

struct StoredPlacement {
    tier_id: Uuid,
    entity_kind: String,
    entity_id: String,
    sub_order: i16,
    description: Option<String>,
    updated_at: DateTime<Utc>,
}

impl StoredPlacement {
    fn with_kind(self, entity_kind: EntityKind) -> PlacementRow {
        PlacementRow {
            tier_id: self.tier_id,
            entity_kind,
            entity_id: self.entity_id,
            sub_order: self.sub_order,
            description: self.description,
            updated_at: self.updated_at,
        }
    }

    fn into_known(self) -> Option<PlacementRow> {
        let Some(kind) = EntityKind::parse(&self.entity_kind) else {
            tracing::warn!(
                tier_id = %self.tier_id,
                entity_id = %self.entity_id,
                "tier_placements: dropping a placement of unknown kind `{}`",
                self.entity_kind
            );
            return None;
        };
        Some(self.with_kind(kind))
    }
}

pub async fn get_tiers(conn: &mut Conn, tier_list: TierListId) -> Result<Vec<TierRow>, DbError> {
    Ok(sqlx::query_as!(
        TierRow,
        "SELECT id, tier_list_id, name, display_order, color, description
         FROM tiers WHERE tier_list_id = $1 ORDER BY display_order",
        tier_list.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_tiers_for_lists(
    conn: &mut Conn,
    tier_lists: &[TierListId],
) -> Result<Vec<TierRow>, DbError> {
    if tier_lists.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = tier_lists.iter().map(|t| t.0).collect();
    Ok(sqlx::query_as!(
        TierRow,
        "SELECT id, tier_list_id, name, display_order, color, description
         FROM tiers WHERE tier_list_id = ANY($1) ORDER BY tier_list_id, display_order",
        &ids,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn create_tier(
    conn: &mut Conn,
    tier_list: TierListId,
    name: &str,
    display_order: i16,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<TierRow, DbError> {
    Ok(sqlx::query_as!(
        TierRow,
        "INSERT INTO tiers (tier_list_id, name, display_order, color, description)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, tier_list_id, name, display_order, color, description",
        tier_list.0,
        name,
        display_order,
        color,
        description,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn update_tier(
    conn: &mut Conn,
    tier_list: TierListId,
    id: Uuid,
    name: &str,
    display_order: i16,
    color: Option<&str>,
    description: Option<&str>,
) -> Result<Option<TierRow>, DbError> {
    Ok(sqlx::query_as!(
        TierRow,
        "UPDATE tiers SET name = $3, display_order = $4, color = $5, description = $6
         WHERE id = $2 AND tier_list_id = $1
         RETURNING id, tier_list_id, name, display_order, color, description",
        tier_list.0,
        id,
        name,
        display_order,
        color,
        description,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn delete_tier(
    conn: &mut Conn,
    tier_list: TierListId,
    id: Uuid,
) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM tiers WHERE id = $2 AND tier_list_id = $1",
        tier_list.0,
        id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_placements(conn: &mut Conn, tier: Uuid) -> Result<Vec<PlacementRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredPlacement,
        "SELECT tier_id, entity_kind, entity_id, sub_order, description, updated_at
         FROM tier_placements WHERE tier_id = $1 ORDER BY sub_order",
        tier,
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .filter_map(StoredPlacement::into_known)
    .collect())
}

pub async fn get_placements_for_tiers(
    conn: &mut Conn,
    tiers: &[Uuid],
) -> Result<Vec<PlacementRow>, DbError> {
    if tiers.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as!(
        StoredPlacement,
        "SELECT tier_id, entity_kind, entity_id, sub_order, description, updated_at
         FROM tier_placements WHERE tier_id = ANY($1) ORDER BY tier_id, sub_order",
        tiers,
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .filter_map(StoredPlacement::into_known)
    .collect())
}

pub async fn add_placement(
    conn: &mut Conn,
    tier_list: TierListId,
    tier: Uuid,
    entity_kind: EntityKind,
    entity_id: &str,
    sub_order: i16,
    description: Option<&str>,
) -> Result<Option<PlacementRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredPlacement,
        "INSERT INTO tier_placements (tier_id, entity_kind, entity_id, sub_order, description)
         SELECT t.id, $3, $4, $5, $6 FROM tiers t
          WHERE t.id = $2 AND t.tier_list_id = $1
         ON CONFLICT (tier_id, entity_kind, entity_id)
         DO UPDATE SET sub_order = EXCLUDED.sub_order, description = EXCLUDED.description
         RETURNING tier_id, entity_kind, entity_id, sub_order, description, updated_at",
        tier_list.0,
        tier,
        entity_kind.as_str(),
        entity_id,
        sub_order,
        description,
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.with_kind(entity_kind)))
}

pub async fn set_placement_description(
    conn: &mut Conn,
    tier_list: TierListId,
    entity_kind: EntityKind,
    entity_id: &str,
    description: Option<&str>,
) -> Result<Option<PlacementRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredPlacement,
        "UPDATE tier_placements tp
            SET description = $4
          FROM tiers t
          WHERE tp.tier_id = t.id
            AND t.tier_list_id = $1
            AND tp.entity_kind = $2
            AND tp.entity_id = $3
          RETURNING tp.tier_id, tp.entity_kind, tp.entity_id, tp.sub_order, tp.description,
                    tp.updated_at",
        tier_list.0,
        entity_kind.as_str(),
        entity_id,
        description,
    )
    .fetch_optional(conn)
    .await?
    .map(|row| row.with_kind(entity_kind)))
}

pub async fn remove_placement(
    conn: &mut Conn,
    tier_list: TierListId,
    tier: Uuid,
    entity_kind: EntityKind,
    entity_id: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM tier_placements tp
          USING tiers t
          WHERE tp.tier_id = t.id
            AND tp.tier_id = $2
            AND tp.entity_kind = $3
            AND tp.entity_id = $4
            AND t.tier_list_id = $1",
        tier_list.0,
        tier,
        entity_kind.as_str(),
        entity_id,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn move_placement(
    conn: &mut Conn,
    tier_list: TierListId,
    old_tier: Uuid,
    new_tier: Uuid,
    entity_kind: EntityKind,
    entity_id: &str,
    sub_order: i16,
) -> Result<Option<PlacementRow>, DbError> {
    let moved = sqlx::query_as!(
        StoredPlacement,
        "WITH gone AS (
             DELETE FROM tier_placements tp
              USING tiers t
              WHERE tp.tier_id = t.id
                AND tp.tier_id = $2
                AND tp.entity_kind = $3
                AND tp.entity_id = $4
                AND t.tier_list_id = $1
                AND EXISTS (SELECT 1 FROM tiers WHERE id = $5 AND tier_list_id = $1)
             RETURNING tp.description
         )
         INSERT INTO tier_placements (tier_id, entity_kind, entity_id, sub_order, description)
         SELECT $5, $3, $4, $6, description FROM gone
         ON CONFLICT (tier_id, entity_kind, entity_id)
         DO UPDATE SET sub_order = EXCLUDED.sub_order, description = EXCLUDED.description
         RETURNING tier_id, entity_kind, entity_id, sub_order, description, updated_at",
        tier_list.0,
        old_tier,
        entity_kind.as_str(),
        entity_id,
        new_tier,
        sub_order,
    )
    .fetch_optional(conn)
    .await?;
    Ok(moved.map(|m| m.with_kind(entity_kind)))
}
