use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use flate2::read::MultiGzDecoder;
use myrtle_db::tools::{self, Column, Table};
use myrtle_db::{Conn, Db, migrate};
use serde_json::Value;

use super::{
    DumpError, Manifest, SEEDED_TABLES, SERIAL_COLUMNS, TABLES, TableEntry, check_manifest,
    closure, file_name, read_manifest, upgrade_legacy_row,
};

const MAX_BATCH_BYTES: usize = 64 * 1024 * 1024;

pub const DEFAULT_BATCH_SIZE: NonZeroUsize = match NonZeroUsize::new(1000) {
    Some(n) => n,
    None => unreachable!(),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportOptions {
    pub truncate: bool,
    pub cascade: bool,
    pub migrate: bool,
    pub batch_size: NonZeroUsize,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            truncate: false,
            cascade: false,
            migrate: false,
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub loaded: Vec<TableEntry>,
    pub skipped: Vec<&'static str>,
    pub kept_seeded: Vec<&'static str>,
    pub cascaded: Vec<String>,
    pub truncated: usize,
    pub elapsed: Duration,
    pub mismatches: Vec<Mismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    pub table: String,
    pub manifest: u64,
    pub actual: i64,
}

pub async fn import(
    db: &Db,
    dir: &Path,
    options: ImportOptions,
) -> Result<ImportReport, DumpError> {
    let manifest = read_manifest(dir)?;
    check_manifest(&manifest)?;
    for entry in &manifest.tables {
        data_file(dir, &entry.file)?;
    }
    let plan = Plan::new(&manifest);

    if options.migrate {
        tracing::info!("running schema migrations");
        migrate::run(db).await?;
    }

    let started = Instant::now();
    let mut tx = db.begin().await?;
    tools::suspend_triggers(&mut tx).await?;

    if !options.truncate {
        for &name in plan.replaced.iter().filter(|t| !SEEDED_TABLES.contains(t)) {
            let table = live(&mut tx, name).await?;
            let count = tools::count_rows(&mut tx, &table).await?;
            if count > 0 {
                return Err(DumpError::Invalid(format!(
                    "refusing to import: table {name} already has {count} rows. \
                     Re-run with --truncate to replace existing data."
                )));
            }
        }
    }

    let mut cascaded = Vec::new();
    let mut replaced = Vec::with_capacity(plan.replaced.len());
    for &name in &plan.replaced {
        replaced.push(live(&mut tx, name).await?);
    }
    let edges = tools::foreign_keys(&mut tx).await?;
    if options.truncate {
        cascaded = closure(plan.replaced.iter().copied(), &edges, false)
            .into_iter()
            .filter(|t| !plan.replaced.contains(t))
            .map(str::to_owned)
            .collect();
        let occupied = occupied(&mut tx, &cascaded).await?;
        if !occupied.is_empty() && !options.cascade {
            return Err(DumpError::Invalid(format!(
                "refusing to import: TRUNCATE ... CASCADE would also empty tables this export \
                 does not restore: {}. Re-run with --cascade to empty them.",
                occupied.join(", ")
            )));
        }
        tools::truncate_cascade(&mut tx, &replaced).await?;
    } else {
        for table in replaced
            .iter()
            .filter(|t| SEEDED_TABLES.contains(&t.name()))
        {
            let children: Vec<String> = edges
                .iter()
                .filter(|e| e.parent == table.name() && !plan.replaced.contains(&e.child.as_str()))
                .map(|e| e.child.clone())
                .collect();
            let occupied = occupied(&mut tx, &children).await?;
            if !occupied.is_empty() {
                return Err(DumpError::Invalid(format!(
                    "refusing to import: replacing {} would orphan rows in tables this export \
                     does not restore: {}. Re-run with --truncate to replace existing data.",
                    table.name(),
                    occupied.join(", ")
                )));
            }
            tools::delete_all(&mut tx, table).await?;
        }
    }

    let mut loaded = Vec::with_capacity(plan.loaded.len());
    for &name in &plan.loaded {
        let table_started = Instant::now();
        let table = live(&mut tx, name).await?;
        let rows = load(
            &mut tx,
            &table,
            &data_file(dir, &file_name(name))?,
            options.batch_size,
        )
        .await?;
        tracing::info!(
            table = name,
            rows,
            secs = table_started.elapsed().as_secs_f64(),
            "imported"
        );
        loaded.push(TableEntry::new(name, rows));
    }

    for &(name, column) in SERIAL_COLUMNS
        .iter()
        .filter(|(t, _)| plan.replaced.contains(t))
    {
        let table = live(&mut tx, name).await?;
        let col = tools::columns(&mut tx, &table)
            .await?
            .into_iter()
            .find(|c| c.name() == column)
            .ok_or_else(|| DumpError::Invalid(format!("{name} has no column {column}")))?;
        tools::reset_sequence(&mut tx, &table, &col).await?;
    }
    tx.commit().await?;
    let elapsed = started.elapsed();

    let mut conn = db.conn().await?;
    let mut mismatches = Vec::new();
    for entry in manifest
        .tables
        .iter()
        .filter(|e| plan.replaced.contains(&e.name.as_str()))
    {
        let table = live(&mut conn, &entry.name).await?;
        let actual = tools::count_rows(&mut conn, &table).await?;
        if u64::try_from(actual).ok() != Some(entry.rows) {
            mismatches.push(Mismatch {
                table: entry.name.clone(),
                manifest: entry.rows,
                actual,
            });
        }
    }

    Ok(ImportReport {
        loaded,
        skipped: plan.skipped,
        kept_seeded: plan.kept_seeded,
        cascaded,
        truncated: if options.truncate { replaced.len() } else { 0 },
        elapsed,
        mismatches,
    })
}

struct Plan {
    loaded: Vec<&'static str>,
    replaced: Vec<&'static str>,
    skipped: Vec<&'static str>,
    kept_seeded: Vec<&'static str>,
}

impl Plan {
    fn new(manifest: &Manifest) -> Self {
        let entry = |t: &str| manifest.tables.iter().find(|e| e.name == t);
        let loaded: Vec<&'static str> = TABLES
            .iter()
            .copied()
            .filter(|t| entry(t).is_some())
            .collect();
        let (replaced, kept_seeded) = loaded
            .iter()
            .partition(|t| !SEEDED_TABLES.contains(t) || entry(t).is_some_and(|e| e.rows > 0));
        Self {
            skipped: TABLES
                .iter()
                .copied()
                .filter(|t| !loaded.contains(t))
                .collect(),
            loaded,
            replaced,
            kept_seeded,
        }
    }
}

fn data_file(dir: &Path, file: &str) -> Result<PathBuf, DumpError> {
    let plain = dir.join(file);
    let gzipped = dir.join(format!("{file}.gz"));
    if plain.is_file() {
        Ok(plain)
    } else if gzipped.is_file() {
        Ok(gzipped)
    } else {
        Err(DumpError::Invalid(format!(
            "missing file {}",
            plain.display()
        )))
    }
}

async fn live(conn: &mut Conn, name: &str) -> Result<Table, DumpError> {
    tools::table(conn, name)
        .await?
        .ok_or_else(|| DumpError::Invalid(format!("the database has no table {name}")))
}

async fn occupied(conn: &mut Conn, names: &[String]) -> Result<Vec<String>, DumpError> {
    let mut found = Vec::new();
    for name in names {
        let table = live(conn, name).await?;
        let count = tools::count_rows(conn, &table).await?;
        if count > 0 {
            found.push(format!("{name} ({count} rows)"));
        }
    }
    Ok(found)
}

async fn load(
    conn: &mut Conn,
    table: &Table,
    path: &Path,
    batch_size: NonZeroUsize,
) -> Result<u64, DumpError> {
    let columns = tools::columns(conn, table).await?;
    let file = File::open(path).map_err(DumpError::io(path))?;
    let reader: Box<dyn BufRead + Send> = if path.extension().is_some_and(|e| e == "gz") {
        Box::new(BufReader::with_capacity(1 << 20, MultiGzDecoder::new(file)))
    } else {
        Box::new(BufReader::with_capacity(1 << 20, file))
    };
    let mut batch: Vec<Value> = Vec::with_capacity(batch_size.get().min(1 << 16));
    let mut batch_bytes = 0;
    let mut rows = 0;
    for line in reader.lines() {
        let line = line.map_err(DumpError::io(path))?;
        if line.trim().is_empty() {
            continue;
        }
        let mut row: Value = serde_json::from_str(&line).map_err(DumpError::json(path))?;
        upgrade_legacy_row(table.name(), &mut row);
        if !batch.is_empty() && batch_bytes + line.len() > MAX_BATCH_BYTES {
            rows += flush(conn, table, &columns, &mut batch).await?;
            batch_bytes = 0;
        }
        batch_bytes += line.len();
        batch.push(row);
        if batch.len() >= batch_size.get() {
            rows += flush(conn, table, &columns, &mut batch).await?;
            batch_bytes = 0;
        }
    }
    if !batch.is_empty() {
        rows += flush(conn, table, &columns, &mut batch).await?;
    }
    Ok(rows)
}

async fn flush(
    conn: &mut Conn,
    table: &Table,
    columns: &[Column],
    batch: &mut Vec<Value>,
) -> Result<u64, DumpError> {
    let present: Vec<Column> = columns
        .iter()
        .filter(|c| batch.iter().any(|r| r.get(c.name()).is_some()))
        .cloned()
        .collect();
    if present.is_empty() {
        return Err(DumpError::Invalid(format!(
            "{}: rows share no column with the table",
            table.name()
        )));
    }
    let rows = Value::Array(std::mem::take(batch));
    tools::insert_rows(conn, table, &present, None, &rows)
        .await
        .map_err(|source| DumpError::Table {
            table: table.name().to_owned(),
            source,
        })
}
