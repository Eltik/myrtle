//! Export logic for PostgreSQL and Redis databases.

use crate::backup::types::{
    ExportManifest, ExportOptions, PostgresExportInfo, RedisExportData, RedisExportInfo,
    RedisValue, TableExportInfo,
};
use crate::backup::verify::compute_sha256;
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;

/// Table export order respecting foreign key dependencies.
/// Tables are listed in order they should be exported (parents first).
pub const TABLE_EXPORT_ORDER: &[&str] = &[
    "users",                 // No FK dependencies
    "tier_lists",            // FK: created_by -> users (nullable)
    "tiers",                 // FK: tier_list_id -> tier_lists
    "tier_placements",       // FK: tier_id -> tiers
    "tier_list_versions",    // FK: tier_list_id -> tier_lists, published_by -> users
    "tier_change_log",       // FK: tier_list_id, version_id, changed_by
    "tier_list_permissions", // FK: tier_list_id, user_id, granted_by
    "tier_list_reports",     // FK: tier_list_id, reporter_id, reviewed_by -> users
    "gacha_records",         // FK: user_id -> users
    "user_gacha_settings",   // FK: user_id -> users
];

/// Default Redis key patterns to export
pub const DEFAULT_REDIS_PATTERNS: &[&str] = &["static:*"];

/// Run the full export operation
pub async fn run_export(
    pool: &PgPool,
    redis: &mut MultiplexedConnection,
    export_dir: &Path,
    options: ExportOptions,
) -> Result<ExportManifest, Box<dyn std::error::Error>> {
    let mut checksums: HashMap<String, String> = HashMap::new();
    let mut postgres_info: Option<PostgresExportInfo> = None;
    let mut redis_info: Option<RedisExportInfo> = None;

    // Export PostgreSQL
    if !options.redis_only {
        let tables_to_export: Vec<&str> = match &options.tables {
            Some(tables) => tables.iter().map(|s| s.as_str()).collect(),
            None => TABLE_EXPORT_ORDER.to_vec(),
        };

        println!("\nExporting PostgreSQL tables...");
        let pg_result =
            export_postgres(pool, export_dir, &tables_to_export, options.compress).await?;

        // Add checksums
        for table_info in &pg_result.tables {
            checksums.insert(table_info.file.clone(), table_info.checksum.clone());
        }

        postgres_info = Some(pg_result);
    }

    // Export Redis
    if !options.postgres_only {
        let patterns: Vec<&str> = match &options.redis_patterns {
            Some(patterns) => patterns.iter().map(|s| s.as_str()).collect(),
            None => DEFAULT_REDIS_PATTERNS.to_vec(),
        };

        println!("\nExporting Redis cache...");
        let redis_result = export_redis(redis, export_dir, &patterns, options.compress).await?;

        checksums.insert(redis_result.file.clone(), redis_result.checksum.clone());
        redis_info = Some(redis_result);
    }

    let export_name = export_dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok(ExportManifest {
        version: "1.0".to_string(),
        created_at: Utc::now(),
        export_name,
        compressed: options.compress,
        postgres: postgres_info,
        redis: redis_info,
        checksums,
    })
}

/// Export PostgreSQL tables to JSON files
async fn export_postgres(
    pool: &PgPool,
    export_dir: &Path,
    tables: &[&str],
    compress: bool,
) -> Result<PostgresExportInfo, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new(tables.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut table_infos = Vec::new();

    for table_name in tables {
        pb.set_message(format!("Exporting {}...", table_name));

        // Export table to JSON using row_to_json
        let query = format!("SELECT row_to_json(t) as data FROM {} t", table_name);
        let rows: Vec<(serde_json::Value,)> = sqlx::query_as(&query).fetch_all(pool).await?;

        let count = rows.len() as i64;
        let data: Vec<serde_json::Value> = rows.into_iter().map(|(v,)| v).collect();

        let json = serde_json::to_string_pretty(&data)?;
        let checksum = compute_sha256(&json);

        let filename = if compress {
            format!("{}.json.gz", table_name)
        } else {
            format!("{}.json", table_name)
        };
        let filepath = export_dir.join(&filename);

        if compress {
            use flate2::Compression;
            use flate2::write::GzEncoder;
            use std::io::Write;

            let file = std::fs::File::create(&filepath)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(json.as_bytes())?;
            encoder.finish()?;
        } else {
            tokio::fs::write(&filepath, &json).await?;
        }

        table_infos.push(TableExportInfo {
            name: table_name.to_string(),
            row_count: count,
            file: filename,
            checksum,
        });

        pb.inc(1);
    }

    pb.finish_with_message("PostgreSQL export complete");

    Ok(PostgresExportInfo {
        tables: table_infos,
        export_order: tables.iter().map(|s| s.to_string()).collect(),
    })
}

/// Export Redis keys to JSON file
async fn export_redis(
    redis: &mut MultiplexedConnection,
    export_dir: &Path,
    patterns: &[&str],
    compress: bool,
) -> Result<RedisExportInfo, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let mut all_data: HashMap<String, RedisValue> = HashMap::new();

    for pattern in patterns {
        pb.set_message(format!("Scanning Redis keys: {}...", pattern));

        // Use SCAN to find all matching keys (memory-safe iteration)
        let mut cursor: u64 = 0;
        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(*pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(redis)
                .await?;

            for key in keys {
                if let Ok(value) = get_redis_value(redis, &key).await {
                    all_data.insert(key, value);
                }
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }
    }

    pb.set_message(format!("Exporting {} Redis keys...", all_data.len()));

    let export_data = RedisExportData {
        patterns: patterns.iter().map(|s| s.to_string()).collect(),
        keys: all_data,
        exported_at: Utc::now(),
    };

    let json = serde_json::to_string_pretty(&export_data)?;
    let checksum = compute_sha256(&json);

    let filename = if compress {
        "redis_cache.json.gz".to_string()
    } else {
        "redis_cache.json".to_string()
    };
    let filepath = export_dir.join(&filename);

    if compress {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;

        let file = std::fs::File::create(&filepath)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json.as_bytes())?;
        encoder.finish()?;
    } else {
        tokio::fs::write(&filepath, &json).await?;
    }

    let key_count = export_data.keys.len() as i64;

    pb.finish_with_message(format!("Redis export complete ({} keys)", key_count));

    Ok(RedisExportInfo {
        patterns: patterns.iter().map(|s| s.to_string()).collect(),
        key_count,
        file: filename,
        checksum,
    })
}

/// Get a Redis value with its type and TTL
async fn get_redis_value(
    redis: &mut MultiplexedConnection,
    key: &str,
) -> Result<RedisValue, Box<dyn std::error::Error>> {
    // Get key type
    let key_type: String = redis::cmd("TYPE").arg(key).query_async(redis).await?;

    // Get TTL (-1 = no expiry, -2 = key doesn't exist)
    let ttl: i64 = redis.ttl(key).await.unwrap_or(-1);
    let ttl_opt = if ttl > 0 { Some(ttl) } else { None };

    let value = match key_type.as_str() {
        "string" => {
            // Try to get as string
            let s: String = redis.get(key).await?;
            RedisValue::String {
                value: s,
                ttl: ttl_opt,
            }
        }
        "list" => {
            let items: Vec<String> = redis.lrange(key, 0, -1).await?;
            RedisValue::List {
                values: items,
                ttl: ttl_opt,
            }
        }
        "set" => {
            let items: Vec<String> = redis.smembers(key).await?;
            RedisValue::Set {
                values: items,
                ttl: ttl_opt,
            }
        }
        "hash" => {
            let items: HashMap<String, String> = redis.hgetall(key).await?;
            RedisValue::Hash {
                fields: items,
                ttl: ttl_opt,
            }
        }
        _ => {
            return Err(format!("Unsupported Redis type: {}", key_type).into());
        }
    };

    Ok(value)
}
