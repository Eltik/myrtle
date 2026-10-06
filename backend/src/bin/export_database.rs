//! Export the entire `PostgreSQL` database to a directory of JSONL files.
//!
//! Usage:
//!   cargo run --release --bin export-database -- --out <dir> [--all | --tables a,b,...]
//!
//! With neither `--all` nor `--tables`, a terminal gets a checklist of tables
//! (all checked); anything else, a cron job or a pipe, exports every table.
//! A selection is widened, after asking, to the tables its foreign keys point
//! at, so restoring it into an empty database leaves no dangling ids.
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).
//!
//! Output layout:
//!   <dir>/
//!     manifest.json     - format version, timestamp, table list with row counts
//!     <table>.jsonl     - one JSON object per row (from `row_to_json(t)`)
//!
//! The export runs inside a single REPEATABLE READ READ ONLY transaction so all
//! tables come from a consistent snapshot. Rows are streamed (never buffered
//! into memory in full) and written through a 1 MiB `BufWriter` per table.

use anyhow::{Context, Result, bail};
use backend::db_export::{FORMAT_VERSION, MANIFEST_FILE, TABLES, closure, foreign_keys};
use chrono::Utc;
use dotenv::dotenv;
use futures_util::TryStreamExt;
use serde::Serialize;
use serde_json::json;
use sqlx::{Row, postgres::PgPoolOptions};
use std::{
    fs::{self, File},
    io::{BufWriter, IsTerminal, Write},
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
        .with_context(|| format!("failed to create output dir {}", args.out_dir.display()))?;

    // One connection: one streaming query at a time inside one transaction. Skips the
    // shared pool so an offline export does not compete with a running server.
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let mut conn = pool.acquire().await?;

    // Chosen before the snapshot opens, so the transaction is not held open
    // while someone reads the checklist.
    let tables = select_tables(&mut conn, &args.selection).await?;

    sqlx::query("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *conn)
        .await
        .context("failed to begin snapshot transaction")?;

    let database_version: String = sqlx::query_scalar("SHOW server_version")
        .fetch_one(&mut *conn)
        .await?;

    let mut entries = Vec::with_capacity(tables.len());
    let total_start = Instant::now();
    let mut total_rows: u64 = 0;

    for &table in &tables {
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
        "\nExported {} tables, {} rows in {:.2}s -> {}",
        tables.len(),
        total_rows,
        total_start.elapsed().as_secs_f64(),
        args.out_dir.display()
    );

    Ok(())
}

enum Selection {
    All,
    Listed(Vec<String>),
    Ask,
}

struct Args {
    out_dir: PathBuf,
    selection: Selection,
}

/// The tables to export, in `TABLES` order so the manifest stays a
/// subsequence of the full list that `import-database` accepts.
async fn select_tables(
    conn: &mut sqlx::PgConnection,
    selection: &Selection,
) -> Result<Vec<&'static str>> {
    let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
    let picked: Vec<&'static str> = match selection {
        Selection::All => return Ok(TABLES.to_vec()),
        Selection::Ask if !interactive => return Ok(TABLES.to_vec()),
        Selection::Listed(names) => {
            let unknown: Vec<&String> = names
                .iter()
                .filter(|n| !TABLES.contains(&n.as_str()))
                .collect();
            if !unknown.is_empty() {
                bail!("not an exported table: {unknown:?}");
            }
            TABLES
                .iter()
                .copied()
                .filter(|t| names.iter().any(|n| n == t))
                .collect()
        }
        Selection::Ask => {
            // `reltuples` is the planner's estimate: instant, and close enough
            // to tell a 600k-row table from an empty one. -1 = never analyzed.
            let estimates: Vec<(String, f32)> = sqlx::query_as(
                "SELECT relname::text, reltuples FROM pg_class \
                 WHERE relnamespace = 'public'::regnamespace AND relkind = 'r'",
            )
            .fetch_all(&mut *conn)
            .await?;
            let labels: Vec<String> = TABLES
                .iter()
                .map(|t| {
                    let rows = estimates
                        .iter()
                        .find(|(n, _)| n == t)
                        .map_or(-1.0, |(_, r)| *r);
                    if rows < 0.0 {
                        format!("{t:<32}      ? rows")
                    } else {
                        format!("{t:<32} ~{:>6} rows", rows as u64)
                    }
                })
                .collect();
            let chosen = dialoguer::MultiSelect::new()
                .with_prompt("Tables to export (space toggles, a toggles all, enter confirms)")
                .items(&labels)
                .defaults(&vec![true; TABLES.len()])
                .max_length(20)
                .interact()?;
            chosen.into_iter().map(|i| TABLES[i]).collect()
        }
    };
    if picked.is_empty() {
        bail!("no tables selected");
    }

    let edges = foreign_keys(conn).await?;
    let needed = closure(picked.iter().copied(), &edges, true);
    let missing: Vec<&'static str> = TABLES
        .iter()
        .copied()
        .filter(|t| needed.contains(t) && !picked.contains(t))
        .collect();
    if missing.is_empty() {
        return Ok(picked);
    }
    let add = if interactive {
        dialoguer::Confirm::new()
            .with_prompt(format!(
                "The selection references {}. Export those too?",
                missing.join(", ")
            ))
            .default(true)
            .interact()?
    } else {
        // `--tables` from a script: say so, and export exactly what was asked.
        eprintln!(
            "note: the selection references tables it leaves out: {}",
            missing.join(", ")
        );
        false
    };
    Ok(TABLES
        .iter()
        .copied()
        .filter(|t| picked.contains(t) || (add && missing.contains(t)))
        .collect())
}

fn parse_args() -> Result<Args> {
    let mut out_dir: Option<PathBuf> = None;
    let mut selection = Selection::Ask;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--out" | "-o" => {
                out_dir = Some(PathBuf::from(it.next().context("--out requires a path")?));
            }
            "--all" => selection = Selection::All,
            "--tables" | "-t" => {
                let list = it.next().context("--tables requires a list")?;
                selection = Selection::Listed(
                    list.split(',')
                        .map(str::trim)
                        .filter(|t| !t.is_empty())
                        .map(str::to_owned)
                        .collect(),
                );
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => bail!("unknown argument: {other}"),
        }
    }
    Ok(Args {
        out_dir: out_dir.context("missing --out <dir>")?,
        selection,
    })
}

fn print_usage() {
    eprintln!(
        "Usage: export-database --out <dir> [--all | --tables a,b,...]\n\
         \n\
         Streams tables to <dir>/<table>.jsonl plus a manifest.json,\n\
         all from a single REPEATABLE READ snapshot.\n\
         \n\
         --all       Every table, no prompt.\n\
         --tables    Only these tables (comma-separated), no prompt.\n\
         (neither)   Pick from a checklist in a terminal; every table otherwise."
    );
}
