//! The search route as the server assembles it: the real handler, over a real
//! `AppState` with real game data and the local roster.
//!
//! The query-layer sweep in `user_search_test` proves the SQL, but it hands
//! the builder operator ids the test itself picked. Two things only this can
//! reach: the handler wiring (extractor, service, cache, error to status),
//! and `Scope::resolve`, which turns `class:WARRIOR` or `sub:centurion` into
//! the ids the SQL counts by reading each loaded server's game data. Here the
//! scopes are checked against `character_table.json` read straight off disk,
//! so the loader's own view is never its own oracle.
//!
//! Needs the local Postgres AND the extracted game data, so it is ignored by
//! default:
//! `cargo test --release --test search_handler_test -- --ignored --nocapture`

use std::collections::{HashMap, HashSet};

use axum::extract::{Query, State};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use backend::app::cache::store::CacheStore;
use backend::app::routes::search::{SearchParams, search};
use backend::app::state::{AppConfig, AppState, derive_game_data_dir, load_server_map};
use backend::core::hypergryph::config::GlobalConfig;
use backend::core::hypergryph::constants::Server;
use backend::core::hypergryph::{config, loaders};
use backend::core::service_account::ServiceAccounts;
use backend::database::models::user::SearchEntry;
use sqlx::PgPool;

/// The same state the server builds, with a private in-memory cache so one
/// case never answers another.
async fn state() -> AppState {
    dotenv::dotenv().ok();
    let config = AppConfig::from_env();
    let servers = load_server_map(&config, |_| ());
    let default_server = config.default_server;
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let db = backend::database::init(&database_url)
        .await
        .expect("database");
    let http_client = reqwest::Client::new();
    config::init_config(GlobalConfig::new());
    loaders::init(&http_client).await;
    let service_accounts = ServiceAccounts::load(&config.servers);
    AppState::new(
        db,
        CacheStore::new_memory(),
        servers,
        default_server,
        config,
        http_client,
        service_accounts,
    )
}

/// One request, exactly as axum would run it: extractor, handler, page.
async fn get(state: &AppState, query: &str) -> Result<(Vec<SearchEntry>, i64), StatusCode> {
    let uri: Uri = format!("http://test/search?{query}").parse().expect("uri");
    let params = Query::<SearchParams>::try_from_uri(&uri)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .0;
    match search(State(state.clone()), Query(params)).await {
        Ok(page) => {
            let page = page.0;
            Ok((page.entries, page.total))
        }
        Err(e) => Err(e.into_response().status()),
    }
}

/// `character_table.json` read off disk: id to (profession, archetype), for
/// the operators a player can obtain. `char_patch_table.json` carries Amiya's
/// branch forms, which the game stores apart from the main table.
fn operators_from_disk(base: &str, server: Server) -> HashMap<String, (String, String)> {
    let dir = derive_game_data_dir(base, server);
    let mut out = HashMap::new();
    let Ok(raw) = std::fs::read_to_string(format!("{dir}/character_table.json")) else {
        return out;
    };
    let table: serde_json::Value = serde_json::from_str(&raw).expect("character_table json");
    let mut take = |key: &str, value: &serde_json::Value| {
        if value
            .get("IsNotObtainable")
            .and_then(serde_json::Value::as_bool)
            != Some(false)
        {
            return;
        }
        let profession = value
            .get("Profession")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        // Tokens and traps are summonables, never a roster entry.
        if profession == "TOKEN" || profession == "TRAP" || profession.is_empty() {
            return;
        }
        let sub = value
            .get("SubProfessionId")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        out.insert(key.to_owned(), (profession, sub));
    };
    for entry in table["Characters"].as_array().expect("Characters") {
        take(entry["key"].as_str().expect("key"), &entry["value"]);
    }
    // `PatchChars` is the same `{key, value}` array as `Characters`, not a map.
    if let Ok(raw) = std::fs::read_to_string(format!("{dir}/char_patch_table.json")) {
        let patch: serde_json::Value = serde_json::from_str(&raw).expect("char_patch json");
        if let Some(chars) = patch["PatchChars"].as_array() {
            for entry in chars {
                take(entry["key"].as_str().expect("patch key"), &entry["value"]);
            }
        }
    }
    out
}

/// Every public player's roster in two queries, for the checks that have to
/// count the whole population rather than one page.
struct PublicRoster {
    server: Server,
    owned: HashSet<String>,
}

async fn all_rosters(pool: &PgPool) -> Vec<PublicRoster> {
    let users: Vec<(sqlx::types::Uuid, String)> = sqlx::query_as(
        "SELECT u.id, lower(s.code) FROM users u JOIN servers s ON s.id = u.server_id \
         WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)",
    )
    .fetch_all(pool)
    .await
    .expect("public users");
    let mut by_id: HashMap<sqlx::types::Uuid, PublicRoster> = users
        .into_iter()
        .filter_map(|(id, code)| {
            Server::parse(&code).map(|server| {
                (
                    id,
                    PublicRoster {
                        server,
                        owned: HashSet::new(),
                    },
                )
            })
        })
        .collect();
    let rows: Vec<(sqlx::types::Uuid, String)> =
        sqlx::query_as("SELECT user_id, operator_id FROM user_operators")
            .fetch_all(pool)
            .await
            .expect("rosters");
    for (user_id, operator_id) in rows {
        if let Some(user) = by_id.get_mut(&user_id) {
            user.owned.insert(operator_id);
        }
    }
    by_id.into_values().collect()
}

/// What one player owns, straight from the roster table.
async fn owned(pool: &PgPool, uid: &str) -> HashSet<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT uo.operator_id FROM user_operators uo JOIN users u ON u.id = uo.user_id WHERE u.uid = $1",
    )
    .bind(uid)
    .fetch_all(pool)
    .await
    .expect("roster")
    .into_iter()
    .collect()
}

#[tokio::test]
#[ignore = "reads the local Postgres roster and the extracted game data"]
async fn the_route_serves_every_shape() {
    let state = state().await;
    let pool = state.db.clone();
    let base = state.config.assets_base_dir.clone();
    let loaded: Vec<Server> = state.config.servers.clone();
    println!("servers loaded: {loaded:?}");

    // The default call: the order the search always had, and no metric.
    let (entries, total) = get(&state, "limit=24").await.expect("default page");
    assert_eq!(entries.len(), 24);
    assert!(entries.iter().all(|e| e.metric.is_none()));
    let baseline: Vec<String> = sqlx::query_scalar(
        "SELECT uid FROM v_user_profile WHERE public_profile = true \
         ORDER BY total_score DESC NULLS LAST, uid LIMIT 24",
    )
    .fetch_all(&pool)
    .await
    .expect("baseline");
    let got: Vec<String> = entries.iter().map(|e| e.profile.uid.clone()).collect();
    assert_eq!(got, baseline, "the default route page changed");
    println!("default page: {total} players, order unchanged");

    // Every plain sort answers, and the metric it carries is the one the
    // column says. `joined` is the only one that can be null.
    for sort in [
        "operators",
        "joined",
        "enemies",
        "potentials",
        "masteries",
        "modules",
        "skins",
    ] {
        let (entries, total) = get(&state, &format!("sort={sort}&limit=5"))
            .await
            .unwrap_or_else(|s| panic!("{sort} answered {s}"));
        assert!(!entries.is_empty(), "{sort}: empty page");
        assert_eq!(total, 2585, "{sort}: total");
        if sort != "joined" {
            assert!(
                entries.iter().all(|e| e.metric.is_some()),
                "{sort}: a row carried no metric"
            );
        }
        println!("sort={sort}: top metric {:?}", entries[0].metric);
    }

    // The direction a caller gets when they ask for none: oldest accounts
    // first for `joined`, highest first for a count. Checked here and not
    // only on the enum, because this is the order a visitor actually sees.
    let (oldest, _) = get(&state, "sort=joined&limit=1").await.expect("joined");
    let (newest, _) = get(&state, "sort=joined&dir=desc&limit=1")
        .await
        .expect("joined desc");
    let (min_ts, max_ts): (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT min(st.register_ts), max(st.register_ts) FROM user_status st \
         JOIN users u ON u.id = st.user_id \
         WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)",
    )
    .fetch_one(&pool)
    .await
    .expect("register_ts bounds");
    assert_eq!(
        oldest[0].metric, min_ts,
        "sort=joined must default to oldest first"
    );
    assert_eq!(
        newest[0].metric, max_ts,
        "sort=joined&dir=desc must lead with the newest"
    );
    let (most, _) = get(&state, "sort=masteries&limit=1")
        .await
        .expect("masteries");
    let top_masteries: Option<i64> = sqlx::query_scalar(
        "SELECT max(n) FROM (SELECT count(*) AS n FROM user_operator_skills s \
           JOIN users u ON u.id = s.user_id \
           WHERE s.specialize_level = 3 AND EXISTS \
             (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile) \
           GROUP BY s.user_id) t",
    )
    .fetch_one(&pool)
    .await
    .expect("top masteries");
    assert_eq!(
        most[0].metric, top_masteries,
        "a count sort must default to highest first"
    );
    println!(
        "defaults: joined leads {:?} (oldest), masteries leads {:?} (highest)",
        oldest[0].metric, most[0].metric
    );

    // The scoped sorts: the ids come from game data, so the count is checked
    // against the professions read off disk for the row's own server.
    let disk: HashMap<Server, HashMap<String, (String, String)>> = loaded
        .iter()
        .map(|&server| (server, operators_from_disk(&base, server)))
        .collect();
    let en = disk.get(&Server::EN).expect("EN game data");
    assert!(
        en.len() > 300,
        "EN should carry hundreds of obtainable operators, saw {}",
        en.len()
    );
    assert_eq!(
        en.get("char_1001_amiya2").map(|(p, _)| p.as_str()),
        Some("WARRIOR"),
        "Amiya's Guard form must come from the patch table"
    );

    for (token, matches) in [
        (
            "class:WARRIOR",
            Box::new(|p: &(String, String)| p.0 == "WARRIOR")
                as Box<dyn Fn(&(String, String)) -> bool>,
        ),
        (
            "sub:centurion",
            Box::new(|p: &(String, String)| p.1 == "centurion"),
        ),
    ] {
        let (entries, total) = get(&state, &format!("sort={token}&limit=5"))
            .await
            .unwrap_or_else(|s| panic!("{token} answered {s}"));
        assert_eq!(total, 2585, "{token}: total");
        assert!(!entries.is_empty(), "{token}: empty page");
        for row in &entries {
            let server = Server::parse(&row.profile.server).expect("server code");
            let table = disk.get(&server).unwrap_or(en);
            let expected = owned(&pool, &row.profile.uid)
                .await
                .iter()
                .filter(|id| table.get(*id).is_some_and(|p| matches(p)))
                .count() as i64;
            assert_eq!(
                row.metric,
                Some(expected),
                "{token}: {} on {}",
                row.profile.uid,
                row.profile.server
            );
        }
        println!("sort={token}: top metric {:?}", entries[0].metric);
    }

    // The filters, through the handler, against counts taken from the tables.
    let (_, has_total) = get(&state, "has=char_002_amiya,char_1001_amiya2&limit=1")
        .await
        .expect("has");
    let has_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true \
         AND (SELECT count(*) FROM user_operators uo WHERE uo.user_id = p.id \
              AND uo.operator_id IN ('char_002_amiya','char_1001_amiya2')) = 2",
    )
    .fetch_one(&pool)
    .await
    .expect("has expected");
    assert_eq!(has_total, has_expected, "has= total");

    let support_id: String = sqlx::query_scalar(
        "SELECT operator_id FROM user_support_units GROUP BY 1 ORDER BY count(*) DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("a support operator");
    let (_, support_total) = get(&state, &format!("support={support_id}&limit=1"))
        .await
        .expect("support");
    let support_expected: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM v_user_profile p WHERE p.public_profile = true \
         AND EXISTS (SELECT 1 FROM user_support_units su WHERE su.user_id = p.id AND su.operator_id = $1)",
    )
    .bind(&support_id)
    .fetch_one(&pool)
    .await
    .expect("support expected");
    assert_eq!(support_total, support_expected, "support= total");
    println!("has=: {has_total} players, support=: {support_total} players");

    // Owns-all, the one filter whose answer depends on the caller's server:
    // every admitted player owns every obtainable operator of the scope that
    // their own server released. Counted independently over every public
    // roster, because checking only the rows that came back would pass an
    // empty page, and an over-wide scope (unobtainable operators left in,
    // say) empties the page rather than dirtying it.
    let rosters = all_rosters(&pool).await;
    let expected_all: Vec<&PublicRoster> = rosters
        .iter()
        .filter(|user| {
            let table = disk.get(&user.server).unwrap_or(en);
            table
                .iter()
                .filter(|(_, p)| p.0 == "MEDIC")
                .all(|(id, _)| user.owned.contains(id))
        })
        .collect();
    let (entries, all_total) = get(&state, "all=class:MEDIC&limit=10")
        .await
        .expect("owns all");
    assert_eq!(
        all_total,
        expected_all.len() as i64,
        "all=class:MEDIC total"
    );
    assert!(
        all_total > 0,
        "nobody owns every medic, so this case proves nothing"
    );
    println!(
        "all=class:MEDIC: {all_total} players, {} on page 1",
        entries.len()
    );
    for row in &entries {
        let server = Server::parse(&row.profile.server).expect("server code");
        let table = disk.get(&server).unwrap_or(en);
        let required: HashSet<&String> = table
            .iter()
            .filter(|(_, p)| p.0 == "MEDIC")
            .map(|(id, _)| id)
            .collect();
        let mine = owned(&pool, &row.profile.uid).await;
        let missing: Vec<&&String> = required.iter().filter(|id| !mine.contains(**id)).collect();
        assert!(
            missing.is_empty(),
            "{} on {} is missing {} medics: {missing:?}",
            row.profile.uid,
            row.profile.server,
            missing.len()
        );
    }

    // Everything at once still answers and still agrees with its own count.
    let (combo, combo_total) = get(
        &state,
        &format!("q=a&sort=masteries&dir=asc&has=char_002_amiya&support={support_id}&all=class:MEDIC&limit=24"),
    )
    .await
    .expect("combination");
    assert!(combo.len() as i64 <= combo_total);
    let mut previous: Option<i64> = None;
    for row in &combo {
        let value = row.metric.expect("a metric under a count sort");
        if let Some(previous) = previous {
            assert!(previous <= value, "asc order broke: {previous} -> {value}");
        }
        previous = Some(value);
        assert!(
            row.profile
                .nickname
                .as_deref()
                .is_some_and(|n| n.to_lowercase().contains('a')),
            "q=a admitted {}",
            row.profile.uid
        );
    }
    println!(
        "everything at once: {combo_total} players, {} on page 1",
        combo.len()
    );

    // And the shapes that must not reach the database at all.
    for bad in [
        "sort=total_score",
        "sort=class:TOKEN",
        "dir=up",
        "support=a%20b",
        "all=warrior",
    ] {
        assert_eq!(
            get(&state, bad).await.expect_err("should be rejected"),
            StatusCode::BAD_REQUEST,
            "`{bad}` should be a 400"
        );
    }
    println!("five malformed requests answered 400");
}
