#![allow(clippy::unwrap_used)]

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use myrtle_app::services::catalog::maintenance::{
    MaintenanceState, Notice, NoticeWindow, state_key,
};
use myrtle_app::services::grading;
use myrtle_app::services::sync::sandbox::{self, MissingTopic, SandboxBackfill};
use myrtle_app::services::sync::{after_grade, refresh, store};
use myrtle_app::snapshots::Snapshot;
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::{items, lookback, progress, roster, squads, story, users};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::campaign::{CampaignRotations, CampaignTableFile};
use myrtle_gamedata_raw::tables::module::{Module, ModuleType};
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_jobs::{Job, JobKind, JobSpec};
use myrtle_testkit::db::TestDb;
use myrtle_types::{Role, Server, UserId, WorkerId};
use serde_json::{Value, json};

const UID: &str = "10000001";
const FIXTURE: &str = include_str!("fixtures/sync/sync_data.json");

fn sync_model() -> GameModel {
    let mut model = GameModel::default();
    for (id, kind) in [
        ("uniequip_001_lin", ModuleType::Initial),
        ("uniequip_002_lemuen", ModuleType::Advanced),
    ] {
        let module = Module {
            module_type: kind,
            ..Default::default()
        };
        model.modules.equip_dict.insert(id.to_owned(), module);
    }
    let table: CampaignTableFile = serde_json::from_value(json!({
        "Campaigns": [
            {"key": "camp_01", "value": {"BreakLadders": [{"KillCnt": 100}, {"KillCnt": 400}]}},
            {"key": "camp_r_01", "value": {"BreakLadders": [{"KillCnt": 400}]}}
        ]
    }))
    .unwrap();
    model.campaign_rotations = CampaignRotations::from_table(table);
    model
}

fn install_model(ctx: &AppContext) {
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(sync_model(), SandboxUniverse::default()),
    );
}

async fn synced() -> (AppContext, TestDb, UserId) {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    let user = users::find(&mut conn, UID, Server::EN)
        .await
        .unwrap()
        .unwrap();
    (ctx, db, UserId(user.id))
}

async fn claim(ctx: &AppContext, kind: JobKind) -> Vec<Job> {
    ctx.queue
        .claim(WorkerId(uuid::Uuid::new_v4()), &[kind], 10)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_sync_payload_lands_in_every_table() {
    let (ctx, db, user) = synced().await;
    let mut conn = ctx.db.conn().await.unwrap();

    let account = users::account(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(account.nickname.as_deref(), Some("Doctor"));
    assert_eq!(account.level, Some(120));

    let operators = roster::roster(&mut conn, user).await.unwrap();
    let ids: BTreeSet<&str> = operators.iter().map(|o| o.operator_id.as_str()).collect();
    assert_eq!(
        ids,
        BTreeSet::from([
            "char_002_amiya",
            "char_1001_amiya2",
            "char_1037_amiya3",
            "char_1028_texas2",
            "char_4080_lin",
            "char_4193_lemuen",
        ]),
        "one row per Amiya form, none for the parent"
    );
    let amiya2 = roster::operator(&mut conn, user, "char_1001_amiya2")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(amiya2.skin_id.as_deref(), Some("char_1001_amiya2@sale#16"));
    assert_eq!(
        amiya2.elite, 2,
        "the parent's progression fills the form's gaps"
    );
    let lemuen = operators
        .iter()
        .find(|o| o.operator_id == "char_4193_lemuen")
        .unwrap();
    assert_eq!((lemuen.elite, lemuen.level, lemuen.potential), (2, 90, 5));
    let modules = lemuen.modules.to_string();
    assert!(modules.contains("uniequip_002_lemuen"));
    assert!(
        !modules.contains("uniequip_001_"),
        "the default badge is no module"
    );

    let held: BTreeMap<String, i32> = items::inventory(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.item_id, r.quantity))
        .collect();
    assert_eq!(held["7002"], 169, "expedited plans from status");
    assert_eq!(held["classic_normal_ticket"], 529);
    assert_eq!(
        held["voucher_elite_II_4"], 2,
        "the expired instance is dropped"
    );
    assert!(!held.contains_key("voucher_item_4pick1_1803"));
    assert_eq!(held["4002"], 1, "originite is pay plus free");
    assert_eq!(held["4001"], 6_217_576);
    assert_eq!(held["30011"], 26);

    let supports = roster::supports(&mut conn, user).await.unwrap();
    let seats: Vec<(i16, &str)> = supports
        .iter()
        .map(|s| (s.slot, s.operator_id.as_str()))
        .collect();
    assert_eq!(seats, [(1, "char_4193_lemuen"), (2, "char_4080_lin")]);
    assert_eq!(supports[0].specialize_level, 3);

    let game = squads::list_game_squads(&mut conn, user).await.unwrap();
    assert_eq!(game.len(), 1);
    let slots = game[0].slots.as_array().unwrap();
    assert_eq!(slots.len(), 12);
    assert_eq!(
        slots[0],
        json!({"char_id": "char_4193_lemuen", "skill_index": 1, "module_id": "uniequip_002_lemuen"})
    );
    assert!(
        slots[1].is_null() && slots[2].is_null(),
        "empty and unresolved seats"
    );
    assert_eq!(slots[3]["module_id"], Value::Null, "an INITIAL equip");

    let clears = progress::stage_clears(&mut conn, user)
        .await
        .unwrap()
        .clears;
    assert_eq!(clears["camp_01"].state, 3);
    assert_eq!(clears["camp_r_01"].state, 2);
    assert_eq!(clears["main_00-01"].state, 3);

    let medals = progress::medals(&mut conn, user).await.unwrap();
    assert_eq!(medals.len(), 3, "progress-only medals are not stored");
    assert_eq!(roster::owned_skins(&mut conn, user).await.unwrap().len(), 3);
    let checkin = users::checkin_by_uid(&mut conn, UID, Some(Server::EN))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkin.history.len(), 28);
    assert_eq!(checkin.cumulative_signin, 1047);
    assert_eq!(checkin.checkin_group_id.as_deref(), Some("signin82"));
    assert_eq!(progress::roguelike(&mut conn, user).await.unwrap().len(), 1);
    assert_eq!(
        progress::encountered_enemies(&mut conn, user)
            .await
            .unwrap()
            .len(),
        3
    );
    drop(conn);
    db.close().await;
}

fn backfill_job(user: UserId) -> SandboxBackfill {
    SandboxBackfill {
        user_id: user.0,
        loaded: "sandbox_2".into(),
        topics: vec![MissingTopic {
            template: "SANDBOX_V2".into(),
            topic: "sandbox_1".into(),
        }],
    }
}

#[tokio::test]
async fn a_sync_queues_the_grade_and_the_backfill() {
    let (ctx, db, user) = synced().await;
    let regrades = claim(&ctx, JobKind::RegradeUser).await;
    assert_eq!(regrades.len(), 1);
    assert_eq!(regrades[0].payload, json!({ "user_id": user.0 }));
    let backfills = claim(&ctx, JobKind::SandboxBackfill).await;
    assert_eq!(backfills.len(), 1);
    let queued: SandboxBackfill = serde_json::from_value(backfills[0].payload.clone()).unwrap();
    assert_eq!(queued, backfill_job(user));
    db.close().await;
}

#[tokio::test]
async fn a_regrade_already_queued_does_not_cost_the_sync_its_backfill() {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    let row = users::find_or_create(&mut ctx.db.conn().await.unwrap(), UID, Server::EN)
        .await
        .unwrap();
    grading::enqueue_regrade(&ctx, UserId(row.id))
        .await
        .unwrap();
    store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    assert_eq!(
        claim(&ctx, JobKind::RegradeUser).await.len(),
        1,
        "one shared run"
    );
    assert_eq!(claim(&ctx, JobKind::SandboxBackfill).await.len(), 1);
    db.close().await;
}

#[tokio::test]
async fn stored_topics_queue_a_regrade_behind_one_already_running() {
    let (ctx, db, user) = synced().await;
    assert_eq!(
        claim(&ctx, JobKind::RegradeUser).await.len(),
        1,
        "now running"
    );
    sandbox::topics_stored(&ctx, user, UID).await.unwrap();
    sandbox::topics_stored(&ctx, user, UID).await.unwrap();
    assert_eq!(
        claim(&ctx, JobKind::RegradeUser).await.len(),
        1,
        "one regrade that reads the new topics"
    );
    db.close().await;
}

#[tokio::test]
async fn every_sync_before_the_ownership_refresh_shares_one_run() {
    let (ctx, db, _) = synced().await;
    store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    assert!(
        claim(&ctx, JobKind::OperatorOwnershipStats)
            .await
            .is_empty(),
        "held about a minute"
    );
    let before = chrono::Utc::now();
    ctx.queue
        .enqueue(JobSpec {
            dedupe_key: Some("sync".into()),
            ..JobSpec::new(JobKind::OperatorOwnershipStats)
        })
        .await
        .unwrap();
    let runs = claim(&ctx, JobKind::OperatorOwnershipStats).await;
    assert_eq!(runs.len(), 1);
    assert!(runs[0].created_at < before, "the syncs queued it");
    db.close().await;
}

#[tokio::test]
async fn a_sync_returns_the_payload_and_drops_the_users_caches() {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    let ttl = Duration::from_mins(5);
    for key in [
        format!("user:{UID}"),
        format!("improvements:{UID}:7"),
        "improvements:20000002:7".to_owned(),
    ] {
        ctx.cache.set(&key, "cached", ttl).await;
    }
    let mut payload = store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    let import = payload
        .as_object_mut()
        .unwrap()
        .remove("gameReadImport")
        .unwrap();
    assert_eq!(import["ok"], true);
    assert_eq!(payload, serde_json::from_str::<Value>(FIXTURE).unwrap());
    assert!(ctx.cache.get(&format!("user:{UID}")).await.is_none());
    assert!(
        ctx.cache
            .get(&format!("improvements:{UID}:7"))
            .await
            .is_none()
    );
    assert!(
        ctx.cache.get("improvements:20000002:7").await.is_some(),
        "another player's body stays"
    );
    db.close().await;
}

#[tokio::test]
async fn a_second_sync_replaces_the_first() {
    let (ctx, db, user) = synced().await;
    let mut lighter: Value = serde_json::from_str(FIXTURE).unwrap();
    let chars = lighter["user"]["troop"]["chars"].as_object_mut().unwrap();
    chars.retain(|inst, _| inst == "293");
    lighter["user"]["inventory"] = json!({ "30011": 1 });
    store(&ctx, UID, Server::EN, lighter.to_string())
        .await
        .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    let operators = roster::roster(&mut conn, user).await.unwrap();
    assert_eq!(operators.len(), 1);
    let held = items::inventory(&mut conn, user).await.unwrap();
    assert!(held.iter().any(|r| r.item_id == "30011" && r.quantity == 1));
    assert!(!held.iter().any(|r| r.item_id == "30012"));
    drop(conn);
    assert_eq!(
        claim(&ctx, JobKind::RegradeUser).await.len(),
        1,
        "one queued regrade per user"
    );
    db.close().await;
}

#[tokio::test]
async fn a_payload_without_a_user_is_refused() {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    assert!(matches!(
        store(&ctx, UID, Server::EN, r#"{"result": 0}"#.to_owned()).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        store(&ctx, UID, Server::EN, "not json".to_owned()).await,
        Err(AppError::Internal(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_sync_without_game_data_is_unavailable() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        store(&ctx, UID, Server::EN, FIXTURE.to_owned()).await,
        Err(AppError::Unavailable)
    ));
    db.close().await;
}

#[tokio::test]
async fn refresh_stops_at_an_ongoing_maintenance() {
    let (ctx, db) = common::context().await;
    let en = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let kr = common::user(&ctx, db.user(2, Server::KR).await, Role::User).await;
    let now = chrono::Utc::now().timestamp();
    let state = MaintenanceState {
        notices: vec![Notice {
            id: 1,
            url: "https://example.invalid/1.html".into(),
            window: NoticeWindow {
                starts_at: now - 3600,
                ends_at: Some(now + 3600),
                extended: false,
                title: "maintenance".into(),
            },
        }],
        checked_at: Some(now),
        ..MaintenanceState::default()
    };
    ctx.cache
        .set_json(&state_key(Server::EN), &state, Duration::from_mins(5))
        .await;

    assert!(matches!(
        refresh(&ctx, &en).await,
        Err(AppError::GameMaintenance { ends_at }) if ends_at == Some(now + 3600)
    ));
    assert!(
        matches!(refresh(&ctx, &kr).await, Err(AppError::GameLoginRequired)),
        "only EN is read; no stored credential means login again"
    );
    assert!(matches!(
        refresh(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn the_backfill_skips_a_topic_the_row_already_holds() {
    let (ctx, db, user) = synced().await;
    let job = backfill_job(user);
    assert!(
        matches!(
            sandbox::backfill(&ctx, &job).await,
            Err(AppError::GameLoginRequired)
        ),
        "a missing topic needs the game"
    );
    let mut conn = ctx.db.conn().await.unwrap();
    progress::store_sandbox_topic(
        &mut conn,
        user,
        "SANDBOX_V2",
        "sandbox_1",
        &json!({"tech": {}}),
    )
    .await
    .unwrap();
    drop(conn);
    assert_eq!(sandbox::backfill(&ctx, &job).await.unwrap(), 0);
    let gone = SandboxBackfill {
        user_id: uuid::Uuid::new_v4(),
        ..job
    };
    assert!(matches!(
        sandbox::backfill(&ctx, &gone).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn after_grade_records_the_periods_lookback_snapshot() {
    let (ctx, db, user) = synced().await;
    after_grade(&ctx, user).await;
    after_grade(&ctx, user).await;
    let mut conn = ctx.db.conn().await.unwrap();
    assert_eq!(lookback::list(&mut conn, user).await.unwrap().len(), 1);
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn a_sync_stores_against_the_real_extract() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    let user = users::find(&mut conn, UID, Server::EN)
        .await
        .unwrap()
        .unwrap();
    let game = squads::list_game_squads(&mut conn, UserId(user.id))
        .await
        .unwrap();
    assert_eq!(
        game[0].slots[3]["module_id"],
        Value::Null,
        "uniequip_001_lin is INITIAL in the extract"
    );
    drop(conn);
    db.close().await;
}

fn install_story_library(ctx: &AppContext) {
    let mut model = sync_model();
    let group = serde_json::from_value(json!({
        "Id": "act9inv", "Name": "Invented", "EntryType": "ACTIVITY", "ActType": "ACTIVITY_STORY",
        "InfoUnlockDatas": [
            {"StoryId": "act9inv_level_act9inv_st01", "StoryTxt": "activities/act9inv/level_act9inv_st01"},
            {"StoryId": "act9inv_level_act9inv_st02", "StoryTxt": "activities/act9inv/level_act9inv_st02",
             "RequiredStages": [{"StageId": "act9inv_02", "MinState": "PASS", "MaxState": "COMPLETE"}]}
        ]
    }))
    .unwrap();
    model.story_reviews.insert("act9inv".to_owned(), group);
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(model, SandboxUniverse::default()),
    );
}

async fn marks(ctx: &AppContext) -> Vec<story::MarkVerdictRow> {
    let mut conn = ctx.db.conn().await.unwrap();
    let user = users::find(&mut conn, UID, Server::EN)
        .await
        .unwrap()
        .unwrap();
    story::mark_verdicts(&mut conn, UserId(user.id))
        .await
        .unwrap()
}

#[tokio::test]
async fn a_sync_imports_both_halves_of_the_games_read_marks() {
    let (ctx, db) = common::context().await;
    install_story_library(&ctx);
    let payload = store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    let import = &payload["gameReadImport"];
    assert_eq!(import["ok"], true);
    assert_eq!(import["rows"], 2);
    assert_eq!(import["read"], 1);
    assert_eq!(import["archived"], 0);
    let rows = marks(&ctx).await;
    let verdict: Vec<(&str, bool, bool)> = rows
        .iter()
        .map(|r| (r.story_id.as_str(), r.archived, r.read_in_game))
        .collect();
    assert_eq!(
        verdict,
        [
            ("act9inv_level_act9inv_st01", false, true),
            ("act9inv_level_act9inv_st02", true, false),
        ]
    );
    db.close().await;
}

#[tokio::test]
async fn an_empty_library_costs_only_the_flags_half() {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    let payload = store(&ctx, UID, Server::EN, FIXTURE.to_owned())
        .await
        .unwrap();
    assert_eq!(payload["gameReadImport"]["rows"], 1);
    let rows = marks(&ctx).await;
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].archived && rows[0].read_in_game,
        "ungated without a library"
    );
    db.close().await;
}

#[tokio::test]
async fn a_payload_without_reading_sources_keeps_the_stored_marks() {
    let (ctx, db, _) = synced().await;
    let mut bare: Value = serde_json::from_str(FIXTURE).unwrap();
    let user = bare["user"].as_object_mut().unwrap();
    user.remove("storyreview");
    user["status"]["flags"] = json!({});
    let payload = store(&ctx, UID, Server::EN, bare.to_string())
        .await
        .unwrap();
    assert_eq!(payload["gameReadImport"]["ok"], true);
    assert_eq!(payload["gameReadImport"]["rows"], 0);
    assert_eq!(marks(&ctx).await.len(), 1, "the first sync's mark stands");
    db.close().await;
}

#[tokio::test]
async fn a_failed_read_mark_store_rides_back_while_the_sync_stands() {
    let (ctx, db) = common::context().await;
    install_model(&ctx);
    let mut payload: Value = serde_json::from_str(FIXTURE).unwrap();
    payload["user"]["storyreview"]["groups"]["act9inv"]["stories"][0]["id"] =
        json!("act9inv_\u{0}_nul"); // Postgres text refuses a NUL byte
    let stored = store(&ctx, UID, Server::EN, payload.to_string())
        .await
        .unwrap();
    let import = &stored["gameReadImport"];
    assert_eq!(import["ok"], false);
    let error = import["error"].as_str().unwrap();
    assert!(
        error.starts_with("the game read store failed at stage delete/insert/commit after "),
        "{error}"
    );
    let mut conn = ctx.db.conn().await.unwrap();
    assert!(
        users::find(&mut conn, UID, Server::EN)
            .await
            .unwrap()
            .is_some()
    );
    drop(conn);
    assert!(marks(&ctx).await.is_empty());
    db.close().await;
}
