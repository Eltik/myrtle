#![allow(clippy::unwrap_used, clippy::too_many_lines)]

mod common;

use chrono::{DateTime, Duration, Utc};
use myrtle_db::repos::users::{self, AdminFilter, LayoutUpdate, SettingsUpdate};
use myrtle_db::{Conn, DbError};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

async fn exec(conn: &mut Conn, sql: &'static str, binds: &[Uuid]) {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.execute(conn).await.unwrap();
}

async fn insert_id(conn: &mut Conn, sql: &'static str, binds: &[Uuid]) -> Uuid {
    let mut q = sqlx::query_scalar::<_, Uuid>(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.fetch_one(conn).await.unwrap()
}

async fn scalar(conn: &mut Conn, sql: &'static str, binds: &[Uuid]) -> i64 {
    let mut q = sqlx::query_scalar::<_, i64>(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.fetch_one(conn).await.unwrap()
}

async fn set_created_at(conn: &mut Conn, user: UserId, at: DateTime<Utc>) {
    sqlx::query("UPDATE users SET created_at = $2 WHERE id = $1")
        .bind(user.0)
        .bind(at)
        .execute(conn)
        .await
        .unwrap();
}

async fn set_updated_at(conn: &mut Conn, user: UserId, at: DateTime<Utc>) {
    sqlx::query("UPDATE users SET updated_at = $2 WHERE id = $1")
        .bind(user.0)
        .bind(at)
        .execute(conn)
        .await
        .unwrap();
}

async fn settings(conn: &mut Conn, user: UserId, layout: Option<Value>) {
    sqlx::query("INSERT INTO user_settings (user_id, profile_layout) VALUES ($1, $2)")
        .bind(user.0)
        .bind(layout)
        .execute(conn)
        .await
        .unwrap();
}

async fn stored_layout(conn: &mut Conn, user: UserId) -> Option<Value> {
    stored_settings(conn, user).await.4
}

async fn stored_settings(conn: &mut Conn, user: UserId) -> (bool, bool, bool, bool, Option<Value>) {
    sqlx::query_as(
        "SELECT public_profile, store_gacha, share_stats, public_gacha, profile_layout
         FROM user_settings WHERE user_id = $1",
    )
    .bind(user.0)
    .fetch_one(conn)
    .await
    .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn create_refuses_a_second_account_on_the_same_uid_and_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let row = users::create(&mut conn, "42", Server::EN).await.unwrap();
    assert_eq!(
        (row.uid.as_str(), row.server_id, row.role.as_str()),
        ("42", 0, "user")
    );
    assert_eq!(row.level, Some(0));

    let again = users::create(&mut conn, "42", Server::EN).await;
    assert!(matches!(again, Err(DbError::Conflict { .. })));
    let jp = users::create(&mut conn, "42", Server::JP).await.unwrap();
    assert_ne!(jp.id, row.id);

    let found = users::find(&mut conn, "42", Server::JP)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, jp.id);
    assert!(
        users::find(&mut conn, "42", Server::KR)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn find_or_create_inserts_once_and_leaves_an_existing_row_unwritten(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let first = users::find_or_create(&mut conn, "7", Server::CN)
        .await
        .unwrap();
    let second = users::find_or_create(&mut conn, "7", Server::CN)
        .await
        .unwrap();
    assert_eq!(second.id, first.id);
    assert_eq!(second.updated_at, first.updated_at);
    assert_eq!(second.server_id, Server::CN.index());

    let audit = scalar(
        &mut conn,
        "SELECT count(*) FROM audit_log WHERE table_name = 'users' AND record_id = $1::text",
        &[first.id],
    )
    .await;
    assert_eq!(audit, 1, "only the insert is audited");

    let other = users::find_or_create(&mut conn, "7", Server::TW)
        .await
        .unwrap();
    assert_ne!(other.id, first.id);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn exists_and_lock_report_whether_the_account_is_there(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let ghost = UserId(Uuid::new_v4());

    assert!(users::exists(&mut conn, user).await.unwrap());
    assert!(!users::exists(&mut conn, ghost).await.unwrap());

    let mut tx = db.begin().await.unwrap();
    assert!(users::lock(&mut tx, user).await.unwrap());
    assert!(!users::lock(&mut tx, ghost).await.unwrap());
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_profile_joins_server_score_settings_status_and_roster_counts(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user_on(&mut conn, 5, Server::KR).await;
    let bare = common::user(&mut conn, 6).await;

    settings(&mut conn, user, Some(json!({"tabs": []}))).await;
    exec(
        &mut conn,
        "INSERT INTO user_scores (user_id, total_score, grade) VALUES ($1, 1234.5, 'S')",
        &[user.0],
    )
    .await;
    exec(
        &mut conn,
        "INSERT INTO user_status (user_id, lmd, register_ts, originite) VALUES ($1, 900, 1600000000, 77)",
        &[user.0],
    )
    .await;
    exec(
        &mut conn,
        "INSERT INTO user_operators (user_id, operator_id) VALUES ($1, 'char_002_amiya'), ($1, 'char_003_kalts')",
        &[user.0],
    )
    .await;
    exec(
        &mut conn,
        "INSERT INTO user_skins (user_id, skin_id) VALUES ($1, 'char_002_amiya#1'), ($1, 'char_002_amiya@epoque#4')",
        &[user.0],
    )
    .await;

    let p = users::profile(&mut conn, user).await.unwrap().unwrap();
    assert_eq!((p.uid.as_str(), p.server.as_str()), ("5", "kr"));
    assert_eq!(p.nickname.as_deref(), Some("user5"));
    assert_eq!(
        (p.total_score, p.grade.as_deref()),
        (Some(1234.5), Some("S"))
    );
    assert_eq!(p.public_profile, Some(true));
    assert_eq!(
        (p.lmd, p.register_ts, p.originite),
        (Some(900), Some(1_600_000_000), Some(77))
    );
    assert_eq!((p.operator_count, p.item_count), (Some(2), Some(0)));
    assert_eq!((p.skin_count, p.non_default_skin_count), (Some(2), Some(1)));
    assert_eq!(p.profile_layout, Some(json!({"tabs": []})));

    let by_uid = users::profile_by_uid(&mut conn, "5", None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_uid.id, user.0);

    let b = users::profile(&mut conn, bare).await.unwrap().unwrap();
    assert_eq!((b.public_profile, b.total_score, b.lmd), (None, None, None));
    assert_eq!(b.profile_layout, None);
    assert!(
        users::profile_by_uid(&mut conn, "404", None)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn uid_reads_take_the_named_server_or_the_oldest_account_alone(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let newer = common::user_on(&mut conn, 1, Server::EN).await;
    let older = common::user_on(&mut conn, 1, Server::JP).await;
    set_created_at(&mut conn, older, Utc::now() - Duration::days(1)).await;
    for user in [newer, older] {
        settings(&mut conn, user, None).await;
    }
    exec(
        &mut conn,
        "INSERT INTO user_checkin (user_id, history, cumulative_signin, reward_index, can_check_in)
         VALUES ($1, '{1}', 9, 0, false)",
        &[newer.0],
    )
    .await;

    let bare = users::profile_by_uid(&mut conn, "1", None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((bare.id, bare.server.as_str()), (older.0, "jp"));
    let on_en = users::profile_by_uid(&mut conn, "1", Some(Server::EN))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(on_en.id, newer.0);
    assert!(
        users::profile_by_uid(&mut conn, "1", Some(Server::KR))
            .await
            .unwrap()
            .is_none()
    );

    assert!(
        users::checkin_by_uid(&mut conn, "1", None)
            .await
            .unwrap()
            .is_none(),
        "the oldest account has no checkin, the other account's is not borrowed"
    );
    let checkin = users::checkin_by_uid(&mut conn, "1", Some(Server::EN))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkin.cumulative_signin, 9);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_checkin_row_counts_its_claims_and_reads_status_times_when_present(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;

    assert!(
        users::checkin_by_uid(&mut conn, "1", None)
            .await
            .unwrap()
            .is_none()
    );

    exec(
        &mut conn,
        "INSERT INTO user_checkin (user_id, history, cumulative_signin, reward_index, can_check_in)
         VALUES ($1, '{1,0,1}', 400, 3, true)",
        &[user.0],
    )
    .await;
    let row = users::checkin_by_uid(&mut conn, "1", None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.history, [1, 0, 1]);
    assert_eq!((row.claimed_this_month, row.cumulative_signin), (3, 400));
    assert_eq!((row.reward_index, row.can_check_in), (3, true));
    assert_eq!((row.register_ts, row.last_online_ts), (None, None));

    exec(
        &mut conn,
        "INSERT INTO user_status (user_id, register_ts, last_online_ts) VALUES ($1, 10, 20)",
        &[user.0],
    )
    .await;
    let row = users::checkin_by_uid(&mut conn, "1", None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((row.register_ts, row.last_online_ts), (Some(10), Some(20)));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_role_is_read_and_replaced_by_account_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;

    assert_eq!(
        users::role(&mut conn, user).await.unwrap().as_deref(),
        Some("user")
    );
    users::set_role(&mut conn, user, "translator")
        .await
        .unwrap();
    assert_eq!(
        users::role(&mut conn, user).await.unwrap().as_deref(),
        Some("translator")
    );
    assert_eq!(
        users::role(&mut conn, other).await.unwrap().as_deref(),
        Some("user")
    );
    assert_eq!(
        users::role(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap(),
        None
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn touch_moves_updated_at_to_now(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let past = Utc::now() - Duration::days(3);
    set_updated_at(&mut conn, user, past).await;

    users::touch(&mut conn, user).await.unwrap();
    let row = users::find(&mut conn, "1", Server::EN)
        .await
        .unwrap()
        .unwrap();
    assert!(row.updated_at > past + Duration::days(2));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn settings_flags_keep_what_is_unset_and_a_missing_settings_row_is_not_found(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let without_row = common::user(&mut conn, 2).await;
    settings(&mut conn, user, Some(json!({"tabs": []}))).await;

    let update = SettingsUpdate {
        public_profile: Some(false),
        public_gacha: Some(true),
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(
        stored_settings(&mut conn, user).await,
        (false, true, true, true, Some(json!({"tabs": []})))
    );

    let update = SettingsUpdate {
        store_gacha: Some(false),
        share_stats: Some(false),
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(
        stored_settings(&mut conn, user).await,
        (false, false, false, true, Some(json!({"tabs": []})))
    );

    let missing = users::update_settings(&mut conn, without_row, &update).await;
    assert!(matches!(missing, Err(DbError::NotFound)));
    let rows = scalar(
        &mut conn,
        "SELECT count(*) FROM user_settings WHERE user_id = $1",
        &[without_row.0],
    )
    .await;
    assert_eq!(rows, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_layout_merge_replaces_carried_keys_drops_removed_ones_and_empties_to_null(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    settings(
        &mut conn,
        user,
        Some(json!({"tabs": [{"id": "score", "visible": false}], "background": {"kind": "operator"}})),
    )
    .await;

    let patch = json!({"showcase": {"blocks": []}, "tabs": []});
    let removed = vec!["background".to_owned()];
    let update = SettingsUpdate {
        layout: LayoutUpdate::Merge {
            patch: &patch,
            removed: &removed,
        },
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(
        stored_layout(&mut conn, user).await,
        Some(json!({"tabs": [], "showcase": {"blocks": []}}))
    );

    users::update_settings(&mut conn, user, &SettingsUpdate::default())
        .await
        .unwrap();
    assert_eq!(
        stored_layout(&mut conn, user).await,
        Some(json!({"tabs": [], "showcase": {"blocks": []}})),
        "Keep leaves the layout"
    );

    let empty = json!({});
    let all = vec!["tabs".to_owned(), "showcase".to_owned()];
    let update = SettingsUpdate {
        layout: LayoutUpdate::Merge {
            patch: &empty,
            removed: &all,
        },
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(
        stored_layout(&mut conn, user).await,
        None,
        "no key left is NULL"
    );

    let patch = json!({"tabs": []});
    let update = SettingsUpdate {
        layout: LayoutUpdate::Merge {
            patch: &patch,
            removed: &[],
        },
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(
        stored_layout(&mut conn, user).await,
        Some(json!({"tabs": []}))
    );

    let update = SettingsUpdate {
        layout: LayoutUpdate::Clear,
        ..SettingsUpdate::default()
    };
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
    assert_eq!(stored_layout(&mut conn, user).await, None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn base_facts_are_upserted_into_the_settings_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let with_row = common::user(&mut conn, 2).await;
    settings(&mut conn, with_row, None).await;

    assert_eq!(users::base_facts(&mut conn, user).await.unwrap(), None);
    assert_eq!(users::base_facts(&mut conn, with_row).await.unwrap(), None);

    users::set_base_facts(&mut conn, user, &json!({"drones": 1}))
        .await
        .unwrap();
    users::set_base_facts(&mut conn, user, &json!({"drones": 2}))
        .await
        .unwrap();
    assert_eq!(
        users::base_facts(&mut conn, user).await.unwrap(),
        Some(json!({"drones": 2}))
    );

    users::set_base_facts(&mut conn, with_row, &json!([]))
        .await
        .unwrap();
    assert!(
        stored_settings(&mut conn, with_row).await.0,
        "the row's flags survive"
    );
    assert_eq!(
        users::base_facts(&mut conn, with_row).await.unwrap(),
        Some(json!([]))
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn account_lookups_carry_the_server_code_and_order_by_creation(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let en = common::user_on(&mut conn, 9, Server::EN).await;
    let jp = common::user_on(&mut conn, 9, Server::JP).await;
    let lone = common::user_on(&mut conn, 10, Server::Bilibili).await;
    set_created_at(&mut conn, en, now - Duration::days(1)).await;
    set_created_at(&mut conn, jp, now - Duration::days(2)).await;
    set_created_at(&mut conn, lone, now - Duration::days(3)).await;

    let row = users::account(&mut conn, lone).await.unwrap().unwrap();
    assert_eq!(
        (row.uid.as_str(), row.server.as_str(), row.server_id),
        ("10", "bili", 4)
    );
    assert_eq!(row.nickname.as_deref(), Some("user10"));
    assert!(
        users::account(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );

    let both: Vec<_> = users::accounts_by_uid(&mut conn, "9")
        .await
        .unwrap()
        .into_iter()
        .map(|a| a.server)
        .collect();
    assert_eq!(both, ["jp", "en"]);

    let recent: Vec<Uuid> = users::recent(&mut conn, 2)
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(recent, [en.0, jp.0]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn accounts_created_together_on_one_uid_list_by_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let high = Uuid::from_u128(u128::MAX);
    let low = Uuid::from_u128(1);
    for (id, server) in [(high, Server::EN), (low, Server::JP)] {
        sqlx::query(
            "INSERT INTO users (id, uid, server_id, created_at)
             VALUES ($1, '9', $2, '2024-01-01T00:00:00Z')",
        )
        .bind(id)
        .bind(server.index())
        .execute(&mut *conn)
        .await
        .unwrap();
    }

    let ids: Vec<Uuid> = users::accounts_by_uid(&mut conn, "9")
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(ids, [low, high]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn account_search_matches_uid_or_nickname_newest_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let by_uid = common::user(&mut conn, 777).await;
    let by_name = common::user(&mut conn, 1).await;
    let neither = common::user(&mut conn, 2).await;
    sqlx::query("UPDATE users SET nickname = 'Doc777' WHERE id = $1")
        .bind(by_name.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    set_created_at(&mut conn, by_uid, now - Duration::days(1)).await;

    let found: Vec<Uuid> = users::search_accounts(&mut conn, "DOC777")
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(found, [by_name.0]);

    let found: Vec<Uuid> = users::search_accounts(&mut conn, "777")
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(found, [by_name.0, by_uid.0]);
    assert!(!found.contains(&neither.0));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn account_search_reads_percent_and_underscore_literally(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let percent = common::user(&mut conn, 1).await;
    let underscore = common::user(&mut conn, 2).await;
    common::user(&mut conn, 3).await;
    for (user, nick) in [(percent, "50%off"), (underscore, "a_b")] {
        sqlx::query("UPDATE users SET nickname = $2 WHERE id = $1")
            .bind(user.0)
            .bind(nick)
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    for (needle, want) in [("%", percent), ("_", underscore)] {
        let found: Vec<Uuid> = users::search_accounts(&mut conn, needle)
            .await
            .unwrap()
            .iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(found, [want.0], "{needle}");
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn recent_with_base_lists_only_synced_bases_by_last_sync(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let old = common::user(&mut conn, 1).await;
    let new = common::user(&mut conn, 2).await;
    let no_base = common::user(&mut conn, 3).await;
    for u in [old, new] {
        exec(
            &mut conn,
            "INSERT INTO user_building (user_id) VALUES ($1)",
            &[u.0],
        )
        .await;
    }
    set_updated_at(&mut conn, old, now - Duration::days(2)).await;
    set_updated_at(&mut conn, new, now - Duration::days(1)).await;
    set_updated_at(&mut conn, no_base, now).await;

    let ids: Vec<Uuid> = users::recent_with_base(&mut conn, i64::MAX)
        .await
        .unwrap()
        .iter()
        .map(|a| a.id)
        .collect();
    assert_eq!(ids, [new.0, old.0]);
    assert_eq!(
        users::recent_with_base(&mut conn, 1).await.unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn counts_cut_by_server_or_uid(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    common::user_on(&mut conn, 1, Server::EN).await;
    common::user_on(&mut conn, 1, Server::CN).await;
    common::user_on(&mut conn, 2, Server::CN).await;

    assert_eq!(users::count(&mut conn, None).await.unwrap(), 3);
    assert_eq!(users::count(&mut conn, Some(Server::CN)).await.unwrap(), 2);
    assert_eq!(users::count(&mut conn, Some(Server::TW)).await.unwrap(), 0);
    for (uid, server, n) in [
        ("1", None, 2),
        ("1", Some(Server::CN), 1),
        ("2", Some(Server::EN), 0),
        ("3", None, 0),
    ] {
        assert_eq!(
            users::count_by_uid(&mut conn, uid, server).await.unwrap(),
            n,
            "{uid} {server:?}"
        );
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn key_pages_walk_every_account_once_in_id_order(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut all = Vec::new();
    for n in 0..5 {
        let server = if n % 2 == 0 { Server::EN } else { Server::JP };
        all.push((common::user_on(&mut conn, n, server).await.0, server));
    }
    all.sort();

    for server in [None, Some(Server::JP)] {
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = users::keys_page(&mut conn, server, after, 2).await.unwrap();
            let Some(last) = page.last() else { break };
            assert!(page.len() <= 2);
            after = Some(UserId(last.id));
            seen.extend(page.iter().map(|k| k.id));
        }
        let expected: Vec<Uuid> = all
            .iter()
            .filter(|(_, s)| server.is_none_or(|want| *s == want))
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(seen, expected, "{server:?}");
    }

    common::user_on(&mut conn, 1, Server::KR).await;
    let keys = users::keys_by_uid(&mut conn, "1", None).await.unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys[0].id < keys[1].id);
    assert!(keys.iter().all(|k| k.uid == "1"));
    let on_kr = users::keys_by_uid(&mut conn, "1", Some(Server::KR))
        .await
        .unwrap();
    assert_eq!(on_kr.len(), 1);
    assert!(
        users::keys_by_uid(&mut conn, "1", Some(Server::EN))
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn site_and_role_counts_split_the_users_table(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = common::user_on(&mut conn, 1, Server::EN).await;
    let b = common::user_on(&mut conn, 2, Server::Bilibili).await;
    let c = common::user_on(&mut conn, 3, Server::TW).await;
    set_created_at(&mut conn, b, Utc::now() - Duration::days(10)).await;
    set_created_at(&mut conn, c, Utc::now() - Duration::days(40)).await;
    settings(&mut conn, a, None).await;
    settings(&mut conn, b, None).await;
    sqlx::query("UPDATE user_settings SET public_profile = false WHERE user_id = $1")
        .bind(b.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    users::set_role(&mut conn, b, "super_admin").await.unwrap();
    users::set_role(&mut conn, c, "tier_list_editor")
        .await
        .unwrap();

    let s = users::site_counts(&mut conn).await.unwrap();
    assert_eq!(
        (s.total, s.en, s.jp, s.kr, s.cn, s.bili, s.tw),
        (3, 1, 0, 0, 0, 1, 1)
    );
    assert_eq!((s.signups_7d, s.signups_30d, s.public_profiles), (1, 2, 1));

    let r = users::role_counts(&mut conn).await.unwrap();
    assert_eq!(
        (
            r.user,
            r.tier_list_editor,
            r.tier_list_admin,
            r.translator,
            r.super_admin
        ),
        (1, 1, 0, 0, 1)
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_people_list_puts_an_exact_uid_first_then_role_then_nickname(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let player_b = common::user(&mut conn, 1).await;
    let admin = common::user(&mut conn, 2).await;
    let translator = common::user(&mut conn, 3).await;
    let player_a = common::user(&mut conn, 4).await;
    let unnamed = common::user(&mut conn, 5).await;
    for (user, nickname) in [
        (player_b, Some("Bravo")),
        (admin, Some("zulu")),
        (translator, Some("mike")),
        (player_a, Some("alpha")),
        (unnamed, None),
    ] {
        sqlx::query("UPDATE users SET nickname = $2 WHERE id = $1")
            .bind(user.0)
            .bind(nickname)
            .execute(&mut *conn)
            .await
            .unwrap();
    }
    users::set_role(&mut conn, admin, "super_admin")
        .await
        .unwrap();
    users::set_role(&mut conn, translator, "translator")
        .await
        .unwrap();
    settings(&mut conn, player_a, None).await;
    exec(
        &mut conn,
        "INSERT INTO user_scores (user_id, total_score, grade) VALUES ($1, 50, 'A')",
        &[player_a.0],
    )
    .await;
    exec(
        &mut conn,
        "INSERT INTO user_operators (user_id, operator_id) VALUES ($1, 'char_002_amiya')",
        &[player_a.0],
    )
    .await;
    exec(
        &mut conn,
        "INSERT INTO user_items (user_id, item_id) VALUES ($1, '4001'), ($1, '4002')",
        &[player_a.0],
    )
    .await;

    let all = AdminFilter::default();
    let page = users::admin_page(&mut conn, &all, 50, 0).await.unwrap();
    let order: Vec<Uuid> = page.iter().map(|r| r.id).collect();
    assert_eq!(
        order,
        [admin.0, translator.0, player_a.0, player_b.0, unnamed.0]
    );
    assert_eq!(users::admin_count(&mut conn, &all).await.unwrap(), 5);

    let a = &page[2];
    assert_eq!((a.server.as_str(), a.role.as_str()), ("en", "user"));
    assert!(a.public_profile);
    assert_eq!((a.total_score, a.grade.as_deref()), (Some(50.0), Some("A")));
    assert_eq!((a.operator_count, a.item_count, a.skin_count), (1, 2, 0));
    let b = &page[3];
    assert!(!b.public_profile, "no settings row reads as private");
    assert_eq!(b.total_score, None);

    let second: Vec<Uuid> = users::admin_page(&mut conn, &all, 2, 2)
        .await
        .unwrap()
        .iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(second, [player_a.0, player_b.0]);

    let exact = AdminFilter {
        q: Some("4"),
        ..AdminFilter::default()
    };
    let page = users::admin_page(&mut conn, &exact, 50, 0).await.unwrap();
    assert_eq!(page.first().map(|r| r.id), Some(player_a.0));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_people_list_filters_by_nickname_uid_prefix_role_and_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let u10 = common::user(&mut conn, 10).await;
    let u100 = common::user(&mut conn, 100).await;
    let u101 = common::user_on(&mut conn, 101, Server::JP).await;
    let u200 = common::user(&mut conn, 200).await;
    let pct = common::user(&mut conn, 300).await;
    sqlx::query("UPDATE users SET nickname = 'a%b' WHERE id = $1")
        .bind(pct.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    users::set_role(&mut conn, u101, "tier_list_admin")
        .await
        .unwrap();
    let staff = vec!["super_admin".to_owned(), "tier_list_admin".to_owned()];
    let players = vec!["user".to_owned()];

    let prefix = AdminFilter {
        q: Some("10"),
        ..AdminFilter::default()
    };
    assert_eq!(
        ids(users::admin_page(&mut conn, &prefix, 50, 0).await.unwrap()),
        [u10.0, u101.0, u100.0],
        "the exact uid outranks a staff role"
    );
    assert_eq!(users::admin_count(&mut conn, &prefix).await.unwrap(), 3);

    let nickname = AdminFilter {
        q: Some("USER2"),
        ..AdminFilter::default()
    };
    assert_eq!(
        ids(users::admin_page(&mut conn, &nickname, 50, 0)
            .await
            .unwrap()),
        [u200.0]
    );

    let literal = AdminFilter {
        q: Some("%"),
        ..AdminFilter::default()
    };
    assert_eq!(
        ids(users::admin_page(&mut conn, &literal, 50, 0).await.unwrap()),
        [pct.0],
        "a typed % is no wildcard"
    );
    assert_eq!(users::admin_count(&mut conn, &literal).await.unwrap(), 1);

    let by_staff = AdminFilter {
        roles: Some(staff.as_slice()),
        ..AdminFilter::default()
    };
    assert_eq!(
        ids(users::admin_page(&mut conn, &by_staff, 50, 0)
            .await
            .unwrap()),
        [u101.0]
    );
    let by_players = AdminFilter {
        roles: Some(players.as_slice()),
        ..AdminFilter::default()
    };
    assert_eq!(users::admin_count(&mut conn, &by_players).await.unwrap(), 4);

    let jp = AdminFilter {
        server: Some(Server::JP),
        ..AdminFilter::default()
    };
    assert_eq!(
        ids(users::admin_page(&mut conn, &jp, 50, 0).await.unwrap()),
        [u101.0]
    );
    let none = AdminFilter {
        q: Some("10"),
        roles: Some(players.as_slice()),
        server: Some(Server::JP),
    };
    assert_eq!(users::admin_count(&mut conn, &none).await.unwrap(), 0);
}

fn ids(rows: Vec<users::AdminUserRow>) -> Vec<Uuid> {
    rows.iter().map(|r| r.id).collect()
}

/// Every (table, column) that can hold a `users.id`
async fn user_columns(conn: &mut Conn) -> Vec<(String, String)> {
    sqlx::query_as(
        "SELECT c.conrelid::regclass::text, a.attname::text
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
         ORDER BY 1, 2",
    )
    .fetch_all(conn)
    .await
    .unwrap()
}

/// Rows naming `id` in each user column, plus the audit rows keyed on it
async fn census(conn: &mut Conn, id: UserId) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for (table, column) in user_columns(conn).await {
        let sql = format!(r#"SELECT count(*) FROM "{table}" WHERE "{column}" = $1"#);
        let n: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(id.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        out.push((format!("{table}.{column}"), n));
    }
    let audit = scalar(
        conn,
        "SELECT count(*) FROM audit_log
         WHERE table_name IN ('users', 'user_scores') AND record_id = $1::text",
        &[id.0],
    )
    .await;
    out.push(("audit_log.record_id".to_owned(), audit));
    out
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_deleted_account_leaves_no_row_naming_it_and_shared_rows_survive(pool: PgPool) {
    let db = common::db(pool).await;
    let mut tx = db.begin().await.unwrap();
    let c: &mut Conn = &mut tx;
    let me = common::user(c, 1).await;
    let other = common::user(c, 2).await;
    let (me_id, other_id) = (me.0, other.0);

    exec(
        c,
        "INSERT INTO user_settings (user_id) VALUES ($1)",
        &[me_id],
    )
    .await;
    exec(c, "INSERT INTO user_scores (user_id) VALUES ($1)", &[me_id]).await;
    exec(
        c,
        "UPDATE users SET nickname = 'Deleted Doctor' WHERE id = $1",
        &[me_id],
    )
    .await;
    let plan = insert_id(
        c,
        "INSERT INTO operator_plans (user_id, operator_id) VALUES ($1, 'char_002_amiya') RETURNING id",
        &[me_id],
    )
    .await;
    let group = insert_id(
        c,
        "INSERT INTO plan_groups (user_id, name) VALUES ($1, 'g') RETURNING id",
        &[me_id],
    )
    .await;
    exec(
        c,
        "INSERT INTO plan_group_members (plan_group_id, operator_plan_id) VALUES ($1, $2)",
        &[group, plan],
    )
    .await;

    let community = insert_id(
        c,
        "INSERT INTO tier_lists (name, slug, list_type, created_by)
         VALUES ('mine', 'mine', 'community', $1) RETURNING id",
        &[me_id],
    )
    .await;
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
        &[community, other_id],
    )
    .await;
    let official = insert_id(
        c,
        "INSERT INTO tier_lists (name, slug, list_type, created_by)
         VALUES ('site', 'site', 'official', $1) RETURNING id",
        &[me_id],
    )
    .await;
    let theirs = insert_id(
        c,
        "INSERT INTO tier_lists (name, slug, list_type, created_by)
         VALUES ('theirs', 'theirs', 'community', $1) RETURNING id",
        &[other_id],
    )
    .await;
    exec(
        c,
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         VALUES ($1, $2, 'edit', $3)",
        &[theirs, other_id, me_id],
    )
    .await;
    exec(
        c,
        "INSERT INTO tier_list_permissions (tier_list_id, user_id, permission, granted_by)
         VALUES ($1, $2, 'edit', $2)",
        &[official, me_id],
    )
    .await;

    let grid = insert_id(
        c,
        "INSERT INTO grids (slug, title, created_by, rows, cols, cells)
         VALUES ('g', 'g', $1, 1, 1, '[]') RETURNING id",
        &[me_id],
    )
    .await;
    let fork = insert_id(
        c,
        "INSERT INTO grids (slug, title, created_by, rows, cols, cells, template_of)
         VALUES ('f', 'f', $1, 1, 1, '[]', $2) RETURNING id",
        &[other_id, grid],
    )
    .await;

    let note = insert_id(
        c,
        "INSERT INTO operator_notes (operator_id) VALUES ('char_deltest') RETURNING id",
        &[],
    )
    .await;
    exec(
        c,
        "INSERT INTO operator_notes_audit_log (note_id, field_name, changed_by)
         VALUES ($1, 'pros', $2)",
        &[note, me_id],
    )
    .await;
    let audit_row: i64 = sqlx::query_scalar(
        "INSERT INTO audit_log (table_name, record_id, action, changed_by)
         VALUES ('tier_lists', 'x', 'UPDATE', $1) RETURNING id",
    )
    .bind(me_id)
    .fetch_one(&mut *c)
    .await
    .unwrap();
    exec(
        c,
        "INSERT INTO release_overrides (kind, cn_id, updated_by) VALUES ('deltest', 'deltest', $1)",
        &[me_id],
    )
    .await;

    let other_before = census(c, other).await;
    let before = census(c, me).await;
    let deleted = users::delete_account(c, me).await.unwrap().unwrap();
    let after = census(c, me).await;

    for (name, n) in &after {
        assert_eq!(*n, 0, "{name} still names the deleted account");
    }
    assert!(
        before.iter().map(|(_, n)| n).sum::<i64>() >= 15,
        "the fixture reached its tables"
    );
    assert_eq!(deleted.uid, "1");
    assert_eq!(
        deleted.tier_list_slugs,
        ["mine"],
        "only the community list goes"
    );

    let gone = [
        ("SELECT count(*) FROM tier_lists WHERE id = $1", community),
        ("SELECT count(*) FROM tiers WHERE id = $1", tier),
        (
            "SELECT count(*) FROM tier_placements WHERE tier_id = $1",
            tier,
        ),
        (
            "SELECT count(*) FROM audit_log WHERE table_name = 'tier_lists' AND record_id = $1::text",
            community,
        ),
        (
            "SELECT count(*) FROM audit_log WHERE table_name = 'tier_placements'
             AND COALESCE(new_data, old_data)->>'tier_id' = $1::text",
            tier,
        ),
        ("SELECT count(*) FROM grids WHERE id = $1", grid),
        (
            "SELECT count(*) FROM operator_notes_audit_log WHERE note_id = $1",
            note,
        ),
    ];
    for (sql, id) in gone {
        assert_eq!(scalar(c, sql, &[id]).await, 0, "{sql}");
    }
    let kept = [
        (
            "SELECT count(*) FROM tier_lists WHERE id = $1 AND created_by IS NULL",
            official,
        ),
        (
            "SELECT count(*) FROM grids WHERE id = $1 AND template_of IS NULL",
            fork,
        ),
        ("SELECT count(*) FROM operator_notes WHERE id = $1", note),
    ];
    for (sql, id) in kept {
        assert_eq!(scalar(c, sql, &[id]).await, 1, "{sql}");
    }
    assert_eq!(
        scalar(
            c,
            "SELECT count(*) FROM tier_list_permissions
             WHERE tier_list_id = $1 AND user_id = $2 AND granted_by IS NULL",
            &[theirs, other_id],
        )
        .await,
        1
    );
    let audit_kept: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE id = $1 AND changed_by IS NULL")
            .bind(audit_row)
            .fetch_one(&mut *c)
            .await
            .unwrap();
    assert_eq!(audit_kept, 1);
    assert_eq!(
        scalar(
            c,
            "SELECT count(*) FROM release_overrides WHERE kind = 'deltest' AND updated_by IS NULL",
            &[],
        )
        .await,
        1
    );

    let other_after = census(c, other).await;
    for ((name, b), (_, a)) in other_before.iter().zip(&other_after) {
        let expected = if name == "tier_list_favorites.user_id" {
            b - 1
        } else {
            *b
        };
        assert_eq!(*a, expected, "the other user's {name} moved");
    }

    assert!(users::delete_account(c, me).await.unwrap().is_none());
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_deletion_rolled_back_by_the_caller_keeps_the_account(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;

    let mut tx = db.begin().await.unwrap();
    assert!(
        users::delete_account(&mut tx, user)
            .await
            .unwrap()
            .is_some()
    );
    assert!(!users::exists(&mut tx, user).await.unwrap());
    tx.rollback().await.unwrap();

    assert!(users::exists(&mut conn, user).await.unwrap());
    assert!(
        users::delete_account(&mut conn, UserId(Uuid::new_v4()))
            .await
            .unwrap()
            .is_none()
    );
}

async fn short_lock_timeout(conn: &mut Conn) {
    sqlx::query("SET LOCAL lock_timeout = '200ms'")
        .execute(conn)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn find_or_create_racing_an_uncommitted_insert_returns_the_committed_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut first = db.begin().await.unwrap();
    let mut second = db.conn().await.unwrap();

    let created = users::find_or_create(&mut first, "7", Server::EN)
        .await
        .unwrap();
    let (found, ()) = tokio::join!(users::find_or_create(&mut second, "7", Server::EN), async {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        first.commit().await.unwrap();
    });
    assert_eq!(found.unwrap().id, created.id);
    assert_eq!(
        users::count_by_uid(&mut second, "7", None).await.unwrap(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_locked_account_refuses_a_second_lock_until_the_holder_commits(pool: PgPool) {
    let db = common::db(pool).await;
    let user = common::user(&mut db.conn().await.unwrap(), 1).await;

    let mut holder = db.begin().await.unwrap();
    assert!(users::lock(&mut holder, user).await.unwrap());

    let mut waiter = db.begin().await.unwrap();
    short_lock_timeout(&mut waiter).await;
    assert!(users::lock(&mut waiter, user).await.is_err());
    waiter.rollback().await.unwrap();

    holder.commit().await.unwrap();
    let mut waiter = db.begin().await.unwrap();
    short_lock_timeout(&mut waiter).await;
    assert!(users::lock(&mut waiter, user).await.unwrap());
    waiter.rollback().await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_deletion_waits_on_the_account_lock_before_writing_anything(pool: PgPool) {
    let db = common::db(pool).await;
    let user = common::user(&mut db.conn().await.unwrap(), 1).await;

    let mut holder = db.begin().await.unwrap();
    assert!(users::lock(&mut holder, user).await.unwrap());

    let mut deleter = db.begin().await.unwrap();
    let pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *deleter)
        .await
        .unwrap();
    let mut watcher = db.conn().await.unwrap();
    let (deleted, waited_on) = tokio::join!(users::delete_account(&mut deleter, user), async {
        let mut waited_on = None;
        for _ in 0..200 {
            waited_on = sqlx::query_scalar::<_, String>(
                "SELECT query FROM pg_stat_activity WHERE pid = $1 AND wait_event_type = 'Lock'",
            )
            .bind(pid)
            .fetch_optional(&mut *watcher)
            .await
            .unwrap();
            if waited_on.is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        holder.commit().await.unwrap();
        waited_on
    });
    let waited_on = waited_on.expect("the deletion never waited on the lock");
    assert!(
        waited_on.contains("FOR UPDATE"),
        "waited on {waited_on:?} instead of the locking read"
    );
    assert!(deleted.unwrap().is_some());
    deleter.commit().await.unwrap();
    assert!(!users::exists(&mut watcher, user).await.unwrap());
}
