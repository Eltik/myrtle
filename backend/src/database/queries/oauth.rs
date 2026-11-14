//! Storage for the OAuth 2.0 provider (`v046_oauth`). Every secret arrives
//! here already hashed (`core::auth::oauth::digest`); nothing in this file
//! sees one in the clear.
//!
//! Functions take a connection rather than the pool so a caller, or a test,
//! can run them inside its own transaction.

use sqlx::PgConnection;
use uuid::Uuid;

use crate::database::models::oauth::{
    AccessTokenOwner, ConsumedCode, OAuthApp, OAuthAppPublicRow, OAuthAuthorizationRow,
    OAuthIdentityRow,
};

/// `oauth_apps` with its live authorization count, as its owner reads it.
const APP_COLUMNS: &str = "a.id, a.name, a.description, a.redirect_uris, a.created_at, \
     a.updated_at, \
     (SELECT COUNT(*) FROM oauth_grants g WHERE g.app_id = a.id) AS authorized_users";

/// `oauth_apps` with the nickname of the account that registered it.
const PUBLIC_APP_COLUMNS: &str = "a.id, a.name, a.description, a.redirect_uris, a.created_at, \
     o.nickname AS owner_nickname, o.nick_number AS owner_nick_number";

pub async fn list_apps(
    conn: &mut PgConnection,
    owner_id: Uuid,
) -> Result<Vec<OAuthApp>, sqlx::Error> {
    sqlx::query_as::<_, OAuthApp>(&format!(
        "SELECT {APP_COLUMNS} FROM oauth_apps a WHERE a.owner_id = $1 ORDER BY a.created_at, a.id"
    ))
    .bind(owner_id)
    .fetch_all(conn)
    .await
}

pub async fn count_apps(conn: &mut PgConnection, owner_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM oauth_apps WHERE owner_id = $1")
        .bind(owner_id)
        .fetch_one(conn)
        .await
}

pub async fn insert_app(
    conn: &mut PgConnection,
    owner_id: Uuid,
    name: &str,
    description: &str,
    redirect_uris: &[String],
    secret_hash: &[u8],
) -> Result<OAuthApp, sqlx::Error> {
    sqlx::query_as::<_, OAuthApp>(&format!(
        "WITH a AS (
             INSERT INTO oauth_apps (owner_id, name, description, redirect_uris, secret_hash)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *
         )
         SELECT {APP_COLUMNS} FROM a"
    ))
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(redirect_uris)
    .bind(secret_hash)
    .fetch_one(conn)
    .await
}

/// Replace the editable fields of an application `owner_id` registered.
/// `None` when they have no such application.
pub async fn update_app(
    conn: &mut PgConnection,
    app_id: Uuid,
    owner_id: Uuid,
    name: &str,
    description: &str,
    redirect_uris: &[String],
) -> Result<Option<OAuthApp>, sqlx::Error> {
    sqlx::query_as::<_, OAuthApp>(&format!(
        "WITH a AS (
             UPDATE oauth_apps SET name = $3, description = $4, redirect_uris = $5
             WHERE id = $1 AND owner_id = $2
             RETURNING *
         )
         SELECT {APP_COLUMNS} FROM a"
    ))
    .bind(app_id)
    .bind(owner_id)
    .bind(name)
    .bind(description)
    .bind(redirect_uris)
    .fetch_optional(conn)
    .await
}

/// Replace an application's secret. `None` when `owner_id` has no such
/// application. Tokens already issued are untouched: the secret authenticates
/// the application, not the sign-ins it has made.
pub async fn replace_secret(
    conn: &mut PgConnection,
    app_id: Uuid,
    owner_id: Uuid,
    secret_hash: &[u8],
) -> Result<Option<OAuthApp>, sqlx::Error> {
    sqlx::query_as::<_, OAuthApp>(&format!(
        "WITH a AS (
             UPDATE oauth_apps SET secret_hash = $3
             WHERE id = $1 AND owner_id = $2
             RETURNING *
         )
         SELECT {APP_COLUMNS} FROM a"
    ))
    .bind(app_id)
    .bind(owner_id)
    .bind(secret_hash)
    .fetch_optional(conn)
    .await
}

/// Delete an application and, by cascade, every grant, code and token under
/// it. `owner_id` limits the delete to that account's applications; `None`
/// deletes whoever owns it, which is the super-admin path. `true` when a row
/// went.
pub async fn delete_app(
    conn: &mut PgConnection,
    app_id: Uuid,
    owner_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM oauth_apps WHERE id = $1 AND ($2::uuid IS NULL OR owner_id = $2)")
            .bind(app_id)
            .bind(owner_id)
            .execute(conn)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// An application as the consent screen shows it, with its redirect URIs for
/// the request to be checked against.
pub async fn find_public_app(
    conn: &mut PgConnection,
    app_id: Uuid,
) -> Result<Option<OAuthAppPublicRow>, sqlx::Error> {
    sqlx::query_as::<_, OAuthAppPublicRow>(&format!(
        "SELECT {PUBLIC_APP_COLUMNS}
         FROM oauth_apps a JOIN users o ON o.id = a.owner_id
         WHERE a.id = $1"
    ))
    .bind(app_id)
    .fetch_optional(conn)
    .await
}

/// The stored hash of an application's secret, for authenticating it at the
/// token endpoint.
pub async fn find_secret_hash(
    conn: &mut PgConnection,
    app_id: Uuid,
) -> Result<Option<Vec<u8>>, sqlx::Error> {
    sqlx::query_scalar("SELECT secret_hash FROM oauth_apps WHERE id = $1")
        .bind(app_id)
        .fetch_optional(conn)
        .await
}

pub async fn has_grant(
    conn: &mut PgConnection,
    app_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM oauth_grants WHERE app_id = $1 AND user_id = $2)",
    )
    .bind(app_id)
    .bind(user_id)
    .fetch_one(conn)
    .await
}

/// Record that `user_id` approved `scope` for an application, returning the
/// grant's id. Approving again keeps the row (and `created_at`, the day they
/// first authorized) and replaces the scope with what was just approved.
pub async fn upsert_grant(
    conn: &mut PgConnection,
    app_id: Uuid,
    user_id: Uuid,
    scope: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO oauth_grants (app_id, user_id, scope) VALUES ($1, $2, $3)
         ON CONFLICT (app_id, user_id) DO UPDATE SET scope = EXCLUDED.scope
         RETURNING id",
    )
    .bind(app_id)
    .bind(user_id)
    .bind(scope)
    .fetch_one(conn)
    .await
}

pub async fn insert_code(
    conn: &mut PgConnection,
    code_hash: &[u8],
    grant_id: Uuid,
    redirect_uri: &str,
    scope: &str,
    code_challenge: Option<&str>,
    ttl_secs: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_codes (code_hash, grant_id, redirect_uri, scope, code_challenge, expires_at)
         VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $6))",
    )
    .bind(code_hash)
    .bind(grant_id)
    .bind(redirect_uri)
    .bind(scope)
    .bind(code_challenge)
    .bind(ttl_secs as f64)
    .execute(conn)
    .await?;
    Ok(())
}

/// Spend an authorization code: one statement, so of two exchanges racing on
/// one code exactly one gets the row. `None` for a code that is unknown,
/// expired, already spent, or was issued to another application (which
/// therefore cannot burn it).
pub async fn consume_code(
    conn: &mut PgConnection,
    code_hash: &[u8],
    app_id: Uuid,
) -> Result<Option<ConsumedCode>, sqlx::Error> {
    sqlx::query_as::<_, ConsumedCode>(
        "UPDATE oauth_codes c SET consumed_at = now()
         FROM oauth_grants g
         WHERE c.code_hash = $1 AND g.id = c.grant_id AND g.app_id = $2
           AND c.consumed_at IS NULL AND c.expires_at > now()
         RETURNING c.grant_id, c.redirect_uri, c.scope, c.code_challenge",
    )
    .bind(code_hash)
    .bind(app_id)
    .fetch_optional(conn)
    .await
}

/// The grant behind a code this application already spent, if that is what
/// `code_hash` is: the sign of a replayed code.
pub async fn find_spent_code_grant(
    conn: &mut PgConnection,
    code_hash: &[u8],
    app_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT c.grant_id FROM oauth_codes c JOIN oauth_grants g ON g.id = c.grant_id
         WHERE c.code_hash = $1 AND g.app_id = $2 AND c.consumed_at IS NOT NULL",
    )
    .bind(code_hash)
    .bind(app_id)
    .fetch_optional(conn)
    .await
}

/// Sign every session of a grant out, leaving the consent itself in place.
pub async fn delete_grant_tokens(
    conn: &mut PgConnection,
    grant_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM oauth_tokens WHERE grant_id = $1")
        .bind(grant_id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

/// Store a new session's token pair and stamp the grant as used. Sessions past
/// `keep` for the grant are dropped, oldest first.
#[allow(clippy::too_many_arguments)]
pub async fn insert_token(
    conn: &mut PgConnection,
    grant_id: Uuid,
    access_hash: &[u8],
    refresh_hash: &[u8],
    scope: &str,
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
    keep: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_tokens
             (grant_id, access_hash, refresh_hash, scope, access_expires_at, refresh_expires_at)
         VALUES ($1, $2, $3, $4,
                 now() + make_interval(secs => $5), now() + make_interval(secs => $6))",
    )
    .bind(grant_id)
    .bind(access_hash)
    .bind(refresh_hash)
    .bind(scope)
    .bind(access_ttl_secs as f64)
    .bind(refresh_ttl_secs as f64)
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "DELETE FROM oauth_tokens WHERE id IN (
             SELECT id FROM oauth_tokens WHERE grant_id = $1
             ORDER BY created_at DESC, id OFFSET $2
         )",
    )
    .bind(grant_id)
    .bind(keep)
    .execute(&mut *conn)
    .await?;
    touch_grant(conn, grant_id).await
}

async fn touch_grant(conn: &mut PgConnection, grant_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE oauth_grants SET last_used_at = now() WHERE id = $1")
        .bind(grant_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Trade a refresh token for a new pair, in place: the row keeps its id and
/// takes both new hashes, so the old refresh token matches nothing afterwards.
/// One statement, so two refreshes racing on one token cannot both win.
/// Returns the session's scope, or `None` when the token is unknown, expired
/// or belongs to another application.
pub async fn rotate_token(
    conn: &mut PgConnection,
    refresh_hash: &[u8],
    app_id: Uuid,
    new_access_hash: &[u8],
    new_refresh_hash: &[u8],
    access_ttl_secs: i64,
    refresh_ttl_secs: i64,
) -> Result<Option<String>, sqlx::Error> {
    let rotated: Option<(Uuid, String)> = sqlx::query_as(
        "UPDATE oauth_tokens t
         SET access_hash = $3, refresh_hash = $4,
             access_expires_at = now() + make_interval(secs => $5),
             refresh_expires_at = now() + make_interval(secs => $6)
         FROM oauth_grants g
         WHERE t.refresh_hash = $1 AND g.id = t.grant_id AND g.app_id = $2
           AND t.refresh_expires_at > now()
         RETURNING t.grant_id, t.scope",
    )
    .bind(refresh_hash)
    .bind(app_id)
    .bind(new_access_hash)
    .bind(new_refresh_hash)
    .bind(access_ttl_secs as f64)
    .bind(refresh_ttl_secs as f64)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((grant_id, scope)) = rotated else {
        return Ok(None);
    };
    touch_grant(conn, grant_id).await?;
    Ok(Some(scope))
}

/// End the session `token_hash` belongs to, whether it is that session's
/// access or refresh token, provided `app_id` issued it. `true` when one went.
pub async fn revoke_token(
    conn: &mut PgConnection,
    token_hash: &[u8],
    app_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM oauth_tokens t USING oauth_grants g
         WHERE g.id = t.grant_id AND g.app_id = $2
           AND (t.access_hash = $1 OR t.refresh_hash = $1)",
    )
    .bind(token_hash)
    .bind(app_id)
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Who a live access token speaks for; `None` when it is unknown or expired.
pub async fn find_access_token(
    conn: &mut PgConnection,
    access_hash: &[u8],
) -> Result<Option<AccessTokenOwner>, sqlx::Error> {
    sqlx::query_as::<_, AccessTokenOwner>(
        "SELECT g.user_id, t.scope
         FROM oauth_tokens t JOIN oauth_grants g ON g.id = t.grant_id
         WHERE t.access_hash = $1 AND t.access_expires_at > now()",
    )
    .bind(access_hash)
    .fetch_optional(conn)
    .await
}

/// The identity `/oauth/userinfo` returns. Read from `users` directly:
/// `v_user_profile` would also count the roster, inventory and skins.
pub async fn find_identity(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Option<OAuthIdentityRow>, sqlx::Error> {
    sqlx::query_as::<_, OAuthIdentityRow>(
        "SELECT u.id, u.uid, s.code AS server, u.nickname, u.nick_number, u.avatar_id
         FROM users u JOIN servers s ON s.id = u.server_id
         WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_optional(conn)
    .await
}

/// The applications `user_id` has authorized, most recently used first.
pub async fn list_authorizations(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<OAuthAuthorizationRow>, sqlx::Error> {
    sqlx::query_as::<_, OAuthAuthorizationRow>(&format!(
        "SELECT {PUBLIC_APP_COLUMNS}, g.scope, g.created_at AS authorized_at, g.last_used_at
         FROM oauth_grants g
         JOIN oauth_apps a ON a.id = g.app_id
         JOIN users o ON o.id = a.owner_id
         WHERE g.user_id = $1
         ORDER BY COALESCE(g.last_used_at, g.created_at) DESC, g.id"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await
}

/// Withdraw `user_id`'s consent to an application: the grant goes, and its
/// codes and tokens with it. `true` when there was one.
pub async fn delete_grant(
    conn: &mut PgConnection,
    app_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM oauth_grants WHERE app_id = $1 AND user_id = $2")
        .bind(app_id)
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Drop what can no longer be used: codes past their expiry and sessions whose
/// refresh token ran out. Both deletes ride an index on the expiry column.
pub async fn delete_expired(conn: &mut PgConnection) -> Result<u64, sqlx::Error> {
    let codes = sqlx::query("DELETE FROM oauth_codes WHERE expires_at < now()")
        .execute(&mut *conn)
        .await?
        .rows_affected();
    let tokens = sqlx::query("DELETE FROM oauth_tokens WHERE refresh_expires_at < now()")
        .execute(conn)
        .await?
        .rows_affected();
    Ok(codes + tokens)
}
