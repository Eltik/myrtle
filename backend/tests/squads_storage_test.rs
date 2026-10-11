//! Squad Builder storage against the real schema: a saved squad round-trips
//! through upsert, list and delete, a replace keeps `created_at`, a synced
//! in-game squad reads back padded to twelve seats, and a re-sync writes only
//! the squads that changed.
//!
//! Runs inside one transaction on the local Postgres and rolls it back, so
//! nothing persists. When the local database predates v035 the migration is
//! applied inside that same transaction. Ignored by default:
//! `cargo test --test squads_storage_test -- --ignored --nocapture`

use backend::database::models::squad::{GameSquadRow, SQUAD_SIZE, SquadBuild, SquadMember};
use backend::database::queries::{roster, squads};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

fn member(char_id: &str, module_id: Option<&str>) -> SquadMember {
    SquadMember {
        char_id: char_id.to_owned(),
        skill_index: 2,
        module_id: module_id.map(str::to_owned),
        build: Some(SquadBuild {
            phase: 2,
            level: 90,
            potential: 5,
            skill_level: 7,
            mastery: 3,
            module_level: 3,
        }),
    }
}

/// The squad tables (applying v035 when the local database predates it) and a
/// throwaway user, all inside `conn`'s open transaction.
async fn setup(conn: &mut PgConnection) -> Uuid {
    let migrated: bool = sqlx::query_scalar("SELECT to_regclass('user_squads') IS NOT NULL")
        .fetch_one(&mut *conn)
        .await
        .expect("catalog");
    if !migrated {
        sqlx::raw_sql(include_str!(
            "../src/database/migrations/v035_user_squads.sql"
        ))
        .execute(&mut *conn)
        .await
        .expect("v035 applies");
    }

    let uid = format!("sqt{}", &Uuid::new_v4().simple().to_string()[..12]);
    sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
        .bind(&uid)
        .fetch_one(&mut *conn)
        .await
        .expect("throwaway user")
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn saved_squads_round_trip() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local Postgres");
    let mut tx = conn.begin().await.expect("begin");
    let user_id = setup(&mut tx).await;

    assert!(
        squads::list_saved(&mut tx, user_id)
            .await
            .expect("list")
            .is_empty()
    );

    // A short member list is stored as sent and padded on read.
    let members = vec![
        Some(member("char_4080_lin", Some("uniequip_002_lin"))),
        None,
        Some(member("char_002_amiya", None)),
    ];
    let support = member("char_1035_wisdel", None);
    let first = squads::upsert_saved(
        &mut tx,
        user_id,
        2,
        "Lin",
        &members,
        Some(&support),
        Some("main_14-19"),
    )
    .await
    .expect("insert")
    .into_saved();
    assert_eq!(first.slot, 2);
    assert_eq!(first.name, "Lin");
    assert_eq!(first.members.len(), SQUAD_SIZE);
    assert_eq!(first.members[..3], members[..]);
    assert!(first.members[3..].iter().all(Option::is_none));
    assert_eq!(first.support.as_ref(), Some(&support));
    assert_eq!(first.stage_id.as_deref(), Some("main_14-19"));

    // A replace keeps the slot's `created_at` and drops what the new squad
    // does not carry.
    let replaced = squads::upsert_saved(&mut tx, user_id, 2, "Empty", &[], None, None)
        .await
        .expect("replace")
        .into_saved();
    assert_eq!(replaced.name, "Empty");
    assert_eq!(replaced.created_at, first.created_at);
    assert!(replaced.members.iter().all(Option::is_none));
    assert_eq!(replaced.support, None);
    assert_eq!(replaced.stage_id, None);

    squads::upsert_saved(&mut tx, user_id, 1, "First", &members, None, None)
        .await
        .expect("second slot");
    let listed: Vec<i32> = squads::list_saved(&mut tx, user_id)
        .await
        .expect("list")
        .into_iter()
        .map(|row| row.into_saved().slot)
        .collect();
    assert_eq!(listed, vec![1, 2]);

    assert!(
        squads::delete_saved(&mut tx, user_id, 2)
            .await
            .expect("delete")
    );
    assert!(
        !squads::delete_saved(&mut tx, user_id, 2)
            .await
            .expect("delete again")
    );
    assert_eq!(
        squads::list_saved(&mut tx, user_id)
            .await
            .expect("list")
            .len(),
        1
    );

    // The table refuses a slot outside 1..=4 even if the service let one by.
    let mut probe = tx.begin().await.expect("savepoint");
    assert!(
        squads::upsert_saved(&mut probe, user_id, 5, "x", &[], None, None)
            .await
            .is_err()
    );
    probe.rollback().await.expect("rollback savepoint");

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn synced_squads_read_back_padded() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local Postgres");
    let mut tx = conn.begin().await.expect("begin");
    let user_id = setup(&mut tx).await;

    // Written the way the roster sync writes them.
    sqlx::query(
        "INSERT INTO user_game_squads (user_id, squad_index, name, slots)
         VALUES ($1, 3, 'IV', $2), ($1, 0, 'I', '[]'::jsonb)",
    )
    .bind(user_id)
    .bind(serde_json::json!([
        {"char_id": "char_4080_lin", "skill_index": 1, "module_id": "uniequip_002_lin"},
        null,
        {"char_id": "char_002_amiya", "skill_index": 0, "module_id": null}
    ]))
    .execute(&mut *tx)
    .await
    .expect("synced squads");
    let synced: Vec<_> = squads::list_game_squads(&mut tx, user_id)
        .await
        .expect("list synced")
        .into_iter()
        .map(GameSquadRow::into_game_squad)
        .collect();
    assert_eq!(synced.iter().map(|s| s.index).collect::<Vec<_>>(), [0, 3]);
    assert!(synced.iter().all(|s| s.slots.len() == SQUAD_SIZE));
    let lin = synced[1].slots[0].as_ref().expect("seat 0");
    assert_eq!(lin.char_id, "char_4080_lin");
    assert_eq!(lin.skill_index, 1);
    assert_eq!(lin.module_id.as_deref(), Some("uniequip_002_lin"));
    assert!(synced[1].slots[1].is_none());
    assert_eq!(synced[1].slots[2].as_ref().expect("seat 2").module_id, None);

    tx.rollback().await.expect("rollback");
}

async fn sync(tx: &mut PgConnection, uid: &str, squads: &serde_json::Value) -> u64 {
    roster::sync_game_squads(tx, uid, 0, squads)
        .await
        .expect("sync")
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn resync_writes_only_changed_squads() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local Postgres");
    let mut tx = conn.begin().await.expect("begin");
    let user_id = setup(&mut tx).await;
    let uid: String = sqlx::query_scalar("SELECT uid FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .expect("uid");
    let seat = serde_json::json!({"char_id": "char_4080_lin", "skill_index": 1, "module_id": null});
    let four = serde_json::json!([
        {"index": 0, "name": "I", "slots": [seat, null]},
        {"index": 1, "name": "II", "slots": [null]},
        {"index": 2, "name": "III", "slots": [null]},
        {"index": 3, "name": "IV", "slots": [null]}
    ]);
    // First sync: four rows. The same payload again: none; the old
    // delete-and-reinsert wrote 8 (4 deletes, 4 inserts) here.
    assert_eq!(sync(&mut tx, &uid, &four).await, 4);
    assert_eq!(sync(&mut tx, &uid, &four).await, 0);
    // One squad renamed and one gone: one update, one delete.
    let three = serde_json::json!([
        {"index": 0, "name": "I", "slots": [seat, null]},
        {"index": 1, "name": "II renamed", "slots": [null]},
        {"index": 2, "name": "III", "slots": [null]}
    ]);
    assert_eq!(sync(&mut tx, &uid, &three).await, 2);
    let synced = squads::list_game_squads(&mut tx, user_id)
        .await
        .expect("list synced");
    assert_eq!(
        synced
            .iter()
            .map(|s| (s.squad_index, s.name.as_str()))
            .collect::<Vec<_>>(),
        [(0, "I"), (1, "II renamed"), (2, "III")]
    );
    // `synced_at` is the account's last sync, the same for every squad.
    let updated: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT updated_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await
            .expect("updated_at");
    assert!(synced.iter().all(|s| s.synced_at == updated));

    tx.rollback().await.expect("rollback");
}
