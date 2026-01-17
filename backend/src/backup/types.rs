//! Shared types for the database backup/restore module.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main export manifest containing metadata about the export
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportManifest {
    /// Manifest format version
    pub version: String,
    /// When the export was created
    pub created_at: DateTime<Utc>,
    /// Name/identifier for this export
    pub export_name: String,
    /// Whether files are gzip compressed
    pub compressed: bool,
    /// PostgreSQL export information (if exported)
    pub postgres: Option<PostgresExportInfo>,
    /// Redis export information (if exported)
    pub redis: Option<RedisExportInfo>,
    /// Map of filename -> SHA-256 checksum
    pub checksums: HashMap<String, String>,
}

/// Information about PostgreSQL tables export
#[derive(Debug, Serialize, Deserialize)]
pub struct PostgresExportInfo {
    /// List of exported tables
    pub tables: Vec<TableExportInfo>,
    /// Order tables were exported (for FK dependency)
    pub export_order: Vec<String>,
}

/// Information about a single exported table
#[derive(Debug, Serialize, Deserialize)]
pub struct TableExportInfo {
    /// Table name
    pub name: String,
    /// Number of rows exported
    pub row_count: i64,
    /// Filename containing the data
    pub file: String,
    /// SHA-256 checksum of the file
    pub checksum: String,
}

/// Information about Redis cache export
#[derive(Debug, Serialize, Deserialize)]
pub struct RedisExportInfo {
    /// Key patterns that were exported
    pub patterns: Vec<String>,
    /// Total number of keys exported
    pub key_count: i64,
    /// Filename containing the data
    pub file: String,
    /// SHA-256 checksum of the file
    pub checksum: String,
}

/// Exported Redis data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct RedisExportData {
    /// Key patterns used for export
    pub patterns: Vec<String>,
    /// Map of key -> value with type information
    pub keys: HashMap<String, RedisValue>,
    /// When the export was created
    pub exported_at: DateTime<Utc>,
}

/// Redis value with type information for proper restoration
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RedisValue {
    /// String value
    String { value: String, ttl: Option<i64> },
    /// Binary data (base64 encoded in JSON)
    Binary { value: Vec<u8>, ttl: Option<i64> },
    /// List of strings
    List {
        values: Vec<String>,
        ttl: Option<i64>,
    },
    /// Set of strings
    Set {
        values: Vec<String>,
        ttl: Option<i64>,
    },
    /// Hash map
    Hash {
        fields: HashMap<String, String>,
        ttl: Option<i64>,
    },
}

/// Options for export operation
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Only export PostgreSQL (skip Redis)
    pub postgres_only: bool,
    /// Only export Redis (skip PostgreSQL)
    pub redis_only: bool,
    /// Specific tables to export (None = all)
    pub tables: Option<Vec<String>>,
    /// Redis key patterns to export (None = default patterns)
    pub redis_patterns: Option<Vec<String>>,
    /// Enable gzip compression
    pub compress: bool,
}

/// Options for import operation
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Only import PostgreSQL (skip Redis)
    pub postgres_only: bool,
    /// Only import Redis (skip PostgreSQL)
    pub redis_only: bool,
    /// How to handle existing data
    pub conflict_strategy: ConflictStrategy,
    /// Skip confirmation prompts
    pub yes: bool,
    /// Validate without actually importing
    pub dry_run: bool,
}

/// Strategy for handling existing data during import
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictStrategy {
    /// Delete all existing data first (default)
    #[default]
    Truncate,
    /// Upsert: update existing, insert new
    Merge,
    /// Skip existing records, only insert new
    Skip,
}

impl std::fmt::Display for ConflictStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictStrategy::Truncate => write!(f, "truncate"),
            ConflictStrategy::Merge => write!(f, "merge"),
            ConflictStrategy::Skip => write!(f, "skip"),
        }
    }
}

impl std::str::FromStr for ConflictStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "truncate" => Ok(ConflictStrategy::Truncate),
            "merge" => Ok(ConflictStrategy::Merge),
            "skip" => Ok(ConflictStrategy::Skip),
            _ => Err(format!("Unknown conflict strategy: {}", s)),
        }
    }
}

/// Result of an import operation
#[derive(Debug, Default)]
pub struct ImportResult {
    /// Whether the import was cancelled
    pub cancelled: bool,
    /// Whether this was a dry run
    pub dry_run: bool,
    /// PostgreSQL import results
    pub postgres: Option<PostgresImportResult>,
    /// Redis import results
    pub redis: Option<RedisImportResult>,
}

impl std::fmt::Display for ImportResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.cancelled {
            return write!(f, "Import cancelled.");
        }
        if self.dry_run {
            return write!(f, "Dry run completed successfully - no data was modified.");
        }

        writeln!(f, "Import completed successfully!")?;

        if let Some(ref pg) = self.postgres {
            writeln!(f, "\nPostgreSQL:")?;
            writeln!(f, "  Strategy: {}", pg.strategy_used)?;
            for (table, count) in &pg.tables_imported {
                writeln!(f, "  {}: {} rows", table, count)?;
            }
        }

        if let Some(ref redis) = self.redis {
            writeln!(f, "\nRedis:")?;
            writeln!(f, "  Keys imported: {}", redis.keys_imported)?;
            if redis.keys_skipped > 0 {
                writeln!(f, "  Keys skipped: {}", redis.keys_skipped)?;
            }
        }

        Ok(())
    }
}

/// PostgreSQL import result details
#[derive(Debug)]
pub struct PostgresImportResult {
    /// Map of table name -> rows imported
    pub tables_imported: HashMap<String, i64>,
    /// Strategy that was used
    pub strategy_used: ConflictStrategy,
}

/// Redis import result details
#[derive(Debug)]
pub struct RedisImportResult {
    /// Number of keys imported
    pub keys_imported: i64,
    /// Number of keys skipped (for skip strategy)
    pub keys_skipped: i64,
}

/// Result of verification operation
#[derive(Debug, Default)]
pub struct VerificationResult {
    /// Whether all checks passed
    pub is_valid: bool,
    /// Files that passed checksum verification
    pub verified_files: Vec<String>,
    /// Critical errors that invalidate the export
    pub errors: Vec<String>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
    /// Database comparison results (for deep verification)
    pub db_comparison: HashMap<String, DbComparison>,
    /// Redis comparison results (for deep verification)
    pub redis_comparison: Option<RedisComparison>,
}

/// Comparison between export and database for a table
#[derive(Debug)]
pub struct DbComparison {
    /// Row count in export
    pub export_count: i64,
    /// Current row count in database
    pub db_count: i64,
}

/// Comparison between export and Redis
#[derive(Debug)]
pub struct RedisComparison {
    /// Key count in export
    pub export_count: i64,
    /// Current key count in Redis
    pub db_count: i64,
}
