//! CLI argument parsing for the database backup tool.

use crate::backup::types::ConflictStrategy;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "db-backup")]
#[command(author = "Myrtle")]
#[command(version = "1.0")]
#[command(about = "Database export/import tool for Myrtle backend")]
#[command(
    long_about = "Export and import PostgreSQL and Redis databases with safety measures including checksums, transactions, and confirmation prompts."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output/input directory for backups
    #[arg(short, long, default_value = "./backups", global = true)]
    pub dir: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Export databases to files
    Export {
        /// Export only PostgreSQL (skip Redis)
        #[arg(long)]
        postgres_only: bool,

        /// Export only Redis (skip PostgreSQL)
        #[arg(long)]
        redis_only: bool,

        /// Specific PostgreSQL tables to export (comma-separated)
        /// Available: users, tier_lists, tiers, tier_placements, tier_list_versions, tier_change_log, tier_list_permissions
        #[arg(long, value_delimiter = ',')]
        tables: Option<Vec<String>>,

        /// Redis key patterns to export (comma-separated)
        /// Default: "static:*"
        #[arg(long, value_delimiter = ',')]
        redis_patterns: Option<Vec<String>>,

        /// Enable gzip compression for exported files
        #[arg(long, short)]
        compress: bool,

        /// Custom export name (default: timestamp YYYY-MM-DD_HHMMSS)
        #[arg(long)]
        name: Option<String>,
    },

    /// Import databases from exported files
    Import {
        /// Export name/timestamp to import from
        export_name: String,

        /// Import only PostgreSQL (skip Redis)
        #[arg(long)]
        postgres_only: bool,

        /// Import only Redis (skip PostgreSQL)
        #[arg(long)]
        redis_only: bool,

        /// How to handle existing data
        #[arg(long, short, default_value = "truncate", value_enum)]
        conflict_strategy: CliConflictStrategy,

        /// Skip confirmation prompts (use with caution!)
        #[arg(long, short)]
        yes: bool,

        /// Validate data without actually importing (dry run)
        #[arg(long)]
        dry_run: bool,
    },

    /// Verify an export's integrity
    Verify {
        /// Export name/timestamp to verify
        export_name: String,

        /// Connect to databases and compare row counts
        #[arg(long)]
        deep: bool,
    },

    /// List available exports
    List,
}

/// CLI-friendly conflict strategy enum
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliConflictStrategy {
    /// Delete all existing data first
    Truncate,
    /// Update existing records, insert new ones
    Merge,
    /// Skip existing records, only insert new
    Skip,
}

impl From<CliConflictStrategy> for ConflictStrategy {
    fn from(cli: CliConflictStrategy) -> Self {
        match cli {
            CliConflictStrategy::Truncate => ConflictStrategy::Truncate,
            CliConflictStrategy::Merge => ConflictStrategy::Merge,
            CliConflictStrategy::Skip => ConflictStrategy::Skip,
        }
    }
}
