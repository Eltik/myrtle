use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use myrtle_db::repos::i18n;
use serde::{Deserialize, Serialize};

use super::CACHE_PREFIX;
use crate::cache::CachedJson;
use crate::{AppContext, AppError};

/// The every-namespace catalog's name in the manifest and in a catalog URL
pub const ALL_NAMESPACES: &str = "-";

const MANIFEST_KEY: &str = "i18n:manifest";
/// Bounds how long a saved edit takes to reach a browser
const MANIFEST_TTL: Duration = Duration::from_secs(30);
/// Content-addressed, so it cannot go stale
const CATALOG_TTL: Duration = Duration::from_hours(24);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nManifestLocale {
    pub code: String,
    pub native_name: String,
    pub english_name: String,
    pub gamedata_server: String,
    /// Namespace to content hash, the cache-busting part of the catalog URL
    pub namespaces: BTreeMap<String, String>,
    pub translated: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nManifest {
    pub default_locale: String,
    pub locales: Vec<I18nManifestLocale>,
    /// The loaded game-data servers, the default first
    pub gamedata_servers: Vec<String>,
}

pub struct CatalogResponse {
    pub json: CachedJson,
    /// `false` for a superseded or `latest` hash: serve it, but never as immutable
    pub is_current: bool,
}

/// Enabled locales with the current hash of each namespace's catalog
pub async fn manifest(ctx: &AppContext) -> Result<CachedJson, AppError> {
    ctx.cache
        .json(MANIFEST_KEY, MANIFEST_TTL, || async {
            let mut conn = ctx.db.conn().await?;
            let locales = i18n::list_locales(&mut conn, false).await?;
            let namespaces = i18n::list_namespaces(&mut conn).await?;
            let completion: HashMap<String, (i64, i64)> = i18n::locale_completion(&mut conn)
                .await?
                .into_iter()
                .map(|r| (r.code, (r.current, r.total)))
                .collect();

            let default_locale = locales
                .iter()
                .find(|l| l.is_source)
                .map_or_else(|| "en".to_owned(), |l| l.code.clone());
            let mut out = Vec::with_capacity(locales.len());
            for locale in locales.into_iter().filter(|l| l.enabled) {
                let mut hashes = BTreeMap::new();
                hashes.insert(
                    ALL_NAMESPACES.to_owned(),
                    i18n::catalog_hash(&mut conn, &locale.code, "").await?,
                );
                for ns in &namespaces {
                    let hash = i18n::catalog_hash(&mut conn, &locale.code, ns).await?;
                    hashes.insert(ns.clone(), hash);
                }
                let (translated, total) = completion.get(&locale.code).copied().unwrap_or((0, 0));
                out.push(I18nManifestLocale {
                    code: locale.code,
                    native_name: locale.native_name,
                    english_name: locale.english_name,
                    gamedata_server: locale.gamedata_server,
                    namespaces: hashes,
                    translated,
                    total,
                });
            }

            let manifest = I18nManifest {
                default_locale,
                locales: out,
                gamedata_servers: ctx
                    .snapshots
                    .loaded()
                    .into_iter()
                    .map(|s| s.as_str().to_owned())
                    .collect(),
            };
            serde_json::to_string(&manifest).map_err(AppError::internal)
        })
        .await
}

/// One locale's rendered catalog; `namespace` is [`ALL_NAMESPACES`] for every one
pub async fn catalog(
    ctx: &AppContext,
    locale: &str,
    namespace: &str,
    requested_hash: &str,
) -> Result<CatalogResponse, AppError> {
    let namespace = if namespace == ALL_NAMESPACES {
        ""
    } else {
        namespace
    };
    let mut conn = ctx.db.conn().await?;
    if i18n::get_locale(&mut conn, locale).await?.is_none() {
        return Err(AppError::NotFound(None));
    }
    let current = i18n::catalog_hash(&mut conn, locale, namespace).await?;
    let key = format!("{CACHE_PREFIX}catalog:{locale}:{namespace}:{current}");
    let conn = &mut *conn;
    let json = ctx
        .cache
        .json(&key, CATALOG_TTL, || async move {
            let map: BTreeMap<String, String> = i18n::get_catalog(conn, locale, namespace)
                .await?
                .into_iter()
                .collect();
            serde_json::to_string(&map).map_err(AppError::internal)
        })
        .await?;
    Ok(CatalogResponse {
        json,
        is_current: requested_hash == current,
    })
}
