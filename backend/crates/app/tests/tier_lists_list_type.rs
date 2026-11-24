#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::tier_lists::{self as tl, CreateListRequest};
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_types::{ContentAuditAction, ContentTargetKind, Role, Server, TierListType};

#[tokio::test]
async fn only_list_admins_change_a_list_type() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let editor = common::user(&ctx, db.user(2, Server::EN).await, Role::TierListEditor).await;
    let req = CreateListRequest {
        name: "Gated".into(),
        description: None,
        list_type: TierListType::Community,
    };
    let list = tl::create(&ctx, &owner, &req).await.unwrap();

    for subject in [&owner, &editor] {
        assert!(matches!(
            tl::set_list_type(&ctx, subject, &list.slug, TierListType::Official).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        tl::set_list_type(
            &ctx,
            &Subject::Anonymous,
            &list.slug,
            TierListType::Official
        )
        .await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        tl::set_list_type(&ctx, &Subject::System, &list.slug, TierListType::Official).await,
        Err(AppError::Unauthorized)
    ));
    let detail = tl::load_detail(&ctx, &list.slug).await.unwrap();
    assert_eq!(detail.list.list_type, "community");
    db.close().await;
}

#[tokio::test]
async fn a_type_change_reaches_the_cached_detail_and_the_audit_once() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let admin = common::user(&ctx, db.user(2, Server::EN).await, Role::TierListAdmin).await;
    let req = CreateListRequest {
        name: "Promoted".into(),
        description: None,
        list_type: TierListType::Community,
    };
    let list = tl::create(&ctx, &owner, &req).await.unwrap();
    assert_eq!(
        tl::load_detail(&ctx, &list.slug)
            .await
            .unwrap()
            .list
            .list_type,
        "community"
    );

    assert!(
        tl::set_list_type(&ctx, &admin, &list.slug, TierListType::Official)
            .await
            .unwrap()
    );
    assert!(
        !tl::set_list_type(&ctx, &admin, &list.slug, TierListType::Official)
            .await
            .unwrap()
    );
    assert_eq!(
        tl::load_detail(&ctx, &list.slug)
            .await
            .unwrap()
            .list
            .list_type,
        "official"
    );
    assert!(matches!(
        tl::set_list_type(&ctx, &admin, "missing", TierListType::Official).await,
        Err(AppError::NotFound(None))
    ));

    let filter = ContentAuditFilter {
        actor: None,
        target_kind: Some(ContentTargetKind::TierList),
        target_id: Some(list.id),
    };
    let entries = ctx
        .audit
        .feed(&ctx.policy, &Subject::System, filter, 50, None)
        .await
        .unwrap()
        .entries;
    let changed: Vec<_> = entries
        .iter()
        .filter(|e| e.action == ContentAuditAction::Edit)
        .collect();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].details.list_type.as_deref(), Some("official"));

    assert!(
        tl::set_list_type(&ctx, &admin, &list.slug, TierListType::Community)
            .await
            .unwrap()
    );
    assert_eq!(
        tl::load_detail(&ctx, &list.slug)
            .await
            .unwrap()
            .list
            .list_type,
        "community"
    );
    db.close().await;
}
