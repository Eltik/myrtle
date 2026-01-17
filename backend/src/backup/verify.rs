//! Verification and checksum logic for database exports.

use crate::backup::types::{
    DbComparison, ExportManifest, RedisComparison, RedisExportData, VerificationResult,
};
use indicatif::{ProgressBar, ProgressStyle};
use redis::aio::MultiplexedConnection;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::Path;

/// Compute SHA-256 checksum of a string
pub fn compute_sha256(content: &str) -> String {
    compute_sha256_bytes(content.as_bytes())
}

/// Compute SHA-256 checksum of bytes
pub fn compute_sha256_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Verify all checksums in an export
pub async fn verify_checksums(
    manifest: &ExportManifest,
    export_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for (filename, expected_hash) in &manifest.checksums {
        let filepath = export_dir.join(filename);

        if !filepath.exists() {
            return Err(format!("Missing file: {}", filename).into());
        }

        // Read content (handle compression)
        let content = if manifest.compressed {
            use flate2::read::GzDecoder;
            use std::io::Read;

            let file = std::fs::File::open(&filepath)?;
            let mut decoder = GzDecoder::new(file);
            let mut content = String::new();
            decoder.read_to_string(&mut content)?;
            content
        } else {
            tokio::fs::read_to_string(&filepath).await?
        };

        let actual_hash = compute_sha256(&content);

        if actual_hash != *expected_hash {
            return Err(format!(
                "Checksum mismatch for {}: expected {}, got {}",
                filename, expected_hash, actual_hash
            )
            .into());
        }
    }

    Ok(())
}

/// Comprehensive export verification
pub async fn verify_export(
    manifest: &ExportManifest,
    export_dir: &Path,
    deep: bool,
    pool: Option<&PgPool>,
    redis: Option<&mut MultiplexedConnection>,
) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    let mut result = VerificationResult::default();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    // Step 1: Verify all file checksums
    pb.set_message("Verifying file checksums...");
    for (filename, expected_hash) in &manifest.checksums {
        let filepath = export_dir.join(filename);

        if !filepath.exists() {
            result.errors.push(format!("Missing file: {}", filename));
            continue;
        }

        // Read content (handle compression)
        let content = if manifest.compressed {
            use flate2::read::GzDecoder;
            use std::io::Read;

            match std::fs::File::open(&filepath) {
                Ok(file) => {
                    let mut decoder = GzDecoder::new(file);
                    let mut content = String::new();
                    match decoder.read_to_string(&mut content) {
                        Ok(_) => content,
                        Err(e) => {
                            result
                                .errors
                                .push(format!("Failed to decompress {}: {}", filename, e));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to open {}: {}", filename, e));
                    continue;
                }
            }
        } else {
            match tokio::fs::read_to_string(&filepath).await {
                Ok(content) => content,
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to read {}: {}", filename, e));
                    continue;
                }
            }
        };

        let actual_hash = compute_sha256(&content);

        if actual_hash != *expected_hash {
            result.errors.push(format!(
                "Checksum mismatch for {}: expected {}, got {}",
                filename, expected_hash, actual_hash
            ));
        } else {
            result.verified_files.push(filename.clone());
        }
    }

    // Step 2: Verify JSON is parseable and row counts match
    pb.set_message("Verifying JSON format...");
    if let Some(ref pg_info) = manifest.postgres {
        for table_info in &pg_info.tables {
            let filepath = export_dir.join(&table_info.file);

            let content = if manifest.compressed {
                use flate2::read::GzDecoder;
                use std::io::Read;

                match std::fs::File::open(&filepath) {
                    Ok(file) => {
                        let mut decoder = GzDecoder::new(file);
                        let mut content = String::new();
                        match decoder.read_to_string(&mut content) {
                            Ok(_) => content,
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                }
            } else {
                match tokio::fs::read_to_string(&filepath).await {
                    Ok(content) => content,
                    Err(_) => continue,
                }
            };

            match serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                Ok(rows) => {
                    if rows.len() as i64 != table_info.row_count {
                        result.warnings.push(format!(
                            "{}: row count mismatch (manifest: {}, actual: {})",
                            table_info.name,
                            table_info.row_count,
                            rows.len()
                        ));
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("{}: JSON parse error: {}", table_info.name, e));
                }
            }
        }
    }

    // Verify Redis JSON
    if let Some(ref redis_info) = manifest.redis {
        let filepath = export_dir.join(&redis_info.file);

        let content = if manifest.compressed {
            use flate2::read::GzDecoder;
            use std::io::Read;

            match std::fs::File::open(&filepath) {
                Ok(file) => {
                    let mut decoder = GzDecoder::new(file);
                    let mut content = String::new();
                    match decoder.read_to_string(&mut content) {
                        Ok(_) => Some(content),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        } else {
            tokio::fs::read_to_string(&filepath).await.ok()
        };

        if let Some(content) = content {
            match serde_json::from_str::<RedisExportData>(&content) {
                Ok(data) => {
                    if data.keys.len() as i64 != redis_info.key_count {
                        result.warnings.push(format!(
                            "Redis: key count mismatch (manifest: {}, actual: {})",
                            redis_info.key_count,
                            data.keys.len()
                        ));
                    }
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Redis cache: JSON parse error: {}", e));
                }
            }
        }
    }

    // Step 3: Deep verification (requires database connection)
    if deep {
        pb.set_message("Performing deep verification...");

        // Compare PostgreSQL row counts
        if let Some(db_pool) = pool
            && let Some(pg_info) = &manifest.postgres
        {
            for table_info in &pg_info.tables {
                let count_query = format!("SELECT COUNT(*) as count FROM {}", table_info.name);
                match sqlx::query_scalar::<_, i64>(&count_query)
                    .fetch_one(db_pool)
                    .await
                {
                    Ok(db_count) => {
                        result.db_comparison.insert(
                            table_info.name.clone(),
                            DbComparison {
                                export_count: table_info.row_count,
                                db_count,
                            },
                        );
                    }
                    Err(e) => {
                        result.warnings.push(format!(
                            "Could not get row count for {}: {}",
                            table_info.name, e
                        ));
                    }
                }
            }
        }

        // Compare Redis key counts
        if let Some(redis_conn) = redis
            && let Some(redis_info) = &manifest.redis
        {
            let mut total_keys = 0i64;

            for pattern in &redis_info.patterns {
                let count = count_redis_keys(redis_conn, pattern).await.unwrap_or(0);
                total_keys += count;
            }

            result.redis_comparison = Some(RedisComparison {
                export_count: redis_info.key_count,
                db_count: total_keys,
            });
        }
    }

    pb.finish_and_clear();

    result.is_valid = result.errors.is_empty();
    Ok(result)
}

/// Count Redis keys matching a pattern
async fn count_redis_keys(
    redis: &mut MultiplexedConnection,
    pattern: &str,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut count = 0i64;
    let mut cursor: u64 = 0;

    loop {
        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(1000)
            .query_async(redis)
            .await?;

        count += keys.len() as i64;
        cursor = new_cursor;

        if cursor == 0 {
            break;
        }
    }

    Ok(count)
}

/// Print verification result summary
pub fn print_verification_result(result: &VerificationResult) {
    println!("\n========================================");
    println!("       VERIFICATION RESULTS");
    println!("========================================\n");

    // Files verified
    println!("Files verified: {}", result.verified_files.len());
    for file in &result.verified_files {
        println!("  [OK] {}", file);
    }

    // Errors
    if !result.errors.is_empty() {
        println!("\nErrors ({}):", result.errors.len());
        for error in &result.errors {
            println!("  [ERROR] {}", error);
        }
    }

    // Warnings
    if !result.warnings.is_empty() {
        println!("\nWarnings ({}):", result.warnings.len());
        for warning in &result.warnings {
            println!("  [WARN] {}", warning);
        }
    }

    // Database comparison
    if !result.db_comparison.is_empty() {
        println!("\nDatabase comparison:");
        println!(
            "  {:<30} {:>12} {:>12} {:>10}",
            "Table", "Export", "Database", "Diff"
        );
        println!("  {}", "-".repeat(66));
        for (table, comparison) in &result.db_comparison {
            let diff = comparison.db_count - comparison.export_count;
            let diff_str = if diff == 0 {
                "=".to_string()
            } else if diff > 0 {
                format!("+{}", diff)
            } else {
                format!("{}", diff)
            };
            println!(
                "  {:<30} {:>12} {:>12} {:>10}",
                table, comparison.export_count, comparison.db_count, diff_str
            );
        }
    }

    // Redis comparison
    if let Some(ref comparison) = result.redis_comparison {
        println!("\nRedis comparison:");
        let diff = comparison.db_count - comparison.export_count;
        let diff_str = if diff == 0 {
            "=".to_string()
        } else if diff > 0 {
            format!("+{}", diff)
        } else {
            format!("{}", diff)
        };
        println!(
            "  Export keys: {}, Current keys: {}, Diff: {}",
            comparison.export_count, comparison.db_count, diff_str
        );
    }

    // Final status
    println!();
    if result.is_valid {
        println!("Status: PASSED");
    } else {
        println!("Status: FAILED");
    }
}
