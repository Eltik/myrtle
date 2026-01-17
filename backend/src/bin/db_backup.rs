//! Database backup/restore CLI tool for Myrtle backend.
//!
//! Usage:
//!   cargo run --bin db-backup -- export [OPTIONS]
//!   cargo run --bin db-backup -- import <EXPORT_NAME> [OPTIONS]
//!   cargo run --bin db-backup -- verify <EXPORT_NAME> [--deep]
//!   cargo run --bin db-backup -- list
//!
//! Examples:
//!   # Export all databases
//!   cargo run --bin db-backup -- export
//!
//!   # Export with compression
//!   cargo run --bin db-backup -- export --compress
//!
//!   # Import with confirmation
//!   cargo run --bin db-backup -- import 2024-01-15_143022
//!
//!   # Dry run import
//!   cargo run --bin db-backup -- import 2024-01-15_143022 --dry-run

use backend::backup::{
    cli::{Cli, Commands},
    export::run_export,
    import::import_all,
    types::{ExportManifest, ExportOptions, ImportOptions},
    verify::{print_verification_result, verify_export},
};
use chrono::Utc;
use clap::Parser;
use std::cmp::Reverse;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let backup_dir = PathBuf::from(&cli.dir);

    // Ensure backup directory exists
    tokio::fs::create_dir_all(&backup_dir).await?;

    match cli.command {
        Commands::Export {
            postgres_only,
            redis_only,
            tables,
            redis_patterns,
            compress,
            name,
        } => {
            run_export_command(
                &backup_dir,
                postgres_only,
                redis_only,
                tables,
                redis_patterns,
                compress,
                name,
                cli.verbose,
            )
            .await?;
        }

        Commands::Import {
            export_name,
            postgres_only,
            redis_only,
            conflict_strategy,
            yes,
            dry_run,
        } => {
            run_import_command(
                &backup_dir,
                &export_name,
                postgres_only,
                redis_only,
                conflict_strategy.into(),
                yes,
                dry_run,
                cli.verbose,
            )
            .await?;
        }

        Commands::Verify { export_name, deep } => {
            run_verify_command(&backup_dir, &export_name, deep, cli.verbose).await?;
        }

        Commands::List => {
            run_list_command(&backup_dir).await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_export_command(
    backup_dir: &Path,
    postgres_only: bool,
    redis_only: bool,
    tables: Option<Vec<String>>,
    redis_patterns: Option<Vec<String>>,
    compress: bool,
    name: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to databases
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    if verbose {
        println!("Connecting to PostgreSQL...");
    }
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    if verbose {
        println!("Connecting to Redis...");
    }
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis = redis_client.get_multiplexed_async_connection().await?;

    // Generate export name
    let export_name = name.unwrap_or_else(|| Utc::now().format("%Y-%m-%d_%H%M%S").to_string());

    let export_dir = backup_dir.join(&export_name);
    tokio::fs::create_dir_all(&export_dir).await?;

    println!("\n========================================");
    println!("    Myrtle Database Export Tool");
    println!("========================================\n");
    println!("Export name: {}", export_name);
    println!("Output directory: {}", export_dir.display());
    if compress {
        println!("Compression: enabled");
    }

    let options = ExportOptions {
        postgres_only,
        redis_only,
        tables,
        redis_patterns,
        compress,
    };

    let manifest = run_export(&pool, &mut redis, &export_dir, options).await?;

    // Save manifest
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    tokio::fs::write(export_dir.join("manifest.json"), &manifest_json).await?;

    // Print summary
    println!("\n========================================");
    println!("          EXPORT COMPLETE");
    println!("========================================\n");
    println!("Export: {}", export_name);

    if let Some(ref pg) = manifest.postgres {
        println!("\nPostgreSQL:");
        for table in &pg.tables {
            println!("  {}: {} rows", table.name, table.row_count);
        }
    }

    if let Some(ref r) = manifest.redis {
        println!("\nRedis:");
        println!("  {} keys exported", r.key_count);
    }

    println!("\nTo import this backup:");
    println!("  cargo run --bin db-backup -- import {}", export_name);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_import_command(
    backup_dir: &Path,
    export_name: &str,
    postgres_only: bool,
    redis_only: bool,
    conflict_strategy: backend::backup::types::ConflictStrategy,
    yes: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let export_dir = backup_dir.join(export_name);

    if !export_dir.exists() {
        eprintln!("Export not found: {}", export_dir.display());
        eprintln!("\nAvailable exports:");
        run_list_command(backup_dir).await?;
        std::process::exit(1);
    }

    // Load manifest
    let manifest_path = export_dir.join("manifest.json");
    if !manifest_path.exists() {
        eprintln!(
            "Invalid export: manifest.json not found in {}",
            export_dir.display()
        );
        std::process::exit(1);
    }

    let manifest_json = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: ExportManifest = serde_json::from_str(&manifest_json)?;

    println!("\n========================================");
    println!("    Myrtle Database Import Tool");
    println!("========================================\n");
    println!("Import from: {}", export_name);
    println!(
        "Created at: {}",
        manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    // Connect to databases
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    if verbose {
        println!("\nConnecting to PostgreSQL...");
    }
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    if verbose {
        println!("Connecting to Redis...");
    }
    let redis_client = redis::Client::open(redis_url)?;
    let mut redis = redis_client.get_multiplexed_async_connection().await?;

    let options = ImportOptions {
        postgres_only,
        redis_only,
        conflict_strategy,
        yes,
        dry_run,
    };

    let result = import_all(&pool, &mut redis, &manifest, &export_dir, &options).await?;

    println!("\n{}", result);

    Ok(())
}

async fn run_verify_command(
    backup_dir: &Path,
    export_name: &str,
    deep: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let export_dir = backup_dir.join(export_name);

    if !export_dir.exists() {
        eprintln!("Export not found: {}", export_dir.display());
        std::process::exit(1);
    }

    // Load manifest
    let manifest_path = export_dir.join("manifest.json");
    let manifest_json = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: ExportManifest = serde_json::from_str(&manifest_json)?;

    println!("\n========================================");
    println!("   Myrtle Export Verification");
    println!("========================================\n");
    println!("Verifying: {}", export_name);
    println!(
        "Created: {}",
        manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );

    let (pool, mut redis_conn) = if deep {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for deep verification");
        let redis_url =
            std::env::var("REDIS_URL").expect("REDIS_URL must be set for deep verification");

        if verbose {
            println!("\nConnecting to databases for deep verification...");
        }

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await?;

        let redis_client = redis::Client::open(redis_url)?;
        let redis = redis_client.get_multiplexed_async_connection().await?;

        (Some(pool), Some(redis))
    } else {
        (None, None)
    };

    let result = verify_export(
        &manifest,
        &export_dir,
        deep,
        pool.as_ref(),
        redis_conn.as_mut(),
    )
    .await?;

    print_verification_result(&result);

    if !result.is_valid {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_list_command(backup_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nAvailable exports in {}:\n", backup_dir.display());

    // Collect directory entries
    let mut entries: Vec<tokio::fs::DirEntry> = Vec::new();
    let mut read_dir = tokio::fs::read_dir(backup_dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            entries.push(entry);
        }
    }

    // Sort by name (which is timestamp-based), newest first
    entries.sort_by_key(|b| Reverse(b.file_name()));

    if entries.is_empty() {
        println!("  No exports found.");
        return Ok(());
    }

    println!(
        "  {:<25} {:<22} {:>10} {:>12}",
        "Name", "Created", "PG Tables", "Redis Keys"
    );
    println!("  {}", "-".repeat(71));

    for entry in entries {
        let manifest_path = entry.path().join("manifest.json");
        if manifest_path.exists()
            && let Ok(manifest_json) = tokio::fs::read_to_string(&manifest_path).await
            && let Ok(manifest) = serde_json::from_str::<ExportManifest>(&manifest_json)
        {
            let pg_tables = manifest
                .postgres
                .as_ref()
                .map(|p| p.tables.len())
                .unwrap_or(0);
            let redis_keys = manifest.redis.as_ref().map(|r| r.key_count).unwrap_or(0);

            println!(
                "  {:<25} {:<22} {:>10} {:>12}",
                entry.file_name().to_string_lossy(),
                manifest.created_at.format("%Y-%m-%d %H:%M:%S"),
                pg_tables,
                redis_keys
            );
        }
    }

    println!();
    Ok(())
}
