#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::services::accounts::account::{
    AdminUsersPage, AdminUsersQuery, list_users, set_user_role, user_key,
};
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::users;
use myrtle_types::{PageRequest, Role, Server, UserId};
use serde_json::json;

const TTL: Duration = Duration::from_mins(5);

async fn stored_role(ctx: &AppContext, user: UserId) -> String {
    users::role(&mut ctx.db.conn().await.unwrap(), user)
        .await
        .unwrap()
        .unwrap()
}

async fn promote(ctx: &AppContext, user: UserId, role: &str) {
    users::set_role(&mut ctx.db.conn().await.unwrap(), user, role)
        .await
        .unwrap();
}

fn query(role: Option<&str>) -> AdminUsersQuery {
    AdminUsersQuery {
        role: role.map(str::to_owned),
        ..AdminUsersQuery::default()
    }
}

async fn by_role(ctx: &AppContext, role: Option<&str>) -> AdminUsersPage {
    list_users(ctx, &Subject::System, &query(role), page(None, None))
        .await
        .unwrap()
}

const fn page(limit: Option<u32>, offset: Option<u32>) -> PageRequest {
    PageRequest { limit, offset }
}

#[tokio::test]
async fn only_a_super_admin_changes_a_role_and_never_their_own() {
    let (ctx, db) = common::context().await;
    let target = db.user(1, Server::EN).await;
    let admin_id = db.user(2, Server::EN).await;
    let admin = common::user(&ctx, admin_id, Role::SuperAdmin).await;

    for role in [
        Role::User,
        Role::Translator,
        Role::TierListEditor,
        Role::TierListAdmin,
    ] {
        let caller = common::user(&ctx, admin_id, role).await;
        assert!(
            matches!(
                set_user_role(&ctx, &caller, target, "translator").await,
                Err(AppError::Forbidden)
            ),
            "{role:?}"
        );
    }
    assert!(matches!(
        set_user_role(&ctx, &Subject::Anonymous, target, "translator").await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        set_user_role(&ctx, &admin, target, "overlord").await,
        Err(AppError::BadRequest(m)) if m == "Unknown role: overlord"
    ));
    assert!(matches!(
        set_user_role(&ctx, &admin, target, "").await,
        Err(AppError::BadRequest(m)) if m == "Unknown role: "
    ));
    assert!(matches!(
        set_user_role(&ctx, &admin, UserId(uuid::Uuid::from_u128(77)), "user").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        set_user_role(&ctx, &admin, admin_id, "user").await,
        Err(AppError::BadRequest(m)) if m == "refusing to change your own role; use the manage_permissions CLI"
    ));
    assert_eq!(stored_role(&ctx, target).await, "user");
    assert_eq!(stored_role(&ctx, admin_id).await, "user");

    set_user_role(&ctx, &Subject::System, target, "tier_list_editor")
        .await
        .unwrap();
    assert_eq!(stored_role(&ctx, target).await, "tier_list_editor");
    db.close().await;
}

#[tokio::test]
async fn a_role_change_drops_the_cached_profile_and_a_no_op_keeps_it() {
    let (ctx, db) = common::context().await;
    let target = db.user(1, Server::EN).await;
    let admin = common::user(&ctx, db.user(2, Server::EN).await, Role::SuperAdmin).await;
    for key in [user_key("1"), user_key("10")] {
        ctx.cache.set(&key, "{}", TTL).await;
    }

    set_user_role(&ctx, &admin, target, "user").await.unwrap();
    assert!(
        ctx.cache.get(&user_key("1")).await.is_some(),
        "an unchanged role writes nothing"
    );

    set_user_role(&ctx, &admin, target, "translator")
        .await
        .unwrap();
    assert_eq!(stored_role(&ctx, target).await, "translator");
    assert_eq!(ctx.cache.get(&user_key("1")).await, None);
    assert!(
        ctx.cache.get(&user_key("10")).await.is_some(),
        "only the target's key"
    );
    db.close().await;
}

#[tokio::test]
async fn listing_users_needs_a_list_admin() {
    let (ctx, db) = common::context().await;
    let id = db.user(1, Server::EN).await;
    for role in [Role::User, Role::Translator, Role::TierListEditor] {
        let caller = common::user(&ctx, id, role).await;
        assert!(
            matches!(
                list_users(&ctx, &caller, &query(None), page(None, None)).await,
                Err(AppError::Forbidden)
            ),
            "{role:?}"
        );
    }
    assert!(matches!(
        list_users(&ctx, &Subject::Anonymous, &query(None), page(None, None)).await,
        Err(AppError::Unauthorized)
    ));
    for role in [Role::TierListAdmin, Role::SuperAdmin] {
        let caller = common::user(&ctx, id, role).await;
        let listed = list_users(&ctx, &caller, &query(None), page(None, None))
            .await
            .unwrap();
        assert_eq!(listed.total, 1, "{role:?}");
    }
    assert!(
        list_users(&ctx, &Subject::System, &query(None), page(None, None))
            .await
            .is_ok()
    );
    db.close().await;
}

#[tokio::test]
async fn the_admin_list_pages_with_its_own_default_and_cap() {
    let (ctx, db) = common::context().await;
    for n in 1..=55 {
        db.user(n, Server::EN).await;
    }
    let admin = Subject::System;
    let all = query(None);

    let first = list_users(&ctx, &admin, &all, page(None, None))
        .await
        .unwrap();
    assert_eq!((first.users.len(), first.total), (50, 55));
    let rest = list_users(&ctx, &admin, &all, page(None, Some(50)))
        .await
        .unwrap();
    assert_eq!((rest.users.len(), rest.total), (5, 55));
    assert!(
        first
            .users
            .iter()
            .all(|u| rest.users.iter().all(|r| r.id != u.id)),
        "pages do not overlap"
    );
    let wide = list_users(&ctx, &admin, &all, page(Some(500), None))
        .await
        .unwrap();
    assert_eq!(
        wide.users.len(),
        55,
        "past the shared 100 cap is fine below 200"
    );
    let one = list_users(&ctx, &admin, &all, page(Some(0), None))
        .await
        .unwrap();
    assert_eq!(one.users.len(), 1, "a zero limit reads as one");
    db.close().await;
}

#[tokio::test]
async fn the_admin_list_filters_by_role_search_and_server() {
    let (ctx, db) = common::context().await;
    let staff = [
        (db.user(1, Server::EN).await, "super_admin"),
        (db.user(2, Server::EN).await, "tier_list_admin"),
        (db.user(3, Server::EN).await, "tier_list_editor"),
        (db.user(4, Server::EN).await, "translator"),
    ];
    for (id, role) in staff {
        promote(&ctx, id, role).await;
    }
    db.user(5, Server::EN).await;
    db.user(56, Server::JP).await;
    let admin = Subject::System;
    let count = |role| by_role(&ctx, role);

    assert_eq!(count(None).await.total, 6);
    assert_eq!(count(Some("all")).await.total, 6);
    assert_eq!(count(Some("  ")).await.total, 6, "blank reads as all");
    let staff_page = count(Some("staff")).await;
    let roles: Vec<&str> = staff_page.users.iter().map(|u| u.role.as_str()).collect();
    assert_eq!(
        roles,
        ["super_admin", "tier_list_admin", "tier_list_editor"]
    );
    assert_eq!(staff_page.total, 3);
    assert_eq!(count(Some(" translators ")).await.total, 1);
    assert_eq!(count(Some("players")).await.total, 2);
    assert!(matches!(
        list_users(&ctx, &admin, &query(Some("admins")), page(None, None)).await,
        Err(AppError::BadRequest(m))
            if m == "role must be `all`, `staff`, `translators` or `players`, got `admins`"
    ));

    let search = AdminUsersQuery {
        q: Some(" 5 ".into()),
        ..AdminUsersQuery::default()
    };
    let found = list_users(&ctx, &admin, &search, page(None, None))
        .await
        .unwrap();
    assert_eq!(found.users[0].uid, "5", "an exact uid ranks first");
    assert_eq!(found.total, 2, "a uid prefix");
    let jp = AdminUsersQuery {
        server: Some(Server::JP),
        ..AdminUsersQuery::default()
    };
    let found = list_users(&ctx, &admin, &jp, page(None, None))
        .await
        .unwrap();
    assert_eq!((found.total, found.users[0].uid.as_str()), (1, "56"));

    let body = serde_json::to_value(&found).unwrap();
    assert_eq!(body["total"], 1);
    let row = &body["users"][0];
    let mut keys: Vec<&str> = row
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "avatar_id",
            "created_at",
            "grade",
            "id",
            "item_count",
            "level",
            "nick_number",
            "nickname",
            "operator_count",
            "public_profile",
            "role",
            "secretary",
            "secretary_skin_id",
            "server",
            "skin_count",
            "total_score",
            "uid",
            "updated_at",
        ]
    );
    assert_eq!(row["public_profile"], false);
    assert_eq!(row["operator_count"], json!(0));
    db.close().await;
}
