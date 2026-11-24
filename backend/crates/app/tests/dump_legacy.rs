#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use myrtle_app::dump::{self, DumpError, ImportOptions, MANIFEST_FILE, TABLES};
use myrtle_db::tools;
use myrtle_testkit::db::database;
use serde_json::{Value, json};

const DOC: &str = "11111111-1111-4111-8111-111111111111";
const PLAIN: &str = "22222222-2222-4222-8222-222222222222";
const LIST: &str = "33333333-3333-4333-8333-333333333331";
const DELETED_LIST: &str = "33333333-3333-4333-8333-333333333332";
const TIER: &str = "44444444-4444-4444-8444-444444444441";
const ORPHAN_TIER: &str = "44444444-4444-4444-8444-444444444442";
const NOTE: &str = "55555555-5555-4555-8555-555555555551";
const AT: &str = "2025-06-01T12:00:00+00:00";

fn doc_data() -> Value {
    json!({
        "status": {
            "nickName": "Doc", "nickNumber": "1234", "level": 120,
            "avatar": {"type": "ICON", "id": "avatar_def_01"}, "secretary": "char_002_amiya",
            "secretarySkinId": "char_002_amiya#2", "resume": "char_002_amiya",
            "exp": 10, "diamondShard": 3000, "gold": 1_000_000, "ap": 100, "maxAp": 135,
            "gachaTicket": 4, "tenGachaTicket": 1, "recruitLicense": 50, "socialPoint": 200,
            "hggShard": 7, "lggShard": 900, "practiceTicket": 3, "registerTs": 1_600_000_000,
            "lastOnlineTs": 1_700_000_000, "mainStageProgress": "main_14-01", "friendNumLimit": 50
        },
        "troop": {"chars": {
            "1": {
                "charId": "char_002_amiya", "evolvePhase": 2, "level": 80, "exp": 0,
                "potentialRank": 5, "mainSkillLvl": 7, "favorPoint": 25570,
                "skin": "char_002_amiya#2", "defaultSkillIndex": 2, "voiceLan": "JP",
                "currentEquip": "uniequip_002_amiya", "gainTime": 1_600_000_000,
                "skills": [{"specializeLevel": 3}, {"specializeLevel": 0}, {"specializeLevel": 2}],
                "equip": {
                    "uniequip_001_amiya": {"level": 1, "locked": 0},
                    "uniequip_002_amiya": {"level": 3, "locked": 1}
                }
            },
            "2": {"charId": "char_002_amiya", "level": 1},
            "3": {"charId": "char_003_kalts", "evolvePhase": 1, "level": 50},
            "4": {"level": 9}
        }},
        "inventory": {"30011": {"amount": 12}, "30012": 4, "30013": {"amount": 0}},
        "skin": {"characterSkins": {"char_002_amiya#2": {"obtainTime": 1_650_000_000}, "char_003_kalts#1": {}}},
        "dungeon": {"stages": {"main_00-01": {"completeTimes": 1}}},
        "rlv2": {"outer": {"rogue_1": {"bp": 3}, "rogue_2": {"bp": 1}}},
        "deepSea": {"places": {}},
        "medal": {"medals": {"medal_a": {"val": [[1, 2]], "fts": 10, "rts": 20}}},
        "building": {"rooms": {}},
        "checkIn": {"checkInHistory": [1, 0, 1]}
    })
}

fn write_backup(dir: &Path) {
    let files = [
        (
            "tier_lists.json",
            json!([
                {"id": LIST, "name": "Main", "slug": "main", "description": "d",
                 "tier_list_type": "official", "created_by": DOC, "is_active": true,
                 "created_at": AT, "updated_at": AT},
                {"id": DELETED_LIST, "name": "Gone", "slug": "gone", "is_deleted": true,
                 "created_at": AT, "updated_at": AT}
            ]),
        ),
        (
            "tiers.json",
            json!([
                {"id": TIER, "tier_list_id": LIST, "name": "S", "display_order": 0, "color": "#ff0000"},
                {"id": ORPHAN_TIER, "tier_list_id": DELETED_LIST, "name": "A", "display_order": 1}
            ]),
        ),
        (
            "tier_placements.json",
            json!([
                {"id": "p1", "tier_id": TIER, "operator_id": "char_002_amiya", "sub_order": 1,
                 "notes": "best", "updated_at": AT},
                {"id": "p2", "tier_id": TIER, "operator_id": "char_002_amiya", "sub_order": 2,
                 "updated_at": AT},
                {"id": "p3", "tier_id": ORPHAN_TIER, "operator_id": "char_003_kalts",
                 "updated_at": AT}
            ]),
        ),
        (
            "operator_notes.json",
            json!([{"id": NOTE, "operator_id": "char_002_amiya", "pros": "p", "cons": "c",
                    "notes": "n", "trivia": "t", "summary": "s",
                    "created_at": AT, "updated_at": AT}]),
        ),
        (
            "operator_notes_audit_log.json",
            json!([
                {"operator_id": "char_002_amiya", "field_name": "pros", "old_value": "",
                 "new_value": "p", "changed_by": DOC, "changed_at": AT},
                {"operator_id": "char_999_gone", "field_name": "pros", "changed_at": AT}
            ]),
        ),
        (
            "user_gacha_settings.json",
            json!([{"user_id": PLAIN, "store_records": false, "share_anonymous_stats": false}]),
        ),
        (
            "users.json",
            json!([
                {"id": DOC, "uid": "1001", "server": "CN", "role": "admin", "data": doc_data(),
                 "settings": {"public_profile": false},
                 "score": {"totalScore": 12.5, "operatorScore": 10.0, "stageScore": 2.5,
                           "grade": {"grade": "A", "calculatedAt": 1_700_000_000}},
                 "created_at": AT, "updated_at": AT},
                {"id": PLAIN, "uid": "1002", "server": "bili", "data": {},
                 "created_at": AT, "updated_at": AT},
                {"id": "66666666-6666-4666-8666-666666666666", "data": {}}
            ]),
        ),
        (
            "gacha_records.json",
            json!([
                {"user_id": DOC, "char_id": "char_002_amiya", "pool_id": "NORM_1", "rarity": 5,
                 "pull_timestamp": 1_700_000_000, "pool_name": "Standard", "gacha_type": "normal",
                 "created_at": AT},
                {"user_id": DOC, "char_id": "char_002_amiya", "pool_id": "NORM_1", "rarity": 5,
                 "pull_timestamp": 1_700_000_000, "created_at": AT},
                {"user_id": DOC, "char_id": "char_003_kalts", "pool_id": "NORM_1", "rarity": 5,
                 "pull_timestamp": 1_700_000_000, "created_at": AT},
                {"user_id": "77777777-7777-4777-8777-777777777777", "char_id": "x", "pool_id": "y",
                 "pull_timestamp": 1, "created_at": AT}
            ]),
        ),
    ];
    for (name, value) in files {
        std::fs::write(
            dir.join(name),
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }
}

fn rows(dir: &Path, table: &str) -> Vec<Value> {
    std::fs::read_to_string(dir.join(format!("{table}.jsonl")))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

fn translated() -> (tempfile::TempDir, tempfile::TempDir) {
    let (backup, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    write_backup(backup.path());
    dump::translate(backup.path(), out.path()).unwrap();
    (backup, out)
}

#[test]
fn the_manifest_lists_every_table_with_its_row_count() {
    let (_backup, out) = translated();
    let manifest = dump::read_manifest(out.path()).unwrap();
    dump::check_manifest(&manifest).unwrap();
    assert_eq!(manifest.database_version, "translated-from-old-myrtle");
    let names: Vec<&str> = manifest.tables.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, TABLES);
    for t in &manifest.tables {
        assert_eq!(rows(out.path(), &t.name).len() as u64, t.rows, "{}", t.name);
    }
    let counts: BTreeMap<&str, u64> = manifest
        .tables
        .iter()
        .map(|t| (t.name.as_str(), t.rows))
        .collect();
    assert_eq!(counts["servers"], 0, "the seed provides servers");
    assert_eq!(counts["users"], 2);
    assert_eq!(counts["tier_lists"], 1);
    assert_eq!(counts["tiers"], 1);
    assert_eq!(counts["tier_placements"], 1);
    assert_eq!(counts["operator_notes_audit_log"], 1);
    assert_eq!(counts["gacha_records"], 2);
    assert_eq!(counts["user_operators"], 2);
    assert_eq!(counts["user_operator_skills"], 3);
    assert_eq!(counts["user_operator_modules"], 1);
    assert_eq!(counts["user_items"], 2);
    assert_eq!(counts["user_skins"], 2);
    assert_eq!(counts["user_roguelike_progress"], 2);
    assert_eq!(counts["user_medals"], 1);
    assert_eq!(counts["user_scores"], 1);
    for t in [
        "user_status",
        "user_settings",
        "user_stage_progress",
        "user_sandbox_progress",
        "user_building",
        "user_checkin",
    ] {
        assert_eq!(counts[t], 2, "{t}");
    }
    assert!(out.path().join(MANIFEST_FILE).is_file());
    for t in ["data_generations", "service_account_sessions", "jobs"] {
        assert!(!counts.contains_key(t), "{t}");
        assert!(!out.path().join(format!("{t}.jsonl")).exists(), "{t}");
    }
}

#[test]
fn a_user_blob_becomes_its_normalized_rows() {
    let (_backup, out) = translated();
    let users = rows(out.path(), "users");
    assert_eq!(users[0]["server_id"], 3);
    assert_eq!(users[0]["nickname"], "Doc");
    assert_eq!(users[0]["avatar_id"], "avatar_def_01");
    assert_eq!(users[0]["role"], "admin");
    assert_eq!(users[1]["server_id"], 4);
    assert_eq!(users[1]["nickname"], "");
    assert_eq!(users[1]["role"], "user");

    let ops = rows(out.path(), "user_operators");
    assert_eq!(ops[0]["operator_id"], "char_002_amiya");
    assert_eq!(ops[0]["elite"], 2);
    assert_eq!(ops[0]["default_skill"], 2);
    assert_eq!(ops[1]["operator_id"], "char_003_kalts");
    assert_eq!(ops[1]["skill_level"], 1);
    let modules = rows(out.path(), "user_operator_modules");
    assert_eq!(modules[0]["module_id"], "uniequip_002_amiya");
    assert_eq!(modules[0]["locked"], true);
    let skills = rows(out.path(), "user_operator_skills");
    assert_eq!(skills[2]["skill_index"], 2);
    assert_eq!(skills[2]["specialize_level"], 2);

    let items = rows(out.path(), "user_items");
    let quantities: Vec<(&str, i64)> = items
        .iter()
        .map(|i| {
            (
                i["item_id"].as_str().unwrap(),
                i["quantity"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(quantities, [("30011", 12), ("30012", 4)]);

    let status = rows(out.path(), "user_status");
    assert_eq!(status[0]["orundum"], 3000);
    assert_eq!(status[0]["lmd"], 1_000_000);
    assert_eq!(status[0]["main_stage_progress"], "main_14-01");
    assert_eq!(status[1]["lmd"], 0);

    let settings = rows(out.path(), "user_settings");
    assert_eq!(
        (&settings[0]["public_profile"], &settings[0]["store_gacha"]),
        (&json!(false), &json!(true))
    );
    assert_eq!(
        (&settings[1]["store_gacha"], &settings[1]["share_stats"]),
        (&json!(false), &json!(false))
    );

    assert_eq!(
        rows(out.path(), "user_sandbox_progress")[0]["progress"],
        json!({"places": {}})
    );
    assert_eq!(
        rows(out.path(), "user_checkin")[0]["history"],
        json!([1, 0, 1])
    );
    assert_eq!(rows(out.path(), "user_checkin")[1]["history"], json!([]));
    let score = &rows(out.path(), "user_scores")[0];
    assert_eq!(score["total_score"], 12.5);
    assert_eq!(score["grade"], "A");
    assert_eq!(score["calculated_at"], "2023-11-14T22:13:20+00:00");

    let placement = &rows(out.path(), "tier_placements")[0];
    assert_eq!(placement["sub_order"], 1, "first sighting wins");
    assert_eq!(placement["description"], "best");
    assert_eq!(
        rows(out.path(), "operator_notes_audit_log")[0]["note_id"],
        NOTE
    );
    let pulls = rows(out.path(), "gacha_records");
    assert_eq!(pulls[0]["id"], 1);
    assert_eq!(pulls[1]["char_id"], "char_003_kalts");
}

#[test]
fn a_missing_backup_file_is_an_error() {
    let (backup, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    write_backup(backup.path());
    std::fs::remove_file(backup.path().join("gacha_records.json")).unwrap();
    assert!(matches!(
        dump::translate(backup.path(), out.path()),
        Err(DumpError::Io { .. })
    ));
    assert!(!out.path().join(MANIFEST_FILE).exists());
}

#[tokio::test]
async fn a_translated_backup_imports_into_a_migrated_database() {
    let (_backup, out) = translated();
    for truncate in [false, true] {
        let target = database().await;
        sqlx::query("INSERT INTO data_generations (server_id, generation) VALUES (0, 9)")
            .execute(&mut *target.db.conn().await.unwrap())
            .await
            .unwrap();
        let options = ImportOptions {
            truncate,
            ..ImportOptions::default()
        };
        let report = dump::import(&target.db, out.path(), options).await.unwrap();
        assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
        assert!(report.kept_seeded.contains(&"servers"));
        assert!(
            report.cascaded.iter().all(|t| t == "oauth_codes"),
            "{:?}",
            report.cascaded
        );
        let mut conn = target.db.conn().await.unwrap();
        for (name, expected) in [
            ("data_generations", 1),
            ("servers", 6),
            ("users", 2),
            ("gacha_records", 2),
            ("locales", 1),
        ] {
            let table = tools::table(&mut conn, name).await.unwrap().unwrap();
            assert_eq!(
                tools::count_rows(&mut conn, &table).await.unwrap(),
                expected,
                "{name}"
            );
        }
        let kinds: Vec<String> = sqlx::query_scalar("SELECT unnest(entity_kinds) FROM tier_lists")
            .fetch_all(&mut *conn)
            .await
            .unwrap();
        assert_eq!(kinds, ["operator"]);
        let defaulted: (i64, i64, i64) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM user_settings WHERE NOT public_gacha),
                    (SELECT count(*) FROM gacha_records WHERE batch_index = 0),
                    (SELECT count(*) FROM tier_lists WHERE is_listed)",
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap();
        assert_eq!(
            defaulted,
            (2, 2, 1),
            "columns the backup predates take their defaults"
        );
        drop(conn);
        target.close().await;
    }
}
