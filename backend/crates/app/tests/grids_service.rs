#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;

use chrono::Utc;
use myrtle_app::audit::ContentAuditEntry;
use myrtle_app::policy::Subject;
use myrtle_app::services::community::grids::{
    self, DESCRIPTION_MAX, GridCellInput, GridDocument, GridInput, GridPreviewIcon, LABEL_MAX,
    MAX_GRIDS_PER_USER, SLUG_BASE_MAX, StoredCell, TITLE_MAX, checked, fallback_order, kinds_of,
    preview_of, romanize_cn_operator, slug_for,
};
use myrtle_app::services::tier_lists::entities::{EntitySummary, FacetValue};
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::content_audit::ContentAuditFilter;
use myrtle_db::repos::grids::{self as repo, GridDoc, GridRow, ListOrder};
use myrtle_types::{ContentAuditAction, ContentTargetKind, EntityKind, Role, Server};
use serde_json::json;
use uuid::Uuid;

fn cell(label: &str) -> GridCellInput {
    GridCellInput {
        label: label.into(),
        entity_kind: None,
        entity_id: None,
    }
}

fn input(rows: i32, cols: i32) -> GridInput {
    let n = usize::try_from(rows * cols).unwrap_or(0);
    GridInput {
        title: "  Favourites  ".into(),
        description: Some("   ".into()),
        rows,
        cols,
        cells: (0..n).map(|_| cell("Favorite Vanguard")).collect(),
        is_listed: true,
        entity_kinds: EntityKind::ALL.to_vec(),
    }
}

fn bad_request<T: std::fmt::Debug>(result: Result<T, AppError>) -> String {
    match result {
        Err(AppError::BadRequest(msg)) => msg,
        other => panic!("expected a 400, got {other:?}"),
    }
}

fn row(template_of: Option<Uuid>, kinds: &[&str]) -> GridRow {
    let now = Utc::now();
    GridRow {
        id: Uuid::from_u128(10),
        slug: "g-abcdef".into(),
        title: "G".into(),
        description: None,
        created_by: Uuid::from_u128(1),
        rows: 1,
        cols: 1,
        cells: json!([]),
        is_listed: true,
        entity_kinds: kinds.iter().map(|k| (*k).to_owned()).collect(),
        template_of,
        created_at: now,
        updated_at: now,
        owner_name: "o".into(),
        fork_count: 0,
        template_slug: None,
        template_title: None,
        template_listed: None,
        template_owner: None,
    }
}

async fn audit(ctx: &AppContext, grid: Uuid) -> Vec<ContentAuditEntry> {
    let filter = ContentAuditFilter {
        actor: None,
        target_kind: Some(ContentTargetKind::Grid),
        target_id: Some(grid),
    };
    ctx.audit
        .feed(&ctx.policy, &myrtle_app::Subject::System, filter, 50, None)
        .await
        .unwrap()
        .entries
}

#[test]
fn a_valid_grid_is_trimmed_and_kept() {
    let doc = checked(input(6, 6)).unwrap();
    assert_eq!(doc.title, "Favourites");
    assert_eq!(doc.description, None);
    assert_eq!((doc.rows, doc.cols), (6, 6));
    assert_eq!(doc.cells.len(), 36);
    assert!(doc.cells.iter().all(|c| c.entity_kind.is_none()));
}

#[test]
fn sizes_are_one_to_ten() {
    assert!(checked(input(1, 1)).is_ok());
    assert!(checked(input(10, 10)).is_ok());
    for (rows, cols) in [(0, 3), (3, 0), (11, 3), (3, 11), (-1, 2)] {
        let mut grid = input(3, 3);
        grid.rows = rows;
        grid.cols = cols;
        assert!(bad_request(checked(grid)).contains("must be 1 to 10"));
    }
}

#[test]
fn the_cell_count_must_match_the_size() {
    let mut grid = input(3, 4);
    grid.cells.pop();
    assert!(bad_request(checked(grid)).contains("has 12 cells, got 11"));
    let mut grid = input(3, 4);
    grid.cells.push(cell(""));
    assert!(bad_request(checked(grid)).contains("got 13"));
}

#[test]
fn a_half_specified_entity_is_refused() {
    let mut grid = input(2, 2);
    grid.cells[1].entity_kind = Some(EntityKind::Operator);
    assert!(bad_request(checked(grid)).starts_with("cell 1:"));
    let mut grid = input(2, 2);
    grid.cells[3].entity_id = Some("char_002_amiya".into());
    assert!(bad_request(checked(grid)).starts_with("cell 3:"));
}

#[test]
fn a_full_entity_is_stored_by_its_kind_name() {
    let mut grid = input(1, 2);
    grid.cells[0].entity_kind = Some(EntityKind::StrongholdBond);
    grid.cells[0].entity_id = Some("bond_x".into());
    let doc = checked(grid).unwrap();
    assert_eq!(doc.cells[0].entity_kind.as_deref(), Some("stronghold_bond"));
    assert_eq!(
        doc.cells[0].entity(),
        Some((EntityKind::StrongholdBond, "bond_x"))
    );
}

#[test]
fn lengths_are_capped_in_chars() {
    let mut grid = input(1, 1);
    grid.cells[0].label = "あ".repeat(LABEL_MAX);
    assert!(checked(grid).is_ok());
    let mut grid = input(1, 1);
    grid.cells[0].label = "a".repeat(LABEL_MAX + 1);
    assert!(bad_request(checked(grid)).contains("cell 0 label"));

    let mut grid = input(1, 1);
    grid.title = "t".repeat(TITLE_MAX + 1);
    assert!(bad_request(checked(grid)).contains("title"));
    let mut grid = input(1, 1);
    grid.title = "   ".into();
    assert_eq!(bad_request(checked(grid)), "title is required");
    let mut grid = input(1, 1);
    grid.description = Some("d".repeat(DESCRIPTION_MAX + 1));
    assert!(bad_request(checked(grid)).contains("description"));
}

#[test]
fn an_empty_label_is_allowed() {
    let mut grid = input(1, 1);
    grid.cells[0].label = String::new();
    assert_eq!(checked(grid).unwrap().cells[0].label, "");
}

#[test]
fn a_grid_allows_at_least_one_kind() {
    let mut grid = input(1, 1);
    grid.entity_kinds = vec![];
    assert!(bad_request(checked(grid)).contains("at least one type"));
}

#[test]
fn allowed_kinds_are_deduplicated_in_canonical_order() {
    let mut grid = input(1, 1);
    grid.entity_kinds = vec![
        EntityKind::Skin,
        EntityKind::Operator,
        EntityKind::Skin,
        EntityKind::Enemy,
    ];
    assert_eq!(
        checked(grid).unwrap().entity_kinds,
        vec![EntityKind::Operator, EntityKind::Enemy, EntityKind::Skin]
    );
}

#[test]
fn a_pick_outside_the_allowed_kinds_is_refused() {
    let mut grid = input(1, 2);
    grid.entity_kinds = vec![EntityKind::Operator, EntityKind::Skin];
    grid.cells[1].entity_kind = Some(EntityKind::Module);
    grid.cells[1].entity_id = Some("uniequip_002_amiya".into());
    let msg = bad_request(checked(grid));
    assert!(msg.starts_with("cell 1:"), "{msg}");
    assert!(msg.contains("`module`"), "{msg}");

    let mut grid = input(1, 2);
    grid.entity_kinds = vec![EntityKind::Operator];
    grid.cells[0].entity_kind = Some(EntityKind::Operator);
    grid.cells[0].entity_id = Some("char_002_amiya".into());
    assert!(checked(grid).is_ok());
}

#[test]
fn stored_kinds_read_leniently() {
    assert_eq!(
        kinds_of(&row(None, &["skin", "boss", "operator", "skin"])),
        vec![EntityKind::Operator, EntityKind::Skin]
    );
    assert_eq!(kinds_of(&row(None, &["boss"])), EntityKind::ALL.to_vec());
    assert!(EntityKind::ALL.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn an_unknown_stored_kind_drops_the_pick_only() {
    let stored = StoredCell {
        label: "Boss".into(),
        entity_kind: Some("boss".into()),
        entity_id: Some("x".into()),
    };
    assert_eq!(stored.entity(), None);
}

#[test]
fn a_slug_always_has_a_readable_base() {
    assert!(slug_for("Favourite Vanguards").starts_with("favourite-vanguards-"));
    assert!(slug_for("!!!").starts_with("grid-"));
    assert_eq!(slug_for(&"a".repeat(TITLE_MAX)).len(), SLUG_BASE_MAX + 7);
}

#[test]
fn fallback_tries_cn_first_then_the_configured_order() {
    use Server::{Bilibili, CN, EN, JP, KR};
    assert_eq!(fallback_order(EN, &[EN, JP, CN, KR]), vec![CN, JP, KR]);
    assert_eq!(fallback_order(JP, &[EN, JP]), vec![EN]);
    assert_eq!(fallback_order(CN, &[EN, CN, KR]), vec![EN, KR]);
    assert_eq!(fallback_order(EN, &[EN]), Vec::<Server>::new());
    assert_eq!(fallback_order(EN, &[EN, CN, CN]), vec![CN]);
    assert_eq!(fallback_order(EN, &[EN, Bilibili]), vec![CN]);
    assert_eq!(fallback_order(Bilibili, &[EN, CN]), vec![EN]);
}

fn summary(kind: EntityKind, appellation: Option<&str>) -> EntitySummary {
    EntitySummary {
        kind,
        id: "char_1015_aglna2".into(),
        name: "予愿安洁莉娜".into(),
        icon: None,
        href: None,
        facets: appellation
            .map(|a| ("appellation".to_owned(), FacetValue::One(a.to_owned())))
            .into_iter()
            .collect(),
    }
}

#[test]
fn a_cn_only_operator_is_named_by_its_appellation_off_cn() {
    let mellow = Some("Angelina the Mellow Wish");
    let name = |e, from| romanize_cn_operator(e, from).name;
    assert_eq!(
        name(summary(EntityKind::Operator, mellow), Some(Server::CN)),
        "Angelina the Mellow Wish"
    );
    for (e, from) in [
        (summary(EntityKind::Operator, mellow), None),
        (summary(EntityKind::Operator, mellow), Some(Server::JP)),
        (summary(EntityKind::Operator, None), Some(Server::CN)),
        (summary(EntityKind::Operator, Some(" ")), Some(Server::CN)),
        (summary(EntityKind::Skin, mellow), Some(Server::CN)),
    ] {
        assert_eq!(name(e, from), "予愿安洁莉娜");
    }
}

fn stored(kind: Option<&str>, id: Option<&str>) -> StoredCell {
    StoredCell {
        label: String::new(),
        entity_kind: kind.map(str::to_owned),
        entity_id: id.map(str::to_owned),
    }
}

/// Knows `amiya` (own art), `aglna2` (CN art) and `noart`
fn preview_resolve(kind: EntityKind, id: &str) -> Option<(EntitySummary, Option<Server>)> {
    let (icon, server) = match id {
        "amiya" => (Some("/avatar/amiya"), None),
        "aglna2" => (Some("/avatar/aglna2"), Some(Server::CN)),
        "noart" => (None, None),
        _ => return None,
    };
    let entity = EntitySummary {
        kind,
        id: id.into(),
        name: id.into(),
        icon: icon.map(str::to_owned),
        href: None,
        facets: BTreeMap::new(),
    };
    Some((entity, server))
}

#[test]
fn the_preview_is_every_cell_in_order() {
    let cells = [
        stored(Some("operator"), Some("amiya")),
        stored(None, None),
        stored(Some("operator"), Some("aglna2")),
        stored(Some("operator"), Some("gone")),
        stored(Some("operator"), Some("noart")),
        stored(Some("not_a_kind"), Some("amiya")),
    ];
    let icon = |icon: &str, server| {
        Some(GridPreviewIcon {
            kind: EntityKind::Operator,
            icon: icon.into(),
            server,
        })
    };
    assert_eq!(
        preview_of(&cells, 6, preview_resolve),
        vec![
            icon("/avatar/amiya", None),
            None,
            icon("/avatar/aglna2", Some(Server::CN)),
            None,
            None,
            None,
        ]
    );
}

#[test]
fn the_preview_has_the_grids_shape_whatever_was_stored() {
    let filled = stored(Some("operator"), Some("amiya"));
    let full = vec![filled.clone(); 36];
    let preview = preview_of(&full, 36, preview_resolve);
    assert_eq!(preview.len(), 36);
    assert!(preview.iter().all(Option::is_some));
    let empty = vec![stored(None, None); 9];
    assert_eq!(preview_of(&empty, 9, preview_resolve), vec![None; 9]);
    let short = preview_of(std::slice::from_ref(&filled), 4, preview_resolve);
    assert_eq!(short.len(), 4);
    assert!(short[0].is_some() && short[1..].iter().all(Option::is_none));
    assert_eq!(preview_of(&full, 2, preview_resolve).len(), 2);
}

fn plain(title: &str, kinds: Vec<EntityKind>, listed: bool) -> GridInput {
    GridInput {
        title: title.into(),
        description: None,
        rows: 2,
        cols: 2,
        cells: ["a", "b", "c", "d"].into_iter().map(cell).collect(),
        is_listed: listed,
        entity_kinds: kinds,
    }
}

#[tokio::test]
async fn create_get_and_the_edit_gate_both_ways() {
    let (ctx, db) = common::context().await;
    let owner_id = db.user(1, Server::EN).await;
    let owner = common::user(&ctx, owner_id, Role::User).await;
    let stranger = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let editor = common::user(&ctx, db.user(3, Server::EN).await, Role::TierListEditor).await;
    let admin = common::user(&ctx, db.user(4, Server::EN).await, Role::TierListAdmin).await;

    assert!(matches!(
        grids::create(&ctx, &Subject::Anonymous, input(1, 1), None).await,
        Err(AppError::Unauthorized)
    ));
    let grid = grids::create(&ctx, &owner, input(2, 2), None)
        .await
        .unwrap();
    assert_eq!(grid.title, "Favourites");
    assert_eq!(grid.owner.id, owner_id.0);
    assert_eq!(grid.owner.name, "1");
    assert!(grid.can_edit && !grid.kinds_locked && grid.template_of.is_none());
    assert!(grid.slug.starts_with("favourites-"));

    for (subject, can) in [
        (&owner, true),
        (&stranger, false),
        (&editor, false),
        (&admin, true),
        (&Subject::Anonymous, false),
    ] {
        let seen = grids::get(&ctx, subject, &grid.slug, None).await.unwrap();
        assert_eq!(seen.can_edit, can);
    }

    for subject in [&stranger, &editor] {
        assert!(matches!(
            grids::update(&ctx, subject, &grid.slug, input(2, 2), None).await,
            Err(AppError::Forbidden)
        ));
        assert!(matches!(
            grids::delete(&ctx, subject, &grid.slug).await,
            Err(AppError::Forbidden)
        ));
    }
    assert!(matches!(
        grids::update(&ctx, &Subject::Anonymous, &grid.slug, input(2, 2), None).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        grids::update(&ctx, &owner, "nope-000000", input(2, 2), None).await,
        Err(AppError::NotFound(None))
    ));

    let mut renamed = input(3, 1);
    renamed.title = "By admin".into();
    let saved = grids::update(&ctx, &admin, &grid.slug, renamed, None)
        .await
        .unwrap();
    assert_eq!(
        (saved.title.as_str(), saved.rows, saved.cols),
        ("By admin", 3, 1)
    );
    assert_eq!(saved.owner.id, owner_id.0);
    assert!(saved.can_edit);

    grids::delete(&ctx, &owner, &grid.slug).await.unwrap();
    assert!(matches!(
        grids::get(&ctx, &owner, &grid.slug, None).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn one_account_keeps_at_most_fifty_grids() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let mut first = None;
    for i in 0..MAX_GRIDS_PER_USER {
        let grid = grids::create(
            &ctx,
            &owner,
            plain(&format!("g{i}"), vec![EntityKind::Operator], true),
            None,
        )
        .await
        .unwrap();
        first.get_or_insert(grid.slug);
    }
    let over = grids::create(&ctx, &owner, input(1, 1), None).await;
    assert!(matches!(over, Err(AppError::Conflict(m)) if m.contains("50 grids")));
    let theirs = grids::create(&ctx, &other, input(1, 1), None)
        .await
        .unwrap();
    assert!(matches!(
        grids::fork(&ctx, &owner, &theirs.slug, None).await,
        Err(AppError::Conflict(_))
    ));
    assert_eq!(
        grids::mine(&ctx, &owner, None).await.unwrap().len(),
        usize::try_from(MAX_GRIDS_PER_USER).unwrap()
    );
    grids::delete(&ctx, &owner, &first.unwrap()).await.unwrap();
    assert!(grids::fork(&ctx, &owner, &theirs.slug, None).await.is_ok());
    db.close().await;
}

#[tokio::test]
async fn a_fork_copies_labels_and_kinds_and_locks_the_kinds() {
    let (ctx, db) = common::context().await;
    let author_id = db.user(1, Server::EN).await;
    let author = common::user(&ctx, author_id, Role::User).await;
    let forker = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let kinds = vec![EntityKind::Skin, EntityKind::Operator];
    let source = grids::create(&ctx, &author, plain("Top picks", kinds, false), None)
        .await
        .unwrap();

    let fork = grids::fork(&ctx, &forker, &source.slug, None)
        .await
        .unwrap();
    assert_ne!(fork.slug, source.slug);
    assert_eq!(fork.title, "Top picks");
    assert!(!fork.is_listed, "an unlisted source stays unlisted");
    assert_eq!(fork.entity_kinds, [EntityKind::Operator, EntityKind::Skin]);
    assert!(fork.kinds_locked && fork.can_edit);
    let labels: Vec<_> = fork.cells.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(labels, ["a", "b", "c", "d"]);
    assert!(fork.cells.iter().all(|c| c.entity_kind.is_none()));
    assert!(
        fork.template_of.is_none(),
        "an unlisted template is the author's to see"
    );
    let as_author = grids::get(&ctx, &author, &fork.slug, None).await.unwrap();
    assert_eq!(as_author.template_of.unwrap().slug, source.slug);
    assert_eq!(
        grids::get(&ctx, &author, &source.slug, None)
            .await
            .unwrap()
            .fork_count,
        1
    );

    let changed = plain("Mine", vec![EntityKind::Operator], false);
    assert_eq!(
        bad_request(grids::update(&ctx, &forker, &fork.slug, changed, None).await),
        "entity_kinds: allowed types are set by the template"
    );
    let same = plain(
        "Mine",
        vec![EntityKind::Skin, EntityKind::Operator, EntityKind::Skin],
        true,
    );
    assert!(
        grids::update(&ctx, &forker, &fork.slug, same, None)
            .await
            .is_ok()
    );

    let entries = audit(&ctx, fork.id).await;
    let forked = entries
        .iter()
        .find(|e| e.action == ContentAuditAction::Fork)
        .unwrap();
    assert_eq!(forked.details.forked_from.as_deref(), Some("Top picks"));
    assert_eq!(forked.owner.as_ref().unwrap().user_id, fork.owner.id);

    grids::delete(&ctx, &author, &source.slug).await.unwrap();
    let orphan = grids::get(&ctx, &forker, &fork.slug, None).await.unwrap();
    assert!(!orphan.kinds_locked && orphan.template_of.is_none());
    assert!(
        grids::update(
            &ctx,
            &forker,
            &fork.slug,
            plain("x", vec![EntityKind::Enemy], true),
            None
        )
        .await
        .is_ok()
    );
    db.close().await;
}

#[tokio::test]
async fn audit_rows_are_written_and_a_run_of_edits_coalesces() {
    let (ctx, db) = common::context().await;
    let owner_id = db.user(1, Server::EN).await;
    let owner = common::user(&ctx, owner_id, Role::User).await;
    let grid = grids::create(
        &ctx,
        &owner,
        plain("First", vec![EntityKind::Operator], true),
        None,
    )
    .await
    .unwrap();
    let created = audit(&ctx, grid.id).await;
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].action, ContentAuditAction::Create);
    assert_eq!(created[0].target_slug.as_deref(), Some(grid.slug.as_str()));

    let unchanged = plain("First", vec![EntityKind::Operator], true);
    grids::update(&ctx, &owner, &grid.slug, unchanged, None)
        .await
        .unwrap();
    assert_eq!(
        audit(&ctx, grid.id).await.len(),
        1,
        "a no-op save writes nothing"
    );

    let mut edit = plain(
        "Second",
        vec![EntityKind::Operator, EntityKind::Enemy],
        false,
    );
    edit.cells[0].label = "z".into();
    edit.description = Some("about".into());
    grids::update(&ctx, &owner, &grid.slug, edit, None)
        .await
        .unwrap();
    let mut resize = input(3, 1);
    resize.title = "Third".into();
    resize.entity_kinds = vec![EntityKind::Operator, EntityKind::Enemy];
    resize.is_listed = false;
    grids::update(&ctx, &owner, &grid.slug, resize, None)
        .await
        .unwrap();

    let entries = audit(&ctx, grid.id).await;
    assert_eq!(entries.len(), 2);
    let edit = entries
        .iter()
        .find(|e| e.action == ContentAuditAction::Edit)
        .unwrap();
    assert_eq!(edit.event_count, 2);
    assert_eq!(edit.target_name, "Third");
    let d = &edit.details;
    assert_eq!(d.renamed_from.as_deref(), Some("First"));
    assert_eq!(d.description_changed, Some(true));
    assert_eq!(d.listed, Some(false));
    assert_eq!(d.size.as_deref(), Some("3x1"));
    assert_eq!(
        d.kinds.as_deref(),
        Some(&["operator".to_owned(), "enemy".to_owned()][..])
    );
    assert_eq!(d.cells_changed, Some(1 + 4));

    grids::delete(&ctx, &owner, &grid.slug).await.unwrap();
    let deleted = audit(&ctx, grid.id).await;
    assert!(
        deleted
            .iter()
            .any(|e| e.action == ContentAuditAction::Delete
                && e.owner.as_ref().is_some_and(|o| o.user_id == owner_id.0))
    );
    db.close().await;
}

#[tokio::test]
async fn listing_pages_searches_and_orders() {
    let (ctx, db) = common::context().await;
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let forker = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let ops = || vec![EntityKind::Operator];
    let a = grids::create(&ctx, &owner, plain("100% guards", ops(), true), None)
        .await
        .unwrap();
    let b = grids::create(&ctx, &owner, plain("100 casters", ops(), true), None)
        .await
        .unwrap();
    grids::create(&ctx, &owner, plain("hidden 100%", ops(), false), None)
        .await
        .unwrap();
    grids::fork(&ctx, &forker, &a.slug, None).await.unwrap();

    let all = grids::list(&ctx, ListOrder::Recent, None, None, None, None)
        .await
        .unwrap();
    assert_eq!((all.total, all.page, all.per_page), (3, 1, 24));
    assert!(all.items.iter().all(|g| g.is_listed));
    assert_eq!(all.items[0].preview.len(), 4);

    let pct = grids::list(&ctx, ListOrder::Recent, Some(" 100% "), None, None, None)
        .await
        .unwrap();
    let slugs: Vec<_> = pct.items.iter().map(|g| g.slug.as_str()).collect();
    assert_eq!(slugs.len(), 2, "{slugs:?}");
    assert!(!slugs.contains(&b.slug.as_str()));

    let popular = grids::list(&ctx, ListOrder::Popular, Some("100"), None, None, None)
        .await
        .unwrap();
    assert_eq!(popular.items[0].slug, a.slug);
    assert_eq!(popular.items[0].fork_count, 1);

    let page = grids::list(&ctx, ListOrder::Recent, None, Some(2), Some(2), None)
        .await
        .unwrap();
    assert_eq!((page.total, page.items.len(), page.page), (3, 1, 2));
    let clamped = grids::list(&ctx, ListOrder::Recent, None, Some(0), Some(500), None)
        .await
        .unwrap();
    assert_eq!((clamped.page, clamped.per_page), (1, 60));

    assert_eq!(grids::mine(&ctx, &owner, None).await.unwrap().len(), 3);
    assert!(matches!(
        grids::mine(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn picks_validate_and_resolve_against_game_data() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let owner_id = db.user(1, Server::EN).await;
    let owner = common::user(&ctx, owner_id, Role::User).await;
    let mut grid = plain(
        "Picks",
        vec![EntityKind::Operator, EntityKind::StoryCg],
        true,
    );
    grid.cells[0].entity_kind = Some(EntityKind::Operator);
    grid.cells[0].entity_id = Some("char_002_amiya".into());
    let made = grids::create(&ctx, &owner, grid.clone(), None)
        .await
        .unwrap();
    let amiya = made.cells[0].entity.as_ref().unwrap();
    assert_eq!(amiya.name, "Amiya");
    assert_eq!(made.cells[0].entity_server, None);
    assert!(made.cells[1].entity.is_none());
    let summary = &grids::mine(&ctx, &owner, None).await.unwrap()[0];
    assert!(summary.preview[0].is_some() || amiya.icon.is_none());

    let mut unknown = grid.clone();
    unknown.cells[1].entity_kind = Some(EntityKind::Operator);
    unknown.cells[1].entity_id = Some("char_999_nobody".into());
    assert!(
        bad_request(grids::create(&ctx, &owner, unknown, None).await).contains("char_999_nobody")
    );
    let mut story = grid.clone();
    story.cells[1].entity_kind = Some(EntityKind::StoryCg);
    story.cells[1].entity_id = Some("avg_nope_0".into());
    assert!(bad_request(grids::create(&ctx, &owner, story, None).await).contains("story_cg"));

    let doc = GridDocument {
        title: "Legacy".into(),
        description: None,
        rows: 1,
        cols: 2,
        cells: vec![
            stored(Some("operator"), Some("char_999_gone")),
            stored(None, None),
        ],
        is_listed: true,
        entity_kinds: vec![EntityKind::Operator],
    };
    let cells = serde_json::to_value(&doc.cells).unwrap();
    let doc_row = GridDoc {
        title: &doc.title,
        description: None,
        rows: doc.rows,
        cols: doc.cols,
        cells: &cells,
        is_listed: true,
        entity_kinds: &doc.entity_kinds,
    };
    let mut conn = ctx.db.conn().await.unwrap();
    assert!(
        repo::create_capped(&mut conn, "legacy-abcdef", doc_row, owner_id, None, 50)
            .await
            .unwrap()
    );
    drop(conn);
    let legacy = grids::get(&ctx, &owner, "legacy-abcdef", None)
        .await
        .unwrap();
    assert_eq!(legacy.cells[0].entity_id.as_deref(), Some("char_999_gone"));
    assert!(legacy.cells[0].entity.is_none());
    let resend = GridInput {
        title: "Legacy kept".into(),
        description: None,
        rows: 1,
        cols: 2,
        cells: vec![
            GridCellInput {
                label: String::new(),
                entity_kind: Some(EntityKind::Operator),
                entity_id: Some("char_999_gone".into()),
            },
            cell(""),
        ],
        is_listed: true,
        entity_kinds: vec![EntityKind::Operator],
    };
    assert!(
        grids::update(&ctx, &owner, "legacy-abcdef", resend.clone(), None)
            .await
            .is_ok()
    );
    let mut moved = resend;
    moved.cells.swap(0, 1);
    assert!(
        grids::update(&ctx, &owner, "legacy-abcdef", moved, None)
            .await
            .is_ok()
    );
    db.close().await;
}

#[tokio::test]
async fn picture_picks_validate_and_resolve_against_the_story_catalogues() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let owner = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let pic = snapshot
        .model
        .story_archives
        .act_archive_res_data
        .pics
        .keys()
        .next()
        .unwrap()
        .clone();
    let mut grid = plain("Pictures", vec![EntityKind::ArchivePic], true);
    grid.cells[0].entity_kind = Some(EntityKind::ArchivePic);
    grid.cells[0].entity_id = Some(pic.clone());
    let made = grids::create(&ctx, &owner, grid.clone(), None)
        .await
        .unwrap();
    let entity = made.cells[0].entity.as_ref().unwrap();
    assert_eq!(
        (entity.kind, entity.id.as_str()),
        (EntityKind::ArchivePic, pic.as_str())
    );
    let read = grids::get(&ctx, &owner, &made.slug, None).await.unwrap();
    assert_eq!(read.cells[0].entity.as_ref(), Some(entity));
    grid.cells[0].entity_id = Some("pic_nope_0".into());
    assert!(bad_request(grids::create(&ctx, &owner, grid, None).await).contains("archive_pic"));
    db.close().await;
}
