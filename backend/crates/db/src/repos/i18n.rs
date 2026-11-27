use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;
use uuid::Uuid;

use crate::sql::escape_like;
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct LocaleRow {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub fallback_locale: Option<String>,
    pub gamedata_server: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub is_source: bool,
}

#[derive(Debug, Clone)]
pub struct EntryRow {
    pub key: String,
    pub namespace: String,
    pub source_text: String,
    pub source_hash: String,
    pub description: Option<String>,
    pub placeholders: Value,
    pub value: Option<String>,
    pub translated_hash: Option<String>,
    pub translated_source_text: Option<String>,
    pub is_stale: bool,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AuditRow {
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub actor_nickname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditFeedRow {
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub actor_user_id: Uuid,
    pub actor_uid: Option<String>,
    pub actor_nickname: Option<String>,
    pub actor_secretary: Option<String>,
    pub actor_secretary_skin_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LocaleCompletionRow {
    pub code: String,
    pub current: i64,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct LocaleProgressRow {
    pub code: String,
    pub total: i64,
    pub translated: i64,
    pub stale: i64,
}

#[derive(Debug, Clone)]
pub struct GrantRow {
    pub locale: String,
    pub user_id: Uuid,
    pub user_uid: Option<String>,
    pub user_nickname: Option<String>,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_by_nickname: Option<String>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PermissionRow {
    pub locale: String,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct OverrideRow {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DocumentRow {
    pub slug: String,
    pub locale: String,
    pub title: String,
    pub body: String,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_locales(conn: &mut Conn, enabled_only: bool) -> Result<Vec<LocaleRow>, DbError> {
    let rows = if enabled_only {
        sqlx::query_as!(
            LocaleRow,
            "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled,
                    sort_order, is_source
             FROM locales WHERE enabled ORDER BY sort_order, code"
        )
        .fetch_all(conn)
        .await?
    } else {
        sqlx::query_as!(
            LocaleRow,
            "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled,
                    sort_order, is_source
             FROM locales ORDER BY sort_order, code"
        )
        .fetch_all(conn)
        .await?
    };
    Ok(rows)
}

pub async fn get_locale(conn: &mut Conn, code: &str) -> Result<Option<LocaleRow>, DbError> {
    Ok(sqlx::query_as!(
        LocaleRow,
        "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled,
                sort_order, is_source
         FROM locales WHERE code = $1",
        code,
    )
    .fetch_optional(conn)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_locale(
    conn: &mut Conn,
    code: &str,
    english_name: &str,
    native_name: &str,
    fallback_locale: Option<&str>,
    gamedata_server: &str,
    enabled: bool,
    sort_order: i32,
) -> Result<LocaleRow, DbError> {
    Ok(sqlx::query_as!(
        LocaleRow,
        "INSERT INTO locales
             (code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (code) DO UPDATE SET
             english_name = EXCLUDED.english_name,
             native_name = EXCLUDED.native_name,
             fallback_locale = CASE WHEN locales.is_source THEN NULL ELSE EXCLUDED.fallback_locale END,
             gamedata_server = EXCLUDED.gamedata_server,
             enabled = CASE WHEN locales.is_source THEN true ELSE EXCLUDED.enabled END,
             sort_order = EXCLUDED.sort_order
         RETURNING code, english_name, native_name, fallback_locale, gamedata_server, enabled,
                   sort_order, is_source",
        code,
        english_name,
        native_name,
        fallback_locale,
        gamedata_server,
        enabled,
        sort_order,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn get_catalog(
    conn: &mut Conn,
    locale: &str,
    namespace: &str,
) -> Result<Vec<(String, String)>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT k.key, COALESCE(m.value, f.value) AS "value!"
           FROM ui_message_keys k
           LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
           LEFT JOIN ui_messages f
             ON f.key = k.key AND f.locale = (SELECT fallback_locale FROM locales WHERE code = $1)
           WHERE k.is_active
             AND ($2 = '' OR k.namespace = $2)
             AND COALESCE(m.value, f.value) IS NOT NULL
           ORDER BY k.key"#,
        locale,
        namespace,
    )
    .fetch_all(conn)
    .await?; // same resolution as `catalog_hash`, so the two cannot disagree
    Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
}

pub async fn catalog_hash(
    conn: &mut Conn,
    locale: &str,
    namespace: &str,
) -> Result<String, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT substr(
             md5(COALESCE(string_agg(r.key || chr(31) || r.value, chr(30) ORDER BY r.key), '')),
             1, 12) AS "hash!"
         FROM (
             SELECT k.key AS key, COALESCE(m.value, f.value) AS value
             FROM ui_message_keys k
             LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
             LEFT JOIN ui_messages f
               ON f.key = k.key AND f.locale = (SELECT fallback_locale FROM locales WHERE code = $1)
             WHERE k.is_active
               AND ($2 = '' OR k.namespace = $2)
               AND COALESCE(m.value, f.value) IS NOT NULL
         ) r"#,
        locale,
        namespace,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn list_namespaces(conn: &mut Conn) -> Result<Vec<String>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT DISTINCT namespace FROM ui_message_keys WHERE is_active ORDER BY 1"
    )
    .fetch_all(conn)
    .await?)
}

pub async fn sync_keys(
    conn: &mut Conn,
    keys: &[String],
    namespaces: &[String],
    source_texts: &[String],
    source_hashes: &[String],
    descriptions: &[Option<String>],
    placeholders: &[Value],
) -> Result<u64, DbError> {
    let result = sqlx::query!(
        "INSERT INTO ui_message_keys
             (key, namespace, source_text, source_hash, description, placeholders)
         SELECT k, n, t, h, d, p
         FROM UNNEST($1::text[], $2::text[], $3::text[], $4::varchar[], $5::text[], $6::jsonb[])
             AS u(k, n, t, h, d, p)
         ON CONFLICT (key) DO UPDATE SET
             namespace = EXCLUDED.namespace,
             source_text = EXCLUDED.source_text,
             source_hash = EXCLUDED.source_hash,
             description = EXCLUDED.description,
             placeholders = EXCLUDED.placeholders,
             last_seen_at = now(),
             is_active = true",
        keys,
        namespaces,
        source_texts,
        source_hashes,
        descriptions as _,
        placeholders,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

pub async fn deactivate_missing_keys(
    conn: &mut Conn,
    seen_keys: &[String],
) -> Result<u64, DbError> {
    let result = sqlx::query!(
        "UPDATE ui_message_keys SET is_active = false WHERE is_active AND key <> ALL($1::text[])",
        seen_keys,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

pub async fn list_entries(
    conn: &mut Conn,
    locale: &str,
    namespace: &str,
    search: &str,
    filter: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<EntryRow>, DbError> {
    Ok(sqlx::query_as!(
        EntryRow,
        r#"SELECT k.key, k.namespace, k.source_text, k.source_hash, k.description, k.placeholders,
                  CASE WHEN COALESCE(l.is_source, false) THEN COALESCE(m.value, k.source_text)
                       ELSE m.value END AS "value?",
                  m.source_hash AS "translated_hash?",
                  m.source_text AS "translated_source_text?",
                  (NOT COALESCE(l.is_source, false)
                   AND m.source_hash IS NOT NULL
                   AND m.source_hash <> k.source_hash) AS "is_stale!",
                  m.updated_at AS "updated_at?",
                  m.updated_by AS "updated_by?"
           FROM ui_message_keys k
           LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
           LEFT JOIN locales l ON l.code = $1
           WHERE k.is_active
             AND ($2 = '' OR k.namespace = $2)
             AND ($3 = '' OR k.key ILIKE '%' || $3 || '%' ESCAPE '\'
                          OR k.source_text ILIKE '%' || $3 || '%' ESCAPE '\')
             AND CASE $4::text
                 WHEN 'untranslated' THEN NOT COALESCE(l.is_source, false) AND m.value IS NULL
                 WHEN 'stale' THEN NOT COALESCE(l.is_source, false)
                     AND m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash
                 WHEN 'todo' THEN NOT COALESCE(l.is_source, false)
                     AND (m.value IS NULL
                          OR (m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash))
                 WHEN 'translated' THEN COALESCE(l.is_source, false)
                     OR (m.value IS NOT NULL AND m.source_hash = k.source_hash)
                 ELSE true END
           ORDER BY k.key
           LIMIT $5 OFFSET $6"#,
        locale,
        namespace,
        escape_like(search),
        filter,
        limit,
        offset,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_entries(
    conn: &mut Conn,
    locale: &str,
    namespace: &str,
    search: &str,
    filter: &str,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM ui_message_keys k
           LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
           LEFT JOIN locales l ON l.code = $1
           WHERE k.is_active
             AND ($2 = '' OR k.namespace = $2)
             AND ($3 = '' OR k.key ILIKE '%' || $3 || '%' ESCAPE '\'
                          OR k.source_text ILIKE '%' || $3 || '%' ESCAPE '\')
             AND CASE $4::text
                 WHEN 'untranslated' THEN NOT COALESCE(l.is_source, false) AND m.value IS NULL
                 WHEN 'stale' THEN NOT COALESCE(l.is_source, false)
                     AND m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash
                 WHEN 'todo' THEN NOT COALESCE(l.is_source, false)
                     AND (m.value IS NULL
                          OR (m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash))
                 WHEN 'translated' THEN COALESCE(l.is_source, false)
                     OR (m.value IS NOT NULL AND m.source_hash = k.source_hash)
                 ELSE true END"#,
        locale,
        namespace,
        escape_like(search),
        filter,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn get_entry(
    conn: &mut Conn,
    locale: &str,
    key: &str,
) -> Result<Option<EntryRow>, DbError> {
    Ok(sqlx::query_as!(
        EntryRow,
        r#"SELECT k.key, k.namespace, k.source_text, k.source_hash, k.description, k.placeholders,
                  CASE WHEN COALESCE(l.is_source, false) THEN COALESCE(m.value, k.source_text)
                       ELSE m.value END AS "value?",
                  m.source_hash AS "translated_hash?",
                  m.source_text AS "translated_source_text?",
                  (NOT COALESCE(l.is_source, false)
                   AND m.source_hash IS NOT NULL
                   AND m.source_hash <> k.source_hash) AS "is_stale!",
                  m.updated_at AS "updated_at?",
                  m.updated_by AS "updated_by?"
           FROM ui_message_keys k
           LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
           LEFT JOIN locales l ON l.code = $1
           WHERE k.is_active AND k.key = $2"#,
        locale,
        key,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn upsert_message(
    conn: &mut Conn,
    key: &str,
    locale: &str,
    value: &str,
    source_hash: &str,
    source_text: &str,
    updated_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO ui_messages (key, locale, value, source_hash, source_text, updated_by, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, now())
         ON CONFLICT (key, locale) DO UPDATE SET
             value = EXCLUDED.value,
             source_hash = EXCLUDED.source_hash,
             source_text = EXCLUDED.source_text,
             updated_by = EXCLUDED.updated_by,
             updated_at = now()",
        key,
        locale,
        value,
        source_hash,
        source_text,
        updated_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_message(conn: &mut Conn, key: &str, locale: &str) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM ui_messages WHERE key = $1 AND locale = $2",
        key,
        locale,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn insert_audit(
    conn: &mut Conn,
    message_key: &str,
    locale: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO ui_message_audit_log (message_key, locale, old_value, new_value, changed_by)
         VALUES ($1, $2, $3, $4, $5)",
        message_key,
        locale,
        old_value,
        new_value,
        changed_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_audit_log(
    conn: &mut Conn,
    message_key: &str,
    locale: &str,
) -> Result<Vec<AuditRow>, DbError> {
    Ok(sqlx::query_as!(
        AuditRow,
        "SELECT a.id, a.message_key, a.locale, a.old_value, a.new_value, a.changed_by, a.changed_at,
                u.nickname AS actor_nickname
         FROM ui_message_audit_log a
         LEFT JOIN users u ON u.id = a.changed_by
         WHERE a.message_key = $1 AND a.locale = $2
         ORDER BY a.changed_at DESC
         LIMIT 100",
        message_key,
        locale,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_audit_log_global(
    conn: &mut Conn,
    locale: Option<&str>,
    actor: Option<UserId>,
    limit: i64,
    before: Option<DateTime<Utc>>,
) -> Result<Vec<AuditFeedRow>, DbError> {
    Ok(sqlx::query_as!(
        AuditFeedRow,
        r#"SELECT a.id, a.message_key, a.locale, a.old_value, a.new_value, a.changed_at,
                  a.changed_by AS actor_user_id,
                  u.uid AS "actor_uid?",
                  u.nickname AS actor_nickname,
                  u.secretary AS actor_secretary,
                  u.secretary_skin_id AS actor_secretary_skin_id
           FROM ui_message_audit_log a
           LEFT JOIN users u ON u.id = a.changed_by
           WHERE ($1::varchar IS NULL OR a.locale = $1)
             AND ($2::uuid IS NULL OR a.changed_by = $2)
             AND ($3::timestamptz IS NULL OR a.changed_at < $3)
           ORDER BY a.changed_at DESC
           LIMIT $4"#,
        locale,
        actor.map(|a| a.0),
        before,
        limit,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_audit_log(
    conn: &mut Conn,
    locale: Option<&str>,
    actor: Option<UserId>,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM ui_message_audit_log
           WHERE ($1::varchar IS NULL OR locale = $1) AND ($2::uuid IS NULL OR changed_by = $2)"#,
        locale,
        actor.map(|a| a.0),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn locale_completion(conn: &mut Conn) -> Result<Vec<LocaleCompletionRow>, DbError> {
    Ok(sqlx::query_as!(
        LocaleCompletionRow,
        r#"WITH total AS (SELECT COUNT(*) AS n FROM ui_message_keys WHERE is_active)
           SELECT l.code,
                  CASE WHEN l.is_source THEN total.n
                       ELSE COUNT(m.key) FILTER (WHERE m.source_hash = k.source_hash) END
                      AS "current!",
                  total.n AS "total!"
           FROM locales l
           CROSS JOIN total
           LEFT JOIN (ui_messages m JOIN ui_message_keys k ON k.key = m.key AND k.is_active)
             ON m.locale = l.code
           GROUP BY l.code, l.is_source, total.n"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn locale_progress(conn: &mut Conn) -> Result<Vec<LocaleProgressRow>, DbError> {
    Ok(sqlx::query_as!(
        LocaleProgressRow,
        r#"WITH total AS (SELECT COUNT(*) AS n FROM ui_message_keys WHERE is_active)
           SELECT l.code,
                  total.n AS "total!",
                  CASE WHEN l.is_source THEN total.n
                       ELSE COUNT(m.key) END AS "translated!",
                  CASE WHEN l.is_source THEN 0
                       ELSE COUNT(m.key) FILTER (WHERE m.source_hash <> k.source_hash) END
                      AS "stale!"
           FROM locales l
           CROSS JOIN total
           LEFT JOIN (ui_messages m JOIN ui_message_keys k ON k.key = m.key AND k.is_active)
             ON m.locale = l.code
           GROUP BY l.code, l.is_source, total.n
           ORDER BY l.code"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_permissions(
    conn: &mut Conn,
    locale: Option<&str>,
    user: Option<UserId>,
) -> Result<Vec<GrantRow>, DbError> {
    Ok(sqlx::query_as!(
        GrantRow,
        r#"SELECT p.locale, p.user_id,
                  u.uid AS "user_uid?",
                  u.nickname AS user_nickname,
                  p.permission, p.granted_by,
                  g.nickname AS granted_by_nickname,
                  p.granted_at
           FROM translation_permissions p
           LEFT JOIN users u ON u.id = p.user_id
           LEFT JOIN users g ON g.id = p.granted_by
           WHERE ($1::varchar IS NULL OR p.locale = $1)
             AND ($2::uuid IS NULL OR p.user_id = $2)
           ORDER BY p.locale, p.granted_at"#,
        locale,
        user.map(|u| u.0),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_permissions_for_user(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<PermissionRow>, DbError> {
    Ok(sqlx::query_as!(
        PermissionRow,
        "SELECT locale, user_id, permission, granted_by, granted_at
         FROM translation_permissions WHERE user_id = $1 ORDER BY locale",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn has_any_permission(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM translation_permissions WHERE user_id = $1) AS "exists!""#,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn grant_permission(
    conn: &mut Conn,
    locale: &str,
    user: UserId,
    permission: &str,
    granted_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO translation_permissions (locale, user_id, permission, granted_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (locale, user_id, permission)
         DO UPDATE SET granted_by = EXCLUDED.granted_by, granted_at = now()",
        locale,
        user.0,
        permission,
        granted_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn revoke_permission(
    conn: &mut Conn,
    locale: &str,
    user: UserId,
    permission: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM translation_permissions WHERE locale = $1 AND user_id = $2 AND permission = $3",
        locale,
        user.0,
        permission,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list_overrides(conn: &mut Conn, locale: &str) -> Result<Vec<OverrideRow>, DbError> {
    Ok(sqlx::query_as!(
        OverrideRow,
        "SELECT locale, kind, entity_id, field, value, updated_at
         FROM gamedata_overrides WHERE locale = $1 ORDER BY kind, entity_id, field",
        locale,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn upsert_override(
    conn: &mut Conn,
    locale: &str,
    kind: &str,
    entity_id: &str,
    field: &str,
    value: &str,
    updated_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO gamedata_overrides (locale, kind, entity_id, field, value, updated_by, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, now())
         ON CONFLICT (locale, kind, entity_id, field) DO UPDATE SET
             value = EXCLUDED.value, updated_by = EXCLUDED.updated_by, updated_at = now()",
        locale,
        kind,
        entity_id,
        field,
        value,
        updated_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_override(
    conn: &mut Conn,
    locale: &str,
    kind: &str,
    entity_id: &str,
    field: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        "DELETE FROM gamedata_overrides
         WHERE locale = $1 AND kind = $2 AND entity_id = $3 AND field = $4",
        locale,
        kind,
        entity_id,
        field,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_document(
    conn: &mut Conn,
    slug: &str,
    locale: &str,
) -> Result<Option<DocumentRow>, DbError> {
    Ok(sqlx::query_as!(
        DocumentRow,
        "SELECT slug, locale, title, body, updated_at FROM ui_documents WHERE slug = $1 AND locale = $2",
        slug,
        locale,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn upsert_document(
    conn: &mut Conn,
    slug: &str,
    locale: &str,
    title: &str,
    body: &str,
    updated_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO ui_documents (slug, locale, title, body, updated_by, updated_at)
         VALUES ($1, $2, $3, $4, $5, now())
         ON CONFLICT (slug, locale) DO UPDATE SET
             title = EXCLUDED.title, body = EXCLUDED.body,
             updated_by = EXCLUDED.updated_by, updated_at = now()",
        slug,
        locale,
        title,
        body,
        updated_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}
