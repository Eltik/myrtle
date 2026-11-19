#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::community::operator_notes::{
    self as notes, UpdateFields, audit_actor_scope,
};
use myrtle_types::{Role, Server, UserId};
use uuid::Uuid;

fn pros(text: &str) -> UpdateFields {
    UpdateFields {
        pros: Some(text.into()),
        ..UpdateFields::default()
    }
}

#[tokio::test]
async fn staff_edit_notes_and_each_change_is_audited() {
    let (ctx, db) = common::context().await;
    let editor_id = db.user(1, Server::EN).await;
    let editor = common::user(&ctx, editor_id, Role::TierListEditor).await;

    let first = notes::update(&ctx, &editor, "char_002_amiya", pros("fast"))
        .await
        .unwrap();
    assert_eq!(first.pros.as_deref(), Some("fast"));
    let same = notes::update(&ctx, &editor, "char_002_amiya", pros("fast"))
        .await
        .unwrap();
    assert_eq!(same.id, first.id);
    let second = UpdateFields {
        pros: Some("faster".into()),
        summary: Some("caster".into()),
        ..UpdateFields::default()
    };
    notes::update(&ctx, &editor, "char_002_amiya", second)
        .await
        .unwrap();

    let read = notes::get(&ctx, "char_002_amiya").await.unwrap();
    assert_eq!(read.pros.as_deref(), Some("faster"));
    assert_eq!(read.summary.as_deref(), Some("caster"));
    assert_eq!(notes::list(&ctx).await.unwrap().len(), 1);
    assert!(matches!(
        notes::get(&ctx, "char_nobody").await,
        Err(AppError::NotFound(None))
    ));

    let log = notes::audit_log(&ctx, &editor, "char_002_amiya")
        .await
        .unwrap();
    let mut fields: Vec<(&str, Option<&str>)> = log
        .iter()
        .map(|e| (e.field_name.as_str(), e.old_value.as_deref()))
        .collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        [("pros", None), ("pros", Some("fast")), ("summary", None)]
    );
    assert!(
        log.iter()
            .all(|e| e.changed_by == editor_id.0 && e.actor.id == editor_id.0)
    );
    db.close().await;
}

#[tokio::test]
async fn only_staff_edit_notes_or_read_their_history() {
    let (ctx, db) = common::context().await;
    let user = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let translator = common::user(&ctx, db.user(2, Server::EN).await, Role::Translator).await;

    for subject in [&user, &translator] {
        assert!(matches!(
            notes::update(&ctx, subject, "char_002_amiya", pros("x")).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            notes::audit_log(&ctx, subject, "char_002_amiya").await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            notes::global_audit_log(&ctx, subject, None, None, None).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        notes::update(&ctx, &Subject::Anonymous, "char_002_amiya", pros("x")).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        notes::get(&ctx, "char_002_amiya").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_global_feed_is_scoped_to_what_the_caller_may_read() {
    let (ctx, db) = common::context().await;
    let editor_id = db.user(1, Server::EN).await;
    let admin_id = db.user(2, Server::EN).await;
    let editor = common::user(&ctx, editor_id, Role::TierListEditor).await;
    let admin = common::user(&ctx, admin_id, Role::TierListAdmin).await;
    notes::update(&ctx, &editor, "char_002_amiya", pros("a"))
        .await
        .unwrap();
    notes::update(&ctx, &admin, "char_003_kalts", pros("b"))
        .await
        .unwrap();

    let everyone = notes::global_audit_log(&ctx, &admin, None, None, None)
        .await
        .unwrap();
    assert_eq!((everyone.total, everyone.entries.len()), (2, 2));
    let paged = notes::global_audit_log(&ctx, &admin, Some(0), None, None)
        .await
        .unwrap();
    assert_eq!((paged.total, paged.entries.len()), (2, 1));

    let own = notes::global_audit_log(&ctx, &editor, None, None, None)
        .await
        .unwrap();
    assert_eq!(own.total, 1);
    assert_eq!(own.entries[0].actor.user_id, editor_id.0);
    assert_eq!(own.entries[0].operator_id, "char_002_amiya");
    assert!(matches!(
        notes::global_audit_log(&ctx, &editor, None, None, Some(&admin_id.0.to_string())).await,
        Err(AppError::Forbidden)
    ));

    let named = notes::global_audit_log(&ctx, &admin, None, None, Some(&editor_id.0.to_string()))
        .await
        .unwrap();
    assert_eq!(named.total, 1);
    db.close().await;
}

#[tokio::test]
async fn audit_scope_rule() {
    let (ctx, db) = common::context().await;
    let me = UserId(Uuid::from_u128(1));
    let other = UserId(Uuid::from_u128(2));
    let other_text = other.0.to_string();
    let me_text = me.0.to_string();

    let admin = common::user(&ctx, me, Role::TierListAdmin).await;
    assert_eq!(audit_actor_scope(&ctx, &admin, None).ok(), Some(None));
    assert_eq!(audit_actor_scope(&ctx, &admin, Some(" ")).ok(), Some(None));
    assert_eq!(
        audit_actor_scope(&ctx, &admin, Some("me")).ok(),
        Some(Some(me))
    );
    assert_eq!(
        audit_actor_scope(&ctx, &admin, Some(&other_text)).ok(),
        Some(Some(other))
    );
    assert!(matches!(
        audit_actor_scope(&ctx, &admin, Some("nobody")),
        Err(AppError::BadRequest(m)) if m == "actor must be `me` or an account id, got `nobody`"
    ));

    let editor = common::user(&ctx, me, Role::TierListEditor).await;
    assert_eq!(audit_actor_scope(&ctx, &editor, None).ok(), Some(Some(me)));
    assert_eq!(
        audit_actor_scope(&ctx, &editor, Some("me")).ok(),
        Some(Some(me))
    );
    assert_eq!(
        audit_actor_scope(&ctx, &editor, Some(&me_text)).ok(),
        Some(Some(me))
    );
    assert!(matches!(
        audit_actor_scope(&ctx, &editor, Some(&other_text)),
        Err(AppError::Forbidden)
    ));

    let super_admin = common::user(&ctx, me, Role::SuperAdmin).await;
    assert_eq!(audit_actor_scope(&ctx, &super_admin, None).ok(), Some(None));
    for role in [Role::User, Role::Translator] {
        assert!(matches!(
            audit_actor_scope(&ctx, &common::user(&ctx, me, role).await, None),
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        audit_actor_scope(&ctx, &Subject::Anonymous, None),
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}
