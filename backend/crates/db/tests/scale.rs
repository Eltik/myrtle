//! Per-request reads stay off whole-table scans: each gate reads the transaction's row counters
//! around a call on a seeded table, then drops the index and shows the same call scanning it.

#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::content_audit::{self, ContentAuditFilter};
use myrtle_db::repos::grids::{self, ListOrder};
use myrtle_db::repos::users::{self, AdminFilter};
use myrtle_db::repos::{gacha, i18n, operator_notes, planner, scores, tier_lists};
use myrtle_types::{ContentTargetKind, Server, UserId};
use sqlx::{PgConnection, PgPool};
use tokio::sync::Semaphore;
use uuid::Uuid;

const N: i64 = 10_000;

/// The sqlx test pools share 20 connections; a gate holds one for seconds
static SLOTS: Semaphore = Semaphore::const_new(8);

const USERS: &str = "ALTER TABLE users DISABLE TRIGGER trg_audit_users;
    INSERT INTO users (uid, server_id, nickname, role, created_at)
    SELECT g::text, (g % 6)::smallint, 'player ' || translate(g::text, '0123456789', 'abcdefghij'),
           CASE WHEN g <= 10 THEN 'super_admin' ELSE 'user' END, now() - g * interval '1 minute'
    FROM generate_series(1, 10000) g;
    ALTER TABLE users ENABLE TRIGGER trg_audit_users;";

const TIER_LISTS: &str = "INSERT INTO tier_lists (name, slug, list_type, created_by)
    SELECT 'List ' || g, 'list-' || g, 'community', u.id
    FROM generate_series(1, 10000) g JOIN users u ON u.uid = (1 + g % 2000)::text;";

/// `[rows read sequentially, index entries returned, index pages read]` from `table` in this
/// transaction so far
async fn reads(conn: &mut PgConnection, table: &str) -> [i64; 3] {
    let read: (i64, i64, i64) = sqlx::query_as(
        "SELECT pg_stat_get_xact_tuples_returned(c.oid),
                COALESCE(sum(pg_stat_get_xact_tuples_returned(i.indexrelid)), 0)::bigint,
                COALESCE(sum(pg_stat_get_xact_blocks_fetched(i.indexrelid)), 0)::bigint
         FROM pg_class c LEFT JOIN pg_index i ON i.indrelid = c.oid
         WHERE c.oid = $1::text::regclass
         GROUP BY c.oid",
    )
    .bind(table)
    .fetch_one(conn)
    .await
    .unwrap();
    read.into()
}

const fn since(after: [i64; 3], before: [i64; 3]) -> [i64; 3] {
    [
        after[0] - before[0],
        after[1] - before[1],
        after[2] - before[2],
    ]
}

fn bounded(table: &str, [seq, idx, _]: [i64; 3], max: i64) {
    assert_eq!(seq, 0, "{table} read sequentially");
    assert!(
        idx <= max,
        "{table}: {idx} index entries, expected at most {max}"
    );
}

fn scanned(table: &str, [seq, idx, _]: [i64; 3]) {
    assert!(
        seq + idx >= N / 2,
        "{table}: {seq} + {idx} rows, the gate missed the lost index"
    );
}

async fn seed(conn: &mut PgConnection, sqls: &[&'static str]) {
    for sql in sqls {
        sqlx::raw_sql(*sql).execute(&mut *conn).await.unwrap();
    }
    sqlx::raw_sql(
        "ANALYZE;
         SELECT gin_clean_pending_list(i.indexrelid) FROM pg_index i
         JOIN pg_class c ON c.oid = i.indexrelid JOIN pg_am a ON a.oid = c.relam
         WHERE a.amname = 'gin';",
    )
    .execute(conn)
    .await
    .unwrap(); // a trigram index still pending looks too costly to the planner
}

async fn user(conn: &mut PgConnection, uid: &str) -> UserId {
    UserId(
        sqlx::query_scalar("SELECT id FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_one(conn)
            .await
            .unwrap(),
    )
}

async fn drop_indexes(conn: &mut PgConnection, ddl: &'static str) {
    sqlx::raw_sql(ddl).execute(conn).await.unwrap();
}

/// Reads `table` makes the delete of the account with uid `7`, before and after `ddl`
async fn delete_reads(
    pool: PgPool,
    data: &'static str,
    table: &str,
    ddl: &'static str,
) -> [[i64; 3]; 2] {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS, data]).await;
    let mut out = [[0; 3]; 2];
    for (i, drop) in [None, Some(ddl)].into_iter().enumerate() {
        let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
        if let Some(ddl) = drop {
            drop_indexes(&mut tx, ddl).await;
        }
        let account = user(&mut tx, "7").await;
        let before = reads(&mut tx, table).await;
        users::delete_account(&mut tx, account)
            .await
            .unwrap()
            .unwrap();
        out[i] = since(reads(&mut tx, table).await, before);
    }
    out
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_gacha_history_page_reads_only_the_players_pulls(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[
            USERS,
            "INSERT INTO gacha_records (user_id, char_id, pool_id, rarity, pull_timestamp)
         SELECT u.id, 'char_' || (g % 300), 'pool_' || (g % 40), (3 + g % 4)::smallint,
                1600000000000 + g * 1000
         FROM users u CROSS JOIN generate_series(1, 10) g WHERE u.uid::int <= 1500;",
        ],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let player = user(&mut tx, "1").await;
    let before = reads(&mut tx, "gacha_records").await;
    assert_eq!(
        gacha::history(&mut tx, player, None, 50, 0)
            .await
            .unwrap()
            .len(),
        10
    );
    bounded(
        "gacha_records",
        since(reads(&mut tx, "gacha_records").await, before),
        100,
    );

    drop_indexes(
        &mut tx,
        "DROP INDEX idx_gacha_user_time; DROP INDEX idx_gacha_records_user_id;
         DROP INDEX idx_gacha_user_rarity;
         ALTER TABLE gacha_records DROP CONSTRAINT gacha_records_batch_unique;",
    )
    .await;
    let before = reads(&mut tx, "gacha_records").await;
    gacha::history(&mut tx, player, None, 50, 0).await.unwrap();
    scanned(
        "gacha_records",
        since(reads(&mut tx, "gacha_records").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_lookup_by_uid_reads_one_account(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let before = reads(&mut tx, "users").await;
    users::find(&mut tx, "1234", Server::Bilibili)
        .await
        .unwrap()
        .unwrap();
    users::profile_by_uid(&mut tx, "1234", None)
        .await
        .unwrap()
        .unwrap();
    bounded("users", since(reads(&mut tx, "users").await, before), 10);

    drop_indexes(
        &mut tx,
        "ALTER TABLE users DROP CONSTRAINT users_uid_server_id_key; DROP INDEX idx_users_uid_trgm;",
    )
    .await; // the trigram index answers `uid =` too
    let before = reads(&mut tx, "users").await;
    users::profile_by_uid(&mut tx, "1234", None)
        .await
        .unwrap()
        .unwrap();
    scanned("users", since(reads(&mut tx, "users").await, before));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_tier_list_by_slug_reads_one_list(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS, TIER_LISTS]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let before = reads(&mut tx, "tier_lists").await;
    tier_lists::find_by_slug(&mut tx, "list-77")
        .await
        .unwrap()
        .unwrap();
    bounded(
        "tier_lists",
        since(reads(&mut tx, "tier_lists").await, before),
        5,
    );

    drop_indexes(
        &mut tx,
        "ALTER TABLE tier_lists DROP CONSTRAINT tier_lists_slug_key;",
    )
    .await;
    let before = reads(&mut tx, "tier_lists").await;
    tier_lists::find_by_slug(&mut tx, "list-77")
        .await
        .unwrap()
        .unwrap();
    scanned(
        "tier_lists",
        since(reads(&mut tx, "tier_lists").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_users_tier_lists_and_their_count_read_only_their_lists(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS, TIER_LISTS]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let author = user(&mut tx, "8").await;
    let before = reads(&mut tx, "tier_lists").await;
    assert_eq!(
        tier_lists::find_by_user(&mut tx, author)
            .await
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        tier_lists::count_by_user(&mut tx, author, None)
            .await
            .unwrap(),
        5
    );
    bounded(
        "tier_lists",
        since(reads(&mut tx, "tier_lists").await, before),
        50,
    );

    drop_indexes(&mut tx, "DROP INDEX idx_tier_lists_created_by;").await;
    let before = reads(&mut tx, "tier_lists").await;
    tier_lists::find_by_user(&mut tx, author).await.unwrap();
    scanned(
        "tier_lists",
        since(reads(&mut tx, "tier_lists").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_lists_granted_to_a_user_read_only_their_grants(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[
            USERS,
            TIER_LISTS,
            "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         SELECT t.id, u.id, 'edit', NULL
         FROM generate_series(1, 10000) g
         JOIN tier_lists t ON t.slug = 'list-' || g
         JOIN users u ON u.uid = (2 + g % 5000)::text;",
        ],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let grantee = user(&mut tx, "9").await;
    let before = reads(&mut tx, "tier_list_permissions").await;
    assert_eq!(
        tier_lists::get_granted_lists(&mut tx, grantee)
            .await
            .unwrap()
            .len(),
        2
    );
    bounded(
        "tier_list_permissions",
        since(reads(&mut tx, "tier_list_permissions").await, before),
        10,
    );

    drop_indexes(&mut tx, "DROP INDEX idx_tl_permissions_user;").await;
    let before = reads(&mut tx, "tier_list_permissions").await;
    tier_lists::get_granted_lists(&mut tx, grantee)
        .await
        .unwrap();
    scanned(
        "tier_list_permissions",
        since(reads(&mut tx, "tier_list_permissions").await, before),
    );
}

const PLANS: &str = "INSERT INTO operator_plans (user_id, operator_id)
    SELECT u.id, 'char_' || k FROM users u CROSS JOIN generate_series(1, 20) k WHERE u.uid::int <= 500;
    INSERT INTO plan_groups (user_id, name)
    SELECT u.id, 'group ' || k FROM users u CROSS JOIN generate_series(0, 1) k WHERE u.uid::int <= 500;
    INSERT INTO plan_group_members (plan_group_id, operator_plan_id)
    SELECT g.id, p.id FROM operator_plans p JOIN plan_groups g ON g.user_id = p.user_id
    WHERE g.name = 'group 0' OR right(p.operator_id, 1) IN ('0', '2', '4', '6', '8');";

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_plans_groups_and_a_users_memberships_read_only_their_members(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS, PLANS]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let owner = user(&mut tx, "5").await;
    let plan: Uuid = sqlx::query_scalar(
        "SELECT id FROM operator_plans WHERE user_id = $1 AND operator_id = 'char_2'",
    )
    .bind(owner.0)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    let before = reads(&mut tx, "plan_group_members").await;
    assert_eq!(
        planner::group_names(&mut tx, plan).await.unwrap(),
        ["group 0", "group 1"]
    );
    assert_eq!(
        planner::list_memberships(&mut tx, owner)
            .await
            .unwrap()
            .len(),
        30
    );
    bounded(
        "plan_group_members",
        since(reads(&mut tx, "plan_group_members").await, before),
        100,
    );

    drop_indexes(&mut tx, "DROP INDEX idx_plan_group_members_plan;").await;
    let before = reads(&mut tx, "plan_group_members").await;
    planner::group_names(&mut tx, plan).await.unwrap();
    scanned(
        "plan_group_members",
        since(reads(&mut tx, "plan_group_members").await, before),
    );

    drop_indexes(
        &mut tx,
        "ALTER TABLE plan_group_members DROP CONSTRAINT plan_group_members_pkey;",
    )
    .await;
    let before = reads(&mut tx, "plan_group_members").await;
    planner::list_memberships(&mut tx, owner).await.unwrap();
    scanned(
        "plan_group_members",
        since(reads(&mut tx, "plan_group_members").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_content_audit_feed_reads_one_page_per_filter(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[USERS, "INSERT INTO content_audit_log (actor_id, target_kind, target_id, target_name, action, changed_at)
         SELECT u.id, CASE WHEN g % 3 = 0 THEN 'grid' ELSE 'tier_list' END,
                md5((g % 500)::text)::uuid, 'name', 'edit', now() - g * interval '1 minute'
         FROM generate_series(1, 10000) g JOIN users u ON u.uid = (1 + g % 3000)::text;"],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let actor = user(&mut tx, "11").await;
    let target: Uuid = sqlx::query_scalar("SELECT md5('1')::uuid")
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    let filters = [
        (
            ContentAuditFilter {
                actor: None,
                target_kind: None,
                target_id: None,
            },
            "DROP INDEX idx_content_audit_changed_at;",
        ),
        (
            ContentAuditFilter {
                actor: Some(actor),
                target_kind: None,
                target_id: None,
            },
            "DROP INDEX idx_content_audit_actor;",
        ),
        (
            ContentAuditFilter {
                actor: None,
                target_kind: Some(ContentTargetKind::TierList),
                target_id: Some(target),
            },
            "DROP INDEX idx_content_audit_target;",
        ),
    ];
    for (filter, _) in &filters {
        let before = reads(&mut tx, "content_audit_log").await;
        assert!(
            !content_audit::list(&mut tx, *filter, 20, None)
                .await
                .unwrap()
                .is_empty()
        );
        bounded(
            "content_audit_log",
            since(reads(&mut tx, "content_audit_log").await, before),
            100,
        );
    }
    for (filter, ddl) in filters {
        let mut sp = sqlx::Connection::begin(&mut *tx).await.unwrap();
        drop_indexes(&mut sp, ddl).await;
        let before = reads(&mut sp, "content_audit_log").await;
        content_audit::list(&mut sp, filter, 20, None)
            .await
            .unwrap();
        scanned(
            "content_audit_log",
            since(reads(&mut sp, "content_audit_log").await, before),
        );
    }
}

const NOTES_AUDIT: &str =
    "INSERT INTO operator_notes (operator_id) SELECT 'char_' || n FROM generate_series(1, 300) n;
    INSERT INTO operator_notes_audit_log (note_id, field_name, changed_by, changed_at)
    SELECT n.id, 'pros', u.id, now() - g * interval '1 minute'
    FROM generate_series(1, 10000) g
    JOIN operator_notes n ON n.operator_id = 'char_' || (1 + g % 300)
    JOIN users u ON u.uid = (1 + g % 2000)::text;";

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_notes_audit_feed_reads_one_page_whole_or_by_actor(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS, NOTES_AUDIT]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let actor = user(&mut tx, "12").await;
    for (who, ddl) in [
        (None, "DROP INDEX idx_notes_audit_changed_at;"),
        (Some(actor), "DROP INDEX idx_notes_audit_changed_by_at;"),
    ] {
        let before = reads(&mut tx, "operator_notes_audit_log").await;
        assert!(
            !operator_notes::get_audit_log_global(&mut tx, 20, None, who)
                .await
                .unwrap()
                .is_empty()
        );
        bounded(
            "operator_notes_audit_log",
            since(reads(&mut tx, "operator_notes_audit_log").await, before),
            40,
        );

        let mut sp = sqlx::Connection::begin(&mut *tx).await.unwrap();
        drop_indexes(&mut sp, ddl).await;
        let before = reads(&mut sp, "operator_notes_audit_log").await;
        operator_notes::get_audit_log_global(&mut sp, 20, None, who)
            .await
            .unwrap();
        scanned(
            "operator_notes_audit_log",
            since(reads(&mut sp, "operator_notes_audit_log").await, before),
        );
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_translation_audit_feed_by_actor_reads_only_their_rows(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[
            USERS,
            "INSERT INTO ui_message_audit_log (message_key, locale, changed_by, changed_at)
         SELECT 'key' || (g % 4000), (ARRAY['ja', 'ko', 'zh', 'fr'])[1 + g % 4], u.id,
                now() - g * interval '1 minute'
         FROM generate_series(1, 10000) g JOIN users u ON u.uid = (1 + g % 2000)::text;",
        ],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let actor = user(&mut tx, "13").await;
    let before = reads(&mut tx, "ui_message_audit_log").await;
    assert_eq!(
        i18n::get_audit_log_global(&mut tx, Some("ja"), Some(actor), 50, None)
            .await
            .unwrap()
            .len(),
        5
    );
    bounded(
        "ui_message_audit_log",
        since(reads(&mut tx, "ui_message_audit_log").await, before),
        50,
    );

    drop_indexes(&mut tx, "DROP INDEX idx_ui_message_audit_changed_by;").await;
    let before = reads(&mut tx, "ui_message_audit_log").await;
    i18n::get_audit_log_global(&mut tx, Some("ja"), Some(actor), 50, None)
        .await
        .unwrap();
    scanned(
        "ui_message_audit_log",
        since(reads(&mut tx, "ui_message_audit_log").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_latest_score_time_reads_one_index_entry(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[
            USERS,
            "INSERT INTO user_scores (user_id, total_score, calculated_at)
         SELECT id, uid::int, now() - uid::int * interval '1 second' FROM users;",
        ],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let before = reads(&mut tx, "user_scores").await;
    assert!(
        scores::last_calculated_at(&mut tx, None)
            .await
            .unwrap()
            .is_some()
    );
    bounded(
        "user_scores",
        since(reads(&mut tx, "user_scores").await, before),
        5,
    );

    drop_indexes(&mut tx, "DROP INDEX idx_scores_calculated_at;").await;
    let before = reads(&mut tx, "user_scores").await;
    scores::last_calculated_at(&mut tx, None).await.unwrap();
    scanned(
        "user_scores",
        since(reads(&mut tx, "user_scores").await, before),
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_grid_title_search_counts_its_matches_from_the_trigram_index(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &[
            USERS,
            "INSERT INTO grids (slug, title, created_by, rows, cols, cells)
         SELECT 'grid-' || g, 'Grid ' || g, u.id, 3, 3, '[]'
         FROM generate_series(1, 10000) g JOIN users u ON u.uid = (1 + g % 2000)::text;",
        ],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let before = reads(&mut tx, "grids").await;
    let (page, total) = grids::find_listed(&mut tx, Some("%rid 12%"), ListOrder::Recent, 24, 0)
        .await
        .unwrap();
    assert_eq!((page.len(), total), (24, 111));
    bounded("grids", since(reads(&mut tx, "grids").await, before), N / 2);

    drop_indexes(&mut tx, "DROP INDEX idx_grids_title_trgm;").await;
    let before = reads(&mut tx, "grids").await;
    grids::find_listed(&mut tx, Some("%rid 12%"), ListOrder::Recent, 24, 0)
        .await
        .unwrap();
    scanned("grids", since(reads(&mut tx, "grids").await, before));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_admin_account_lists_read_one_page(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(&mut conn, &[USERS]).await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let everyone = AdminFilter {
        q: None,
        roles: None,
        server: None,
    };
    let staff = ["super_admin".to_owned()];
    let staff = AdminFilter {
        q: None,
        roles: Some(&staff),
        server: None,
    };
    let uid = AdminFilter {
        q: Some("1234"),
        roles: None,
        server: None,
    };

    let before = reads(&mut tx, "users").await;
    assert_eq!(users::recent(&mut tx, 50).await.unwrap().len(), 50);
    assert_eq!(
        users::admin_page(&mut tx, &everyone, 50, 0)
            .await
            .unwrap()
            .len(),
        50
    );
    assert_eq!(
        users::admin_page(&mut tx, &staff, 50, 0)
            .await
            .unwrap()
            .len(),
        10
    );
    assert_eq!(users::admin_count(&mut tx, &uid).await.unwrap(), 1);
    bounded("users", since(reads(&mut tx, "users").await, before), 300);

    for ddl in [
        "DROP INDEX idx_users_created_at;",
        "DROP INDEX idx_users_admin_order;",
        "DROP INDEX idx_users_staff;",
        "DROP INDEX idx_users_uid_trgm;",
    ] {
        let mut sp = sqlx::Connection::begin(&mut *tx).await.unwrap();
        drop_indexes(&mut sp, ddl).await;
        let before = reads(&mut sp, "users").await;
        let found = match ddl {
            "DROP INDEX idx_users_created_at;" => users::recent(&mut sp, 50).await.unwrap().len(),
            "DROP INDEX idx_users_admin_order;" => users::admin_page(&mut sp, &everyone, 50, 0)
                .await
                .unwrap()
                .len(),
            "DROP INDEX idx_users_staff;" => users::admin_page(&mut sp, &staff, 50, 0)
                .await
                .unwrap()
                .len(),
            _ => usize::try_from(users::admin_count(&mut sp, &uid).await.unwrap()).unwrap(),
        };
        assert!(found > 0);
        scanned(ddl, since(reads(&mut sp, "users").await, before));
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn trending_reads_only_the_last_week_of_views(pool: PgPool) {
    let _slot = SLOTS.acquire().await.unwrap();
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    seed(
        &mut conn,
        &["ALTER TABLE tier_list_view_events SET (autovacuum_enabled = false);
         INSERT INTO tier_lists (name, slug) SELECT 'L' || g, 'l-' || g FROM generate_series(1, 200) g;
         INSERT INTO tier_list_stats (tier_list_id) SELECT id FROM tier_lists;
         INSERT INTO tier_list_view_events (tier_list_id, session_hash, viewed_at)
         SELECT t.id, md5(g::text) || md5(g::text), now() - (g % 365) * interval '1 day' - interval '1 hour'
         FROM generate_series(1, 20000) g JOIN tier_lists t ON t.slug = 'l-' || (1 + g % 200);"],
    )
    .await;
    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let before = reads(&mut tx, "tier_list_view_events").await;
    tier_lists::recompute_trending(&mut tx, 10).await.unwrap();
    let with = since(reads(&mut tx, "tier_list_view_events").await, before);
    bounded("tier_list_view_events", with, 1_000);

    drop_indexes(&mut tx, "DROP INDEX idx_tl_view_events_viewed_at;").await;
    let before = reads(&mut tx, "tier_list_view_events").await;
    tier_lists::recompute_trending(&mut tx, 10).await.unwrap();
    let without = since(reads(&mut tx, "tier_list_view_events").await, before);
    assert!(without[0] >= N / 2, "{with:?} {without:?}");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_its_tier_lists(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        TIER_LISTS,
        "tier_lists",
        "DROP INDEX idx_tier_lists_created_by;",
    )
    .await;
    bounded("tier_lists", with, 100);
    scanned("tier_lists", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_the_grants_it_made(pool: PgPool) {
    const DATA: &str = "INSERT INTO tier_lists (name, slug) SELECT 'L' || g, 'l-' || g FROM generate_series(1, 10000) g;
        INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
        SELECT t.id, u.id, 'edit', b.id FROM generate_series(1, 10000) g
        JOIN tier_lists t ON t.slug = 'l-' || g
        JOIN users u ON u.uid = (5001 + g % 5000)::text
        JOIN users b ON b.uid = (1 + g % 2000)::text;";
    let [with, without] = delete_reads(
        pool,
        DATA,
        "tier_list_permissions",
        "DROP INDEX idx_tl_permissions_granted_by;",
    )
    .await;
    bounded("tier_list_permissions", with, 100);
    scanned("tier_list_permissions", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_the_versions_it_published(pool: PgPool) {
    const DATA: &str = "INSERT INTO tier_lists (name, slug) SELECT 'L' || g, 'l-' || g FROM generate_series(1, 1000) g;
        INSERT INTO tier_list_versions (tier_list_id, version, snapshot, published_by)
        SELECT t.id, v, '{}', u.id FROM generate_series(1, 1000) g CROSS JOIN generate_series(1, 10) v
        JOIN tier_lists t ON t.slug = 'l-' || g
        JOIN users u ON u.uid = (1 + (g * 10 + v) % 2000)::text;";
    let [with, without] = delete_reads(
        pool,
        DATA,
        "tier_list_versions",
        "DROP INDEX idx_tl_versions_published_by;",
    )
    .await;
    bounded("tier_list_versions", with, 50);
    scanned("tier_list_versions", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_its_tier_list_views(pool: PgPool) {
    const DATA: &str = "INSERT INTO tier_lists (name, slug) SELECT 'L' || g, 'l-' || g FROM generate_series(1, 200) g;
        INSERT INTO tier_list_view_events (tier_list_id, user_id)
        SELECT t.id, u.id FROM generate_series(1, 20000) g
        JOIN tier_lists t ON t.slug = 'l-' || (1 + g % 200)
        JOIN users u ON u.uid = (101 + g % 2000)::text;"; // none for uid 7: only the lookup is read
    let [with, without] = delete_reads(
        pool,
        DATA,
        "tier_list_view_events",
        "DROP INDEX idx_tl_view_events_user;",
    )
    .await;
    bounded("tier_list_view_events", with, 0);
    assert!(
        with[2] <= 10 && without[2] >= 30,
        "{with:?} {without:?}: index pages read"
    );
}

const AUDIT: &str = "INSERT INTO audit_log (table_name, record_id, action, new_data, changed_by)
    SELECT (ARRAY['users', 'user_scores', 'tier_lists', 'tier_placements'])[1 + g % 4], md5(g::text),
           'UPDATE', jsonb_build_object('tier_id', md5(g::text)::uuid), u.id
    FROM generate_series(1, 40000) g JOIN users u ON u.uid = (1 + g % 2000)::text;";

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_the_audit_rows_it_wrote(pool: PgPool) {
    let [with, without] =
        delete_reads(pool, AUDIT, "audit_log", "DROP INDEX idx_audit_changed_by;").await;
    bounded("audit_log", with, 100);
    scanned("audit_log", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_purges_its_audit_rows_by_record(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        AUDIT,
        "audit_log",
        "DROP INDEX idx_audit_table_record;",
    )
    .await;
    bounded("audit_log", with, 100);
    scanned("audit_log", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_purges_placement_audit_rows_by_tier(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        AUDIT,
        "audit_log",
        "DROP INDEX idx_audit_placement_tier;",
    )
    .await;
    bounded("audit_log", with, 100);
    scanned("audit_log", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_its_notes_audit_rows(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        NOTES_AUDIT,
        "operator_notes_audit_log",
        "DROP INDEX idx_notes_audit_changed_by_at;",
    )
    .await;
    bounded("operator_notes_audit_log", with, 100);
    scanned("operator_notes_audit_log", without);
}

const CONTENT_AUDIT_PARTIES: &str = "INSERT INTO content_audit_log
        (actor_id, target_kind, target_id, target_name, action, owner_id, subject_id)
    SELECT a.id, 'tier_list', md5(g::text)::uuid, 'name', 'grant', o.id, o.id
    FROM generate_series(1, 10000) g
    JOIN users a ON a.uid = (5001 + g % 5000)::text
    JOIN users o ON o.uid = (1 + g % 2000)::text;";

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_the_content_audit_rows_it_owns(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        CONTENT_AUDIT_PARTIES,
        "content_audit_log",
        "DROP INDEX idx_content_audit_owner;",
    )
    .await;
    bounded("content_audit_log", with, 100);
    scanned("content_audit_log", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_the_content_audit_rows_about_it(pool: PgPool) {
    let [with, without] = delete_reads(
        pool,
        CONTENT_AUDIT_PARTIES,
        "content_audit_log",
        "DROP INDEX idx_content_audit_subject;",
    )
    .await;
    bounded("content_audit_log", with, 100);
    scanned("content_audit_log", without);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_an_account_reads_only_its_translation_audit_rows(pool: PgPool) {
    const DATA: &str = "INSERT INTO ui_message_audit_log (message_key, locale, changed_by)
        SELECT 'key' || g, 'ja', u.id FROM generate_series(1, 10000) g
        JOIN users u ON u.uid = (1 + g % 2000)::text;";
    let [with, without] = delete_reads(
        pool,
        DATA,
        "ui_message_audit_log",
        "DROP INDEX idx_ui_message_audit_changed_by;",
    )
    .await;
    bounded("ui_message_audit_log", with, 100);
    scanned("ui_message_audit_log", without);
}
