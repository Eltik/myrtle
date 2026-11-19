#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::services::improvements::{
    base_improvements, cache_key, improvements, layout_summary,
};
use myrtle_app::{AppError, Subject};
use myrtle_db::repos::roster;
use myrtle_db::repos::users::{self, LayoutUpdate, SettingsUpdate};
use myrtle_engine_base::types::{UserBuilding, UserRoom};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const OPERATORS: [&str; 5] = [
    "char_002_amiya",
    "char_102_texas",
    "char_103_angel",
    "char_123_fang",
    "char_326_glacus",
];

fn building_json(now: i64) -> Value {
    let inst = |id: &str| OPERATORS.iter().position(|o| *o == id).unwrap() + 1;
    let chars: serde_json::Map<String, Value> = OPERATORS
        .iter()
        .map(|id| {
            (
                inst(id).to_string(),
                json!({"charId": id, "ap": 6_480_000, "lastApAddTime": now - 3600}),
            )
        })
        .collect();
    let slot = |room: &str, level: i32, ops: &[&str]| {
        json!({"roomId": room, "level": level, "state": 2,
               "charInstIds": ops.iter().map(|o| inst(o)).collect::<Vec<_>>()})
    };
    json!({
        "chars": chars,
        "roomSlots": {
            "slot_34": slot("CONTROL", 1, &["char_002_amiya"]),
            "slot_14": slot("TRADING", 2, &["char_102_texas", "char_103_angel"]),
            "slot_15": slot("MANUFACTURE", 2, &["char_123_fang"]),
            "slot_16": slot("POWER", 1, &["char_326_glacus"]),
            "slot_9": slot("DORMITORY", 1, &[]),
        },
        "rooms": { "MANUFACTURE": {"slot_15": {"formulaId": "4"}} }
    })
}

/// A synced public player at E2 L60 with a five-room base
async fn seed(ctx: &myrtle_app::AppContext, uid: &str) -> UserId {
    let now = chrono::Utc::now().timestamp();
    let operators: Vec<Value> = OPERATORS
        .iter()
        .map(|id| {
            json!({
                "operator_id": id, "elite": 2, "level": 60, "exp": 0, "potential": 0,
                "skill_level": 7, "favor_point": 100, "skin_id": format!("{id}#1"),
                "default_skill": 0, "voice_lan": "JP", "current_equip": null,
                "current_tmpl": null, "obtained_at": 1_600_000_000
            })
        })
        .collect();
    let operators = Value::Array(operators);
    let building = building_json(now);
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
            level: 120,
            avatar_id: None,
            secretary: None,
            secretary_skin_id: None,
            resume_id: None,
            operators: &operators,
            skills: &list,
            modules: &list,
            items: &list,
            skins: &list,
            status: &status,
            stages: &map,
            roguelike: &list,
            sandbox: &map,
            medals: &list,
            building: &building,
            checkin: &map,
            supports: &list,
            nick_number: Some("1234"),
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

async fn settings(
    ctx: &myrtle_app::AppContext,
    user: UserId,
    public: bool,
    layout: LayoutUpdate<'_>,
) {
    let mut conn = ctx.db.conn().await.unwrap();
    users::set_base_facts(&mut conn, user, &json!({}))
        .await
        .unwrap(); // update_settings needs the row
    users::update_settings(
        &mut conn,
        user,
        &SettingsUpdate {
            public_profile: Some(public),
            store_gacha: None,
            share_stats: None,
            public_gacha: None,
            layout,
        },
    )
    .await
    .unwrap();
}

#[test]
fn the_cache_key_is_uid_and_sync_millis() {
    assert_eq!(
        cache_key("123456", 1_700_000_000_123),
        "improvements:123456:1700000000123"
    );
}

#[test]
fn layout_summary_groups_by_type_levels_highest_first() {
    let room = |slot: &str, room_type: &str, level| UserRoom {
        slot_id: slot.into(),
        room_type: room_type.into(),
        level,
        current_operators: Vec::new(),
        current_formula: None,
        preset_shifts: Vec::new(),
        comfort: 0,
        frozen: false,
    };
    let building = UserBuilding {
        rooms: vec![
            room("a", "TRADING", 2),
            room("b", "MANUFACTURE", 1),
            room("c", "TRADING", 3),
        ],
        drone_capacity: None,
    };
    let layout = serde_json::to_value(layout_summary(&building)).unwrap();
    assert_eq!(
        layout,
        json!([
            {"room_type": "MANUFACTURE", "count": 1, "levels": [1]},
            {"room_type": "TRADING", "count": 2, "levels": [3, 2]}
        ])
    );
}

#[tokio::test]
async fn reads_follow_profile_privacy_and_the_score_tab() {
    let (ctx, db) = common::context().await;
    let owner = db.user(7101, Server::EN).await;
    let own = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(7102, Server::EN).await, Role::User).await;
    let past_gate = |r: Result<_, AppError>| matches!(r, Err(AppError::Unavailable)); // no game data loaded

    assert!(matches!(
        improvements(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        improvements(&ctx, &visitor, Some("7101")).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        improvements(&ctx, &visitor, Some("9999")).await,
        Err(AppError::NotFound(_))
    ));
    assert!(past_gate(improvements(&ctx, &own, None).await));
    assert!(past_gate(improvements(&ctx, &own, Some("7101")).await));

    let hide = json!({ "tabs": [{ "id": "score", "visible": false }] });
    settings(
        &ctx,
        owner,
        true,
        LayoutUpdate::Merge {
            patch: &hide,
            removed: &[],
        },
    )
    .await;
    assert!(matches!(
        improvements(&ctx, &visitor, Some("7101")).await,
        Err(AppError::Forbidden)
    ));
    assert!(past_gate(improvements(&ctx, &own, Some("7101")).await));

    settings(&ctx, owner, true, LayoutUpdate::Clear).await;
    assert!(past_gate(improvements(&ctx, &visitor, Some("7101")).await));
    assert!(past_gate(
        improvements(&ctx, &Subject::Anonymous, Some("7101")).await
    ));
    db.close().await;
}

#[tokio::test]
async fn a_synced_player_gets_every_section_once_per_sync() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = seed(&ctx, "7201").await;
    settings(&ctx, owner, true, LayoutUpdate::Keep).await;
    let own = common::user(&ctx, owner, Role::User).await;

    let first = improvements(&ctx, &own, None).await.unwrap();
    let body: Value = serde_json::from_str(&first.body).unwrap();
    assert_eq!(body["uid"], "7201");
    assert!(body["stages"]["permanent"]["missing"].is_array()); // no known stages without a stage sync
    assert!(body["roguelike"].as_array().unwrap().len() >= 4);
    let gaps = body["operators"]["below_milestone"].as_array().unwrap();
    assert!(gaps.iter().any(|g| {
        g["operator_id"] == "char_102_texas"
            && g["missing"]
                .as_array()
                .unwrap()
                .contains(&json!("MAX_LEVEL"))
    }));
    assert!(
        !body["operators"]["score_breakdown"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let base = &body["base"];
    assert_eq!(
        base["current"]["rooms"].as_array().unwrap().len(),
        4,
        "dorms are not staffed rooms"
    );
    assert!(base["optimal"].is_object() && base["rotation"].is_object());
    assert!(
        base["shift_rotation"].is_null(),
        "one factory is not a shift-rotation layout"
    );
    assert_eq!(base["layout"].as_array().unwrap().len(), 5);
    assert!(!snapshot.grade.medals.medals.is_empty(), "EN has medals");
    let medal_gaps = |list: &str| body["medals"][list].as_array().unwrap().len();
    assert!(
        medal_gaps("permanent_missing") > 0,
        "a player with no medal synced misses every permanent one"
    );
    assert!(
        medal_gaps("permanent_missing")
            + medal_gaps("event_in_window_missing")
            + medal_gaps("operator_locked")
            + medal_gaps("unobtainable_missing")
            <= snapshot.grade.medals.medals.len()
    );

    let visitor = common::user(&ctx, db.user(7202, Server::EN).await, Role::User).await;
    let seen = improvements(&ctx, &visitor, Some("7201")).await.unwrap();
    assert_eq!(seen.etag, first.etag, "one cached body per sync");
    db.close().await;
}

#[test]
fn an_unbuilt_base_has_no_assignments() {
    let Some(snapshot) = common::snapshot() else {
        return;
    };
    let base = base_improvements(&[], &snapshot.base, &json!({}), None);
    assert!(base.current.is_none() && base.optimal.is_none() && base.layout.is_empty());
}
