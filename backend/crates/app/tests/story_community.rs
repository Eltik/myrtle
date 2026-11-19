#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use myrtle_app::cache::CachedJson;
use myrtle_app::services::story::community::{
    self, AccountRead, CommunityPlan, StoryCommunity, Tally, cache_key,
};
use myrtle_app::snapshots::Snapshot;
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::roster;
use myrtle_db::repos::story::{self as repo, GameStoryMark};
use myrtle_db::repos::users::{self, SettingsUpdate};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_gamedata_raw::tables::story::StoryRequiredStage;
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};

fn gate(stage: &str, min_state: &str) -> Vec<StoryRequiredStage> {
    vec![StoryRequiredStage {
        stage_id: stage.into(),
        min_state: min_state.into(),
        max_state: String::new(),
    }]
}

/// `main_0`: three scripted stories behind three gates; `rec_a`: one ungated record
fn plan() -> CommunityPlan {
    let mut gates = HashMap::new();
    gates.insert("s1".to_owned(), gate("main_00-01", "PLAYED"));
    gates.insert("s2".to_owned(), gate("main_00-02", "PLAYED"));
    gates.insert("s3".to_owned(), gate("main_00-03", "PASS"));
    gates.insert("r1".to_owned(), Vec::new());
    let story = |id: &str| (id.to_owned(), true);
    CommunityPlan::new(
        vec![
            (
                "main_0".to_owned(),
                vec![story("s1"), story("s2"), story("s3")],
                true,
            ),
            ("rec_a".to_owned(), vec![story("r1")], false),
        ],
        &gates,
    )
}

fn account(verdict: &[&str], stages: Option<&str>, document: Option<&str>) -> AccountRead {
    AccountRead {
        verdict: verdict.iter().map(|s| (*s).to_owned()).collect(),
        stages: stages.map(str::to_owned),
        document: document.map(str::to_owned),
    }
}

#[test]
fn three_accounts_one_per_source() {
    let plan = plan();
    assert_eq!((plan.stories(), plan.groups()), (4, 2));
    let mut tally = Tally::new(&plan);
    tally.add(
        &plan,
        &account(
            &["s1", "s2", "s3", "r1"],
            Some(r#"{"main_00-01":{"state":3}}"#),
            None,
        ),
    );
    tally.add(
        &plan,
        &account(
            &[],
            Some(
                r#"{"main_00-01":{"state":3,"startTimes":2,"completeTimes":1},
                    "main_00-02":{"state":2,"startTimes":1,"completeTimes":0},
                    "main_00-03":{"state":0,"startTimes":0,"completeTimes":0}}"#,
            ),
            None,
        ),
    );
    tally.add(
        &plan,
        &account(
            &[],
            Some(r#"{"main_00-01":{"state":3,"startTimes":1}}"#),
            Some(r#"{"v":2,"read":{"r1":1},"unread":{"s1":1},"pos":{}}"#),
        ),
    );

    let out = tally.finish(&plan, 1_700_000_000);
    assert_eq!(out.players, 3);
    assert_eq!(out.computed_at, 1_700_000_000);
    let readers: HashMap<&str, u32> = out
        .stories
        .iter()
        .map(|s| (s.id.as_str(), s.readers))
        .collect();
    assert_eq!(readers["s1"], 2, "the third account hand-cleared s1");
    assert_eq!(readers["s2"], 2);
    assert_eq!(readers["s3"], 1);
    assert_eq!(readers["r1"], 2, "the gates cannot reach an ungated record");

    let main = &out.groups[0];
    assert_eq!(
        (main.id.as_str(), main.readers, main.finished),
        ("main_0", 2, 1)
    );
    assert_eq!(main.depth.as_deref(), Some(&[2, 2, 1][..]));
    let rec = &out.groups[1];
    assert_eq!(
        (rec.id.as_str(), rec.readers, rec.finished),
        ("rec_a", 2, 0)
    );
    assert!(rec.depth.is_none(), "a record group carries no curve");

    let c = tally.census;
    assert_eq!(
        (c.from_verdict, c.from_gates, c.from_document, c.empty),
        (1, 2, 1, 0)
    );
    assert_eq!(c.reads, 4 + 2 + 1);
}

#[test]
fn an_empty_account_is_not_counted_and_batches_sum() {
    let plan = plan();
    let mut tally = Tally::new(&plan);
    tally.add(&plan, &AccountRead::default());
    assert_eq!((tally.census.scanned, tally.census.players), (1, 0));

    let one = account(&["s1"], None, None);
    let mut a = Tally::new(&plan);
    a.add(&plan, &one);
    let mut b = Tally::new(&plan);
    b.add(&plan, &one);
    a.merge(&b);
    let out = a.finish(&plan, 0);
    assert_eq!(out.stories[0].readers, 2);
    assert_eq!((out.groups[0].readers, out.groups[0].finished), (2, 0));
    assert_eq!(out.groups[0].depth.as_deref(), Some(&[2, 0, 0][..]));
    assert_eq!(a.census.players, 2);
}

#[test]
fn the_wire_shape_keeps_main_field_names() {
    let out = Tally::new(&plan()).finish(&plan(), 5);
    let wire = serde_json::to_value(&out).unwrap();
    assert_eq!(wire["computedAt"], 5);
    assert_eq!(wire["groups"][0]["depth"], json!([0, 0, 0]));
    assert!(wire["groups"][1].get("depth").is_none());
}

/// One event, `act1`: `st01` ungated, `st02` behind stage `act1_01`
fn install_library(ctx: &AppContext) {
    let mut model = GameModel::default();
    let group = serde_json::from_value(json!({
        "Id": "act1", "Name": "act1", "EntryType": "ACTIVITY", "ActType": "ACTIVITY_STORY",
        "InfoUnlockDatas": [
            {"StoryId": "act1_st01", "StoryTxt": "activities/act1/st01"},
            {"StoryId": "act1_st02", "StoryTxt": "activities/act1/st02",
             "RequiredStages": [{"StageId": "act1_01", "MinState": "PASS", "MaxState": "COMPLETE"}]}
        ]
    }))
    .unwrap();
    model.story_reviews.insert("act1".to_owned(), group);
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(model, SandboxUniverse::default()),
    );
}

/// A synced account whose only reading source is `stages`
async fn seed(ctx: &AppContext, uid: &str, stages: &Value) -> UserId {
    let (list, map) = (json!([]), json!({}));
    let mut status = json!({});
    for key in [
        "exp",
        "orundum",
        "orundum_shard",
        "lmd",
        "sanity",
        "max_sanity",
        "gacha_tickets",
        "ten_pull_tickets",
        "classic_gacha_tickets",
        "classic_ten_pull_tickets",
        "recruit_permits",
        "social_point",
        "hgg_shard",
        "lgg_shard",
        "practice_tickets",
        "gold",
        "friend_num_limit",
        "originite",
    ] {
        status[key] = json!(0);
    }
    let mut conn = ctx.db.conn().await.unwrap();
    roster::sync_user_data(
        &mut conn,
        &roster::SyncUserData {
            uid,
            server: Server::EN,
            nickname: "Doctor",
            level: 1,
            avatar_id: None,
            secretary: None,
            secretary_skin_id: None,
            resume_id: None,
            operators: &list,
            skills: &list,
            modules: &list,
            items: &list,
            skins: &list,
            status: &status,
            stages,
            roguelike: &list,
            sandbox: &map,
            medals: &list,
            building: &map,
            checkin: &map,
            supports: &list,
            nick_number: None,
            enemies: &map,
            game_squads: &list,
        },
    )
    .await
    .unwrap();
    UserId(
        users::find(&mut conn, uid, Server::EN)
            .await
            .unwrap()
            .unwrap()
            .id,
    )
}

async fn hide(ctx: &AppContext, user: UserId, public_profile: bool, share_stats: bool) {
    let update = SettingsUpdate {
        public_profile: Some(public_profile),
        share_stats: Some(share_stats),
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut ctx.db.conn().await.unwrap(), user, &update)
        .await
        .unwrap();
}

fn readers(out: &StoryCommunity) -> HashMap<&str, u32> {
    out.stories
        .iter()
        .map(|s| (s.id.as_str(), s.readers))
        .collect()
}

#[tokio::test]
async fn either_opt_out_removes_an_account_from_every_count() {
    let (ctx, db) = common::context().await;
    install_library(&ctx);
    let cleared = json!({ "act1_01": { "state": 3, "startTimes": 1, "completeTimes": 1 } });
    seed(&ctx, "100", &cleared).await;
    let private = seed(&ctx, "101", &cleared).await;
    let unshared = seed(&ctx, "102", &cleared).await;
    hide(&ctx, private, false, true).await;
    hide(&ctx, unshared, true, false).await;

    let (out, census) = community::compute(&ctx, Server::EN).await.unwrap();
    assert_eq!(census.scanned, 1);
    assert_eq!(out.players, 1);
    assert_eq!(readers(&out)["act1_st02"], 1);
    assert_eq!(census.from_gates, 1);
    db.close().await;
}

#[tokio::test]
async fn the_verdict_replaces_the_gates_and_the_document_withdraws_last() {
    let (ctx, db) = common::context().await;
    install_library(&ctx);
    let cleared = json!({ "act1_01": { "state": 3 } });
    let judged = seed(&ctx, "200", &cleared).await;
    let mark = GameStoryMark {
        story_id: "act1_st01".to_owned(),
        reread_count: 0,
        unlocked_at: None,
        played: true,
        archived: false,
        cleared: false,
        read_in_game: true,
    };
    let mut conn = ctx.db.conn().await.unwrap();
    repo::replace_game_marks(&mut conn, judged, &[mark])
        .await
        .unwrap();
    let reader = seed(&ctx, "201", &cleared).await;
    let hand =
        json!({ "v": 2, "read": { "act1_st01": 1 }, "unread": { "act1_st02": 1 }, "pos": {} });
    repo::put_document(&mut conn, reader, &hand).await.unwrap();
    drop(conn);

    let (out, census) = community::compute(&ctx, Server::EN).await.unwrap();
    assert_eq!(out.players, 2);
    let r = readers(&out);
    assert_eq!(r["act1_st01"], 2);
    assert_eq!(
        r["act1_st02"], 0,
        "the verdict ignores the stages; the hand clear wins"
    );
    assert_eq!(
        (census.from_verdict, census.from_gates, census.from_document),
        (1, 1, 1)
    );
    assert_eq!(census.empty, 0);
    db.close().await;
}

#[tokio::test]
async fn pages_past_one_batch_count_every_account() {
    let (ctx, db) = common::context().await;
    install_library(&ctx);
    let cleared = json!({ "act1_01": { "state": 3 } });
    for n in 0..130 {
        seed(&ctx, &format!("3{n:03}"), &cleared).await;
    }
    let (out, census) = community::compute(&ctx, Server::EN).await.unwrap();
    assert_eq!(census.scanned, 130);
    assert_eq!(readers(&out)["act1_st02"], 130);
    db.close().await;
}

#[tokio::test]
async fn the_job_body_stores_the_aggregate_readers_then_hit() {
    let (ctx, db) = common::context().await;
    install_library(&ctx);
    seed(&ctx, "400", &json!({ "act1_01": { "state": 3 } })).await;
    community::aggregate(&ctx, Server::EN).await.unwrap();
    let stored: CachedJson = ctx
        .cache
        .get_json(&cache_key(Server::EN, ""))
        .await
        .unwrap();
    let body: StoryCommunity = serde_json::from_str(&stored.body).unwrap();
    assert_eq!(body.players, 1);

    seed(&ctx, "401", &json!({})).await;
    let served = community::community(&ctx, Server::EN).await.unwrap();
    assert_eq!(
        served.body, stored.body,
        "a reader is served the stored run"
    );

    ctx.cache.invalidate(&cache_key(Server::EN, "")).await;
    let fresh = community::community(&ctx, Server::EN).await.unwrap();
    let body: StoryCommunity = serde_json::from_str(&fresh.body).unwrap();
    assert_eq!(body.players, 2, "a miss computes inline");
    db.close().await;
}

#[tokio::test]
async fn an_unloaded_server_is_not_found() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        community::community(&ctx, Server::EN).await,
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        community::aggregate(&ctx, Server::EN).await,
        Err(AppError::NotFound(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_real_library_fits_under_the_wire_ceiling() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let (out, _) = community::compute(&ctx, Server::EN).await.unwrap();
    assert!(out.stories.len() > 1_000 && out.groups.len() > 500);
    let bytes = serde_json::to_vec(&out).unwrap().len();
    assert!(bytes < 150 * 1024, "{bytes} bytes");
    db.close().await;
}
