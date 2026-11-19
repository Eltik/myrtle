#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::planner::{self, PlanInput};
use myrtle_db::{Conn, DbError};
use myrtle_types::UserId;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const fn targets(level: i16, skills: &Value, display_on_profile: bool) -> PlanInput<'_> {
    PlanInput {
        target_elite: 2,
        target_level: level,
        target_skill_level: 7,
        target_skills: skills,
        target_modules: skills,
        display_on_profile,
    }
}

async fn plan(conn: &mut Conn, user: UserId, operator: &str, groups: &[&str]) -> planner::PlanRow {
    let groups: Vec<String> = groups.iter().map(|&g| g.to_owned()).collect();
    planner::upsert_plan(
        conn,
        user,
        operator,
        targets(1, &json!([]), true),
        Some(&groups[..]),
    )
    .await
    .unwrap()
}

async fn groups_of(conn: &mut Conn, plan: Uuid) -> Vec<String> {
    planner::group_names(conn, plan).await.unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn upserting_a_plan_replaces_its_targets_and_sets_its_groups_exactly(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let skills = json!([{"skill": 1, "mastery": 3}]);

    let groups = ["b".to_owned(), "a".to_owned(), "a".to_owned()];
    let first = planner::upsert_plan(
        &mut conn,
        user,
        "char_002_amiya",
        targets(40, &skills, false),
        Some(&groups[..]),
    )
    .await
    .unwrap();
    assert_eq!(first.user_id, user.0);
    assert_eq!(first.target_level, 40);
    assert_eq!(first.target_skills, skills);
    assert!(!first.display_on_profile);
    assert_eq!(groups_of(&mut conn, first.id).await, ["a", "b"]);

    let second = planner::upsert_plan(
        &mut conn,
        user,
        "char_002_amiya",
        targets(90, &json!([]), true),
        Some(&["c".to_owned()][..]),
    )
    .await
    .unwrap();
    assert_eq!(second.id, first.id);
    assert_eq!(second.created_at, first.created_at);
    assert!(second.updated_at > first.updated_at);
    assert_eq!(second.target_level, 90);
    assert!(second.display_on_profile);
    assert_eq!(groups_of(&mut conn, first.id).await, ["c"]);

    planner::upsert_plan(
        &mut conn,
        user,
        "char_002_amiya",
        targets(1, &skills, true),
        None,
    )
    .await
    .unwrap();
    assert_eq!(groups_of(&mut conn, first.id).await, ["c"]);
    let names: Vec<String> = planner::list_groups(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect();
    assert_eq!(names, ["a", "b", "c"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_failed_group_write_leaves_the_plan_unchanged(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let kept = plan(&mut conn, user, "char_002_amiya", &["kept"]).await;

    let too_long = ["x".repeat(101)];
    let failed = planner::upsert_plan(
        &mut conn,
        user,
        "char_002_amiya",
        targets(90, &json!([]), false),
        Some(&too_long[..]),
    )
    .await;
    assert!(matches!(failed, Err(DbError::TooLong)));

    let plans = planner::list_plans(&mut conn, user).await.unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].target_level, kept.target_level);
    assert!(plans[0].display_on_profile);
    assert_eq!(groups_of(&mut conn, kept.id).await, ["kept"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn plans_list_latest_edit_first_and_only_the_users_own(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    plan(&mut conn, user, "char_old", &[]).await;
    plan(&mut conn, user, "char_new", &[]).await;
    plan(&mut conn, other, "char_theirs", &[]).await;
    sqlx::query(
        "UPDATE operator_plans SET updated_at = now() - interval '1 hour'
         WHERE operator_id = 'char_old'",
    )
    .execute(&mut *conn)
    .await
    .unwrap();

    let operators: Vec<String> = planner::list_plans(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.operator_id)
        .collect();
    assert_eq!(operators, ["char_new", "char_old"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn deleting_plans_takes_their_memberships_and_counts_what_went(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    plan(&mut conn, user, "char_a", &["g"]).await;
    plan(&mut conn, user, "char_b", &["g"]).await;
    plan(&mut conn, user, "char_c", &[]).await;
    plan(&mut conn, other, "char_a", &[]).await;

    planner::delete_plan(&mut conn, user, "char_c")
        .await
        .unwrap();
    let removed = planner::delete_plans(
        &mut conn,
        user,
        &[
            "char_a".to_owned(),
            "char_b".to_owned(),
            "char_z".to_owned(),
        ],
    )
    .await
    .unwrap();
    assert_eq!(removed, 2);
    assert!(
        planner::list_plans(&mut conn, user)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        planner::list_memberships(&mut conn, user)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        planner::list_plans(&mut conn, other).await.unwrap().len(),
        1
    );
    assert_eq!(
        planner::list_groups(&mut conn, user).await.unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn groups_list_pinned_first_then_by_name_and_update_in_place(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let b = planner::create_group(&mut conn, user, "b").await.unwrap();
    planner::create_group(&mut conn, user, "a").await.unwrap();
    planner::create_group(&mut conn, user, "z").await.unwrap();
    let again = planner::create_group(&mut conn, user, "b").await.unwrap();
    assert_eq!(again.id, b.id);

    let pinned = planner::update_group(&mut conn, user, "z", None, Some(true))
        .await
        .unwrap();
    assert!(pinned.pinned);
    assert_eq!(pinned.name, "z");
    let renamed = planner::update_group(&mut conn, user, "b", Some("c"), None)
        .await
        .unwrap();
    assert_eq!(renamed.id, b.id);
    assert!(!renamed.pinned);
    assert!(renamed.updated_at > b.updated_at);

    let names: Vec<String> = planner::list_groups(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|g| g.name)
        .collect();
    assert_eq!(names, ["z", "a", "c"]);

    let missing = planner::update_group(&mut conn, user, "nope", Some("x"), None).await;
    assert!(matches!(missing, Err(DbError::NotFound)));
    let clash = planner::update_group(&mut conn, user, "a", Some("c"), None).await;
    assert!(matches!(clash, Err(DbError::Conflict { .. })));

    planner::delete_group(&mut conn, user, "a").await.unwrap();
    assert_eq!(
        planner::list_groups(&mut conn, user).await.unwrap().len(),
        2
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_group_update_bumps_updated_at_without_the_trigger(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let group = planner::create_group(&mut conn, user, "g").await.unwrap();
    sqlx::query("ALTER TABLE plan_groups DISABLE TRIGGER trg_plan_groups_timestamp")
        .execute(&mut *conn)
        .await
        .unwrap();

    let pinned = planner::update_group(&mut conn, user, "g", None, Some(true))
        .await
        .unwrap();
    assert!(pinned.updated_at > group.updated_at);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn memberships_come_by_plan_then_group_name(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let x = plan(&mut conn, user, "char_x", &["zeta", "alpha"]).await;
    let y = plan(&mut conn, user, "char_y", &["mid"]).await;
    plan(&mut conn, other, "char_x", &["theirs"]).await;

    let rows = planner::list_memberships(&mut conn, user).await.unwrap();
    let mut expected = vec![
        (x.id, "alpha".to_owned()),
        (x.id, "zeta".to_owned()),
        (y.id, "mid".to_owned()),
    ];
    expected.sort();
    let got: Vec<(Uuid, String)> = rows
        .into_iter()
        .map(|r| (r.operator_plan_id, r.name))
        .collect();
    assert_eq!(got, expected);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_preset_is_replaced_by_name_and_listed_by_name(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let first = planner::upsert_preset(&mut conn, user, "max", &json!({"elite": 1}))
        .await
        .unwrap();
    planner::upsert_preset(&mut conn, user, "e1", &json!({"elite": 1}))
        .await
        .unwrap();
    let replaced = planner::upsert_preset(&mut conn, user, "max", &json!({"elite": 2}))
        .await
        .unwrap();
    assert_eq!(replaced.id, first.id);
    assert_eq!(replaced.target, json!({"elite": 2}));

    let names: Vec<String> = planner::list_presets(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(names, ["e1", "max"]);

    planner::delete_preset(&mut conn, user, "e1").await.unwrap();
    assert_eq!(
        planner::list_presets(&mut conn, user).await.unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn shown_plans_match_an_id_or_an_operator_among_the_owners_profile_plans(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let owner = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let by_id = plan(&mut conn, owner, "char_a", &[]).await;
    let by_operator = plan(&mut conn, owner, "char_b", &[]).await;
    let hidden = planner::upsert_plan(
        &mut conn,
        owner,
        "char_c",
        targets(1, &json!([]), false),
        None,
    )
    .await
    .unwrap();
    let theirs = plan(&mut conn, other, "char_b", &[]).await;

    let mut shown: Vec<(Uuid, String)> = planner::shown_plans(
        &mut conn,
        owner,
        &[by_id.id, hidden.id, theirs.id],
        &["char_b".to_owned(), "char_c".to_owned()],
    )
    .await
    .unwrap()
    .into_iter()
    .map(|p| (p.id, p.operator_id))
    .collect();
    shown.sort();
    let mut expected = vec![
        (by_id.id, "char_a".to_owned()),
        (by_operator.id, "char_b".to_owned()),
    ];
    expected.sort();
    assert_eq!(shown, expected);
}
