//! Score synchronization CLI tool for Myrtle backend.
//!
//! Recalculates scores for all users based on their current user data.
//!
//! Usage:
//!   cargo run --bin sync-scores
//!   cargo run --bin sync-scores -- --dry-run
//!   cargo run --bin sync-scores -- --user <UID>
//!   cargo run --bin sync-scores -- --batch-size 50
//!
//! Environment Variables:
//!   DATABASE_URL - PostgreSQL connection string (required)
//!   DATA_DIR - Path to game data directory (required)
//!   ASSETS_DIR - Path to assets directory (optional, defaults to ./assets)
//!
//! Examples:
//!   # Sync all users
//!   DATA_DIR=/path/to/gamedata cargo run --bin sync-scores
//!
//!   # Dry run (don't save changes)
//!   DATA_DIR=/path/to/gamedata cargo run --bin sync-scores -- --dry-run
//!
//!   # Sync specific user
//!   DATA_DIR=/path/to/gamedata cargo run --bin sync-scores -- --user 12345678

use backend::core::local::handler::init_game_data;
use backend::core::local::types::GameData;
use backend::core::user::score::calculate_user_score;
use backend::core::user::types::User as UserData;
use backend::database::models::user::User as DbUser;
use backend::events::EventEmitter;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "sync-scores")]
#[command(about = "Recalculate scores for all users based on their user data")]
struct Cli {
    /// Don't actually save changes to the database
    #[arg(long)]
    dry_run: bool,

    /// Only sync a specific user by UID
    #[arg(long)]
    user: Option<String>,

    /// Number of users to process in each batch
    #[arg(long, default_value = "100")]
    batch_size: i64,

    /// Show detailed progress for each user
    #[arg(short, long)]
    verbose: bool,

    /// Path to game data directory (overrides DATA_DIR env var)
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Path to assets directory (overrides ASSETS_DIR env var)
    #[arg(long)]
    assets_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    println!();
    println!("========================================");
    println!("    Myrtle Score Sync Tool");
    println!("========================================");
    println!();

    // Load game data
    let data_dir = cli
        .data_dir
        .or_else(|| std::env::var("DATA_DIR").ok().map(PathBuf::from))
        .expect("DATA_DIR environment variable or --data-dir argument must be set");

    let assets_dir = cli
        .assets_dir
        .or_else(|| std::env::var("ASSETS_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("./assets"));

    println!("Loading game data from: {}", data_dir.display());
    println!("Assets directory: {}", assets_dir.display());

    let events = Arc::new(EventEmitter::new());
    let game_data = init_game_data(&data_dir, &assets_dir, &events)?;

    println!(
        "Game data loaded: {} operators, {} stages",
        game_data.operators.len(),
        game_data.stages.len()
    );
    println!();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Connected to database");
    println!();

    if cli.dry_run {
        println!("*** DRY RUN MODE - No changes will be saved ***");
        println!();
    }

    // Run sync
    if let Some(uid) = cli.user {
        sync_single_user(&pool, &game_data, &uid, cli.dry_run, cli.verbose).await?;
    } else {
        sync_all_users(&pool, &game_data, cli.batch_size, cli.dry_run, cli.verbose).await?;
    }

    Ok(())
}

async fn sync_single_user(
    pool: &sqlx::PgPool,
    game_data: &GameData,
    uid: &str,
    dry_run: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Syncing score for user: {}", uid);
    println!();

    let user = DbUser::find_by_uid(pool, uid)
        .await?
        .ok_or_else(|| format!("User not found: {}", uid))?;

    match process_user(pool, game_data, &user, dry_run, verbose).await {
        Ok(result) => {
            println!("User {} synced successfully", uid);
            println!("  Old score: {:.2}", result.old_score);
            println!("  New score: {:.2}", result.new_score);
            println!("  Grade: {:?}", result.grade);
        }
        Err(e) => {
            eprintln!("Failed to sync user {}: {}", uid, e);
        }
    }

    Ok(())
}

async fn sync_all_users(
    pool: &sqlx::PgPool,
    game_data: &GameData,
    batch_size: i64,
    dry_run: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Count total users
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    println!("Total users to sync: {}", total_users);
    println!("Batch size: {}", batch_size);
    println!();

    let start_time = Instant::now();
    let mut processed = 0i64;
    let mut successful = 0i64;
    let mut failed = 0i64;
    let mut skipped = 0i64;
    let mut offset = 0i64;

    loop {
        let users = DbUser::find_all(pool, batch_size, offset).await?;

        if users.is_empty() {
            break;
        }

        for user in &users {
            processed += 1;

            let nickname = get_nickname(&user.data);

            match process_user(pool, game_data, user, dry_run, verbose).await {
                Ok(result) => {
                    successful += 1;
                    if verbose {
                        println!(
                            "[{}/{}] {} ({}) - Score: {:.2} -> {:.2} (Grade: {:?})",
                            processed,
                            total_users,
                            nickname,
                            user.uid,
                            result.old_score,
                            result.new_score,
                            result.grade
                        );
                    } else if processed % 10 == 0 || processed == total_users {
                        print_progress(processed, total_users, successful, failed, skipped);
                    }
                }
                Err(e) => {
                    if e.to_string().contains("missing field")
                        || e.to_string().contains("invalid type")
                    {
                        skipped += 1;
                        if verbose {
                            println!(
                                "[{}/{}] {} ({}) - SKIPPED (invalid data)",
                                processed, total_users, nickname, user.uid
                            );
                        }
                    } else {
                        failed += 1;
                        eprintln!(
                            "[{}/{}] {} ({}) - FAILED: {}",
                            processed, total_users, nickname, user.uid, e
                        );
                    }
                }
            }
        }

        offset += batch_size;
    }

    let elapsed = start_time.elapsed();

    println!();
    println!("========================================");
    println!("          SYNC COMPLETE");
    println!("========================================");
    println!();
    println!("Total processed: {}", processed);
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    println!("  Skipped (invalid data): {}", skipped);
    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());
    println!(
        "Average: {:.2}ms per user",
        elapsed.as_millis() as f64 / processed.max(1) as f64
    );

    if dry_run {
        println!();
        println!("*** DRY RUN - No changes were saved ***");
    }

    Ok(())
}

struct SyncResult {
    old_score: f32,
    new_score: f32,
    grade: backend::core::user::score::Grade,
}

async fn process_user(
    pool: &sqlx::PgPool,
    game_data: &GameData,
    db_user: &DbUser,
    dry_run: bool,
    _verbose: bool,
) -> Result<SyncResult, Box<dyn std::error::Error>> {
    // Parse user data from JSON
    let user_data: UserData = serde_json::from_value(db_user.data.clone())?;

    // Calculate new score
    let score = calculate_user_score(&user_data, game_data);

    // Get old score for comparison
    let old_score = db_user
        .score
        .get("totalScore")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let new_score = score.total_score;
    let grade = score.grade.grade;

    // Save to database (unless dry run)
    if !dry_run {
        let score_json = serde_json::to_value(&score)?;
        DbUser::update_score(pool, db_user.id, &score_json).await?;
    }

    Ok(SyncResult {
        old_score,
        new_score,
        grade,
    })
}

fn get_nickname(data: &serde_json::Value) -> String {
    data.get("status")
        .and_then(|s| s.get("nickName"))
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn print_progress(processed: i64, total: i64, successful: i64, failed: i64, skipped: i64) {
    let percentage = (processed as f64 / total as f64) * 100.0;
    print!(
        "\rProgress: {}/{} ({:.1}%) | Success: {} | Failed: {} | Skipped: {}",
        processed, total, percentage, successful, failed, skipped
    );
    use std::io::Write;
    std::io::stdout().flush().ok();
    if processed == total {
        println!();
    }
}
