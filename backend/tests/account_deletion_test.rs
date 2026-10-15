//! Self-serve account deletion (`POST /auth/delete-account`) leaves no row that
//! names the account, and keeps the shared rows it only touched.
//!
//! "Names the account" is read off the catalog, not a hand list: every column
//! with a foreign key to `users`, plus every uuid column called `user_id`,
//! `created_by`, `changed_by`, `updated_by`, `granted_by` or `published_by`
//! (the audit and override columns carry no key). A table added later is
//! swept without touching this file. On top of that, `audit_log` must hold no
//! row keyed on the account, since its trigger copies the whole `users` and
//! `user_scores` rows as they are deleted.
//!
//! Both tests run inside one transaction on the local Postgres and roll it
//! back, so nothing persists. Ignored by default:
//! `cargo test --test account_deletion_test -- --ignored --nocapture`

use backend::database::queries::account_deletion::delete_account;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

/// Every (table, column) that can hold a `users.id`, base tables only.
async fn user_columns(conn: &mut PgConnection) -> Vec<(String, String)> {
    sqlx::query_as(
        r"
        SELECT c.conrelid::regclass::text, a.attname::text
        FROM pg_constraint c
        JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY (c.conkey)
        WHERE c.contype = 'f' AND c.confrelid = 'users'::regclass
        UNION
        SELECT col.table_name::text, col.column_name::text
        FROM information_schema.columns col
        JOIN information_schema.tables t
          ON t.table_schema = col.table_schema AND t.table_name = col.table_name
        WHERE col.table_schema = 'public' AND t.table_type = 'BASE TABLE'
          AND col.data_type = 'uuid'
          AND col.column_name IN
              ('user_id', 'created_by', 'changed_by', 'updated_by', 'granted_by', 'published_by')
        UNION
        SELECT 'users', 'id'
        ORDER BY 1, 2
        ",
    )
    .fetch_all(conn)
    .await
    .expect("catalog")
}

/// Rows naming `id` in each user column, plus the audit rows keyed on it.
async fn census(conn: &mut PgConnection, id: Uuid) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for (table, column) in user_columns(conn).await {
        // Identifiers come from the catalog above, never from input.
        let n: i64 = sqlx::query_scalar(&format!(
            r#"SELECT COUNT(*) FROM "{table}" WHERE "{column}" = $1"#
        ))
        .bind(id)
        .fetch_one(&mut *conn)
        .await
        .expect("count");
        out.push((format!("{table}.{column}"), n));
    }
    let audit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log
         WHERE table_name IN ('users', 'user_scores') AND record_id = $1::text",
    )
    .bind(id)
    .fetch_one(&mut *conn)
    .await
    .expect("audit count");
    out.push(("audit_log.record_id".to_owned(), audit));
    out
}

fn print_census(label: &str, before: &[(String, i64)], after: &[(String, i64)]) {
    println!("{label}");
    for ((name, b), (_, a)) in before.iter().zip(after) {
        println!("  {name:<44} {b:>7} -> {a}");
    }
    let total_before: i64 = before.iter().map(|(_, n)| n).sum();
    let total_after: i64 = after.iter().map(|(_, n)| n).sum();
    println!("  {:<44} {total_before:>7} -> {total_after}", "TOTAL");
}

async fn new_user(conn: &mut PgConnection, tag: &str) -> Uuid {
    let uid = format!("del-{tag}-{}", &Uuid::new_v4().simple().to_string()[..8]);
    sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
        .bind(uid)
        .fetch_one(conn)
        .await
        .expect("insert user")
}

async fn exec(conn: &mut PgConnection, sql: &str, binds: &[Uuid]) {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.execute(conn)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"));
}

async fn scalar_i64(conn: &mut PgConnection, sql: &str, binds: &[Uuid]) -> i64 {
    let mut q = sqlx::query_scalar::<_, i64>(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.fetch_one(conn)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"))
}

async fn insert_id(conn: &mut PgConnection, sql: &str, binds: &[Uuid]) -> Uuid {
    let mut q = sqlx::query_scalar::<_, Uuid>(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.fetch_one(conn)
        .await
        .unwrap_or_else(|e| panic!("{sql}: {e}"))
}

/// A user made in the test, with a row behind every hand-done branch of the
/// deletion, next to a second user whose rows must survive.
#[tokio::test]
#[ignore = "needs the local Postgres"]
async fn a_created_user_leaves_no_row_and_shared_rows_survive() {
    let pool = PgPool::connect(DATABASE_URL).await.expect("connect");
    let mut tx = pool.begin().await.expect("tx");
    let c: &mut PgConnection = &mut tx;

    let me = new_user(c, "me").await;
    let other = new_user(c, "other").await;

    // Account rows; the score insert and the nickname update fire the audit
    // trigger, so `audit_log` holds the old rows too.
    exec(c, "INSERT INTO user_settings (user_id) VALUES ($1)", &[me]).await;
    exec(c, "INSERT INTO user_scores (user_id) VALUES ($1)", &[me]).await;
    exec(
        c,
        "UPDATE users SET nickname = 'Deleted Doctor' WHERE id = $1",
        &[me],
    )
    .await;
    let plan = insert_id(c, "INSERT INTO operator_plans (user_id, operator_id) VALUES ($1, 'char_002_amiya') RETURNING id", &[me]).await;
    let group = insert_id(
        c,
        "INSERT INTO plan_groups (user_id, name) VALUES ($1, 'g') RETURNING id",
        &[me],
    )
    .await;
    exec(
        c,
        "INSERT INTO plan_group_members (plan_group_id, operator_plan_id) VALUES ($1, $2)",
        &[group, plan],
    )
    .await;

    // Tier lists: my community list (deleted, with its tier, placement and
    // the other user's favourite), an official list I created (kept, author
    // removed), and the other user's list where I granted them a permission
    // (kept, grantor removed).
    let community = insert_id(c, "INSERT INTO tier_lists (name, slug, list_type, created_by) VALUES ('mine', 'deltest-' || gen_random_uuid(), 'community', $1) RETURNING id", &[me]).await;
    let tier = insert_id(
        c,
        "INSERT INTO tiers (tier_list_id, name, display_order) VALUES ($1, 'S', 0) RETURNING id",
        &[community],
    )
    .await;
    exec(
        c,
        "INSERT INTO tier_placements (tier_id, entity_id) VALUES ($1, 'char_002_amiya')",
        &[tier],
    )
    .await;
    exec(
        c,
        "INSERT INTO tier_list_favorites (tier_list_id, user_id) VALUES ($1, $2)",
        &[community, other],
    )
    .await;
    let official = insert_id(c, "INSERT INTO tier_lists (name, slug, list_type, created_by) VALUES ('site', 'deltest-' || gen_random_uuid(), 'official', $1) RETURNING id", &[me]).await;
    let theirs = insert_id(c, "INSERT INTO tier_lists (name, slug, list_type, created_by) VALUES ('theirs', 'deltest-' || gen_random_uuid(), 'community', $1) RETURNING id", &[other]).await;
    exec(c, "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by) VALUES ($1, $2, 'edit', $3)", &[theirs, other, me]).await;
    exec(c, "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by) VALUES ($1, $2, 'edit', $2)", &[official, me]).await;

    // Grids: mine is deleted, the other user's fork of it survives.
    let grid = insert_id(c, "INSERT INTO grids (slug, title, created_by, rows, cols, cells) VALUES ('deltest-' || gen_random_uuid(), 'g', $1, 1, 1, '[]') RETURNING id", &[me]).await;
    let fork = insert_id(c, "INSERT INTO grids (slug, title, created_by, rows, cols, cells, template_of) VALUES ('deltest-' || gen_random_uuid(), 'f', $1, 1, 1, '[]', $2) RETURNING id", &[other, grid]).await;

    // Shared rows I touched: an operator note's audit trail, the generic audit
    // log's actor, a release override.
    let note = insert_id(
        c,
        "INSERT INTO operator_notes (operator_id) VALUES ('char_deltest') RETURNING id",
        &[],
    )
    .await;
    exec(c, "INSERT INTO operator_notes_audit_log (note_id, field_name, changed_by) VALUES ($1, 'pros', $2)", &[note, me]).await;
    let audit_row: i64 = sqlx::query_scalar("INSERT INTO audit_log (table_name, record_id, action, changed_by) VALUES ('tier_lists', 'x', 'UPDATE', $1) RETURNING id")
        .bind(me)
        .fetch_one(&mut *c)
        .await
        .expect("audit row");
    exec(
        c,
        "INSERT INTO release_overrides (kind, cn_id, updated_by) VALUES ('deltest', 'deltest', $1)",
        &[me],
    )
    .await;

    let other_before = census(c, other).await;
    let before = census(c, me).await;
    let deleted = delete_account(c, me)
        .await
        .expect("delete")
        .expect("account existed");
    let after = census(c, me).await;
    print_census("created user", &before, &after);

    for (name, n) in &after {
        assert_eq!(*n, 0, "{name} still names the deleted account");
    }
    assert!(
        before.iter().map(|(_, n)| n).sum::<i64>() >= 15,
        "the fixture reached the tables it means to"
    );
    assert_eq!(
        deleted.tier_list_slugs.len(),
        1,
        "only the community list is deleted"
    );

    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM tier_lists WHERE id = $1",
            &[community]
        )
        .await,
        0
    );
    assert_eq!(
        scalar_i64(c, "SELECT COUNT(*) FROM tiers WHERE id = $1", &[tier]).await,
        0
    );
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM tier_placements WHERE tier_id = $1",
            &[tier]
        )
        .await,
        0
    );
    assert_eq!(scalar_i64(c, "SELECT COUNT(*) FROM audit_log WHERE table_name = 'tier_lists' AND record_id = $1::text", &[community]).await, 0);
    assert_eq!(scalar_i64(c, "SELECT COUNT(*) FROM audit_log WHERE table_name = 'tier_placements' AND COALESCE(new_data, old_data)->>'tier_id' = $1::text", &[tier]).await, 0);
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM tier_lists WHERE id = $1 AND created_by IS NULL",
            &[official]
        )
        .await,
        1
    );
    assert_eq!(scalar_i64(c, "SELECT COUNT(*) FROM tier_list_permissions WHERE tier_list_id = $1 AND user_id = $2 AND granted_by IS NULL", &[theirs, other]).await, 1);
    assert_eq!(
        scalar_i64(c, "SELECT COUNT(*) FROM grids WHERE id = $1", &[grid]).await,
        0
    );
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM grids WHERE id = $1 AND template_of IS NULL",
            &[fork]
        )
        .await,
        1
    );
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM operator_notes WHERE id = $1",
            &[note]
        )
        .await,
        1
    );
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM operator_notes_audit_log WHERE note_id = $1",
            &[note]
        )
        .await,
        0
    );
    let audit_kept: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE id = $1 AND changed_by IS NULL")
            .bind(audit_row)
            .fetch_one(&mut *c)
            .await
            .expect("audit kept");
    assert_eq!(audit_kept, 1);
    assert_eq!(
        scalar_i64(
            c,
            "SELECT COUNT(*) FROM release_overrides WHERE kind = 'deltest' AND updated_by IS NULL",
            &[]
        )
        .await,
        1
    );

    // The other user loses only the favourite on my deleted list.
    let other_after = census(c, other).await;
    for ((name, b), (_, a)) in other_before.iter().zip(&other_after) {
        let expected = if name == "tier_list_favorites.user_id" {
            b - 1
        } else {
            *b
        };
        assert_eq!(*a, expected, "the other user's {name} moved");
    }

    tx.rollback().await.expect("rollback");
}

/// The real account with the most rows naming it, deleted inside a
/// transaction that is rolled back. Prints the census before and after.
#[tokio::test]
#[ignore = "needs the local Postgres with real roster data"]
async fn the_largest_real_account_leaves_no_row() {
    let pool = PgPool::connect(DATABASE_URL).await.expect("connect");
    let mut tx = pool.begin().await.expect("tx");
    let c: &mut PgConnection = &mut tx;

    // Most roster rows, ties broken by id so the pick is stable; the account
    // that also authored tier lists is preferred, as it reaches more tables.
    let subject: Uuid = sqlx::query_scalar(
        r"
        SELECT u.id FROM users u
        LEFT JOIN (SELECT created_by, COUNT(*) n FROM tier_lists GROUP BY 1) t ON t.created_by = u.id
        LEFT JOIN (SELECT user_id, COUNT(*) n FROM user_operators GROUP BY 1) o ON o.user_id = u.id
        ORDER BY COALESCE(t.n, 0) > 0 DESC, COALESCE(o.n, 0) DESC, u.id
        LIMIT 1
        ",
    )
    .fetch_one(&mut *c)
    .await
    .expect("subject");

    let users_before = scalar_i64(c, "SELECT COUNT(*) FROM users", &[]).await;
    let before = census(c, subject).await;
    let started = std::time::Instant::now();
    let deleted = delete_account(c, subject)
        .await
        .expect("delete")
        .expect("account existed");
    let took = started.elapsed();
    let after = census(c, subject).await;
    let users_after = scalar_i64(c, "SELECT COUNT(*) FROM users", &[]).await;
    print_census("largest real account", &before, &after);
    println!(
        "  users {users_before} -> {users_after}, community tier lists deleted {}, delete took {took:?}",
        deleted.tier_list_slugs.len()
    );

    for (name, n) in &after {
        assert_eq!(*n, 0, "{name} still names the deleted account");
    }
    assert_eq!(users_after, users_before - 1, "exactly one account goes");

    tx.rollback().await.expect("rollback");
}
