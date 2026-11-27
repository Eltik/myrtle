use chrono::{DateTime, Utc};
use myrtle_types::{EntityKind, GridId, UserId};
use serde_json::Value;
use uuid::Uuid;

use super::kind_names;
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct GridRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub rows: i16,
    pub cols: i16,
    pub cells: Value,
    pub is_listed: bool,
    pub entity_kinds: Vec<String>,
    pub template_of: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Nickname, else uid
    pub owner_name: String,
    pub fork_count: i64,
    pub template_slug: Option<String>,
    pub template_title: Option<String>,
    pub template_listed: Option<bool>,
    pub template_owner: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct GridDoc<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub rows: i16,
    pub cols: i16,
    pub cells: &'a Value,
    pub is_listed: bool,
    pub entity_kinds: &'a [EntityKind],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ListOrder {
    #[default]
    Recent,
    Popular,
}

pub async fn find_by_slug(conn: &mut Conn, slug: &str) -> Result<Option<GridRow>, DbError> {
    Ok(sqlx::query_as!(
        GridRow,
        r#"SELECT g.id, g.slug, g.title, g.description, g.created_by, g.rows, g.cols, g.cells,
                  g.is_listed, g.entity_kinds, g.template_of, g.created_at, g.updated_at,
                  COALESCE(NULLIF(u.nickname, ''), u.uid) AS "owner_name!",
                  (SELECT COUNT(*) FROM grids f WHERE f.template_of = g.id) AS "fork_count!",
                  t.slug AS "template_slug?", t.title AS "template_title?",
                  t.is_listed AS "template_listed?", t.created_by AS "template_owner?"
           FROM grids g
           JOIN users u ON u.id = g.created_by
           LEFT JOIN grids t ON t.id = g.template_of
           WHERE g.slug = $1"#,
        slug,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_by_user(conn: &mut Conn, user: UserId) -> Result<Vec<GridRow>, DbError> {
    Ok(sqlx::query_as!(
        GridRow,
        r#"SELECT g.id, g.slug, g.title, g.description, g.created_by, g.rows, g.cols, g.cells,
                  g.is_listed, g.entity_kinds, g.template_of, g.created_at, g.updated_at,
                  COALESCE(NULLIF(u.nickname, ''), u.uid) AS "owner_name!",
                  (SELECT COUNT(*) FROM grids f WHERE f.template_of = g.id) AS "fork_count!",
                  t.slug AS "template_slug?", t.title AS "template_title?",
                  t.is_listed AS "template_listed?", t.created_by AS "template_owner?"
           FROM grids g
           JOIN users u ON u.id = g.created_by
           LEFT JOIN grids t ON t.id = g.template_of
           WHERE g.created_by = $1 ORDER BY g.updated_at DESC"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn find_listed(
    conn: &mut Conn,
    pattern: Option<&str>,
    order: ListOrder,
    limit: i64,
    offset: i64,
) -> Result<(Vec<GridRow>, i64), DbError> {
    let rows = match order {
        ListOrder::Recent => {
            sqlx::query_as!(
                GridRow,
                r#"SELECT g.id, g.slug, g.title, g.description, g.created_by, g.rows, g.cols,
                          g.cells, g.is_listed, g.entity_kinds, g.template_of, g.created_at,
                          g.updated_at,
                          COALESCE(NULLIF(u.nickname, ''), u.uid) AS "owner_name!",
                          (SELECT COUNT(*) FROM grids f WHERE f.template_of = g.id)
                            AS "fork_count!",
                          t.slug AS "template_slug?", t.title AS "template_title?",
                          t.is_listed AS "template_listed?", t.created_by AS "template_owner?"
                   FROM (
                       SELECT g.id, g.updated_at
                       FROM grids g
                       WHERE g.is_listed AND ($1::text IS NULL OR g.title ILIKE $1 ESCAPE '\')
                       ORDER BY g.updated_at DESC, g.id
                       LIMIT $2 OFFSET $3
                   ) p
                   JOIN grids g ON g.id = p.id
                   JOIN users u ON u.id = g.created_by
                   LEFT JOIN grids t ON t.id = g.template_of
                   ORDER BY p.updated_at DESC, p.id"#,
                pattern,
                limit,
                offset,
            )
            .fetch_all(&mut *conn)
            .await?
        }
        ListOrder::Popular => {
            sqlx::query_as!(
                GridRow,
                r#"SELECT g.id, g.slug, g.title, g.description, g.created_by, g.rows, g.cols,
                          g.cells, g.is_listed, g.entity_kinds, g.template_of, g.created_at,
                          g.updated_at,
                          COALESCE(NULLIF(u.nickname, ''), u.uid) AS "owner_name!",
                          p.fork_count AS "fork_count!",
                          t.slug AS "template_slug?", t.title AS "template_title?",
                          t.is_listed AS "template_listed?", t.created_by AS "template_owner?"
                   FROM (
                       SELECT g.id, g.updated_at,
                              (SELECT COUNT(*) FROM grids f WHERE f.template_of = g.id) AS fork_count
                       FROM grids g
                       WHERE g.is_listed AND ($1::text IS NULL OR g.title ILIKE $1 ESCAPE '\')
                       ORDER BY fork_count DESC, g.updated_at DESC, g.id
                       LIMIT $2 OFFSET $3
                   ) p
                   JOIN grids g ON g.id = p.id
                   JOIN users u ON u.id = g.created_by
                   LEFT JOIN grids t ON t.id = g.template_of
                   ORDER BY p.fork_count DESC, p.updated_at DESC, p.id"#,
                pattern,
                limit,
                offset,
            )
            .fetch_all(&mut *conn)
            .await?
        }
    };
    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM grids g
           WHERE g.is_listed AND ($1::text IS NULL OR g.title ILIKE $1 ESCAPE '\')"#,
        pattern,
    )
    .fetch_one(conn)
    .await?;
    Ok((rows, total))
}

pub async fn create_capped(
    conn: &mut Conn,
    slug: &str,
    doc: GridDoc<'_>,
    created_by: UserId,
    template_of: Option<GridId>,
    max: i64,
) -> Result<bool, DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    sqlx::query!(
        "SELECT pg_advisory_xact_lock(hashtext($1::uuid::text))",
        created_by.0,
    )
    .execute(&mut *tx)
    .await?;
    let inserted = sqlx::query!(
        "INSERT INTO grids (slug, title, description, created_by, rows, cols, cells, is_listed,
                            template_of, entity_kinds)
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
         WHERE (SELECT COUNT(*) FROM grids WHERE created_by = $4) < $11",
        slug,
        doc.title,
        doc.description,
        created_by.0,
        doc.rows,
        doc.cols,
        doc.cells,
        doc.is_listed,
        template_of.map(|t| t.0),
        &kind_names(doc.entity_kinds),
        max,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    Ok(inserted > 0)
}

pub async fn update(conn: &mut Conn, id: GridId, doc: GridDoc<'_>) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE grids SET title = $2, description = $3, rows = $4, cols = $5, cells = $6,
                          is_listed = $7, entity_kinds = $8
         WHERE id = $1",
        id.0,
        doc.title,
        doc.description,
        doc.rows,
        doc.cols,
        doc.cells,
        doc.is_listed,
        &kind_names(doc.entity_kinds),
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete(conn: &mut Conn, id: GridId) -> Result<(), DbError> {
    sqlx::query!("DELETE FROM grids WHERE id = $1", id.0)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn existing_slugs(conn: &mut Conn, slugs: &[String]) -> Result<Vec<String>, DbError> {
    Ok(
        sqlx::query_scalar!("SELECT slug FROM grids WHERE slug = ANY($1)", slugs)
            .fetch_all(conn)
            .await?,
    )
}
