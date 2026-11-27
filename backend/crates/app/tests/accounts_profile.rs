#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::Subject;
use myrtle_app::privacy::layout::{ProfileTabId, ShowcaseBlock};
use myrtle_app::privacy::{UpdateSettingsRequest, update_settings};
use myrtle_app::services::accounts::account::{delete_account, get_user, user_key, verify};
use myrtle_app::services::accounts::game_session::{disconnect, ensure_fresh, load};
use myrtle_app::services::accounts::jwt::{Claims, verify_token};
use myrtle_app::services::accounts::login::complete_login;
use myrtle_app::services::accounts::sessions::{portal_session_key, session_key};
use myrtle_app::services::accounts::{subject, subject_optional};
use myrtle_db::repos::roster::{SyncUserData, sync_user_data};
use myrtle_db::repos::{gacha, i18n, users};
use myrtle_hypergryph::{AuthSession, SessionKey};
use myrtle_types::{GameUid, Role, Server, UserId};
use serde_json::{Value, json};

const TTL: Duration = Duration::from_mins(5);

fn session(uid: &str) -> AuthSession {
    AuthSession {
        yostar_uid: "y-uid".into(),
        yostar_token: "y-token".into(),
        ..AuthSession::new(Some(uid), Some("secret"), Some(1), Some("token"))
    }
}

fn key(uid: &str) -> SessionKey {
    SessionKey::Player {
        server: Server::EN,
        uid: GameUid(uid.into()),
    }
}

async fn signed_in(ctx: &myrtle_app::AppContext, uid: &str) -> (UserId, Claims) {
    let response = complete_login(ctx, Server::EN, &session(uid), None, true)
        .await
        .unwrap();
    let claims = verify_token(common::JWT_SECRET, &response.token).unwrap();
    (claims.user_id().unwrap(), claims)
}

#[tokio::test]
async fn the_subject_carries_the_token_identity() {
    let (ctx, db) = common::context().await;
    let (id, mut claims) = signed_in(&ctx, "4242").await;
    let me = subject(&ctx, &claims).await.unwrap();
    assert_eq!(me.user_id(), Some(id));
    assert_eq!(me.uid(), Some("4242"));
    assert_eq!(me.server(), Some(Server::EN));
    assert_eq!(me.role(), Some(Role::User));
    assert_eq!(Subject::Anonymous.uid(), None);
    assert_eq!(me.game_server().unwrap(), Server::EN);
    for unknown in ["nowhere", "EN", "bilibili"] {
        claims.server = unknown.into();
        let odd = subject(&ctx, &claims).await.unwrap();
        assert_eq!(odd.server(), None, "{unknown}");
        assert!(
            matches!(odd.game_server(), Err(AppError::BadRequest(m)) if m == "unknown server"),
            "{unknown}"
        );
    }
    assert!(matches!(
        Subject::Anonymous.game_server(),
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

fn claims_for(id: UserId, role: &str) -> Claims {
    Claims {
        sub: id.0.to_string(),
        uid: "9".into(),
        server: "en".into(),
        role: role.into(),
        exp: u64::MAX,
        iat: 0,
    }
}

#[tokio::test]
async fn only_a_claim_above_user_reads_the_stored_role() {
    let (ctx, db) = common::context().await;
    let gone = UserId(uuid::Uuid::from_u128(9));
    let plain = subject(&ctx, &claims_for(gone, "user")).await.unwrap();
    assert_eq!(
        plain.role(),
        Some(Role::User),
        "no row read for a plain user"
    );
    assert!(matches!(
        subject(&ctx, &claims_for(gone, "super_admin")).await,
        Err(AppError::Unauthorized)
    ));

    let id = db.user(9, Server::EN).await;
    users::set_role(&mut ctx.db.conn().await.unwrap(), id, "translator")
        .await
        .unwrap();
    let capped = subject(&ctx, &claims_for(id, "super_admin")).await.unwrap();
    assert_eq!(capped.role(), Some(Role::Translator));
    db.close().await;
}

#[tokio::test]
async fn an_optional_subject_degrades_to_anonymous() {
    let (ctx, db) = common::context().await;
    let (id, claims) = signed_in(&ctx, "4343").await;
    let token = myrtle_app::services::accounts::jwt::create_token(
        common::JWT_SECRET,
        id,
        &claims.uid,
        Server::EN,
        Role::SuperAdmin,
        claims.iat,
    )
    .unwrap();
    assert!(matches!(
        subject_optional(&ctx, None).await,
        Subject::Anonymous
    ));
    assert!(matches!(
        subject_optional(&ctx, Some("not.a.token")).await,
        Subject::Anonymous
    ));
    let me = subject_optional(&ctx, Some(&token)).await;
    assert_eq!(me.user_id(), Some(id));
    assert_eq!(me.role(), Some(Role::User), "capped by the stored role");

    users::delete_account(&mut ctx.db.conn().await.unwrap(), id)
        .await
        .unwrap();
    assert!(
        matches!(
            subject_optional(&ctx, Some(&token)).await,
            Subject::Anonymous
        ),
        "an elevated claim whose account is gone"
    );
    db.close().await;
}

#[tokio::test]
async fn load_reads_the_cached_session_only() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        load(&ctx, "1").await,
        Err(AppError::GameLoginRequired)
    ));
    ctx.cache.set(&session_key("1"), "not json", TTL).await;
    assert!(
        matches!(load(&ctx, "1").await, Err(AppError::BadRequest(m)) if m == "invalid game session")
    );
    ctx.cache
        .set_json(&session_key("1"), &session("1"), TTL)
        .await;
    assert_eq!(load(&ctx, "1").await.unwrap(), session("1"));
    db.close().await;
}

#[tokio::test]
async fn ensure_fresh_with_nothing_held_asks_for_a_login() {
    let (ctx, db) = common::context().await;
    db.user(2, Server::EN).await;
    let result = ensure_fresh(&ctx, "2", Server::EN).await;
    assert!(
        matches!(result, Err(AppError::GameLoginRequired)),
        "{result:?}"
    );
    db.close().await;
}

#[tokio::test]
async fn disconnect_forgets_the_sessions_and_the_credential_once() {
    let (ctx, db) = common::context().await;
    let (id, _) = signed_in(&ctx, "3").await;
    ctx.cache.set(&portal_session_key("3"), "{}", TTL).await;
    ctx.cache.set(&session_key("4"), "{}", TTL).await;
    let subject = common::user(&ctx, id, Role::User).await;

    let first = disconnect(&ctx, &subject).await.unwrap();
    assert!(first.removed);
    assert_eq!(first.status, "ok");
    assert_eq!(ctx.cache.get(&session_key("3")).await, None);
    assert_eq!(ctx.cache.get(&portal_session_key("3")).await, None);
    assert_eq!(ctx.sessions.load_durable(&key("3")).await.unwrap(), None);
    assert!(
        ctx.cache.get(&session_key("4")).await.is_some(),
        "only the subject's own session"
    );
    assert!(!disconnect(&ctx, &subject).await.unwrap().removed);
    assert!(matches!(
        disconnect(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn the_profile_is_cached_until_invalidated() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        get_user(&ctx, &Subject::Anonymous, "4").await,
        Err(AppError::NotFound(None))
    ));
    let (id, _) = signed_in(&ctx, "4").await;
    let me = common::user(&ctx, id, Role::User).await;

    let profile = get_user(&ctx, &me, "4").await.unwrap();
    assert_eq!((profile.id, profile.role.as_str()), (id.0, "user"));
    let json = serde_json::to_value(&profile).unwrap();
    assert_eq!(json["public_gacha"], false, "read fresh for every caller");
    assert_eq!(json["server"], "en");

    users::set_role(&mut ctx.db.conn().await.unwrap(), id, "translator")
        .await
        .unwrap();
    assert_eq!(get_user(&ctx, &me, "4").await.unwrap().role, "user");
    ctx.cache.invalidate(&user_key("4")).await;
    assert_eq!(get_user(&ctx, &me, "4").await.unwrap().role, "translator");
    db.close().await;
}

#[tokio::test]
async fn verify_rereads_the_role_and_the_admin_panel_gate() {
    let (mut ctx, db) = common::context().await;
    let (id, claims) = signed_in(&ctx, "5").await;
    let mut conn = ctx.db.conn().await.unwrap();

    let live = verify(&ctx, &claims).await.unwrap();
    assert_eq!(
        (live.role.as_str(), live.can_access_admin_panel),
        ("user", false)
    );
    assert_eq!((live.uid.as_str(), live.server.as_str()), ("5", "en"));
    let json = serde_json::to_value(&live).unwrap();
    assert_eq!(json["userId"], id.0.to_string());
    assert_eq!(json["canAccessAdminPanel"], false);

    i18n::grant_permission(&mut conn, "en", id, "view", id)
        .await
        .unwrap();
    assert!(verify(&ctx, &claims).await.unwrap().can_access_admin_panel);

    let (other, other_claims) = signed_in(&ctx, "6").await;
    users::set_role(&mut conn, other, "tier_list_editor")
        .await
        .unwrap();
    let promoted = verify(&ctx, &other_claims).await.unwrap();
    assert_eq!(promoted.role, "tier_list_editor");
    assert!(promoted.can_access_admin_panel);

    drop(conn);
    delete_account(&ctx, &common::user(&ctx, other, Role::User).await)
        .await
        .unwrap();
    assert!(matches!(
        verify(&ctx, &other_claims).await,
        Err(AppError::Unauthorized)
    ));
    ctx.policy.role_recheck = false;
    assert_eq!(verify(&ctx, &other_claims).await.unwrap().role, "user");
    db.close().await;
}

#[tokio::test]
async fn deleting_an_account_removes_it_and_flushes_what_held_it() {
    let (ctx, db) = common::context().await;
    let (id, _) = signed_in(&ctx, "7").await;
    let subject = common::user(&ctx, id, Role::User).await;
    get_user(&ctx, &subject, "7").await.unwrap();
    let held = [
        portal_session_key("7"),
        "leaderboard:score:all".to_owned(),
        "search:abc".to_owned(),
        "improvements:7:v1".to_owned(),
        "base:rotation:7:x".to_owned(),
        "base:optimize:7:x".to_owned(),
    ];
    for key in &held {
        ctx.cache.set(key, "1", TTL).await;
    }
    let kept = ["improvements:70:v1", "user:70"];
    for key in kept {
        ctx.cache.set(key, "1", TTL).await;
    }

    delete_account(&ctx, &subject).await.unwrap();

    let mut conn = ctx.db.conn().await.unwrap();
    assert!(!users::exists(&mut conn, id).await.unwrap());
    for key in held.iter().chain([&user_key("7"), &session_key("7")]) {
        assert_eq!(ctx.cache.get(key).await, None, "{key}");
    }
    for key in kept {
        assert!(ctx.cache.get(key).await.is_some(), "{key}");
    }
    assert!(matches!(
        delete_account(&ctx, &subject).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        delete_account(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    drop(conn);
    db.close().await;
}

async fn seed_status(ctx: &myrtle_app::AppContext, uid: &str) {
    let empty = json!({});
    let none = json!([]);
    let mut status = json!({ "main_stage_progress": "", "resume": "" });
    for key in [
        "exp",
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
        "monthly_sub_end",
        "register_ts",
        "last_online_ts",
        "friend_num_limit",
    ] {
        status[key] = json!(0);
    }
    status["lmd"] = json!(9000);
    status["orundum"] = json!(5);
    let data = SyncUserData {
        uid,
        server: Server::EN,
        nickname: "Doctor",
        level: 120,
        avatar_id: None,
        secretary: None,
        secretary_skin_id: None,
        resume_id: None,
        operators: &none,
        skills: &none,
        modules: &none,
        items: &none,
        skins: &none,
        status: &status,
        stages: &empty,
        roguelike: &none,
        sandbox: &empty,
        medals: &none,
        building: &empty,
        checkin: &empty,
        supports: &none,
        nick_number: None,
        enemies: &empty,
        game_squads: &none,
    };
    sync_user_data(&mut ctx.db.conn().await.unwrap(), &data)
        .await
        .unwrap();
}

async fn settings(ctx: &myrtle_app::AppContext, owner: &Subject, body: Value) {
    let id = owner.user_id().unwrap();
    gacha::update_flags(&mut ctx.db.conn().await.unwrap(), id, None, None)
        .await
        .unwrap();
    let request: UpdateSettingsRequest = serde_json::from_value(body).unwrap();
    update_settings(ctx, owner, &request).await.unwrap();
}

#[tokio::test]
async fn a_visitor_gets_the_projected_profile_and_the_owner_all_of_it() {
    let (ctx, db) = common::context().await;
    let (id, _) = signed_in(&ctx, "8").await;
    let owner = common::user(&ctx, id, Role::User).await;
    let visitor = common::user(&ctx, signed_in(&ctx, "9").await.0, Role::User).await;
    seed_status(&ctx, "8").await;
    let layout = json!({
        "tabs": [{ "id": "roster", "visible": false }],
        "header": { "hidden": ["lmd"] },
        "showcase": { "blocks": [{ "type": "grid", "slug": "gone" }] },
    });
    settings(
        &ctx,
        &owner,
        json!({ "public_profile": true, "profile_layout": layout }),
    )
    .await;

    let mine = get_user(&ctx, &owner, "8").await.unwrap();
    assert_eq!((mine.lmd, mine.orundum), (Some(9000), Some(5)));
    let my_layout = mine.profile_layout.unwrap();
    assert!(!my_layout.is_visible(ProfileTabId::Roster));
    assert!(my_layout.tabs.iter().any(|t| t.id == ProfileTabId::Roster));
    assert_eq!(my_layout.showcase_blocks().len(), 1);

    let seen = get_user(&ctx, &visitor, "8").await.unwrap();
    assert_eq!((seen.lmd, seen.orundum), (None, Some(5)));
    let seen_layout = seen.profile_layout.unwrap();
    assert!(
        seen_layout
            .tabs
            .iter()
            .all(|t| t.id != ProfileTabId::Roster)
    );
    assert!(
        seen_layout.showcase_blocks().is_empty(),
        "a gone grid is pruned"
    );
    assert_eq!(seen.public_gacha, Some(false));

    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "tabs": [{ "id": "showcase", "visible": false }], "showcase": { "blocks": [{ "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya"] }] } } }),
    )
    .await;
    let mine = get_user(&ctx, &owner, "8").await.unwrap();
    assert!(matches!(
        mine.profile_layout.unwrap().showcase_blocks(),
        [ShowcaseBlock::Favourites { .. }]
    ));
    let seen = get_user(&ctx, &Subject::Anonymous, "8").await.unwrap();
    assert!(
        seen.profile_layout.unwrap().showcase.is_none(),
        "the tab is hidden"
    );

    settings(&ctx, &owner, json!({ "public_profile": false })).await;
    for subject in [&visitor, &Subject::Anonymous] {
        assert!(matches!(
            get_user(&ctx, subject, "8").await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(get_user(&ctx, &owner, "8").await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn the_layout_reads_back_normalized() {
    let (ctx, db) = common::context().await;
    let (id, _) = signed_in(&ctx, "10").await;
    let owner = common::user(&ctx, id, Role::User).await;
    settings(
        &ctx,
        &owner,
        json!({ "profile_layout": { "tabs": [{ "id": "stats", "visible": false }, { "id": "nope" }] } }),
    )
    .await;
    let body = serde_json::to_value(get_user(&ctx, &owner, "10").await.unwrap()).unwrap();
    let tabs = body["profile_layout"]["tabs"].as_array().unwrap();
    assert_eq!(tabs.len(), ProfileTabId::ALL.len(), "every known tab, once");
    assert_eq!(tabs[0], json!({ "id": "showcase", "visible": true }));
    assert!(tabs.contains(&json!({ "id": "stats", "visible": false })));
    db.close().await;
}

#[tokio::test]
async fn an_unknown_stored_role_opens_no_admin_panel() {
    let (ctx, db) = common::context().await;
    let (id, claims) = signed_in(&ctx, "11").await;
    users::set_role(&mut ctx.db.conn().await.unwrap(), id, "mystery")
        .await
        .unwrap();
    let live = verify(&ctx, &claims).await.unwrap();
    assert_eq!(
        (live.role.as_str(), live.can_access_admin_panel),
        ("mystery", false)
    );
    db.close().await;
}
