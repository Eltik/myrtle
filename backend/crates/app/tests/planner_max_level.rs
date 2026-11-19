#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::planner::max_level::{
    LevelTarget, days_to_earn, max_level_cost, module_targets_by_rarity, operator_target,
};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::{gacha, roster};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const AMIYA: &str = "char_002_amiya";

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

/// Amiya at E1 40 with 500k LMD and ten Advanced Battle Records, on uid "1"
async fn sync_roster(ctx: &AppContext) {
    let operators = json!([{
        "operator_id": AMIYA, "elite": 1, "level": 40, "exp": 0, "potential": 0,
        "skill_level": 4, "favor_point": 0, "skin_id": null, "default_skill": 0,
        "voice_lan": "JP", "current_equip": null, "current_tmpl": null, "obtained_at": 1
    }]);
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
    status["lmd"] = json!(500_000);
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
fn days_round_up_and_handle_the_edges() {
    assert_eq!(days_to_earn(0, 100.0), Some(0));
    assert_eq!(days_to_earn(-5, 0.0), Some(0));
    assert_eq!(days_to_earn(250, 100.0), Some(3));
    assert_eq!(days_to_earn(300, 100.0), Some(3));
    assert_eq!(days_to_earn(1, 0.0), None);
}

#[tokio::test]
async fn max_level_cost_is_gated_by_the_profile() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": false })).await;

    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            max_level_cost(&ctx, subject, Some("1"), LevelTarget::Max).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        max_level_cost(&ctx, &Subject::Anonymous, None, LevelTarget::Max).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        max_level_cost(&ctx, &me, Some("1"), LevelTarget::Max).await,
        Err(AppError::Unavailable)
    ));
    settings(&ctx, owner, json!({ "public_profile": true })).await;
    assert!(matches!(
        max_level_cost(&ctx, &visitor, Some("1"), LevelTarget::Max).await,
        Err(AppError::Unavailable)
    ));
    db.close().await;
}

#[tokio::test]
async fn max_level_cost_on_a_seeded_roster() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = db.user(1, Server::EN).await;
    let me = common::user(&ctx, owner, Role::User).await;
    sync_roster(&ctx).await;

    let full = max_level_cost(&ctx, &me, None, LevelTarget::Max)
        .await
        .unwrap();
    assert_eq!((full.operators_total, full.operators_remaining), (1, 1));
    let amiya = &full.operators[0];
    assert_eq!(amiya.operator_id, AMIYA);
    assert_eq!((amiya.elite, amiya.level, amiya.target_elite), (1, 40, 2));
    assert!(amiya.exp > 0 && amiya.level_lmd > 0 && amiya.promotion_lmd > 0);
    assert_eq!(full.lmd_needed, amiya.level_lmd + amiya.promotion_lmd);
    assert_eq!(full.lmd_owned, Some(500_000));
    assert_eq!(full.lmd_missing, Some((full.lmd_needed - 500_000).max(0)));
    assert!(full.exp_owned > 0);
    assert_eq!(full.exp_missing, (full.exp_needed - full.exp_owned).max(0));
    assert_eq!(full.income.farming_stage, "CE-6");
    assert!(full.income.base_per_day.is_none());

    let module = max_level_cost(&ctx, &me, None, LevelTarget::Module)
        .await
        .unwrap();
    let stop = operator_target(
        &snapshot.model.operators[AMIYA],
        LevelTarget::Module,
        &module_targets_by_rarity(&snapshot.model),
    )
    .unwrap();
    assert!(module.lmd_needed <= full.lmd_needed);
    assert_eq!(
        (
            module.operators[0].target_elite,
            module.operators[0].target_level
        ),
        stop
    );

    settings(
        &ctx,
        owner,
        json!({ "public_profile": true, "profile_layout": { "header": { "hidden": ["lmd"] } } }),
    )
    .await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let seen = max_level_cost(&ctx, &visitor, Some("1"), LevelTarget::Max)
        .await
        .unwrap();
    assert_eq!((seen.lmd_owned, seen.lmd_missing), (None, None));
    assert_eq!(seen.income.days_without_farming, Some(0));
    assert_eq!(seen.exp_needed, full.exp_needed);
    let own = max_level_cost(&ctx, &me, Some("1"), LevelTarget::Max)
        .await
        .unwrap();
    assert_eq!(own.lmd_owned, Some(500_000));
    db.close().await;
}
