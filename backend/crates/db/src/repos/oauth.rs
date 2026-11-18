use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct AppRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub redirect_uris: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authorized_users: i64,
}

#[derive(Debug, Clone)]
pub struct AppPublicRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub redirect_uris: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub owner_nickname: Option<String>,
    pub owner_nick_number: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthorizationRow {
    pub app: AppPublicRow,
    pub scope: String,
    pub authorized_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ConsumedCodeRow {
    pub grant_id: Uuid,
    pub redirect_uri: String,
    pub scope: String,
    pub code_challenge: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AccessTokenOwnerRow {
    pub user_id: Uuid,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct IdentityRow {
    pub id: Uuid,
    pub uid: String,
    pub server: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
}

pub async fn list_apps(conn: &mut Conn, owner: UserId) -> Result<Vec<AppRow>, DbError> {
    Ok(sqlx::query_as!(
        AppRow,
        r#"SELECT a.id, a.name, a.description, a.redirect_uris, a.created_at, a.updated_at,
                  (SELECT COUNT(*) FROM oauth_grants g WHERE g.app_id = a.id) AS "authorized_users!"
           FROM oauth_apps a WHERE a.owner_id = $1 ORDER BY a.created_at, a.id"#,
        owner.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_apps(conn: &mut Conn, owner: UserId) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM oauth_apps WHERE owner_id = $1"#,
        owner.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn insert_app(
    conn: &mut Conn,
    owner: UserId,
    name: &str,
    description: &str,
    redirect_uris: &[String],
    secret_hash: &[u8],
) -> Result<AppRow, DbError> {
    Ok(sqlx::query_as!(
        AppRow,
        r#"INSERT INTO oauth_apps (owner_id, name, description, redirect_uris, secret_hash)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, name, description, redirect_uris, created_at, updated_at,
                     (SELECT COUNT(*) FROM oauth_grants g WHERE g.app_id = oauth_apps.id)
                       AS "authorized_users!""#,
        owner.0,
        name,
        description,
        redirect_uris,
        secret_hash,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn update_app(
    conn: &mut Conn,
    app_id: Uuid,
    owner: UserId,
    name: &str,
    description: &str,
    redirect_uris: &[String],
) -> Result<Option<AppRow>, DbError> {
    Ok(sqlx::query_as!(
        AppRow,
        r#"UPDATE oauth_apps a SET name = $3, description = $4, redirect_uris = $5
           WHERE a.id = $1 AND a.owner_id = $2
           RETURNING a.id, a.name, a.description, a.redirect_uris, a.created_at, a.updated_at,
                     (SELECT COUNT(*) FROM oauth_grants g WHERE g.app_id = a.id)
                       AS "authorized_users!""#,
        app_id,
        owner.0,
        name,
        description,
        redirect_uris,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn replace_secret(
    conn: &mut Conn,
    app_id: Uuid,
    owner: UserId,
    secret_hash: &[u8],
) -> Result<Option<AppRow>, DbError> {
    Ok(sqlx::query_as!(
        AppRow,
        r#"UPDATE oauth_apps a SET secret_hash = $3
           WHERE a.id = $1 AND a.owner_id = $2
           RETURNING a.id, a.name, a.description, a.redirect_uris, a.created_at, a.updated_at,
                     (SELECT COUNT(*) FROM oauth_grants g WHERE g.app_id = a.id)
                       AS "authorized_users!""#,
        app_id,
        owner.0,
        secret_hash,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn delete_app(
    conn: &mut Conn,
    app_id: Uuid,
    owner: Option<UserId>,
) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM oauth_apps WHERE id = $1 AND ($2::uuid IS NULL OR owner_id = $2)",
        app_id,
        owner.map(|o| o.0),
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn find_public_app(
    conn: &mut Conn,
    app_id: Uuid,
) -> Result<Option<AppPublicRow>, DbError> {
    Ok(sqlx::query_as!(
        AppPublicRow,
        "SELECT a.id, a.name, a.description, a.redirect_uris, a.created_at,
                o.nickname AS owner_nickname, o.nick_number AS owner_nick_number
         FROM oauth_apps a JOIN users o ON o.id = a.owner_id
         WHERE a.id = $1",
        app_id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_secret_hash(conn: &mut Conn, app_id: Uuid) -> Result<Option<Vec<u8>>, DbError> {
    Ok(
        sqlx::query_scalar!("SELECT secret_hash FROM oauth_apps WHERE id = $1", app_id)
            .fetch_optional(conn)
            .await?,
    )
}

pub async fn has_grant(conn: &mut Conn, app_id: Uuid, user: UserId) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM oauth_grants WHERE app_id = $1 AND user_id = $2)
             AS "exists!""#,
        app_id,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn upsert_grant(
    conn: &mut Conn,
    app_id: Uuid,
    user: UserId,
    scope: &str,
) -> Result<Uuid, DbError> {
    Ok(sqlx::query_scalar!(
        "INSERT INTO oauth_grants (app_id, user_id, scope) VALUES ($1, $2, $3)
         ON CONFLICT (app_id, user_id) DO UPDATE SET scope = EXCLUDED.scope
         RETURNING id",
        app_id,
        user.0,
        scope,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn insert_code(
    conn: &mut Conn,
    code_hash: &[u8],
    grant_id: Uuid,
    redirect_uri: &str,
    scope: &str,
    code_challenge: Option<&str>,
    ttl_secs: i64,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO oauth_codes
             (code_hash, grant_id, redirect_uri, scope, code_challenge, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $6))",
        code_hash,
        grant_id,
        redirect_uri,
        scope,
        code_challenge,
        ttl_secs as f64,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn consume_code(
    conn: &mut Conn,
    code_hash: &[u8],
    app_id: Uuid,
) -> Result<Option<ConsumedCodeRow>, DbError> {
    Ok(sqlx::query_as!(
        ConsumedCodeRow,
        "UPDATE oauth_codes c SET consumed_at = now()
         FROM oauth_grants g
         WHERE c.code_hash = $1 AND g.id = c.grant_id AND g.app_id = $2
           AND c.consumed_at IS NULL AND c.expires_at > now()
         RETURNING c.grant_id, c.redirect_uri, c.scope, c.code_challenge",
        code_hash,
        app_id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_spent_code_grant(
    conn: &mut Conn,
    code_hash: &[u8],
    app_id: Uuid,
) -> Result<Option<Uuid>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT c.grant_id FROM oauth_codes c JOIN oauth_grants g ON g.id = c.grant_id
         WHERE c.code_hash = $1 AND g.app_id = $2 AND c.consumed_at IS NOT NULL",
        code_hash,
        app_id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn delete_grant_tokens(conn: &mut Conn, grant_id: Uuid) -> Result<u64, DbError> {
    let result = sqlx::query!("DELETE FROM oauth_tokens WHERE grant_id = $1", grant_id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_token(
    conn: &mut Conn,
    grant_id: Uuid,
    access_hash: &[u8],
    refresh_hash: &[u8],
    scope: &str,
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
    keep: i64,
) -> Result<(), DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    sqlx::query!(
        "INSERT INTO oauth_tokens
             (grant_id, access_hash, refresh_hash, scope, access_expires_at, refresh_expires_at)
         VALUES ($1, $2, $3, $4,
                 now() + make_interval(secs => $5), now() + make_interval(secs => $6))",
        grant_id,
        access_hash,
        refresh_hash,
        scope,
        access_ttl_secs as f64,
        refresh_ttl_secs as f64,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "WITH pruned AS (
             DELETE FROM oauth_tokens WHERE id IN (
                 SELECT id FROM oauth_tokens WHERE grant_id = $1
                 ORDER BY created_at DESC, id OFFSET $2
             )
         )
         UPDATE oauth_grants SET last_used_at = now() WHERE id = $1",
        grant_id,
        keep,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn rotate_token(
    conn: &mut Conn,
    refresh_hash: &[u8],
    app_id: Uuid,
    new_access_hash: &[u8],
    new_refresh_hash: &[u8],
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
) -> Result<Option<String>, DbError> {
    Ok(sqlx::query_scalar!(
        "WITH rotated AS (
             UPDATE oauth_tokens t
             SET access_hash = $3, refresh_hash = $4,
                 access_expires_at = now() + make_interval(secs => $5),
                 refresh_expires_at = now() + make_interval(secs => $6)
             FROM oauth_grants g
             WHERE t.refresh_hash = $1 AND g.id = t.grant_id AND g.app_id = $2
               AND t.refresh_expires_at > now()
             RETURNING t.grant_id, t.scope
         ), stamped AS (
             UPDATE oauth_grants SET last_used_at = now()
             WHERE id IN (SELECT grant_id FROM rotated)
         )
         SELECT scope FROM rotated",
        refresh_hash,
        app_id,
        new_access_hash,
        new_refresh_hash,
        access_ttl_secs as f64,
        refresh_ttl_secs as f64,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn revoke_token(
    conn: &mut Conn,
    token_hash: &[u8],
    app_id: Uuid,
) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM oauth_tokens t USING oauth_grants g
         WHERE g.id = t.grant_id AND g.app_id = $2
           AND (t.access_hash = $1 OR t.refresh_hash = $1)",
        token_hash,
        app_id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn find_access_token(
    conn: &mut Conn,
    access_hash: &[u8],
) -> Result<Option<AccessTokenOwnerRow>, DbError> {
    Ok(sqlx::query_as!(
        AccessTokenOwnerRow,
        "SELECT g.user_id, t.scope
         FROM oauth_tokens t JOIN oauth_grants g ON g.id = t.grant_id
         WHERE t.access_hash = $1 AND t.access_expires_at > now()",
        access_hash,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn find_identity(conn: &mut Conn, user: UserId) -> Result<Option<IdentityRow>, DbError> {
    Ok(sqlx::query_as!(
        IdentityRow,
        "SELECT u.id, u.uid, s.code AS server, u.nickname, u.nick_number, u.avatar_id
         FROM users u JOIN servers s ON s.id = u.server_id
         WHERE u.id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn list_authorizations(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<AuthorizationRow>, DbError> {
    let rows = sqlx::query!(
        "SELECT a.id, a.name, a.description, a.redirect_uris, a.created_at,
                o.nickname AS owner_nickname, o.nick_number AS owner_nick_number,
                g.scope, g.created_at AS authorized_at, g.last_used_at
         FROM oauth_grants g
         JOIN oauth_apps a ON a.id = g.app_id
         JOIN users o ON o.id = a.owner_id
         WHERE g.user_id = $1
         ORDER BY COALESCE(g.last_used_at, g.created_at) DESC, g.id",
        user.0,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| AuthorizationRow {
            app: AppPublicRow {
                id: r.id,
                name: r.name,
                description: r.description,
                redirect_uris: r.redirect_uris,
                created_at: r.created_at,
                owner_nickname: r.owner_nickname,
                owner_nick_number: r.owner_nick_number,
            },
            scope: r.scope,
            authorized_at: r.authorized_at,
            last_used_at: r.last_used_at,
        })
        .collect())
}

pub async fn delete_grant(conn: &mut Conn, app_id: Uuid, user: UserId) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM oauth_grants WHERE app_id = $1 AND user_id = $2",
        app_id,
        user.0,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_expired(conn: &mut Conn) -> Result<u64, DbError> {
    let swept = sqlx::query_scalar!(
        r#"WITH codes AS (DELETE FROM oauth_codes WHERE expires_at < now() RETURNING 1),
                tokens AS (DELETE FROM oauth_tokens WHERE refresh_expires_at < now() RETURNING 1)
           SELECT (SELECT COUNT(*) FROM codes) + (SELECT COUNT(*) FROM tokens) AS "swept!""#
    )
    .fetch_one(conn)
    .await?;
    Ok(u64::try_from(swept).unwrap_or_default())
}
