#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::release::{self, OverrideInput, Sighting, UpsertStats};
use myrtle_types::Server;
use serde_json::json;
use sqlx::PgPool;

const fn sighting<'a>(
    kind: &'a str,
    id: &'a str,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Sighting<'a> {
    Sighting {
        kind,
        id,
        start_time,
        end_time,
    }
}

const fn manual<'a>(kind: &'a str, cn_id: &'a str) -> OverrideInput<'a> {
    OverrideInput {
        kind,
        cn_id,
        en_id: None,
        en_name: None,
        en_start: None,
        en_end: None,
        featured_chars: None,
        source: "manual",
        note: "",
        updated_by: None,
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_repeated_sighting_is_refreshed_and_keeps_its_first_resource_version(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    assert_eq!(
        release::upsert_sightings(&mut conn, Server::CN, Some("v1"), &[])
            .await
            .unwrap(),
        UpsertStats::default()
    );

    let first = [
        sighting("pool", "p1", Some(10), Some(20)),
        sighting("char", "c1", None, None),
    ];
    assert_eq!(
        release::upsert_sightings(&mut conn, Server::CN, Some("v1"), &first)
            .await
            .unwrap(),
        UpsertStats {
            inserted: 2,
            refreshed: 0
        }
    );

    let second = [
        sighting("pool", "p1", Some(11), None),
        sighting("char", "c2", None, None),
    ];
    assert_eq!(
        release::upsert_sightings(&mut conn, Server::CN, Some("v2"), &second)
            .await
            .unwrap(),
        UpsertStats {
            inserted: 1,
            refreshed: 1
        }
    );

    let (version, start, end): (Option<String>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT res_version, start_time, end_time FROM gamedata_sightings
         WHERE server_id = $1 AND kind = 'pool' AND id = 'p1'",
    )
    .bind(Server::CN.index())
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!(version.as_deref(), Some("v1"));
    assert_eq!((start, end), (Some(11), None));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn debut_pairs_join_pools_to_operators_first_seen_in_the_same_version(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let v1 = [
        sighting("pool", "p1", None, None),
        sighting("char", "c2", None, None),
        sighting("char", "c1", None, None),
    ];
    release::upsert_sightings(&mut conn, Server::CN, Some("v1"), &v1)
        .await
        .unwrap();
    release::upsert_sightings(
        &mut conn,
        Server::CN,
        Some("v2"),
        &[sighting("char", "c3", None, None)],
    )
    .await
    .unwrap();
    release::upsert_sightings(
        &mut conn,
        Server::CN,
        None,
        &[sighting("pool", "p0", None, None)],
    )
    .await
    .unwrap();
    release::upsert_sightings(&mut conn, Server::EN, Some("v1"), &v1)
        .await
        .unwrap();
    release::upsert_sightings(
        &mut conn,
        Server::EN,
        Some("v1"),
        &[sighting("char", "e1", None, None)],
    )
    .await
    .unwrap();

    let pairs = release::debut_pairs(&mut conn, Server::CN).await.unwrap();
    assert_eq!(
        pairs,
        [
            ("p1".to_owned(), "c1".to_owned()),
            ("p1".to_owned(), "c2".to_owned())
        ]
    );
    assert_eq!(
        release::debut_pairs(&mut conn, Server::EN)
            .await
            .unwrap()
            .len(),
        3
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_override_is_replaced_whole_and_lists_most_recent_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let featured = ["char_1".to_owned(), "char_2".to_owned()];

    let first = release::put_override(
        &mut conn,
        &OverrideInput {
            en_id: Some("en_pool"),
            en_name: Some("Banner"),
            en_start: Some(100),
            en_end: Some(200),
            featured_chars: Some(&featured),
            note: "checked",
            updated_by: Some(user),
            ..manual("pool", "cn_pool")
        },
    )
    .await
    .unwrap();
    assert_eq!(first.featured_chars.as_deref(), Some(&featured[..]));
    assert_eq!(first.en_start, Some(100));

    let replaced = release::put_override(
        &mut conn,
        &OverrideInput {
            source: "auto",
            ..manual("pool", "cn_pool")
        },
    )
    .await
    .unwrap();
    assert_eq!(replaced.en_id, None);
    assert_eq!(replaced.featured_chars, None);
    assert_eq!(
        (replaced.source.as_str(), replaced.note.as_str()),
        ("auto", "")
    );

    release::put_override(&mut conn, &manual("skin", "cn_skin"))
        .await
        .unwrap();
    sqlx::query(
        "UPDATE release_overrides SET updated_at = CASE kind
             WHEN 'pool' THEN timestamptz '2026-01-01' ELSE timestamptz '2026-02-01' END",
    )
    .execute(&mut *conn)
    .await
    .unwrap();

    let kinds: Vec<String> = release::list_overrides(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|o| o.kind)
        .collect();
    assert_eq!(kinds, ["skin", "pool"]);
    let auto = release::overrides_by_source(&mut conn, "auto")
        .await
        .unwrap();
    assert_eq!(auto.len(), 1);
    assert_eq!(auto[0].cn_id, "cn_pool");

    assert!(
        release::delete_override(&mut conn, "pool", "cn_pool")
            .await
            .unwrap()
    );
    assert!(
        !release::delete_override(&mut conn, "pool", "cn_pool")
            .await
            .unwrap()
    );
    assert!(
        release::overrides_by_source(&mut conn, "auto")
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_release_plan_is_one_row_per_user_replaced_on_put(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    assert!(release::get_plan(&mut conn, user).await.unwrap().is_none());

    release::put_plan(&mut conn, user, 50, true, &json!(["a"]), &json!({"s": 1}))
        .await
        .unwrap();
    let put = release::put_plan(&mut conn, user, 70, false, &json!([]), &json!({}))
        .await
        .unwrap();
    assert_eq!((put.initial, put.initial_manual), (70, false));

    let plan = release::get_plan(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(plan.initial, 70);
    assert_eq!(plan.picks, json!([]));
    assert_eq!(plan.stages, json!({}));
    assert_eq!(plan.updated_at, put.updated_at);
    assert!(release::get_plan(&mut conn, other).await.unwrap().is_none());
}
