#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::content_audit::{self, ContentAuditFilter, NewContentAudit};
use myrtle_types::{ContentAuditAction, ContentTargetKind, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const fn entry(
    actor: UserId,
    kind: ContentTargetKind,
    target: Uuid,
    details: &Value,
) -> NewContentAudit<'_> {
    NewContentAudit {
        actor_id: actor,
        target_kind: kind,
        target_id: target,
        target_name: "Meta",
        target_slug: Some("meta"),
        owner_id: None,
        subject_id: None,
        action: ContentAuditAction::Edit,
        details,
    }
}

/// Inserts `entry` and backdates it `minutes`, returning its id
async fn logged(conn: &mut Conn, entry: &NewContentAudit<'_>, minutes: i32) -> i64 {
    content_audit::insert(conn, entry).await.unwrap();
    sqlx::query_scalar(
        "UPDATE content_audit_log
         SET changed_at = NOW() - make_interval(mins => $1),
             first_at = NOW() - make_interval(mins => $1)
         WHERE id = (SELECT MAX(id) FROM content_audit_log)
         RETURNING id",
    )
    .bind(minutes)
    .fetch_one(conn)
    .await
    .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_insert_stores_every_field_as_one_event(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let actor = common::user(&mut conn, 1).await;
    let owner = common::user(&mut conn, 2).await;
    let subject = common::user(&mut conn, 3).await;
    let target = Uuid::new_v4();
    let details = json!({"tiersAdded": [{"id": target, "name": "S"}]});

    content_audit::insert(
        &mut conn,
        &NewContentAudit {
            owner_id: Some(owner),
            subject_id: Some(subject),
            action: ContentAuditAction::Grant,
            ..entry(actor, ContentTargetKind::Grid, target, &details)
        },
    )
    .await
    .unwrap();

    let row = content_audit::list(&mut conn, ContentAuditFilter::default(), 10, None)
        .await
        .unwrap()
        .remove(0)
        .row;
    assert_eq!(
        (row.actor_id, row.target_kind.as_str(), row.target_id),
        (actor.0, "grid", target)
    );
    assert_eq!(
        (row.target_name.as_str(), row.target_slug.as_deref()),
        ("Meta", Some("meta"))
    );
    assert_eq!(
        (row.owner_id, row.subject_id),
        (Some(owner.0), Some(subject.0))
    );
    assert_eq!((row.action.as_str(), row.event_count), ("grant", 1));
    assert_eq!(row.details, details);
    assert_eq!(row.first_at, row.changed_at);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_open_row_is_the_actors_newest_on_that_target_inside_the_window(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let actor = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let target = Uuid::new_v4();
    let details = json!({});

    logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, target, &details),
        30,
    )
    .await;
    let newest = logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, target, &details),
        5,
    )
    .await;
    logged(
        &mut conn,
        &entry(other, ContentTargetKind::TierList, target, &details),
        1,
    )
    .await;
    logged(
        &mut conn,
        &entry(actor, ContentTargetKind::Grid, target, &details),
        1,
    )
    .await;
    logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, Uuid::new_v4(), &details),
        1,
    )
    .await;

    let open =
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 600.0)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(open.id, newest);
    assert!(
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 240.0)
            .await
            .unwrap()
            .is_none()
    );

    // Same `changed_at`: the higher id wins
    let tie = logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, target, &details),
        5,
    )
    .await;
    sqlx::query(
        "UPDATE content_audit_log
         SET changed_at = (SELECT changed_at FROM content_audit_log WHERE id = $1)
         WHERE id = $2",
    )
    .bind(newest)
    .bind(tie)
    .execute(&mut *conn)
    .await
    .unwrap();
    let open =
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 600.0)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(open.id, tie);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_open_row_stays_locked_until_the_callers_transaction_ends(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let actor = common::user(&mut conn, 1).await;
    let target = Uuid::new_v4();
    let details = json!({});
    let id = logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, target, &details),
        0,
    )
    .await;
    let probe = "SELECT id FROM content_audit_log WHERE id = $1 FOR UPDATE NOWAIT";

    let mut tx = db.begin().await.unwrap();
    content_audit::latest_locked(&mut tx, actor, ContentTargetKind::TierList, target, 60.0)
        .await
        .unwrap()
        .unwrap();
    let err = sqlx::query(probe)
        .bind(id)
        .execute(&mut *conn)
        .await
        .unwrap_err();
    assert_eq!(
        err.as_database_error().unwrap().code().as_deref(),
        Some("55P03")
    );
    tx.commit().await.unwrap();

    sqlx::query(probe)
        .bind(id)
        .execute(&mut *conn)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_coalesced_edit_counts_one_more_event_and_keeps_the_slug_when_given_none(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let actor = common::user(&mut conn, 1).await;
    let target = Uuid::new_v4();
    let details = json!({"a": 1});
    let id = logged(
        &mut conn,
        &entry(actor, ContentTargetKind::TierList, target, &details),
        3,
    )
    .await;
    let before =
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 600.0)
            .await
            .unwrap()
            .unwrap();

    let merged = json!({"a": 1, "b": 2});
    content_audit::update_coalesced(&mut conn, id, "Renamed", None, &merged)
        .await
        .unwrap();
    let after =
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 600.0)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(after.event_count, 2);
    assert_eq!(after.details, merged);
    assert_eq!(
        (after.target_name.as_str(), after.target_slug.as_deref()),
        ("Renamed", Some("meta"))
    );
    assert_eq!(after.first_at, before.first_at);
    assert!(after.changed_at > before.changed_at);

    content_audit::update_coalesced(&mut conn, id, "Renamed", Some("renamed"), &merged)
        .await
        .unwrap();
    let last =
        content_audit::latest_locked(&mut conn, actor, ContentTargetKind::TierList, target, 600.0)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(
        (last.event_count, last.target_slug.as_deref()),
        (3, Some("renamed"))
    );
}

/// Three rows, 30, 20 and 10 minutes old: the actor on a list, another user on a grid
/// naming an owner, the actor on that grid
struct Feed {
    actor: UserId,
    list: Uuid,
    grid: Uuid,
    oldest: i64,
    middle: i64,
    newest: i64,
}

async fn feed(conn: &mut Conn) -> Feed {
    let actor = common::user(conn, 1).await;
    let other = common::user(conn, 2).await;
    let owner = common::user(conn, 3).await;
    sqlx::query(
        "UPDATE users
         SET secretary = 'char_002_amiya', secretary_skin_id = 'char_002_amiya@epoque#4'
         WHERE id = $1",
    )
    .bind(actor.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    let list = Uuid::new_v4();
    let grid = Uuid::new_v4();
    let details = json!({});

    let oldest = logged(
        conn,
        &entry(actor, ContentTargetKind::TierList, list, &details),
        30,
    )
    .await;
    let middle = logged(
        conn,
        &NewContentAudit {
            owner_id: Some(owner),
            ..entry(other, ContentTargetKind::Grid, grid, &details)
        },
        20,
    )
    .await;
    let newest = logged(
        conn,
        &entry(actor, ContentTargetKind::Grid, grid, &details),
        10,
    )
    .await;
    Feed {
        actor,
        list,
        grid,
        oldest,
        middle,
        newest,
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_feed_pages_newest_first_with_the_people_it_names(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let Feed {
        oldest,
        middle,
        newest,
        ..
    } = feed(&mut conn).await;

    let all = content_audit::list(&mut conn, ContentAuditFilter::default(), 10, None)
        .await
        .unwrap();
    let ids: Vec<i64> = all.iter().map(|r| r.row.id).collect();
    assert_eq!(ids, [newest, middle, oldest]);
    let first = &all[0];
    assert_eq!(
        (first.actor_uid.as_deref(), first.actor_nickname.as_deref()),
        (Some("1"), Some("user1"))
    );
    assert_eq!(first.actor_secretary.as_deref(), Some("char_002_amiya"));
    assert_eq!(
        first.actor_secretary_skin_id.as_deref(),
        Some("char_002_amiya@epoque#4")
    );
    assert_eq!(
        (first.owner_uid.as_deref(), first.subject_uid.as_deref()),
        (None, None)
    );
    assert_eq!(
        (
            all[1].owner_uid.as_deref(),
            all[1].owner_nickname.as_deref()
        ),
        (Some("3"), Some("user3"))
    );
    assert_eq!(all[1].subject_nickname, None);

    let page = content_audit::list(
        &mut conn,
        ContentAuditFilter::default(),
        1,
        Some(all[0].row.changed_at),
    )
    .await
    .unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].row.id, middle);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_feed_and_its_count_apply_every_filter(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let Feed {
        actor,
        list,
        grid,
        oldest,
        middle,
        newest,
    } = feed(&mut conn).await;

    let by_actor = ContentAuditFilter {
        actor: Some(actor),
        ..ContentAuditFilter::default()
    };
    let grids = ContentAuditFilter {
        target_kind: Some(ContentTargetKind::Grid),
        ..ContentAuditFilter::default()
    };
    let on_list = ContentAuditFilter {
        target_id: Some(list),
        ..ContentAuditFilter::default()
    };
    let actor_on_grid = ContentAuditFilter {
        actor: Some(actor),
        target_kind: Some(ContentTargetKind::Grid),
        target_id: Some(grid),
    };
    for (filter, expected) in [
        (by_actor, vec![newest, oldest]),
        (grids, vec![newest, middle]),
        (on_list, vec![oldest]),
        (actor_on_grid, vec![newest]),
    ] {
        let ids: Vec<i64> = content_audit::list(&mut conn, filter, 10, None)
            .await
            .unwrap()
            .iter()
            .map(|r| r.row.id)
            .collect();
        assert_eq!(ids, expected, "{filter:?}");
        assert_eq!(
            content_audit::count(&mut conn, filter).await.unwrap(),
            expected.len() as i64,
            "{filter:?}"
        );
    }
    assert_eq!(
        content_audit::count(&mut conn, ContentAuditFilter::default())
            .await
            .unwrap(),
        3
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn every_action_and_target_kind_stores_as_its_serde_name(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let actor = common::user(&mut conn, 1).await;
    let details = json!({});
    assert_eq!(
        ContentAuditAction::ALL.map(ContentAuditAction::as_str),
        [
            "create",
            "fork",
            "edit",
            "publish",
            "visibility",
            "flair",
            "grant",
            "revoke",
            "delete"
        ]
    );
    assert_eq!(
        ContentTargetKind::ALL.map(ContentTargetKind::as_str),
        ["tier_list", "grid"]
    );
    let actions = ContentAuditAction::ALL;
    for (minutes, action) in (0..).zip(actions) {
        for kind in ContentTargetKind::ALL {
            let entry = NewContentAudit {
                action,
                ..entry(actor, kind, Uuid::new_v4(), &details)
            };
            logged(&mut conn, &entry, minutes).await;
        }
    }

    let rows = content_audit::list(&mut conn, ContentAuditFilter::default(), 100, None)
        .await
        .unwrap();
    assert_eq!(rows.len(), actions.len() * 2);
    for (pair, action) in rows.chunks(2).zip(actions) {
        assert_eq!(serde_json::to_value(action).unwrap(), action.as_str());
        for feed in pair {
            assert_eq!(ContentAuditAction::parse(&feed.row.action), Some(action));
            let kind = ContentTargetKind::parse(&feed.row.target_kind).unwrap();
            assert_eq!(serde_json::to_value(kind).unwrap(), kind.as_str());
        }
    }
    assert_eq!(ContentAuditAction::parse("Edit"), None);
    assert_eq!(ContentTargetKind::parse("tierlist"), None);
}
