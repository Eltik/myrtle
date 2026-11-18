#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::tier_lists::{self, TierListRow, Viewer};
use myrtle_db::{Conn, DbError};
use myrtle_types::{EntityKind, TierListId, TierListType, UserId};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const SESSION: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

async fn list(conn: &mut Conn, owner: UserId, slug: &str) -> TierListRow {
    tier_lists::create(conn, slug, slug, None, TierListType::Community, owner)
        .await
        .unwrap()
}

const fn id(row: &TierListRow) -> TierListId {
    TierListId(row.id)
}

async fn deactivate(conn: &mut Conn, list: TierListId) {
    sqlx::query("UPDATE tier_lists SET is_active = false WHERE id = $1")
        .bind(list.0)
        .execute(conn)
        .await
        .unwrap();
}

async fn age(conn: &mut Conn, list: TierListId, minutes: i32) {
    sqlx::query(
        "UPDATE tier_lists SET updated_at = NOW() - make_interval(mins => $2) WHERE id = $1",
    )
    .bind(list.0)
    .bind(minutes)
    .execute(conn)
    .await
    .unwrap();
}

async fn tier(conn: &mut Conn, list: TierListId, name: &str, order: i16) -> Uuid {
    tier_lists::create_tier(conn, list, name, order, None, None)
        .await
        .unwrap()
        .id
}

async fn count(conn: &mut Conn, sql: &'static str, list: TierListId) -> i64 {
    sqlx::query_scalar(sql)
        .bind(list.0)
        .fetch_one(conn)
        .await
        .unwrap()
}

/// (event rows, `view_count`, `unique_view_count`)
async fn view_counts(conn: &mut Conn, list: TierListId) -> (i64, i64, i64) {
    let events = count(
        conn,
        "SELECT COUNT(*) FROM tier_list_view_events WHERE tier_list_id = $1",
        list,
    )
    .await;
    let stats = tier_lists::get_stats(conn, list).await.unwrap().unwrap();
    (events, stats.view_count, stats.unique_view_count)
}

async fn favorite_count(conn: &mut Conn, list: TierListId) -> i32 {
    tier_lists::get_stats(conn, list)
        .await
        .unwrap()
        .unwrap()
        .favorite_count
}

fn slugs(rows: &[TierListRow]) -> Vec<&str> {
    rows.iter().map(|r| r.slug.as_str()).collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_created_list_reads_by_slug_only_while_active(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;

    let created = tier_lists::create(
        &mut conn,
        "Meta",
        "meta",
        Some("desc"),
        TierListType::Official,
        user,
    )
    .await
    .unwrap();
    assert_eq!(created.entity_kinds, [EntityKind::Operator]);
    assert!(created.is_listed && created.is_active);
    assert_eq!(created.created_by, Some(user.0));

    let found = tier_lists::find_by_slug(&mut conn, "meta")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, created.id);
    assert_eq!(found.list_type, "official");

    deactivate(&mut conn, id(&created)).await;
    assert!(
        tier_lists::find_by_slug(&mut conn, "meta")
            .await
            .unwrap()
            .is_none()
    );
    let visibility = tier_lists::find_visibility_by_slug(&mut conn, "meta")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (visibility.id, visibility.name.as_str()),
        (created.id, "Meta")
    );
    assert!(visibility.is_listed);

    let err = tier_lists::create(
        &mut conn,
        "Again",
        "meta",
        None,
        TierListType::Community,
        user,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_index_shows_active_listed_lists_newest_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;

    let a = list(&mut conn, user, "a").await;
    let b = tier_lists::create(&mut conn, "b", "b", None, TierListType::Official, other)
        .await
        .unwrap();
    let hidden = list(&mut conn, user, "hidden").await;
    let gone = list(&mut conn, user, "gone").await;
    tier_lists::set_visibility(&mut conn, id(&hidden), false)
        .await
        .unwrap();
    deactivate(&mut conn, id(&gone)).await;
    for (row, minutes) in [(&a, 30), (&b, 10), (&hidden, 5), (&gone, 1)] {
        age(&mut conn, id(row), minutes).await;
    }

    let all = tier_lists::find_all_active(&mut conn, None).await.unwrap();
    assert_eq!(slugs(&all), ["b", "a"]);
    let community = tier_lists::find_all_active(&mut conn, Some(TierListType::Community))
        .await
        .unwrap();
    assert_eq!(slugs(&community), ["a"]);

    let newest = tier_lists::find_all_active_limited(&mut conn, 1, None)
        .await
        .unwrap();
    assert_eq!(slugs(&newest), ["b"]);
    let mine = tier_lists::find_all_active_limited(&mut conn, 10, Some(user))
        .await
        .unwrap();
    assert_eq!(slugs(&mine), ["a"]);

    let by_user = tier_lists::find_by_user(&mut conn, user).await.unwrap();
    assert_eq!(slugs(&by_user), ["hidden", "a"]);
    assert_eq!(
        tier_lists::count_by_user(&mut conn, user, None)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        tier_lists::count_by_user(&mut conn, other, None)
            .await
            .unwrap(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_cap_count_covers_every_active_list_or_only_one_type(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;

    for i in 0..10 {
        list(&mut conn, user, &format!("mine-{i}")).await;
    }
    tier_lists::create(
        &mut conn,
        "site",
        "site",
        None,
        TierListType::Official,
        user,
    )
    .await
    .unwrap();
    assert_eq!(
        tier_lists::count_by_user(&mut conn, user, None)
            .await
            .unwrap(),
        11
    );
    assert_eq!(
        tier_lists::count_by_user(&mut conn, user, Some(TierListType::Community))
            .await
            .unwrap(),
        10
    );
    let gone = id(&list(&mut conn, user, "gone").await);
    deactivate(&mut conn, gone).await;
    assert_eq!(
        tier_lists::count_by_user(&mut conn, user, Some(TierListType::Official))
            .await
            .unwrap(),
        1
    );

    let stored: Vec<(String, i64)> =
        sqlx::query_as("SELECT list_type, count(*) FROM tier_lists GROUP BY 1 ORDER BY 1")
            .fetch_all(&mut *conn)
            .await
            .unwrap();
    assert_eq!(
        stored,
        [("community".to_owned(), 11), ("official".to_owned(), 1)]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn update_keeps_the_offered_kinds_when_given_none_and_reads_skip_unknown_kinds(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = list(&mut conn, user, "kinds").await;

    let kinds = [EntityKind::Enemy, EntityKind::Operator];
    let updated = tier_lists::update(&mut conn, id(&row), "Kinds", Some("d"), Some(&kinds[..]))
        .await
        .unwrap();
    assert_eq!(updated.entity_kinds, kinds);
    let renamed = tier_lists::update(&mut conn, id(&row), "Renamed", None, None)
        .await
        .unwrap();
    assert_eq!(renamed.name, "Renamed");
    assert_eq!(renamed.description, None);
    assert_eq!(renamed.entity_kinds, kinds);

    sqlx::query("UPDATE tier_lists SET entity_kinds = '{boss,enemy}' WHERE id = $1")
        .bind(row.id)
        .execute(&mut *conn)
        .await
        .unwrap();
    let read = tier_lists::find_by_slug(&mut conn, "kinds")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        read.entity_kinds,
        [EntityKind::Enemy],
        "an unknown kind is skipped"
    );
    sqlx::query("UPDATE tier_lists SET entity_kinds = '{boss}' WHERE id = $1")
        .bind(row.id)
        .execute(&mut *conn)
        .await
        .unwrap();
    let read = tier_lists::find_by_slug(&mut conn, "kinds")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        read.entity_kinds,
        [EntityKind::Operator],
        "nothing known reads as operator-only"
    );

    let missing = TierListId(Uuid::new_v4());
    let err = tier_lists::update(&mut conn, missing, "x", None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::NotFound));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn list_type_and_flair_are_set_in_place(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = list(&mut conn, user, "typed").await;

    let official = tier_lists::set_list_type(&mut conn, id(&row), TierListType::Official)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(official.list_type, "official");
    assert!(
        tier_lists::set_list_type(
            &mut conn,
            TierListId(Uuid::new_v4()),
            TierListType::Official
        )
        .await
        .unwrap()
        .is_none()
    );

    let flair = tier_lists::create_flair(&mut conn, "hot", "Hot", None, 0)
        .await
        .unwrap();
    tier_lists::set_flair(&mut conn, id(&row), Some(flair.id))
        .await
        .unwrap();
    let read = tier_lists::find_by_slug(&mut conn, "typed")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(read.flair_id, Some(flair.id));
    tier_lists::set_flair(&mut conn, id(&row), None)
        .await
        .unwrap();
    let cleared = tier_lists::find_by_slug(&mut conn, "typed")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(cleared.flair_id, None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn versions_read_newest_first_and_a_number_is_taken_once(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = list(&mut conn, user, "versioned").await;

    assert_eq!(
        tier_lists::latest_version(&mut conn, id(&row))
            .await
            .unwrap(),
        None
    );
    let first = tier_lists::create_version(&mut conn, id(&row), 1, &json!({"t": 1}), None, user)
        .await
        .unwrap();
    assert_eq!(first.published_by, Some(user.0));
    tier_lists::create_version(&mut conn, id(&row), 2, &json!({"t": 2}), Some("two"), user)
        .await
        .unwrap();

    let versions = tier_lists::get_versions(&mut conn, id(&row)).await.unwrap();
    let numbers: Vec<i32> = versions.iter().map(|v| v.version).collect();
    assert_eq!(numbers, [2, 1]);
    assert_eq!(versions[0].snapshot, json!({"t": 2}));
    assert_eq!(versions[0].changelog.as_deref(), Some("two"));
    assert_eq!(
        tier_lists::latest_version(&mut conn, id(&row))
            .await
            .unwrap(),
        Some(2)
    );

    let err = tier_lists::create_version(&mut conn, id(&row), 2, &json!({}), None, user)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_a_list_takes_everything_hanging_off_it(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = list(&mut conn, user, "doomed").await;
    let list_id = id(&row);
    let t = tier(&mut conn, list_id, "S", 0).await;
    tier_lists::add_placement(
        &mut conn,
        list_id,
        t,
        EntityKind::Operator,
        "char_002_amiya",
        0,
        None,
    )
    .await
    .unwrap();
    tier_lists::create_version(&mut conn, list_id, 1, &json!({}), None, user)
        .await
        .unwrap();
    tier_lists::grant_permission(&mut conn, list_id, user, "edit", user)
        .await
        .unwrap();
    tier_lists::ensure_stats_row(&mut conn, list_id)
        .await
        .unwrap();
    tier_lists::add_favorite(&mut conn, list_id, user)
        .await
        .unwrap();

    tier_lists::delete_list(&mut conn, list_id).await.unwrap();

    for sql in [
        "SELECT COUNT(*) FROM tier_lists WHERE id = $1",
        "SELECT COUNT(*) FROM tiers WHERE tier_list_id = $1",
        "SELECT COUNT(*) FROM tier_list_versions WHERE tier_list_id = $1",
        "SELECT COUNT(*) FROM tier_list_permissions WHERE tier_list_id = $1",
        "SELECT COUNT(*) FROM tier_list_stats WHERE tier_list_id = $1",
        "SELECT COUNT(*) FROM tier_list_favorites WHERE tier_list_id = $1",
    ] {
        assert_eq!(count(&mut conn, sql, list_id).await, 0, "{sql}");
    }
    let placements: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tier_placements")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(placements, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn tiers_read_in_display_order_and_writes_stay_on_their_list(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let mine = id(&list(&mut conn, user, "mine").await);
    let theirs = id(&list(&mut conn, user, "theirs").await);

    let b = tier(&mut conn, mine, "B", 1).await;
    tier(&mut conn, mine, "S", 0).await;
    let foreign = tier(&mut conn, theirs, "X", 0).await;

    let names: Vec<String> = tier_lists::get_tiers(&mut conn, mine)
        .await
        .unwrap()
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(names, ["S", "B"]);

    let both = tier_lists::get_tiers_for_lists(&mut conn, &[mine, theirs])
        .await
        .unwrap();
    let mut expected: Vec<(Uuid, i16)> = vec![(mine.0, 0), (mine.0, 1), (theirs.0, 0)];
    expected.sort_unstable();
    let got: Vec<(Uuid, i16)> = both
        .iter()
        .map(|t| (t.tier_list_id, t.display_order))
        .collect();
    assert_eq!(got, expected);
    assert!(
        tier_lists::get_tiers_for_lists(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );

    let renamed = tier_lists::update_tier(&mut conn, mine, b, "A", 2, Some("#ff0000"), Some("d"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            renamed.name.as_str(),
            renamed.display_order,
            renamed.color.as_deref()
        ),
        ("A", 2, Some("#ff0000"))
    );
    assert!(
        tier_lists::update_tier(&mut conn, mine, foreign, "hijack", 5, None, None)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        !tier_lists::delete_tier(&mut conn, mine, foreign)
            .await
            .unwrap()
    );
    assert!(tier_lists::delete_tier(&mut conn, mine, b).await.unwrap());
    let left = tier_lists::get_tiers(&mut conn, theirs).await.unwrap();
    assert_eq!(left[0].name, "X");

    let err = tier_lists::create_tier(&mut conn, mine, "dup", 0, None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_placement_upserts_by_kind_and_id_into_its_own_list_and_unknown_kinds_drop(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let mine = id(&list(&mut conn, user, "mine").await);
    let theirs = id(&list(&mut conn, user, "theirs").await);
    let s = tier(&mut conn, mine, "S", 0).await;
    let a = tier(&mut conn, mine, "A", 1).await;
    let foreign = tier(&mut conn, theirs, "X", 0).await;

    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        2,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    let upserted = tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        1,
        Some("x"),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        (upserted.sub_order, upserted.description.as_deref()),
        (1, Some("x"))
    );
    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Skin,
        "char_002_amiya",
        0,
        None,
    )
    .await
    .unwrap()
    .unwrap();
    sqlx::query(
        "INSERT INTO tier_placements (tier_id, entity_kind, entity_id)
         VALUES ($1, 'boss', 'e_001')",
    )
    .bind(a)
    .execute(&mut *conn)
    .await
    .unwrap();
    assert!(
        tier_lists::add_placement(
            &mut conn,
            mine,
            foreign,
            EntityKind::Operator,
            "char_003_kalts",
            0,
            None
        )
        .await
        .unwrap()
        .is_none()
    );

    let in_s: Vec<(EntityKind, i16)> = tier_lists::get_placements(&mut conn, s)
        .await
        .unwrap()
        .into_iter()
        .map(|p| (p.entity_kind, p.sub_order))
        .collect();
    assert_eq!(in_s, [(EntityKind::Skin, 0), (EntityKind::Operator, 1)]);

    let all = tier_lists::get_placements_for_tiers(&mut conn, &[s, a, foreign])
        .await
        .unwrap();
    let got: Vec<(Uuid, i16)> = all.iter().map(|p| (p.tier_id, p.sub_order)).collect();
    assert_eq!(got, [(s, 0), (s, 1)], "the unknown `boss` kind is dropped");
    assert!(
        tier_lists::get_placements(&mut conn, a)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        tier_lists::get_placements_for_tiers(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
}

const STORED_KINDS: [&str; 17] = [
    "operator",
    "class",
    "subclass",
    "enemy",
    "event",
    "faction",
    "stronghold_bond",
    "skin",
    "module",
    "skill",
    "integrated_strategies",
    "story_sprite",
    "main_story",
    "background",
    "story_cg",
    "story_scene",
    "archive_pic",
];

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn every_entity_kind_stores_as_its_serde_name_and_reads_back(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = id(&list(&mut conn, user, "kinds").await);
    let s = tier(&mut conn, row, "S", 0).await;

    assert_eq!(EntityKind::ALL.map(EntityKind::as_str), STORED_KINDS);
    for (order, kind) in (0..).zip(EntityKind::ALL) {
        assert_eq!(serde_json::to_value(kind).unwrap(), kind.as_str());
        assert_eq!(EntityKind::parse(kind.as_str()), Some(kind));
        tier_lists::add_placement(&mut conn, row, s, kind, "x", order, None)
            .await
            .unwrap()
            .unwrap();
    }
    let stored: Vec<String> = sqlx::query_scalar(
        "SELECT entity_kind FROM tier_placements WHERE tier_id = $1 ORDER BY sub_order",
    )
    .bind(s)
    .fetch_all(&mut *conn)
    .await
    .unwrap();
    assert_eq!(stored, STORED_KINDS);
    let read: Vec<EntityKind> = tier_lists::get_placements(&mut conn, s)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.entity_kind)
        .collect();
    assert_eq!(read, EntityKind::ALL);
    assert_eq!(EntityKind::parse("Operator"), None);

    assert_eq!(
        TierListType::ALL.map(TierListType::as_str),
        ["official", "community"]
    );
    for list_type in TierListType::ALL {
        assert_eq!(serde_json::to_value(list_type).unwrap(), list_type.as_str());
        assert_eq!(TierListType::parse(list_type.as_str()), Some(list_type));
    }
    assert_eq!(TierListType::parse("x"), None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn description_and_removal_are_scoped_to_the_list(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let mine = id(&list(&mut conn, user, "mine").await);
    let theirs = id(&list(&mut conn, user, "theirs").await);
    let s = tier(&mut conn, mine, "S", 0).await;
    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        0,
        None,
    )
    .await
    .unwrap();

    let described = tier_lists::set_placement_description(
        &mut conn,
        mine,
        EntityKind::Operator,
        "char_002_amiya",
        Some("d"),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(described.description.as_deref(), Some("d"));
    assert!(
        tier_lists::set_placement_description(
            &mut conn,
            theirs,
            EntityKind::Operator,
            "char_002_amiya",
            None
        )
        .await
        .unwrap()
        .is_none()
    );
    assert!(
        tier_lists::set_placement_description(
            &mut conn,
            mine,
            EntityKind::Skin,
            "char_002_amiya",
            None
        )
        .await
        .unwrap()
        .is_none()
    );

    tier_lists::remove_placement(&mut conn, theirs, s, EntityKind::Operator, "char_002_amiya")
        .await
        .unwrap();
    assert_eq!(
        tier_lists::get_placements(&mut conn, s)
            .await
            .unwrap()
            .len(),
        1
    );
    tier_lists::remove_placement(&mut conn, mine, s, EntityKind::Operator, "char_002_amiya")
        .await
        .unwrap();
    assert!(
        tier_lists::get_placements(&mut conn, s)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_move_racing_a_committed_move_of_the_same_placement_moves_nothing(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let mine = id(&list(&mut conn, user, "raced-move").await);
    let s = tier(&mut conn, mine, "S", 0).await;
    let a = tier(&mut conn, mine, "A", 1).await;
    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        0,
        Some("kept"),
    )
    .await
    .unwrap();

    let mut first = db.begin().await.unwrap();
    assert!(
        tier_lists::move_placement(
            &mut first,
            mine,
            s,
            a,
            EntityKind::Operator,
            "char_002_amiya",
            1
        )
        .await
        .unwrap()
        .is_some()
    );
    let (second, ()) = tokio::join!(
        tier_lists::move_placement(
            &mut conn,
            mine,
            s,
            a,
            EntityKind::Operator,
            "char_002_amiya",
            2
        ),
        async {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            first.commit().await.unwrap();
        }
    );
    assert!(second.unwrap().is_none());

    let placements = count(
        &mut conn,
        "SELECT COUNT(*) FROM tier_placements tp JOIN tiers t ON t.id = tp.tier_id
          WHERE t.tier_list_id = $1",
        mine,
    )
    .await;
    assert_eq!(placements, 1);
    let kept = tier_lists::get_placements(&mut conn, a).await.unwrap();
    assert_eq!(
        (kept[0].sub_order, kept[0].description.as_deref()),
        (1, Some("kept"))
    );

    let within = tier_lists::move_placement(
        &mut conn,
        mine,
        a,
        a,
        EntityKind::Operator,
        "char_002_amiya",
        7,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        (
            within.tier_id,
            within.sub_order,
            within.description.as_deref()
        ),
        (a, 7, Some("kept"))
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_move_carries_the_description_and_refuses_a_foreign_tier(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let mine = id(&list(&mut conn, user, "mine").await);
    let theirs = id(&list(&mut conn, user, "theirs").await);
    let s = tier(&mut conn, mine, "S", 0).await;
    let a = tier(&mut conn, mine, "A", 1).await;
    let foreign = tier(&mut conn, theirs, "X", 0).await;
    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        0,
        Some("kept"),
    )
    .await
    .unwrap();

    assert!(
        tier_lists::move_placement(
            &mut conn,
            mine,
            s,
            foreign,
            EntityKind::Operator,
            "char_002_amiya",
            0
        )
        .await
        .unwrap()
        .is_none()
    );
    assert_eq!(
        tier_lists::get_placements(&mut conn, s)
            .await
            .unwrap()
            .len(),
        1
    );

    let moved = tier_lists::move_placement(
        &mut conn,
        mine,
        s,
        a,
        EntityKind::Operator,
        "char_002_amiya",
        3,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(moved.tier_id, a);
    assert_eq!(
        (moved.sub_order, moved.description.as_deref()),
        (3, Some("kept"))
    );
    assert!(
        tier_lists::get_placements(&mut conn, s)
            .await
            .unwrap()
            .is_empty()
    );

    // The entity is already in `a`: the move lands on the upsert, one row survives
    tier_lists::add_placement(
        &mut conn,
        mine,
        s,
        EntityKind::Operator,
        "char_002_amiya",
        0,
        None,
    )
    .await
    .unwrap();
    let merged = tier_lists::move_placement(
        &mut conn,
        mine,
        s,
        a,
        EntityKind::Operator,
        "char_002_amiya",
        5,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!((merged.sub_order, merged.description), (5, None));
    assert_eq!(
        tier_lists::get_placements(&mut conn, a)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        tier_lists::get_placements(&mut conn, s)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_user_permission_is_their_highest_level_on_the_list(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let editor = common::user(&mut conn, 2).await;
    let other_admin = common::user(&mut conn, 3).await;
    let row = id(&list(&mut conn, owner, "perm").await);

    tier_lists::grant_permission(&mut conn, row, editor, "edit", owner)
        .await
        .unwrap();
    tier_lists::grant_permission(&mut conn, row, editor, "edit", other_admin)
        .await
        .unwrap();
    tier_lists::grant_permission(&mut conn, row, editor, "publish", owner)
        .await
        .unwrap();
    tier_lists::grant_permission(&mut conn, row, editor, "view", owner)
        .await
        .unwrap();

    let highest = tier_lists::get_user_permission(&mut conn, row, editor)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(highest.permission, "publish");
    tier_lists::grant_permission(&mut conn, row, other_admin, "admin", owner)
        .await
        .unwrap();
    let all = tier_lists::get_permissions(&mut conn, row).await.unwrap();
    let order: Vec<(Uuid, &str)> = all
        .iter()
        .map(|p| (p.user_id, p.permission.as_str()))
        .collect();
    assert_eq!(
        order,
        [
            (other_admin.0, "admin"),
            (editor.0, "publish"),
            (editor.0, "edit"),
            (editor.0, "view")
        ]
    );
    assert_eq!(all[2].granted_by, Some(owner.0));
    tier_lists::revoke_permission(&mut conn, row, other_admin, "admin")
        .await
        .unwrap();

    tier_lists::revoke_permission(&mut conn, row, editor, "publish")
        .await
        .unwrap();
    let after = tier_lists::get_user_permission(&mut conn, row, editor)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after.permission, "edit");
    assert!(
        tier_lists::get_user_permission(&mut conn, row, other_admin)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_upserted_grant_refreshes_its_time_and_keeps_its_granter(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let editor = common::user(&mut conn, 2).await;
    let row = id(&list(&mut conn, owner, "cli").await);

    tier_lists::upsert_grant(&mut conn, row, editor, "edit", Some(owner))
        .await
        .unwrap();
    sqlx::query("UPDATE tier_list_permissions SET granted_at = NOW() - INTERVAL '1 day'")
        .execute(&mut *conn)
        .await
        .unwrap();
    let before = tier_lists::get_permissions(&mut conn, row).await.unwrap()[0].granted_at;
    tier_lists::upsert_grant(&mut conn, row, editor, "edit", None)
        .await
        .unwrap();
    let after = &tier_lists::get_permissions(&mut conn, row).await.unwrap()[0];
    assert!(after.granted_at > before);
    assert_eq!(after.granted_by, Some(owner.0));

    tier_lists::upsert_grant(&mut conn, row, editor, "view", None)
        .await
        .unwrap();
    let view = tier_lists::get_user_permission(&mut conn, row, editor)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(view.permission, "edit");
    let grants = tier_lists::get_permissions(&mut conn, row).await.unwrap();
    let anonymous = grants.iter().find(|p| p.permission == "view").unwrap();
    assert_eq!(anonymous.granted_by, None);
}

/// Users 1 (owner), 2 and 3 with grants on lists `alpha`, `beta` and the inactive `gone`.
/// Returns (editor, viewer, alpha).
async fn granted(conn: &mut Conn) -> (UserId, UserId, TierListId) {
    let owner = common::user(conn, 1).await;
    let editor = common::user(conn, 2).await;
    let viewer = common::user(conn, 3).await;
    let beta = id(
        &tier_lists::create(conn, "beta", "beta", None, TierListType::Community, owner)
            .await
            .unwrap(),
    );
    let alpha =
        id(
            &tier_lists::create(conn, "Alpha", "alpha", None, TierListType::Official, owner)
                .await
                .unwrap(),
        );
    let gone = id(&list(conn, owner, "gone").await);

    tier_lists::grant_permission(conn, beta, editor, "view", owner)
        .await
        .unwrap();
    tier_lists::grant_permission(conn, beta, editor, "admin", owner)
        .await
        .unwrap();
    tier_lists::upsert_grant(conn, alpha, editor, "edit", None)
        .await
        .unwrap();
    tier_lists::grant_permission(conn, alpha, viewer, "view", owner)
        .await
        .unwrap();
    tier_lists::grant_permission(conn, gone, editor, "admin", owner)
        .await
        .unwrap();
    deactivate(conn, gone).await;
    sqlx::query(
        "UPDATE tier_list_permissions
         SET granted_at = NOW() - make_interval(mins => (CASE permission
             WHEN 'view' THEN 1 WHEN 'admin' THEN 2 ELSE 3 END))",
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    (editor, viewer, alpha)
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn grants_on_inactive_lists_drop_out_of_the_admin_feed(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let (_, viewer, _) = granted(&mut conn).await;

    let all = tier_lists::get_all_grants(&mut conn, None).await.unwrap();
    let shape: Vec<(&str, &str)> = all
        .iter()
        .map(|g| (g.slug.as_str(), g.permission.as_str()))
        .collect();
    assert_eq!(
        shape,
        [
            ("alpha", "view"),
            ("beta", "view"),
            ("beta", "admin"),
            ("alpha", "edit")
        ]
    );
    let edit = &all[3];
    assert_eq!(
        (edit.title.as_str(), edit.list_type.as_str()),
        ("Alpha", "official")
    );
    assert_eq!(
        (edit.user_uid.as_str(), edit.user_nickname.as_deref()),
        ("2", Some("user2"))
    );
    assert_eq!(
        (edit.granted_by, edit.granted_by_nickname.as_deref()),
        (None, None)
    );
    assert_eq!(all[2].granted_by_nickname.as_deref(), Some("user1"));

    let for_viewer = tier_lists::get_all_grants(&mut conn, Some(viewer))
        .await
        .unwrap();
    assert_eq!(for_viewer.len(), 1);
    assert_eq!(for_viewer[0].user_id, viewer.0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_grantee_sees_each_active_list_once_at_their_highest_level(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let (editor, viewer, alpha) = granted(&mut conn).await;

    let lists = tier_lists::get_granted_lists(&mut conn, editor)
        .await
        .unwrap();
    let shape: Vec<(&str, &str, &str)> = lists
        .iter()
        .map(|g| (g.title.as_str(), g.slug.as_str(), g.permission.as_str()))
        .collect();
    assert_eq!(
        shape,
        [("Alpha", "alpha", "edit"), ("beta", "beta", "admin")]
    );
    assert_eq!(lists[0].tier_list_id, alpha.0);
    assert_eq!(lists[0].list_type, "official");

    let publisher = common::user(&mut conn, 4).await;
    tier_lists::grant_permission(&mut conn, alpha, publisher, "publish", editor)
        .await
        .unwrap();
    let grantees = tier_lists::list_grantees(&mut conn, alpha).await.unwrap();
    let shape: Vec<(&str, &str)> = grantees
        .iter()
        .map(|g| (g.permission.as_str(), g.uid.as_str()))
        .collect();
    assert_eq!(shape, [("publish", "4"), ("edit", "2"), ("view", "3")]);
    assert_eq!(grantees[2].nickname.as_deref(), Some("user3"));
    assert_eq!(grantees[2].user_id, viewer.0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_share_counts_even_before_the_stats_row_exists(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let a = id(&list(&mut conn, user, "a").await);
    let b = id(&list(&mut conn, user, "b").await);

    assert!(tier_lists::get_stats(&mut conn, a).await.unwrap().is_none());
    tier_lists::bump_share(&mut conn, a).await.unwrap();
    let stats = tier_lists::get_stats(&mut conn, a).await.unwrap().unwrap();
    assert_eq!(stats.share_count, 1);

    tier_lists::ensure_stats_row(&mut conn, a).await.unwrap();
    tier_lists::bump_share(&mut conn, a).await.unwrap();
    let stats = tier_lists::get_stats(&mut conn, a).await.unwrap().unwrap();
    assert_eq!((stats.share_count, stats.view_count), (2, 0));
    tier_lists::bump_share(&mut conn, TierListId(Uuid::new_v4()))
        .await
        .unwrap();
    assert!(!stats.is_trending);
    assert_eq!(stats.last_viewed_at, None);

    tier_lists::ensure_stats_row(&mut conn, b).await.unwrap();
    let both = tier_lists::get_stats_for_lists(&mut conn, &[a, b])
        .await
        .unwrap();
    assert_eq!(both.len(), 2);
    assert!(
        tier_lists::get_stats_for_lists(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_returning_viewer_counts_as_a_view_but_not_as_a_new_unique_one(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = id(&list(&mut conn, user, "viewed").await);
    tier_lists::ensure_stats_row(&mut conn, row).await.unwrap();

    assert!(
        tier_lists::record_view(&mut conn, row, Viewer::Session(SESSION))
            .await
            .unwrap()
    );
    assert!(
        !tier_lists::record_view(&mut conn, row, Viewer::Session(SESSION))
            .await
            .unwrap()
    );
    assert_eq!(view_counts(&mut conn, row).await, (1, 1, 1));

    assert!(
        tier_lists::record_view(&mut conn, row, Viewer::Account(user))
            .await
            .unwrap()
    );
    assert!(
        !tier_lists::record_view(&mut conn, row, Viewer::Account(user))
            .await
            .unwrap()
    );
    assert_eq!(view_counts(&mut conn, row).await, (2, 2, 2));
    let stats = tier_lists::get_stats(&mut conn, row)
        .await
        .unwrap()
        .unwrap();
    assert!(stats.last_viewed_at.is_some());

    sqlx::query("UPDATE tier_list_view_events SET viewed_at = NOW() - INTERVAL '31 minutes'")
        .execute(&mut *conn)
        .await
        .unwrap();
    assert!(
        tier_lists::record_view(&mut conn, row, Viewer::Session(SESSION))
            .await
            .unwrap()
    );
    assert_eq!(view_counts(&mut conn, row).await, (3, 3, 2));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_first_view_racing_an_uncommitted_one_by_the_same_viewer_is_not_counted(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = id(&list(&mut conn, user, "raced").await);

    let mut first = db.begin().await.unwrap();
    assert!(
        tier_lists::record_view(&mut first, row, Viewer::Session(SESSION))
            .await
            .unwrap()
    );
    let (second, ()) = tokio::join!(
        tier_lists::record_view(&mut conn, row, Viewer::Session(SESSION)),
        async {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            first.commit().await.unwrap();
        }
    );
    assert!(!second.unwrap());
    assert_eq!(view_counts(&mut conn, row).await, (1, 1, 1));
}

#[test]
fn a_viewer_is_the_account_first_then_the_session() {
    let user = UserId(Uuid::new_v4());
    assert_eq!(
        Viewer::from_parts(Some(user), Some(SESSION)),
        Some(Viewer::Account(user))
    );
    assert_eq!(
        Viewer::from_parts(None, Some(SESSION)),
        Some(Viewer::Session(SESSION))
    );
    assert_eq!(Viewer::from_parts(None, None), None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_view_window_on_one_list_does_not_hold_back_another(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let a = id(&list(&mut conn, user, "a").await);
    let b = id(&list(&mut conn, user, "b").await);

    for viewer in [Viewer::Account(user), Viewer::Session(SESSION)] {
        assert!(tier_lists::record_view(&mut conn, a, viewer).await.unwrap());
        assert!(tier_lists::record_view(&mut conn, b, viewer).await.unwrap());
    }
    assert_eq!(view_counts(&mut conn, a).await, (2, 2, 2));
    assert_eq!(view_counts(&mut conn, b).await, (2, 2, 2));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_counted_view_creates_the_missing_stats_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let row = id(&list(&mut conn, user, "fresh").await);

    assert!(
        tier_lists::record_view(&mut conn, row, Viewer::Account(user))
            .await
            .unwrap()
    );
    let stats = tier_lists::get_stats(&mut conn, row)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((stats.view_count, stats.unique_view_count), (1, 1));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn favorites_move_the_counter_only_on_a_change(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let row = id(&list(&mut conn, user, "fav").await);
    tier_lists::ensure_stats_row(&mut conn, row).await.unwrap();

    assert!(
        !tier_lists::is_favorited(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert!(
        tier_lists::add_favorite(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert!(
        !tier_lists::add_favorite(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert!(
        tier_lists::is_favorited(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert!(
        !tier_lists::is_favorited(&mut conn, row, other)
            .await
            .unwrap()
    );
    assert_eq!(favorite_count(&mut conn, row).await, 1);

    assert!(
        tier_lists::remove_favorite(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert!(
        !tier_lists::remove_favorite(&mut conn, row, user)
            .await
            .unwrap()
    );
    assert_eq!(favorite_count(&mut conn, row).await, 0);

    tier_lists::add_favorite(&mut conn, row, other)
        .await
        .unwrap();
    sqlx::query("UPDATE tier_list_stats SET favorite_count = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    assert!(
        tier_lists::remove_favorite(&mut conn, row, other)
            .await
            .unwrap()
    );
    assert_eq!(favorite_count(&mut conn, row).await, 0);
}

async fn raw_favorite(conn: &mut Conn, list: TierListId, user: UserId) {
    sqlx::query("INSERT INTO tier_list_favorites (tier_list_id, user_id) VALUES ($1, $2)")
        .bind(list.0)
        .bind(user.0)
        .execute(conn)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_favorite_sets_the_counter_from_the_favorites_without_a_stats_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let fans = [
        common::user(&mut conn, 2).await,
        common::user(&mut conn, 3).await,
    ];
    let added = id(&list(&mut conn, owner, "added").await);
    let removed = id(&list(&mut conn, owner, "removed").await);
    raw_favorite(&mut conn, removed, owner).await;
    assert!(
        tier_lists::remove_favorite(&mut conn, removed, owner)
            .await
            .unwrap()
    );
    assert!(
        tier_lists::get_stats(&mut conn, removed)
            .await
            .unwrap()
            .is_none()
    );

    for fan in fans {
        raw_favorite(&mut conn, added, fan).await;
    }
    assert!(
        tier_lists::add_favorite(&mut conn, added, owner)
            .await
            .unwrap()
    );
    assert_eq!(favorite_count(&mut conn, added).await, 3);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_favorite_racing_an_uncommitted_one_counts_both(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let fan = common::user(&mut conn, 2).await;
    let row = id(&list(&mut conn, owner, "raced-fav").await);

    let mut first = db.begin().await.unwrap();
    assert!(
        tier_lists::add_favorite(&mut first, row, owner)
            .await
            .unwrap()
    );
    let (second, ()) = tokio::join!(tier_lists::add_favorite(&mut conn, row, fan), async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        first.commit().await.unwrap();
    });
    assert!(second.unwrap());
    assert_eq!(favorite_count(&mut conn, row).await, 2);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn favorited_lists_include_unlisted_ones_newest_favorite_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let older = id(&list(&mut conn, user, "older").await);
    let unlisted = id(&list(&mut conn, user, "unlisted").await);
    let gone = id(&list(&mut conn, user, "gone").await);
    for (row, minutes) in [(older, 30), (unlisted, 10), (gone, 1)] {
        tier_lists::add_favorite(&mut conn, row, user)
            .await
            .unwrap();
        sqlx::query(
            "UPDATE tier_list_favorites SET favorited_at = NOW() - make_interval(mins => $2)
             WHERE tier_list_id = $1",
        )
        .bind(row.0)
        .bind(minutes)
        .execute(&mut *conn)
        .await
        .unwrap();
    }
    tier_lists::set_visibility(&mut conn, unlisted, false)
        .await
        .unwrap();
    deactivate(&mut conn, gone).await;

    let rows = tier_lists::find_favorited_by_user(&mut conn, user)
        .await
        .unwrap();
    assert_eq!(slugs(&rows), ["unlisted", "older"]);
    assert!(!rows[0].is_listed);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn flairs_list_by_display_order_then_label(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let b = tier_lists::create_flair(&mut conn, "b", "Bravo", Some("#000000"), 1)
        .await
        .unwrap();
    let a = tier_lists::create_flair(&mut conn, "a", "Alpha", None, 1)
        .await
        .unwrap();
    let z = tier_lists::create_flair(&mut conn, "z", "Zulu", None, 0)
        .await
        .unwrap();
    assert!(b.is_active);
    let retired = tier_lists::update_flair(&mut conn, z.id, "Zed", Some("#ffffff"), 0, false)
        .await
        .unwrap();
    assert_eq!(
        (
            retired.label.as_str(),
            retired.color.as_deref(),
            retired.is_active
        ),
        ("Zed", Some("#ffffff"), false)
    );

    let labels = |rows: Vec<tier_lists::FlairRow>| -> Vec<String> {
        rows.into_iter().map(|f| f.label).collect()
    };
    assert_eq!(
        labels(tier_lists::list_flairs(&mut conn, true).await.unwrap()),
        ["Alpha", "Bravo"]
    );
    assert_eq!(
        labels(tier_lists::list_flairs(&mut conn, false).await.unwrap()),
        ["Zed", "Alpha", "Bravo"]
    );

    let mut by_ids: Vec<i16> = tier_lists::get_flairs_by_ids(&mut conn, &[a.id, z.id])
        .await
        .unwrap()
        .iter()
        .map(|f| f.id)
        .collect();
    by_ids.sort_unstable();
    let mut expected = vec![a.id, z.id];
    expected.sort_unstable();
    assert_eq!(by_ids, expected);
    assert!(
        tier_lists::get_flairs_by_ids(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
    let one = tier_lists::get_flair_by_id(&mut conn, b.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(one.code, "b");
    assert!(
        tier_lists::get_flair_by_id(&mut conn, -1)
            .await
            .unwrap()
            .is_none()
    );

    let err = tier_lists::create_flair(&mut conn, "a", "Again", None, 0)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::Conflict { .. }));
    let err = tier_lists::update_flair(&mut conn, -1, "x", None, 0, true)
        .await
        .unwrap_err();
    assert!(matches!(err, DbError::NotFound));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn trending_goes_to_the_top_scored_active_lists_and_inactive_ones_reset(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let hot = id(&list(&mut conn, user, "hot").await);
    let warm = id(&list(&mut conn, user, "warm").await);
    let cold = id(&list(&mut conn, user, "cold").await);
    let gone = id(&list(&mut conn, user, "gone").await);
    for row in [hot, warm, cold, gone] {
        tier_lists::ensure_stats_row(&mut conn, row).await.unwrap();
    }
    sqlx::query(
        "INSERT INTO tier_list_view_events (tier_list_id, viewed_at)
         SELECT $1::uuid, NOW() FROM generate_series(1, 10)
         UNION ALL SELECT $2::uuid, NOW() - INTERVAL '2 days' FROM generate_series(1, 20)
         UNION ALL SELECT $3::uuid, NOW() - INTERVAL '30 days'",
    )
    .bind(hot.0)
    .bind(warm.0)
    .bind(gone.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    tier_lists::add_favorite(&mut conn, warm, user)
        .await
        .unwrap();
    sqlx::query("UPDATE tier_list_stats SET is_trending = true WHERE tier_list_id = $1")
        .bind(gone.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    deactivate(&mut conn, gone).await;
    let dark = id(&list(&mut conn, user, "dark").await);
    tier_lists::ensure_stats_row(&mut conn, dark).await.unwrap();
    sqlx::query(
        "INSERT INTO tier_list_view_events (tier_list_id, viewed_at)
         SELECT $1::uuid, NOW() FROM generate_series(1, 50)",
    )
    .bind(dark.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE tier_list_stats SET trending_score = 1000, views_last_24h = 50, views_last_7d = 50
         WHERE tier_list_id = $1",
    )
    .bind(dark.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    deactivate(&mut conn, dark).await;

    tier_lists::recompute_trending(&mut conn, 1).await.unwrap();

    let stats = tier_lists::get_stats_for_lists(&mut conn, &[hot, warm, cold, gone, dark])
        .await
        .unwrap();
    let of = |list: TierListId| stats.iter().find(|s| s.tier_list_id == list.0).unwrap();
    assert_eq!((of(hot).views_last_24h, of(hot).views_last_7d), (10, 10));
    assert_eq!((of(warm).views_last_24h, of(warm).views_last_7d), (0, 20));
    assert!(of(hot).trending_score > of(warm).trending_score);
    assert!(of(warm).trending_score > of(cold).trending_score);
    assert!(of(hot).is_trending);
    assert!(!of(warm).is_trending && !of(cold).is_trending && !of(gone).is_trending);
    assert_eq!(of(gone).views_last_7d, 0);
    for inactive in [of(gone), of(dark)] {
        assert_eq!(
            (
                inactive.views_last_24h,
                inactive.views_last_7d,
                inactive.trending_score,
                inactive.is_trending
            ),
            (0, 0, 0.0, false)
        );
    }

    tier_lists::recompute_trending(&mut conn, 5).await.unwrap();
    let stats = tier_lists::get_stats_for_lists(&mut conn, &[hot, warm, cold])
        .await
        .unwrap();
    let trending: Vec<bool> = [hot, warm, cold]
        .iter()
        .map(|l| {
            stats
                .iter()
                .find(|s| s.tier_list_id == l.0)
                .unwrap()
                .is_trending
        })
        .collect();
    assert_eq!(trending, [true, true, false]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn authors_and_showcase_slugs_resolve_in_batches(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = common::user(&mut conn, 1).await;
    let b = common::user(&mut conn, 2).await;

    let author = tier_lists::find_author(&mut conn, a)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (author.uid.as_str(), author.nickname.as_deref()),
        ("1", Some("user1"))
    );
    assert_eq!(author.avatar_id, None);
    assert!(
        tier_lists::find_author(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );
    let mut uids: Vec<String> =
        tier_lists::find_authors(&mut conn, &[a, b, UserId(Uuid::new_v4())])
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.uid)
            .collect();
    uids.sort_unstable();
    assert_eq!(uids, ["1", "2"]);
    assert!(
        tier_lists::find_authors(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );

    list(&mut conn, a, "live").await;
    let gone = id(&list(&mut conn, a, "gone").await);
    deactivate(&mut conn, gone).await;
    let asked = ["live".to_owned(), "gone".to_owned(), "missing".to_owned()];
    assert_eq!(
        tier_lists::active_slugs(&mut conn, &asked).await.unwrap(),
        ["live"]
    );
    assert!(
        tier_lists::active_slugs(&mut conn, &[])
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn site_counts_and_the_cli_listings_include_what_they_should(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let zeta = id(&tier_lists::create(
        &mut conn,
        "Zeta",
        "zeta",
        None,
        TierListType::Community,
        user,
    )
    .await
    .unwrap());
    let alpha = id(&tier_lists::create(
        &mut conn,
        "Alpha",
        "alpha",
        None,
        TierListType::Official,
        user,
    )
    .await
    .unwrap());
    let gone = id(&tier_lists::create(
        &mut conn,
        "Gone",
        "gone",
        None,
        TierListType::Community,
        user,
    )
    .await
    .unwrap());
    sqlx::query(
        "UPDATE tier_lists SET created_at = NOW() - make_interval(mins => CASE slug
             WHEN 'zeta' THEN 3 WHEN 'alpha' THEN 2 ELSE 1 END)",
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    let s = tier(&mut conn, zeta, "S", 0).await;
    tier(&mut conn, zeta, "A", 1).await;
    for op in ["char_002_amiya", "char_003_kalts"] {
        tier_lists::add_placement(&mut conn, zeta, s, EntityKind::Operator, op, 0, None)
            .await
            .unwrap();
    }
    tier_lists::create_version(&mut conn, alpha, 1, &json!({}), None, user)
        .await
        .unwrap();
    deactivate(&mut conn, gone).await;

    let counts = tier_lists::site_counts(&mut conn).await.unwrap();
    assert_eq!(
        (
            counts.total,
            counts.active,
            counts.total_versions,
            counts.total_placements
        ),
        (3, 2, 1, 2)
    );

    let names: Vec<String> = tier_lists::list_active_names(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(names, ["Alpha", "Zeta"]);

    let details = tier_lists::list_details(&mut conn).await.unwrap();
    let shape: Vec<(&str, bool, i64, i64)> = details
        .iter()
        .map(|d| {
            (
                d.slug.as_str(),
                d.is_active,
                d.tier_count,
                d.placement_count,
            )
        })
        .collect();
    assert_eq!(
        shape,
        [
            ("gone", false, 0, 0),
            ("alpha", true, 0, 0),
            ("zeta", true, 2, 2)
        ]
    );
    assert_eq!(details[1].list_type, "official");
    assert!(details[1].is_listed);
}
