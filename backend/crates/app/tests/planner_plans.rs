#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::planner::{self, UpsertPlanRequest, slim_plan_operator};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::{gacha, planner as repo, roster};
use myrtle_engine_planner::PresetTarget;
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const AMIYA: &str = "char_002_amiya";

fn slim(v: Value) -> Value {
    let Value::Object(mut map) = v else {
        panic!("fixture must be an object")
    };
    slim_plan_operator(&mut map);
    Value::Object(map)
}

fn plan(elite: i16, level: i16, skill_level: i16, groups: Option<&[&str]>) -> UpsertPlanRequest {
    UpsertPlanRequest {
        target_elite: elite,
        target_level: level,
        target_skill_level: skill_level,
        target_skills: json!([]),
        target_modules: json!([]),
        display_on_profile: true,
        groups: groups.map(|g| g.iter().map(|s| (*s).to_owned()).collect()),
    }
}

const fn preset(elite: i16, skill_level: i16, masteries: [i16; 3]) -> PresetTarget {
    PresetTarget {
        elite,
        level: None,
        skill_level,
        masteries,
        module_stage: 0,
        display_on_profile: false,
    }
}

async fn settings(ctx: &AppContext, owner: UserId, body: Value) {
    let mut conn = ctx.db.conn().await.unwrap();
    gacha::update_flags(&mut conn, owner, None, None)
        .await
        .unwrap();
    drop(conn);
    let request: UpdateSettingsRequest = serde_json::from_value(body).unwrap();
    update_settings(ctx, &common::user(ctx, owner, Role::User).await, &request)
        .await
        .unwrap();
}

async fn seed_plan(ctx: &AppContext, owner: UserId, operator_id: &str, shown: bool) {
    let mut conn = ctx.db.conn().await.unwrap();
    let skills = json!([]);
    let input = repo::PlanInput {
        target_elite: 2,
        target_level: 1,
        target_skill_level: 7,
        target_skills: &skills,
        target_modules: &skills,
        display_on_profile: shown,
    };
    repo::upsert_plan(&mut conn, owner, operator_id, input, None)
        .await
        .unwrap();
}

async fn sync_roster(ctx: &AppContext) {
    let op = json!({
        "operator_id": AMIYA, "elite": 2, "level": 50, "exp": 0, "potential": 0,
        "skill_level": 7, "favor_point": 100, "skin_id": null, "default_skill": 0,
        "voice_lan": "JP", "current_equip": null, "current_tmpl": null, "obtained_at": 1
    });
    let mut status = json!({});
    for key in [
        "exp",
        "orundum",
        "orundum_shard",
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
    ] {
        status[key] = json!(0);
    }
    status["lmd"] = json!(900_000);
    let operators = json!([op]);
    let items = json!([{ "item_id": "2004", "quantity": 10 }]);
    let list = json!([]);
    let map = json!({});
    let mut conn = ctx.db.conn().await.unwrap();
    roster::sync_user_data(
        &mut conn,
        &roster::SyncUserData {
            uid: "1",
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
            items: &items,
            skins: &list,
            status: &status,
            stages: &map,
            roguelike: &list,
            sandbox: &map,
            medals: &list,
            building: &map,
            checkin: &map,
            supports: &list,
            nick_number: Some("1234"),
            enemies: &map,
            game_squads: &list,
        },
    )
    .await
    .unwrap();
}

#[test]
fn slim_empties_and_nulls_the_unread_fields_that_are_present() {
    let out = slim(json!({
        "name": "Amiya",
        "talents": [{ "x": 1 }],
        "potentialRanks": [1, 2],
        "baseSkills": [1],
        "handbook": { "big": true },
        "profile": "text",
    }));
    assert_eq!(
        out,
        json!({
            "name": "Amiya",
            "talents": [],
            "potentialRanks": [],
            "baseSkills": [],
            "handbook": null,
            "profile": null,
        })
    );
    assert_eq!(slim(json!({ "name": "Amiya" })), json!({ "name": "Amiya" }));
}

#[test]
fn slim_keeps_the_first_skill_level_and_strips_module_data() {
    let out = slim(json!({
        "skills": [
            { "skillId": "s1", "static": { "Levels": [{ "Name": "L1" }, { "Name": "L2" }] } },
            { "skillId": "s3", "static": null },
            "not an object",
        ],
        "modules": [
            { "uniEquipId": "a", "data": { "phases": [1] }, "uniEquipDesc": "long text" },
            { "uniEquipId": "b" },
            7,
        ],
    }));
    assert_eq!(
        out,
        json!({
            "skills": [
                { "skillId": "s1", "static": { "Levels": [{ "Name": "L1" }] } },
                { "skillId": "s3", "static": null },
                "not an object",
            ],
            "modules": [
                { "uniEquipId": "a", "data": null, "uniEquipDesc": "" },
                { "uniEquipId": "b", "data": null, "uniEquipDesc": "" },
                7,
            ],
        })
    );
}

#[tokio::test]
async fn plan_writes_refuse_bad_input() {
    let (ctx, db) = common::context().await;
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;

    assert!(matches!(
        planner::upsert_plan(&ctx, &me, "char_4195_radian", plan(0, 1, 1, None)).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        planner::upsert_plan(&ctx, &Subject::Anonymous, AMIYA, plan(0, 1, 1, None)).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        planner::upsert_plan(&ctx, &me, AMIYA, plan(0, 1, 1, None)).await,
        Err(AppError::NotFound(None))
    ));
    for tier in [0, 6] {
        assert!(matches!(
            planner::list_plans(&ctx, &me, None, Some(tier), false).await,
            Err(AppError::BadRequest(_))
        ));
    }

    let too_many: Vec<String> = (0..1001).map(|i| format!("char_{i}")).collect();
    assert!(matches!(
        planner::delete_plans(&ctx, &me, &too_many).await,
        Err(AppError::BadRequest(_))
    ));
    assert_eq!(
        planner::delete_plans(&ctx, &me, &[]).await.unwrap().deleted,
        0
    );

    let owner = me.user_id().unwrap();
    seed_plan(&ctx, owner, AMIYA, true).await;
    seed_plan(&ctx, owner, "char_003_kalts", true).await;
    let ids = vec![AMIYA.to_owned(), "char_missing".to_owned()];
    assert_eq!(
        planner::delete_plans(&ctx, &me, &ids)
            .await
            .unwrap()
            .deleted,
        1
    );
    planner::delete_plan(&ctx, &me, "char_003_kalts")
        .await
        .unwrap();
    planner::delete_plan(&ctx, &me, "char_003_kalts")
        .await
        .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    assert!(repo::list_plans(&mut conn, owner).await.unwrap().is_empty());
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn group_rules() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;

    let first = planner::create_group(&ctx, &me, "Farm").await.unwrap();
    let again = planner::create_group(&ctx, &me, "Farm").await.unwrap();
    assert_eq!(first.id, again.id);
    planner::create_group(&ctx, &me, "Later").await.unwrap();

    assert!(matches!(
        planner::update_group(&ctx, &me, "Farm", None, None).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        planner::update_group(&ctx, &me, "Nope", None, Some(true)).await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        planner::update_group(&ctx, &me, "Farm", Some("Later"), None).await,
        Err(AppError::Conflict(_))
    ));

    let renamed = planner::update_group(&ctx, &me, "Later", Some("Now"), Some(true))
        .await
        .unwrap();
    assert_eq!((renamed.name.as_str(), renamed.pinned), ("Now", true));

    let mut conn = ctx.db.conn().await.unwrap();
    let names: Vec<String> = repo::list_groups(&mut conn, owner)
        .await
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect();
    assert_eq!(names, ["Now", "Farm"]);
    drop(conn);

    planner::delete_group(&ctx, &me, "Now").await.unwrap();
    planner::delete_group(&ctx, &me, "Now").await.unwrap();
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    planner::delete_group(&ctx, &other, "Farm").await.unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    assert_eq!(repo::list_groups(&mut conn, owner).await.unwrap().len(), 1);
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn preset_rules() {
    let (ctx, db) = common::context().await;
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;

    for (name, target) in [
        ("   ", preset(2, 7, [0; 3])),
        (&"x".repeat(101) as &str, preset(2, 7, [0; 3])),
        ("bad elite", preset(3, 7, [0; 3])),
        ("mastery early", preset(1, 7, [3, 0, 0])),
        ("skill early", preset(0, 7, [0; 3])),
    ] {
        assert!(
            matches!(
                planner::upsert_preset(&ctx, &me, name, &target).await,
                Err(AppError::BadRequest(_))
            ),
            "{name} should be refused"
        );
    }

    let saved = planner::upsert_preset(&ctx, &me, "  M3  ", &preset(2, 7, [3, 3, 3]))
        .await
        .unwrap();
    assert_eq!(saved.name, "M3");
    let replaced = planner::upsert_preset(&ctx, &me, "M3", &preset(2, 7, [3, 0, 0]))
        .await
        .unwrap();
    assert_eq!(replaced.id, saved.id);
    planner::upsert_preset(&ctx, &me, "E1", &preset(1, 4, [0; 3]))
        .await
        .unwrap();

    let listed = planner::list_presets(&ctx, &me).await.unwrap();
    let names: Vec<&str> = listed.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["E1", "M3"]);
    assert_eq!(listed[1].target.masteries, [3, 0, 0]);

    planner::delete_preset(&ctx, &me, "M3").await.unwrap();
    planner::delete_preset(&ctx, &me, "M3").await.unwrap();
    assert_eq!(planner::list_presets(&ctx, &me).await.unwrap().len(), 1);
    assert!(matches!(
        planner::list_presets(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn public_plans_gate() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;
    let by_id = owner.0.to_string();

    for subject in [&visitor, &Subject::Anonymous, &me] {
        planner::list_public_plans(&ctx, subject, "1")
            .await
            .unwrap();
        planner::list_public_plans(&ctx, subject, &by_id)
            .await
            .unwrap();
    }
    assert!(matches!(
        planner::list_public_plans(&ctx, &visitor, "999").await,
        Err(AppError::NotFound(None))
    ));

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "plans", "visible": false }] } }),
    )
    .await;
    for uid in ["1", by_id.as_str()] {
        assert!(matches!(
            planner::list_public_plans(&ctx, &visitor, uid).await,
            Err(AppError::Forbidden)
        ));
        planner::list_public_plans(&ctx, &me, uid).await.unwrap();
    }

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": null, "public_profile": false }),
    )
    .await;
    for subject in [&visitor, &Subject::Anonymous] {
        for uid in ["1", by_id.as_str()] {
            assert!(matches!(
                planner::list_public_plans(&ctx, subject, uid).await,
                Err(AppError::Forbidden)
            ));
        }
    }
    planner::list_public_plans(&ctx, &me, "1").await.unwrap();
    db.close().await;
}

#[tokio::test]
async fn plans_and_requirements_on_a_seeded_roster() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    sync_roster(&ctx).await;
    let cap = i16::try_from(snapshot.model.operators[AMIYA].phases[2].max_level).unwrap();

    let raised = planner::upsert_plan(&ctx, &me, AMIYA, plan(1, 1, 1, Some(&["b", "a", "a"])))
        .await
        .unwrap();
    assert_eq!(
        (
            raised.target_elite,
            raised.target_level,
            raised.target_skill_level
        ),
        (2, 50, 7)
    );
    assert!(raised.met);
    assert_eq!(raised.groups, ["a", "b"]);
    assert_eq!(raised.operator["server"], "en");
    assert_eq!(raised.operator["talents"], json!([]));

    assert!(matches!(
        planner::upsert_plan(&ctx, &me, AMIYA, plan(2, cap + 1, 7, None)).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        planner::upsert_plan(&ctx, &me, AMIYA, plan(5, 1, 7, None)).await,
        Err(AppError::BadRequest(_))
    ));

    let open = planner::upsert_plan(&ctx, &me, AMIYA, plan(2, cap, 7, None))
        .await
        .unwrap();
    assert!(!open.met);
    assert_eq!(open.groups, ["a", "b"]);

    let listed = planner::list_plans(&ctx, &me, None, None, true)
        .await
        .unwrap();
    assert_eq!(listed.plans.len(), 1);
    assert!(listed.last_synced_at.is_some());
    assert_eq!(listed.groups.len(), 2);
    let lmd = listed
        .aggregated_requirements
        .iter()
        .find(|i| i.id == "4001")
        .unwrap();
    assert!(lmd.required_count > 0);
    assert_eq!(lmd.inventory_count, 900_000);
    assert!(
        listed
            .aggregated_requirements
            .iter()
            .any(|i| i.id == "5001" && i.inventory_count > 0)
    );
    let by_operator = listed.operator_requirements.unwrap();
    assert!(!by_operator[AMIYA].is_empty());

    let none = planner::list_plans(&ctx, &me, Some("none"), Some(3), false)
        .await
        .unwrap();
    assert!(none.aggregated_requirements.is_empty());
    assert!(none.operator_requirements.is_none());

    settings(&ctx, owner, json!({ "public_profile": true })).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let shown = planner::list_public_plans(&ctx, &visitor, "1")
        .await
        .unwrap();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].operator_id, AMIYA);
    db.close().await;
}
