#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppContext;
use myrtle_app::AppError;
use myrtle_app::audit::ContentAuditEntry;
use myrtle_app::policy::Subject;
use myrtle_app::services::tier_lists::{
    self as tl, CreateListRequest, GrantRequest, PlacementDetail, PublishRequest, TierRequest,
    UpdateListRequest,
};
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_db::repos::{tier_lists, users};
use myrtle_types::{
    ApiKeyId, ContentAuditAction, ContentTargetKind, EntityKind, Role, Server, TierListId,
    TierListType, UserId,
};
use uuid::Uuid;

fn create_req(name: &str, list_type: TierListType) -> CreateListRequest {
    CreateListRequest {
        name: name.to_owned(),
        description: None,
        list_type,
    }
}

fn tier_req(name: &str, order: i16) -> TierRequest {
    TierRequest {
        name: name.to_owned(),
        display_order: order,
        color: None,
        description: None,
    }
}

async fn audit(ctx: &AppContext, list: Uuid) -> Vec<ContentAuditEntry> {
    let filter = ContentAuditFilter {
        actor: None,
        target_kind: Some(ContentTargetKind::TierList),
        target_id: Some(list),
    };
    ctx.audit
        .feed(&ctx.policy, &myrtle_app::Subject::System, filter, 50, None)
        .await
        .unwrap()
        .entries
}

async fn grant_row(ctx: &AppContext, list: Uuid, user: UserId, permission: &str, by: UserId) {
    let mut conn = ctx.db.conn().await.unwrap();
    tier_lists::grant_permission(&mut conn, TierListId(list), user, permission, by)
        .await
        .unwrap();
}

#[test]
fn a_slug_is_the_name_in_dashes_and_six_random_characters() {
    let slug = tl::generate_slug("  Best   Guards, 2026!! ");
    let (base, suffix) = slug.rsplit_once('-').unwrap();
    assert_eq!(base, "best-guards-2026");
    assert_eq!(suffix.len(), 6);
    assert!(
        suffix
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase())
    );
    assert_ne!(tl::generate_slug("x"), tl::generate_slug("x"));
    assert!(tl::generate_slug("Été").starts_with("été-"));
}

#[test]
fn offered_kinds_dedupe_in_order_and_refuse_none() {
    let kinds =
        tl::offered_kinds(&[EntityKind::Enemy, EntityKind::Operator, EntityKind::Enemy]).unwrap();
    assert_eq!(kinds, [EntityKind::Enemy, EntityKind::Operator]);
    assert!(matches!(
        tl::offered_kinds(&[]),
        Err(AppError::BadRequest(_))
    ));
}

#[test]
fn a_color_is_exactly_hash_and_six_hex_digits() {
    assert!(tl::validate_hex_color(None).is_ok());
    assert!(tl::validate_hex_color(Some("#dc4D56")).is_ok());
    for bad in [
        "dc4d56",
        "#dc4d5",
        "#dc4d567",
        "#gggggg",
        "red url('x')",
        "",
    ] {
        assert!(tl::validate_hex_color(Some(bad)).is_err(), "{bad}");
    }
}

#[test]
fn a_length_limit_counts_characters_not_bytes() {
    assert!(tl::validate_length("tier name", Some(&"é".repeat(40)), 40).is_ok());
    match tl::validate_length("tier name", Some(&"a".repeat(41)), 40) {
        Err(AppError::BadRequest(m)) => assert_eq!(m, "tier name max 40 chars"),
        other => panic!("{other:?}"),
    }
    assert!(tl::validate_length("x", None, 0).is_ok());
}

#[test]
fn an_old_snapshot_reads_as_operators() {
    let old = serde_json::json!([{
        "id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
        "tier_list_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02",
        "name": "S",
        "display_order": 0,
        "color": "#dc4d56",
        "description": null,
        "placements": [{
            "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
            "operator_id": "char_002_amiya",
            "sub_order": 3,
            "description": "why",
            "updated_at": "2026-01-02T03:04:05Z"
        }]
    }]);
    let tiers = tl::read_snapshot(&old).unwrap();
    let p = &tiers[0].placements[0];
    assert_eq!(p.entity_kind, EntityKind::Operator);
    assert_eq!(p.entity_id, "char_002_amiya");
    assert_eq!(p.sub_order, 3);
    assert_eq!(p.description.as_deref(), Some("why"));
}

#[test]
fn a_cache_entry_from_before_kinds_still_reads() {
    let old = serde_json::json!({
        "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
        "operator_id": "char_002_amiya",
        "sub_order": 2,
        "description": null,
        "updated_at": "2026-01-02T03:04:05Z"
    });
    let p: PlacementDetail = serde_json::from_value(old).unwrap();
    assert_eq!(p.placement.entity_kind, EntityKind::Operator);
    assert_eq!(p.placement.entity_id, "char_002_amiya");
    assert!(p.entity.is_none());
}

#[test]
fn a_new_snapshot_round_trips_without_resolved_entities() {
    let old = serde_json::json!([{
        "id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
        "tier_list_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02",
        "name": "S", "display_order": 0, "color": null, "description": null,
        "placements": [{
            "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
            "operator_id": "char_002_amiya",
            "sub_order": 0, "description": null,
            "updated_at": "2026-01-02T03:04:05Z"
        }]
    }]);
    let written = serde_json::to_value(tl::read_snapshot(&old).unwrap()).unwrap();
    let placement = &written[0]["placements"][0];
    assert_eq!(placement["entity_kind"], "operator");
    assert_eq!(placement["entity_id"], "char_002_amiya");
    assert!(placement.get("operator_id").is_none());
    assert!(placement.get("entity").is_none());
    assert_eq!(
        tl::read_snapshot(&written).unwrap()[0].placements[0].entity_id,
        "char_002_amiya"
    );
}

#[tokio::test]
async fn creation_is_gated_by_list_type_and_capped_for_players() {
    let (ctx, db) = common::context().await;
    let player = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let editor = common::user(&ctx, db.user(2, Server::EN).await, Role::TierListEditor).await;

    assert!(matches!(
        tl::create(
            &ctx,
            &Subject::Anonymous,
            &create_req("a", TierListType::Community)
        )
        .await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        tl::create(&ctx, &player, &create_req("a", TierListType::Official)).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::create(
            &ctx,
            &player,
            &create_req(&"n".repeat(81), TierListType::Community)
        )
        .await,
        Err(AppError::BadRequest(_))
    ));
    for i in 0..10 {
        let list = tl::create(
            &ctx,
            &player,
            &create_req(&format!("Mine {i}"), TierListType::Community),
        )
        .await
        .unwrap();
        assert_eq!(list.list_type, "community");
        assert!(list.slug.starts_with(&format!("mine-{i}-")));
    }
    assert!(matches!(
        tl::create(
            &ctx,
            &player,
            &create_req("eleventh", TierListType::Community)
        )
        .await,
        Err(AppError::Conflict(_))
    ));
    for i in 0..11 {
        tl::create(
            &ctx,
            &editor,
            &create_req(&format!("Staff {i}"), TierListType::Official),
        )
        .await
        .unwrap();
    }

    let list = tl::create(
        &ctx,
        &editor,
        &create_req("Audited", TierListType::Official),
    )
    .await
    .unwrap();
    let entries = audit(&ctx, list.id).await;
    assert_eq!(entries[0].action, ContentAuditAction::Create);
    assert_eq!(entries[0].details.list_type.as_deref(), Some("official"));
    assert!(tl::stats(&ctx, &list.slug).await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn edits_need_ownership_a_grant_or_a_staff_role() {
    let (ctx, db) = common::context().await;
    let owner_id = db.user(1, Server::EN).await;
    let owner = common::user(&ctx, owner_id, Role::User).await;
    let stranger_id = db.user(2, Server::EN).await;
    let stranger = common::user(&ctx, stranger_id, Role::User).await;
    let admin = common::user(&ctx, db.user(3, Server::EN).await, Role::TierListAdmin).await;
    let editor_id = db.user(4, Server::EN).await;
    let editor = common::user(&ctx, editor_id, Role::TierListEditor).await;
    let list = tl::create(&ctx, &owner, &create_req("Gated", TierListType::Community))
        .await
        .unwrap();
    let slug = list.slug.as_str();

    let rename = |name: &str| UpdateListRequest {
        name: name.to_owned(),
        description: None,
        entity_kinds: None,
    };
    assert!(matches!(
        tl::update(&ctx, &stranger, slug, &rename("x")).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::update(&ctx, &Subject::Anonymous, slug, &rename("x")).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        tl::update(&ctx, &owner, "no-such-list", &rename("x")).await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        tl::update(&ctx, &editor, slug, &rename("x")).await,
        Err(AppError::Forbidden)
    ));
    grant_row(&ctx, list.id, editor_id, "edit", owner_id).await;
    tl::update(&ctx, &editor, slug, &rename("By editor"))
        .await
        .unwrap();
    tl::update(&ctx, &admin, slug, &rename("By admin"))
        .await
        .unwrap();

    let tier = tl::create_tier(&ctx, &editor, slug, &tier_req("S", 0))
        .await
        .unwrap();
    assert!(matches!(
        tl::delete_tier(&ctx, &editor, slug, tier.id).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::publish(&ctx, &editor, slug, &PublishRequest { changelog: None }).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::delete(&ctx, &editor, slug).await,
        Err(AppError::Forbidden)
    ));
    tl::delete_tier(&ctx, &owner, slug, tier.id).await.unwrap();
    assert!(matches!(
        tl::delete_tier(&ctx, &owner, slug, tier.id).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_write_invalidates_the_cached_detail() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = tl::create(&ctx, &owner, &create_req("Cached", TierListType::Community))
        .await
        .unwrap();
    let slug = list.slug.as_str();
    assert!(tl::load_detail(&ctx, slug).await.unwrap().tiers.is_empty());

    tl::create_tier(&ctx, &owner, slug, &tier_req("S", 0))
        .await
        .unwrap();
    let detail = tl::load_detail(&ctx, slug).await.unwrap();
    assert_eq!(detail.tiers.len(), 1);
    assert_eq!(detail.author.unwrap().uid, "1");

    let update = UpdateListRequest {
        name: "Renamed".into(),
        description: Some("d".into()),
        entity_kinds: Some(vec![EntityKind::Enemy, EntityKind::Enemy]),
    };
    tl::update(&ctx, &owner, slug, &update).await.unwrap();
    let detail = tl::load_detail(&ctx, slug).await.unwrap();
    assert_eq!(detail.list.name, "Renamed");
    assert_eq!(detail.list.entity_kinds, [EntityKind::Enemy]);

    let edit = &audit(&ctx, list.id).await[0];
    assert_eq!(edit.action, ContentAuditAction::Edit);
    assert_eq!(edit.details.renamed_from.as_deref(), Some("Cached"));
    assert_eq!(edit.details.description_changed, Some(true));
    assert_eq!(
        edit.details.kinds.as_deref(),
        Some(&["enemy".to_owned()][..])
    );
    assert_eq!(edit.details.tiers_added.as_ref().unwrap()[0].name, "S");

    let json = serde_json::to_value(tl::load_detail(&ctx, slug).await.unwrap()).unwrap();
    assert_eq!(json["slug"], slug);
    assert!(json["tiers"][0]["placements"].is_array());
    db.close().await;
}

#[tokio::test]
async fn tier_validation_runs_before_anything_is_written() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = tl::create(&ctx, &owner, &create_req("Tiers", TierListType::Community))
        .await
        .unwrap();
    let mut bad = tier_req(&"n".repeat(41), 0);
    assert!(
        tl::create_tier(&ctx, &owner, &list.slug, &bad)
            .await
            .is_err()
    );
    bad = TierRequest {
        color: Some("red".into()),
        ..tier_req("S", 0)
    };
    assert!(
        tl::create_tier(&ctx, &owner, &list.slug, &bad)
            .await
            .is_err()
    );
    assert!(
        tl::load_detail(&ctx, &list.slug)
            .await
            .unwrap()
            .tiers
            .is_empty()
    );

    let tier = tl::create_tier(&ctx, &owner, &list.slug, &tier_req("S", 0))
        .await
        .unwrap();
    let restyled = TierRequest {
        color: Some("#000000".into()),
        ..tier_req("S+", 1)
    };
    let updated = tl::update_tier(&ctx, &owner, &list.slug, tier.id, &restyled)
        .await
        .unwrap();
    assert_eq!(updated.name, "S+");
    assert!(matches!(
        tl::update_tier(&ctx, &owner, &list.slug, Uuid::new_v4(), &restyled).await,
        Err(AppError::NotFound(None))
    ));
    let details = &audit(&ctx, list.id).await[0].details;
    assert!(
        details
            .tiers_added
            .as_ref()
            .unwrap()
            .iter()
            .any(|t| t.name == "S+")
    );
    assert!(details.tiers_reordered.is_some());
    assert!(details.tiers_restyled.is_some());
    db.close().await;
}

#[tokio::test]
async fn each_publish_is_the_next_version_of_the_current_rows() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = tl::create(
        &ctx,
        &owner,
        &create_req("Versions", TierListType::Community),
    )
    .await
    .unwrap();
    tl::create_tier(&ctx, &owner, &list.slug, &tier_req("S", 0))
        .await
        .unwrap();
    let first = tl::publish(
        &ctx,
        &owner,
        &list.slug,
        &PublishRequest {
            changelog: Some(format!("  {}  ", "c".repeat(300))),
        },
    )
    .await
    .unwrap();
    let second = tl::publish(
        &ctx,
        &owner,
        &list.slug,
        &PublishRequest { changelog: None },
    )
    .await
    .unwrap();
    assert_eq!((first.version, second.version), (1, 2));
    assert_eq!(
        tl::read_snapshot(&first.snapshot).unwrap()[0].tier.name,
        "S"
    );

    let entries = audit(&ctx, list.id).await;
    let published: Vec<_> = entries
        .iter()
        .filter(|e| e.action == ContentAuditAction::Publish)
        .collect();
    assert_eq!(published.len(), 2);
    let changelog = published[1].details.changelog.as_deref().unwrap();
    assert_eq!(changelog.chars().count(), 280);
    assert!(!changelog.starts_with(' '));
    db.close().await;
}

#[tokio::test]
async fn deleting_a_list_records_what_went_with_it() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = tl::create(&ctx, &owner, &create_req("Gone", TierListType::Community))
        .await
        .unwrap();
    tl::create_tier(&ctx, &owner, &list.slug, &tier_req("S", 0))
        .await
        .unwrap();
    tl::create_tier(&ctx, &owner, &list.slug, &tier_req("A", 1))
        .await
        .unwrap();
    tl::load_detail(&ctx, &list.slug).await.unwrap();
    tl::delete(&ctx, &owner, &list.slug).await.unwrap();
    assert!(matches!(
        tl::load_detail(&ctx, &list.slug).await,
        Err(AppError::NotFound(None))
    ));
    let deleted = &audit(&ctx, list.id).await[0];
    assert_eq!(deleted.action, ContentAuditAction::Delete);
    assert_eq!(
        (deleted.details.tiers, deleted.details.entries),
        (Some(2), Some(0))
    );
    db.close().await;
}

#[tokio::test]
async fn grants_are_made_by_list_admins_for_any_user() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let admin_id = db.user(2, Server::EN).await;
    let admin = common::user(&ctx, admin_id, Role::TierListAdmin).await;
    let player_id = db.user(3, Server::EN).await;
    let editor_id = db.user(4, Server::EN).await;
    users::set_role(
        &mut ctx.db.conn().await.unwrap(),
        editor_id,
        "tier_list_editor",
    )
    .await
    .unwrap();
    let list = tl::create(&ctx, &owner, &create_req("Shared", TierListType::Community))
        .await
        .unwrap();
    let req = |user: UserId, permission: &str| GrantRequest {
        user_id: user.0,
        permission: permission.to_owned(),
    };

    assert!(matches!(
        tl::grant(&ctx, &owner, &list.slug, &req(editor_id, "edit")).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::permissions(&ctx, &owner, &list.slug).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        tl::grant(&ctx, &admin, &list.slug, &req(editor_id, "owner")).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        tl::grant(
            &ctx,
            &admin,
            &list.slug,
            &req(UserId(Uuid::new_v4()), "edit")
        )
        .await,
        Err(AppError::NotFound(Some(_)))
    ));
    tl::grant(&ctx, &admin, &list.slug, &req(editor_id, "Publish"))
        .await
        .unwrap();

    let perms = tl::permissions(&ctx, &admin, &list.slug).await.unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms[0].permission, "publish");
    assert_eq!(perms[0].granted_by, Some(admin_id.0));
    let editor = common::user(&ctx, editor_id, Role::TierListEditor).await;
    let granted = tl::granted(&ctx, &editor).await.unwrap();
    assert_eq!(granted[0].slug, list.slug);
    assert_eq!(
        tl::all_grants(&ctx, &admin, Some(editor_id))
            .await
            .unwrap()
            .len(),
        1
    );
    tl::publish(
        &ctx,
        &editor,
        &list.slug,
        &PublishRequest { changelog: None },
    )
    .await
    .unwrap();

    tl::revoke(&ctx, &admin, &list.slug, editor_id, "publish")
        .await
        .unwrap();
    assert!(
        tl::permissions(&ctx, &admin, &list.slug)
            .await
            .unwrap()
            .is_empty()
    );
    let entries = audit(&ctx, list.id).await;
    let revoke = entries
        .iter()
        .find(|e| e.action == ContentAuditAction::Revoke)
        .unwrap();
    assert_eq!(revoke.subject.as_ref().unwrap().user_id, editor_id.0);
    assert_eq!(revoke.details.permission.as_deref(), Some("publish"));
    assert!(
        entries
            .iter()
            .any(|e| e.action == ContentAuditAction::Grant)
    );

    tl::grant(&ctx, &admin, &list.slug, &req(player_id, "edit"))
        .await
        .unwrap();
    let player = common::user(&ctx, player_id, Role::User).await;
    tl::set_visibility(&ctx, &player, &list.slug, false)
        .await
        .unwrap();
    let service = Subject::ApiKey {
        id: ApiKeyId(Uuid::new_v4()),
        scopes: ["manage_tier_list_grants".to_owned()].into(),
        owner: Some(admin_id),
    };
    tl::revoke(&ctx, &service, &list.slug, player_id, "edit")
        .await
        .unwrap();
    assert!(matches!(
        tl::set_visibility(&ctx, &player, &list.slug, true).await,
        Err(AppError::Forbidden)
    ));
    let revokes = audit(&ctx, list.id)
        .await
        .into_iter()
        .filter(|e| e.action == ContentAuditAction::Revoke)
        .count();
    assert_eq!(revokes, 1, "a service revoke leaves no audit row");
    db.close().await;
}

#[tokio::test]
async fn listings_follow_ownership_and_visibility() {
    let (ctx, db) = common::context().await;
    let owner_id = db.user(1, Server::EN).await;
    let owner = common::user(&ctx, owner_id, Role::User).await;
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let shown = tl::create(&ctx, &owner, &create_req("Shown", TierListType::Community))
        .await
        .unwrap();
    let hidden = tl::create(&ctx, &owner, &create_req("Hidden", TierListType::Community))
        .await
        .unwrap();
    tl::create(&ctx, &other, &create_req("Other", TierListType::Community))
        .await
        .unwrap();
    tl::set_visibility(&ctx, &owner, &hidden.slug, false)
        .await
        .unwrap();
    tl::set_visibility(&ctx, &owner, &hidden.slug, false)
        .await
        .unwrap();
    assert!(matches!(
        tl::set_visibility(&ctx, &other, &shown.slug, false).await,
        Err(AppError::Forbidden)
    ));

    let listed: Vec<String> = tl::list(&ctx)
        .await
        .unwrap()
        .into_iter()
        .map(|l| l.name)
        .collect();
    assert!(listed.contains(&"Shown".to_owned()));
    assert!(!listed.contains(&"Hidden".to_owned()));
    assert_eq!(tl::mine(&ctx, &owner).await.unwrap().len(), 2);
    assert!(matches!(
        tl::mine(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    let visibility: Vec<_> = audit(&ctx, hidden.id)
        .await
        .into_iter()
        .filter(|e| e.action == ContentAuditAction::Visibility)
        .collect();
    assert_eq!(visibility.len(), 1);
    assert_eq!(visibility[0].details.listed, Some(false));
    db.close().await;
}
