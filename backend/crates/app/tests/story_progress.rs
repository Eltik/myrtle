#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use myrtle_app::services::story::progress::{
    self, StoryProgressDocument, stored_payload, validate,
};
use myrtle_app::snapshots::Snapshot;
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::story::{self as repo, GameStoryMark, StoredMarkRow};
use myrtle_engine_story::StoryRef;
use myrtle_engine_story::verdict::{GameStoryRead, parse_game_story_read};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_gamedata_raw::tables::story::{StoryRequiredStage, StoryReviewGroup};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const fn doc(json: Value) -> StoryProgressDocument {
    StoryProgressDocument(json)
}

fn gate(stage: &str, min_state: &str) -> Vec<StoryRequiredStage> {
    vec![StoryRequiredStage {
        stage_id: stage.to_owned(),
        min_state: min_state.to_owned(),
        max_state: "COMPLETE".to_owned(),
    }]
}

#[test]
fn admits_an_empty_and_a_full_document() {
    assert!(validate(&doc(json!({ "v": 1, "read": {}, "pos": {} }))).is_ok());
    let full = doc(json!({
        "v": 1,
        "read": { "main_00-01_beg": 1 },
        "pos": { "main_00-01_end": { "halt": 12, "total": 40, "ts": 1_759_000_000_000u64, "choices": { "0": "1" } } },
        "last": "main_00-01_end",
    }));
    assert!(validate(&full).is_ok());
}

#[test]
fn refuses_a_wrong_version_and_a_bad_unread() {
    assert!(validate(&doc(json!({ "v": 3, "read": {}, "pos": {} }))).is_err());
    assert!(validate(&doc(json!({ "v": 2, "read": {}, "pos": {} }))).is_ok());
    assert!(
        validate(&doc(
            json!({ "v": 2, "read": {}, "pos": {}, "unread": { "a": 5 } })
        ))
        .is_ok()
    );
    for unread in [
        json!({ "": 5 }),
        json!([]),
        json!({ "a": "x" }),
        json!({ "a": -1 }),
    ] {
        let d = doc(json!({ "v": 2, "read": {}, "pos": {}, "unread": unread }));
        assert!(validate(&d).is_err(), "{unread}");
    }
    assert!(validate(&doc(json!({ "read": {}, "pos": {} }))).is_err());
}

#[test]
fn refuses_a_non_object_root_or_branch() {
    assert!(validate(&doc(json!([]))).is_err());
    assert!(validate(&doc(json!({ "v": 1, "read": [], "pos": {} }))).is_err());
    assert!(validate(&doc(json!({ "v": 1, "read": {}, "pos": 3 }))).is_err());
    assert!(validate(&doc(json!({ "v": 1, "read": {}, "pos": { "s": 3 } }))).is_err());
}

#[test]
fn admits_an_optional_reach_and_refuses_a_bad_one() {
    let with = |reach: Value| {
        doc(
            json!({ "v": 2, "read": {}, "pos": { "s": { "halt": 16, "total": 90, "ts": 1, "choices": {}, "reach": reach } } }),
        )
    };
    assert!(validate(&with(json!(85))).is_ok());
    assert!(validate(&with(json!(-1))).is_err());
    assert!(validate(&with(json!("85"))).is_err());
    assert!(
        validate(&doc(
            json!({ "v": 2, "read": {}, "pos": { "s": { "halt": 16, "total": 90, "ts": 1 } } })
        ))
        .is_ok()
    );
}

#[test]
fn refuses_a_negative_or_missing_number() {
    let neg = json!({ "v": 1, "read": {}, "pos": { "s": { "halt": -1, "total": 4, "ts": 1 } } });
    assert!(validate(&doc(neg)).is_err());
    let missing = json!({ "v": 1, "read": {}, "pos": { "s": { "halt": 0, "ts": 1 } } });
    let err = validate(&doc(missing)).unwrap_err();
    assert!(
        matches!(err, AppError::BadRequest(ref m) if m == "progress.pos.s.total must be a finite number at or above zero"),
        "{err:?}"
    );
}

#[test]
fn refuses_an_empty_story_id_and_a_non_string_last() {
    assert!(validate(&doc(json!({ "v": 1, "read": { "": 1 }, "pos": {} }))).is_err());
    assert!(validate(&doc(json!({ "v": 1, "read": {}, "pos": { "": {} } }))).is_err());
    assert!(validate(&doc(json!({ "v": 1, "read": {}, "pos": {}, "last": 4 }))).is_err());
}

#[test]
fn refuses_a_document_over_the_cap() {
    let read: serde_json::Map<String, Value> = (0..40_000)
        .map(|i| (format!("story_{i}"), json!(1)))
        .collect();
    let err = validate(&doc(json!({ "v": 1, "read": read, "pos": {} }))).unwrap_err();
    assert!(
        matches!(err, AppError::BadRequest(ref m) if m.contains("byte limit")),
        "{err:?}"
    );
}

#[test]
fn a_stored_set_rebuilds_to_the_refresh_payload_and_the_same_verdict() {
    let lookup: HashMap<String, StoryRef> = [
        (
            "main_0_level_main_00-01_beg",
            "obt/main/level_main_00-01_beg",
        ),
        (
            "1stact_level_a001_01_end",
            "activities/a001/level_a001_01_end",
        ),
    ]
    .into_iter()
    .map(|(id, txt)| {
        (
            id.to_owned(),
            StoryRef {
                story_txt: txt.to_owned(),
                name: String::new(),
                group_id: String::new(),
                story_info: None,
            },
        )
    })
    .collect();
    let row = |id: &str, rc: i32, at: Option<i64>, played: bool, archived: bool| StoredMarkRow {
        story_id: id.to_owned(),
        reread_count: rc,
        unlocked_at: at.and_then(|t| chrono::DateTime::from_timestamp(t, 0)),
        played,
        archived,
    };
    let rows = vec![
        row("main_0_level_main_00-01_beg", 0, None, true, false),
        row(
            "1stact_level_a001_01_end",
            2,
            Some(1_700_000_200),
            false,
            true,
        ),
        row("gone_level_x_beg", 0, None, true, false),
    ];
    let stages = json!({ "main_08_st_01": { "state": 1, "startTimes": 1, "completeTimes": 0 } });
    let raw = stored_payload(&rows, Some(stages), &lookup);
    assert_eq!(
        raw["user"]["status"]["flags"]["obt/main/level_main_00-01_beg"],
        1
    );
    let stored = &raw["user"]["storyreview"]["groups"]["stored"]["stories"][0];
    assert_eq!(stored["id"], "1stact_level_a001_01_end");
    assert_eq!(stored["uts"], 1_700_000_200);

    let by_txt: HashMap<String, String> = lookup
        .iter()
        .map(|(id, r)| (r.story_txt.clone(), id.clone()))
        .collect();
    let mut gates = HashMap::new();
    gates.insert(
        "main_8_level_st_08-01".to_owned(),
        gate("main_08_st_01", "PLAYED"),
    );
    gates.insert(
        "1stact_level_a001_01_end".to_owned(),
        gate("a001_01", "PASS"),
    );
    let set = parse_game_story_read(&raw, &by_txt, &gates);
    let by_id: HashMap<&str, &GameStoryRead> =
        set.read.iter().map(|r| (r.story_id.as_str(), r)).collect();
    let beg = by_id["main_0_level_main_00-01_beg"];
    assert!(beg.played && beg.read);
    assert!(by_id["main_8_level_st_08-01"].cleared && by_id["main_8_level_st_08-01"].read);
    let unlocked = by_id["1stact_level_a001_01_end"];
    assert!(
        unlocked.archived
            && !unlocked.read
            && unlocked.reread_count == 2
            && unlocked.unlocked_at == Some(1_700_000_200)
    );
    assert!(!by_id.contains_key("gone_level_x_beg"));
    assert_eq!(set.read_count(), 2);
    assert_eq!(set.flag_misses, 0);
}

fn group(id: &str, stories: &[(&str, &str, Option<&str>)]) -> StoryReviewGroup {
    let datas: Vec<Value> = stories
        .iter()
        .map(|(story, txt, stage)| {
            let gates = stage.map_or_else(Vec::new, |s| {
                vec![json!({ "StageId": s, "MinState": "PASS", "MaxState": "COMPLETE" })]
            });
            json!({ "StoryId": story, "StoryGroup": id, "StoryName": story, "StoryTxt": txt, "RequiredStages": gates })
        })
        .collect();
    serde_json::from_value(json!({
        "Id": id, "Name": id, "EntryType": "ACTIVITY", "ActType": "ACTIVITY_STORY",
        "InfoUnlockDatas": datas
    }))
    .unwrap()
}

fn install_library(ctx: &AppContext) {
    let mut model = GameModel::default();
    model.story_reviews.insert(
        "act1".to_owned(),
        group(
            "act1",
            &[
                ("act1_level_st_01", "activities/act1/level_st_01", None),
                (
                    "act1_level_01_end",
                    "activities/act1/level_01_end",
                    Some("act1_01"),
                ),
            ],
        ),
    );
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(model, SandboxUniverse::default()),
    );
}

fn mark(id: &str, archived: bool, read: bool) -> GameStoryMark {
    GameStoryMark {
        story_id: id.to_owned(),
        reread_count: 0,
        unlocked_at: None,
        played: false,
        archived,
        cleared: false,
        read_in_game: read,
    }
}

async fn signed_in() -> (AppContext, myrtle_testkit::db::TestDb, UserId, Subject) {
    let (ctx, db) = common::context().await;
    let user = db.user(1, Server::EN).await;
    let subject = common::user(&ctx, user, Role::User).await;
    (ctx, db, user, subject)
}

#[tokio::test]
async fn every_entry_point_refuses_an_anonymous_caller() {
    let (ctx, db) = common::context().await;
    let anon = Subject::Anonymous;
    let empty = doc(json!({ "v": 2, "read": {}, "pos": {} }));
    assert!(matches!(
        progress::get(&ctx, &anon).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        progress::put(&ctx, &anon, empty).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        progress::import(&ctx, &anon).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn a_put_round_trips_and_the_get_carries_the_game_marks() {
    let (ctx, db, user, me) = signed_in().await;
    let never = progress::get(&ctx, &me).await.unwrap();
    assert!(never.progress.is_none() && never.updated_at.is_none());
    assert!(never.game_read.is_empty() && never.game_synced_at.is_none());

    let marks = [
        mark("a", true, true),
        mark("b", false, true),
        mark("c", true, false),
    ];
    repo::replace_game_marks(&mut ctx.db.conn().await.unwrap(), user, &marks)
        .await
        .unwrap();
    let body = json!({ "v": 2, "read": { "x": 1 }, "pos": {} });
    let put = progress::put(&ctx, &me, doc(body.clone())).await.unwrap();
    assert_eq!(put.progress, Some(doc(body.clone())));
    assert!(put.updated_at.is_some());
    assert_eq!(put.game_read, ["a", "b"]);
    assert_eq!(put.game_unread, ["c"]);
    assert_eq!(put.game_archived, 1, "only a read archived story counts");
    assert!(put.game_synced_at.is_some());
    assert_eq!(progress::get(&ctx, &me).await.unwrap(), put);

    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let theirs = progress::get(&ctx, &other).await.unwrap();
    assert!(
        theirs.progress.is_none() && theirs.game_read.is_empty(),
        "one account never reads another's"
    );
    db.close().await;
}

#[tokio::test]
async fn a_refused_document_writes_nothing() {
    let (ctx, db, _, me) = signed_in().await;
    let bad = doc(json!({ "v": 9, "read": {}, "pos": {} }));
    assert!(matches!(
        progress::put(&ctx, &me, bad).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(progress::get(&ctx, &me).await.unwrap().progress.is_none());
    db.close().await;
}

#[tokio::test]
async fn a_set_without_either_source_never_erases_the_stored_marks() {
    let (ctx, db, user, me) = signed_in().await;
    repo::replace_game_marks(
        &mut ctx.db.conn().await.unwrap(),
        user,
        &[mark("a", true, true)],
    )
    .await
    .unwrap();
    let empty = myrtle_engine_story::verdict::GameStoryReadSet::default();
    assert_eq!(
        progress::store_game_read(&ctx, user, &empty).await.unwrap(),
        0
    );
    assert_eq!(progress::get(&ctx, &me).await.unwrap().game_read, ["a"]);
    db.close().await;
}

#[tokio::test]
async fn import_reruns_the_verdict_over_the_stored_rows() {
    let (ctx, db, user, me) = signed_in().await;
    install_library(&ctx);
    let marks = [
        mark("act1_level_st_01", true, false),
        mark("act1_level_01_end", true, true),
    ];
    repo::replace_game_marks(&mut ctx.db.conn().await.unwrap(), user, &marks)
        .await
        .unwrap();
    let out = progress::import(&ctx, &me).await.unwrap();
    assert_eq!(out.game_read, ["act1_level_st_01"]);
    assert_eq!(
        out.game_unread,
        ["act1_level_01_end"],
        "archived behind an uncleared gate"
    );
    assert_eq!(out.game_archived, 1);
    db.close().await;
}

#[tokio::test]
async fn import_with_nothing_stored_answers_as_get() {
    let (ctx, db, _, me) = signed_in().await;
    let out = progress::import(&ctx, &me).await.unwrap();
    assert_eq!(out, progress::get(&ctx, &me).await.unwrap());
    db.close().await;
}
