use std::num::NonZeroUsize;
use std::path::PathBuf;

use anyhow::bail;
use clap::Subcommand;
use myrtle_app::dump::{self, DEFAULT_BATCH_SIZE, ImportOptions, Selection, TableEntry};

use crate::Env;

#[derive(Debug, Subcommand)]
pub enum Command {
    Export {
        /// Directory to write
        #[arg(long, short)]
        out: PathBuf,
        #[arg(long, short, value_delimiter = ',')]
        tables: Option<Vec<String>>,
        #[arg(long, requires = "tables")]
        with_parents: bool,
    },
    Import {
        #[arg(long = "in", short)]
        input: PathBuf,
        #[arg(long)]
        truncate: bool,
        /// Let the truncate cascade into populated tables the export does not restore
        #[arg(long, requires = "truncate")]
        cascade: bool,
        /// Apply pending migrations first
        #[arg(long)]
        migrate: bool,
        /// Rows per insert
        #[arg(long, default_value_t = DEFAULT_BATCH_SIZE)]
        batch_size: NonZeroUsize,
    },
    TranslateLegacy {
        #[arg(long = "in", short)]
        input: PathBuf,
        #[arg(long, short)]
        out: PathBuf,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Export {
            out,
            tables,
            with_parents,
        } => {
            let selection = tables.map_or(Selection::All, |names| Selection::Tables {
                names: names
                    .iter()
                    .map(|n| n.trim().to_owned())
                    .filter(|n| !n.is_empty())
                    .collect(),
                with_parents,
            });
            let report = dump::export(&env.db().await?, &out, &selection).await?;
            if !report.left_out.is_empty() {
                eprintln!(
                    "note: the selection references tables it leaves out: {}",
                    report.left_out.join(", ")
                );
            }
            if !report.added.is_empty() {
                println!("with parents: {}", report.added.join(", "));
            }
            print_tables("exported", &report.manifest.tables);
            println!(
                "\nExported {} tables, {} rows in {:.2}s -> {}",
                report.manifest.tables.len(),
                total(&report.manifest.tables),
                report.elapsed.as_secs_f64(),
                out.display()
            );
        }
        Command::Import {
            input,
            truncate,
            cascade,
            migrate,
            batch_size,
        } => {
            let options = ImportOptions {
                truncate,
                cascade,
                migrate,
                batch_size,
            };
            let report = dump::import(&env.db().await?, &input, options).await?;
            if !report.skipped.is_empty() {
                println!(
                    "not in this export, left as they are: {}",
                    report.skipped.join(", ")
                );
            }
            if !report.kept_seeded.is_empty() {
                println!(
                    "no rows in this export, migration rows kept: {}",
                    report.kept_seeded.join(", ")
                );
            }
            if !report.cascaded.is_empty() {
                println!(
                    "CASCADE also emptied tables this export does not restore: {}",
                    report.cascaded.join(", ")
                );
            }
            if truncate {
                println!("truncated {} tables", report.truncated);
            }
            print_tables("imported", &report.loaded);
            println!(
                "\nImported {} tables, {} rows in {:.2}s",
                report.loaded.len(),
                total(&report.loaded),
                report.elapsed.as_secs_f64()
            );
            for m in &report.mismatches {
                eprintln!(
                    "  count mismatch on {}: manifest={} actual={}",
                    m.table, m.manifest, m.actual
                );
            }
            if !report.mismatches.is_empty() {
                bail!(
                    "{} table(s) failed count verification",
                    report.mismatches.len()
                );
            }
            println!(
                "verified row counts for {} tables",
                report.loaded.len() - report.kept_seeded.len()
            );
        }
        Command::TranslateLegacy { input, out } => {
            let manifest =
                tokio::task::spawn_blocking(move || dump::translate(&input, &out)).await??;
            for t in &manifest.tables {
                println!("  {:<32} {:>10}", t.name, t.rows);
            }
            println!(
                "\nWrote {} tables, {} rows",
                manifest.tables.len(),
                total(&manifest.tables)
            );
        }
    }
    Ok(())
}

fn print_tables(verb: &str, tables: &[TableEntry]) {
    for t in tables {
        println!("  {verb} {:<32} {:>10} rows", t.name, t.rows);
    }
}

fn total(tables: &[TableEntry]) -> u64 {
    tables.iter().map(|t| t.rows).sum()
}
