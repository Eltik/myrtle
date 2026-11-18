use myrtle_types::{Server, UserId};
use serde_json::Value;
use sqlx::{AssertSqlSafe, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Table(String);

impl Table {
    #[must_use]
    pub const fn name(&self) -> &str {
        self.0.as_str()
    }

    fn qualified(&self) -> String {
        format!("public.{}", quote_ident(&self.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Column(String);

impl Column {
    #[must_use]
    pub const fn name(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct TableEstimate {
    pub table: Table,
    pub rows: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKey {
    pub child: String,
    pub parent: String,
}

#[derive(Debug, Clone)]
pub struct AccountRow {
    pub id: Uuid,
    pub uid: String,
    pub server_id: i16,
    pub server_code: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Owner,
    PlanGroup,
}

impl Scope {
    const fn filter(self) -> &'static str {
        match self {
            Self::Owner => "t.user_id = $1",
            Self::PlanGroup => {
                "t.plan_group_id IN (SELECT id FROM public.plan_groups WHERE user_id = $1)"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportGuard {
    MissingTierList,
}

impl ImportGuard {
    const fn keep(self) -> &'static str {
        match self {
            Self::MissingTierList => "tier_list_id IN (SELECT id FROM public.tier_lists)",
        }
    }
}

pub async fn table_estimates(conn: &mut Conn) -> Result<Vec<TableEstimate>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT relname::text AS "name!", reltuples AS "reltuples!" FROM pg_catalog.pg_class
           WHERE relnamespace = 'public'::regnamespace AND relkind = 'r'"#
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| TableEstimate {
            table: Table(r.name),
            rows: r.reltuples,
        })
        .collect())
}

pub async fn table(conn: &mut Conn, name: &str) -> Result<Option<Table>, DbError> {
    let name = sqlx::query_scalar!(
        r#"SELECT relname::text AS "name!" FROM pg_catalog.pg_class
           WHERE relnamespace = 'public'::regnamespace AND relkind = 'r' AND relname = $1::text"#,
        name,
    )
    .fetch_optional(conn)
    .await?;
    Ok(name.map(Table))
}

pub async fn columns(conn: &mut Conn, table: &Table) -> Result<Vec<Column>, DbError> {
    let names = sqlx::query_scalar!(
        r#"SELECT a.attname::text AS "name!" FROM pg_catalog.pg_attribute a
           WHERE a.attrelid = $1::text::regclass AND a.attnum > 0 AND NOT a.attisdropped
           ORDER BY a.attnum"#,
        table.qualified(),
    )
    .fetch_all(conn)
    .await?;
    Ok(names.into_iter().map(Column).collect())
}

pub async fn foreign_keys(conn: &mut Conn) -> Result<Vec<ForeignKey>, DbError> {
    Ok(sqlx::query_as!(
        ForeignKey,
        r#"SELECT DISTINCT conrelid::regclass::text AS "child!",
                           confrelid::regclass::text AS "parent!"
           FROM pg_catalog.pg_constraint
           WHERE contype = 'f' AND connamespace = 'public'::regnamespace
             AND conrelid <> confrelid"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn begin_snapshot(conn: &mut Conn) -> Result<Transaction<'_, Postgres>, DbError> {
    Ok(
        sqlx::Connection::begin_with(conn, "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .await?,
    )
}

pub async fn server_version(conn: &mut Conn) -> Result<String, DbError> {
    Ok(
        sqlx::query_scalar!(r#"SELECT current_setting('server_version') AS "version!""#)
            .fetch_one(conn)
            .await?,
    )
}

pub async fn suspend_triggers(conn: &mut Conn) -> Result<(), DbError> {
    sqlx::raw_sql("SET LOCAL session_replication_role = replica")
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn count_rows(conn: &mut Conn, table: &Table) -> Result<i64, DbError> {
    let sql = format!("SELECT COUNT(*) FROM {}", table.qualified());
    let count: i64 = sqlx::query_scalar(AssertSqlSafe(sql))
        .fetch_one(conn)
        .await?;
    Ok(count)
}

pub async fn declare_json_rows(conn: &mut Conn, table: &Table) -> Result<(), DbError> {
    let sql = format!(
        "DECLARE json_rows NO SCROLL CURSOR FOR SELECT row_to_json(t)::text FROM {} t",
        table.qualified()
    );
    sqlx::raw_sql(AssertSqlSafe(sql)).execute(conn).await?;
    Ok(())
}

pub async fn fetch_json_rows(conn: &mut Conn) -> Result<Vec<String>, DbError> {
    let rows = sqlx::raw_sql("FETCH FORWARD 1000 FROM json_rows")
        .fetch_all(&mut *conn)
        .await?;
    if rows.is_empty() {
        sqlx::raw_sql("CLOSE json_rows").execute(conn).await?;
    }
    Ok(rows
        .iter()
        .map(|r| r.try_get(0))
        .collect::<Result<Vec<String>, sqlx::Error>>()?)
}

pub async fn truncate_cascade(conn: &mut Conn, tables: &[Table]) -> Result<(), DbError> {
    if tables.is_empty() {
        return Ok(());
    }
    let list = tables
        .iter()
        .map(Table::qualified)
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("TRUNCATE {list} RESTART IDENTITY CASCADE");
    sqlx::query(AssertSqlSafe(sql)).execute(conn).await?;
    Ok(())
}

pub async fn delete_all(conn: &mut Conn, table: &Table) -> Result<u64, DbError> {
    let sql = format!("DELETE FROM {}", table.qualified());
    let result = sqlx::query(AssertSqlSafe(sql)).execute(conn).await?;
    Ok(result.rows_affected())
}

pub async fn restore_rows(conn: &mut Conn, table: &Table, rows: &Value) -> Result<u64, DbError> {
    let t = table.qualified();
    let sql =
        format!("INSERT INTO {t} SELECT * FROM jsonb_populate_recordset(NULL::{t}, $1::jsonb)");
    let result = sqlx::query(AssertSqlSafe(sql))
        .bind(rows)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

pub async fn reset_sequence(
    conn: &mut Conn,
    table: &Table,
    column: &Column,
) -> Result<(), DbError> {
    let (t, c) = (table.qualified(), quote_ident(&column.0));
    let sql = format!(
        "SELECT setval(pg_get_serial_sequence($1, $2),
                       COALESCE((SELECT MAX({c}) FROM {t}), 1),
                       (SELECT MAX({c}) FROM {t}) IS NOT NULL)"
    );
    sqlx::query(AssertSqlSafe(sql))
        .bind(&t) // parsed as an identifier, the column argument taken literally
        .bind(&column.0)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn find_account(conn: &mut Conn, user: UserId) -> Result<Option<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server_code, u.nickname
         FROM public.users u JOIN public.servers s ON s.id = u.server_id
         WHERE u.id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_accounts_by_uid(conn: &mut Conn, uid: &str) -> Result<Vec<AccountRow>, DbError> {
    Ok(sqlx::query_as!(
        AccountRow,
        "SELECT u.id, u.uid, u.server_id, s.code AS server_code, u.nickname
         FROM public.users u JOIN public.servers s ON s.id = u.server_id
         WHERE u.uid = $1 ORDER BY u.created_at",
        uid,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn user_json(conn: &mut Conn, user: UserId) -> Result<String, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT row_to_json(t)::text AS "row!" FROM public.users t WHERE t.id = $1"#,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn scoped_json(
    conn: &mut Conn,
    table: &Table,
    scope: Scope,
    user: UserId,
) -> Result<String, DbError> {
    let sql = format!(
        "SELECT COALESCE(json_agg(j ORDER BY j::text), '[]'::json)::text
         FROM (SELECT row_to_json(t) AS j FROM {} t WHERE {}) q",
        table.qualified(),
        scope.filter()
    );
    let json: String = sqlx::query_scalar(AssertSqlSafe(sql))
        .bind(user.0)
        .fetch_one(conn)
        .await?;
    Ok(json)
}

pub async fn count_scoped(
    conn: &mut Conn,
    table: &Table,
    scope: Scope,
    user: UserId,
) -> Result<i64, DbError> {
    let sql = format!(
        "SELECT COUNT(*) FROM {} t WHERE {}",
        table.qualified(),
        scope.filter()
    );
    let count: i64 = sqlx::query_scalar(AssertSqlSafe(sql))
        .bind(user.0)
        .fetch_one(conn)
        .await?;
    Ok(count)
}

pub async fn delete_scoped(
    conn: &mut Conn,
    table: &Table,
    scope: Scope,
    user: UserId,
) -> Result<u64, DbError> {
    let sql = format!(
        "DELETE FROM {} t WHERE {}",
        table.qualified(),
        scope.filter()
    );
    let result = sqlx::query(AssertSqlSafe(sql))
        .bind(user.0)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

pub async fn find_user_id(
    conn: &mut Conn,
    uid: &str,
    server: Server,
) -> Result<Option<UserId>, DbError> {
    let id = sqlx::query_scalar!(
        "SELECT id FROM public.users WHERE uid = $1 AND server_id = $2",
        uid,
        server.index(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(id.map(UserId))
}

pub async fn user_id_taken(conn: &mut Conn, id: UserId) -> Result<bool, DbError> {
    let found = sqlx::query_scalar!("SELECT id FROM public.users WHERE id = $1", id.0)
        .fetch_optional(conn)
        .await?;
    Ok(found.is_some())
}

pub async fn upsert_user_json(
    conn: &mut Conn,
    columns: &[Column],
    row: &Value,
) -> Result<UserId, DbError> {
    let list = column_list(columns);
    let updates: Vec<String> = columns
        .iter()
        .filter(|c| !matches!(c.0.as_str(), "id" | "uid" | "server_id" | "created_at"))
        .map(|c| {
            let c = quote_ident(&c.0);
            format!("{c} = EXCLUDED.{c}")
        })
        .collect();
    let conflict = if updates.is_empty() {
        "DO NOTHING".to_owned()
    } else {
        format!("DO UPDATE SET {}", updates.join(", "))
    };
    let records = Value::Array(vec![row.clone()]);
    let upserted: Option<Uuid> = sqlx::query_scalar(AssertSqlSafe(format!(
        "INSERT INTO public.users ({list})
         SELECT {list} FROM jsonb_populate_recordset(NULL::public.users, $1::jsonb)
         ON CONFLICT (uid, server_id) {conflict} RETURNING id"
    )))
    .bind(&records)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(id) = upserted {
        return Ok(UserId(id));
    }
    let existing = sqlx::query_scalar!(
        "SELECT u.id FROM public.users u WHERE (u.uid, u.server_id) = (
             SELECT r.uid, r.server_id FROM jsonb_populate_record(NULL::public.users, $1::jsonb -> 0) r
         )",
        &records,
    )
    .fetch_optional(conn)
    .await?;
    existing.map(UserId).ok_or(DbError::NotFound)
}

pub async fn insert_rows(
    conn: &mut Conn,
    table: &Table,
    columns: &[Column],
    guard: Option<ImportGuard>,
    rows: &Value,
) -> Result<u64, DbError> {
    let t = table.qualified();
    let list = column_list(columns);
    let guard = guard.map_or_else(String::new, |g| format!(" WHERE {}", g.keep()));
    let sql = format!(
        "INSERT INTO {t} ({list})
         SELECT {list} FROM jsonb_populate_recordset(NULL::{t}, $1::jsonb){guard}"
    );
    let result = sqlx::query(AssertSqlSafe(sql))
        .bind(rows)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

fn column_list(columns: &[Column]) -> String {
    columns
        .iter()
        .map(|c| quote_ident(&c.0))
        .collect::<Vec<_>>()
        .join(", ")
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
