#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::services::leaderboard::board::{
    self, LeaderboardParams, MoversParams, StandingParams,
};
use myrtle_app::services::leaderboard::items::{
    self, ItemLeaderboardParams, ItemStandingParams, is_valid_item_id,
};
use myrtle_app::services::leaderboard::search::{SearchParams, is_valid_operator_id, search};
use myrtle_app::services::leaderboard::{jobs, stats};
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::scores::{self, ScoreWrite};
use myrtle_db::repos::users::{self, LayoutUpdate, SettingsUpdate};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

async fn publish(ctx: &AppContext, user: UserId, layout: Option<Value>) {
    let mut conn = ctx.db.conn().await.unwrap();
    users::set_base_facts(&mut conn, user, &json!({}))
        .await
        .unwrap();
    let patch = layout.unwrap_or_else(|| json!({}));
    let update = SettingsUpdate {
        public_profile: Some(true),
        store_gacha: None,
        share_stats: None,
        public_gacha: None,
        layout: LayoutUpdate::Merge {
            patch: &patch,
            removed: &[],
        },
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
}

async fn score(ctx: &AppContext, user: UserId, total: f64) {
    let mut conn = ctx.db.conn().await.unwrap();
    let write = ScoreWrite {
        total_score: total,
        operator_score: total,
        stage_score: 0.0,
        roguelike_score: 0.0,
        sandbox_score: 0.0,
        medal_score: 0.0,
        base_score: 0.0,
        base_utilization: None,
        base_infrastructure: None,
        skin_score: 0.0,
        grade: Some("A"),
    };
    scores::upsert(&mut conn, user, &write).await.unwrap();
}

fn bad_request<T: std::fmt::Debug>(result: Result<T, AppError>) -> String {
    match result {
        Err(AppError::BadRequest(m)) => m,
        other => panic!("expected a bad request, got {other:?}"),
    }
}

#[test]
fn operator_ids_are_game_style() {
    assert!(is_valid_operator_id("char_1035_wisdel"));
    assert!(is_valid_operator_id("A"));
    assert!(is_valid_operator_id(&"a".repeat(50)));
    assert!(!is_valid_operator_id(""));
    assert!(!is_valid_operator_id(&"a".repeat(51)));
    for bad in [
        "char-002",
        "char 002",
        "char_002;drop",
        "char/002",
        "char_\u{30a2}",
    ] {
        assert!(!is_valid_operator_id(bad), "{bad}");
    }
    assert!(is_valid_item_id("4002") && is_valid_item_id("mod_unlock_token"));
    assert!(!is_valid_item_id("") && !is_valid_item_id("a-b"));
}

#[tokio::test]
async fn leaderboard_pages_rank_and_validate() {
    let (ctx, db) = common::context().await;
    let low = db.user(8001, Server::EN).await;
    let high = db.user(8002, Server::JP).await;
    let hidden = db.user(8003, Server::EN).await;
    for (user, total) in [(low, 10.0), (high, 20.0), (hidden, 30.0)] {
        publish(&ctx, user, None).await;
        score(&ctx, user, total).await;
    }
    publish(
        &ctx,
        hidden,
        Some(json!({ "tabs": [{ "id": "score", "visible": false }] })),
    )
    .await;

    let page = board::page(&ctx, &LeaderboardParams::default())
        .await
        .unwrap();
    assert_eq!(page.total, 2);
    let uids: Vec<&str> = page.entries.iter().map(|e| e.uid.as_str()).collect();
    assert_eq!(uids, ["8002", "8001"]);
    assert_eq!(page.entries[0].rank_delta, None);

    let jp = LeaderboardParams {
        server: Some("JP".into()),
        ..LeaderboardParams::default()
    };
    assert_eq!(board::page(&ctx, &jp).await.unwrap().total, 1);
    let moved = LeaderboardParams {
        movement_interval: Some("7 days".into()),
        movement_only: true,
        ..LeaderboardParams::default()
    };
    assert_eq!(board::page(&ctx, &moved).await.unwrap().total, 0);

    let msg = bad_request(
        board::page(
            &ctx,
            &LeaderboardParams {
                movement_only: true,
                ..LeaderboardParams::default()
            },
        )
        .await,
    );
    assert!(msg.contains("requires movement_interval"));
    bad_request(
        board::page(
            &ctx,
            &LeaderboardParams {
                movement_interval: Some("2 days".into()),
                ..LeaderboardParams::default()
            },
        )
        .await,
    );
    bad_request(
        board::page(
            &ctx,
            &LeaderboardParams {
                server: Some("mars".into()),
                ..LeaderboardParams::default()
            },
        )
        .await,
    );

    bad_request(
        board::top_movers(
            &ctx,
            &MoversParams {
                direction: Some("sideways".into()),
                ..MoversParams::default()
            },
        )
        .await,
    );
    bad_request(
        board::top_movers(
            &ctx,
            &MoversParams {
                interval: Some("1 week".into()),
                ..MoversParams::default()
            },
        )
        .await,
    );
    assert!(
        board::top_movers(&ctx, &MoversParams::default())
            .await
            .unwrap()
            .is_empty()
    );

    let shares = board::distribution(&ctx, None).await.unwrap();
    assert_eq!(shares.iter().map(|s| s.players).sum::<i64>(), 2);
    db.close().await;
}

#[tokio::test]
async fn a_cached_page_answers_until_it_expires() {
    let (ctx, db) = common::context().await;
    let user = db.user(8101, Server::EN).await;
    publish(&ctx, user, None).await;
    score(&ctx, user, 5.0).await;
    let first = board::page(&ctx, &LeaderboardParams::default())
        .await
        .unwrap();
    let other = db.user(8102, Server::EN).await;
    publish(&ctx, other, None).await;
    let cached = board::page(&ctx, &LeaderboardParams::default())
        .await
        .unwrap();
    assert_eq!(first.total, cached.total);
    ctx.cache.invalidate_prefix("leaderboard:").await;
    let fresh = board::page(&ctx, &LeaderboardParams::default())
        .await
        .unwrap();
    assert_eq!(fresh.total, first.total + 1);
    db.close().await;
}

#[tokio::test]
async fn standing_is_gated_on_profile_and_score_tab() {
    let (ctx, db) = common::context().await;
    let owner = db.user(8201, Server::EN).await;
    let other = db.user(8202, Server::EN).await;
    let visitor = common::user(&ctx, other, Role::User).await;
    score(&ctx, owner, 50.0).await;
    let params = StandingParams {
        uid: "8201".into(),
        server: "en".into(),
        window: None,
        interval: None,
    };

    assert!(matches!(
        board::standing(&ctx, &Subject::Anonymous, &params).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        board::standing(&ctx, &visitor, &params).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        board::standing(&ctx, &common::user(&ctx, owner, Role::User).await, &params).await,
        Err(AppError::NotFound(_))
    ));

    publish(&ctx, owner, None).await;
    publish(&ctx, other, None).await;
    score(&ctx, other, 40.0).await;
    let standing = board::standing(&ctx, &visitor, &params).await.unwrap();
    assert_eq!(standing.player.uid, "8201");
    assert_eq!(standing.neighbors.len(), 1);
    assert!((standing.percentile - 0.5).abs() < f64::EPSILON);
    assert_eq!(standing.rank_delta, None);
    assert!(
        board::score_history(&ctx, &visitor, "8201")
            .await
            .unwrap()
            .is_empty()
    );

    publish(
        &ctx,
        owner,
        Some(json!({ "tabs": [{ "id": "score", "visible": false }] })),
    )
    .await;
    assert!(matches!(
        board::standing(&ctx, &visitor, &params).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        board::score_history(&ctx, &visitor, "8201").await,
        Err(AppError::Forbidden)
    ));
    bad_request(
        board::standing(
            &ctx,
            &visitor,
            &StandingParams {
                interval: Some("1 hour".into()),
                ..params
            },
        )
        .await,
    );
    db.close().await;
}

#[tokio::test]
async fn item_reads_validate_and_gate_the_inventory_tab() {
    let (ctx, db) = common::context().await;
    let owner = db.user(8301, Server::EN).await;
    let visitor = common::user(&ctx, db.user(8302, Server::EN).await, Role::User).await;
    publish(&ctx, owner, None).await;

    let msg = bad_request(
        items::page(
            &ctx,
            &ItemLeaderboardParams {
                item: "bad id".into(),
                ..ItemLeaderboardParams::default()
            },
        )
        .await,
    );
    assert!(msg.starts_with("item must be a game item id"));
    let page = items::page(
        &ctx,
        &ItemLeaderboardParams {
            item: " 4002 ".into(),
            ..ItemLeaderboardParams::default()
        },
    )
    .await
    .unwrap();
    assert_eq!((page.item_id.as_str(), page.total), ("4002", 0));
    let catalog = items::catalog(&ctx, None).await.unwrap();
    assert_eq!(catalog.population, 1);

    let params = ItemStandingParams {
        item: "4002".into(),
        uid: "8301".into(),
        server: "en".into(),
    };
    assert!(matches!(
        items::standing(&ctx, &visitor, &params).await,
        Err(AppError::NotFound(_))
    ));
    publish(
        &ctx,
        owner,
        Some(json!({ "tabs": [{ "id": "inventory", "visible": false }] })),
    )
    .await;
    assert!(matches!(
        items::standing(&ctx, &visitor, &params).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn admin_stats_need_a_list_admin() {
    let (ctx, db) = common::context().await;
    let user = db.user(8401, Server::KR).await;
    publish(&ctx, user, None).await;

    let site = stats::site(&ctx).await.unwrap();
    assert_eq!((site.users.total, site.users.by_server.kr), (1, 1));
    assert_eq!(site.users.public_profiles, 1);
    let body = serde_json::to_value(&site).unwrap();
    assert_eq!(body["users"]["byServer"]["kr"], 1);
    assert!(body["gameData"].is_object() && body["computedAt"].is_string());

    assert!(matches!(
        stats::admin(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        stats::admin(&ctx, &common::user(&ctx, user, Role::TierListEditor).await).await,
        Err(AppError::Forbidden)
    ));
    let admin = stats::admin(&ctx, &common::user(&ctx, user, Role::TierListAdmin).await)
        .await
        .unwrap();
    assert_eq!(admin.users_by_role.user, 1);
    assert_eq!(admin.recent_users[0].uid, "8401");
    let body = serde_json::to_value(&admin).unwrap();
    assert!(body["usersByRole"].is_object() && body["users"]["total"] == 1);
    db.close().await;
}

#[tokio::test]
async fn search_validates_its_params() {
    let (ctx, db) = common::context().await;
    let cases = [
        SearchParams {
            sort: Some("height".into()),
            ..SearchParams::default()
        },
        SearchParams {
            sort: Some("class:TOKEN".into()),
            ..SearchParams::default()
        },
        SearchParams {
            dir: Some("up".into()),
            ..SearchParams::default()
        },
        SearchParams {
            has: Some(
                (0..21)
                    .map(|i| format!("char_{i}"))
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            ..SearchParams::default()
        },
        SearchParams {
            has: Some("char_002_amiya,char-1".into()),
            ..SearchParams::default()
        },
        SearchParams {
            support: Some("x y".into()),
            ..SearchParams::default()
        },
        SearchParams {
            all: Some("sub:Bad".into()),
            ..SearchParams::default()
        },
    ];
    for params in &cases {
        bad_request(search(&ctx, params).await);
    }
    let duplicates = SearchParams {
        has: Some(
            (0..25)
                .map(|i| format!("char_{}", i % 20))
                .collect::<Vec<_>>()
                .join(","),
        ),
        ..SearchParams::default()
    };
    assert!(search(&ctx, &duplicates).await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn search_rows_carry_a_visitors_view() {
    let (ctx, db) = common::context().await;
    let open = db.user(8501, Server::EN).await;
    let guarded = db.user(8502, Server::EN).await;
    let private = db.user(8503, Server::EN).await;
    score(&ctx, open, 10.0).await;
    score(&ctx, guarded, 20.0).await;
    score(&ctx, private, 30.0).await;
    publish(&ctx, open, None).await;
    publish(
        &ctx,
        guarded,
        Some(json!({
            "tabs": [{ "id": "roster", "visible": true }, { "id": "score", "visible": false }],
            "header": { "hidden": ["level", "skins"] },
            "background": { "kind": "skin", "id": "char_002_amiya@winter#1" },
            "showcase": { "blocks": [{ "type": "grid", "slug": "my-grid" }] },
        })),
    )
    .await;

    for pass in ["built", "cached"] {
        let page = search(&ctx, &SearchParams::default()).await.unwrap();
        assert_eq!(page.total, 2);
        let open_row = page
            .entries
            .iter()
            .find(|e| e.profile.uid == "8501")
            .unwrap();
        assert_eq!(open_row.profile.total_score, Some(10.0));
        assert!(open_row.profile.profile_layout.is_none());

        let row = &page
            .entries
            .iter()
            .find(|e| e.profile.uid == "8502")
            .unwrap()
            .profile;
        assert_eq!(
            (row.total_score, row.grade.as_deref()),
            (None, None),
            "{pass}"
        );
        assert_eq!((row.level, row.exp, row.skin_count), (None, None, None));
        let body = serde_json::to_value(row).unwrap();
        let layout = &body["profile_layout"];
        let ids: Vec<&str> = layout["tabs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "showcase",
                "roster",
                "stats",
                "plans",
                "inventory",
                "enemies",
                "optimizer",
                "lookback"
            ],
            "{pass}"
        );
        assert_eq!(layout.as_object().unwrap().len(), 1, "{pass}: tabs only");
        let entry = serde_json::to_value(&page.entries[0]).unwrap();
        assert!(entry.get("public_gacha").is_none() && entry.get("metric").is_some());
    }
    db.close().await;
}

#[tokio::test]
async fn jobs_skip_a_second_daily_run_and_invalidate_ownership() {
    let (ctx, db) = common::context().await;
    let user = db.user(8601, Server::EN).await;
    publish(&ctx, user, None).await;
    score(&ctx, user, 1.0).await;

    assert_eq!(
        jobs::snapshot_leaderboard(&ctx, false).await.unwrap(),
        "snapshot 1 taken"
    );
    assert_eq!(
        jobs::snapshot_leaderboard(&ctx, false).await.unwrap(),
        "already snapshotted today"
    );
    assert_eq!(
        jobs::snapshot_leaderboard(&ctx, true).await.unwrap(),
        "snapshot 2 taken",
        "a manual refresh always runs"
    );
    let movement = LeaderboardParams {
        movement_interval: Some("1 day".into()),
        ..LeaderboardParams::default()
    };
    assert_eq!(board::page(&ctx, &movement).await.unwrap().total, 1);

    assert!(
        jobs::recompute_trending(&ctx)
            .await
            .unwrap()
            .starts_with("recomputed trending windows")
    );

    ctx.cache
        .set("operators:ownership:en", "stale", Duration::from_mins(5))
        .await;
    ctx.cache
        .set(
            "operators:buildstats:en:char_002_amiya",
            "stale",
            Duration::from_mins(5),
        )
        .await;
    jobs::refresh_operator_ownership(&ctx).await.unwrap();
    assert!(ctx.cache.get("operators:ownership:en").await.is_none());
    assert!(
        ctx.cache
            .get("operators:buildstats:en:char_002_amiya")
            .await
            .is_none()
    );

    assert!(
        jobs::refresh_medal_ownership(&ctx, false)
            .await
            .unwrap()
            .ends_with("medal ownership rows")
    );
    assert!(
        jobs::refresh_medal_ownership(&ctx, true)
            .await
            .unwrap()
            .ends_with("medal ownership rows")
    );
    db.close().await;
}

#[tokio::test]
async fn movers_are_cached_per_limit() {
    let (ctx, db) = common::context().await;
    let planted = json!([{
        "uid": "1", "nickname": null, "nick_number": null, "avatar_id": null, "server": "en",
        "current_rank": 1, "previous_rank": 2, "rank_delta": 1,
        "current_score": 1.0, "score_delta": 1.0,
    }]);
    for key in [
        "leaderboard:movers:up:7_days:all",
        "leaderboard:movers:up:7_days:all:10",
    ] {
        ctx.cache
            .set(key, &planted.to_string(), Duration::from_mins(5))
            .await;
    }
    let movers = |limit| {
        let params = MoversParams {
            limit: Some(limit),
            ..MoversParams::default()
        };
        let ctx = &ctx;
        async move { board::top_movers(ctx, &params).await.unwrap().len() }
    };
    assert_eq!(movers(10).await, 1, "served from the limit's own key");
    assert_eq!(
        movers(100).await,
        0,
        "another limit is not served that body"
    );
    db.close().await;
}
