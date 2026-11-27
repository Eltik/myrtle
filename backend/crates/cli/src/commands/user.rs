use std::path::PathBuf;

use anyhow::{Context, bail};
use clap::{Args, Subcommand};
use myrtle_app::services::accounts::export::{
    self, ImportOptions, TableLoad, export_user, import_user,
};
use myrtle_db::Db;
use myrtle_db::repos::users::{self, AccountRow};
use myrtle_types::{Server, UserId};

use crate::Env;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write one account and every table that it uses to a JSON file
    Export(ExportArgs),
    /// Replace the account matched by (uid, server) with a file's rows
    Import(ImportArgs),
}

#[derive(Debug, Args)]
pub struct Who {
    /// Arknights uid
    #[arg(long, conflicts_with = "user_id", required_unless_present = "user_id")]
    pub uid: Option<String>,
    /// users.id
    #[arg(long, value_parser = parse_user_id)]
    pub user_id: Option<UserId>,
    /// Server code
    #[arg(long)]
    pub server: Option<String>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[command(flatten)]
    pub who: Who,
    /// Output file; default user_<uid>_<server>_<unix>.json
    #[arg(long, short)]
    pub output: Option<PathBuf>,
    /// Also export encrypted game credentials
    #[arg(long)]
    pub include_credentials: bool,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// Export file to load
    #[arg(long, short)]
    pub file: PathBuf,
    /// Overwrite an account
    #[arg(long)]
    pub replace: bool,
    #[arg(long)]
    pub dry_run: bool,
    /// Suspend audit triggers and foreign-key checks for the load
    #[arg(long)]
    pub no_triggers: bool,
}

pub fn parse_user_id(s: &str) -> Result<UserId, String> {
    s.parse()
        .map(UserId)
        .map_err(|e| format!("not a UUID: {e}"))
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Export(args) => run_export(env, args).await,
        Command::Import(args) => run_import(env, args).await,
    }
}

async fn run_export(env: &Env, args: ExportArgs) -> anyhow::Result<()> {
    let db = env.db().await?;
    let account = args.who.resolve(&db).await?;
    println!(
        "exporting {} (uid={} server={} id={})",
        account.nickname.as_deref().unwrap_or("<no nickname>"),
        account.uid,
        account.server,
        account.id
    );
    let file = export_user(&db, UserId(account.id), args.include_credentials).await?;
    let mut total = 0;
    for table in export::user_tables(args.include_credentials) {
        let rows = file.counts.get(table.name).copied().unwrap_or_default();
        total += rows;
        println!("  {:<24} {rows:>7} rows", table.name);
    }
    let output = args.output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "user_{}_{}_{}.json",
            account.uid,
            account.server,
            chrono::Utc::now().timestamp()
        ))
    });
    let json = serde_json::to_vec_pretty(&file)?;
    tokio::fs::write(&output, &json)
        .await
        .with_context(|| format!("write {}", output.display()))?;
    println!(
        "\nwrote {total} rows across {} tables ({} KiB) -> {}",
        file.tables.len(),
        json.len().div_ceil(1024),
        output.display()
    );
    if args.include_credentials {
        println!(
            "WARNING: this file holds the user's encrypted game credentials; they decrypt only \
             under the same credential key and the same users.id"
        );
    }
    Ok(())
}

async fn run_import(env: &Env, args: ImportArgs) -> anyhow::Result<()> {
    let bytes = tokio::fs::read(&args.file)
        .await
        .with_context(|| format!("read {}", args.file.display()))?;
    let file = export::parse_export(&bytes).with_context(|| args.file.display().to_string())?;
    println!(
        "file: {} (exported {} from {} uid={} server={})",
        args.file.display(),
        file.exported_at,
        file.source.id,
        file.source.uid,
        file.source.server_code
    );
    let db = env.db().await?;
    let options = ImportOptions {
        replace: args.replace,
        dry_run: args.dry_run,
        suspend_triggers: args.no_triggers,
    };
    let report = import_user(&db, &file, options).await?;
    match report.existing {
        Some((id, rows)) => println!("target: existing user id={} ({rows} rows replaced)", id.0),
        None => println!("target: no such user yet, created"),
    }
    println!("anchor: users.id = {}", report.target.0);
    println!("cleared {} existing rows", report.cleared);
    for (name, load) in &report.tables {
        match load {
            TableLoad::Absent => println!("  {name:<24} {:>7}  (absent from file)", "-"),
            TableLoad::Loaded { inserted, skipped } if *skipped > 0 => println!(
                "  {name:<24} {inserted:>7} rows  ({skipped} skipped: referenced row absent)"
            ),
            TableLoad::Loaded { inserted, .. } => println!("  {name:<24} {inserted:>7} rows"),
        }
    }
    let (inserted, skipped) = (report.inserted(), report.skipped());
    if !report.committed {
        println!(
            "\nDRY RUN: rolled back. Would have written {inserted} rows (and skipped {skipped}) \
             for uid={}",
            file.source.uid
        );
        return Ok(());
    }
    println!(
        "\nimported {inserted} rows for uid={} (users.id={})",
        file.source.uid, report.target.0
    );
    if skipped > 0 {
        println!(
            "{skipped} row(s) were skipped because the tier list they reference does not exist here"
        );
    }
    if file.includes_credentials && report.target.0 != file.source.id {
        println!(
            "NOTE: users.id changed ({} -> {}), so the imported game credentials cannot be \
             decrypted: they are sealed against the original id",
            file.source.id, report.target.0
        );
    }
    Ok(())
}

impl Who {
    pub async fn resolve(&self, db: &Db) -> anyhow::Result<AccountRow> {
        let server = self
            .server
            .as_deref()
            .map(|code| Server::parse(code).with_context(|| format!("unknown server {code:?}")))
            .transpose()?;
        if let Some(id) = self.user_id {
            let account = users::account(&mut *db.conn().await?, id)
                .await?
                .with_context(|| format!("no user with id {}", id.0))?;
            if let Some(server) = server
                && account.server_id != server.index()
            {
                bail!(
                    "user {} is on {}, not {}",
                    id.0,
                    account.server,
                    server.as_str()
                );
            }
            return Ok(account);
        }
        let uid = self.uid.as_deref().context("pass --uid or --user-id")?;
        crate::env::find_account(db, uid, server).await
    }
}
