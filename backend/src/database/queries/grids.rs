use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::database::models::grid::{GridDocument, GridRow};

/// Every read: the row plus its owner's name, its fork count and its
/// template's slug, title, listing and owner. Callers append the `WHERE` and
/// `ORDER BY`.
const SELECT_ROWS: &str = "\
SELECT g.id, g.slug, g.title, g.description, g.created_by, g.rows, g.cols, g.cells,
       g.is_listed, g.entity_kinds, g.template_of, g.created_at, g.updated_at,
       COALESCE(NULLIF(u.nickname, ''), u.uid) AS owner_name,
       (SELECT COUNT(*) FROM grids f WHERE f.template_of = g.id) AS fork_count,
       t.slug AS template_slug, t.title AS template_title,
       t.is_listed AS template_listed, t.created_by AS template_owner
FROM grids g
JOIN users u ON u.id = g.created_by
LEFT JOIN grids t ON t.id = g.template_of";

/// How the public listing orders grids: latest edit first, or most forked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListOrder {
    #[default]
    Recent,
    Popular,
}

pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<GridRow>, sqlx::Error> {
    sqlx::query_as::<_, GridRow>(&format!("{SELECT_ROWS} WHERE g.slug = $1"))
        .bind(slug)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<GridRow>, sqlx::Error> {
    sqlx::query_as::<_, GridRow>(&format!(
        "{SELECT_ROWS} WHERE g.created_by = $1 ORDER BY g.updated_at DESC"
    ))
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// One page of listed grids whose title contains `pattern` (an `ILIKE`
/// pattern, already escaped), and the total that matched.
pub async fn find_listed(
    pool: &PgPool,
    pattern: Option<&str>,
    order: ListOrder,
    limit: i64,
    offset: i64,
) -> Result<(Vec<GridRow>, i64), sqlx::Error> {
    const FILTER: &str = r"g.is_listed AND ($1::text IS NULL OR g.title ILIKE $1 ESCAPE '\')";
    let order_by = match order {
        ListOrder::Recent => "g.updated_at DESC, g.id",
        ListOrder::Popular => "fork_count DESC, g.updated_at DESC, g.id",
    };
    let rows_sql = format!("{SELECT_ROWS} WHERE {FILTER} ORDER BY {order_by} LIMIT $2 OFFSET $3");
    let total_sql = format!("SELECT COUNT(*) FROM grids g WHERE {FILTER}");
    let rows_fut = sqlx::query_as::<_, GridRow>(&rows_sql)
        .bind(pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool);
    let total_fut = sqlx::query_scalar::<_, i64>(&total_sql)
        .bind(pattern)
        .fetch_one(pool);
    tokio::try_join!(rows_fut, total_fut)
}

/// Insert the grid unless `created_by` already owns `max` grids. The count and
/// the insert run in one transaction under a per-user advisory lock, so two
/// concurrent creates cannot both pass the cap. `false` when at the cap.
pub async fn create_capped(
    pool: &PgPool,
    slug: &str,
    doc: &GridDocument,
    created_by: Uuid,
    template_of: Option<Uuid>,
    max: i64,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
        .bind(created_by)
        .execute(&mut *tx)
        .await?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM grids WHERE created_by = $1")
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;
    if count >= max {
        return Ok(false);
    }
    sqlx::query(
        "INSERT INTO grids (slug, title, description, created_by, rows, cols, cells, is_listed, template_of, entity_kinds)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(slug)
    .bind(&doc.title)
    .bind(doc.description.as_deref())
    .bind(created_by)
    .bind(doc.rows)
    .bind(doc.cols)
    .bind(Json(&doc.cells))
    .bind(doc.is_listed)
    .bind(template_of)
    .bind(doc.kind_names())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn update(pool: &PgPool, id: Uuid, doc: &GridDocument) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE grids SET title = $2, description = $3, rows = $4, cols = $5, cells = $6, is_listed = $7,
             entity_kinds = $8
         WHERE id = $1",
    )
    .bind(id)
    .bind(&doc.title)
    .bind(doc.description.as_deref())
    .bind(doc.rows)
    .bind(doc.cols)
    .bind(Json(&doc.cells))
    .bind(doc.is_listed)
    .bind(doc.kind_names())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM grids WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
