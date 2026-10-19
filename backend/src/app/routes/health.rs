use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::app::state::{AppState, ServerData};
use crate::core::gamedata::stale::StaleTable;
use crate::core::gamedata::types::GameData;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    status: &'static str,
    cache: CacheHealth,
    database: DatabaseHealth,
    game_data: GameDataHealth,
    timestamp: String,
    response_time_ms: f64,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct CacheHealth {
    #[schema(value_type = String)]
    backend: &'static str,
    #[schema(value_type = String)]
    status: &'static str,
    response_time_ms: f64,
}

/// Game data problems from the most recent load, per server, and what each
/// server serves. Two kinds of problem, kept apart:
///
/// - `degraded` / `tables`: a table failed to DESERIALIZE and fell back to an empty
///   default, so the site serves nothing where that table's content should be.
/// - `stale`: a table the unpacker SKIPPED because it verified under no schema, so
///   the file on disk is an earlier extract's. It parses and serves plausible data
///   that is missing whatever the newer patch added. Never counted in `degraded`.
///
/// Neither flips the top-level `status`: both are data problems, not availability
/// ones, and paging an uptime monitor for a cosmetic table would train everyone to
/// ignore it.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct GameDataHealth {
    /// True when any server has a table in `tables`. Says nothing about `stale`.
    degraded: bool,
    /// `{ "cn": ["item_table: ...", ...] }`, tables that fell back to an empty
    /// default. Absent servers loaded cleanly.
    tables: BTreeMap<String, Vec<String>>,
    /// `{ "en": [{ "table": "activity_table", "since": ..., "resVersion": ... }] }`,
    /// tables served from an earlier extract since `since`. Absent servers are
    /// current, or predate the unpacker's `gamedata/.skipped.json`.
    stale: BTreeMap<String, Vec<StaleTable>>,
    /// `{ "en": { "version": "26-10-03-16-06-39_0a6acf", "loadedAt": ..., "lazy": false } }`,
    /// the build every configured server serves now. Present for every
    /// configured server, loaded or not.
    loaded: BTreeMap<String, LoadedGameData>,
}

/// The game data one server serves now: the asset version it was built from
/// and when that build was published (`ServerData::load_record`).
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct LoadedGameData {
    /// The asset pipeline's `update_complete` version the served build was
    /// read under, the same string as the watcher's `.version` file. A build
    /// with no version of its own keeps the previous one; null when the
    /// server has never had one since boot (the boot load knows none) or
    /// holds no data (failed to load, or lazy and unloaded).
    version: Option<String>,
    /// RFC 3339 UTC of the last full build published; null when the server
    /// holds no data. A sidecar patch does not move it.
    loaded_at: Option<String>,
    /// The server loads on its first request and unloads when idle
    /// (`SERVERS_LAZY`). A lazy server that is not resident has null
    /// `version` and `loadedAt`.
    lazy: bool,
}

impl LoadedGameData {
    fn of(sd: &ServerData) -> Self {
        let stamp = sd
            .loaded
            .load(Ordering::Acquire)
            .then(|| sd.load_record.last())
            .flatten();
        Self {
            version: stamp.as_ref().and_then(|s| s.version.clone()),
            loaded_at: stamp.map(|s| s.at.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
            lazy: sd.residency.lazy,
        }
    }
}

impl GameDataHealth {
    fn from_servers<'a>(
        servers: impl IntoIterator<Item = (&'a str, &'a GameData)>,
        loaded: BTreeMap<String, LoadedGameData>,
    ) -> Self {
        let mut tables = BTreeMap::new();
        let mut stale = BTreeMap::new();
        for (srv, gd) in servers {
            if !gd.table_warnings.is_empty() {
                tables.insert(srv.to_owned(), gd.table_warnings.clone());
            }
            if !gd.stale_tables.is_empty() {
                stale.insert(srv.to_owned(), gd.stale_tables.clone());
            }
        }
        Self {
            degraded: !tables.is_empty(),
            tables,
            stale,
            loaded,
        }
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct DatabaseHealth {
    #[schema(value_type = String)]
    status: &'static str,
    response_time_ms: f64,
}

fn elapsed_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// Liveness and build information.
///
/// Unauthenticated and never rate-limited into uselessness; this is the probe
/// a monitor should poll.
#[utoipa::path(
    get,
    path = "/health",
    tag = "meta",
    responses(
        (status = 200, description = "The service is up.", body = HealthResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let start = Instant::now();

    let cache_start = Instant::now();
    let cache_ok = state.cache.ping().await;
    let cache_ms = elapsed_ms(cache_start.elapsed());

    let backend = if state.cache.is_redis() {
        "redis"
    } else {
        "memory"
    };

    let db_start = Instant::now();
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let db_ms = elapsed_ms(db_start.elapsed());

    let all_ok = cache_ok && db_ok;

    let loaded: Vec<_> = state
        .config
        .servers
        .iter()
        .filter_map(|&srv| {
            state
                .try_server_data(srv)
                .map(|sd| (srv.as_str(), sd.game_data.load_full()))
        })
        .collect();
    let versions = state
        .config
        .servers
        .iter()
        .filter_map(|&srv| {
            state
                .entry(srv)
                .map(|sd| (srv.as_str().to_owned(), LoadedGameData::of(&sd)))
        })
        .collect();
    let game_data =
        GameDataHealth::from_servers(loaded.iter().map(|(srv, gd)| (*srv, &**gd)), versions);

    Json(HealthResponse {
        status: if all_ok { "ok" } else { "degraded" },
        game_data,
        cache: CacheHealth {
            backend,
            status: if cache_ok {
                "connected"
            } else {
                "disconnected"
            },
            response_time_ms: cache_ms,
        },
        database: DatabaseHealth {
            status: if db_ok { "connected" } else { "disconnected" },
            response_time_ms: db_ms,
        },
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        response_time_ms: elapsed_ms(start.elapsed()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::GameTables;
    use std::collections::HashMap;

    fn stale(table: &str) -> StaleTable {
        StaleTable {
            table: table.into(),
            since: "2026-10-14T05:00:00Z".into(),
            res_version: Some("26-10-03-16-06-39_0a6acf".into()),
        }
    }

    /// A stale table shows under `stale` and leaves `degraded` false; a fallen-back
    /// one is the only thing that sets `degraded`. Clean servers are absent from both.
    #[test]
    fn stale_and_degraded_are_reported_apart() {
        let tables = |t: GameTables| GameData::from_parts(t, Default::default(), HashMap::new());
        let en = tables(GameTables {
            stale_tables: vec![stale("activity_table")],
            ..GameTables::default()
        });
        let cn = tables(GameTables {
            table_warnings: vec!["item_table: unknown variant".into()],
            ..GameTables::default()
        });
        let jp = GameData::default();

        let only_stale = GameDataHealth::from_servers([("en", &en), ("jp", &jp)], BTreeMap::new());
        assert_eq!(
            serde_json::to_value(&only_stale).unwrap(),
            serde_json::json!({
                "degraded": false,
                "tables": {},
                "stale": { "en": [{
                    "table": "activity_table",
                    "since": "2026-10-14T05:00:00Z",
                    "resVersion": "26-10-03-16-06-39_0a6acf"
                }] },
                "loaded": {}
            })
        );

        let both =
            GameDataHealth::from_servers([("en", &en), ("cn", &cn), ("jp", &jp)], BTreeMap::new());
        let v = serde_json::to_value(&both).unwrap();
        assert_eq!(v["degraded"], true);
        assert_eq!(
            v["tables"],
            serde_json::json!({ "cn": ["item_table: unknown variant"] })
        );
        assert_eq!(
            v["stale"].as_object().unwrap().keys().collect::<Vec<_>>(),
            ["en"]
        );
    }

    /// `gameData.loaded` names the version each configured server serves: none
    /// after the boot load, the `update_complete` version once a reload under
    /// it swaps, the same version after a versionless rebuild, and nothing for
    /// a lazy server while it is not resident. Through the real handler.
    #[allow(clippy::await_holding_lock)] // test-only serialisation
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn loaded_names_the_served_version_per_server() {
        use crate::app::residency::{ensure_loaded, tests as res};
        use crate::core::asset_watcher::perform_reload;
        use crate::core::hypergryph::constants::Server;

        async fn loaded(state: &AppState) -> serde_json::Value {
            let body = health(State(state.clone())).await.0;
            serde_json::to_value(&body).unwrap()["gameData"]["loaded"].clone()
        }
        fn at(v: &serde_json::Value) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::parse_from_rfc3339(v["loadedAt"].as_str().expect("loadedAt"))
                .expect("RFC 3339")
                .to_utc()
        }

        let _serial = res::serial();
        let _cpu = crate::app::cpu::tests::POOL.lock().await;
        let scratch = res::Scratch::new("health");
        scratch.write_server(Server::EN);
        scratch.write_server(Server::JP);
        let state = res::state_for(scratch.base(), &[Server::EN, Server::JP], &[Server::JP]);

        let boot = loaded(&state).await;
        assert_eq!(
            boot["en"]["version"],
            serde_json::Value::Null,
            "boot knows no version"
        );
        assert_eq!(boot["en"]["lazy"], false);
        let boot_at = at(&boot["en"]);
        assert_eq!(
            boot["jp"],
            serde_json::json!({ "version": null, "loadedAt": null, "lazy": true }),
            "lazy and not resident"
        );

        perform_reload(&state, Server::EN, Some("26-10-19-v2")).await;
        let v2 = loaded(&state).await;
        assert_eq!(v2["en"]["version"], "26-10-19-v2");
        assert!(at(&v2["en"]) >= boot_at);

        perform_reload(&state, Server::EN, Some("26-10-20-v3")).await;
        let v3 = loaded(&state).await;
        assert_eq!(
            v3["en"]["version"], "26-10-20-v3",
            "a reload moves the version"
        );
        assert!(at(&v3["en"]) >= at(&v2["en"]));

        // A rebuild with no version of its own read the same extract.
        perform_reload(&state, Server::EN, None).await;
        assert_eq!(loaded(&state).await["en"]["version"], "26-10-20-v3");

        // An update while JP is unloaded waits for its load, which reports it.
        perform_reload(&state, Server::JP, Some("26-10-20-v3")).await;
        assert_eq!(
            loaded(&state).await["jp"]["version"],
            serde_json::Value::Null
        );
        ensure_loaded(&state, Server::JP, std::time::Duration::from_secs(20))
            .await
            .expect("JP loads on request");
        let jp = loaded(&state).await;
        assert_eq!(jp["jp"]["version"], "26-10-20-v3");
        assert_eq!(jp["jp"]["lazy"], true);
        at(&jp["jp"]);
        assert_eq!(jp.as_object().unwrap().len(), 2, "configured servers only");

        // Unloaded when idle: no data, so no version, until the next load.
        crate::app::residency::unload_idle(&state, std::time::Duration::ZERO).await;
        assert_eq!(
            loaded(&state).await["jp"],
            serde_json::json!({ "version": null, "loadedAt": null, "lazy": true })
        );
    }
}
