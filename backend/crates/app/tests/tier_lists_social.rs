#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppContext;
use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::tier_lists::{
    self as tl, AddPlacementRequest, CreateFlairRequest, CreateListRequest, MovePlacementRequest,
    TierList, TierRequest,
};
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_testkit::fixtures::has_story_avg;
use myrtle_types::{ContentAuditAction, ContentTargetKind, EntityKind, Role, Server, TierListType};

async fn new_list(ctx: &AppContext, owner: &Subject, name: &str) -> TierList {
    let req = CreateListRequest {
        name: name.to_owned(),
        description: None,
        list_type: TierListType::Community,
    };
    tl::create(ctx, owner, &req).await.unwrap()
}

fn tier(name: &str, order: i16) -> TierRequest {
    TierRequest {
        name: name.to_owned(),
        display_order: order,
        color: None,
        description: None,
    }
}

#[test]
fn the_session_hash_is_hmac_sha256() {
    assert_eq!(
        tl::session_hash("Jefe", "what do ya want for nothing?"),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    assert_eq!(
        tl::session_hash(&"k".repeat(64), "m"),
        "3b7a8d453e76edb519238a515105a57508d0f169f480ebeee120a9e7803289fa"
    );
}

#[test]
fn a_key_longer_than_a_block_is_hashed_first() {
    assert_eq!(
        tl::session_hash(&"k".repeat(131), "m"),
        "4e214422d91081ee13c979c05d9a47f6c13d8b6d334cb4b39ab10532da02f673"
    );
}

#[tokio::test]
async fn a_view_needs_an_account_or_a_session_and_dedupes() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = new_list(&ctx, &owner, "Viewed").await;
    let slug = list.slug.as_str();
    let session = "ab".repeat(32);

    let nobody = tl::record_view(&ctx, &Subject::Anonymous, slug, None, None)
        .await
        .unwrap();
    assert!(!nobody.unique);
    let malformed = tl::record_view(&ctx, &Subject::Anonymous, slug, Some("short"), None)
        .await
        .unwrap();
    assert!(!malformed.unique);
    assert_eq!(tl::stats(&ctx, slug).await.unwrap().view_count, 0);

    let first = tl::record_view(&ctx, &Subject::Anonymous, slug, Some(&session), None)
        .await
        .unwrap();
    let again = tl::record_view(&ctx, &Subject::Anonymous, slug, Some(&session), None)
        .await
        .unwrap();
    let account = tl::record_view(&ctx, &owner, slug, Some(&session), None)
        .await
        .unwrap();
    assert!(first.unique && !again.unique && account.unique);
    let stats = tl::stats(&ctx, slug).await.unwrap();
    assert_eq!((stats.view_count, stats.unique_view_count), (2, 2));
    assert!(matches!(
        tl::record_view(&ctx, &owner, "missing", None, None).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn an_address_gets_ten_views_a_minute() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = new_list(&ctx, &owner, "Busy").await;
    let ip = "198.51.100.77";
    for _ in 0..10 {
        tl::record_view(&ctx, &Subject::Anonymous, &list.slug, None, Some(ip))
            .await
            .unwrap();
    }
    assert!(matches!(
        tl::record_view(&ctx, &Subject::Anonymous, &list.slug, None, Some(ip)).await,
        Err(AppError::RateLimited)
    ));
    tl::record_view(
        &ctx,
        &Subject::Anonymous,
        &list.slug,
        None,
        Some("198.51.100.78"),
    )
    .await
    .unwrap();
    db.close().await;
}

#[tokio::test]
async fn favourites_toggle_and_reach_the_cached_detail() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let fan = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let list = new_list(&ctx, &owner, "Loved").await;
    let slug = list.slug.as_str();
    tl::load_detail(&ctx, slug).await.unwrap();

    assert!(
        tl::toggle_favorite(&ctx, &fan, slug)
            .await
            .unwrap()
            .favorited
    );
    assert!(tl::favorite(&ctx, &fan, slug).await.unwrap().favorited);
    let detail = tl::load_detail(&ctx, slug).await.unwrap();
    assert_eq!(detail.stats.unwrap().favorite_count, 1);
    assert_eq!(tl::favorites(&ctx, &fan).await.unwrap()[0].slug, slug);

    assert!(
        !tl::toggle_favorite(&ctx, &fan, slug)
            .await
            .unwrap()
            .favorited
    );
    assert_eq!(
        tl::load_detail(&ctx, slug)
            .await
            .unwrap()
            .stats
            .unwrap()
            .favorite_count,
        0
    );
    assert!(matches!(
        tl::toggle_favorite(&ctx, &Subject::Anonymous, slug).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn flairs_are_made_by_list_admins_and_set_by_editors() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let admin = common::user(&ctx, db.user(2, Server::EN).await, Role::TierListAdmin).await;
    let list = new_list(&ctx, &owner, "Flaired").await;
    let req = CreateFlairRequest {
        code: "meta".into(),
        label: "Meta".into(),
        color: Some("#123abc".into()),
        display_order: None,
    };
    assert!(matches!(
        tl::create_flair(&ctx, &owner, &req).await,
        Err(AppError::Forbidden)
    ));
    let bad = CreateFlairRequest {
        color: Some("blue".into()),
        ..req.clone()
    };
    assert!(matches!(
        tl::create_flair(&ctx, &admin, &bad).await,
        Err(AppError::BadRequest(_))
    ));
    let flair = tl::create_flair(&ctx, &admin, &req).await.unwrap();
    assert!(
        tl::flairs(&ctx)
            .await
            .unwrap()
            .iter()
            .any(|f| f.id == flair.id)
    );

    tl::load_detail(&ctx, &list.slug).await.unwrap();
    tl::set_flair(&ctx, &owner, &list.slug, Some(flair.id))
        .await
        .unwrap();
    tl::set_flair(&ctx, &owner, &list.slug, Some(flair.id))
        .await
        .unwrap();
    assert_eq!(
        tl::load_detail(&ctx, &list.slug)
            .await
            .unwrap()
            .flair
            .unwrap()
            .label,
        "Meta"
    );

    let filter = ContentAuditFilter {
        actor: None,
        target_kind: Some(ContentTargetKind::TierList),
        target_id: Some(list.id),
    };
    let entries = ctx
        .audit
        .feed(&ctx.policy, &myrtle_app::Subject::System, filter, 50, None)
        .await
        .unwrap()
        .entries;
    let flaired: Vec<_> = entries
        .iter()
        .filter(|e| e.action == ContentAuditAction::Flair)
        .collect();
    assert_eq!(flaired.len(), 1);
    assert_eq!(flaired[0].details.flair.as_deref(), Some("Meta"));
    db.close().await;
}

#[tokio::test]
async fn placements_resolve_move_and_leave_an_audit_trail() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = new_list(&ctx, &owner, "Placed").await;
    let slug = list.slug.as_str();
    let s = tl::create_tier(&ctx, &owner, slug, &tier("S", 0))
        .await
        .unwrap();
    let a = tl::create_tier(&ctx, &owner, slug, &tier("A", 1))
        .await
        .unwrap();
    let add = |id: &str| AddPlacementRequest {
        tier_id: s.id,
        entity_kind: EntityKind::Operator,
        entity_id: id.to_owned(),
        sub_order: None,
        description: Some("note".into()),
    };

    assert!(matches!(
        tl::add_placement(&ctx, &owner, slug, &add("char_999_nobody")).await,
        Err(AppError::BadRequest(_))
    ));
    let story = AddPlacementRequest {
        entity_kind: EntityKind::StoryCg,
        ..add("avg_nope_0")
    };
    assert!(matches!(
        tl::add_placement(&ctx, &owner, slug, &story).await,
        Err(AppError::BadRequest(_))
    ));
    let foreign = AddPlacementRequest {
        tier_id: uuid::Uuid::new_v4(),
        ..add("char_002_amiya")
    };
    assert!(matches!(
        tl::add_placement(&ctx, &owner, slug, &foreign).await,
        Err(AppError::NotFound(None))
    ));
    tl::add_placement(&ctx, &owner, slug, &add("char_002_amiya"))
        .await
        .unwrap();

    let detail = tl::get(&ctx, slug, Some(Server::EN)).await.unwrap();
    let placed = &detail.tiers[0].placements[0];
    assert_eq!(placed.entity.as_ref().unwrap().name, "Amiya");
    assert!(
        tl::load_detail(&ctx, slug).await.unwrap().tiers[0].placements[0]
            .entity
            .is_none()
    );

    let to_a = MovePlacementRequest {
        new_tier_id: a.id,
        sub_order: Some(2),
    };
    let moved = tl::move_placement(
        &ctx,
        &owner,
        slug,
        EntityKind::Operator,
        "char_002_amiya",
        &to_a,
    )
    .await
    .unwrap();
    assert_eq!((moved.tier_id, moved.sub_order), (a.id, 2));
    assert_eq!(moved.description.as_deref(), Some("note"));
    let stay = MovePlacementRequest {
        new_tier_id: a.id,
        sub_order: Some(0),
    };
    tl::move_placement(
        &ctx,
        &owner,
        slug,
        EntityKind::Operator,
        "char_002_amiya",
        &stay,
    )
    .await
    .unwrap();
    let away = MovePlacementRequest {
        new_tier_id: uuid::Uuid::new_v4(),
        sub_order: None,
    };
    assert!(matches!(
        tl::move_placement(
            &ctx,
            &owner,
            slug,
            EntityKind::Operator,
            "char_002_amiya",
            &away
        )
        .await,
        Err(AppError::BadRequest(_))
    ));
    tl::set_placement_description(
        &ctx,
        &owner,
        slug,
        EntityKind::Operator,
        "char_002_amiya",
        None,
    )
    .await
    .unwrap();
    tl::remove_placement(&ctx, &owner, slug, EntityKind::Operator, "char_002_amiya")
        .await
        .unwrap();
    tl::remove_placement(&ctx, &owner, slug, EntityKind::Operator, "char_002_amiya")
        .await
        .unwrap();
    assert!(matches!(
        tl::move_placement(
            &ctx,
            &owner,
            slug,
            EntityKind::Operator,
            "char_002_amiya",
            &to_a
        )
        .await,
        Err(AppError::NotFound(None))
    ));
    assert!(
        tl::get(&ctx, slug, None).await.unwrap().tiers[1]
            .placements
            .is_empty()
    );

    let filter = ContentAuditFilter {
        actor: None,
        target_kind: Some(ContentTargetKind::TierList),
        target_id: Some(list.id),
    };
    let edit = ctx
        .audit
        .feed(&ctx.policy, &myrtle_app::Subject::System, filter, 50, None)
        .await
        .unwrap()
        .entries
        .remove(0);
    let key = vec!["operator:char_002_amiya".to_owned()];
    assert_eq!(edit.details.entries_added.as_ref(), Some(&key));
    assert_eq!(edit.details.entries_moved.as_ref(), Some(&key));
    assert_eq!(edit.details.entries_reordered.as_ref(), Some(&key));
    assert_eq!(edit.details.notes_edited.as_ref(), Some(&key));
    assert_eq!(edit.details.entries_removed.as_ref(), Some(&key));

    let catalogue = tl::catalogue(&ctx, EntityKind::Operator, Some(Server::EN))
        .await
        .unwrap();
    let none = &tl::entities::StoryCatalogues::NONE;
    assert_eq!(
        catalogue.len(),
        tl::entities::catalogue(&snapshot.model, none, EntityKind::Operator).len()
    );
    db.close().await;
}

#[tokio::test]
async fn story_kinds_place_and_resolve_against_the_story_catalogues() {
    if !has_story_avg() {
        eprintln!(
            "skipping story_kinds_place_and_resolve_against_the_story_catalogues: no en/textures/avg"
        );
        return;
    }
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let list = new_list(&ctx, &owner, "Story").await;
    let slug = list.slug.as_str();
    let s = tl::create_tier(&ctx, &owner, slug, &tier("S", 0))
        .await
        .unwrap();
    let pics = tl::catalogue(&ctx, EntityKind::ArchivePic, Some(Server::EN))
        .await
        .unwrap();
    let chapter = tl::catalogue(&ctx, EntityKind::MainStory, Some(Server::EN))
        .await
        .unwrap()
        .remove(0);
    let sprite = &snapshot
        .model
        .assets
        .story_sprites()
        .iter()
        .next()
        .unwrap()
        .id;
    for (kind, id) in [
        (EntityKind::ArchivePic, pics[0].id.as_str()),
        (EntityKind::MainStory, chapter.id.as_str()),
        (EntityKind::StorySprite, sprite.as_str()),
    ] {
        let req = AddPlacementRequest {
            tier_id: s.id,
            entity_kind: kind,
            entity_id: id.to_owned(),
            sub_order: None,
            description: None,
        };
        tl::add_placement(&ctx, &owner, slug, &req).await.unwrap();
        let unknown = AddPlacementRequest {
            entity_id: "nope_0".to_owned(),
            ..req
        };
        assert!(matches!(
            tl::add_placement(&ctx, &owner, slug, &unknown).await,
            Err(AppError::BadRequest(_))
        ));
    }
    let detail = tl::get(&ctx, slug, Some(Server::EN)).await.unwrap();
    let placed = &detail.tiers[0].placements;
    assert_eq!(placed.len(), 3);
    assert!(placed.iter().all(|p| p.entity.is_some()), "{placed:?}");
    assert_eq!(placed[0].entity.as_ref().unwrap(), &pics[0]);
    db.close().await;
}
