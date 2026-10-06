//! Import a JSONL export produced by `export-database` back into `PostgreSQL`.
//!
//! Usage:
//!   cargo run --release --bin import-database -- --in <dir> [--truncate] [--batch-size N]
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).
//!
//! Strategy:
//!   - Wraps the entire import in ONE transaction: all or nothing.
//!   - Sets `session_replication_role = replica` so audit triggers and FK
//!     constraints are not re-fired while restoring (the export is already a
//!     consistent snapshot; FKs are re-validated implicitly by the ordering).
//!   - Each batch ships as one JSONB array parameter, unpacked server-side with
//!     `jsonb_populate_recordset(null::<table>, $1)`, so Postgres infers column
//!     types from the live schema and a new column needs no code change as long
//!     as the JSON keys match.
//!   - After loading, `setval(pg_get_serial_sequence(...))` advances each
//!     BIGSERIAL sequence past the largest imported id.
//!
//! Partial exports:
//!   - The manifest may list any subsequence of `TABLES`: an export written
//!     before a table existed, or one made from a checklist. Tables it leaves
//!     out are not touched, apart from what `--truncate`'s CASCADE reaches,
//!     which is printed before anything loads.
//!
//! Safety:
//!   - Refuses to run if any table the export loads is non-empty unless
//!     `--truncate` is passed. `SEEDED_TABLES` are exempt: their migration
//!     rows are replaced when the export carries rows for them and kept when
//!     it carries none (`translate-backup` writes an empty `locales`).
//!   - `--truncate` uses `TRUNCATE ... RESTART IDENTITY CASCADE` inside the
//!     transaction so a failure rolls back to the pre-import state.

// CLI tool: a long top-level `main` and an intentional sign-dropping cast are fine here.
#![allow(clippy::too_many_lines, clippy::cast_sign_loss)]

use anyhow::{Context, Result, bail};
use backend::database::run_migrations;
use backend::db_export::{
    FORMAT_VERSION, MANIFEST_FILE, SEEDED_TABLES, SERIAL_COLUMNS, TABLES, closure, foreign_keys,
    upgrade_legacy_row,
};
use dotenv::dotenv;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Executor, postgres::PgPoolOptions, types::Json};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const DEFAULT_BATCH_SIZE: usize = 1000;

/// Conservative per-batch payload cap. Postgres rejects a jsonb value above
/// ~256MB ("total size of jsonb array elements exceeds the maximum"), and the
/// server-side binary representation can run larger than the JSON text we
/// measure here, so flush well below the hard limit. Row-count batching alone
/// is not enough: 1000 rows of a jumbo-JSONB table (synced building blobs)
/// overflow the cap long before the row limit. An oversized SINGLE row still
/// ships alone; if one row crosses the server cap by itself, no batching saves it.
const MAX_BATCH_BYTES: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
struct Manifest {
    format_version: u32,
    #[allow(dead_code)]
    exported_at: String,
    #[allow(dead_code)]
    database_version: String,
    tables: Vec<TableEntry>,
}

#[derive(Deserialize)]
struct TableEntry {
    name: String,
    rows: u64,
    file: String,
}

struct Args {
    in_dir: PathBuf,
    truncate: bool,
    migrate: bool,
    batch_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = parse_args()?;

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let manifest = load_manifest(&args.in_dir)?;
    if manifest.format_version != FORMAT_VERSION {
        bail!(
            "manifest format_version {} is not supported (expected {})",
            manifest.format_version,
            FORMAT_VERSION
        );
    }
    verify_manifest_tables(&manifest)?;

    // In `TABLES` order, which the manifest was just checked to follow.
    let loaded: Vec<&'static str> = TABLES
        .iter()
        .copied()
        .filter(|t| manifest.tables.iter().any(|e| e.name == *t))
        .collect();
    let skipped: Vec<&'static str> = TABLES
        .iter()
        .copied()
        .filter(|t| !loaded.contains(t))
        .collect();
    // A seeded table the export has no rows for keeps its migration rows.
    let replaced: Vec<&'static str> = loaded
        .iter()
        .copied()
        .filter(|t| {
            !SEEDED_TABLES.contains(t) || manifest.tables.iter().any(|e| e.name == *t && e.rows > 0)
        })
        .collect();

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    if args.migrate {
        println!("running schema migrations...");
        run_migrations(&pool)
            .await
            .context("failed to run migrations")?;
    }

    let mut tx = pool.begin().await.context("failed to begin transaction")?;

    // Disable user-defined triggers (audit log) and FK enforcement for the
    // duration of the restore. The export order + FK-intact source data keep
    // referential integrity; re-firing triggers would pollute audit_log.
    tx.execute("SET LOCAL session_replication_role = replica")
        .await
        .context("failed to set session_replication_role")?;

    for &table in &replaced {
        if SEEDED_TABLES.contains(&table) {
            continue;
        }
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&mut *tx)
            .await
            .with_context(|| format!("failed to count {table}"))?;

        if count > 0 {
            if args.truncate {
                // Single pass at the start: truncate every table, cascaded, in
                // one statement so we don't fight FK ordering.
                break;
            }
            bail!(
                "refusing to import: table {table} already has {count} rows. \
                 Re-run with --truncate to replace existing data."
            );
        }
    }

    if !skipped.is_empty() {
        println!(
            "not in this export, left as they are: {}",
            skipped.join(", ")
        );
    }

    if args.truncate {
        let edges = foreign_keys(&mut tx).await?;
        let cascaded: Vec<&str> = closure(replaced.iter().copied(), &edges, false)
            .into_iter()
            .filter(|t| !replaced.contains(t))
            .collect();
        if !cascaded.is_empty() {
            println!(
                "CASCADE also empties tables this export does not restore: {}",
                cascaded.join(", ")
            );
        }
        let list = replaced.join(", ");
        let sql = format!("TRUNCATE {list} RESTART IDENTITY CASCADE");
        tx.execute(sql.as_str()).await.context("TRUNCATE failed")?;
        println!("truncated {} tables", replaced.len());
    } else {
        // Every other table was just checked empty, so nothing still points
        // at the seeded rows this replaces.
        for &table in SEEDED_TABLES.iter().filter(|t| replaced.contains(t)) {
            tx.execute(format!("DELETE FROM {table}").as_str())
                .await
                .with_context(|| format!("failed to clear seeded {table}"))?;
        }
    }

    let total_start = Instant::now();
    let mut total_rows: u64 = 0;

    for &table in &loaded {
        let path = args.in_dir.join(format!("{table}.jsonl"));
        if !path.exists() {
            // Export always writes a file per table (possibly empty). Missing
            // file means corrupted archive; bail rather than silently skip.
            bail!("missing file {}", path.display());
        }

        let started = Instant::now();
        let mut rows_loaded: u64 = 0;
        let file =
            File::open(&path).with_context(|| format!("failed to open {}", path.display()))?;
        let reader = BufReader::with_capacity(1 << 20, file);

        let mut batch: Vec<Value> = Vec::with_capacity(args.batch_size);
        let mut batch_bytes: usize = 0;
        let insert_sql = format!(
            "INSERT INTO {table} \
             SELECT * FROM jsonb_populate_recordset(NULL::{table}, $1::jsonb)"
        );

        for line in reader.lines() {
            let line = line.with_context(|| format!("read error in {}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let mut v: Value = serde_json::from_str(&line)
                .with_context(|| format!("invalid JSON row in {}", path.display()))?;
            upgrade_legacy_row(table, &mut v);
            // Flush BEFORE this row would push the payload past the byte cap,
            // so every shipped batch stays under it (a lone oversized row
            // still ships by itself).
            if !batch.is_empty() && batch_bytes + line.len() > MAX_BATCH_BYTES {
                flush_batch(&mut tx, &insert_sql, &mut batch).await?;
                batch_bytes = 0;
            }
            batch_bytes += line.len();
            batch.push(v);

            if batch.len() >= args.batch_size {
                flush_batch(&mut tx, &insert_sql, &mut batch).await?;
                batch_bytes = 0;
            }
            rows_loaded += 1;
        }
        if !batch.is_empty() {
            flush_batch(&mut tx, &insert_sql, &mut batch).await?;
        }

        println!(
            "  imported {table:<32} {rows_loaded:>10} rows  ({:.2}s)",
            started.elapsed().as_secs_f64()
        );
        total_rows += rows_loaded;
    }

    // Advance sequences so future INSERTs don't collide with restored ids.
    for &(table, col) in SERIAL_COLUMNS {
        let sql = format!(
            "SELECT setval(pg_get_serial_sequence('{table}', '{col}'), \
             COALESCE((SELECT MAX({col}) FROM {table}), 1), \
             (SELECT MAX({col}) FROM {table}) IS NOT NULL)"
        );
        sqlx::query(&sql)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("failed to reset sequence for {table}.{col}"))?;
    }

    tx.commit().await.context("failed to commit import")?;

    println!(
        "\nImported {} tables, {} rows in {:.2}s",
        loaded.len(),
        total_rows,
        total_start.elapsed().as_secs_f64()
    );

    // Sanity-check counts against manifest (post-commit; informational only).
    let conn = pool.acquire().await?;
    verify_counts(conn, &manifest).await?;

    Ok(())
}

async fn flush_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sql: &str,
    batch: &mut Vec<Value>,
) -> Result<()> {
    let array = Value::Array(std::mem::take(batch));
    sqlx::query(sql)
        .bind(Json(array))
        .execute(&mut **tx)
        .await
        .context("batch insert failed")?;
    Ok(())
}

async fn verify_counts(
    mut conn: sqlx::pool::PoolConnection<sqlx::Postgres>,
    manifest: &Manifest,
) -> Result<()> {
    let mut mismatches = 0u32;
    for entry in &manifest.tables {
        let actual: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", entry.name))
            .fetch_one(&mut *conn)
            .await?;
        if actual as u64 != entry.rows {
            eprintln!(
                "  count mismatch on {}: manifest={} actual={}",
                entry.name, entry.rows, actual
            );
            mismatches += 1;
        }
    }
    if mismatches > 0 {
        bail!("{mismatches} table(s) failed count verification");
    }
    println!("verified row counts for {} tables", manifest.tables.len());
    Ok(())
}

fn load_manifest(dir: &Path) -> Result<Manifest> {
    let path = dir.join(MANIFEST_FILE);
    let f = File::open(&path)
        .with_context(|| format!("failed to open manifest at {}", path.display()))?;
    let m: Manifest = serde_json::from_reader(BufReader::new(f))
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(m)
}

/// The manifest must list known tables in `TABLES` order (FK order, so the
/// load never meets a child before its parent), each at most once.
fn verify_manifest_tables(manifest: &Manifest) -> Result<()> {
    let mut rest = TABLES.iter();
    for entry in &manifest.tables {
        if !TABLES.contains(&entry.name.as_str()) {
            bail!(
                "manifest lists {}, which this binary does not know; \
                 was the export made by a newer build?",
                entry.name
            );
        }
        if !rest.any(|t| *t == entry.name) {
            bail!(
                "manifest lists {} out of order or twice; expected a subsequence of {TABLES:?}",
                entry.name
            );
        }
    }
    for entry in &manifest.tables {
        if entry.file != format!("{}.jsonl", entry.name) {
            bail!(
                "unexpected filename in manifest for {}: {}",
                entry.name,
                entry.file
            );
        }
    }
    Ok(())
}

fn parse_args() -> Result<Args> {
    let mut in_dir: Option<PathBuf> = None;
    let mut truncate = false;
    let mut migrate = false;
    let mut batch_size = DEFAULT_BATCH_SIZE;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--in" | "-i" => {
                in_dir = Some(PathBuf::from(it.next().context("--in requires a path")?));
            }
            "--truncate" => truncate = true,
            "--migrate" => migrate = true,
            "--batch-size" => {
                batch_size = it
                    .next()
                    .context("--batch-size requires a value")?
                    .parse()
                    .context("--batch-size must be a positive integer")?;
                if batch_size == 0 {
                    bail!("--batch-size must be > 0");
                }
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    Ok(Args {
        in_dir: in_dir.context("missing --in <dir>")?,
        truncate,
        migrate,
        batch_size,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: import-database --in <dir> [--migrate] [--truncate] [--batch-size N]\n\
         \n\
         Replays <dir>/<table>.jsonl (as produced by export-database) into the\n\
         database pointed to by DATABASE_URL, inside a single transaction with\n\
         audit triggers and FK checks suspended.\n\
         \n\
         --migrate   Run v3 schema migrations first (useful on a fresh DB).\n\
         --truncate  TRUNCATE every table the export restores before loading.\n\
         \n\
         An export may cover only some tables; the rest are left alone."
    );
}
