use std::path::PathBuf;

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use myrtle_app::context::GameData;
use myrtle_app::services::gacha::rarity::{plan_against, reconcile_against};
use myrtle_db::Db;
use myrtle_db::repos::gacha::{self, GachaRecordRow};
use myrtle_db::repos::users::AccountRow;
use myrtle_types::UserId;
use serde::Serialize;

use super::user::Who;
use crate::Env;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write one user's gacha records to JSON file
    Export(ExportArgs),
    /// Rewrite gacha rarities to match default server's game data
    Resync(ResyncArgs),
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[command(flatten)]
    pub who: Who,
    /// Output file; default gacha_<uid>_<unix>.json
    #[arg(long, short)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ResyncArgs {
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct GachaExport {
    pub user: ExportedUser,
    pub exported_at: String,
    pub record_count: usize,
    pub records: Vec<ExportedRecord>,
}

#[derive(Debug, Serialize)]
pub struct ExportedUser {
    pub id: UserId,
    pub uid: String,
    pub server: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportedRecord {
    pub id: i64,
    pub user_id: UserId,
    pub char_id: String,
    pub pool_id: String,
    pub rarity: i16,
    pub pull_timestamp: i64,
    pub pool_name: Option<String>,
    pub gacha_type: Option<String>,
    pub batch_index: i16,
    pub created_at: DateTime<Utc>,
}

impl From<GachaRecordRow> for ExportedRecord {
    fn from(row: GachaRecordRow) -> Self {
        Self {
            id: row.id,
            user_id: UserId(row.user_id),
            char_id: row.char_id,
            pool_id: row.pool_id,
            rarity: row.rarity,
            pull_timestamp: row.pull_timestamp,
            pool_name: row.pool_name,
            gacha_type: row.gacha_type,
            batch_index: row.batch_index,
            created_at: row.created_at,
        }
    }
}

pub async fn export_document(db: &Db, account: &AccountRow) -> anyhow::Result<GachaExport> {
    let records: Vec<ExportedRecord> =
        gacha::list_in_pools(&mut *db.conn().await?, UserId(account.id), None)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
    Ok(GachaExport {
        user: ExportedUser {
            id: UserId(account.id),
            uid: account.uid.clone(),
            server: account.server.clone(),
            nickname: account.nickname.clone(),
        },
        exported_at: Utc::now().to_rfc3339(),
        record_count: records.len(),
        records,
    })
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Export(args) => run_export(env, args).await,
        Command::Resync(args) => run_resync(env, args).await,
    }
}

async fn run_export(env: &Env, args: ExportArgs) -> anyhow::Result<()> {
    let db = env.db().await?;
    let account = args.who.resolve(&db).await?;
    let document = export_document(&db, &account).await?;
    let output = args.output.unwrap_or_else(|| {
        PathBuf::from(format!(
            "gacha_{}_{}.json",
            account.uid,
            Utc::now().timestamp()
        ))
    });
    tokio::fs::write(&output, serde_json::to_vec_pretty(&document)?)
        .await
        .with_context(|| format!("write {}", output.display()))?;
    println!(
        "exported {} record(s) for uid={} (id={}) to {}",
        document.record_count,
        account.uid,
        account.id,
        output.display()
    );
    Ok(())
}

async fn run_resync(env: &Env, args: ResyncArgs) -> anyhow::Result<()> {
    let ctx = env.ctx(GameData::Default).await?;
    let server = ctx.snapshots.default_server();
    let Some(snapshot) = ctx.snapshots.get(server) else {
        bail!("no game data loaded for {}", server.as_str());
    };
    let operators = &snapshot.model.operators;
    if args.dry_run {
        let plan = plan_against(&ctx, operators).await?;
        println!("distinct (char_id, rarity) pairs: {}", plan.distinct_pairs);
        println!("char_ids that would be fixed:     {}", plan.fixes.len());
        for fix in &plan.fixes {
            println!("  {}: {} -> {}", fix.char_id, fix.from, fix.to);
        }
        println!("(dry run - no rows updated)");
        return Ok(());
    }
    let sync = reconcile_against(&ctx, operators).await?;
    println!("distinct (char_id, rarity) pairs: {}", sync.distinct_pairs);
    println!("char_ids fixed:                   {}", sync.fixed_char_ids);
    println!("rows updated:                     {}", sync.rows_updated);
    Ok(())
}
