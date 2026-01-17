//! Import logic for PostgreSQL and Redis databases with safety measures.

use crate::backup::types::{
    ConflictStrategy, ExportManifest, ImportOptions, ImportResult, PostgresExportInfo,
    PostgresImportResult, RedisExportData, RedisExportInfo, RedisImportResult, RedisValue,
};
use crate::backup::verify::verify_checksums;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;

/// Table import order respecting foreign key dependencies.
/// Same as export order - parents must be imported first.
pub const TABLE_IMPORT_ORDER: &[&str] = &[
    "users",
    "tier_lists",
    "tiers",
    "tier_placements",
    "tier_list_versions",
    "tier_change_log",
    "tier_list_permissions",
];

/// Get the primary conflict target for a table.
/// This returns the best column(s) to use for ON CONFLICT.
fn get_primary_conflict_target(table_name: &str) -> &'static str {
    match table_name {
        // For users, uid is the natural key
        "users" => "uid",
        // For tier_lists, slug is the natural key
        "tier_lists" => "slug",
        // For tier_placements, the combination of tier_id and operator_id is unique
        "tier_placements" => "tier_id, operator_id",
        // For tier_list_versions, the combination is unique
        "tier_list_versions" => "tier_list_id, version",
        // For tier_list_permissions, the combination is unique
        "tier_list_permissions" => "tier_list_id, user_id, permission",
        // For tiers, use the composite unique constraint
        "tiers" => "tier_list_id, display_order",
        // Default to id for other tables
        _ => "id",
    }
}

/// Run the full import operation with safety measures
pub async fn import_all(
    pool: &PgPool,
    redis: &mut MultiplexedConnection,
    manifest: &ExportManifest,
    export_dir: &Path,
    options: &ImportOptions,
) -> Result<ImportResult, Box<dyn std::error::Error>> {
    // Step 1: Validate checksums before any modification
    println!("\nValidating export integrity...");
    verify_checksums(manifest, export_dir).await?;
    println!("  Checksums verified.");

    // Step 2: Confirmation prompt (unless --yes or --dry-run)
    if !options.yes && !options.dry_run {
        let strategy_desc = match options.conflict_strategy {
            ConflictStrategy::Truncate => "DELETE all existing data and replace with backup",
            ConflictStrategy::Merge => {
                "MERGE backup with existing data (update existing, insert new)"
            }
            ConflictStrategy::Skip => "ADD new records only (skip existing)",
        };

        println!("\n========================================");
        println!("         IMPORT CONFIRMATION");
        println!("========================================\n");
        println!("Export: {}", manifest.export_name);
        println!(
            "Created: {}",
            manifest.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("\nData to import:");
        if let Some(ref pg) = manifest.postgres
            && !options.redis_only
        {
            println!("  PostgreSQL: {} tables", pg.tables.len());
            for table in &pg.tables {
                println!("    - {}: {} rows", table.name, table.row_count);
            }
        }
        if let Some(ref r) = manifest.redis
            && !options.postgres_only
        {
            println!("  Redis: {} keys", r.key_count);
        }
        println!("\nStrategy: {}", strategy_desc);
        println!();

        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to proceed?")
            .default(false)
            .interact()?;

        if !confirmed {
            return Ok(ImportResult {
                cancelled: true,
                ..Default::default()
            });
        }
    }

    // Step 4: Dry run validation
    if options.dry_run {
        println!("\nPerforming dry run validation...");
        validate_import_data(manifest, export_dir).await?;
        println!("  Validation passed!");
        return Ok(ImportResult {
            dry_run: true,
            ..Default::default()
        });
    }

    let mut result = ImportResult::default();

    // Step 5: Import PostgreSQL (within transaction)
    if let Some(ref pg_info) = manifest.postgres
        && !options.redis_only
    {
        println!("\nImporting PostgreSQL...");
        result.postgres = Some(
            import_postgres_transactional(pool, pg_info, export_dir, options, manifest.compressed)
                .await?,
        );
    }

    // Step 6: Import Redis
    if let Some(ref redis_info) = manifest.redis
        && !options.postgres_only
    {
        println!("\nImporting Redis...");
        result.redis =
            Some(import_redis(redis, redis_info, export_dir, options, manifest.compressed).await?);
    }

    Ok(result)
}

/// Validate import data without modifying anything
async fn validate_import_data(
    manifest: &ExportManifest,
    export_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate PostgreSQL JSON files
    if let Some(ref pg_info) = manifest.postgres {
        for table_info in &pg_info.tables {
            let filepath = export_dir.join(&table_info.file);
            let content = read_file_content(&filepath, manifest.compressed).await?;
            let rows: Vec<serde_json::Value> = serde_json::from_str(&content)?;

            if rows.len() as i64 != table_info.row_count {
                return Err(format!(
                    "Row count mismatch for {}: expected {}, found {}",
                    table_info.name,
                    table_info.row_count,
                    rows.len()
                )
                .into());
            }

            // Validate each row has expected structure (is an object)
            for (i, row) in rows.iter().enumerate() {
                if !row.is_object() {
                    return Err(format!(
                        "Invalid row {} in {}: expected JSON object",
                        i, table_info.name
                    )
                    .into());
                }
            }
        }
    }

    // Validate Redis JSON file
    if let Some(ref redis_info) = manifest.redis {
        let filepath = export_dir.join(&redis_info.file);
        let content = read_file_content(&filepath, manifest.compressed).await?;
        let _data: RedisExportData = serde_json::from_str(&content)?;
    }

    Ok(())
}

/// Import PostgreSQL data within a transaction
async fn import_postgres_transactional(
    pool: &PgPool,
    info: &PostgresExportInfo,
    export_dir: &Path,
    options: &ImportOptions,
    compressed: bool,
) -> Result<PostgresImportResult, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new(info.tables.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Start transaction
    let mut tx = pool.begin().await?;

    // Truncate tables if using truncate strategy (in reverse order for FK constraints)
    if options.conflict_strategy == ConflictStrategy::Truncate {
        pb.set_message("Truncating tables...");
        for table_name in TABLE_IMPORT_ORDER.iter().rev() {
            if info.export_order.contains(&table_name.to_string()) {
                sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table_name))
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }

    let mut imported_counts: HashMap<String, i64> = HashMap::new();

    // Import tables in dependency order
    for table_name in TABLE_IMPORT_ORDER {
        if let Some(table_info) = info.tables.iter().find(|t| t.name == *table_name) {
            pb.set_message(format!("Importing {}...", table_name));

            let filepath = export_dir.join(&table_info.file);
            let content = read_file_content(&filepath, compressed).await?;
            let rows: Vec<serde_json::Value> = serde_json::from_str(&content)?;

            let count = import_table(&mut tx, table_name, &rows, options).await?;
            imported_counts.insert(table_name.to_string(), count);

            pb.inc(1);
        }
    }

    // Commit transaction
    tx.commit().await?;

    pb.finish_with_message("PostgreSQL import complete");

    Ok(PostgresImportResult {
        tables_imported: imported_counts,
        strategy_used: options.conflict_strategy,
    })
}

/// Import a single table's data
async fn import_table(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    table_name: &str,
    rows: &[serde_json::Value],
    options: &ImportOptions,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut count = 0i64;

    for row in rows {
        let obj = row
            .as_object()
            .ok_or_else(|| format!("Row in {} is not a JSON object", table_name))?;

        if obj.is_empty() {
            continue;
        }

        // Build column list and values
        let columns: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
        let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();

        // Build conflict clause based on strategy
        let conflict_target = get_primary_conflict_target(table_name);
        let conflict_columns: Vec<&str> = conflict_target.split(", ").collect();

        let conflict_clause = match options.conflict_strategy {
            ConflictStrategy::Truncate => String::new(),
            ConflictStrategy::Merge => {
                // Exclude conflict columns from updates
                let updates: Vec<String> = columns
                    .iter()
                    .filter(|c| !conflict_columns.contains(c) && **c != "id")
                    .map(|c| format!("{} = EXCLUDED.{}", c, c))
                    .collect();
                if updates.is_empty() {
                    // If all columns are conflict columns, just do nothing on conflict
                    format!(" ON CONFLICT ({}) DO NOTHING", conflict_target)
                } else {
                    format!(
                        " ON CONFLICT ({}) DO UPDATE SET {}",
                        conflict_target,
                        updates.join(", ")
                    )
                }
            }
            ConflictStrategy::Skip => format!(" ON CONFLICT ({}) DO NOTHING", conflict_target),
        };

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}){}",
            table_name,
            columns.join(", "),
            placeholders.join(", "),
            conflict_clause
        );

        // Build the query with bound parameters
        let mut q = sqlx::query(&query);
        for col in &columns {
            let value = &obj[*col];
            q = bind_json_value(q, value);
        }

        let result = q.execute(&mut **tx).await?;
        count += result.rows_affected() as i64;
    }

    Ok(count)
}

/// Bind a JSON value to a SQLx query
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: &'q serde_json::Value,
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    match value {
        serde_json::Value::Null => query.bind(None::<String>),
        serde_json::Value::Bool(b) => query.bind(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            // Try to parse as UUID if it looks like one
            if s.len() == 36
                && s.chars().filter(|c| *c == '-').count() == 4
                && let Ok(uuid) = uuid::Uuid::parse_str(s)
            {
                return query.bind(uuid);
            }
            // Try to parse as datetime if it looks like ISO 8601
            if s.contains('T')
                && (s.ends_with('Z') || s.contains('+'))
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s)
            {
                return query.bind(dt.with_timezone(&chrono::Utc));
            }
            query.bind(s.as_str())
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            // Store as JSONB
            query.bind(value)
        }
    }
}

/// Import Redis data
async fn import_redis(
    redis: &mut MultiplexedConnection,
    info: &RedisExportInfo,
    export_dir: &Path,
    options: &ImportOptions,
    compressed: bool,
) -> Result<RedisImportResult, Box<dyn std::error::Error>> {
    let filepath = export_dir.join(&info.file);
    let content = read_file_content(&filepath, compressed).await?;
    let data: RedisExportData = serde_json::from_str(&content)?;

    let pb = ProgressBar::new(data.keys.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("Importing Redis keys...");

    let mut imported = 0i64;
    let mut skipped = 0i64;

    // Use pipeline for batch operations
    let mut pipe = redis::pipe();
    let mut pipe_count = 0;
    const BATCH_SIZE: usize = 100;

    for (key, value) in &data.keys {
        // Check if key exists (for skip strategy)
        if options.conflict_strategy == ConflictStrategy::Skip {
            let exists: bool = redis.exists(key).await?;
            if exists {
                skipped += 1;
                pb.inc(1);
                continue;
            }
        }

        // Delete existing key for truncate/merge
        if options.conflict_strategy == ConflictStrategy::Truncate
            || options.conflict_strategy == ConflictStrategy::Merge
        {
            pipe.del(key);
        }

        // Set new value based on type
        match value {
            RedisValue::String { value: s, ttl } => {
                pipe.set(key, s);
                if let Some(t) = ttl {
                    pipe.expire(key, *t);
                }
            }
            RedisValue::Binary { value: bytes, ttl } => {
                pipe.set(key, bytes.as_slice());
                if let Some(t) = ttl {
                    pipe.expire(key, *t);
                }
            }
            RedisValue::List { values, ttl } => {
                for item in values {
                    pipe.rpush(key, item);
                }
                if let Some(t) = ttl {
                    pipe.expire(key, *t);
                }
            }
            RedisValue::Set { values, ttl } => {
                for item in values {
                    pipe.sadd(key, item);
                }
                if let Some(t) = ttl {
                    pipe.expire(key, *t);
                }
            }
            RedisValue::Hash { fields, ttl } => {
                for (field, val) in fields {
                    pipe.hset(key, field, val);
                }
                if let Some(t) = ttl {
                    pipe.expire(key, *t);
                }
            }
        }

        imported += 1;
        pipe_count += 1;

        // Execute batch
        if pipe_count >= BATCH_SIZE {
            pipe.query_async::<()>(redis).await?;
            pipe = redis::pipe();
            pipe_count = 0;
        }

        pb.inc(1);
    }

    // Execute remaining commands
    if pipe_count > 0 {
        pipe.query_async::<()>(redis).await?;
    }

    pb.finish_with_message(format!(
        "Redis import complete ({} imported, {} skipped)",
        imported, skipped
    ));

    Ok(RedisImportResult {
        keys_imported: imported,
        keys_skipped: skipped,
    })
}

/// Read file content, handling compression if needed
async fn read_file_content(
    filepath: &Path,
    compressed: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    if compressed {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let file = std::fs::File::open(filepath)?;
        let mut decoder = GzDecoder::new(file);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    } else {
        Ok(tokio::fs::read_to_string(filepath).await?)
    }
}
