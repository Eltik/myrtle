use dotenv::dotenv;
use std::env;
use std::path::Path;
use std::time::Instant;

use backend::database;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // ── Game data loading ───────────────────────────────────────────────
    let data_dir = Path::new("../assets/output/gamedata/excel");
    let assets_dir = Path::new("../assets/output");

    println!("Loading game data...");
    let start = Instant::now();

    match backend::core::gamedata::init_game_data(data_dir, assets_dir) {
        Ok(data) => {
            let elapsed = start.elapsed();
            println!("Loaded in {elapsed:.2?}");
            println!("  Operators: {}", data.operators.len());
            println!("  Skills: {}", data.skills.len());
            println!("  Modules: {}", data.modules.equip_dict.len());
            println!("  Skins: {}", data.skins.char_skins.len());
            println!("  Voices: {}", data.voices.char_words.len());
            println!("  Enemies: {}", data.enemies.enemy_data.len());
            println!("  Stages: {}", data.stages.len());
            println!("  Zones: {}", data.zones.len());
            println!("  Medals: {}", data.medals.medals.len());
            println!("  Chibis: {}", data.chibis.characters.len());
        }
        Err(e) => {
            eprintln!("Failed to load game data: {e}");
            std::process::exit(1);
        }
    }

    // ── Database init ───────────────────────────────────────────────────
    println!("\nInitializing database...");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = match database::init(&database_url).await {
        Ok(pool) => {
            println!("Database ready.");
            pool
        }
        Err(e) => {
            eprintln!("Database init failed: {e}");
            std::process::exit(1);
        }
    };

    // ── Load fixture and sync ───────────────────────────────────────────
    println!("\nLoading test fixture...");
    let fixture_path = Path::new("tests/fixtures/user_09525371.json");
    let fixture_str = match std::fs::read_to_string(fixture_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read fixture: {e}");
            std::process::exit(1);
        }
    };
    let fixture: serde_json::Value = serde_json::from_str(&fixture_str).unwrap();

    let uid = fixture["uid"].as_str().unwrap();
    let server_id = fixture["server_id"].as_i64().unwrap() as i16;
    let nickname = fixture["nickname"].as_str().unwrap();
    let level = fixture["level"].as_i64().unwrap() as i16;
    let avatar_id = fixture["avatar_id"].as_str();
    let secretary = fixture["secretary"].as_str();
    let secretary_skin_id = fixture["secretary_skin_id"].as_str();
    let resume_id = fixture["resume_id"].as_str();

    let checkin: Vec<i16> = fixture["checkin"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|n| n as i16))
                .collect()
        })
        .unwrap_or_default();

    println!("Syncing user {uid} ({nickname})...");
    let sync_start = Instant::now();

    match database::queries::roster::sync_user_data(
        &pool,
        uid,
        server_id,
        nickname,
        level,
        avatar_id,
        secretary,
        secretary_skin_id,
        resume_id,
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
    {
        Ok(()) => println!("Synced in {:.2?}", sync_start.elapsed()),
        Err(e) => {
            eprintln!("Sync failed: {e}");
            std::process::exit(1);
        }
    }

    // ── Query back and verify ───────────────────────────────────────────
    println!("\nQuerying data back...");

    // Find user by UID
    match database::queries::users::find_by_uid(&pool, uid).await {
        Ok(Some(profile)) => {
            println!(
                "  User: {} (level {})",
                profile.nickname.as_deref().unwrap_or("?"),
                profile.level.unwrap_or(0)
            );
            println!("  Server: {}", profile.server);
            println!("  Operators: {}", profile.operator_count.unwrap_or(0));
            println!("  Items: {}", profile.item_count.unwrap_or(0));
            println!("  Skins: {}", profile.skin_count.unwrap_or(0));
            println!("  Orundum: {}", profile.orundum.unwrap_or(0));
            println!("  LMD: {}", profile.lmd.unwrap_or(0));
        }
        Ok(None) => eprintln!("  User not found after sync!"),
        Err(e) => eprintln!("  Query failed: {e}"),
    }

    // Get roster
    match database::queries::users::find_raw_by_uid(&pool, uid, server_id).await {
        Ok(Some(user)) => {
            match database::queries::roster::get_roster(&pool, user.id).await {
                Ok(roster) => println!("  Roster: {} operators", roster.len()),
                Err(e) => eprintln!("  Roster query failed: {e}"),
            }

            match database::queries::items::get_inventory(&pool, user.id).await {
                Ok(items) => println!("  Inventory: {} items", items.len()),
                Err(e) => eprintln!("  Inventory query failed: {e}"),
            }
        }
        Ok(None) => eprintln!("  Raw user not found!"),
        Err(e) => eprintln!("  Raw user query failed: {e}"),
    }

    println!("\nDone.");
}
