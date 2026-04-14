//! Import a JSONL export produced by `export-database` back into PostgreSQL.
//!
//! Usage:
//!   cargo run --release --bin import-database -- --in <dir> [--truncate] [--batch-size N]
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).
//!
//! Strategy:
//!   - Wraps the entire import in ONE transaction — all or nothing.
//!   - Sets `session_replication_role = replica` so audit triggers and FK
//!     constraints are not re-fired while restoring (the export is already a
//!     consistent snapshot; FKs are re-validated implicitly by the ordering).
//!   - Each batch is shipped as a single JSONB array parameter, then unpacked
//!     server-side with `jsonb_populate_recordset(null::<table>, $1)`. This lets
//!     Postgres infer every column type from the live table schema — no code
//!     changes needed when a new column is added, as long as the JSON keys match.
//!   - After loading, `setval(pg_get_serial_sequence(...))` advances each
//!     BIGSERIAL sequence past the largest imported id.
//!
//! Safety:
//!   - Refuses to run if any non-empty target table exists unless `--truncate`
//!     is passed (aside from the seeded `servers` table, which is always
//!     ON CONFLICT DO NOTHING-friendly).
//!   - `--truncate` uses `TRUNCATE ... RESTART IDENTITY CASCADE` inside the
//!     transaction so a failure rolls back to the pre-import state.

use anyhow::{Context, Result, bail};
use backend::db_export::{FORMAT_VERSION, MANIFEST_FILE, SERIAL_COLUMNS, TABLES};
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

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let mut tx = pool.begin().await.context("failed to begin transaction")?;

    // Disable user-defined triggers (audit log) and FK enforcement for the
    // duration of the restore. The export order + FK-intact source data keep
    // referential integrity; re-firing triggers would pollute audit_log.
    tx.execute("SET LOCAL session_replication_role = replica")
        .await
        .context("failed to set session_replication_role")?;

    // Pre-flight: ensure every target table is either empty or we have
    // permission to truncate it.
    for &table in TABLES {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&mut *tx)
            .await
            .with_context(|| format!("failed to count {table}"))?;

        if count > 0 {
            if args.truncate {
                // Single pass at the start: truncate every table, cascaded, in
                // one statement so we don't fight FK ordering.
                break;
            } else {
                bail!(
                    "refusing to import: table {table} already has {count} rows. \
                     Re-run with --truncate to replace existing data."
                );
            }
        }
    }

    if args.truncate {
        let list = TABLES.join(", ");
        let sql = format!("TRUNCATE {list} RESTART IDENTITY CASCADE");
        tx.execute(sql.as_str()).await.context("TRUNCATE failed")?;
        println!("truncated {} tables", TABLES.len());
    }

    let total_start = Instant::now();
    let mut total_rows: u64 = 0;

    for &table in TABLES {
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
        let insert_sql = format!(
            "INSERT INTO {table} \
             SELECT * FROM jsonb_populate_recordset(NULL::{table}, $1::jsonb)"
        );

        for line in reader.lines() {
            let line = line.with_context(|| format!("read error in {}", path.display()))?;
            if line.trim().is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(&line)
                .with_context(|| format!("invalid JSON row in {}", path.display()))?;
            batch.push(v);

            if batch.len() >= args.batch_size {
                flush_batch(&mut tx, &insert_sql, &mut batch).await?;
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
        TABLES.len(),
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

fn verify_manifest_tables(manifest: &Manifest) -> Result<()> {
    let manifest_names: Vec<&str> = manifest.tables.iter().map(|t| t.name.as_str()).collect();
    if manifest_names != TABLES {
        bail!(
            "manifest table list does not match current binary.\n  manifest: {:?}\n  expected: {:?}",
            manifest_names,
            TABLES
        );
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
    let mut batch_size = DEFAULT_BATCH_SIZE;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--in" | "-i" => {
                in_dir = Some(PathBuf::from(it.next().context("--in requires a path")?));
            }
            "--truncate" => truncate = true,
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
        batch_size,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: import-database --in <dir> [--truncate] [--batch-size N]\n\
         \n\
         Replays <dir>/<table>.jsonl (as produced by export-database) into the\n\
         database pointed to by DATABASE_URL, inside a single transaction with\n\
         audit triggers and FK checks suspended."
    );
}
