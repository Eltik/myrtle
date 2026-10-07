//! The profile tab gate, through the real handlers, over the local Postgres
//! roster and the EN game data.
//!
//! One public EN player is borrowed for the run. With their layout NULL every
//! tab endpoint must answer a visitor and the owner exactly as before layouts
//! existed (200), and `/get-user` must carry `profile_layout: null` to both.
//! Then each tab in turn is made private through the real `update-settings`
//! handler: that tab's endpoints answer a visitor 403 and the owner 200, every
//! other tab's endpoints stay 200, and the visitor's `/get-user` layout no
//! longer names the tab. The settings row is put back byte for byte (its
//! `updated_at` included, with the timestamp trigger held off) whether or not
//! an assertion fails, and the test checks it was.
//!
//! The Showcase rides the same row: a valid showcase round-trips, a tabs-only
//! save keeps it, a block naming an unknown entity, a missing grid or tier
//! list, or another player's plan is refused 400 with the row unmoved, a
//! private Showcase or Plans tab withholds it or its plan block from a
//! visitor, and a block whose referent is gone is dropped for a visitor and
//! marked `removed` for the owner.
//!
//! The header background rides it too: a background-only save keeps the
//! tabs and the showcase, a visitor reads it whatever tabs are private, an
//! id no server knows, or a skin id that is no outfit, is refused 400 with
//! the row unmoved, one the data has since dropped is withheld from a
//! visitor, and `null` removes only it.
//!
//! A private Score tab withholds `total_score` and `grade` from a visitor's
//! `/get-user` and search rows and sorts the subject with the unscored; a
//! private Roster refuses `/user-skins` too, but not the `support` filter.
//!
//! Running it applies any pending migration to that database, as the server
//! would on its next start. Ignored by default:
//! `cargo test --test profile_layout_real_data_test -- --ignored --nocapture`

mod common;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::IntoResponse;
use backend::app::cache::store::CacheStore;
use backend::app::extractors::auth::{AuthUser, MaybeAuthUser};
use backend::app::routes;
use backend::app::state::{AppConfig, AppState, ServerData};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::permissions::GlobalRole;
use backend::core::gamedata::assets::AssetIndex;
use backend::core::hypergryph::constants::Server;
use backend::database::models::profile_layout::ProfileTabId;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

/// Every uid-taking endpoint behind a profile tab, with the tab it belongs to.
#[derive(Debug, Clone, Copy)]
enum Endpoint {
    Checkin,
    UserSkins,
    Score,
    ScoreHistory,
    Improvements,
    Roster,
    Plans,
    Inventory,
    Enemies,
    BaseLayout,
    MaxLevelCost,
    Showcase,
}

impl Endpoint {
    const ALL: [Self; 12] = [
        Self::Checkin,
        Self::UserSkins,
        Self::Score,
        Self::ScoreHistory,
        Self::Improvements,
        Self::Roster,
        Self::Plans,
        Self::Inventory,
        Self::Enemies,
        Self::BaseLayout,
        Self::MaxLevelCost,
        Self::Showcase,
    ];

    /// The tabs a visitor needs visible: owned skins are roster data the
    /// Stats tab draws, so `/user-skins` needs both.
    const fn tabs(self) -> &'static [ProfileTabId] {
        match self {
            Self::Checkin => &[ProfileTabId::Stats],
            Self::UserSkins => &[ProfileTabId::Stats, ProfileTabId::Roster],
            Self::Score | Self::ScoreHistory | Self::Improvements => &[ProfileTabId::Score],
            Self::Roster => &[ProfileTabId::Roster],
            Self::Plans => &[ProfileTabId::Plans],
            Self::Inventory => &[ProfileTabId::Inventory],
            Self::Enemies => &[ProfileTabId::Enemies],
            Self::BaseLayout | Self::MaxLevelCost => &[ProfileTabId::Optimizer],
            Self::Showcase => &[ProfileTabId::Showcase],
        }
    }
}

fn query<T: serde::de::DeserializeOwned>(path: &str, uid: &str) -> Query<T> {
    let uri: Uri = format!("http://test{path}?uid={uid}").parse().expect("uri");
    Query::<T>::try_from_uri(&uri).expect("query params")
}

/// One request through the real handler, as the given viewer.
async fn hit(state: &AppState, ep: Endpoint, uid: &str, viewer: Option<&AuthUser>) -> StatusCode {
    let s = State(state.clone());
    let auth = MaybeAuthUser(viewer.cloned());
    let response = match ep {
        Endpoint::Checkin => routes::user::get_user_checkin(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::UserSkins => routes::skins::get_owned_skins(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Score => routes::user::get_user_score(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::ScoreHistory => routes::leaderboard::score_history(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Improvements => {
            routes::improvements::get_user_improvements(s, auth, HeaderMap::new(), query("/", uid))
                .await
                .into_response()
        }
        Endpoint::Roster => routes::roster::get_roster(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Plans => routes::planner::list_public(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Inventory => routes::inventory::get_inventory(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Enemies => routes::enemies::get_encountered_enemies(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::BaseLayout => routes::base::get_layout(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::MaxLevelCost => routes::account::get_max_level_cost(s, auth, query("/", uid))
            .await
            .into_response(),
        Endpoint::Showcase => routes::user::get_user_showcase(s, auth, query("/", uid))
            .await
            .into_response(),
    };
    response.status()
}

/// `/get-user` as the given viewer: its status and, on 200, its body.
async fn get_user(state: &AppState, uid: &str, viewer: Option<&AuthUser>) -> (StatusCode, Value) {
    match routes::user::get_user(
        State(state.clone()),
        MaybeAuthUser(viewer.cloned()),
        query("/get-user", uid),
    )
    .await
    {
        Ok(Json(profile)) => (StatusCode::OK, serde_json::to_value(profile).expect("json")),
        Err(e) => (e.into_response().status(), Value::Null),
    }
}

/// The real EN game data and asset index, and a pool that has run every
/// migration (v033 adds the column these handlers read).
async fn state() -> AppState {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en");
    let server_data = Arc::new(ServerData {
        game_data: arc_swap::ArcSwap::new(common::shared_game_data()),
        asset_index: arc_swap::ArcSwap::from_pointee(AssetIndex::build(&dir)),
        game_data_dir: dir.join("gamedata/excel").display().to_string(),
        assets_dir: dir.display().to_string(),
        art_dir: dir.display().to_string(),
        loaded: std::sync::atomic::AtomicBool::new(true),
        residency: Default::default(),
    });
    let config = AppConfig {
        jwt_secret: "test-jwt-secret-not-used-by-this-test".into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "test-service-key".into(),
        assets_base_dir: dir
            .parent()
            .map_or_else(|| dir.display().to_string(), |p| p.display().to_string()),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
    };
    let db = backend::database::init(DATABASE_URL)
        .await
        .expect("database + migrations");
    AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::from([(Server::EN, server_data)]),
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    )
}

#[derive(Debug, Clone)]
struct Subject {
    id: Uuid,
    uid: String,
    public_profile: bool,
    store_gacha: bool,
    share_stats: bool,
    /// Matches at most five players, so a name search finds the subject on
    /// its first page.
    nickname: String,
    /// An operator the subject owns, for the `has` filter.
    operator: String,
    /// An item the subject holds, for the item leaderboard.
    item: String,
    /// An operator in one of the subject's support slots, for the `support`
    /// filter.
    support: String,
}

/// A public EN player with a roster, items, a sign-in row, a score and a
/// distinctive nickname, and no layout, preferring one who shows plans on
/// their profile so `/plans/public` returns rows rather than an empty list.
async fn pick_subject(db: &PgPool) -> Subject {
    let row: (Uuid, String, bool, bool, bool, String, String, String, String) = sqlx::query_as(
        "SELECT u.id, u.uid, us.public_profile, us.store_gacha, us.share_stats, u.nickname,
                (SELECT o.operator_id FROM user_operators o WHERE o.user_id = u.id
                 ORDER BY o.operator_id LIMIT 1),
                (SELECT i.item_id FROM user_items i WHERE i.user_id = u.id AND i.quantity > 0
                 ORDER BY i.quantity DESC, i.item_id LIMIT 1),
                (SELECT su.operator_id FROM user_support_units su WHERE su.user_id = u.id
                 ORDER BY su.slot LIMIT 1)
         FROM users u
         JOIN servers s ON s.id = u.server_id AND s.code = 'EN'
         JOIN user_settings us ON us.user_id = u.id
         WHERE us.public_profile AND us.profile_layout IS NULL
           AND u.nickname ~ '^[A-Za-z0-9]{4,}$'
           AND (SELECT count(*) FROM users u2 WHERE u2.nickname ILIKE '%' || u.nickname || '%') <= 5
           AND EXISTS (SELECT 1 FROM user_scores sc WHERE sc.user_id = u.id AND sc.total_score IS NOT NULL)
           AND EXISTS (SELECT 1 FROM user_operators o WHERE o.user_id = u.id)
           AND EXISTS (SELECT 1 FROM user_items i WHERE i.user_id = u.id AND i.quantity > 0)
           AND EXISTS (SELECT 1 FROM user_checkin c WHERE c.user_id = u.id)
           AND EXISTS (SELECT 1 FROM user_support_units su WHERE su.user_id = u.id)
         ORDER BY EXISTS (SELECT 1 FROM operator_plans p
                          WHERE p.user_id = u.id AND p.display_on_profile) DESC,
                  u.uid
         LIMIT 1",
    )
    .fetch_one(db)
    .await
    .expect("a public EN player with a roster");
    Subject {
        id: row.0,
        uid: row.1,
        public_profile: row.2,
        store_gacha: row.3,
        share_stats: row.4,
        nickname: row.5,
        operator: row.6,
        item: row.7,
        support: row.8,
    }
}

async fn settings_row(db: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar("SELECT row_to_json(us)::text FROM user_settings us WHERE user_id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .expect("settings row")
}

async fn stored_layout(db: &PgPool, id: Uuid) -> Option<Value> {
    sqlx::query_scalar("SELECT profile_layout FROM user_settings WHERE user_id = $1")
        .bind(id)
        .fetch_one(db)
        .await
        .expect("profile_layout")
}

/// The real `POST /auth/update-settings`, as the owner, with `extra` merged
/// into a body that keeps the three privacy flags as they are.
async fn save(state: &AppState, owner: &AuthUser, subject: &Subject, extra: Value) {
    let mut body = json!({
        "public_profile": subject.public_profile,
        "store_gacha": subject.store_gacha,
        "share_stats": subject.share_stats,
    });
    if let (Some(body), Some(extra)) = (body.as_object_mut(), extra.as_object()) {
        body.extend(extra.clone());
    }
    save_raw(state, owner, body).await;
}

/// The same handler with exactly `body`: a flag left out must stay as stored.
async fn save_raw(state: &AppState, owner: &AuthUser, body: Value) {
    try_save(state, owner, body).await.expect("update-settings");
}

async fn flags(db: &PgPool, id: Uuid) -> (bool, bool, bool) {
    sqlx::query_as(
        "SELECT public_profile, store_gacha, share_stats FROM user_settings WHERE user_id = $1",
    )
    .bind(id)
    .fetch_one(db)
    .await
    .expect("flags")
}

fn uri_query<T: serde::de::DeserializeOwned>(query: &str) -> Query<T> {
    let uri: Uri = format!("http://test/?{query}").parse().expect("uri");
    Query::<T>::try_from_uri(&uri).expect("query params")
}

/// Every place that lists or ranks players, as one row of booleans: is the
/// subject in it. `standing` and `item standing` are statuses, visitor/owner.
#[derive(Debug, PartialEq, Eq)]
struct Listings {
    name_search: bool,
    has_filter: bool,
    support_filter: bool,
    masteries_sort: bool,
    operators_sort: bool,
    skins_sort: bool,
    enemies_sort: bool,
    score_board: bool,
    item_board: bool,
    standing: (u16, u16),
    item_standing: (u16, u16),
}

async fn listings(state: &AppState, subject: &Subject, owner: &AuthUser) -> Listings {
    let nick = subject.nickname.as_str();
    let uid = subject.uid.as_str();
    let search = |q: String| {
        let state = state.clone();
        async move {
            let Json(page) = routes::search::search(State(state), uri_query(&q))
                .await
                .expect("search");
            page.entries
                .into_iter()
                .map(|e| e.profile.uid)
                .collect::<Vec<_>>()
        }
    };
    let has = |v: Vec<String>| v.iter().any(|u| u == uid);
    let Json(board) =
        routes::leaderboard::leaderboard(State(state.clone()), uri_query(&format!("q={uid}")))
            .await
            .expect("leaderboard");
    let Json(items) = routes::item_leaderboard::item_leaderboard(
        State(state.clone()),
        uri_query(&format!("item={}&q={uid}", subject.item)),
    )
    .await
    .expect("item leaderboard");
    let standing = |viewer: Option<&AuthUser>| {
        let state = state.clone();
        let auth = MaybeAuthUser(viewer.cloned());
        async move {
            routes::leaderboard::standing(
                State(state),
                auth,
                uri_query(&format!("uid={uid}&server=EN")),
            )
            .await
            .into_response()
            .status()
            .as_u16()
        }
    };
    let item_standing = |viewer: Option<&AuthUser>| {
        let state = state.clone();
        let auth = MaybeAuthUser(viewer.cloned());
        let item = subject.item.clone();
        async move {
            routes::item_leaderboard::item_standing(
                State(state),
                auth,
                uri_query(&format!("item={item}&uid={uid}&server=EN")),
            )
            .await
            .into_response()
            .status()
            .as_u16()
        }
    };
    Listings {
        name_search: has(search(format!("q={nick}&limit=100")).await),
        has_filter: has(search(format!("q={nick}&has={}&limit=100", subject.operator)).await),
        support_filter: has(
            search(format!("q={nick}&support={}&limit=100", subject.support)).await,
        ),
        masteries_sort: has(search(format!("q={nick}&sort=masteries&limit=100")).await),
        operators_sort: has(search(format!("q={nick}&sort=operators&limit=100")).await),
        skins_sort: has(search(format!("q={nick}&sort=skins&limit=100")).await),
        enemies_sort: has(search(format!("q={nick}&sort=enemies&limit=100")).await),
        score_board: board.entries.iter().any(|e| e.uid == uid),
        item_board: items.entries.iter().any(|e| e.uid == uid),
        standing: (standing(None).await, standing(Some(owner)).await),
        item_standing: (item_standing(None).await, item_standing(Some(owner)).await),
    }
}

/// The real `update-settings` with exactly `body`, its refusal as a status
/// and message instead of a panic.
async fn try_save(
    state: &AppState,
    owner: &AuthUser,
    body: Value,
) -> Result<(), (StatusCode, String)> {
    let request = serde_json::from_value(body).expect("UpdateSettingsRequest");
    match routes::auth::update_settings(State(state.clone()), owner.clone(), Json(request)).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let message = format!("{e:?}");
            Err((e.into_response().status(), message))
        }
    }
}

/// [`try_save`] of a background-only layout patch.
async fn try_save_background(
    state: &AppState,
    owner: &AuthUser,
    background: &Value,
) -> Result<(), (StatusCode, String)> {
    try_save(
        state,
        owner,
        json!({ "profile_layout": { "background": background } }),
    )
    .await
}

/// Write `background` straight into the stored layout, past the save check,
/// as a later game-data change would leave it.
async fn set_stored_background(db: &PgPool, id: Uuid, background: Value) {
    sqlx::query(
        "UPDATE user_settings SET profile_layout = jsonb_set(profile_layout, '{background}', $2::jsonb)
         WHERE user_id = $1",
    )
    .bind(id)
    .bind(background)
    .execute(db)
    .await
    .expect("dangling background");
}

/// Every tab, in canonical order, visible unless it is `hidden`.
fn tabs_without(hidden: Option<ProfileTabId>) -> Vec<Value> {
    ProfileTabId::ALL
        .iter()
        .map(|t| json!({ "id": t.as_str(), "visible": Some(*t) != hidden }))
        .collect()
}

/// `/get-user-showcase` as the given viewer: its status and, on 200, its body.
async fn showcase(state: &AppState, uid: &str, viewer: Option<&AuthUser>) -> (StatusCode, Value) {
    match routes::user::get_user_showcase(
        State(state.clone()),
        MaybeAuthUser(viewer.cloned()),
        query("/get-user-showcase", uid),
    )
    .await
    {
        Ok(Json(view)) => (StatusCode::OK, serde_json::to_value(view).expect("json")),
        Err(e) => (e.into_response().status(), Value::Null),
    }
}

/// How many showcase blocks a `/get-user` body carries.
fn block_count(profile: &Value) -> usize {
    profile["profile_layout"]["showcase"]["blocks"]
        .as_array()
        .map_or(0, Vec::len)
}

/// Real referents for the showcase checks.
#[derive(Debug)]
struct Fixtures {
    grid: String,
    tier_list: String,
    /// One of the subject's plans shown on their profile, when they have one.
    own_plan: Option<Uuid>,
    /// A plan of some other player, when there is one.
    other_plan: Option<Uuid>,
}

async fn fixtures(db: &PgPool, subject: Uuid) -> Fixtures {
    let grid: String = sqlx::query_scalar("SELECT slug FROM grids ORDER BY slug LIMIT 1")
        .fetch_one(db)
        .await
        .expect("a grid");
    let tier_list: String =
        sqlx::query_scalar("SELECT slug FROM tier_lists WHERE is_active ORDER BY slug LIMIT 1")
            .fetch_one(db)
            .await
            .expect("an active tier list");
    let own_plan = sqlx::query_scalar(
        "SELECT id FROM operator_plans WHERE user_id = $1 AND display_on_profile ORDER BY id LIMIT 1",
    )
    .bind(subject)
    .fetch_optional(db)
    .await
    .expect("own plan");
    let other_plan =
        sqlx::query_scalar("SELECT id FROM operator_plans WHERE user_id <> $1 ORDER BY id LIMIT 1")
            .bind(subject)
            .fetch_optional(db)
            .await
            .expect("other plan");
    Fixtures {
        grid,
        tier_list,
        own_plan,
        other_plan,
    }
}

fn layout_ids(profile: &Value) -> Vec<(String, bool)> {
    profile["profile_layout"]["tabs"]
        .as_array()
        .expect("tabs")
        .iter()
        .map(|t| {
            (
                t["id"].as_str().expect("id").to_owned(),
                t["visible"].as_bool().expect("visible"),
            )
        })
        .collect()
}

async fn run(state: AppState, subject: Subject) {
    let owner = AuthUser {
        user_id: subject.id.to_string(),
        uid: subject.uid.clone(),
        server: "EN".into(),
        role: GlobalRole::User,
    };
    let mut failures: Vec<String> = Vec::new();
    check_tab_gates(&state, &subject, &owner, &mut failures).await;
    check_listings(&state, &subject, &owner, &mut failures).await;
    check_showcase(&state, &subject, &owner, &mut failures).await;
    check_background(&state, &subject, &owner, &mut failures).await;
    check_search_row_and_private_profile(&state, &subject, &owner, &mut failures).await;
    assert!(failures.is_empty(), "{failures:#?}");
}

/// With no layout every tab endpoint answers as before; each tab made
/// private refuses only its own endpoints to a visitor, and a layout-only
/// save keeps the three privacy flags.
async fn check_tab_gates(
    state: &AppState,
    subject: &Subject,
    owner: &AuthUser,
    failures: &mut Vec<String>,
) {
    let uid = subject.uid.as_str();
    // Kill switch: NULL layout, everything as before.
    let (vs, visitor_profile) = get_user(state, uid, None).await;
    let (os, owner_profile) = get_user(state, uid, Some(owner)).await;
    println!("NULL layout: /get-user visitor {vs}, owner {os}");
    assert_eq!((vs, os), (StatusCode::OK, StatusCode::OK));
    assert_eq!(visitor_profile["profile_layout"], Value::Null);
    assert_eq!(
        visitor_profile, owner_profile,
        "with no layout a visitor and the owner read the same profile"
    );
    for ep in Endpoint::ALL {
        let v = hit(state, ep, uid, None).await;
        let o = hit(state, ep, uid, Some(owner)).await;
        println!("NULL layout: {ep:?} visitor {v}, owner {o}");
        if (v, o) != (StatusCode::OK, StatusCode::OK) {
            failures.push(format!("NULL layout: {ep:?} visitor {v}, owner {o}"));
        }
    }

    // Write path: absent leaves the column alone, a messy layout is stored
    // normalized, explicit null clears it.
    save(state, owner, subject, json!({})).await;
    assert_eq!(
        stored_layout(&state.db, subject.id).await,
        None,
        "absent wrote"
    );
    save(
        state,
        owner,
        subject,
        json!({ "profile_layout": { "tabs": [
            { "id": "inventory", "visible": false },
            { "id": "bogus", "visible": false },
            { "id": "inventory", "visible": true },
        ] } }),
    )
    .await;
    let stored = stored_layout(&state.db, subject.id).await.expect("stored");
    let stored_ids: Vec<&str> = stored["tabs"]
        .as_array()
        .expect("tabs")
        .iter()
        .map(|t| t["id"].as_str().expect("id"))
        .collect();
    assert_eq!(
        stored_ids,
        [
            "showcase",
            "inventory",
            "stats",
            "score",
            "roster",
            "plans",
            "enemies",
            "optimizer"
        ]
    );
    assert_eq!(stored["tabs"][1]["visible"], json!(false));
    save(state, owner, subject, json!({})).await;
    assert!(
        stored_layout(&state.db, subject.id).await.is_some(),
        "absent cleared"
    );
    save(state, owner, subject, json!({ "profile_layout": null })).await;
    assert_eq!(
        stored_layout(&state.db, subject.id).await,
        None,
        "null kept"
    );

    // An explicit all-visible layout: every endpoint still 200 to a visitor.
    save(
        state,
        owner,
        subject,
        json!({ "profile_layout": { "tabs": tabs_without(None) } }),
    )
    .await;
    for ep in Endpoint::ALL {
        let v = hit(state, ep, uid, None).await;
        if v != StatusCode::OK {
            failures.push(format!("all public: {ep:?} visitor {v}"));
        }
    }

    // Each tab private in turn.
    for private in ProfileTabId::ALL {
        // Only the layout: the three flags must survive being left out.
        save_raw(
            state,
            owner,
            json!({ "profile_layout": { "tabs": tabs_without(Some(private)) } }),
        )
        .await;

        let (_, visitor_profile) = get_user(state, uid, None).await;
        let (_, owner_profile) = get_user(state, uid, Some(owner)).await;
        let seen = layout_ids(&visitor_profile);
        if seen.iter().any(|(id, _)| id == private.as_str())
            || seen.len() != ProfileTabId::ALL.len() - 1
        {
            failures.push(format!("{private:?} private: visitor layout {seen:?}"));
        }
        let owned = layout_ids(&owner_profile);
        if !owned.contains(&(private.as_str().to_owned(), false))
            || owned.len() != ProfileTabId::ALL.len()
        {
            failures.push(format!("{private:?} private: owner layout {owned:?}"));
        }
        // `total_score` and `grade` are the Score tab's headline: withheld from
        // a visitor exactly while that tab is private, kept for the owner.
        let score_fields = |p: &Value| (p["total_score"].clone(), p["grade"].clone());
        let (visitor_score, owner_score) =
            (score_fields(&visitor_profile), score_fields(&owner_profile));
        println!(
            "{private:?} private: score/grade visitor {visitor_score:?}, owner {owner_score:?}"
        );
        let visitor_hidden = visitor_score == (Value::Null, Value::Null);
        if visitor_hidden != (private == ProfileTabId::Score)
            || owner_score.0.is_null()
            || owner_score.1.is_null()
        {
            failures.push(format!(
                "{private:?} private: score/grade visitor {visitor_score:?}, owner {owner_score:?}"
            ));
        }

        for ep in Endpoint::ALL {
            let v = hit(state, ep, uid, None).await;
            let o = hit(state, ep, uid, Some(owner)).await;
            let want = if ep.tabs().contains(&private) {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::OK
            };
            println!("{private:?} private: {ep:?} visitor {v}, owner {o}");
            if v != want || o != StatusCode::OK {
                failures.push(format!(
                    "{private:?} private: {ep:?} visitor {v} (want {want}), owner {o}"
                ));
            }
        }
    }

    let kept = flags(&state.db, subject.id).await;
    println!("flags after layout-only saves: {kept:?}");
    if kept
        != (
            subject.public_profile,
            subject.store_gacha,
            subject.share_stats,
        )
    {
        failures.push(format!("a layout-only save changed the flags: {kept:?}"));
    }
}

/// Search and the leaderboards drop a player from exactly the listings
/// computed from a tab they made private.
async fn check_listings(
    state: &AppState,
    subject: &Subject,
    owner: &AuthUser,
    failures: &mut Vec<String>,
) {
    let uid = subject.uid.as_str();
    // Search and the leaderboards: with no layout the subject is in every
    // listing; a private tab drops them from exactly the listings computed
    // from that tab's data. Each save flushes the search and leaderboard
    // caches, so every probe reads fresh.
    save_raw(state, owner, json!({ "profile_layout": null })).await;
    let base = listings(state, subject, owner).await;
    println!("NULL layout listings: {base:?}");
    let everywhere = Listings {
        name_search: true,
        has_filter: true,
        support_filter: true,
        masteries_sort: true,
        operators_sort: true,
        skins_sort: true,
        enemies_sort: true,
        score_board: true,
        item_board: true,
        standing: (200, 200),
        item_standing: (200, 200),
    };
    if base != everywhere {
        failures.push(format!("NULL layout listings: {base:?}"));
    }
    for private in ProfileTabId::ALL {
        save_raw(
            state,
            owner,
            json!({ "profile_layout": { "tabs": tabs_without(Some(private)) } }),
        )
        .await;
        let got = listings(state, subject, owner).await;
        let mut want = Listings { ..everywhere };
        match private {
            // Not `support_filter`: support units are public in game, so a
            // private roster still matches it.
            ProfileTabId::Roster => {
                want.has_filter = false;
                want.masteries_sort = false;
                want.operators_sort = false;
                want.skins_sort = false;
            }
            ProfileTabId::Enemies => want.enemies_sort = false,
            // Off the board: a visitor is refused, and the owner has no rank.
            ProfileTabId::Score => {
                want.score_board = false;
                want.standing = (403, 404);
            }
            ProfileTabId::Inventory => {
                want.item_board = false;
                want.item_standing = (403, 404);
            }
            ProfileTabId::Showcase
            | ProfileTabId::Stats
            | ProfileTabId::Plans
            | ProfileTabId::Optimizer => {}
        }
        println!("{private:?} private listings: {got:?}");
        if got != want {
            failures.push(format!(
                "{private:?} private listings: {got:?}, want {want:?}"
            ));
        }
    }

    // The score order with the Score tab private. Score is also the default
    // order of every name search, so the subject stays findable; the row
    // carries no score or grade and sorts after every scored row, in either
    // direction. The query is the shortest prefix of the nickname matching
    // at most 100 players, so the one page holds every match and the
    // subject's place among scored players is visible.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": tabs_without(Some(ProfileTabId::Score)) } }),
    )
    .await;
    let score_page = |q: String| {
        let state = state.clone();
        async move {
            let Json(page) = routes::search::search(State(state), uri_query(&q))
                .await
                .expect("search");
            page
        }
    };
    let mut prefix = subject.nickname.clone();
    for len in 2..=subject.nickname.len() {
        let candidate = subject.nickname[..len].to_owned();
        if score_page(format!("q={candidate}&limit=100")).await.total <= 100 {
            prefix = candidate;
            break;
        }
    }
    for dir in ["desc", "asc"] {
        let page = score_page(format!("q={prefix}&sort=score&dir={dir}&limit=100")).await;
        let at = page.entries.iter().position(|e| e.profile.uid == uid);
        let scored: Vec<usize> = page
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.profile.total_score.is_some())
            .map(|(i, _)| i)
            .collect();
        let row = at.map(|i| &page.entries[i].profile);
        println!(
            "Score private, sort=score {dir}, q={prefix}: {} rows, {} scored, subject at {at:?} with score {:?} grade {:?}",
            page.entries.len(),
            scored.len(),
            row.and_then(|p| p.total_score),
            row.and_then(|p| p.grade.clone())
        );
        let ok = at.is_some_and(|i| {
            row.is_some_and(|p| p.total_score.is_none() && p.grade.is_none())
                && scored.iter().all(|&s| s < i)
        });
        if !ok || scored.is_empty() {
            failures.push(format!(
                "Score private, sort=score {dir}: subject at {at:?}, scored rows at {scored:?}"
            ));
        }
    }
}

/// The Showcase: round-trips, partial saves, refused writes, tab gating and
/// dangling referents.
async fn check_showcase(
    state: &AppState,
    subject: &Subject,
    owner: &AuthUser,
    failures: &mut Vec<String>,
) {
    let uid = subject.uid.as_str();
    // The Showcase. Kill switch: with no layout both readers get an empty
    // showcase and `/get-user` carries no layout at all.
    save_raw(state, owner, json!({ "profile_layout": null })).await;
    let (vs, visitor_view) = showcase(state, uid, None).await;
    let (os, owner_view) = showcase(state, uid, Some(owner)).await;
    println!(
        "NULL layout: showcase visitor {vs} {}, owner {os} {}",
        visitor_view["blocks"], owner_view["blocks"]
    );
    if (vs, os) != (StatusCode::OK, StatusCode::OK)
        || visitor_view["blocks"] != json!([])
        || owner_view["blocks"] != json!([])
    {
        failures.push(format!(
            "NULL layout showcase: {visitor_view} / {owner_view}"
        ));
    }

    let fx = fixtures(&state.db, subject.id).await;
    println!("showcase fixtures: {fx:?}");
    let all_visible_tabs = tabs_without(None);
    // Two distinct operators: a repeat would be deduplicated away.
    let second = if subject.operator == "char_002_amiya" {
        "char_003_kalts"
    } else {
        "char_002_amiya"
    };
    let mut blocks = vec![
        json!({ "type": "favourites", "entity_kind": "operator", "ids": [subject.operator, second], "title": "Mains" }),
        json!({ "type": "favourites", "entity_kind": "main_story", "ids": ["main_0"] }),
        json!({ "type": "grid", "slug": fx.grid }),
        json!({ "type": "tier_list", "slug": fx.tier_list }),
    ];
    if let Some(plan) = fx.own_plan {
        blocks.push(json!({ "type": "plan", "id": plan.to_string() }));
    }
    let n = blocks.len();
    let sent = json!({ "blocks": blocks });
    try_save(
        state,
        owner,
        json!({ "profile_layout": { "tabs": all_visible_tabs, "showcase": sent } }),
    )
    .await
    .expect("a valid showcase saves");
    let stored = stored_layout(&state.db, subject.id).await.expect("stored");
    if stored["showcase"] != sent {
        failures.push(format!(
            "showcase did not round-trip: {}",
            stored["showcase"]
        ));
    }

    let (_, visitor_profile) = get_user(state, uid, None).await;
    let (vs, visitor_view) = showcase(state, uid, None).await;
    let (os, owner_view) = showcase(state, uid, Some(owner)).await;
    let resolved = |view: &Value| {
        view["blocks"].as_array().map_or(0, |b| {
            b.iter().filter(|b| b["removed"] == json!(false)).count()
        })
    };
    println!(
        "valid showcase: get-user visitor blocks {}, endpoint visitor {vs} {} resolved, owner {os} {} resolved",
        block_count(&visitor_profile),
        resolved(&visitor_view),
        resolved(&owner_view)
    );
    if block_count(&visitor_profile) != n
        || (vs, os) != (StatusCode::OK, StatusCode::OK)
        || resolved(&visitor_view) != n
        || resolved(&owner_view) != n
    {
        failures.push(format!(
            "valid showcase read: {visitor_view} / {owner_view}"
        ));
    }
    let favourites = &visitor_view["blocks"][0]["entities"];
    if favourites.as_array().map(Vec::len) != Some(2)
        || favourites
            .as_array()
            .is_some_and(|e| e.iter().any(|e| e["entity"].is_null()))
    {
        failures.push(format!("favourites did not resolve: {favourites}"));
    }

    // A tabs-only save, the shape the tab editor and older clients send,
    // keeps the stored showcase.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "roster", "visible": true }] } }),
    )
    .await;
    let kept = stored_layout(&state.db, subject.id).await.expect("stored");
    // Roster leads what was sent; the Showcase tab, left out, takes its
    // canonical place ahead of it.
    println!(
        "tabs-only save: tabs {} then {}, showcase kept {}",
        kept["tabs"][0]["id"],
        kept["tabs"][1]["id"],
        kept["showcase"] == sent
    );
    if kept["showcase"] != sent || kept["tabs"][1]["id"] != json!("roster") {
        failures.push(format!("a tabs-only save lost the showcase: {kept}"));
    }

    // A showcase-only save, the shape the showcase editor sends (an empty
    // showcase as `{"blocks": []}`), keeps the stored tabs; putting the
    // blocks back the same way keeps them again.
    for blocks in [json!({ "blocks": [] }), sent.clone()] {
        save_raw(
            state,
            owner,
            json!({ "profile_layout": { "showcase": blocks } }),
        )
        .await;
        let after = stored_layout(&state.db, subject.id).await.expect("stored");
        println!(
            "showcase-only save of {} blocks: showcase stored {}, tabs kept {}",
            blocks["blocks"].as_array().map_or(0, Vec::len),
            after["showcase"] == blocks,
            after["tabs"] == kept["tabs"]
        );
        if after["showcase"] != blocks || after["tabs"] != kept["tabs"] {
            failures.push(format!(
                "a showcase-only save moved the tabs: {kept} -> {after}"
            ));
        }
    }

    // Refused writes: 400, and the stored row does not move.
    let before_bad = settings_row(&state.db, subject.id).await;
    let mut bad: Vec<(&str, Value)> = vec![
        (
            "unknown entity id",
            json!({ "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya", "char_999_nobody"] }),
        ),
        (
            "nonexistent grid",
            json!({ "type": "grid", "slug": "no-such-grid-zz9" }),
        ),
        (
            "nonexistent tier list",
            json!({ "type": "tier_list", "slug": "no-such-tier-list-zz9" }),
        ),
    ];
    if let Some(plan) = fx.other_plan {
        bad.push((
            "someone else's plan",
            json!({ "type": "plan", "id": plan.to_string() }),
        ));
    }
    for (what, block) in bad {
        let got = try_save(
            state,
            owner,
            json!({ "profile_layout": { "tabs": [], "showcase": { "blocks": [block] } } }),
        )
        .await;
        println!("{what}: {got:?}");
        match got {
            Err((StatusCode::BAD_REQUEST, _)) => {}
            other => failures.push(format!("{what}: {other:?}, want 400")),
        }
    }
    if settings_row(&state.db, subject.id).await != before_bad {
        failures.push("a refused showcase save changed the row".into());
    }

    // Plans private: a visitor loses the plan block, the owner keeps it.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": tabs_without(Some(ProfileTabId::Plans)) } }),
    )
    .await;
    let plan_blocks = usize::from(fx.own_plan.is_some());
    let (_, visitor_profile) = get_user(state, uid, None).await;
    let (_, visitor_view) = showcase(state, uid, None).await;
    let (_, owner_view) = showcase(state, uid, Some(owner)).await;
    println!(
        "Plans private: visitor blocks {} / {}, owner {}",
        block_count(&visitor_profile),
        resolved(&visitor_view),
        resolved(&owner_view)
    );
    if block_count(&visitor_profile) != n - plan_blocks
        || resolved(&visitor_view) != n - plan_blocks
        || resolved(&owner_view) != n
    {
        failures.push("Plans private: the plan block reached a visitor".into());
    }

    // Showcase private: no showcase for a visitor, from either endpoint.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": tabs_without(Some(ProfileTabId::Showcase)) } }),
    )
    .await;
    let (_, visitor_profile) = get_user(state, uid, None).await;
    let (vs, _) = showcase(state, uid, None).await;
    let (os, owner_view) = showcase(state, uid, Some(owner)).await;
    println!(
        "Showcase private: get-user visitor showcase {}, endpoint visitor {vs}, owner {os} {} blocks",
        visitor_profile["profile_layout"].get("showcase").is_some(),
        resolved(&owner_view)
    );
    if visitor_profile["profile_layout"].get("showcase").is_some()
        || vs != StatusCode::FORBIDDEN
        || os != StatusCode::OK
        || resolved(&owner_view) != n
    {
        failures.push("Showcase private: a visitor still saw it".into());
    }

    // Dangling referents, written past the save check as a later deletion
    // would leave them: a visitor never sees them, the owner sees `removed`.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": all_visible_tabs } }),
    )
    .await;
    sqlx::query(
        "UPDATE user_settings SET profile_layout = jsonb_set(profile_layout, '{showcase,blocks}',
             (profile_layout -> 'showcase' -> 'blocks') || $2::jsonb) WHERE user_id = $1",
    )
    .bind(subject.id)
    .bind(json!([
        { "type": "grid", "slug": "gone-grid-zz9" },
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_999_nobody"] },
        { "type": "favourites", "entity_kind": "operator", "ids": [subject.operator, "char_999_nobody"] },
    ]))
    .execute(&state.db)
    .await
    .expect("dangling blocks");
    // An empty save flushes the cached profile row and changes nothing else.
    save_raw(state, owner, json!({})).await;
    let (_, visitor_profile) = get_user(state, uid, None).await;
    let (_, visitor_view) = showcase(state, uid, None).await;
    let (_, owner_view) = showcase(state, uid, Some(owner)).await;
    let owner_blocks = owner_view["blocks"].as_array().cloned().unwrap_or_default();
    let removed: Vec<bool> = owner_blocks
        .iter()
        .map(|b| b["removed"] == json!(true))
        .collect();
    let mixed_for_visitor = visitor_view["blocks"][n]["block"]["ids"].clone();
    println!(
        "dangling: owner removed flags {removed:?}, visitor blocks {} / {}, mixed block ids for visitor {mixed_for_visitor}",
        block_count(&visitor_profile),
        visitor_view["blocks"].as_array().map_or(0, Vec::len)
    );
    let mut want_removed = vec![false; n];
    want_removed.extend([true, true, false]);
    if removed != want_removed
        || block_count(&visitor_profile) != n + 1
        || visitor_view["blocks"].as_array().map(Vec::len) != Some(n + 1)
        || mixed_for_visitor != json!([subject.operator])
        || owner_blocks[n + 2]["entities"][1]["entity"] != Value::Null
    {
        failures.push(format!(
            "dangling: owner {owner_view}, visitor {visitor_view}"
        ));
    }
}

/// The header background: partial saves, visitor reads, every kind, zoom,
/// refused ids, dangling ids and clearing.
async fn check_background(
    state: &AppState,
    subject: &Subject,
    owner: &AuthUser,
    failures: &mut Vec<String>,
) {
    let uid = subject.uid.as_str();
    // The header background. A background-only save, the shape the picker
    // sends, sets that key and keeps the stored tabs and showcase.
    let before_bg = stored_layout(&state.db, subject.id).await.expect("stored");
    let skin_bg =
        json!({ "kind": "skin", "id": "char_002_amiya@epoque#4", "focus_x": 40, "focus_y": 18 });
    try_save_background(state, owner, &skin_bg)
        .await
        .expect("a valid background saves");
    let after_bg = stored_layout(&state.db, subject.id).await.expect("stored");
    println!(
        "background save: stored {}, tabs kept {}, showcase kept {}",
        after_bg["background"],
        after_bg["tabs"] == before_bg["tabs"],
        after_bg["showcase"] == before_bg["showcase"]
    );
    if after_bg["background"] != skin_bg
        || after_bg["tabs"] != before_bg["tabs"]
        || after_bg["showcase"] != before_bg["showcase"]
    {
        failures.push(format!("background-only save: {before_bg} -> {after_bg}"));
    }
    // A visitor reads it on a public profile, with tabs private or not.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": tabs_without(Some(ProfileTabId::Showcase)) } }),
    )
    .await;
    let (_, visitor_profile) = get_user(state, uid, None).await;
    let (_, owner_profile) = get_user(state, uid, Some(owner)).await;
    let kept_bg = stored_layout(&state.db, subject.id).await.expect("stored");
    println!(
        "background read: visitor {}, owner {}, kept by a tabs-only save {}",
        visitor_profile["profile_layout"]["background"],
        owner_profile["profile_layout"]["background"],
        kept_bg["background"] == skin_bg
    );
    if visitor_profile["profile_layout"]["background"] != skin_bg
        || owner_profile["profile_layout"]["background"] != skin_bg
        || kept_bg["background"] != skin_bg
        || kept_bg["showcase"] != before_bg["showcase"]
    {
        failures.push(format!(
            "background read: visitor {visitor_profile}, stored {kept_bg}"
        ));
    }
    // An operator background, the subject's own operator.
    let op_bg = json!({ "kind": "operator", "id": subject.operator });
    try_save_background(state, owner, &op_bg)
        .await
        .expect("an operator background saves");
    let stored_op = stored_layout(&state.db, subject.id).await.expect("stored");
    if stored_op["background"] != op_bg {
        failures.push(format!("operator background: {stored_op}"));
    }
    // A gallery picture, by its table key, both focus axes kept; a visitor
    // reads it as stored.
    let pic_bg =
        json!({ "kind": "archive_pic", "id": "pic_rogue_1_KV1", "focus_x": 70, "focus_y": 35 });
    try_save_background(state, owner, &pic_bg)
        .await
        .expect("a gallery background saves");
    let stored_pic = stored_layout(&state.db, subject.id).await.expect("stored");
    let (_, visitor_pic) = get_user(state, uid, None).await;
    println!(
        "gallery background: stored {}, visitor {}",
        stored_pic["background"], visitor_pic["profile_layout"]["background"]
    );
    if stored_pic["background"] != pic_bg || visitor_pic["profile_layout"]["background"] != pic_bg {
        failures.push(format!("gallery background: {stored_pic}"));
    }
    // A story CG and a story scene, by the catalogue's own first id: each
    // round-trips, focus and zoom kept; the same id in the other kind, an
    // unknown id and an upper-cased one are 400 and the row stays
    // byte-for-byte; one the data has since dropped never reaches a visitor.
    for (kind, wire, other) in [
        (
            backend::app::services::story::StoryArtKind::Cg,
            "story_cg",
            "story_scene",
        ),
        (
            backend::app::services::story::StoryArtKind::Scene,
            "story_scene",
            "story_cg",
        ),
    ] {
        let Json(art) = routes::story::art_gallery(State(state.clone()), axum::extract::Path(kind))
            .await
            .expect("art gallery");
        let id = art.groups[0].pictures[0].id.clone();
        let art_bg = json!({ "kind": wire, "id": id, "focus_x": 40, "focus_y": 60, "scale": 150 });
        try_save_background(state, owner, &art_bg)
            .await
            .expect("a story art background saves");
        let stored = stored_layout(&state.db, subject.id).await.expect("stored");
        let (_, visitor) = get_user(state, uid, None).await;
        println!(
            "{wire} background: stored {}, visitor {}",
            stored["background"], visitor["profile_layout"]["background"]
        );
        if stored["background"] != art_bg || visitor["profile_layout"]["background"] != art_bg {
            failures.push(format!("{wire} background: {stored}"));
        }
        let before = settings_row(&state.db, subject.id).await;
        for (what, bg) in [
            ("unknown", json!({ "kind": wire, "id": "no_such_art_99" })),
            (
                "upper-cased",
                json!({ "kind": wire, "id": id.to_ascii_uppercase() }),
            ),
            (
                "an archive pic id",
                json!({ "kind": wire, "id": "pic_rogue_1_KV1" }),
            ),
        ] {
            let got = try_save_background(state, owner, &bg).await;
            println!("{wire} background {what}: {got:?}");
            match got {
                Err((StatusCode::BAD_REQUEST, _)) => {}
                other => failures.push(format!("{wire} background {what}: {other:?}, want 400")),
            }
        }
        // The other kind's catalogue does not list it (a CG key is no scene).
        let crossed = try_save_background(state, owner, &json!({ "kind": other, "id": id })).await;
        println!("{wire} id as {other}: {crossed:?}");
        if !matches!(crossed, Err((StatusCode::BAD_REQUEST, _))) {
            failures.push(format!("{wire} id as {other}: {crossed:?}, want 400"));
        }
        if settings_row(&state.db, subject.id).await != before {
            failures.push(format!("a refused {wire} save changed the row"));
        }
        set_stored_background(
            &state.db,
            subject.id,
            json!({ "kind": wire, "id": "no_such_art_99" }),
        )
        .await;
        save_raw(state, owner, json!({})).await;
        let (_, visitor) = get_user(state, uid, None).await;
        let (_, owner_view) = get_user(state, uid, Some(owner)).await;
        println!(
            "dangling {wire}: visitor {}, owner {}",
            visitor["profile_layout"].get("background").is_some(),
            owner_view["profile_layout"]["background"]
        );
        if visitor["profile_layout"].get("background").is_some()
            || owner_view["profile_layout"]["background"]["id"] != json!("no_such_art_99")
        {
            failures.push(format!("dangling {wire} background reached a visitor"));
        }
    }
    // Back to the gallery picture the checks below start from.
    try_save_background(state, owner, &pic_bg)
        .await
        .expect("the gallery background saves again");
    // Zoom: a scale is stored as sent inside 100..=300 and a visitor reads
    // it; one past either end is clamped, not refused.
    for (sent, want) in [(180, 180), (999, 300), (20, 100)] {
        let zoom_bg = json!({ "kind": "archive_pic", "id": "pic_rogue_1_KV1", "focus_x": 70, "focus_y": 35, "scale": sent });
        try_save_background(state, owner, &zoom_bg)
            .await
            .expect("a zoomed background saves");
        let stored_zoom = stored_layout(&state.db, subject.id).await.expect("stored");
        let (_, visitor_zoom) = get_user(state, uid, None).await;
        println!(
            "zoomed background sent scale {sent}: stored {}, visitor {}",
            stored_zoom["background"], visitor_zoom["profile_layout"]["background"]
        );
        let mut want_bg = zoom_bg.clone();
        want_bg["scale"] = json!(want);
        if stored_zoom["background"] != want_bg
            || visitor_zoom["profile_layout"]["background"] != want_bg
        {
            failures.push(format!("zoomed background {sent}: {stored_zoom}"));
        }
    }
    // Refused: an id no server knows, or one of the other kind. The row
    // does not move.
    let before_bad_bg = settings_row(&state.db, subject.id).await;
    for (what, bg) in [
        (
            "unknown operator",
            json!({ "kind": "operator", "id": "char_999_nobody" }),
        ),
        (
            "unknown skin",
            json!({ "kind": "skin", "id": "char_002_amiya@nothing#99" }),
        ),
        (
            "an operator id as a skin",
            json!({ "kind": "skin", "id": subject.operator }),
        ),
        (
            "unknown gallery picture",
            json!({ "kind": "archive_pic", "id": "act99side_pic_0" }),
        ),
        // Ids are the table's keys exactly: the lowercased spelling is none.
        (
            "a gallery id in the wrong case",
            json!({ "kind": "archive_pic", "id": "pic_rogue_1_kv1" }),
        ),
        (
            "an operator id as a gallery picture",
            json!({ "kind": "archive_pic", "id": subject.operator }),
        ),
        // Known to `skin_table`, so only the outfit rule refuses it: the
        // default art, which the client draws no skin art for.
        (
            "a default art id as a skin",
            json!({ "kind": "skin", "id": format!("{}#1", subject.operator) }),
        ),
    ] {
        let got = try_save_background(state, owner, &bg).await;
        println!("background {what}: {got:?}");
        match got {
            Err((StatusCode::BAD_REQUEST, _)) => {}
            other => failures.push(format!("background {what}: {other:?}, want 400")),
        }
    }
    if settings_row(&state.db, subject.id).await != before_bad_bg {
        failures.push("a refused background save changed the row".into());
    }
    // A background whose id the data has since dropped, written past the
    // check: a visitor's header falls back, the owner still reads the key.
    // The same for a gallery picture the data has since dropped.
    for (label, kind, id) in [
        ("dangling background", "operator", "char_999_nobody"),
        (
            "dangling gallery background",
            "archive_pic",
            "act99side_pic_0",
        ),
    ] {
        set_stored_background(&state.db, subject.id, json!({ "kind": kind, "id": id })).await;
        save_raw(state, owner, json!({})).await;
        let (_, visitor_profile) = get_user(state, uid, None).await;
        let (_, owner_profile) = get_user(state, uid, Some(owner)).await;
        println!(
            "{label}: visitor {}, owner {}",
            visitor_profile["profile_layout"]
                .get("background")
                .is_some(),
            owner_profile["profile_layout"]["background"]
        );
        if visitor_profile["profile_layout"]
            .get("background")
            .is_some()
            || owner_profile["profile_layout"]["background"]["id"] != json!(id)
        {
            failures.push(format!("{label} reached a visitor"));
        }
    }
    // `null` removes it and nothing else.
    let before_clear = stored_layout(&state.db, subject.id).await.expect("stored");
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "background": null } }),
    )
    .await;
    let cleared = stored_layout(&state.db, subject.id).await.expect("stored");
    println!(
        "background null: key gone {}, tabs kept {}, showcase kept {}",
        cleared.get("background").is_none(),
        cleared["tabs"] == before_clear["tabs"],
        cleared["showcase"] == before_clear["showcase"]
    );
    if cleared.get("background").is_some()
        || cleared["tabs"] != before_clear["tabs"]
        || cleared["showcase"] != before_clear["showcase"]
    {
        failures.push(format!("background null: {cleared}"));
    }
    // Back to every tab visible, with a background, for the search check.
    save_raw(
        state,
        owner,
        json!({ "profile_layout": { "tabs": tabs_without(None), "background": skin_bg } }),
    )
    .await;
}

/// A search row carries no showcase or background, and a private profile
/// refuses every tab endpoint to a visitor.
async fn check_search_row_and_private_profile(
    state: &AppState,
    subject: &Subject,
    owner: &AuthUser,
    failures: &mut Vec<String>,
) {
    let uid = subject.uid.as_str();
    // Search rows carry the tabs but never the showcase.
    let Json(found) = routes::search::search(
        State(state.clone()),
        uri_query(&format!("q={}&limit=100", subject.nickname)),
    )
    .await
    .expect("search");
    let row = found.entries.iter().find(|e| e.profile.uid == uid);
    let row_showcase = row.map(|e| {
        e.profile
            .profile_layout
            .as_ref()
            .is_some_and(|l| l.showcase.is_some() || l.background.is_some())
    });
    println!(
        "search row found {}, carries showcase or background {row_showcase:?}",
        row.is_some()
    );
    if row_showcase != Some(false) {
        failures.push(format!("search row showcase: {row_showcase:?}"));
    }

    // A private profile: every tab endpoint 403 to a visitor and 200 to the
    // owner. `/plans/public` refused the owner here before it took the
    // shared gate.
    let hidden = Subject {
        public_profile: false,
        ..subject.clone()
    };
    save(state, owner, &hidden, json!({ "profile_layout": null })).await;
    for ep in Endpoint::ALL {
        let v = hit(state, ep, uid, None).await;
        let o = hit(state, ep, uid, Some(owner)).await;
        println!("private profile: {ep:?} visitor {v}, owner {o}");
        if (v, o) != (StatusCode::FORBIDDEN, StatusCode::OK) {
            failures.push(format!("private profile: {ep:?} visitor {v}, owner {o}"));
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "reads and temporarily rewrites one row of the local Postgres, needs the EN game data"]
async fn private_tabs_refuse_visitors_and_a_null_layout_changes_nothing() {
    let state = state().await;
    let subject = pick_subject(&state.db).await;
    let before = settings_row(&state.db, subject.id).await;
    println!("subject uid {} ({})", subject.uid, subject.id);

    let outcome = tokio::spawn(run(state.clone(), subject.clone())).await;

    // Put the row back exactly, `updated_at` included: replica mode holds the
    // timestamp trigger off for this transaction only.
    let after: String = {
        let mut tx = state.db.begin().await.expect("tx");
        sqlx::query("SET LOCAL session_replication_role = replica")
            .execute(&mut *tx)
            .await
            .expect("replica mode");
        sqlx::query(
            "UPDATE user_settings us SET
                 public_profile = b.public_profile, store_gacha = b.store_gacha,
                 share_stats = b.share_stats, profile_layout = b.profile_layout,
                 updated_at = b.updated_at
             FROM json_populate_record(NULL::user_settings, $2::json) b
             WHERE us.user_id = $1",
        )
        .bind(subject.id)
        .bind(&before)
        .execute(&mut *tx)
        .await
        .expect("restore");
        tx.commit().await.expect("commit");
        settings_row(&state.db, subject.id).await
    };
    assert_eq!(before, after, "the settings row was not restored");
    println!("settings row restored byte for byte");

    if let Err(e) = outcome {
        std::panic::resume_unwind(e.into_panic());
    }
}

/// The gallery a `archive_pic` background picks from: every EN picture listed
/// once, under the event or theme whose archive shows it, and each served as
/// a 320 px and a 1600 px JPEG through the real route. Writes the variants it
/// renders into `assets/output/en/derived/archive-pics/`, the cache the route
/// itself fills. Prints the bytes and the render time of each.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the EN game data and the local Postgres the state connects to"]
async fn the_gallery_lists_every_picture_once_and_serves_small_variants() {
    use axum::extract::Path as AxumPath;
    use backend::app::services::story::GallerySize;

    let state = state().await;
    let Json(gallery) = routes::story::gallery(State(state.clone()))
        .await
        .expect("gallery");
    let ids: Vec<&str> = gallery
        .groups
        .iter()
        .flat_map(|g| g.pictures.iter().map(|p| p.id.as_str()))
        .collect();
    let unique: std::collections::HashSet<&str> = ids.iter().copied().collect();
    let table = common::shared_game_data()
        .story_archives
        .act_archive_res_data
        .pics
        .len();
    for g in &gallery.groups {
        println!("group {} `{}`: {} pictures", g.id, g.name, g.pictures.len());
    }
    let missing = gallery
        .groups
        .iter()
        .flat_map(|g| &g.pictures)
        .filter(|p| p.url.is_none())
        .count();
    println!(
        "gallery: {} groups, {} pictures, {} unique, table {table}, {missing} without a file",
        gallery.groups.len(),
        ids.len(),
        unique.len()
    );
    assert_eq!(ids.len(), table, "every table picture listed");
    assert_eq!(unique.len(), ids.len(), "no picture listed twice");
    assert_eq!(missing, 0, "every picture resolves to a file");
    assert!(
        gallery.groups.iter().all(|g| g.name != g.id),
        "every group is named by the data"
    );

    // `GALLERY_ALL=1` renders every picture instead of four, for the census
    // the module doc quotes (and warms the whole cache).
    let sample: Vec<&str> = if std::env::var_os("GALLERY_ALL").is_some() {
        ids.clone()
    } else {
        vec![
            "act13side_pic_0",
            "act25side_pic_11",
            "pic_rogue_1_KV1",
            "pic_rogue_5_KV1",
        ]
    };
    let mut totals = [0usize; 2];
    for &id in &sample {
        for size in [GallerySize::Thumb, GallerySize::Header] {
            let started = std::time::Instant::now();
            let response = routes::story::gallery_picture(
                State(state.clone()),
                HeaderMap::new(),
                AxumPath((id.to_owned(), size)),
            )
            .await
            .expect("variant");
            let status = response.status();
            let kind = response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_owned();
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body");
            let img = image::load_from_memory(&bytes).expect("decodes");
            println!(
                "{id} {}: {status} {kind} {}x{} {} bytes in {} ms",
                size.as_str(),
                img.width(),
                img.height(),
                bytes.len(),
                started.elapsed().as_millis()
            );
            assert_eq!(status, StatusCode::OK);
            assert_eq!(kind, "image/jpeg");
            assert_eq!(img.width(), size.width());
            assert_eq!(img.height(), size.width() * 9 / 16);
            totals[usize::from(size == GallerySize::Header)] += bytes.len();
        }
    }
    println!(
        "{} pictures: thumb mean {} bytes, header mean {} bytes",
        sample.len(),
        totals[0] / sample.len(),
        totals[1] / sample.len()
    );
    let unknown = routes::story::gallery_picture(
        State(state.clone()),
        HeaderMap::new(),
        AxumPath(("act99side_pic_0".to_owned(), GallerySize::Thumb)),
    )
    .await;
    assert!(unknown.is_err(), "an unknown id is no file");
}

/// The story CG and scene catalogues a `story_cg` / `story_scene` background
/// picks from: the census per kind (distinct files, passed, kept out and
/// why), the JSON bytes of each payload, and a sample of each served as a
/// thumb and a header through the real route, with their bytes against the
/// source PNG's. `STORY_ART_ALL=1` renders every picture instead of a sample
/// (and warms the whole `derived/story-art` cache).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs the EN game data and the local Postgres the state connects to"]
async fn story_art_lists_each_header_sized_cg_and_scene_once() {
    use axum::extract::Path as AxumPath;
    use backend::app::services::story::story_art::{
        MAX_ASPECT, MIN_ASPECT, MIN_WIDTH, story_art_census,
    };
    use backend::app::services::story::{GallerySize, StoryArtKind};

    let state = state().await;
    // What the index itself holds per category, before any resolve or filter:
    // (groups, groups naming any image, distinct image names, any background, distinct background names).
    let index = backend::app::services::story::cached_index(&state, state.default_server)
        .await
        .expect("index");
    let mut raw: std::collections::BTreeMap<String, [usize; 5]> = std::collections::BTreeMap::new();
    for g in &index.index.groups {
        let e = raw.entry(format!("{:?}", g.category)).or_default();
        e[0] += 1;
        if let Some(r) = index.illustrations.get(&g.id) {
            e[1] += usize::from(!r.images.is_empty());
            e[2] += r.images.len();
            e[3] += usize::from(!r.backgrounds.is_empty());
            e[4] += r.backgrounds.len();
        }
    }
    println!(
        "index refs by category [groups, with images, image refs, with bgs, bg refs]: {raw:?}"
    );
    for kind in [StoryArtKind::Cg, StoryArtKind::Scene] {
        let census = story_art_census(&state, state.default_server, kind)
            .await
            .expect("census");
        let Json(gallery) = routes::story::art_gallery(State(state.clone()), AxumPath(kind))
            .await
            .expect("art gallery");
        let pictures: Vec<_> = gallery.groups.iter().flat_map(|g| &g.pictures).collect();
        let unique: std::collections::HashSet<&str> =
            pictures.iter().map(|p| p.id.as_str()).collect();
        let bytes = serde_json::to_vec(&gallery).expect("json").len();
        let mut by_category: std::collections::BTreeMap<String, (usize, usize)> =
            std::collections::BTreeMap::new();
        for g in &gallery.groups {
            let e = by_category.entry(format!("{:?}", g.category)).or_default();
            e.0 += 1;
            e.1 += g.pictures.len();
        }
        let mut sizes: std::collections::BTreeMap<(u32, u32), usize> =
            std::collections::BTreeMap::new();
        for p in &pictures {
            *sizes.entry((p.width, p.height)).or_default() += 1;
        }
        let mut excluded_sizes: std::collections::BTreeMap<(u32, u32), Vec<&str>> =
            std::collections::BTreeMap::new();
        let mut sparse: Vec<(String, f64)> = Vec::new();
        for (id, w, h, bytes) in &census.excluded {
            excluded_sizes.entry((*w, *h)).or_default().push(id);
            #[allow(clippy::cast_precision_loss)]
            let bpp = *bytes as f64 / (f64::from(*w) * f64::from(*h));
            if bpp < backend::app::services::story::story_art::MIN_BYTES_PER_PX {
                sparse.push((id.clone(), (bpp * 1000.0).round() / 1000.0));
            }
        }
        println!("{} sparse (bytes per px): {sparse:?}", kind.as_str());
        println!("{} excluded by size: {excluded_sizes:?}", kind.as_str());
        println!(
            "{}: distinct {} passed {} narrow {} aspect {} sparse {} unresolved {}; {} groups, {} pictures, {} unique, {bytes} JSON bytes",
            kind.as_str(),
            census.distinct,
            census.passed,
            census.narrow,
            census.aspect,
            census.sparse,
            census.unresolved,
            gallery.groups.len(),
            pictures.len(),
            unique.len()
        );
        println!(
            "{} by category (groups, pictures): {by_category:?}",
            kind.as_str()
        );
        println!("{} passing sizes: {sizes:?}", kind.as_str());
        assert_eq!(unique.len(), pictures.len(), "no picture listed twice");
        assert_eq!(pictures.len(), census.passed);
        assert_eq!(
            census.distinct,
            census.passed + census.narrow + census.aspect + census.sparse + census.unresolved
        );
        assert!(pictures.iter().all(|p| {
            let a = f64::from(p.width) / f64::from(p.height);
            p.width >= MIN_WIDTH && (MIN_ASPECT..=MAX_ASPECT).contains(&a)
        }));
        assert!(pictures.iter().all(|p| p.id == p.id.to_ascii_lowercase()));

        let sample: Vec<&str> = if std::env::var_os("STORY_ART_ALL").is_some() {
            pictures.iter().map(|p| p.id.as_str()).collect()
        } else {
            let step = (pictures.len() / 40).max(1);
            pictures
                .iter()
                .step_by(step)
                .map(|p| p.id.as_str())
                .collect()
        };
        let mut totals = [0usize; 2];
        let mut png_total = 0usize;
        let root = std::path::PathBuf::from(
            &state
                .try_server_data(state.default_server)
                .expect("server")
                .assets_dir,
        );
        let sub = if kind == StoryArtKind::Cg {
            "imgs"
        } else {
            "bg"
        };
        let on_disk: std::collections::HashMap<String, u64> =
            std::fs::read_dir(root.join(format!("textures/avg/{sub}")))
                .expect("art dir")
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .flat_map(|d| std::fs::read_dir(d.path()).into_iter().flatten())
                .filter_map(Result::ok)
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_ascii_lowercase();
                    let stem = name.strip_suffix(".png")?.to_owned();
                    Some((stem, e.metadata().ok()?.len()))
                })
                .collect();
        for &id in &sample {
            png_total += on_disk
                .get(id)
                .map_or(0, |&n| usize::try_from(n).unwrap_or(0));
            for size in [GallerySize::Thumb, GallerySize::Header] {
                let response = routes::story::art_picture(
                    State(state.clone()),
                    HeaderMap::new(),
                    AxumPath((kind, id.to_owned(), size)),
                )
                .await
                .expect("variant");
                let status = response.status();
                let ctype = response
                    .headers()
                    .get(axum::http::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_owned();
                let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .expect("body");
                let img = image::load_from_memory(&body).expect("decodes");
                if std::env::var_os("STORY_ART_ALL").is_none() {
                    println!(
                        "{} {id} {}: {status} {ctype} {}x{} {} bytes",
                        kind.as_str(),
                        size.as_str(),
                        img.width(),
                        img.height(),
                        body.len()
                    );
                }
                assert_eq!(status, StatusCode::OK);
                assert_eq!(ctype, "image/jpeg");
                assert!(img.width() <= size.width());
                totals[usize::from(size == GallerySize::Header)] += body.len();
            }
        }
        println!(
            "{} {} pictures: thumb mean {} bytes, header mean {} bytes, PNG mean {} bytes",
            kind.as_str(),
            sample.len(),
            totals[0] / sample.len(),
            totals[1] / sample.len(),
            png_total / sample.len()
        );
        let unknown = routes::story::art_picture(
            State(state.clone()),
            HeaderMap::new(),
            AxumPath((kind, "no_such_art_99".to_owned(), GallerySize::Thumb)),
        )
        .await;
        assert!(unknown.is_err(), "an unknown id is no file");
    }
}
