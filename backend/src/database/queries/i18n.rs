use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::i18n::{
    GamedataOverride, Locale, TranslationEntry, TranslationPermission, UiDocument,
    UiMessageAuditEntry, UiMessageAuditEntryWithContext,
};

// ---------------------------------------------------------------- locales

pub async fn list_locales(pool: &PgPool, enabled_only: bool) -> Result<Vec<Locale>, sqlx::Error> {
    let sql = if enabled_only {
        "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order, is_source
         FROM locales WHERE enabled ORDER BY sort_order, code"
    } else {
        "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order, is_source
         FROM locales ORDER BY sort_order, code"
    };
    sqlx::query_as::<_, Locale>(sql).fetch_all(pool).await
}

pub async fn get_locale(pool: &PgPool, code: &str) -> Result<Option<Locale>, sqlx::Error> {
    sqlx::query_as::<_, Locale>(
        "SELECT code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order, is_source
         FROM locales WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_locale(
    pool: &PgPool,
    code: &str,
    english_name: &str,
    native_name: &str,
    fallback_locale: Option<&str>,
    gamedata_server: &str,
    enabled: bool,
    sort_order: i32,
) -> Result<Locale, sqlx::Error> {
    sqlx::query_as::<_, Locale>(
        r"
        INSERT INTO locales (code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (code) DO UPDATE SET
            english_name = EXCLUDED.english_name,
            native_name = EXCLUDED.native_name,
            -- The source locale is the floor of every fallback chain and can
            -- neither be disabled nor given a fallback of its own.
            fallback_locale = CASE WHEN locales.is_source THEN NULL ELSE EXCLUDED.fallback_locale END,
            gamedata_server = EXCLUDED.gamedata_server,
            enabled = CASE WHEN locales.is_source THEN true ELSE EXCLUDED.enabled END,
            sort_order = EXCLUDED.sort_order
        RETURNING code, english_name, native_name, fallback_locale, gamedata_server, enabled, sort_order, is_source
        ",
    )
    .bind(code)
    .bind(english_name)
    .bind(native_name)
    .bind(fallback_locale)
    .bind(gamedata_server)
    .bind(enabled)
    .bind(sort_order)
    .fetch_one(pool)
    .await
}

// ---------------------------------------------------------------- catalog
//
// One resolution rule, written once and shared by the catalog read and the
// manifest fingerprint so the two can never disagree about what is in a
// catalog: this locale's value, else the fallback locale's value, else the key
// is absent and the client renders the bundled English source.

const RESOLVED: &str = r"
    SELECT k.key AS key,
           COALESCE(m.value, f.value) AS value
    FROM ui_message_keys k
    LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
    LEFT JOIN locales l ON l.code = $1
    LEFT JOIN ui_messages f ON f.key = k.key AND f.locale = l.fallback_locale
    WHERE k.is_active
      AND ($2 = '' OR k.namespace = $2)
      AND COALESCE(m.value, f.value) IS NOT NULL
";

/// The rendered catalog for one locale and namespace (`""` for every
/// namespace), as `(key, value)` pairs ordered by key.
pub async fn get_catalog(
    pool: &PgPool,
    locale: &str,
    namespace: &str,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String)>(&format!("{RESOLVED} ORDER BY k.key"))
        .bind(locale)
        .bind(namespace)
        .fetch_all(pool)
        .await
}

/// A short content fingerprint of one locale+namespace catalog. Changing any
/// value - or any value in the fallback the catalog inherits from - moves this
/// hash, which is what the immutable per-hash catalog URLs are keyed on.
pub async fn catalog_hash(
    pool: &PgPool,
    locale: &str,
    namespace: &str,
) -> Result<String, sqlx::Error> {
    let (hash,): (Option<String>,) = sqlx::query_as(&format!(
        r"
        SELECT substr(
            md5(COALESCE(string_agg(r.key || chr(31) || r.value, chr(30) ORDER BY r.key), '')),
            1, 12)
        FROM ({RESOLVED}) r
        "
    ))
    .bind(locale)
    .bind(namespace)
    .fetch_one(pool)
    .await?;
    Ok(hash.unwrap_or_else(|| "empty".to_owned()))
}

/// Every active namespace, for building the manifest.
pub async fn list_namespaces(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT namespace FROM ui_message_keys WHERE is_active ORDER BY 1")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

// ---------------------------------------------------------------- key sync

/// Bulk-upsert extracted keys. `source_text`/`source_hash` always win: the
/// code is authoritative for what the English *source* is, even though the
/// database is authoritative for what is *rendered*.
pub async fn sync_keys(
    pool: &PgPool,
    keys: &[String],
    namespaces: &[String],
    source_texts: &[String],
    source_hashes: &[String],
    descriptions: &[Option<String>],
    placeholders: &[serde_json::Value],
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r"
        INSERT INTO ui_message_keys
            (key, namespace, source_text, source_hash, description, placeholders)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::varchar[], $5::text[], $6::jsonb[])
        ON CONFLICT (key) DO UPDATE SET
            namespace = EXCLUDED.namespace,
            source_text = EXCLUDED.source_text,
            source_hash = EXCLUDED.source_hash,
            description = EXCLUDED.description,
            placeholders = EXCLUDED.placeholders,
            last_seen_at = now(),
            is_active = true
        ",
    )
    .bind(keys)
    .bind(namespaces)
    .bind(source_texts)
    .bind(source_hashes)
    .bind(descriptions)
    .bind(placeholders)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// Retire keys the extractor no longer sees. Deactivated, never deleted, so a
/// key that returns keeps the translations it already had.
pub async fn deactivate_missing_keys(
    pool: &PgPool,
    seen_keys: &[String],
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE ui_message_keys SET is_active = false WHERE is_active AND key <> ALL($1::text[])",
    )
    .bind(seen_keys)
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

// ---------------------------------------------------------------- editor

/// The source locale has no `ui_messages` rows - its text lives in
/// `ui_message_keys.source_text` - so it needs `source_text` projected as the
/// value and can never be stale. Without that the editor showed every source
/// row as "Not translated" while the progress chips above it correctly said
/// zero untranslated, which is the same number contradicting itself on one
/// screen. A row that HAS been overridden for the source locale still wins,
/// because that override is exactly what editing English in the panel writes.
const ENTRY_SELECT: &str = r"
    SELECT k.key,
           k.namespace,
           k.source_text,
           k.source_hash,
           k.description,
           k.placeholders,
           CASE WHEN COALESCE(l.is_source, false) THEN COALESCE(m.value, k.source_text) ELSE m.value END AS value,
           m.source_hash AS translated_hash,
           (NOT COALESCE(l.is_source, false)
            AND m.source_hash IS NOT NULL
            AND m.source_hash <> k.source_hash) AS is_stale,
           m.updated_at,
           m.updated_by
    FROM ui_message_keys k
    LEFT JOIN ui_messages m ON m.key = k.key AND m.locale = $1
    LEFT JOIN locales l ON l.code = $1
    WHERE k.is_active
      AND ($2 = '' OR k.namespace = $2)
      AND ($3 = '' OR k.key ILIKE '%' || $3 || '%' OR k.source_text ILIKE '%' || $3 || '%')
";

/// `filter` is one of `all`, `untranslated`, `stale`, `translated`. Unknown
/// values fall through to `all` rather than erroring, because this is a UI
/// affordance and not a contract.
fn filter_clause(filter: &str) -> &'static str {
    // Each clause carries the same source-locale guard as the projection
    // above, so the filters and the rows they return cannot disagree: for the
    // source locale everything is translated, nothing is outstanding, and
    // nothing is stale.
    match filter {
        "untranslated" => " AND NOT COALESCE(l.is_source, false) AND m.value IS NULL",
        "stale" => {
            " AND NOT COALESCE(l.is_source, false) AND m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash"
        }
        "translated" => {
            " AND (COALESCE(l.is_source, false) OR (m.value IS NOT NULL AND m.source_hash = k.source_hash))"
        }
        _ => "",
    }
}

pub async fn list_entries(
    pool: &PgPool,
    locale: &str,
    namespace: &str,
    search: &str,
    filter: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<TranslationEntry>, sqlx::Error> {
    let sql = format!(
        "{ENTRY_SELECT}{} ORDER BY k.key LIMIT $4 OFFSET $5",
        filter_clause(filter)
    );
    sqlx::query_as::<_, TranslationEntry>(&sql)
        .bind(locale)
        .bind(namespace)
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn count_entries(
    pool: &PgPool,
    locale: &str,
    namespace: &str,
    search: &str,
    filter: &str,
) -> Result<i64, sqlx::Error> {
    let sql = format!(
        "SELECT COUNT(*) FROM ({ENTRY_SELECT}{}) q",
        filter_clause(filter)
    );
    let (count,): (i64,) = sqlx::query_as(&sql)
        .bind(locale)
        .bind(namespace)
        .bind(search)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn get_entry(
    pool: &PgPool,
    locale: &str,
    key: &str,
) -> Result<Option<TranslationEntry>, sqlx::Error> {
    let sql = format!("{ENTRY_SELECT} AND k.key = $4");
    sqlx::query_as::<_, TranslationEntry>(&sql)
        .bind(locale)
        .bind("")
        .bind("")
        .bind(key)
        .fetch_optional(pool)
        .await
}

/// Write a translation, stamping it with the source hash it was written
/// against. That stamp is what later marks it stale.
pub async fn upsert_message(
    pool: &PgPool,
    key: &str,
    locale: &str,
    value: &str,
    source_hash: &str,
    updated_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO ui_messages (key, locale, value, source_hash, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, now())
        ON CONFLICT (key, locale) DO UPDATE SET
            value = EXCLUDED.value,
            source_hash = EXCLUDED.source_hash,
            updated_by = EXCLUDED.updated_by,
            updated_at = now()
        ",
    )
    .bind(key)
    .bind(locale)
    .bind(value)
    .bind(source_hash)
    .bind(updated_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_message(pool: &PgPool, key: &str, locale: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM ui_messages WHERE key = $1 AND locale = $2")
        .bind(key)
        .bind(locale)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------- audit

pub async fn insert_audit(
    pool: &PgPool,
    message_key: &str,
    locale: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO ui_message_audit_log (message_key, locale, old_value, new_value, changed_by) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(message_key)
    .bind(locale)
    .bind(old_value)
    .bind(new_value)
    .bind(changed_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_audit_log(
    pool: &PgPool,
    message_key: &str,
    locale: &str,
) -> Result<Vec<UiMessageAuditEntry>, sqlx::Error> {
    sqlx::query_as::<_, UiMessageAuditEntry>(
        "SELECT * FROM ui_message_audit_log WHERE message_key = $1 AND locale = $2 ORDER BY changed_at DESC LIMIT 100",
    )
    .bind(message_key)
    .bind(locale)
    .fetch_all(pool)
    .await
}

pub async fn get_audit_log_global(
    pool: &PgPool,
    locale: Option<&str>,
    limit: i64,
    before: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Vec<UiMessageAuditEntryWithContext>, sqlx::Error> {
    // LEFT JOIN users so audit rows survive a hard-deleted actor, matching the
    // operator-notes global feed.
    const SELECT: &str = r"
        SELECT
            a.id,
            a.message_key,
            a.locale,
            a.old_value,
            a.new_value,
            a.changed_at,
            a.changed_by         AS actor_user_id,
            u.uid                AS actor_uid,
            u.nickname           AS actor_nickname,
            u.secretary          AS actor_secretary,
            u.secretary_skin_id  AS actor_secretary_skin_id
        FROM ui_message_audit_log a
        LEFT JOIN users u ON u.id = a.changed_by
        WHERE ($1::varchar IS NULL OR a.locale = $1)
    ";

    if let Some(before) = before {
        sqlx::query_as::<_, UiMessageAuditEntryWithContext>(&format!(
            "{SELECT} AND a.changed_at < $2 ORDER BY a.changed_at DESC LIMIT $3"
        ))
        .bind(locale)
        .bind(before)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, UiMessageAuditEntryWithContext>(&format!(
            "{SELECT} ORDER BY a.changed_at DESC LIMIT $2"
        ))
        .bind(locale)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}

pub async fn count_audit_log(pool: &PgPool, locale: Option<&str>) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ui_message_audit_log WHERE ($1::varchar IS NULL OR locale = $1)",
    )
    .bind(locale)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

// ---------------------------------------------------------------- progress

/// `(locale, total_active_keys, translated, stale)` - drives the admin
/// sidebar badge and the per-locale progress bars.
pub async fn locale_progress(pool: &PgPool) -> Result<Vec<(String, i64, i64, i64)>, sqlx::Error> {
    sqlx::query_as::<_, (String, i64, i64, i64)>(
        r"
        WITH total AS (SELECT COUNT(*) AS n FROM ui_message_keys WHERE is_active)
        SELECT l.code,
               total.n,
               -- The source locale is complete by definition: every active key
               -- carries its text in `ui_message_keys.source_text`, so there is
               -- nothing left to translate and no `ui_messages` row to count.
               -- Counting rows for it would report 0% forever.
               CASE WHEN l.is_source THEN total.n
                    ELSE COUNT(m.key) FILTER (WHERE m.value IS NOT NULL) END,
               CASE WHEN l.is_source THEN 0
                    ELSE COUNT(m.key) FILTER (WHERE m.source_hash IS NOT NULL AND m.source_hash <> k.source_hash) END
        FROM locales l
        CROSS JOIN total
        LEFT JOIN ui_messages m ON m.locale = l.code
        LEFT JOIN ui_message_keys k ON k.key = m.key AND k.is_active
        GROUP BY l.code, l.is_source, total.n
        ORDER BY l.code
        ",
    )
    .fetch_all(pool)
    .await
}

// ---------------------------------------------------------------- grants

pub async fn list_permissions(
    pool: &PgPool,
    locale: Option<&str>,
) -> Result<Vec<TranslationPermission>, sqlx::Error> {
    sqlx::query_as::<_, TranslationPermission>(
        "SELECT * FROM translation_permissions WHERE ($1::varchar IS NULL OR locale = $1) ORDER BY locale, granted_at",
    )
    .bind(locale)
    .fetch_all(pool)
    .await
}

pub async fn list_permissions_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<TranslationPermission>, sqlx::Error> {
    sqlx::query_as::<_, TranslationPermission>(
        "SELECT * FROM translation_permissions WHERE user_id = $1 ORDER BY locale",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn grant_permission(
    pool: &PgPool,
    locale: &str,
    user_id: Uuid,
    permission: &str,
    granted_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO translation_permissions (locale, user_id, permission, granted_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (locale, user_id, permission) DO UPDATE SET granted_by = EXCLUDED.granted_by, granted_at = now()
        ",
    )
    .bind(locale)
    .bind(user_id)
    .bind(permission)
    .bind(granted_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn revoke_permission(
    pool: &PgPool,
    locale: &str,
    user_id: Uuid,
    permission: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM translation_permissions WHERE locale = $1 AND user_id = $2 AND permission = $3",
    )
    .bind(locale)
    .bind(user_id)
    .bind(permission)
    .execute(pool)
    .await?;
    Ok(())
}

// ---------------------------------------------------------------- overrides

pub async fn list_overrides(
    pool: &PgPool,
    locale: &str,
) -> Result<Vec<GamedataOverride>, sqlx::Error> {
    sqlx::query_as::<_, GamedataOverride>(
        "SELECT locale, kind, entity_id, field, value, updated_at FROM gamedata_overrides WHERE locale = $1 ORDER BY kind, entity_id, field",
    )
    .bind(locale)
    .fetch_all(pool)
    .await
}

pub async fn upsert_override(
    pool: &PgPool,
    locale: &str,
    kind: &str,
    entity_id: &str,
    field: &str,
    value: &str,
    updated_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO gamedata_overrides (locale, kind, entity_id, field, value, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (locale, kind, entity_id, field) DO UPDATE SET
            value = EXCLUDED.value, updated_by = EXCLUDED.updated_by, updated_at = now()
        ",
    )
    .bind(locale)
    .bind(kind)
    .bind(entity_id)
    .bind(field)
    .bind(value)
    .bind(updated_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_override(
    pool: &PgPool,
    locale: &str,
    kind: &str,
    entity_id: &str,
    field: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM gamedata_overrides WHERE locale = $1 AND kind = $2 AND entity_id = $3 AND field = $4")
        .bind(locale)
        .bind(kind)
        .bind(entity_id)
        .bind(field)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------- documents

pub async fn get_document(
    pool: &PgPool,
    slug: &str,
    locale: &str,
) -> Result<Option<UiDocument>, sqlx::Error> {
    sqlx::query_as::<_, UiDocument>(
        "SELECT slug, locale, title, body, updated_at FROM ui_documents WHERE slug = $1 AND locale = $2",
    )
    .bind(slug)
    .bind(locale)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_document(
    pool: &PgPool,
    slug: &str,
    locale: &str,
    title: &str,
    body: &str,
    updated_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO ui_documents (slug, locale, title, body, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, now())
        ON CONFLICT (slug, locale) DO UPDATE SET
            title = EXCLUDED.title, body = EXCLUDED.body,
            updated_by = EXCLUDED.updated_by, updated_at = now()
        ",
    )
    .bind(slug)
    .bind(locale)
    .bind(title)
    .bind(body)
    .bind(updated_by)
    .execute(pool)
    .await?;
    Ok(())
}
