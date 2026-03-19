//! Integration tests for the database layer.
//! Run with: cargo test --test db_integration -- --test-threads=1
//! Requires a local PostgreSQL instance.

use backend::database;
use sqlx::PgPool;
use std::time::Instant;

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432".to_string())
}

fn load_fixture() -> serde_json::Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/user_09525371.json");
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture at {}: {e}", path.display()));
    serde_json::from_str(&data).expect("Failed to parse fixture")
}

fn extract_checkin(fixture: &serde_json::Value) -> Vec<i16> {
    fixture["checkin"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|n| n as i16))
                .collect()
        })
        .unwrap_or_default()
}

async fn init_db() -> PgPool {
    let pool = database::pool::create_pool(&database_url())
        .await
        .expect("Failed to create pool");

    // Reset schema
    sqlx::raw_sql("DROP SCHEMA IF EXISTS public CASCADE; CREATE SCHEMA public;")
        .execute(&pool)
        .await
        .expect("Failed to reset schema");

    // Run migrations
    database::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Seed servers
    use backend::core::hypergryph::constants::Server;
    for &server in Server::all() {
        sqlx::query(
            "INSERT INTO servers (id, code, name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(server.index() as i16)
        .bind(server.as_str())
        .bind(server.display_name())
        .execute(&pool)
        .await
        .unwrap();
    }

    pool
}

async fn sync_fixture_user(pool: &PgPool) {
    let fixture = load_fixture();
    let checkin = extract_checkin(&fixture);

    database::queries::roster::sync_user_data(
        pool,
        fixture["uid"].as_str().unwrap(),
        fixture["server_id"].as_i64().unwrap() as i16,
        fixture["nickname"].as_str().unwrap(),
        fixture["level"].as_i64().unwrap() as i16,
        fixture["avatar_id"].as_str(),
        fixture["secretary"].as_str(),
        fixture["secretary_skin_id"].as_str(),
        fixture["resume_id"].as_str(),
        &fixture["operators"],
        &fixture["skills"],
        &fixture["modules"],
        &fixture["items"],
        &fixture["skins"],
        &fixture["status"],
        &fixture["stages"],
        &fixture["roguelike"],
        &fixture["sandbox"],
        &fixture["medals"],
        &fixture["building"],
        &checkin,
    )
    .await
    .expect("Failed to sync user data");
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 1: sp_sync_user_data with real game data shape
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_sync_user_data() {
    let pool = init_db().await;
    sync_fixture_user(&pool).await;

    let profile = database::queries::users::find_by_uid(&pool, "09525371")
        .await
        .unwrap()
        .expect("User should exist");

    assert_eq!(profile.nickname.as_deref(), Some("Eltik"));
    assert_eq!(profile.level, Some(120));
    assert_eq!(profile.server, "en");
    assert_eq!(profile.operator_count, Some(324));
    assert_eq!(profile.item_count, Some(82));
    assert_eq!(profile.skin_count, Some(128));
    assert_eq!(profile.orundum, Some(41745));
    assert_eq!(profile.lmd, Some(2464420));
    assert_eq!(profile.secretary.as_deref(), Some("char_180_amgoat"));
    assert_eq!(
        profile.secretary_skin_id.as_deref(),
        Some("char_180_amgoat@sanrio#2")
    );

    // Re-sync should be idempotent
    sync_fixture_user(&pool).await;

    let profile2 = database::queries::users::find_by_uid(&pool, "09525371")
        .await
        .unwrap()
        .expect("User should still exist");
    assert_eq!(profile2.operator_count, Some(324));
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 2: v_user_roster returns correct mastery/module aggregation
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_roster_view_aggregation() {
    let pool = init_db().await;
    sync_fixture_user(&pool).await;

    let user = database::queries::users::find_raw_by_uid(&pool, "09525371", 0)
        .await
        .unwrap()
        .expect("User should exist");

    let roster = database::queries::roster::get_roster(&pool, user.id)
        .await
        .unwrap();

    assert_eq!(roster.len(), 324);

    // char_340_shwaz: E2 Lv60, M3 on skill 3, module level 3
    let shwaz = roster
        .iter()
        .find(|r| r.operator_id == "char_340_shwaz")
        .expect("char_340_shwaz should be in roster");

    assert_eq!(shwaz.elite, 2);
    assert_eq!(shwaz.level, 60);
    assert_eq!(shwaz.skill_level, 7);
    assert_eq!(shwaz.potential, 1);
    assert_eq!(shwaz.skin_id.as_deref(), Some("char_340_shwaz@striker#1"));
    assert_eq!(shwaz.current_equip.as_deref(), Some("uniequip_003_shwaz"));

    // Masteries should be a JSON array with at least one M3
    let masteries = shwaz
        .masteries
        .as_array()
        .expect("masteries should be array");
    assert!(!masteries.is_empty());
    let has_m3 = masteries.iter().any(|m| m["mastery"].as_i64() == Some(3));
    assert!(has_m3, "char_340_shwaz should have an M3 mastery");

    // Modules should contain a level 3 module
    let modules = shwaz.modules.as_array().expect("modules should be array");
    assert!(!modules.is_empty());
    let has_max_module = modules.iter().any(|m| m["level"].as_i64() == Some(3));
    assert!(
        has_max_module,
        "char_340_shwaz should have a level 3 module"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 3: v_leaderboard ranks correctly
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_leaderboard_ranking() {
    let pool = init_db().await;
    sync_fixture_user(&pool).await;

    let user = database::queries::users::find_raw_by_uid(&pool, "09525371", 0)
        .await
        .unwrap()
        .expect("User should exist");

    // Write a score
    let score = database::models::score::UserScore {
        user_id: user.id,
        total_score: 262820.78,
        operator_score: 223211.28,
        stage_score: 2954.0,
        roguelike_score: 9805.0,
        sandbox_score: 2709.0,
        medal_score: 11654.5,
        base_score: 12487.0,
        skin_score: 0.0,
        grade: Some("S".to_string()),
        composite_score: 92.91,
        breakdown: Some(serde_json::json!({"test": true})),
        calculated_at: chrono::Utc::now(),
    };

    database::queries::score::update_score(&pool, &score)
        .await
        .unwrap();

    // Global leaderboard
    let board = database::queries::score::get_leaderboard(&pool, None, "total_score", 10, 0)
        .await
        .unwrap();

    assert_eq!(board.len(), 1);
    assert_eq!(board[0].uid, "09525371");
    assert!((board[0].total_score.unwrap() - 262820.78).abs() < 0.01);
    assert_eq!(board[0].grade.as_deref(), Some("S"));
    assert_eq!(board[0].rank_global, Some(1));
    assert_eq!(board[0].rank_server, Some(1));

    // Filter by server
    let en = database::queries::score::get_leaderboard(&pool, Some("en"), "total_score", 10, 0)
        .await
        .unwrap();
    assert_eq!(en.len(), 1);

    let jp = database::queries::score::get_leaderboard(&pool, Some("jp"), "total_score", 10, 0)
        .await
        .unwrap();
    assert_eq!(jp.len(), 0);

    // Count
    let count = database::queries::score::count_leaderboard(&pool, None)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // Sort by operator_score
    let by_op = database::queries::score::get_leaderboard(&pool, None, "operator_score", 10, 0)
        .await
        .unwrap();
    assert!((by_op[0].operator_score.unwrap() - 223211.28).abs() < 0.01);
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 4: audit_log populates on tier list changes
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_audit_log_on_tier_list_changes() {
    let pool = init_db().await;
    sync_fixture_user(&pool).await;

    let user = database::queries::users::find_raw_by_uid(&pool, "09525371", 0)
        .await
        .unwrap()
        .expect("User should exist");

    // Create a tier list
    let tier_list = database::queries::tier_lists::create(
        &pool,
        "Test Tier List",
        "test-tier-list",
        Some("A test"),
        "official",
        user.id,
    )
    .await
    .unwrap();

    assert_eq!(tier_list.name, "Test Tier List");
    assert!(tier_list.is_active);

    // Audit log should have INSERT
    let inserts: Vec<database::models::audit::AuditLogEntry> = sqlx::query_as(
        "SELECT * FROM audit_log WHERE table_name = 'tier_lists' AND action = 'INSERT'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(!inserts.is_empty(), "Should have INSERT audit entry");
    assert_eq!(inserts[0].record_id, tier_list.id.to_string());

    // Soft delete
    database::queries::tier_lists::soft_delete(&pool, tier_list.id)
        .await
        .unwrap();

    // Audit log should have UPDATE
    let updates: Vec<database::models::audit::AuditLogEntry> = sqlx::query_as(
        "SELECT * FROM audit_log WHERE table_name = 'tier_lists' AND action = 'UPDATE'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(!updates.is_empty(), "Should have UPDATE audit entry");
    let old = updates[0].old_data.as_ref().unwrap();
    let new = updates[0].new_data.as_ref().unwrap();
    assert_eq!(old["is_active"], serde_json::json!(true));
    assert_eq!(new["is_active"], serde_json::json!(false));

    // Soft-deleted tier list should not be found
    let found = database::queries::tier_lists::find_by_slug(&pool, "test-tier-list")
        .await
        .unwrap();
    assert!(found.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 5: Benchmark leaderboard with many users
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn bench_leaderboard_with_many_users() {
    let pool = init_db().await;
    sync_fixture_user(&pool).await;

    let start = Instant::now();
    let num_users: i64 = 1000;

    for i in 0..num_users {
        let uid = format!("bench_{i:05}");
        let nickname = format!("User{i}");

        let user_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (uid, server_id, nickname, level, role) VALUES ($1, $2, $3, $4, 'user') RETURNING id",
        )
        .bind(&uid)
        .bind((i % 6) as i16)
        .bind(&nickname)
        .bind((i % 120 + 1) as i16)
        .fetch_one(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, true)")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO user_scores (user_id, total_score, operator_score, stage_score, roguelike_score, sandbox_score, medal_score, base_score, skin_score, grade, composite_score) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(user_id)
        .bind(i as f64 * 100.0)
        .bind(i as f64 * 60.0)
        .bind(i as f64 * 10.0)
        .bind(i as f64 * 15.0)
        .bind(i as f64 * 5.0)
        .bind(i as f64 * 8.0)
        .bind(i as f64 * 2.0)
        .bind(0.0_f64)
        .bind("A")
        .bind(i as f64 * 0.1)
        .execute(&pool)
        .await
        .unwrap();
    }

    let insert_elapsed = start.elapsed();
    println!("Inserted {num_users} users in {insert_elapsed:.2?}");

    // Global leaderboard top 50
    let q_start = Instant::now();
    let board = database::queries::score::get_leaderboard(&pool, None, "total_score", 50, 0)
        .await
        .unwrap();
    let q_elapsed = q_start.elapsed();
    println!("Leaderboard top 50 of {}: {q_elapsed:.2?}", num_users + 1);
    assert_eq!(board.len(), 50);
    assert!(board[0].total_score.unwrap() > board[1].total_score.unwrap());
    // Top scored user should be rank 1 or 2 (fixture user may also be on leaderboard)
    assert!(board[0].rank_global.unwrap() <= 2);

    // Server-filtered
    let s_start = Instant::now();
    let en = database::queries::score::get_leaderboard(&pool, Some("en"), "total_score", 50, 0)
        .await
        .unwrap();
    let s_elapsed = s_start.elapsed();
    println!("EN leaderboard: {s_elapsed:.2?} ({} results)", en.len());

    // Count
    let c_start = Instant::now();
    let total = database::queries::score::count_leaderboard(&pool, None)
        .await
        .unwrap();
    let c_elapsed = c_start.elapsed();
    println!("Count: {c_elapsed:.2?} ({total} total)");
    assert_eq!(total, num_users + 1);

    assert!(
        q_elapsed.as_millis() < 500,
        "Leaderboard too slow: {q_elapsed:?}"
    );
}
