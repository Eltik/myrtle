#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;

use myrtle_app::AppError;
use myrtle_app::base_planner::{
    AccountFactsReq, DraftRoom, EvaluateRequest, OptimizeRequest, RotationRequest, build_pins,
    cached_plan, catalog, evaluate, layout, optimize, optimize_key, plan_cache_key, rotation,
    rotation_key, save_facts, trainer_value,
};
use myrtle_app::policy::Subject;
use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_db::repos::{roster, users};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const OPERATORS: [&str; 7] = [
    "char_002_amiya",
    "char_102_texas",
    "char_103_angel",
    "char_123_fang",
    "char_326_glacus",
    "char_101_sora",
    "char_017_huang",
];

fn draft() -> Vec<DraftRoom> {
    serde_json::from_value(json!([
        {"slot_id": "slot_34", "room_type": "CONTROL", "level": 1, "operators": ["char_002_amiya"]},
        {"slot_id": "slot_14", "room_type": "TRADING", "level": 2,
         "operators": ["char_102_texas", "char_103_angel"]},
        {"slot_id": "slot_15", "room_type": "MANUFACTURE", "level": 2,
         "operators": ["char_123_fang"], "formula_type": "F_GOLD"},
        {"slot_id": "slot_16", "room_type": "POWER", "level": 1, "operators": ["char_326_glacus"]},
        {"slot_id": "slot_9", "room_type": "DORMITORY", "level": 1,
         "operators": ["char_101_sora"], "comfort": 1000},
        {"slot_id": "slot_13", "room_type": "TRAINING", "level": 1}
    ]))
    .unwrap()
}

fn evaluate_request(layout: Vec<DraftRoom>) -> EvaluateRequest {
    EvaluateRequest {
        layout,
        claim_interval_hours: Some(8.0),
        ignore_promotion: false,
        facts: AccountFactsReq::default(),
    }
}

fn optimize_request() -> OptimizeRequest {
    OptimizeRequest {
        layout: draft(),
        scope: Vec::new(),
        locked: Vec::new(),
        excluded: Vec::new(),
        ignore_promotion: false,
        facts: AccountFactsReq::default(),
    }
}

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
            "slot_9": slot("DORMITORY", 1, &["char_101_sora"]),
            "slot_13": slot("TRAINING", 1, &[]),
            "slot_20": {"roomId": "DORMITORY", "level": 0, "state": 0}
        },
        "rooms": {
            "MANUFACTURE": {"slot_15": {"formulaId": "4"}},
            "TRADING": {"slot_14": {"presetQueue": [[2, 3], [7]]}},
            "POWER": {"slot_16": {"presetQueue": [[], []]}},
            "DORMITORY": {"slot_9": {"comfort": 1000}}
        },
        "status": {"labor": {"value": 10, "maxValue": 100, "buffSpeed": 0.0,
                             "lastUpdateTime": now - 7200}}
    })
}

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

async fn make_private(ctx: &myrtle_app::AppContext, owner: UserId) {
    let request: UpdateSettingsRequest =
        serde_json::from_value(json!({"public_profile": false})).unwrap();
    update_settings(ctx, &common::user(ctx, owner, Role::User).await, &request)
        .await
        .unwrap();
}

#[test]
fn trainer_value_reads_class_and_agnostic_texts() {
    let agnostic = "Operators' Specialization training speed +25%";
    assert_eq!(trainer_value(agnostic, "Sniper"), Some(25.0));
    let sniper = "When training a Sniper, Specialization training speed +30%";
    assert_eq!(trainer_value(sniper, "Sniper"), Some(30.0));
    assert_eq!(trainer_value(sniper, "Guard"), None);
    let scaled = "Specialization training speed +5% for each Sniper Operator";
    assert_eq!(trainer_value(scaled, "Sniper"), None);
    assert_eq!(trainer_value("Morale +1", "Guard"), None);
    let signed = "Specialization training speed +x, +12.5%";
    assert_eq!(trainer_value(signed, "Caster"), Some(12.5));
}

#[test]
fn pins_cover_frozen_rooms_unmodelled_rooms_and_locked_operators() {
    let layout = draft();
    let mut pins = build_pins(&layout, &[], &["char_123_fang".into()]);
    pins.sort();
    assert_eq!(
        pins,
        [("char_123_fang".to_owned(), "MANUFACTURE".to_owned())]
    );

    let mut layout = draft();
    layout[5].operators = vec!["char_017_huang".into()];
    let scoped = build_pins(&layout, &["slot_14".into()], &[]);
    let pinned: Vec<&str> = scoped.iter().map(|(id, _)| id.as_str()).collect();
    assert!(pinned.contains(&"char_017_huang"));
    assert!(pinned.contains(&"char_002_amiya"));
    assert!(!pinned.contains(&"char_102_texas"));
    assert!(!pinned.contains(&"char_103_angel"));
}

#[test]
fn equal_requests_share_one_cache_key() {
    let key = optimize_key("77", &optimize_request());
    assert_eq!(key, optimize_key("77", &optimize_request()));
    assert!(key.starts_with("base:optimize:77:"));
    assert_eq!(key.len(), "base:optimize:77:".len() + 16);
    assert_ne!(key, optimize_key("78", &optimize_request()));
    let mut locked = optimize_request();
    locked.locked.push("char_102_texas".into());
    assert_ne!(key, optimize_key("77", &locked));

    let rotation_request = RotationRequest {
        layout: draft(),
        locked: Vec::new(),
        excluded: Vec::new(),
        ignore_promotion: false,
        facts: AccountFactsReq::default(),
    };
    let key = rotation_key("77", &rotation_request);
    assert!(key.starts_with("base:rotation:77:"));
    assert_eq!(
        key,
        rotation_key(
            "77",
            &serde_json::from_value(json!(rotation_request)).unwrap()
        )
    );
}

#[tokio::test]
async fn facts_save_and_read_back_through_the_layout() {
    let (ctx, db) = common::context().await;
    let me = db.user(1, Server::EN).await;
    let subject = common::user(&ctx, me, Role::User).await;

    let empty = layout(&ctx, &subject, None).await.unwrap();
    assert!(empty.rooms.is_empty() && empty.presets.is_empty());
    assert!(empty.facts.open_recruit_slots.is_none());

    let facts = AccountFactsReq {
        open_recruit_slots: Some(2),
        training_class: Some("Sniper".into()),
    };
    let saved = save_facts(&ctx, &subject, facts).await.unwrap();
    assert_eq!(saved.open_recruit_slots, Some(2));
    let read = layout(&ctx, &subject, None).await.unwrap().facts;
    assert_eq!(read.open_recruit_slots, Some(2));
    assert_eq!(read.training_class.as_deref(), Some("Sniper"));

    assert!(matches!(
        save_facts(&ctx, &Subject::Anonymous, AccountFactsReq::default()).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn reads_of_another_player_follow_their_privacy() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let me = common::user(&ctx, owner, Role::User).await;
    save_facts(&ctx, &me, AccountFactsReq::default())
        .await
        .unwrap();

    assert!(matches!(
        layout(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    assert!(layout(&ctx, &visitor, Some("1")).await.is_ok());
    assert!(matches!(
        layout(&ctx, &visitor, Some("404")).await,
        Err(AppError::NotFound(_))
    ));

    make_private(&ctx, owner).await;
    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            layout(&ctx, subject, Some("1")).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            optimize(&ctx, subject, Some("1"), optimize_request()).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            evaluate(&ctx, subject, Some("1"), evaluate_request(draft())).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(layout(&ctx, &me, Some("1")).await.is_ok());
    assert!(matches!(
        optimize(&ctx, &Subject::Anonymous, None, optimize_request()).await,
        Err(AppError::Unauthorized)
    ));
    let gone = common::user(&ctx, UserId(uuid::Uuid::from_u128(404)), Role::User).await;
    assert!(
        matches!(
            optimize(&ctx, &gone, None, optimize_request()).await,
            Err(AppError::Unauthorized)
        ),
        "a deleted account's own plan"
    );
    db.close().await;
}

#[tokio::test]
async fn nonsense_layouts_are_refused_by_name() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let refused = |mutate: fn(&mut Vec<DraftRoom>), needle: &'static str| {
        let ctx = ctx.clone();
        let me = me.clone();
        async move {
            let mut layout = draft();
            mutate(&mut layout);
            match evaluate(&ctx, &me, None, evaluate_request(layout)).await {
                Err(AppError::BadRequest(message)) => {
                    assert!(message.contains(needle), "{message}");
                }
                other => panic!("expected a refusal naming {needle}, got {other:?}"),
            }
        }
    };
    refused(Vec::clear, "layout is empty").await;
    refused(
        |l| {
            let dorm = l[4].clone();
            l.extend((0..40).map(|i| DraftRoom {
                slot_id: format!("x{i}"),
                ..dorm.clone()
            }));
        },
        "more than the 40",
    )
    .await;
    refused(
        |l| l[1].slot_id = "slot_34".into(),
        "duplicate slot 'slot_34'",
    )
    .await;
    refused(
        |l| l[0].room_type = "CASINO".into(),
        "unknown room type 'CASINO'",
    )
    .await;
    refused(|l| l[3].level = 0, "outside 1..=3").await;
    refused(|l| l[3].level = 4, "outside 1..=3").await;
    refused(
        |l| l[3].operators.push("char_017_huang".into()),
        "only 1 seats",
    )
    .await;
    refused(
        |l| {
            let mut second = l[0].clone();
            second.slot_id = "slot_99".into();
            second.operators.clear();
            l.push(second);
        },
        "2 'CONTROL' rooms",
    )
    .await;
    refused(
        |l| l[5].operators = vec!["char_102_texas".into()],
        "'char_102_texas' is stationed in more than one room",
    )
    .await;
    db.close().await;
}

#[tokio::test]
async fn layout_and_evaluate_read_the_seeded_base() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, seed(&ctx, "77").await, Role::User).await;

    let drafted = layout(&ctx, &me, None).await.unwrap();
    let slots: Vec<&str> = drafted.rooms.iter().map(|r| r.slot_id.as_str()).collect();
    assert_eq!(
        slots,
        [
            "slot_13", "slot_14", "slot_15", "slot_16", "slot_34", "slot_9"
        ]
    );
    let factory = drafted
        .rooms
        .iter()
        .find(|r| r.slot_id == "slot_15")
        .unwrap();
    assert_eq!(factory.formula_type.as_deref(), Some("F_GOLD"));
    assert_eq!(
        drafted.presets.len(),
        1,
        "the all-empty power queue is dropped"
    );
    assert_eq!(
        drafted.presets[0].shifts,
        [
            vec!["char_102_texas".to_owned(), "char_103_angel".to_owned()],
            vec!["char_017_huang".to_owned()]
        ]
    );

    let mut request = evaluate_request(draft());
    request.facts.training_class = Some("Guard".into());
    let scored = evaluate(&ctx, &me, None, request).await.unwrap();
    let trading = scored
        .assignment
        .rooms
        .iter()
        .find(|r| r.slot_id == "slot_14")
        .unwrap();
    assert!(trading.total_efficiency > 0.0);
    assert!(scored.assignment.yield_lmd_per_day > 0.0);
    assert_eq!(
        scored.power.net,
        scored.power.generated - scored.power.consumed
    );
    assert!(scored.power.generated > 0 && scored.power.consumed > 0);
    assert_eq!(scored.dorms.count, 1);
    assert_eq!(scored.dorms.per_dorm[0].comfort, 1000);
    assert!(scored.sustain.iter().all(|s| s.room_type != "DORMITORY"));
    let texas = scored
        .sustain
        .iter()
        .find(|s| s.operator_id == "char_102_texas")
        .unwrap();
    assert!(texas.morale.is_some_and(|m| m < 24.0));
    assert!(scored.morale_synced_hours_ago.is_some_and(|h| h >= 1.0));
    let drones = scored.drones.unwrap();
    assert_eq!(drones.max, 100);
    assert!(drones.current > 10.0 && drones.current < 100.0);
    let claim = scored.claim.unwrap();
    let cadences: Vec<f64> = claim.intervals.iter().map(|i| i.hours).collect();
    assert_eq!(cadences, [6.0, 8.0, 12.0, 24.0]);
    assert!(scored.unrotated.is_some());
    db.close().await;
}

#[tokio::test]
async fn optimize_proposes_and_caches_the_search() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, seed(&ctx, "77").await, Role::User).await;
    let mut request = optimize_request();
    request.layout[1].operators.clear();

    let first = optimize(&ctx, &me, None, request.clone()).await.unwrap();
    let body: Value = serde_json::from_str(&first.body).unwrap();
    let proposal = &body["proposal"]["rooms"];
    let trading = proposal
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["slot_id"] == "slot_14")
        .unwrap();
    assert!(!trading["operators"].as_array().unwrap().is_empty());
    assert!(
        body["room_diffs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["slot_id"] == "slot_14")
    );
    assert!(body["power"]["generated"].as_i64().unwrap() > 0);

    let key = snapshot.key(&optimize_key("77", &request));
    assert!(ctx.cache.get(&key).await.is_some());
    let again = optimize(&ctx, &me, None, request.clone()).await.unwrap();
    assert_eq!(again.etag, first.etag);

    let mut scoped = request;
    scoped.scope = vec!["slot_14".into()];
    let scoped = optimize(&ctx, &me, None, scoped).await.unwrap();
    let body: Value = serde_json::from_str(&scoped.body).unwrap();
    let changed: Vec<&Value> = body["room_diffs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| &d["slot_id"])
        .collect();
    assert_eq!(changed, [&json!("slot_14")]);
    db.close().await;
}

#[tokio::test]
async fn rotation_plans_three_shifts() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, seed(&ctx, "77").await, Role::User).await;
    let request = RotationRequest {
        layout: draft(),
        locked: Vec::new(),
        excluded: Vec::new(),
        ignore_promotion: false,
        facts: AccountFactsReq::default(),
    };
    let planned = rotation(&ctx, &me, None, request.clone()).await.unwrap();
    let body: Value = serde_json::from_str(&planned.body).unwrap();
    assert_eq!(body["shift_count"], 3);
    assert_eq!(body["rotation"]["shifts"].as_array().unwrap().len(), 3);
    assert!(body["rotation"]["sustainability"]["verdict"].is_string());
    let comparison = &body["comparison"];
    assert!(comparison["recommended"]["total_value"].as_f64().unwrap() > 0.0);
    assert!(
        comparison["matching_cells"].as_u64().unwrap()
            <= comparison["production_cells"].as_u64().unwrap()
    );
    let saved_crew = body["rotation"]["shifts"][0]["rooms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["slot_id"] == "slot_14")
        .unwrap();
    assert!(saved_crew["current"].is_array());
    assert!(
        ctx.cache
            .get(&snapshot.key(&rotation_key("77", &request)))
            .await
            .is_some()
    );
    db.close().await;
}

#[tokio::test]
async fn catalog_lists_rooms_formulas_and_the_floorplan() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let catalog = catalog(&ctx).unwrap();
    assert_eq!(catalog.shift_count, 3);
    let trading = catalog
        .rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .unwrap();
    assert_eq!(trading.phases.len(), 3);
    assert_eq!(trading.phases[0].level, 1);
    let formulas: Vec<&str> = catalog
        .formulas
        .iter()
        .map(|f| f.formula_type.as_str())
        .collect();
    assert_eq!(formulas, ["F_DIAMOND", "F_EXP", "F_GOLD"]);
    assert!(catalog.slots.iter().any(|s| s.slot_id == "slot_34"));
    assert!(
        catalog
            .slots
            .windows(2)
            .all(|w| w[0].offset_row >= w[1].offset_row)
    );
    assert!(!catalog.storeys.is_empty());
    db.close().await;
}

#[test]
fn plan_keys_ignore_order_and_split_on_what_the_search_reads() {
    let facts = AccountFactsReq::default();
    let key = |layout: &[DraftRoom], scope: &[String]| {
        plan_cache_key("77", layout, scope, &[], &[], false, &facts)
    };
    let mut reordered = draft();
    reordered.reverse();
    reordered[4].operators.reverse();
    assert_eq!(key(&draft(), &[]), key(&reordered, &[]));
    assert!(key(&draft(), &[]).starts_with("77|false|0|||||"));
    assert_ne!(key(&draft(), &[]), key(&draft(), &["slot_14".into()]));
    let mut moved = draft();
    moved[1].operators.pop();
    assert_ne!(key(&draft(), &[]), key(&moved, &[]));
}

#[tokio::test]
async fn a_rotation_after_an_optimize_reuses_its_search() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let me = common::user(&ctx, seed(&ctx, "79").await, Role::User).await;
    let key = snapshot.key(&plan_cache_key(
        "79",
        &draft(),
        &[],
        &[],
        &[],
        false,
        &AccountFactsReq::default(),
    ));
    assert!(cached_plan(&key).is_none());
    optimize(&ctx, &me, None, optimize_request()).await.unwrap();
    let searched = cached_plan(&key).expect("optimize keeps its plan");
    let request = RotationRequest {
        layout: draft(),
        locked: Vec::new(),
        excluded: Vec::new(),
        ignore_promotion: false,
        facts: AccountFactsReq::default(),
    };
    rotation(&ctx, &me, None, request).await.unwrap();
    assert!(
        Arc::ptr_eq(&searched, &cached_plan(&key).unwrap()),
        "a second search would have replaced the plan"
    );
    db.close().await;
}
