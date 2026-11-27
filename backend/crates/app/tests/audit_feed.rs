#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::audit::{ContentAuditDetails, ContentEvent};
use myrtle_app::{AppError, Subject};
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_types::{ContentAuditAction, ContentTargetKind, Role, Server};
use uuid::Uuid;

#[tokio::test]
async fn the_feed_is_for_tier_list_admins_and_up() {
    let (ctx, db) = common::context().await;
    let actor = db.user(1, Server::EN).await;
    ctx.audit
        .record(ContentEvent {
            actor,
            target_kind: ContentTargetKind::Grid,
            target_id: Uuid::from_u128(5),
            target_name: "Grid",
            target_slug: Some("grid"),
            owner: Some(actor),
            subject: None,
            action: ContentAuditAction::Create,
            details: ContentAuditDetails::default(),
        })
        .await;

    let filter = ContentAuditFilter::default();
    let feed = |subject: Subject| {
        let ctx = ctx.clone();
        async move {
            ctx.audit
                .feed(&ctx.policy, &subject, filter, 50, None)
                .await
        }
    };
    for refused in [Role::User, Role::Translator, Role::TierListEditor] {
        assert!(matches!(
            feed(common::user(&ctx, actor, refused).await).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        feed(Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    for allowed in [Role::TierListAdmin, Role::SuperAdmin] {
        let page = feed(common::user(&ctx, actor, allowed).await)
            .await
            .unwrap();
        assert_eq!((page.total, page.entries.len()), (1, 1));
        assert_eq!(page.entries[0].action, ContentAuditAction::Create);
    }
    db.close().await;
}
