#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::accounts::account::{get_user_checkin, get_user_score};
use myrtle_app::services::sync::roster::stage_clears;
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::roster::{SyncUserData, sync_user_data};
use myrtle_db::repos::scores::{self, ScoreWrite};
use myrtle_db::repos::users;
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

async fn seed(ctx: &AppContext, uid: &str, stages: &Value, checkin: &Value) -> UserId {
    let (list, map) = (json!([]), json!({}));
    let mut status = json!({ "main_stage_progress": "", "resume": "" });
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
        "monthly_sub_end",
        "last_online_ts",
        "friend_num_limit",
    ] {
        status[key] = json!(0);
    }
    status["register_ts"] = json!(1_600_000_000);
    let mut conn = ctx.db.conn().await.unwrap();
    sync_user_data(
        &mut conn,
        &SyncUserData {
            uid,
            server: Server::EN,
            nickname: "Doctor",
            level: 120,
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
            checkin,
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

async fn settings(ctx: &AppContext, owner: UserId, body: Value) {
    let request: UpdateSettingsRequest = serde_json::from_value(body).unwrap();
    update_settings(ctx, &common::user(ctx, owner, Role::User).await, &request)
        .await
        .unwrap();
}

async fn score(ctx: &AppContext, user: UserId) {
    let write = ScoreWrite {
        total_score: 1234.5,
        operator_score: 1000.0,
        stage_score: 100.0,
        roguelike_score: 50.0,
        sandbox_score: 10.0,
        medal_score: 20.0,
        base_score: 54.5,
        base_utilization: Some(0.8),
        base_infrastructure: None,
        skin_score: 0.0,
        grade: Some("A"),
    };
    scores::upsert(&mut ctx.db.conn().await.unwrap(), user, &write)
        .await
        .unwrap();
}

#[tokio::test]
async fn the_score_follows_the_profile_and_score_tab_gates() {
    let (ctx, db) = common::context().await;
    let owner = seed(&ctx, "100", &json!({}), &json!({ "history": [] })).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    assert!(
        get_user_score(&ctx, &visitor, "100")
            .await
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        get_user_score(&ctx, &visitor, "999").await,
        Err(AppError::NotFound(None))
    ));
    score(&ctx, owner).await;

    let seen = get_user_score(&ctx, &Subject::Anonymous, "100")
        .await
        .unwrap()
        .unwrap();
    let json = serde_json::to_value(&seen).unwrap();
    let mut keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "base_infrastructure",
            "base_score",
            "base_utilization",
            "calculated_at",
            "grade",
            "medal_score",
            "operator_score",
            "roguelike_score",
            "sandbox_score",
            "skin_score",
            "stage_score",
            "total_score",
            "user_id",
        ]
    );
    assert_eq!(json["user_id"], owner.0.to_string());
    assert_eq!(json["total_score"], 1234.5);
    assert_eq!(json["base_infrastructure"], Value::Null);

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "score", "visible": false }] } }),
    )
    .await;
    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            get_user_score(&ctx, subject, "100").await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(get_user_score(&ctx, &me, "100").await.unwrap().is_some());

    settings(&ctx, owner, json!({ "public_profile": false })).await;
    assert!(matches!(
        get_user_score(&ctx, &visitor, "100").await,
        Err(AppError::Forbidden)
    ));
    assert!(get_user_score(&ctx, &me, "100").await.unwrap().is_some());
    db.close().await;
}

#[tokio::test]
async fn a_visitor_loses_the_hidden_checkin_numbers_and_the_owner_keeps_them() {
    let (ctx, db) = common::context().await;
    let checkin = json!({
        "history": [1, 0, 1],
        "cumulative_signin": 40,
        "group_id": "signin7",
        "reward_index": 3,
        "can_check_in": true,
    });
    let owner = seed(&ctx, "200", &json!({}), &checkin).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(3, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    let open = get_user_checkin(&ctx, &visitor, "200")
        .await
        .unwrap()
        .unwrap();
    let json = serde_json::to_value(&open).unwrap();
    let mut keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "can_check_in",
            "checkin_group_id",
            "claimed_this_month",
            "cumulative_signin",
            "last_online_ts",
            "monthly_card_flags",
            "register_ts",
            "reward_index",
            "updated_at",
        ]
    );
    assert_eq!(json["monthly_card_flags"], json!([1, 0, 1]));
    assert_eq!(json["claimed_this_month"], 3);
    assert_eq!(json["cumulative_signin"], 40);
    assert_eq!(json["checkin_group_id"], "signin7");
    assert_eq!(json["register_ts"], 1_600_000_000);

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "header": { "hidden": ["sign_in", "registered"] } } }),
    )
    .await;
    let seen = get_user_checkin(&ctx, &visitor, "200")
        .await
        .unwrap()
        .unwrap();
    assert_eq!((seen.cumulative_signin, seen.register_ts), (None, None));
    assert_eq!(seen.monthly_card_flags, [1, 0, 1], "the calendar stays");
    let mine = get_user_checkin(&ctx, &me, "200").await.unwrap().unwrap();
    assert_eq!(
        (mine.cumulative_signin, mine.register_ts),
        (Some(40), Some(1_600_000_000))
    );

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "stats", "visible": false }] } }),
    )
    .await;
    assert!(matches!(
        get_user_checkin(&ctx, &visitor, "200").await,
        Err(AppError::Forbidden)
    ));
    assert!(get_user_checkin(&ctx, &me, "200").await.unwrap().is_some());

    settings(&ctx, owner, json!({ "public_profile": false })).await;
    assert!(matches!(
        get_user_checkin(&ctx, &Subject::Anonymous, "200").await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        get_user_checkin(&ctx, &visitor, "999").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn an_unsynced_checkin_reads_as_none() {
    let (ctx, db) = common::context().await;
    let owner = db.user(4, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    assert!(get_user_checkin(&ctx, &me, "4").await.unwrap().is_none());
    db.close().await;
}

#[tokio::test]
async fn stage_clears_are_keyed_by_stage_and_gated_by_the_profile() {
    let (ctx, db) = common::context().await;
    let stages = json!({
        "main_01-07": { "state": 3, "stateMax": 3, "completeTimes": 12, "practiceTimes": 2 },
        "main_02-01": { "state": 2, "inferred": "from-later-stage" },
    });
    let owner = seed(&ctx, "300", &stages, &json!({ "history": [] })).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(5, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    let mine = stage_clears(&ctx, &me, None).await.unwrap();
    let json = serde_json::to_value(&mine).unwrap();
    assert_eq!(
        json,
        json!({
            "main_01-07": {
                "state": 3, "stateMax": 3, "inferred": false,
                "completeTimes": 12, "practiceTimes": 2,
            },
            "main_02-01": {
                "state": 2, "stateMax": 2, "inferred": true,
                "completeTimes": 0, "practiceTimes": 0,
            },
        })
    );
    assert_eq!(
        serde_json::to_value(stage_clears(&ctx, &visitor, Some("300")).await.unwrap()).unwrap(),
        json
    );

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "roster", "visible": false }, { "id": "stats", "visible": false }] } }),
    )
    .await;
    assert!(
        stage_clears(&ctx, &visitor, Some("300")).await.is_ok(),
        "no tab gates the clears"
    );

    settings(&ctx, owner, json!({ "public_profile": false })).await;
    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            stage_clears(&ctx, subject, Some("300")).await,
            Err(AppError::Forbidden)
        ));
    }
    assert_eq!(stage_clears(&ctx, &me, Some("300")).await.unwrap().len(), 2);
    assert!(matches!(
        stage_clears(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    assert!(stage_clears(&ctx, &visitor, None).await.unwrap().is_empty());
    db.close().await;
}
