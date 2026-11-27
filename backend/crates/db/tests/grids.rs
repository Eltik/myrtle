#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::grids::{self, GridDoc, ListOrder};
use myrtle_types::{EntityKind, GridId, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

fn kinds() -> Vec<EntityKind> {
    vec![EntityKind::Operator, EntityKind::Enemy]
}

async fn create(conn: &mut Conn, slug: &str, title: &str, owner: UserId, listed: bool) {
    let cells = json!([{"label": "a", "entity_kind": null, "entity_id": null}]);
    let kinds = kinds();
    let doc = GridDoc {
        title,
        description: None,
        rows: 1,
        cols: 1,
        cells: &cells,
        is_listed: listed,
        entity_kinds: &kinds,
    };
    assert!(
        grids::create_capped(conn, slug, doc, owner, None, 50)
            .await
            .unwrap()
    );
}

async fn id_of(conn: &mut Conn, slug: &str) -> GridId {
    GridId(grids::find_by_slug(conn, slug).await.unwrap().unwrap().id)
}

/// Pins `updated_at` so ordering by it does not depend on statement timing
async fn edited_hours_ago(conn: &mut Conn, slug: &str, hours: i32) {
    sqlx::query("UPDATE grids SET updated_at = now() - make_interval(hours => $2) WHERE slug = $1")
        .bind(slug)
        .bind(hours)
        .execute(conn)
        .await
        .unwrap();
}

fn slugs(rows: &[grids::GridRow]) -> Vec<&str> {
    rows.iter().map(|g| g.slug.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_grid_reads_back_with_its_owner_fork_count_and_template(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let forker = common::user(&mut conn, 2).await;
    create(&mut conn, "base", "Base", owner, false).await;
    let base = id_of(&mut conn, "base").await;
    let cells: Value = json!([]);
    let kinds = kinds();
    let doc = GridDoc {
        title: "Fork",
        description: Some("mine"),
        rows: 2,
        cols: 3,
        cells: &cells,
        is_listed: true,
        entity_kinds: &kinds,
    };
    assert!(
        grids::create_capped(&mut conn, "fork", doc, forker, Some(base), 50)
            .await
            .unwrap()
    );
    sqlx::query("UPDATE users SET nickname = '' WHERE id = $1")
        .bind(forker.0)
        .execute(&mut *conn)
        .await
        .unwrap();

    let fork = grids::find_by_slug(&mut conn, "fork")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fork.title, "Fork");
    assert_eq!(fork.description.as_deref(), Some("mine"));
    assert_eq!((fork.rows, fork.cols), (2, 3));
    assert_eq!(fork.entity_kinds, ["operator", "enemy"]);
    assert_eq!(fork.owner_name, "2");
    assert_eq!(fork.created_by, forker.0);
    assert_eq!(fork.template_of, Some(base.0));
    assert_eq!(fork.template_slug.as_deref(), Some("base"));
    assert_eq!(fork.template_title.as_deref(), Some("Base"));
    assert_eq!(fork.template_listed, Some(false));
    assert_eq!(fork.template_owner, Some(owner.0));
    assert_eq!(fork.fork_count, 0);

    let base_row = grids::find_by_slug(&mut conn, "base")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(base_row.owner_name, "user1");
    assert_eq!(base_row.fork_count, 1);
    assert_eq!(base_row.template_slug, None);
    assert_eq!(base_row.template_listed, None);
    assert!(
        grids::find_by_slug(&mut conn, "missing")
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_create_at_the_cap_inserts_nothing(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    create(&mut conn, "one", "One", owner, true).await;
    let cells = json!([]);
    let kinds = kinds();
    let doc = GridDoc {
        title: "Two",
        description: None,
        rows: 1,
        cols: 1,
        cells: &cells,
        is_listed: true,
        entity_kinds: &kinds,
    };

    assert!(
        !grids::create_capped(&mut conn, "two", doc, owner, None, 1)
            .await
            .unwrap()
    );
    assert!(
        grids::find_by_slug(&mut conn, "two")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        grids::create_capped(&mut conn, "two", doc, other, None, 1)
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_users_grids_come_latest_edit_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    create(&mut conn, "old", "Old", owner, false).await;
    create(&mut conn, "new", "New", owner, true).await;
    create(&mut conn, "theirs", "Theirs", other, true).await;
    edited_hours_ago(&mut conn, "old", 2).await;
    edited_hours_ago(&mut conn, "new", 1).await;

    let mine = grids::find_by_user(&mut conn, owner).await.unwrap();
    assert_eq!(slugs(&mine), ["new", "old"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_listing_shows_listed_title_matches_in_either_order_with_the_total(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    create(&mut conn, "a", "Best 6*", owner, true).await;
    create(&mut conn, "b", "Best guards", owner, true).await;
    create(&mut conn, "c", "Worst", owner, true).await;
    create(&mut conn, "hidden", "Best hidden", owner, false).await;
    edited_hours_ago(&mut conn, "a", 3).await;
    edited_hours_ago(&mut conn, "b", 2).await;
    edited_hours_ago(&mut conn, "c", 1).await;
    let a = id_of(&mut conn, "a").await;
    let cells = json!([]);
    let kinds = kinds();
    let doc = GridDoc {
        title: "fork of a",
        description: None,
        rows: 1,
        cols: 1,
        cells: &cells,
        is_listed: false,
        entity_kinds: &kinds,
    };
    grids::create_capped(&mut conn, "a-fork", doc, owner, Some(a), 50)
        .await
        .unwrap();

    let (recent, total) = grids::find_listed(&mut conn, None, ListOrder::Recent, 10, 0)
        .await
        .unwrap();
    assert_eq!(slugs(&recent), ["c", "b", "a"]);
    assert_eq!(total, 3);

    let (popular, _) = grids::find_listed(&mut conn, None, ListOrder::Popular, 10, 0)
        .await
        .unwrap();
    assert_eq!(slugs(&popular), ["a", "c", "b"]);

    let (page, total) = grids::find_listed(&mut conn, Some("best%"), ListOrder::Recent, 1, 1)
        .await
        .unwrap();
    assert_eq!(slugs(&page), ["a"]);
    assert_eq!(total, 2);

    let (escaped, total) = grids::find_listed(&mut conn, Some(r"%6\*%"), ListOrder::Recent, 10, 0)
        .await
        .unwrap();
    assert_eq!(slugs(&escaped), ["a"]);
    assert_eq!(total, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_update_rewrites_the_document_and_a_delete_orphans_its_forks(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    create(&mut conn, "base", "Base", owner, true).await;
    let base = id_of(&mut conn, "base").await;
    let cells = json!([{"label": "x", "entity_kind": "operator", "entity_id": "char_002_amiya"}]);
    let kinds = vec![EntityKind::Skin];
    let doc = GridDoc {
        title: "Edited",
        description: Some("now described"),
        rows: 4,
        cols: 5,
        cells: &cells,
        is_listed: false,
        entity_kinds: &kinds,
    };
    grids::update(&mut conn, base, doc).await.unwrap();

    let edited = grids::find_by_slug(&mut conn, "base")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(edited.title, "Edited");
    assert_eq!(edited.description.as_deref(), Some("now described"));
    assert_eq!((edited.rows, edited.cols), (4, 5));
    assert_eq!(edited.cells, cells);
    assert!(!edited.is_listed);
    assert_eq!(edited.entity_kinds, ["skin"]);

    grids::create_capped(&mut conn, "fork", doc, owner, Some(base), 50)
        .await
        .unwrap();
    grids::delete(&mut conn, base).await.unwrap();
    assert!(
        grids::find_by_slug(&mut conn, "base")
            .await
            .unwrap()
            .is_none()
    );
    let fork = grids::find_by_slug(&mut conn, "fork")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fork.template_of, None);
    assert_eq!(fork.template_slug, None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn existing_slugs_names_only_the_grids_that_exist(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    create(&mut conn, "kept", "Kept", owner, false).await;

    let found = grids::existing_slugs(&mut conn, &["kept".to_owned(), "gone".to_owned()])
        .await
        .unwrap();
    assert_eq!(found, ["kept"]);
    assert!(
        grids::existing_slugs(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_second_create_for_the_same_owner_waits_for_the_first_to_commit(pool: PgPool) {
    let db = common::db(pool).await;
    let owner = common::user(&mut db.conn().await.unwrap(), 1).await;
    let cells = json!([]);
    let kinds = kinds();
    let doc = GridDoc {
        title: "Two",
        description: None,
        rows: 1,
        cols: 1,
        cells: &cells,
        is_listed: true,
        entity_kinds: &kinds,
    };

    let mut first = db.begin().await.unwrap();
    create(&mut first, "one", "One", owner, true).await;

    let mut second = db.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout = '200ms'")
        .execute(&mut *second)
        .await
        .unwrap();
    assert!(
        grids::create_capped(&mut second, "two", doc, owner, None, 50)
            .await
            .is_err()
    );
    second.rollback().await.unwrap();

    first.commit().await.unwrap();
    let mut second = db.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout = '200ms'")
        .execute(&mut *second)
        .await
        .unwrap();
    assert!(
        grids::create_capped(&mut second, "two", doc, owner, None, 50)
            .await
            .unwrap()
    );
    second.commit().await.unwrap();
}
