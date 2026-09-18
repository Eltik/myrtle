use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;
use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::{ApiError, FieldError};
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::i18n::{
    GamedataOverride, Locale, TranslationEntry, TranslationPermission, UiMessageAuditEntry,
};
use crate::database::queries::i18n as queries;

/// Every cache key this feature writes lives under this prefix, so one
/// `invalidate_by_prefix` after a write clears the manifest and any catalog
/// bodies in one call.
const CACHE_PREFIX: &str = "i18n:";

/// The manifest key (and catalog path segment) for the every-namespace
/// catalog. An empty namespace cannot be spelled in a URL, so `-` stands in.
pub const ALL_NAMESPACES: &str = "-";

/// The source-text fingerprint. Computed here rather than in the extractor so
/// that exactly one implementation decides whether two English strings are the
/// same string, and a change to it can never half-apply.
fn source_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))[..16].to_owned()
}

// ---------------------------------------------------------------- manifest

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nManifestLocale {
    pub code: String,
    pub native_name: String,
    pub english_name: String,
    pub gamedata_server: String,
    /// namespace -> current content hash. The hash is the cache-busting path
    /// component of the catalog URL.
    pub namespaces: BTreeMap<String, String>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nManifest {
    pub default_locale: String,
    pub locales: Vec<I18nManifestLocale>,
}

/// The manifest is the only i18n read with a short TTL: it is how a new
/// catalog hash becomes visible, so it bounds how long a saved edit takes to
/// reach a browser.
pub async fn get_manifest(state: &AppState) -> Result<CachedJson, ApiError> {
    cached_json(state, &CacheKey::I18nManifest, || async {
        let locales = queries::list_locales(&state.db, true).await?;
        let namespaces = queries::list_namespaces(&state.db).await?;

        let mut out = Vec::with_capacity(locales.len());
        for locale in locales {
            let mut map = BTreeMap::new();
            // `-` is the every-namespace catalog. The client fetches this one
            // until the catalog is measured large enough to be worth splitting
            // per route; the per-namespace hashes below make that split a
            // client-side change only.
            map.insert(
                ALL_NAMESPACES.to_owned(),
                queries::catalog_hash(&state.db, &locale.code, "").await?,
            );
            for ns in &namespaces {
                let hash = queries::catalog_hash(&state.db, &locale.code, ns).await?;
                map.insert(ns.clone(), hash);
            }
            out.push(I18nManifestLocale {
                code: locale.code,
                native_name: locale.native_name,
                english_name: locale.english_name,
                gamedata_server: locale.gamedata_server,
                namespaces: map,
            });
        }

        // The default is the locale flagged as the source, not a hardcoded
        // string: it is the language the bundled catalog is written in and the
        // floor of every fallback chain, and it should be visible and
        // changeable rather than compiled in. `en` remains the fallback answer
        // if no locale is flagged, so a database mid-migration still serves.
        let default_locale = queries::list_locales(&state.db, false)
            .await?
            .into_iter()
            .find(|l| l.is_source)
            .map_or_else(|| "en".to_owned(), |l| l.code);

        let manifest = I18nManifest {
            default_locale,
            locales: out,
        };
        serde_json::to_string(&manifest).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

// ---------------------------------------------------------------- catalog

/// Whether the hash a client asked for is still the current one. A request for
/// a superseded hash is answered with current content but WITHOUT
/// `immutable`, so a stale URL can never be pinned in a cache forever.
pub struct CatalogResponse {
    pub json: CachedJson,
    pub is_current: bool,
}

pub async fn get_catalog(
    state: &AppState,
    locale: &str,
    namespace: &str,
    requested_hash: &str,
) -> Result<CatalogResponse, ApiError> {
    if queries::get_locale(&state.db, locale).await?.is_none() {
        return Err(ApiError::NotFound);
    }

    let current = queries::catalog_hash(&state.db, locale, namespace).await?;
    let json = cached_json(
        state,
        &CacheKey::I18nCatalog {
            locale,
            namespace,
            hash: &current,
        },
        || async {
            let rows = queries::get_catalog(&state.db, locale, namespace).await?;
            let map: BTreeMap<String, String> = rows.into_iter().collect();
            serde_json::to_string(&map).map_err(|e| ApiError::Internal(e.into()))
        },
    )
    .await?;

    Ok(CatalogResponse {
        json,
        is_current: requested_hash == current,
    })
}

// ---------------------------------------------------------------- authorization

/// The two-part check: the global role is the ticket into the feature, and a
/// row in `translation_permissions` says which locales. `SuperAdmin` skips the
/// grant lookup entirely.
///
/// Because the authoritative half is a database read, a freshly granted locale
/// works on the very next request - unlike `users.role`, which is carried in
/// the JWT and only refreshes with the token.
pub async fn assert_can_write(
    state: &AppState,
    auth: &AuthUser,
    locale: &str,
    required: Permission,
) -> Result<Uuid, ApiError> {
    let user_id = auth.user_uuid()?;

    if auth.role.is_super_admin() {
        return Ok(user_id);
    }

    // The grant row is the authority, not the global role. Requiring
    // `is_translator()` first made a grant inert on its own: the role rides the
    // JWT and is frozen at login, so a freshly granted translator was refused
    // until they logged out and back in - and one left at `User` was refused
    // forever, which is the state a locale grant alone used to produce.
    // Grants are read per request, so one takes effect on the very next call.
    let grants = queries::list_permissions_for_user(&state.db, user_id).await?;
    let allowed = grants.iter().any(|g| {
        g.locale == locale
            && g.permission
                .parse::<Permission>()
                .is_ok_and(|p| p.grants(required))
    });

    if allowed {
        Ok(user_id)
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Whether this caller may load the admin panel's translation surface.
///
/// A staff role admits, and so does any single translation grant - otherwise a
/// locale grant would be unusable, since its holder could not reach the screen
/// that spends it. Resolved per request from the database, like the grants
/// themselves.
pub async fn can_access_admin_panel(state: &AppState, auth: &AuthUser) -> Result<bool, ApiError> {
    if auth.role.can_access_admin_panel() {
        return Ok(true);
    }
    let Ok(user_id) = auth.user_uuid() else {
        return Ok(false);
    };
    Ok(queries::has_any_permission(&state.db, user_id).await?)
}

/// The locales this caller may edit, for the admin UI's locale picker. A
/// super-admin gets every locale; anyone else gets exactly their grants.
pub async fn writable_locales(state: &AppState, auth: &AuthUser) -> Result<Vec<String>, ApiError> {
    if auth.role.is_super_admin() {
        return Ok(queries::list_locales(&state.db, false)
            .await?
            .into_iter()
            .map(|l| l.code)
            .collect());
    }
    // No role precondition, for the same reason `assert_can_write` has none:
    // the grants ARE the answer to "what may this caller edit", and gating the
    // lookup on the JWT role handed a freshly granted translator an empty
    // locale picker on a screen they were otherwise allowed to open.
    let Ok(user_id) = auth.user_uuid() else {
        return Ok(Vec::new());
    };
    let mut codes: Vec<String> = queries::list_permissions_for_user(&state.db, user_id)
        .await?
        .into_iter()
        .filter(|g| {
            g.permission
                .parse::<Permission>()
                .is_ok_and(|p| p.grants(Permission::Edit))
        })
        .map(|g| g.locale)
        .collect();
    codes.sort();
    codes.dedup();
    Ok(codes)
}

// ---------------------------------------------------------------- validation

/// Collect the placeholder names a message references.
///
/// This has to be structural rather than a brace scan. In ICU the braces
/// inside a `plural`/`select` argument delimit BRANCH BODIES, and a branch
/// body is a message, not an argument: in
/// `{count, plural, one {tier} other {tiers}}` the only placeholder is
/// `count`, while `tier` and `tiers` are literal text.
///
/// Reading the first token after every `{` collected those too, so a message
/// whose branch body opens with a word was rejected against its own declared
/// placeholders - 42 catalogue entries could not be translated at all,
/// because even pasting the English source verbatim came back as "unknown
/// placeholder(s)". The old scan tracked brace depth but never consulted it,
/// which is what hid this: `{# operator}` is fine, `{tier}` is not, and every
/// test case happened to start with `#`.
///
/// Mirrors the parse `format.ts` performs on the client, so the validator and
/// the renderer agree about what a message references.
fn referenced_placeholders(message: &str) -> Result<HashSet<String>, String> {
    /// What the next `{` opens.
    enum Ctx {
        /// Inside a message: `{` opens an argument.
        Message,
        /// Inside a `plural`/`select` argument's branch list: `{` opens a
        /// branch body, and the token before it is a branch key (`one`, `=0`,
        /// `other`) rather than a placeholder.
        Branches,
    }

    /// Argument types whose body is a branch list instead of a format style.
    const SELECTORS: [&str; 3] = ["plural", "selectordinal", "select"];

    const fn skip_ws(chars: &[char], mut j: usize) -> usize {
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }
        j
    }

    fn ident_end(chars: &[char], mut j: usize) -> usize {
        while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
            j += 1;
        }
        j
    }

    let chars: Vec<char> = message.chars().collect();
    let mut found = HashSet::new();
    let mut stack = vec![Ctx::Message];
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\'' => {
                // ICU 4.8 "real literal" quoting, matching `format.ts`:
                //
                //   ''            -> a literal apostrophe
                //   '{ '} '# '|   -> opens a quoted run, closed by the next '
                //   ' anywhere else -> a plain apostrophe, not a quote
                //
                // The last clause is load-bearing. Skipping to the next quote
                // on ANY apostrophe made a translation like
                // "don't show {count}" scan past its own argument, so the
                // validator rejected it for a missing `{count}` that was
                // right there.
                if chars.get(i + 1) == Some(&'\'') {
                    i += 1;
                } else if matches!(chars.get(i + 1), Some('{' | '}' | '#' | '|')) {
                    i += 1;
                    while i < chars.len() && chars[i] != '\'' {
                        i += 1;
                    }
                }
            }
            '{' => {
                if matches!(stack.last(), Some(Ctx::Branches)) {
                    stack.push(Ctx::Message);
                } else {
                    let start = skip_ws(&chars, i + 1);
                    let end = ident_end(&chars, start);
                    if end > start {
                        found.insert(chars[start..end].iter().collect::<String>());
                    }

                    let after = skip_ws(&chars, end);
                    if chars.get(after) == Some(&',') {
                        let kind_start = skip_ws(&chars, after + 1);
                        let mut kind_end = kind_start;
                        while kind_end < chars.len() && chars[kind_end].is_alphabetic() {
                            kind_end += 1;
                        }
                        let kind: String = chars[kind_start..kind_end].iter().collect();
                        if SELECTORS.contains(&kind.as_str()) {
                            stack.push(Ctx::Branches);
                            i = kind_end;
                            continue;
                        }
                    }

                    // A plain `{name}`, or a formatted argument such as
                    // `{n, number}` / `{ts, date, short}`. Neither holds a
                    // nested brace, so it closes at the next `}`.
                    stack.push(Ctx::Message);
                    i = end;
                    continue;
                }
            }
            '}' => {
                stack.pop();
                if stack.is_empty() {
                    return Err("unbalanced '}' in message".to_owned());
                }
            }
            _ => {}
        }
        i += 1;
    }

    if stack.len() != 1 {
        return Err("unbalanced '{' in message".to_owned());
    }
    Ok(found)
}

/// Reject a translation that invents a placeholder the source never declared,
/// or drops one the source did.
///
/// This is both a correctness guard and the main abuse guard for direct edit:
/// an invented argument is the one way a translator could make a message do
/// something the developer did not sanction, and an unbalanced brace would
/// throw inside the client formatter and blank the subtree.
fn validate_message(value: &str, declared: &serde_json::Value) -> Result<(), ApiError> {
    let referenced = referenced_placeholders(value).map_err(|e| {
        ApiError::ValidationFailed(vec![FieldError {
            field: "value".to_owned(),
            message: e,
        }])
    })?;

    let declared: HashSet<String> = declared
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(std::borrow::ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();

    let mut errors = Vec::new();

    let mut undeclared: Vec<&String> = referenced.difference(&declared).collect();
    undeclared.sort();
    if !undeclared.is_empty() {
        errors.push(FieldError {
            field: "value".to_owned(),
            message: format!(
                "unknown placeholder(s): {}. The source string declares: {}",
                undeclared
                    .iter()
                    .map(|s| format!("{{{s}}}"))
                    .collect::<Vec<_>>()
                    .join(", "),
                if declared.is_empty() {
                    "none".to_owned()
                } else {
                    let mut d: Vec<&String> = declared.iter().collect();
                    d.sort();
                    d.iter()
                        .map(|s| format!("{{{s}}}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
        });
    }

    let mut missing: Vec<&String> = declared.difference(&referenced).collect();
    missing.sort();
    if !missing.is_empty() {
        errors.push(FieldError {
            field: "value".to_owned(),
            message: format!(
                "missing placeholder(s): {}",
                missing
                    .iter()
                    .map(|s| format!("{{{s}}}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ApiError::ValidationFailed(errors))
    }
}

// ---------------------------------------------------------------- reads

pub async fn list_locales(state: &AppState, enabled_only: bool) -> Result<Vec<Locale>, ApiError> {
    queries::list_locales(&state.db, enabled_only)
        .await
        .map_err(std::convert::Into::into)
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct TranslationListResponse {
    pub entries: Vec<TranslationEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

pub struct ListParams<'a> {
    pub locale: &'a str,
    pub namespace: &'a str,
    pub search: &'a str,
    pub filter: &'a str,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_entries(
    state: &AppState,
    params: &ListParams<'_>,
) -> Result<TranslationListResponse, ApiError> {
    let limit = params.limit.clamp(1, 500);
    let offset = params.offset.max(0);

    let (entries, total) = tokio::try_join!(
        queries::list_entries(
            &state.db,
            params.locale,
            params.namespace,
            params.search,
            params.filter,
            limit,
            offset
        ),
        queries::count_entries(
            &state.db,
            params.locale,
            params.namespace,
            params.search,
            params.filter
        ),
    )?;

    Ok(TranslationListResponse { entries, total })
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct LocaleProgress {
    pub locale: String,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub translated: i64,
    #[ts(type = "number")]
    pub stale: i64,
}

/// Every active namespace, for the translation editor's filter.
///
/// Read straight from the key table rather than derived from a page of
/// messages: a page is at most 500 keys out of several thousand, so deriving
/// the list from one meant a namespace only appeared in the filter once the
/// translator had already paged onto it - which is the opposite of what a
/// filter is for.
pub async fn list_namespaces(state: &AppState) -> Result<Vec<String>, ApiError> {
    queries::list_namespaces(&state.db)
        .await
        .map_err(std::convert::Into::into)
}

pub async fn locale_progress(state: &AppState) -> Result<Vec<LocaleProgress>, ApiError> {
    Ok(queries::locale_progress(&state.db)
        .await?
        .into_iter()
        .map(|(locale, total, translated, stale)| LocaleProgress {
            locale,
            total,
            translated,
            stale,
        })
        .collect())
}

pub async fn get_audit_log(
    state: &AppState,
    key: &str,
    locale: &str,
) -> Result<Vec<UiMessageAuditEntry>, ApiError> {
    queries::get_audit_log(&state.db, key, locale)
        .await
        .map_err(std::convert::Into::into)
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct TranslationAuditActor {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct TranslationAuditEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub actor: TranslationAuditActor,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct TranslationAuditResponse {
    pub entries: Vec<TranslationAuditEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

pub async fn get_global_audit_log(
    state: &AppState,
    locale: Option<&str>,
    limit: i64,
    before: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<TranslationAuditResponse, ApiError> {
    let capped = limit.clamp(1, 500);
    let (rows, total) = tokio::try_join!(
        queries::get_audit_log_global(&state.db, locale, capped, before),
        queries::count_audit_log(&state.db, locale),
    )?;

    let entries = rows
        .into_iter()
        .map(|r| TranslationAuditEntry {
            id: r.id,
            message_key: r.message_key,
            locale: r.locale,
            old_value: r.old_value,
            new_value: r.new_value,
            changed_at: r.changed_at,
            actor: TranslationAuditActor {
                user_id: r.actor_user_id,
                uid: r.actor_uid,
                nickname: r.actor_nickname,
                secretary: r.actor_secretary,
                secretary_skin_id: r.actor_secretary_skin_id,
            },
        })
        .collect();

    Ok(TranslationAuditResponse { entries, total })
}

// ---------------------------------------------------------------- writes

/// Write one translation. Validates against the key's declared placeholders,
/// stamps the source hash it was translated against, writes one audit row when
/// the value actually changed, and drops the i18n cache so the manifest
/// publishes a new hash.
pub async fn update_message(
    state: &AppState,
    locale: &str,
    key: &str,
    value: &str,
    changed_by: Uuid,
) -> Result<TranslationEntry, ApiError> {
    let existing = queries::get_entry(&state.db, locale, key)
        .await?
        .ok_or(ApiError::NotFound)?;

    validate_message(value, &existing.placeholders)?;

    // Re-stamping the current source hash is what clears the stale flag, so a
    // no-op save on a stale row is still meaningful and must go through.
    let changed = existing.value.as_deref() != Some(value);

    // The snapshot is the source that was on screen for this save, which is
    // the key's current source_text - the same text the hash is taken over.
    queries::upsert_message(
        &state.db,
        key,
        locale,
        value,
        &existing.source_hash,
        &existing.source_text,
        changed_by,
    )
    .await?;

    if changed {
        queries::insert_audit(
            &state.db,
            key,
            locale,
            existing.value.as_deref(),
            Some(value),
            changed_by,
        )
        .await?;
    }

    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;

    queries::get_entry(&state.db, locale, key)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Clear a translation, falling the key back to its fallback locale or the
/// bundled English source. Audited as a change to `NULL` so it can be reverted
/// like any other edit.
pub async fn clear_message(
    state: &AppState,
    locale: &str,
    key: &str,
    changed_by: Uuid,
) -> Result<(), ApiError> {
    let existing = queries::get_entry(&state.db, locale, key)
        .await?
        .ok_or(ApiError::NotFound)?;

    if existing.value.is_none() {
        return Ok(());
    }

    queries::delete_message(&state.db, key, locale).await?;
    queries::insert_audit(
        &state.db,
        key,
        locale,
        existing.value.as_deref(),
        None,
        changed_by,
    )
    .await?;
    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;
    Ok(())
}

// ---------------------------------------------------------------- key sync

#[derive(Debug, Deserialize)]
pub struct SourceEntry {
    pub key: String,
    pub namespace: String,
    pub source_text: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub placeholders: Vec<String>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Serialize)]
pub struct SyncResult {
    #[ts(type = "number")]
    pub upserted: u64,
    #[ts(type = "number")]
    pub deactivated: u64,
}

/// Take the extractor's output as the truth about what keys exist and what
/// their English source is. Keys that vanish are deactivated rather than
/// deleted, so a reverted refactor does not destroy translations.
pub async fn sync_source_catalog(
    state: &AppState,
    entries: &[SourceEntry],
) -> Result<SyncResult, ApiError> {
    if entries.is_empty() {
        return Err(ApiError::BadRequest(
            "refusing to sync an empty catalog: that would deactivate every key".to_owned(),
        ));
    }

    let keys: Vec<String> = entries.iter().map(|e| e.key.clone()).collect();
    let namespaces: Vec<String> = entries.iter().map(|e| e.namespace.clone()).collect();
    let source_texts: Vec<String> = entries.iter().map(|e| e.source_text.clone()).collect();
    let hashes: Vec<String> = entries
        .iter()
        .map(|e| source_hash(&e.source_text))
        .collect();
    let descriptions: Vec<Option<String>> = entries.iter().map(|e| e.description.clone()).collect();
    let placeholders: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| serde_json::json!(e.placeholders))
        .collect();

    let upserted = queries::sync_keys(
        &state.db,
        &keys,
        &namespaces,
        &source_texts,
        &hashes,
        &descriptions,
        &placeholders,
    )
    .await?;
    let deactivated = queries::deactivate_missing_keys(&state.db, &keys).await?;

    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;

    Ok(SyncResult {
        upserted,
        deactivated,
    })
}

// ---------------------------------------------------------------- grants

pub async fn list_permissions(
    state: &AppState,
    locale: Option<&str>,
) -> Result<Vec<TranslationPermission>, ApiError> {
    queries::list_permissions(&state.db, locale)
        .await
        .map_err(std::convert::Into::into)
}

pub async fn grant_permission(
    state: &AppState,
    locale: &str,
    user_id: Uuid,
    permission: Permission,
    granted_by: Uuid,
) -> Result<(), ApiError> {
    queries::grant_permission(
        &state.db,
        locale,
        user_id,
        &permission.to_string(),
        granted_by,
    )
    .await
    .map_err(std::convert::Into::into)
}

pub async fn revoke_permission(
    state: &AppState,
    locale: &str,
    user_id: Uuid,
    permission: Permission,
) -> Result<(), ApiError> {
    queries::revoke_permission(&state.db, locale, user_id, &permission.to_string())
        .await
        .map_err(std::convert::Into::into)
}

// ---------------------------------------------------------------- overrides

pub async fn list_overrides(
    state: &AppState,
    locale: &str,
) -> Result<Vec<GamedataOverride>, ApiError> {
    queries::list_overrides(&state.db, locale)
        .await
        .map_err(std::convert::Into::into)
}

pub struct OverrideInput<'a> {
    pub locale: &'a str,
    pub kind: &'a str,
    pub entity_id: &'a str,
    pub field: &'a str,
    pub value: &'a str,
}

/// Game-data overrides invalidate the static-data cache as well as the i18n
/// one, because the override is applied when game data is serialized.
pub async fn upsert_override(
    state: &AppState,
    input: &OverrideInput<'_>,
    updated_by: Uuid,
) -> Result<(), ApiError> {
    queries::upsert_override(
        &state.db,
        input.locale,
        input.kind,
        input.entity_id,
        input.field,
        input.value,
        updated_by,
    )
    .await?;
    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;
    state.cache.invalidate_by_prefix("static:").await;
    Ok(())
}

pub async fn delete_override(
    state: &AppState,
    locale: &str,
    kind: &str,
    entity_id: &str,
    field: &str,
) -> Result<(), ApiError> {
    queries::delete_override(&state.db, locale, kind, entity_id, field).await?;
    state.cache.invalidate_by_prefix(CACHE_PREFIX).await;
    state.cache.invalidate_by_prefix("static:").await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(names: &[&str]) -> serde_json::Value {
        serde_json::json!(names)
    }

    #[test]
    fn accepts_a_translation_using_exactly_the_declared_placeholders() {
        assert!(validate_message("{count} operators", &declared(&["count"])).is_ok());
        assert!(validate_message("No operators", &declared(&[])).is_ok());
    }

    /// The regression that made 42 catalogue entries untranslatable: a branch
    /// body opening with a word was read as a placeholder, so the English
    /// source failed validation against its own declared placeholders.
    #[test]
    fn a_branch_body_is_text_not_a_placeholder() {
        let found =
            referenced_placeholders("+ {names}{extra} {count, plural, one {tier} other {tiers}}")
                .expect("balanced");
        assert_eq!(
            found,
            ["names", "extra", "count"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<HashSet<String>>()
        );

        // The shape that reported it, end to end: pasting the source verbatim
        // has to validate against the source's own declaration.
        assert!(
            validate_message(
                "+ {names}{extra} {count, plural, one {tier} other {tiers}}",
                &declared(&["count", "extra", "names"]),
            )
            .is_ok()
        );
    }

    #[test]
    fn branch_bodies_of_every_selector_are_text() {
        for message in [
            "{count, plural, one {Delete plan?} other {Delete # plans?}}",
            "{names} and {count, plural, one {its} other {their}} goals are removed.",
            "{scale, select, day {Previous day} week {Previous week} other {Previous month}}",
            "{count, plural, =0 {None} one {# copy} other {# copies}}",
            "{noun, select, healer {Add a healer} other {Add an operator}}",
            "{n, selectordinal, one {#st} two {#nd} few {#rd} other {#th}}",
        ] {
            let found = referenced_placeholders(message).expect("balanced");
            for phantom in [
                "Delete", "its", "their", "Previous", "None", "Add", "day", "week",
            ] {
                assert!(
                    !found.contains(phantom),
                    "{message:?} leaked branch text {phantom:?}"
                );
            }
        }
    }

    /// A nested argument inside a branch body is still an argument - the fix
    /// must not skip those.
    #[test]
    fn a_nested_argument_inside_a_branch_is_still_collected() {
        let found = referenced_placeholders(
            "{count, plural, one {# of {total}} other {# of {total}, {extra}}}",
        )
        .expect("balanced");
        assert!(found.contains("count"));
        assert!(found.contains("total"));
        assert!(found.contains("extra"));
    }

    /// A format style is not a branch list, so its argument name still counts
    /// and its style tokens are not placeholders.
    #[test]
    fn formatted_arguments_keep_their_name_only() {
        for (message, name) in [
            ("{n, number}", "n"),
            ("{ts, date, short}", "ts"),
            ("{ts, time}", "ts"),
        ] {
            let found = referenced_placeholders(message).expect("balanced");
            assert_eq!(found.len(), 1, "{message:?}");
            assert!(found.contains(name), "{message:?}");
        }
    }

    #[test]
    fn reads_the_argument_name_out_of_icu_forms() {
        let found =
            referenced_placeholders("{count, plural, one {# operator} other {# operators}}")
                .expect("balanced");
        assert_eq!(found.len(), 1);
        assert!(found.contains("count"));
    }

    #[test]
    fn rejects_an_invented_placeholder() {
        let err = validate_message("{count} of {total}", &declared(&["count"]));
        assert!(matches!(err, Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn rejects_a_dropped_placeholder() {
        let err = validate_message("many operators", &declared(&["count"]));
        assert!(matches!(err, Err(ApiError::ValidationFailed(_))));
    }

    #[test]
    fn rejects_unbalanced_braces() {
        assert!(referenced_placeholders("{count operators").is_err());
        assert!(referenced_placeholders("count} operators").is_err());
    }

    #[test]
    fn quoted_braces_are_literals_not_arguments() {
        let found = referenced_placeholders("'{'not an arg'}'").expect("balanced");
        assert!(found.is_empty());
    }

    #[test]
    fn a_contraction_does_not_hide_the_arguments_after_it() {
        // Regression: treating every apostrophe as a quote made the scan run
        // past `{count}` and reject the translation for omitting it.
        let found = referenced_placeholders("don't show {count} of {total}").expect("balanced");
        assert_eq!(found.len(), 2);
        assert!(found.contains("count"));
        assert!(found.contains("total"));

        assert!(validate_message("don't show {count}", &declared(&["count"])).is_ok());
    }

    #[test]
    fn a_doubled_apostrophe_is_a_literal_and_hides_nothing() {
        let found = referenced_placeholders("Most E2''d: {count}").expect("balanced");
        assert_eq!(found.len(), 1);
        assert!(found.contains("count"));
    }

    #[test]
    fn source_hash_is_stable_and_distinguishes_edits() {
        assert_eq!(source_hash("Next"), source_hash("Next"));
        assert_ne!(source_hash("Next"), source_hash("Next "));
        assert_eq!(source_hash("Next").len(), 16);
    }
}
