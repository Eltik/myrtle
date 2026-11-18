#![allow(clippy::unwrap_used)]

mod common;

use chrono::{DateTime, Duration, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::lookback::{self, NewSnapshot};
use myrtle_types::UserId;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const KEEP_EVERY_DAYS: i32 = 365;

const fn snapshot(period_start: i64, summary: &serde_json::Value) -> NewSnapshot<'_> {
    NewSnapshot {
        period_start,
        activity_id: None,
        activity_name: None,
        summary,
        build: summary,
    }
}

async fn old_row(conn: &mut Conn, user: UserId, at: DateTime<Utc>) {
    sqlx::query(
        "INSERT INTO user_lookback_snapshots (user_id, period_start, summary, build)
         VALUES ($1, $2, '{}', '{}')",
    )
    .bind(user.0)
    .bind(at)
    .execute(conn)
    .await
    .unwrap();
}

async fn starts(conn: &mut Conn, user: UserId) -> Vec<DateTime<Utc>> {
    lookback::list(conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.period_start)
        .collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_period_is_written_once_and_read_back_oldest_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let now = Utc::now().timestamp();
    let (week, event) = (now - 7 * 86_400, now - 86_400);
    let summary = json!({"operators": 10});
    let build = json!({"o": [["char_002_amiya", 2, 50, 0, 7, [3], []]]});

    assert!(
        !lookback::period_taken(&mut conn, user, event)
            .await
            .unwrap()
    );
    let written = lookback::insert(
        &mut conn,
        user,
        &NewSnapshot {
            period_start: event,
            activity_id: Some("act1side"),
            activity_name: Some("Grani"),
            summary: &summary,
            build: &build,
        },
        KEEP_EVERY_DAYS,
    )
    .await
    .unwrap();
    assert!(written);
    assert!(
        lookback::period_taken(&mut conn, user, event)
            .await
            .unwrap()
    );

    let again = json!({"operators": 99});
    assert!(
        !lookback::insert(&mut conn, user, &snapshot(event, &again), KEEP_EVERY_DAYS)
            .await
            .unwrap()
    );
    assert!(
        lookback::insert(&mut conn, user, &snapshot(week, &again), KEEP_EVERY_DAYS)
            .await
            .unwrap()
    );

    let list = lookback::list(&mut conn, user).await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].period_start.timestamp(), week);
    assert_eq!(list[0].activity_id, None);
    assert_eq!(list[1].activity_id.as_deref(), Some("act1side"));
    assert_eq!(list[1].activity_name.as_deref(), Some("Grani"));
    assert_eq!(list[1].summary, summary);

    let one = lookback::get(&mut conn, user, event)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(one.summary, summary);
    assert_eq!(one.build, build);
    assert_eq!(one.taken_at, list[1].taken_at);
    assert!(lookback::get(&mut conn, user, now).await.unwrap().is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_write_prunes_old_rows_except_the_first_of_each_month(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let month = |y, m, d| {
        DateTime::parse_from_rfc3339(&format!("{y}-{m:02}-{d:02}T12:00:00Z"))
            .unwrap()
            .with_timezone(&Utc)
    };
    let old = [
        month(2020, 3, 2),
        month(2020, 3, 9),
        month(2020, 3, 30),
        month(2020, 4, 6),
    ];
    for at in old {
        old_row(&mut conn, user, at).await;
        old_row(&mut conn, other, at).await;
    }
    let recent = Utc::now() - Duration::days(30);
    let recent_same_month = recent + Duration::hours(1);
    old_row(&mut conn, user, recent).await;
    old_row(&mut conn, user, recent_same_month).await;

    let summary = json!({});
    let duplicate = old[1].timestamp();
    assert!(
        !lookback::insert(
            &mut conn,
            user,
            &snapshot(duplicate, &summary),
            KEEP_EVERY_DAYS
        )
        .await
        .unwrap()
    );
    assert_eq!(starts(&mut conn, user).await.len(), 6);

    let now = Utc::now().timestamp();
    assert!(
        lookback::insert(&mut conn, user, &snapshot(now, &summary), KEEP_EVERY_DAYS)
            .await
            .unwrap()
    );
    let kept: Vec<i64> = starts(&mut conn, user)
        .await
        .iter()
        .map(DateTime::timestamp)
        .collect();
    assert_eq!(
        kept,
        [
            old[0].timestamp(),
            old[3].timestamp(),
            recent.timestamp(),
            recent_same_month.timestamp(),
            now
        ]
    );
    assert_eq!(starts(&mut conn, other).await.len(), 4);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn build_rows_list_masteries_by_skill_and_unlocked_modules_by_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    sqlx::query(
        "INSERT INTO user_operators (user_id, operator_id, elite, level, potential, skill_level)
         VALUES ($1, 'char_003_kalts', 2, 90, 5, 7), ($1, 'char_002_amiya', 1, 40, 0, 4)",
    )
    .bind(user.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_operator_skills (user_id, operator_id, skill_index, specialize_level)
         VALUES ($1, 'char_003_kalts', 2, 3), ($1, 'char_003_kalts', 0, 1)",
    )
    .bind(user.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_operator_modules (user_id, operator_id, module_id, module_level, locked)
         VALUES ($1, 'char_003_kalts', 'uniequip_003_kalts', 3, false),
                ($1, 'char_003_kalts', 'uniequip_002_kalts', 1, false),
                ($1, 'char_003_kalts', 'uniequip_001_kalts', 1, true)",
    )
    .bind(user.0)
    .execute(&mut *conn)
    .await
    .unwrap();

    let rows = lookback::build_rows(&mut conn, user).await.unwrap();
    let ids: Vec<&str> = rows.iter().map(|r| r.operator_id.as_str()).collect();
    assert_eq!(ids, ["char_002_amiya", "char_003_kalts"]);
    let amiya = &rows[0];
    assert_eq!((amiya.elite, amiya.level, amiya.skill_level), (1, 40, 4));
    assert!(amiya.masteries.is_empty() && amiya.module_ids.is_empty());
    assert!(amiya.module_levels.is_empty());
    let kalts = &rows[1];
    assert_eq!(kalts.potential, 5);
    assert_eq!(kalts.masteries, [1, 3]);
    assert_eq!(
        kalts.module_ids,
        ["uniequip_002_kalts", "uniequip_003_kalts"]
    );
    assert_eq!(kalts.module_levels, [1, 3]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn summary_extras_count_non_default_skins_and_carry_the_grade(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let graded = common::user(&mut conn, 1).await;
    let ungraded = common::user(&mut conn, 2).await;
    sqlx::query(
        "INSERT INTO user_skins (user_id, skin_id) VALUES
         ($1, 'char_002_amiya@epoque#4'), ($1, 'char_002_amiya#1'), ($2, 'char_003_kalts#1')",
    )
    .bind(graded.0)
    .bind(ungraded.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_scores (user_id, total_score, grade) VALUES ($1, 812.5, 'A')")
        .bind(graded.0)
        .execute(&mut *conn)
        .await
        .unwrap();

    let a = lookback::summary_extras(&mut conn, graded)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((a.server_id, a.skins), (0, 1));
    assert!(a.total_score.is_some_and(|s| (s - 812.5).abs() < 1e-9));
    assert_eq!(a.grade.as_deref(), Some("A"));

    let b = lookback::summary_extras(&mut conn, ungraded)
        .await
        .unwrap()
        .unwrap();
    assert_eq!((b.skins, b.total_score, b.grade), (0, None, None));

    let missing = lookback::summary_extras(&mut conn, UserId(Uuid::nil()))
        .await
        .unwrap();
    assert!(missing.is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn obtained_times_read_a_missing_time_as_zero(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    sqlx::query(
        "INSERT INTO user_operators (user_id, operator_id, obtained_at)
         VALUES ($1, 'char_002_amiya', 1600000000), ($1, 'char_003_kalts', NULL)",
    )
    .bind(user.0)
    .execute(&mut *conn)
    .await
    .unwrap();

    let mut times = lookback::obtained_times(&mut conn, user).await.unwrap();
    times.sort_unstable();
    assert_eq!(times, [0, 1_600_000_000]);
}
