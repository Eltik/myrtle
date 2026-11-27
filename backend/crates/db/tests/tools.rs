#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::tools::{self, Column, ImportGuard, Scope, Table};
use myrtle_db::{Conn, DbError};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

async fn table(conn: &mut Conn, name: &str) -> Table {
    tools::table(conn, name).await.unwrap().unwrap()
}

/// The live columns of `table` that `row` has a key for
async fn columns_in(conn: &mut Conn, table: &Table, row: &Value) -> Vec<Column> {
    tools::columns(conn, table)
        .await
        .unwrap()
        .into_iter()
        .filter(|c| row.get(c.name()).is_some())
        .collect()
}

/// A table whose name and serial column need quoting
async fn odd_table(conn: &mut Conn) -> Table {
    sqlx::query(r#"CREATE TABLE "Odd ""Name""" ("Id" bigserial PRIMARY KEY, label text NOT NULL)"#)
        .execute(&mut *conn)
        .await
        .unwrap();
    table(conn, r#"Odd "Name""#).await
}

async fn count(conn: &mut Conn, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(conn).await.unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_table_is_found_only_by_its_exact_catalog_name(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let users = table(&mut conn, "users").await;
    assert_eq!(users.name(), "users");
    for unknown in [
        "Users",
        "nope",
        "users; DROP TABLE users",
        "pg_class",
        "_sqlx_migrations ",
    ] {
        assert!(
            tools::table(&mut conn, unknown).await.unwrap().is_none(),
            "{unknown}"
        );
    }
    let estimates = tools::table_estimates(&mut conn).await.unwrap();
    assert!(estimates.iter().any(|e| e.table == users));
    assert!(estimates.iter().all(|e| e.table.name() != "v_user_profile"));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_temp_table_of_the_same_name_does_not_shadow_the_catalog_table(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    common::user(&mut conn, 2).await;
    sqlx::query("CREATE TEMP TABLE users (id uuid, uid text, server_id smallint)")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users SELECT gen_random_uuid(), 'temp', 0 FROM generate_series(1, 5)")
        .execute(&mut *conn)
        .await
        .unwrap();

    let users = table(&mut conn, "users").await;
    assert_eq!(tools::count_rows(&mut conn, &users).await.unwrap(), 2);
    assert_eq!(
        tools::find_user_id(&mut conn, "1", Server::EN)
            .await
            .unwrap(),
        Some(user)
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn columns_come_in_physical_order(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let users = table(&mut conn, "users").await;
    let odd = odd_table(&mut conn).await;

    let names: Vec<String> = tools::columns(&mut conn, &users)
        .await
        .unwrap()
        .iter()
        .map(|c| c.name().to_owned())
        .collect();
    assert_eq!(names[..3], ["id", "uid", "server_id"]);
    let odd_names: Vec<String> = tools::columns(&mut conn, &odd)
        .await
        .unwrap()
        .iter()
        .map(|c| c.name().to_owned())
        .collect();
    assert_eq!(odd_names, ["Id", "label"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn foreign_keys_are_child_to_parent_edges_between_two_tables(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let edges = tools::foreign_keys(&mut conn).await.unwrap();
    let has =
        |child: &str, parent: &str| edges.iter().any(|e| e.child == child && e.parent == parent);
    assert!(has("users", "servers"));
    assert!(has("oauth_tokens", "oauth_grants"));
    assert!(!has("servers", "users"));
    assert!(!has("grids", "grids"));
    let oauth_users = edges
        .iter()
        .filter(|e| e.child == "oauth_grants" && e.parent == "users")
        .count();
    assert_eq!(oauth_users, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_snapshot_cursor_yields_every_row_in_batches_then_closes(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    sqlx::query(
        "INSERT INTO users (uid, server_id) SELECT g::text, 0 FROM generate_series(1, 1001) g",
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    let users = table(&mut conn, "users").await;
    let servers = table(&mut conn, "servers").await;

    let mut tx = tools::begin_snapshot(&mut conn).await.unwrap();
    assert!(!tools::server_version(&mut tx).await.unwrap().is_empty());
    tools::declare_json_rows(&mut tx, &users).await.unwrap();
    let first = tools::fetch_json_rows(&mut tx).await.unwrap();
    let second = tools::fetch_json_rows(&mut tx).await.unwrap();
    assert_eq!((first.len(), second.len()), (1000, 1));
    assert!(tools::fetch_json_rows(&mut tx).await.unwrap().is_empty());
    let row: Value = serde_json::from_str(&second[0]).unwrap();
    assert!(row["uid"].is_string());
    assert_eq!(row["server_id"], 0);

    tools::declare_json_rows(&mut tx, &servers).await.unwrap();
    let servers_rows = tools::fetch_json_rows(&mut tx).await.unwrap();
    assert_eq!(servers_rows.len(), Server::ALL.len());
    let written = sqlx::query("INSERT INTO servers (id, code, name) VALUES (99, 'xx', 'x')")
        .execute(&mut *tx)
        .await;
    assert!(written.is_err(), "the snapshot is read only");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn restored_rows_count_and_their_sequence_moves_past_them(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let odd = odd_table(&mut conn).await;
    let id = tools::columns(&mut conn, &odd).await.unwrap().remove(0);
    let next_id = r#"INSERT INTO "Odd ""Name""" (label) VALUES ('next') RETURNING "Id""#;

    let rows = json!([{"Id": 5, "label": "a"}, {"Id": 9, "label": "b"}]);
    assert_eq!(
        tools::restore_rows(&mut conn, &odd, &rows).await.unwrap(),
        2
    );
    assert_eq!(tools::count_rows(&mut conn, &odd).await.unwrap(), 2);
    tools::reset_sequence(&mut conn, &odd, &id).await.unwrap();
    assert_eq!(count(&mut conn, next_id).await, 10);

    assert_eq!(tools::delete_all(&mut conn, &odd).await.unwrap(), 3);
    tools::reset_sequence(&mut conn, &odd, &id).await.unwrap();
    assert_eq!(count(&mut conn, next_id).await, 1);

    tools::truncate_cascade(&mut conn, std::slice::from_ref(&odd))
        .await
        .unwrap();
    assert_eq!(tools::count_rows(&mut conn, &odd).await.unwrap(), 0);
    assert_eq!(count(&mut conn, next_id).await, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_truncate_cascades_to_the_tables_that_reference_it(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    sqlx::query("INSERT INTO plan_presets (user_id, name, target) VALUES ($1, 'p', '{}')")
        .bind(user.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    let users = table(&mut conn, "users").await;

    tools::truncate_cascade(&mut conn, &[users]).await.unwrap();
    assert_eq!(
        count(&mut conn, "SELECT COUNT(*) FROM plan_presets").await,
        0
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn suspended_triggers_let_a_restore_skip_audit_rows_and_foreign_keys(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let users = table(&mut conn, "users").await;
    let audited = count(&mut conn, "SELECT COUNT(*) FROM audit_log").await;
    let row = json!([{
        "id": Uuid::new_v4(),
        "uid": "orphan",
        "server_id": 99,
        "role": "user",
        "created_at": "2026-01-02T03:04:05Z",
        "updated_at": "2026-01-02T03:04:05Z"
    }]);

    let refused = tools::restore_rows(&mut conn, &users, &row).await;
    assert!(matches!(refused, Err(DbError::ForeignKey { .. })));

    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    tools::suspend_triggers(&mut tx).await.unwrap();
    assert_eq!(tools::restore_rows(&mut tx, &users, &row).await.unwrap(), 1);
    assert_eq!(
        count(&mut tx, "SELECT COUNT(*) FROM audit_log").await,
        audited
    );
    tx.rollback().await.unwrap();
    assert_eq!(count(&mut conn, "SELECT COUNT(*) FROM users").await, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn accounts_resolve_by_id_by_uid_oldest_first_and_by_natural_key(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let en = common::user(&mut conn, 5).await;
    let jp = common::user_on(&mut conn, 5, Server::JP).await;
    common::user(&mut conn, 6).await;

    let account = tools::find_account(&mut conn, jp).await.unwrap().unwrap();
    assert_eq!(account.id, jp.0);
    assert_eq!(account.uid, "5");
    assert_eq!(account.server_id, Server::JP.index());
    assert_eq!(account.server_code, "jp");
    assert_eq!(account.nickname.as_deref(), Some("user5"));
    assert!(
        tools::find_account(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );

    let by_uid: Vec<Uuid> = tools::find_accounts_by_uid(&mut conn, "5")
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(by_uid, [en.0, jp.0]);

    assert_eq!(
        tools::find_user_id(&mut conn, "5", Server::JP)
            .await
            .unwrap(),
        Some(jp)
    );
    assert_eq!(
        tools::find_user_id(&mut conn, "5", Server::KR)
            .await
            .unwrap(),
        None
    );
    assert!(tools::user_id_taken(&mut conn, en).await.unwrap());
    assert!(
        !tools::user_id_taken(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
    );

    let row: Value = serde_json::from_str(&tools::user_json(&mut conn, en).await.unwrap()).unwrap();
    assert_eq!(row["id"], json!(en.0));
    assert_eq!(row["uid"], "5");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn scoped_rows_export_count_and_delete_one_accounts_rows_only(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    for (owner, name) in [(user, "b"), (user, "a"), (other, "c")] {
        sqlx::query(
            "WITH g AS (INSERT INTO plan_groups (user_id, name) VALUES ($1, $2) RETURNING id),
                  p AS (INSERT INTO operator_plans (user_id, operator_id) VALUES ($1, $2) RETURNING id)
             INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
             SELECT g.id, p.id FROM g, p",
        )
        .bind(owner.0)
        .bind(name)
        .execute(&mut *conn)
        .await
        .unwrap();
    }
    let groups = table(&mut conn, "plan_groups").await;
    let members = table(&mut conn, "plan_group_members").await;

    let text = tools::scoped_json(&mut conn, &groups, Scope::Owner, user)
        .await
        .unwrap();
    let exported: Vec<Value> = serde_json::from_str(&text).unwrap();
    let mut names: Vec<&str> = exported
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    let ids: Vec<&str> = exported.iter().map(|r| r["id"].as_str().unwrap()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(
        ids, sorted,
        "rows come in text order, which starts with the id"
    );
    names.sort_unstable();
    assert_eq!(names, ["a", "b"]);
    let none = tools::scoped_json(&mut conn, &groups, Scope::Owner, UserId(Uuid::new_v4()))
        .await
        .unwrap();
    assert_eq!(none, "[]");

    assert_eq!(
        tools::count_scoped(&mut conn, &members, Scope::PlanGroup, user)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        tools::delete_scoped(&mut conn, &members, Scope::PlanGroup, user)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        tools::count_scoped(&mut conn, &members, Scope::PlanGroup, other)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        tools::delete_scoped(&mut conn, &groups, Scope::Owner, user)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        tools::count_scoped(&mut conn, &groups, Scope::Owner, other)
            .await
            .unwrap(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_account_upsert_keeps_identity_and_creation_and_refreshes_the_rest(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let existing = common::user(&mut conn, 1).await;
    let users = table(&mut conn, "users").await;
    let created = tools::find_account(&mut conn, existing)
        .await
        .unwrap()
        .unwrap();
    let before: Value =
        serde_json::from_str(&tools::user_json(&mut conn, existing).await.unwrap()).unwrap();

    let row = json!({
        "id": Uuid::new_v4(),
        "uid": "1",
        "server_id": Server::EN.index(),
        "nickname": "renamed",
        "created_at": "2000-01-01T00:00:00Z"
    });
    let columns = columns_in(&mut conn, &users, &row).await;
    let id = tools::upsert_user_json(&mut conn, &columns, &row)
        .await
        .unwrap();
    assert_eq!(id, existing);
    let after: Value =
        serde_json::from_str(&tools::user_json(&mut conn, existing).await.unwrap()).unwrap();
    assert_eq!(after["nickname"], "renamed");
    assert_eq!(after["created_at"], before["created_at"]);
    assert_eq!(created.nickname.as_deref(), Some("user1"));

    let fresh_id = Uuid::new_v4();
    let fresh = json!({"id": fresh_id, "uid": "2", "server_id": Server::EN.index()});
    let columns = columns_in(&mut conn, &users, &fresh).await;
    let id = tools::upsert_user_json(&mut conn, &columns, &fresh)
        .await
        .unwrap();
    assert_eq!(id, UserId(fresh_id));
    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
        .bind(fresh_id)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(role, "user");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_account_upsert_with_nothing_to_refresh_returns_the_existing_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let existing = common::user(&mut conn, 1).await;
    let users = table(&mut conn, "users").await;
    let before = tools::user_json(&mut conn, existing).await.unwrap();

    let row = json!({"id": Uuid::new_v4(), "uid": "1", "server_id": Server::EN.index()});
    let columns = columns_in(&mut conn, &users, &row).await;
    let id = tools::upsert_user_json(&mut conn, &columns, &row)
        .await
        .unwrap();
    assert_eq!(id, existing);
    assert_eq!(tools::user_json(&mut conn, existing).await.unwrap(), before);

    let fresh_id = Uuid::new_v4();
    let fresh = json!({"id": fresh_id, "uid": "2", "server_id": Server::EN.index()});
    let id = tools::upsert_user_json(&mut conn, &columns, &fresh)
        .await
        .unwrap();
    assert_eq!(id, UserId(fresh_id));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_account_upsert_with_nothing_to_refresh_returns_a_concurrently_committed_row(
    pool: PgPool,
) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let users = table(&mut conn, "users").await;
    let row = json!({"id": Uuid::new_v4(), "uid": "1", "server_id": Server::EN.index()});
    let columns = columns_in(&mut conn, &users, &row).await;

    let mut first = db.begin().await.unwrap();
    let winner = common::user(&mut first, 1).await;
    let (id, ()) = tokio::join!(tools::upsert_user_json(&mut conn, &columns, &row), async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        first.commit().await.unwrap();
    });
    assert_eq!(id.unwrap(), winner);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_truncate_of_no_tables_does_nothing(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    common::user(&mut conn, 1).await;

    tools::truncate_cascade(&mut conn, &[]).await.unwrap();
    assert_eq!(count(&mut conn, "SELECT COUNT(*) FROM users").await, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_account_insert_defaults_missing_columns_and_skips_rows_its_guard_rejects(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let list: Uuid =
        sqlx::query_scalar("INSERT INTO tier_lists (name, slug) VALUES ('l', 'l') RETURNING id")
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    let favorites = table(&mut conn, "tier_list_favorites").await;
    let rows = json!([
        {"tier_list_id": list, "user_id": user.0},
        {"tier_list_id": Uuid::new_v4(), "user_id": user.0}
    ]);
    let columns = columns_in(&mut conn, &favorites, &rows[0]).await;

    let inserted = tools::insert_rows(
        &mut conn,
        &favorites,
        &columns,
        Some(ImportGuard::MissingTierList),
        &rows,
    )
    .await
    .unwrap();
    assert_eq!(inserted, 1);
    let stamped = count(
        &mut conn,
        "SELECT COUNT(*) FROM tier_list_favorites WHERE favorited_at IS NOT NULL",
    )
    .await;
    assert_eq!(stamped, 1);

    let unguarded = tools::insert_rows(&mut conn, &favorites, &columns, None, &rows).await;
    assert!(matches!(
        unguarded,
        Err(DbError::Conflict { .. } | DbError::ForeignKey { .. })
    ));
}
