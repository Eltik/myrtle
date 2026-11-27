#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::policy::Subject;
use myrtle_app::privacy::UpdateSettingsRequest;
use myrtle_app::privacy::layout::{
    ProfileBackground, ProfileBackgroundKind, ProfileLayout, ProfileShowcase, ShowcaseBlock,
};
use myrtle_app::services::community::grids::{self, GridCellInput, GridInput};
use myrtle_app::services::community::showcase::{
    prune_for_visitor, save_profile, user_showcase, validate, validate_background, view,
};
use myrtle_app::services::tier_lists::{self as tl, CreateListRequest};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::planner::{self, PlanInput};
use myrtle_db::repos::{gacha, users};
use myrtle_types::{EntityKind, Role, Server, TierListType, UserId};
use serde_json::{Value, json};
use uuid::Uuid;

fn bad_request<T: std::fmt::Debug>(result: Result<T, AppError>) -> String {
    match result {
        Err(AppError::BadRequest(msg)) => msg,
        other => panic!("expected a 400, got {other:?}"),
    }
}

fn showcase(blocks: Value) -> ProfileShowcase {
    ProfileShowcase::normalize(&json!({ "blocks": blocks }))
}

fn background(kind: ProfileBackgroundKind, id: &str, elite: Option<u8>) -> ProfileBackground {
    ProfileBackground {
        kind,
        id: id.into(),
        focus_x: None,
        focus_y: None,
        scale: None,
        elite,
    }
}

async fn a_grid(ctx: &AppContext, owner: UserId) -> String {
    let input = GridInput {
        title: "Board".into(),
        description: None,
        rows: 1,
        cols: 1,
        cells: vec![GridCellInput {
            label: String::new(),
            entity_kind: None,
            entity_id: None,
        }],
        is_listed: true,
        entity_kinds: vec![EntityKind::Operator],
    };
    grids::create(
        ctx,
        &common::user(ctx, owner, Role::User).await,
        input,
        None,
    )
    .await
    .unwrap()
    .slug
}

async fn a_tier_list(ctx: &AppContext, owner: UserId) -> String {
    let req = CreateListRequest {
        name: "Best".into(),
        description: None,
        list_type: TierListType::Community,
    };
    tl::create(ctx, &common::user(ctx, owner, Role::User).await, &req)
        .await
        .unwrap()
        .slug
}

async fn a_plan(ctx: &AppContext, owner: UserId, operator: &str, shown: bool) -> Uuid {
    let empty = json!([]);
    let input = PlanInput {
        target_elite: 2,
        target_level: 90,
        target_skill_level: 7,
        target_skills: &empty,
        target_modules: &empty,
        display_on_profile: shown,
    };
    let mut conn = ctx.db.conn().await.unwrap();
    planner::upsert_plan(&mut conn, owner, operator, input, None)
        .await
        .unwrap()
        .id
}

#[tokio::test]
async fn grid_tier_list_and_plan_blocks_need_their_referent() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let other = db.user(2, Server::EN).await;
    let grid = a_grid(&ctx, owner).await;
    let list = a_tier_list(&ctx, owner).await;
    let gone_list = a_tier_list(&ctx, owner).await;
    tl::delete(
        &ctx,
        &common::user(&ctx, owner, Role::User).await,
        &gone_list,
    )
    .await
    .unwrap();
    let shown = a_plan(&ctx, owner, "char_002_amiya", true).await;
    let hidden = a_plan(&ctx, owner, "char_123_fang", false).await;
    let theirs = a_plan(&ctx, other, "char_103_angel", true).await;

    let ok = showcase(json!([
        { "type": "grid", "slug": grid },
        { "type": "tier_list", "slug": list },
        { "type": "plan", "id": shown.to_string() },
    ]));
    validate(&ctx, owner, &ok).await.unwrap();
    validate(&ctx, owner, &showcase(json!([]))).await.unwrap();

    for (block, expect) in [
        (
            json!({ "type": "grid", "slug": "nope-000000" }),
            "showcase block 2: no grid `nope-000000`",
        ),
        (
            json!({ "type": "tier_list", "slug": "nope-000000" }),
            "showcase block 2: no tier list `nope-000000`",
        ),
        (
            json!({ "type": "tier_list", "slug": gone_list }),
            "no tier list",
        ),
        (
            json!({ "type": "plan", "id": hidden.to_string() }),
            "is not one of your plans shown",
        ),
        (
            json!({ "type": "plan", "id": theirs.to_string() }),
            "is not one of your plans shown",
        ),
    ] {
        let blocks = showcase(json!([{ "type": "grid", "slug": grid }, block]));
        let msg = bad_request(validate(&ctx, owner, &blocks).await);
        assert!(msg.contains(expect), "{msg}");
    }

    let follows = showcase(json!([
        { "type": "plan", "id": Uuid::from_u128(7).to_string(), "operator_id": "char_002_amiya" },
    ]));
    validate(&ctx, owner, &follows).await.unwrap();
    db.close().await;
}

#[tokio::test]
async fn favourites_and_backgrounds_without_game_data_are_refused() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    let favs = showcase(json!([
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya"] },
    ]));
    let msg = bad_request(validate(&ctx, owner, &favs).await);
    assert!(
        msg.starts_with("showcase block 1: unknown operator"),
        "{msg}"
    );
    let bg = background(ProfileBackgroundKind::Operator, "char_002_amiya", None);
    assert!(
        bad_request(validate_background(&ctx, &bg).await)
            .starts_with("background: unknown operator")
    );
    db.close().await;
}

#[tokio::test]
async fn unknown_picture_backgrounds_are_refused_by_their_noun() {
    let (ctx, db) = common::context().await;
    for (kind, noun) in [
        (ProfileBackgroundKind::ArchivePic, "gallery picture"),
        (ProfileBackgroundKind::StoryCg, "story CG"),
        (ProfileBackgroundKind::StoryScene, "story scene"),
    ] {
        let msg = bad_request(validate_background(&ctx, &background(kind, "pic_x", None)).await);
        assert_eq!(msg, format!("background: `pic_x` is not a {noun}"));
    }
    let skin = background(ProfileBackgroundKind::Skin, "char_002_amiya#2", None);
    assert_eq!(
        bad_request(validate_background(&ctx, &skin).await),
        "background: `char_002_amiya#2` is not an outfit"
    );
    db.close().await;
}

#[tokio::test]
async fn favourites_and_backgrounds_check_the_game_data() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let owner = db.user(1, Server::EN).await;
    let favs = showcase(json!([
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya", "char_123_fang"] },
        { "type": "favourites", "entity_kind": "skin", "ids": ["char_002_amiya@winter#1"] },
    ]));
    validate(&ctx, owner, &favs).await.unwrap();
    let missing = showcase(json!([
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya"] },
        { "type": "favourites", "entity_kind": "operator", "ids": ["char_999_nobody"] },
    ]));
    let msg = bad_request(validate(&ctx, owner, &missing).await);
    assert!(
        msg.starts_with("showcase block 2: unknown operator `char_999_nobody`"),
        "{msg}"
    );
    let story = showcase(json!([
        { "type": "favourites", "entity_kind": "story_cg", "ids": ["avg_1"] },
    ]));
    assert!(bad_request(validate(&ctx, owner, &story).await).contains("unknown story_cg"));

    use ProfileBackgroundKind::{Operator, Skin};
    for ok in [
        background(Skin, "char_002_amiya@winter#1", None),
        background(Operator, "char_002_amiya", Some(2)),
        background(Operator, "char_123_fang", Some(1)),
    ] {
        validate_background(&ctx, &ok).await.unwrap();
    }
    assert_eq!(
        bad_request(
            validate_background(&ctx, &background(Operator, "char_123_fang", Some(2))).await
        ),
        "background: `char_123_fang` has no elite 2 art"
    );
    assert!(
        bad_request(
            validate_background(&ctx, &background(Skin, "char_002_amiya@nope#9", None)).await
        )
        .starts_with("background: unknown skin")
    );
    db.close().await;
}

async fn with_settings(ctx: &AppContext, user: UserId) {
    let mut conn = ctx.db.conn().await.unwrap();
    gacha::update_flags(&mut conn, user, None, None)
        .await
        .unwrap();
}

async fn stored_layout(ctx: &AppContext, user: UserId) -> Option<Value> {
    let mut conn = ctx.db.conn().await.unwrap();
    users::profile(&mut conn, user)
        .await
        .unwrap()
        .unwrap()
        .profile_layout
}

#[tokio::test]
async fn save_profile_validates_before_it_writes() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    with_settings(&ctx, owner).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let grid = a_grid(&ctx, owner).await;

    let bad: UpdateSettingsRequest = serde_json::from_value(json!({
        "profile_layout": { "showcase": { "blocks": [{ "type": "grid", "slug": "nope-000000" }] } }
    }))
    .unwrap();
    assert!(matches!(
        save_profile(&ctx, &me, &bad).await,
        Err(AppError::BadRequest(_))
    ));
    assert_eq!(stored_layout(&ctx, owner).await, None);

    let bad_bg: UpdateSettingsRequest = serde_json::from_value(json!({
        "profile_layout": { "background": { "kind": "story_cg", "id": "x" } }
    }))
    .unwrap();
    assert!(matches!(
        save_profile(&ctx, &me, &bad_bg).await,
        Err(AppError::BadRequest(_))
    ));

    let good: UpdateSettingsRequest = serde_json::from_value(json!({
        "profile_layout": { "showcase": { "blocks": [{ "type": "grid", "slug": grid }] } }
    }))
    .unwrap();
    assert!(matches!(
        save_profile(&ctx, &Subject::Anonymous, &good).await,
        Err(AppError::Unauthorized)
    ));
    save_profile(&ctx, &me, &good).await.unwrap();
    let layout = ProfileLayout::normalize(&stored_layout(&ctx, owner).await.unwrap());
    assert_eq!(
        layout.showcase_blocks(),
        [ShowcaseBlock::Grid { slug: grid }]
    );
    db.close().await;
}

#[tokio::test]
async fn the_view_marks_gone_blocks_for_the_owner_and_hides_them_from_visitors() {
    let (ctx, db) = common::context().await;
    let owner = db.user(1, Server::EN).await;
    with_settings(&ctx, owner).await;
    let me = common::user(&ctx, owner, Role::User).await;
    let visitor = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let grid = a_grid(&ctx, owner).await;
    let doomed = a_grid(&ctx, owner).await;
    let old_plan = a_plan(&ctx, owner, "char_002_amiya", true).await;
    let request: UpdateSettingsRequest = serde_json::from_value(json!({
        "profile_layout": { "showcase": { "blocks": [
            { "type": "grid", "slug": grid },
            { "type": "grid", "slug": doomed },
            { "type": "plan", "id": old_plan.to_string(), "operator_id": "char_002_amiya" },
            { "type": "favourites", "entity_kind": "operator", "ids": ["char_999_nobody"] },
        ] } }
    }))
    .unwrap();
    myrtle_app::privacy::update_settings(&ctx, &me, &request)
        .await
        .unwrap();
    grids::delete(&ctx, &me, &doomed).await.unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    planner::delete_plan(&mut conn, owner, "char_002_amiya")
        .await
        .unwrap();
    drop(conn);
    let new_plan = a_plan(&ctx, owner, "char_002_amiya", true).await;

    let own = user_showcase(&ctx, &me, "1", None).await.unwrap();
    let removed: Vec<bool> = own.blocks.iter().map(|b| b.removed).collect();
    assert_eq!(removed, [false, true, false, true]);
    assert_eq!(
        own.blocks[2].block,
        ShowcaseBlock::Plan {
            id: new_plan,
            operator_id: Some("char_002_amiya".into())
        }
    );
    assert_eq!(own.blocks[3].entities.len(), 1);

    let seen = user_showcase(&ctx, &visitor, "1", None).await.unwrap();
    assert_eq!(seen.blocks.len(), 2);
    assert!(seen.blocks.iter().all(|b| !b.removed));

    let mut layout =
        ProfileLayout::normalize(&stored_layout(&ctx, owner).await.unwrap()).visible_only();
    layout.background = Some(background(
        ProfileBackgroundKind::Operator,
        "char_999_nobody",
        None,
    ));
    prune_for_visitor(&ctx, owner, &mut layout).await.unwrap();
    assert_eq!(layout.showcase_blocks().len(), 2);
    assert_eq!(layout.background, None);
    layout.background = Some(background(ProfileBackgroundKind::StoryCg, "avg_x", None));
    prune_for_visitor(&ctx, owner, &mut layout).await.unwrap();
    assert_eq!(layout.background, None);

    let hide: UpdateSettingsRequest = serde_json::from_value(json!({
        "profile_layout": { "tabs": [{ "id": "showcase", "visible": false }] }
    }))
    .unwrap();
    myrtle_app::privacy::update_settings(&ctx, &me, &hide)
        .await
        .unwrap();
    assert!(matches!(
        user_showcase(&ctx, &visitor, "1", None).await,
        Err(AppError::Forbidden)
    ));
    assert_eq!(
        user_showcase(&ctx, &me, "1", None)
            .await
            .unwrap()
            .blocks
            .len(),
        4
    );
    assert!(
        view(&ctx, owner, None, false, Server::EN)
            .await
            .unwrap()
            .blocks
            .is_empty()
    );
    db.close().await;
}

#[tokio::test]
async fn picture_backgrounds_and_favourites_check_the_story_catalogues() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = db.user(1, Server::EN).await;
    let pics = &snapshot.model.story_archives.act_archive_res_data.pics;
    let pic = pics.keys().next().unwrap().clone();
    validate_background(
        &ctx,
        &background(ProfileBackgroundKind::ArchivePic, &pic, None),
    )
    .await
    .unwrap();
    let mut layout = ProfileLayout {
        background: Some(background(ProfileBackgroundKind::ArchivePic, &pic, None)),
        ..Default::default()
    };
    prune_for_visitor(&ctx, owner, &mut layout).await.unwrap();
    assert!(layout.background.is_some());

    let art = myrtle_app::services::story::story_art_shared(&ctx, Server::EN)
        .await
        .unwrap();
    let cg = art.gallery(myrtle_engine_story::dto::StoryArtKind::Cg);
    if let Some(id) = cg
        .groups
        .first()
        .and_then(|g| g.pictures.first())
        .map(|p| p.id.clone())
    {
        validate_background(&ctx, &background(ProfileBackgroundKind::StoryCg, &id, None))
            .await
            .unwrap();
        let favs = showcase(json!([
            { "type": "favourites", "entity_kind": "story_cg", "ids": [id] },
        ]));
        validate(&ctx, owner, &favs).await.unwrap();
        let msg = bad_request(
            validate_background(
                &ctx,
                &background(ProfileBackgroundKind::StoryScene, &id, None),
            )
            .await,
        );
        assert!(msg.ends_with("is not a story scene"), "{msg}");
    } else {
        eprintln!("skip: the EN story art catalogue is empty (evicted art tree)");
    }
    db.close().await;
}
