#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::catalog::static_data::encountered_enemies;
use myrtle_app::services::sync::roster::{OwnedSkin, owned_skins};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::{roster, users};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

const UID: &str = "501";

async fn seed(ctx: &AppContext, enemies: &Value) -> UserId {
    let skins = json!([
        { "skin_id": "char_002_amiya@epoque#4", "obtained_at": 5 },
        { "skin_id": "char_002_amiya#1", "obtained_at": 1 },
        { "skin_id": "char_102_texas@wild#1", "obtained_at": null },
    ]);
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
            uid: UID,
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
            skins: &skins,
            status: &status,
            stages: &map,
            roguelike: &list,
            sandbox: &map,
            medals: &list,
            building: &map,
            checkin: &map,
            supports: &list,
            nick_number: Some("1234"),
            enemies,
            game_squads: &list,
        },
    )
    .await
    .unwrap();
    UserId(
        users::find(&mut conn, UID, Server::EN)
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

#[tokio::test]
async fn owned_skins_list_non_default_skins_to_the_owner_and_public_visitors() {
    let (ctx, db) = common::context().await;
    let owner = seed(&ctx, &json!({})).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    let owned = owned_skins(&ctx, &me, None).await.unwrap();
    assert_eq!(
        owned,
        [
            OwnedSkin {
                skin_id: "char_002_amiya@epoque#4".into(),
                obtained_at: Some(5)
            },
            OwnedSkin {
                skin_id: "char_102_texas@wild#1".into(),
                obtained_at: None
            },
        ]
    );
    assert_eq!(
        serde_json::to_value(&owned[0]).unwrap(),
        json!({ "skin_id": "char_002_amiya@epoque#4", "obtained_at": 5 })
    );
    assert_eq!(owned_skins(&ctx, &visitor, Some(UID)).await.unwrap(), owned);
    assert!(matches!(
        owned_skins(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));

    settings(&ctx, owner, json!({ "public_profile": false })).await;
    assert!(matches!(
        owned_skins(&ctx, &Subject::Anonymous, Some(UID)).await,
        Err(AppError::Forbidden)
    ));
    assert_eq!(owned_skins(&ctx, &me, Some(UID)).await.unwrap(), owned);
    db.close().await;
}

#[tokio::test]
async fn owned_skins_need_both_the_stats_and_the_roster_tab() {
    let (ctx, db) = common::context().await;
    let owner = seed(&ctx, &json!({})).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    for hidden in ["stats", "roster"] {
        settings(
            &ctx,
            owner,
            json!({ "profile_layout": { "tabs": [{ "id": hidden, "visible": false }] } }),
        )
        .await;
        for subject in [&visitor, &Subject::Anonymous] {
            assert!(
                matches!(
                    owned_skins(&ctx, subject, Some(UID)).await,
                    Err(AppError::Forbidden)
                ),
                "{hidden} hidden"
            );
        }
        assert_eq!(
            owned_skins(&ctx, &me, Some(UID)).await.unwrap().len(),
            2,
            "the owner reads it with {hidden} hidden"
        );
        settings(&ctx, owner, json!({ "profile_layout": null })).await;
        assert!(owned_skins(&ctx, &visitor, Some(UID)).await.is_ok());
    }
    db.close().await;
}

#[tokio::test]
async fn encountered_enemies_sort_by_handbook_with_unknown_ids_last() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let handbook = &snapshot.model.enemies.enemy_data;
    let mut listed: Vec<_> = handbook
        .iter()
        .filter(|(_, e)| !e.hide_in_handbook)
        .collect();
    listed.sort_by_key(|(id, e)| (e.sort_id, *id));
    let (early, late) = (listed[0], listed[1]);
    let mut met = json!({ "enemy_not_in_any_handbook": 1 });
    met[late.0] = json!(1);
    met[early.0] = json!(1);
    let owner = seed(&ctx, &met).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    settings(&ctx, owner, json!({ "public_profile": true })).await;

    let seen = encountered_enemies(&ctx, &me, None).await.unwrap();
    assert_eq!(seen.encountered_count, 3);
    assert_eq!(seen.handbook_total, listed.len());
    let ids: Vec<&str> = seen.enemies.iter().map(|e| e.enemy_id.as_str()).collect();
    assert_eq!(
        ids,
        [
            early.0.as_str(),
            late.0.as_str(),
            "enemy_not_in_any_handbook"
        ]
    );
    assert_eq!(seen.enemies[0].name.as_deref(), Some(early.1.name.as_str()));
    assert_eq!(seen.enemies[2].name, None);
    let wire = serde_json::to_value(&seen).unwrap();
    assert!(wire["encounteredCount"].is_u64() && wire["enemies"][0]["enemyIndex"].is_string());

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "roster", "visible": false }] } }),
    )
    .await;
    assert!(
        encountered_enemies(&ctx, &visitor, Some(UID)).await.is_ok(),
        "only the enemies tab gates them"
    );

    settings(
        &ctx,
        owner,
        json!({ "profile_layout": { "tabs": [{ "id": "enemies", "visible": false }] } }),
    )
    .await;
    assert!(matches!(
        encountered_enemies(&ctx, &visitor, Some(UID)).await,
        Err(AppError::Forbidden)
    ));
    assert_eq!(
        encountered_enemies(&ctx, &me, Some(UID)).await.unwrap(),
        seen
    );
    db.close().await;
}
