//! Gacha record export CLI tool for Myrtle backend.
//!
//! Exports a user's gacha records to a JSON file by Arknights UID or internal UUID.
//!
//! Usage:
//!   cargo run --bin export-gacha -- <USER>
//!   cargo run --bin export-gacha -- <USER> --output pulls.json
//!   cargo run --bin export-gacha -- <USER> --rarity 6
//!   cargo run --bin export-gacha -- <USER> --gacha-type limited
//!
//! Examples:
//!   # Export by Arknights UID
//!   cargo run --bin export-gacha -- 12345678
//!
//!   # Export by internal UUID
//!   cargo run --bin export-gacha -- "a1b2c3d4-e5f6-..."
//!
//!   # Export only 6-star pulls to a specific file
//!   cargo run --bin export-gacha -- 12345678 --rarity 6 --output six_stars.json
//!
//!   # Export limited banner pulls only
//!   cargo run --bin export-gacha -- 12345678 --gacha-type limited

use backend::database::models::gacha::GachaRecord;
use backend::database::models::user::User as DbUser;
use clap::Parser;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "export-gacha")]
#[command(about = "Export a user's gacha records to JSON")]
struct Cli {
    /// Arknights UID or internal UUID of the user
    user: String,

    /// Output file path (defaults to gacha_export_{uid}.json)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Filter by rarity (3-6)
    #[arg(long)]
    rarity: Option<i16>,

    /// Filter by gacha type (limited, regular, special)
    #[arg(long)]
    gacha_type: Option<String>,

    /// Compact JSON output (no pretty-printing)
    #[arg(long)]
    compact: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    // Resolve user: try UUID first, then fall back to Arknights UID
    let user = if let Ok(uuid) = cli.user.parse::<Uuid>() {
        DbUser::find_by_id(&pool, uuid).await?
    } else {
        DbUser::find_by_uid(&pool, &cli.user).await?
    };

    let user = user.ok_or_else(|| format!("User not found: {}", cli.user))?;

    let nickname = user
        .data
        .get("status")
        .and_then(|s| s.get("nickName"))
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown");

    println!("User: {} (UID: {}, ID: {})", nickname, user.uid, user.id);

    // Fetch records
    let records = GachaRecord::get_user_history(
        &pool,
        user.id,
        i64::MAX, // fetch all
        0,
        cli.rarity,
        cli.gacha_type.as_deref(),
        None, // no char_id filter
        None, // no from timestamp
        None, // no to timestamp
        true, // descending (newest first)
    )
    .await?;

    println!("Records found: {}", records.len());

    if records.is_empty() {
        println!("No gacha records to export.");
        return Ok(());
    }

    // Determine output path
    let output = cli
        .output
        .unwrap_or_else(|| PathBuf::from(format!("gacha_export_{}.json", user.uid)));

    // Serialize
    let json = if cli.compact {
        serde_json::to_string(&records)?
    } else {
        serde_json::to_string_pretty(&records)?
    };

    tokio::fs::write(&output, &json).await?;

    println!("Exported to: {}", output.display());

    Ok(())
}
