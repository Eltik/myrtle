mod flairs;
mod permissions;
mod stats;
mod tiers;

pub use flairs::*;
pub use permissions::*;
pub use stats::*;
pub use tiers::*;

use chrono::{DateTime, Utc};
use myrtle_types::{EntityKind, TierListId, TierListType, UserId};
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct TierListRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub list_type: String,
    pub created_by: Option<Uuid>,
    pub is_active: bool,
    pub is_listed: bool,
    pub flair_id: Option<i16>,
    pub entity_kinds: Vec<EntityKind>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

struct StoredList {
    id: Uuid,
    name: String,
    slug: String,
    description: Option<String>,
    list_type: String,
    created_by: Option<Uuid>,
    is_active: bool,
    is_listed: bool,
    flair_id: Option<i16>,
    entity_kinds: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<StoredList> for TierListRow {
    fn from(row: StoredList) -> Self {
        let mut entity_kinds: Vec<EntityKind> = row
            .entity_kinds
            .iter()
            .filter_map(|raw| {
                let kind = EntityKind::parse(raw);
                if kind.is_none() {
                    tracing::warn!(slug = %row.slug, "tier_lists.entity_kinds: skipping `{raw}`");
                }
                kind
            })
            .collect();
        if entity_kinds.is_empty() {
            entity_kinds.push(EntityKind::Operator);
        }
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            list_type: row.list_type,
            created_by: row.created_by,
            is_active: row.is_active,
            is_listed: row.is_listed,
            flair_id: row.flair_id,
            entity_kinds,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersionRow {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version: i32,
    pub snapshot: Value,
    pub changelog: Option<String>,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthorRow {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub avatar_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SiteCounts {
    pub total: i64,
    pub active: i64,
    pub total_versions: i64,
    pub total_placements: i64,
}

#[derive(Debug, Clone)]
pub struct ListNameRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct ListDetailRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub is_listed: bool,
    pub list_type: String,
    pub tier_count: i64,
    pub placement_count: i64,
}

#[derive(Debug, Clone)]
pub struct VisibilityRow {
    pub id: Uuid,
    pub name: String,
    pub is_listed: bool,
}

pub async fn find_by_slug(conn: &mut Conn, slug: &str) -> Result<Option<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "SELECT id, name, slug, description, list_type, created_by, is_active, is_listed,
                flair_id, entity_kinds, created_at, updated_at
         FROM tier_lists WHERE slug = $1 AND is_active = true",
        slug,
    )
    .fetch_optional(conn)
    .await?
    .map(Into::into))
}

pub async fn find_all_active(
    conn: &mut Conn,
    list_type: Option<TierListType>,
) -> Result<Vec<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "SELECT id, name, slug, description, list_type, created_by, is_active, is_listed,
                flair_id, entity_kinds, created_at, updated_at
         FROM tier_lists
         WHERE is_active = true AND is_listed = true
           AND ($1::varchar IS NULL OR list_type = $1)
         ORDER BY updated_at DESC",
        list_type.map(TierListType::as_str),
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(Into::into)
    .collect())
}

pub async fn find_all_active_limited(
    conn: &mut Conn,
    limit: i64,
    author: Option<UserId>,
) -> Result<Vec<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "SELECT id, name, slug, description, list_type, created_by, is_active, is_listed,
                flair_id, entity_kinds, created_at, updated_at
         FROM tier_lists
         WHERE is_active = true AND is_listed = true AND ($2::uuid IS NULL OR created_by = $2)
         ORDER BY updated_at DESC LIMIT $1",
        limit,
        author.map(|a| a.0),
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(Into::into)
    .collect())
}

pub async fn find_by_user(conn: &mut Conn, user: UserId) -> Result<Vec<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "SELECT id, name, slug, description, list_type, created_by, is_active, is_listed,
                flair_id, entity_kinds, created_at, updated_at
         FROM tier_lists WHERE created_by = $1 AND is_active = true
         ORDER BY updated_at DESC",
        user.0,
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(Into::into)
    .collect())
}

pub async fn count_by_user(
    conn: &mut Conn,
    user: UserId,
    list_type: Option<TierListType>,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM tier_lists
           WHERE created_by = $1 AND is_active = true
             AND ($2::varchar IS NULL OR list_type = $2)"#,
        user.0,
        list_type.map(TierListType::as_str),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn create(
    conn: &mut Conn,
    name: &str,
    slug: &str,
    description: Option<&str>,
    list_type: TierListType,
    created_by: UserId,
) -> Result<TierListRow, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "INSERT INTO tier_lists (name, slug, description, list_type, created_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, slug, description, list_type, created_by, is_active, is_listed,
                   flair_id, entity_kinds, created_at, updated_at",
        name,
        slug,
        description,
        list_type.as_str(),
        created_by.0,
    )
    .fetch_one(conn)
    .await?
    .into())
}

pub async fn update(
    conn: &mut Conn,
    id: TierListId,
    name: &str,
    description: Option<&str>,
    entity_kinds: Option<&[EntityKind]>,
) -> Result<TierListRow, DbError> {
    let entity_kinds = entity_kinds.map(super::kind_names);
    Ok(sqlx::query_as!(
        StoredList,
        "UPDATE tier_lists SET name = $2, description = $3,
             entity_kinds = COALESCE($4, entity_kinds)
         WHERE id = $1
         RETURNING id, name, slug, description, list_type, created_by, is_active, is_listed,
                   flair_id, entity_kinds, created_at, updated_at",
        id.0,
        name,
        description,
        entity_kinds.as_deref(),
    )
    .fetch_one(conn)
    .await?
    .into())
}

pub async fn set_visibility(
    conn: &mut Conn,
    id: TierListId,
    is_listed: bool,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE tier_lists SET is_listed = $2 WHERE id = $1",
        id.0,
        is_listed,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn set_list_type(
    conn: &mut Conn,
    id: TierListId,
    list_type: TierListType,
) -> Result<Option<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "UPDATE tier_lists SET list_type = $2 WHERE id = $1
         RETURNING id, name, slug, description, list_type, created_by, is_active, is_listed,
                   flair_id, entity_kinds, created_at, updated_at",
        id.0,
        list_type.as_str(),
    )
    .fetch_optional(conn)
    .await?
    .map(Into::into))
}

pub async fn set_flair(
    conn: &mut Conn,
    id: TierListId,
    flair_id: Option<i16>,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE tier_lists SET flair_id = $2 WHERE id = $1",
        id.0,
        flair_id,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_list(conn: &mut Conn, id: TierListId) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM tier_lists WHERE id = $1", id.0)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn create_version(
    conn: &mut Conn,
    tier_list: TierListId,
    version: i32,
    snapshot: &Value,
    changelog: Option<&str>,
    published_by: UserId,
) -> Result<VersionRow, DbError> {
    Ok(sqlx::query_as!(
        VersionRow,
        "INSERT INTO tier_list_versions (tier_list_id, version, snapshot, changelog, published_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, tier_list_id, version, snapshot, changelog, published_by, published_at",
        tier_list.0,
        version,
        snapshot,
        changelog,
        published_by.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn get_versions(
    conn: &mut Conn,
    tier_list: TierListId,
) -> Result<Vec<VersionRow>, DbError> {
    Ok(sqlx::query_as!(
        VersionRow,
        "SELECT id, tier_list_id, version, snapshot, changelog, published_by, published_at
         FROM tier_list_versions WHERE tier_list_id = $1 ORDER BY version DESC",
        tier_list.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn latest_version(
    conn: &mut Conn,
    tier_list: TierListId,
) -> Result<Option<i32>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT MAX(version) FROM tier_list_versions WHERE tier_list_id = $1",
        tier_list.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn find_author(conn: &mut Conn, user: UserId) -> Result<Option<AuthorRow>, DbError> {
    Ok(sqlx::query_as!(
        AuthorRow,
        "SELECT id, uid, nickname, avatar_id FROM users WHERE id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_authors(conn: &mut Conn, users: &[UserId]) -> Result<Vec<AuthorRow>, DbError> {
    if users.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = users.iter().map(|u| u.0).collect();
    Ok(sqlx::query_as!(
        AuthorRow,
        "SELECT id, uid, nickname, avatar_id FROM users WHERE id = ANY($1)",
        &ids,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn active_slugs(conn: &mut Conn, slugs: &[String]) -> Result<Vec<String>, DbError> {
    if slugs.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_scalar!(
        "SELECT slug FROM tier_lists WHERE slug = ANY($1) AND is_active",
        slugs,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn site_counts(conn: &mut Conn) -> Result<SiteCounts, DbError> {
    Ok(sqlx::query_as!(
        SiteCounts,
        r#"SELECT
            (SELECT COUNT(*) FROM tier_lists) AS "total!",
            (SELECT COUNT(*) FROM tier_lists WHERE is_active = true) AS "active!",
            (SELECT COUNT(*) FROM tier_list_versions) AS "total_versions!",
            (SELECT COUNT(*) FROM tier_placements) AS "total_placements!""#,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn list_active_names(conn: &mut Conn) -> Result<Vec<ListNameRow>, DbError> {
    Ok(sqlx::query_as!(
        ListNameRow,
        "SELECT id, name, slug FROM tier_lists WHERE is_active = true ORDER BY name",
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_details(conn: &mut Conn) -> Result<Vec<ListDetailRow>, DbError> {
    Ok(sqlx::query_as!(
        ListDetailRow,
        r#"SELECT
              t.id, t.name, t.slug, t.is_active, t.is_listed, t.list_type,
              (SELECT COUNT(*) FROM tiers WHERE tier_list_id = t.id) AS "tier_count!",
              (SELECT COUNT(*) FROM tier_placements tp
                 JOIN tiers ti ON tp.tier_id = ti.id
                WHERE ti.tier_list_id = t.id) AS "placement_count!"
           FROM tier_lists t
           ORDER BY t.created_at DESC"#,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn find_visibility_by_slug(
    conn: &mut Conn,
    slug: &str,
) -> Result<Option<VisibilityRow>, DbError> {
    Ok(sqlx::query_as!(
        VisibilityRow,
        "SELECT id, name, is_listed FROM tier_lists WHERE slug = $1",
        slug,
    )
    .fetch_optional(conn)
    .await?)
}
