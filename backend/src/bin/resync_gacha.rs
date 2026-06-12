//! Reconcile every `gacha_records.rarity` against the current game data.
//!
//! Same logic as the post-hot-reload background task in `core::asset_watcher`,
//! but invokable directly. Useful for one-shots - e.g. after restoring from a
//! backup, after a manual gamedata patch, or to verify the resync logic on a
//! given snapshot.
//!
//! Usage:
//!   cargo run --release --bin resync-gacha
//!   cargo run --release --bin resync-gacha -- --dry-run

use std::{path::Path, time::Instant};

use anyhow::{Context, Result};
use backend::app::state::{default_bin_server_from_env, derive_assets_dir, derive_game_data_dir};
use backend::core::{gacha_resync::reconcile_rarities, gamedata::init_game_data};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

struct Args {
    dry_run: bool,
}

fn parse_args() -> Args {
    let mut a = Args { dry_run: false };
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--dry-run" => a.dry_run = true,
            "-h" | "--help" => {
                eprintln!(
                    "Usage: resync-gacha [--dry-run]\n\
                     \n\
                     Rewrites gacha_records.rarity to match the canonical value\n\
                     from the loaded game data (character_table). Char_ids not\n\
                     present in game data are left untouched."
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }
    a
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "resync_gacha=info,backend=warn".into()),
        )
        .init();

    let args = parse_args();

    // ASSETS_DIR is a base dir; the server's data loads from `{base}/{server}`
    // (+ `/gamedata/excel`). The game-data server is BIN_SERVER/SERVERS-driven.
    let base = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());
    let server = default_bin_server_from_env();
    let data_dir = derive_game_data_dir(&base, server);
    let assets_dir = derive_assets_dir(&base, server);

    tracing::info!("loading game data...");
    let t0 = Instant::now();
    let game_data = init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
        .context("failed to load game data")?;
    tracing::info!(
        operators = game_data.operators.len(),
        elapsed_s = format!("{:.2}", t0.elapsed().as_secs_f64()),
        "game data loaded",
    );

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    if args.dry_run {
        // Mirror reconcile_rarities' scan, but report the deltas instead of
        // issuing the UPDATE. Keeps the bin self-contained so we don't need a
        // second public entry point on the module.
        let pairs: Vec<(String, i16)> =
            sqlx::query_as("SELECT DISTINCT char_id, rarity FROM gacha_records")
                .fetch_all(&pool)
                .await?;
        let mut would_fix: Vec<(String, i16, i16)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (char_id, stored) in &pairs {
            let Some(canonical) = game_data
                .operators
                .get(char_id)
                .map(|op| op.rarity.to_star_int())
            else {
                continue;
            };
            if canonical != *stored && seen.insert(char_id.clone()) {
                would_fix.push((char_id.clone(), *stored, canonical));
            }
        }
        println!("distinct (char_id, rarity) pairs: {}", pairs.len());
        println!("char_ids that would be fixed:     {}", would_fix.len());
        for (char_id, from, to) in &would_fix {
            println!("  {char_id}: {from} -> {to}");
        }
        println!("(dry run - no rows updated)");
        return Ok(());
    }

    let t1 = Instant::now();
    let stats = reconcile_rarities(&pool, &game_data)
        .await
        .context("reconcile_rarities failed")?;
    tracing::info!(
        distinct_pairs = stats.distinct_pairs,
        fixed_char_ids = stats.fixed_char_ids,
        rows_updated = stats.rows_updated,
        elapsed_s = format!("{:.2}", t1.elapsed().as_secs_f64()),
        "resync complete",
    );

    if stats.rows_updated > 0 {
        tracing::info!(
            "note: gacha:* cache keys may be stale until the next ttl expiry; \
             restart the server or flush redis (DEL gacha:*) to force a refresh"
        );
    }

    Ok(())
}
