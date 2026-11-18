#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::{roster, squads};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

struct Payload {
    operators: Value,
    skills: Value,
    modules: Value,
    skins: Value,
    status: Value,
    supports: Value,
    game_squads: Value,
    empty_list: Value,
    empty_map: Value,
}

impl Payload {
    fn new() -> Self {
        let op = |id: &str, elite: i16| {
            json!({
                "operator_id": id, "elite": elite, "level": 50, "exp": 0, "potential": 0,
                "skill_level": 7, "favor_point": 100, "skin_id": format!("{id}#1"),
                "default_skill": 1, "voice_lan": "JP", "current_equip": null,
                "current_tmpl": null, "obtained_at": 1_600_000_000
            })
        };
        Self {
            operators: json!([op("char_002_amiya", 2), op("char_003_kalts", 1)]),
            skills: json!([
                {"operator_id": "char_002_amiya", "skill_index": 1, "specialize_level": 3},
                {"operator_id": "char_002_amiya", "skill_index": 0, "specialize_level": 1}
            ]),
            modules: json!([
                {"operator_id": "char_002_amiya", "module_id": "uniequip_002_amiya",
                 "module_level": 2, "locked": false}
            ]),
            skins: json!([{"skin_id": "char_002_amiya@epoque#4", "obtained_at": 1_700_000_000}]),
            status: status(Some(7)),
            supports: json!([]),
            game_squads: json!([{"index": 0, "name": "I", "slots": [null]}]),
            empty_list: json!([]),
            empty_map: json!({}),
        }
    }

    const fn data(&self) -> roster::SyncUserData<'_> {
        roster::SyncUserData {
            uid: "1",
            server: Server::EN,
            nickname: "Doctor",
            level: 120,
            avatar_id: None,
            secretary: Some("char_002_amiya"),
            secretary_skin_id: None,
            resume_id: None,
            operators: &self.operators,
            skills: &self.skills,
            modules: &self.modules,
            items: &self.empty_list,
            skins: &self.skins,
            status: &self.status,
            stages: &self.empty_map,
            roguelike: &self.empty_list,
            sandbox: &self.empty_map,
            medals: &self.empty_list,
            building: &self.empty_map,
            checkin: &self.empty_map,
            supports: &self.supports,
            nick_number: Some("1234"),
            enemies: &self.empty_map,
            game_squads: &self.game_squads,
        }
    }
}

fn status(originite: Option<i64>) -> Value {
    let mut status = json!({});
    for key in [
        "exp",
        "orundum",
        "orundum_shard",
        "lmd",
        "sanity",
        "max_sanity",
        "gacha_tickets",
        "ten_pull_tickets",
        "classic_gacha_tickets",
        "classic_ten_pull_tickets",
        "recruit_permits",
        "social_point",
        "hgg_shard",
        "lgg_shard",
        "practice_tickets",
        "gold",
        "friend_num_limit",
    ] {
        status[key] = json!(0);
    }
    if let Some(o) = originite {
        status["originite"] = json!(o);
    }
    status
}

async fn synced_user(conn: &mut Conn) -> UserId {
    let id = sqlx::query_scalar("SELECT id FROM users WHERE uid = '1' AND server_id = $1")
        .bind(Server::EN.index())
        .fetch_one(conn)
        .await
        .unwrap();
    UserId(id)
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_sync_writes_the_roster_the_view_reads_back(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let payload = Payload::new();

    roster::sync_user_data(&mut conn, &payload.data())
        .await
        .unwrap();
    let user = synced_user(&mut conn).await;

    let entries = roster::roster(&mut conn, user).await.unwrap();
    assert_eq!(entries.len(), 2);
    let amiya = entries
        .iter()
        .find(|e| e.operator_id == "char_002_amiya")
        .unwrap();
    assert_eq!(amiya.operator_id, "char_002_amiya");
    assert_eq!((amiya.elite, amiya.level, amiya.skill_level), (2, 50, 7));
    assert_eq!(amiya.default_skill, Some(1));
    assert_eq!(amiya.obtained_at, Some(1_600_000_000));
    assert_eq!(
        amiya.masteries,
        json!([{"index": 0, "mastery": 1}, {"index": 1, "mastery": 3}])
    );
    assert_eq!(
        amiya.modules,
        json!([{"id": "uniequip_002_amiya", "level": 2, "locked": false}])
    );
    let kalts = entries
        .iter()
        .find(|e| e.operator_id == "char_003_kalts")
        .unwrap();
    assert_eq!(kalts.masteries, json!([]));
    assert_eq!(kalts.user_id, user.0);

    let one = roster::operator(&mut conn, user, "char_003_kalts")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(one.elite, 1);
    assert!(
        roster::operator(&mut conn, user, "char_004_none")
            .await
            .unwrap()
            .is_none()
    );

    let originite: Option<i32> =
        sqlx::query_scalar("SELECT originite FROM user_status WHERE user_id = $1")
            .bind(user.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(originite, Some(7));
    let game_squads = squads::list_game_squads(&mut conn, user).await.unwrap();
    assert_eq!(game_squads.len(), 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_resync_replaces_the_roster_and_clears_originite_when_absent(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut payload = Payload::new();
    roster::sync_user_data(&mut conn, &payload.data())
        .await
        .unwrap();

    payload.operators = json!([payload.operators[1].clone()]);
    payload.skills = json!([]);
    payload.modules = json!([]);
    payload.status = status(None);
    roster::sync_user_data(&mut conn, &payload.data())
        .await
        .unwrap();
    let user = synced_user(&mut conn).await;

    let ids: Vec<String> = roster::roster(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|e| e.operator_id)
        .collect();
    assert_eq!(ids, ["char_003_kalts"]);
    let originite: Option<i32> =
        sqlx::query_scalar("SELECT originite FROM user_status WHERE user_id = $1")
            .bind(user.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    assert_eq!(originite, None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_originite_count_past_the_column_range_saturates(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut payload = Payload::new();
    for (sent, stored) in [
        (i64::from(i32::MAX) + 8, i32::MAX),
        (i64::from(i32::MIN) - 8, i32::MIN),
    ] {
        payload.status = status(Some(sent));
        roster::sync_user_data(&mut conn, &payload.data())
            .await
            .unwrap();
        let user = synced_user(&mut conn).await;
        let originite: Option<i32> =
            sqlx::query_scalar("SELECT originite FROM user_status WHERE user_id = $1")
                .bind(user.0)
                .fetch_one(&mut *conn)
                .await
                .unwrap();
        assert_eq!(originite, Some(stored), "{sent}");
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_sync_whose_squads_fail_leaves_nothing_behind(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut payload = Payload::new();
    payload.game_squads = json!([{"index": "not a number", "name": "I", "slots": []}]);

    assert!(
        roster::sync_user_data(&mut conn, &payload.data())
            .await
            .is_err()
    );

    let accounts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(accounts, 0);
}

async fn sync(conn: &mut Conn, squads: &Value) -> u64 {
    roster::sync_game_squads(conn, "1", Server::EN, squads)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_resync_writes_only_the_squads_that_changed(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let seat = json!({"char_id": "char_4080_lin", "skill_index": 1, "module_id": null});
    let four = json!([
        {"index": 0, "name": "I", "slots": [seat, null]},
        {"index": 1, "name": "II", "slots": [null]},
        {"index": 2, "name": "III", "slots": [null]},
        {"index": 3, "name": "IV", "slots": [null]}
    ]);

    assert_eq!(sync(&mut conn, &four).await, 4);
    assert_eq!(sync(&mut conn, &four).await, 0);
    let three = json!([
        {"index": 0, "name": "I", "slots": [seat, null]},
        {"index": 1, "name": "II renamed", "slots": [null]},
        {"index": 2, "name": "III", "slots": [null]}
    ]);
    assert_eq!(sync(&mut conn, &three).await, 2);

    let synced: Vec<(i16, String)> = squads::list_game_squads(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|s| (s.squad_index, s.name))
        .collect();
    assert_eq!(
        synced,
        [
            (0, "I".to_owned()),
            (1, "II renamed".to_owned()),
            (2, "III".to_owned())
        ]
    );

    assert_eq!(sync(&mut conn, &json!({"not": "an array"})).await, 3);
    assert!(
        squads::list_game_squads(&mut conn, user)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn squads_of_an_unknown_account_write_nothing(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    common::user(&mut conn, 1).await;

    let written = roster::sync_game_squads(
        &mut conn,
        "1",
        Server::JP,
        &json!([{"index": 0, "name": "I", "slots": []}]),
    )
    .await
    .unwrap();
    assert_eq!(written, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn supports_carry_the_operator_state_and_default_to_no_mastery(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let mut payload = Payload::new();
    payload.supports = json!([
        {"slot": 1, "operator_id": "char_404_gone", "skill_index": 0},
        {"slot": 0, "operator_id": "char_002_amiya", "skill_index": 1, "skin_id": null},
        {"slot": 2, "operator_id": "char_003_kalts", "skill_index": 2,
         "skin_id": "char_003_kalts@own#1", "current_equip": "uniequip_001_kalts"},
        {"slot": 3, "operator_id": null}
    ]);
    roster::sync_user_data(&mut conn, &payload.data())
        .await
        .unwrap();
    let user = synced_user(&mut conn).await;

    let supports = roster::supports(&mut conn, user).await.unwrap();
    let slots: Vec<i16> = supports.iter().map(|s| s.slot).collect();
    assert_eq!(slots, [0, 1, 2]);

    let amiya = &supports[0];
    assert_eq!(amiya.skin_id.as_deref(), Some("char_002_amiya#1"));
    assert_eq!(amiya.elite, Some(2));
    assert_eq!(amiya.specialize_level, 3);

    let gone = &supports[1];
    assert_eq!(gone.elite, None);
    assert_eq!(gone.favor_point, None);
    assert_eq!(gone.specialize_level, 0);

    let kalts = &supports[2];
    assert_eq!(kalts.skin_id.as_deref(), Some("char_003_kalts@own#1"));
    assert_eq!(kalts.current_equip.as_deref(), Some("uniequip_001_kalts"));
    assert_eq!(kalts.specialize_level, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn owned_skins_and_popularity_skip_default_skins(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = common::user(&mut conn, 1).await;
    let b = common::user(&mut conn, 2).await;
    let c = common::user(&mut conn, 3).await;
    sqlx::query(
        "INSERT INTO user_skins (user_id, skin_id, obtained_at) VALUES
         ($1, 'char_003_kalts@own#1', 10), ($1, 'char_002_amiya@epoque#4', NULL),
         ($1, 'char_002_amiya#1', 1), ($2, 'char_002_amiya@epoque#4', 20),
         ($3, 'char_002_amiya#1', 1)",
    )
    .bind(a.0)
    .bind(b.0)
    .bind(c.0)
    .execute(&mut *conn)
    .await
    .unwrap();

    let owned: Vec<(String, Option<i64>)> = roster::owned_skins(&mut conn, a)
        .await
        .unwrap()
        .into_iter()
        .map(|s| (s.skin_id, s.obtained_at))
        .collect();
    assert_eq!(
        owned,
        [
            ("char_002_amiya@epoque#4".to_owned(), None),
            ("char_003_kalts@own#1".to_owned(), Some(10))
        ]
    );

    let (total, rows) = roster::skin_popularity(&mut conn).await.unwrap();
    assert_eq!(total, 3);
    let mut counts: Vec<(String, i64)> = rows.into_iter().map(|r| (r.skin_id, r.owners)).collect();
    counts.sort();
    assert_eq!(
        counts,
        [
            ("char_002_amiya@epoque#4".to_owned(), 2),
            ("char_003_kalts@own#1".to_owned(), 1)
        ]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn users_with_roster_counts_accounts_not_operators(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert_eq!(roster::users_with_roster(&mut conn).await.unwrap(), 0);
    let a = common::user(&mut conn, 1).await;
    let b = common::user(&mut conn, 2).await;
    common::user(&mut conn, 3).await;
    sqlx::query(
        "INSERT INTO user_operators (user_id, operator_id) VALUES
         ($1, 'char_002_amiya'), ($1, 'char_003_kalts'), ($2, 'char_002_amiya')",
    )
    .bind(a.0)
    .bind(b.0)
    .execute(&mut *conn)
    .await
    .unwrap();

    assert_eq!(roster::users_with_roster(&mut conn).await.unwrap(), 2);
}
