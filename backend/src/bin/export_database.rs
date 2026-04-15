//! Export the entire PostgreSQL database to a directory of JSONL files.
//!
//! Usage:
//!   cargo run --release --bin export-database -- --out <dir>
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).
//!
//! Output layout:
//!   <dir>/
//!     manifest.json     — format version, timestamp, table list with row counts
//!     <table>.jsonl     — one JSON object per row (from `row_to_json(t)`)
//!
//! The export runs inside a single REPEATABLE READ READ ONLY transaction so all
//! tables come from a consistent snapshot. Rows are streamed (never buffered
//! into memory in full) and written through a 1 MiB BufWriter per table.

use anyhow::{Context, Result};
use backend::db_export::{FORMAT_VERSION, MANIFEST_FILE, TABLES};
use chrono::Utc;
use dotenv::dotenv;
use futures_util::TryStreamExt;
use serde::Serialize;
use serde_json::json;
use sqlx::{Row, postgres::PgPoolOptions};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Serialize)]
struct TableEntry {
    name: String,
    rows: u64,
    file: String,
}

#[derive(Serialize)]
struct Manifest {
    format_version: u32,
    exported_at: String,
    database_version: String,
    tables: Vec<TableEntry>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let args = parse_args()?;

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("failed to create output dir {:?}", args.out_dir))?;

    // A single connection is enough — one streaming query at a time inside one
    // transaction. Avoid the shared pool so we don't compete with a running
    // server (this binary is intended for offline use, but be polite anyway).
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let mut conn = pool.acquire().await?;

    // Snapshot isolation across the whole export.
    sqlx::query("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *conn)
        .await
        .context("failed to begin snapshot transaction")?;

    let database_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&mut *conn)
        .await?;

    let mut entries = Vec::with_capacity(TABLES.len());
    let total_start = Instant::now();
    let mut total_rows: u64 = 0;

    for &table in TABLES {
        let file_name = format!("{table}.jsonl");
        let path = args.out_dir.join(&file_name);
        let file =
            File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
        let mut w = BufWriter::with_capacity(1 << 20, file);

        let started = Instant::now();
        let mut rows: u64 = 0;

        // `row_to_json(t)::text` lets us stream the row's JSON encoding as a
        // String straight to disk, with no JSON parse round-trip on our side.
        let sql = format!("SELECT row_to_json(t)::text AS j FROM {table} t");
        let mut stream = sqlx::query(&sql).fetch(&mut *conn);

        while let Some(row) = stream
            .try_next()
            .await
            .with_context(|| format!("failed to read {table}"))?
        {
            let j: String = row.try_get("j")?;
            w.write_all(j.as_bytes())?;
            w.write_all(b"\n")?;
            rows += 1;
        }
        drop(stream);
        w.flush()?;

        println!(
            "  exported {table:<32} {rows:>10} rows  ({:.2}s)",
            started.elapsed().as_secs_f64()
        );
        total_rows += rows;
        entries.push(TableEntry {
            name: table.to_string(),
            rows,
            file: file_name,
        });
    }

    sqlx::query("COMMIT").execute(&mut *conn).await.ok();

    let manifest = Manifest {
        format_version: FORMAT_VERSION,
        exported_at: Utc::now().to_rfc3339(),
        database_version,
        tables: entries,
    };
    let manifest_path = args.out_dir.join(MANIFEST_FILE);
    let f = File::create(&manifest_path)?;
    serde_json::to_writer_pretty(BufWriter::new(f), &json!(manifest))?;

    println!(
        "\nExported {} tables, {} rows in {:.2}s → {}",
        TABLES.len(),
        total_rows,
        total_start.elapsed().as_secs_f64(),
        args.out_dir.display()
    );

    Ok(())
}

struct Args {
    out_dir: PathBuf,
}

fn parse_args() -> Result<Args> {
    let mut out_dir: Option<PathBuf> = None;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--out" | "-o" => {
                out_dir = Some(PathBuf::from(it.next().context("--out requires a path")?));
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }
    Ok(Args {
        out_dir: out_dir.context("missing --out <dir>")?,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: export-database --out <dir>\n\
         \n\
         Streams every table to <dir>/<table>.jsonl plus a manifest.json,\n\
         all from a single REPEATABLE READ snapshot."
    );
}
