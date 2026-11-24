use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use myrtle_db::tools;
use myrtle_db::{Conn, Db, DbError};

use super::{
    DumpError, FORMAT_VERSION, MANIFEST_FILE, Manifest, TABLES, TableEntry, closure, file_name,
    write_manifest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    All,
    Tables {
        names: Vec<String>,
        with_parents: bool,
    },
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub manifest: Manifest,
    pub elapsed: Duration,
    pub left_out: Vec<&'static str>,
    pub added: Vec<&'static str>,
}

pub async fn export(db: &Db, dir: &Path, selection: &Selection) -> Result<ExportReport, DumpError> {
    fs::create_dir_all(dir).map_err(DumpError::io(dir))?;
    let started = Instant::now();
    let mut conn = db.conn().await?;
    let picked = select(&mut conn, selection).await?;
    let stale = dir.join(MANIFEST_FILE);
    match fs::remove_file(&stale) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            return Err(DumpError::io(&stale)(e));
        }
        _ => {}
    }

    let mut tx = tools::begin_snapshot(&mut conn).await?;
    let database_version = tools::server_version(&mut tx).await?;
    let mut entries = Vec::with_capacity(picked.tables.len());
    for &name in &picked.tables {
        let table_started = Instant::now();
        let table = tools::table(&mut tx, name)
            .await?
            .ok_or_else(|| DumpError::Invalid(format!("the database has no table {name}")))?;
        let path = dir.join(file_name(name));
        let file = File::create(&path).map_err(DumpError::io(&path))?;
        let mut w = BufWriter::with_capacity(1 << 20, file);
        let mut rows: u64 = 0;
        tools::declare_json_rows(&mut tx, &table).await?;
        loop {
            let batch = tools::fetch_json_rows(&mut tx).await?;
            if batch.is_empty() {
                break;
            }
            for row in &batch {
                w.write_all(row.as_bytes())
                    .and_then(|()| w.write_all(b"\n"))
                    .map_err(DumpError::io(&path))?;
            }
            rows += batch.len() as u64;
        }
        w.flush().map_err(DumpError::io(&path))?;
        tracing::info!(
            table = name,
            rows,
            secs = table_started.elapsed().as_secs_f64(),
            "exported"
        );
        entries.push(TableEntry::new(name, rows));
    }
    tx.commit().await.map_err(DbError::from)?;

    let manifest = Manifest {
        format_version: FORMAT_VERSION,
        exported_at: chrono::Utc::now().to_rfc3339(),
        database_version,
        tables: entries,
    };
    write_manifest(dir, &manifest)?;
    Ok(ExportReport {
        manifest,
        elapsed: started.elapsed(),
        left_out: picked.left_out,
        added: picked.added,
    })
}

struct Picked {
    tables: Vec<&'static str>,
    left_out: Vec<&'static str>,
    added: Vec<&'static str>,
}

async fn select(conn: &mut Conn, selection: &Selection) -> Result<Picked, DumpError> {
    let (names, with_parents) = match selection {
        Selection::All => {
            return Ok(Picked {
                tables: TABLES.to_vec(),
                left_out: Vec::new(),
                added: Vec::new(),
            });
        }
        Selection::Tables {
            names,
            with_parents,
        } => (names, *with_parents),
    };
    let unknown: Vec<&String> = names
        .iter()
        .filter(|n| !TABLES.contains(&n.as_str()))
        .collect();
    if !unknown.is_empty() {
        return Err(DumpError::Invalid(format!(
            "not an exported table: {unknown:?}"
        )));
    }
    let picked: Vec<&'static str> = TABLES
        .iter()
        .copied()
        .filter(|t| names.iter().any(|n| n == t))
        .collect();
    if picked.is_empty() {
        return Err(DumpError::Invalid("no tables selected".to_owned()));
    }

    let edges = tools::foreign_keys(conn).await?;
    let needed = closure(picked.iter().copied(), &edges, true);
    let missing: Vec<&'static str> = TABLES
        .iter()
        .copied()
        .filter(|t| needed.contains(t) && !picked.contains(t))
        .collect();
    if with_parents {
        let tables = TABLES
            .iter()
            .copied()
            .filter(|t| picked.contains(t) || missing.contains(t))
            .collect();
        return Ok(Picked {
            tables,
            left_out: Vec::new(),
            added: missing,
        });
    }
    Ok(Picked {
        tables: picked,
        left_out: missing,
        added: Vec::new(),
    })
}
