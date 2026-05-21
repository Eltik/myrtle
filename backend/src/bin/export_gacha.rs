//! Export a single user's gacha history to a JSON file.
//!
//! Resolves the user by Arknights UID or by internal database UUID, then writes
//! every row in `gacha_records` to disk. The output is a single JSON object
//! containing the resolved user info and the records array.
//!
//! Usage:
//!   cargo run --release --bin export-gacha -- --uid 123456789
//!   cargo run --release --bin export-gacha -- --user-id 8e3b...c1
//!   cargo run --release --bin export-gacha -- --uid 123 --server CN --output out.json
//!
//! When the same Arknights UID exists on multiple servers, pass `--server`
//! (server code, e.g. `EN`/`CN`/`JP`/`KR`) to disambiguate. Otherwise the
//! earliest-created row wins.
//!
//! Reads `DATABASE_URL` from the environment (or `.env`).

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use backend::database::{models::gacha::GachaRecord, queries};
use dotenv::dotenv;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

struct Args {
    uid: Option<String>,
    user_id: Option<Uuid>,
    server: Option<String>,
    output: Option<PathBuf>,
}

fn print_help() {
    eprintln!(
        "Usage: export-gacha [--uid <arknights_uid> | --user-id <uuid>] \
[--server <code>] [--output <path>]\n\
         \n\
         Exports every gacha_records row for the resolved user as JSON.\n\
         \n\
         Options:\n\
           --uid <arknights_uid>   Arknights UID (e.g. 123456789)\n\
           --user-id <uuid>        Internal database UUID\n\
           --server <code>         Disambiguate when the UID exists on multiple servers\n\
           --output <path>         Output file (default: gacha_<uid>_<unix>.json)\n\
           -h, --help              Show this message"
    );
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        uid: None,
        user_id: None,
        server: None,
        output: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--uid" => {
                args.uid = Some(it.next().context("--uid requires a value")?);
            }
            "--user-id" => {
                let raw = it.next().context("--user-id requires a value")?;
                args.user_id =
                    Some(Uuid::parse_str(&raw).context("--user-id must be a valid UUID")?);
            }
            "--server" => {
                args.server = Some(it.next().context("--server requires a value")?);
            }
            "--output" | "-o" => {
                args.output = Some(PathBuf::from(
                    it.next().context("--output requires a value")?,
                ));
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                bail!("unknown arg: {other}");
            }
        }
    }
    if args.uid.is_none() && args.user_id.is_none() {
        print_help();
        bail!("must provide --uid or --user-id");
    }
    if args.uid.is_some() && args.user_id.is_some() {
        bail!("--uid and --user-id are mutually exclusive");
    }
    Ok(args)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct ResolvedUser {
    id: Uuid,
    uid: String,
    server: String,
    nickname: Option<String>,
}

async fn resolve_by_uid(
    pool: &sqlx::PgPool,
    uid: &str,
    server: Option<&str>,
) -> Result<ResolvedUser> {
    let rows: Vec<ResolvedUser> = sqlx::query_as(
        "SELECT u.id, u.uid, s.code AS server, u.nickname \
         FROM users u JOIN servers s ON s.id = u.server_id \
         WHERE u.uid = $1 \
         ORDER BY u.created_at",
    )
    .bind(uid)
    .fetch_all(pool)
    .await
    .context("user lookup by uid failed")?;

    if rows.is_empty() {
        bail!("no user found with uid={uid}");
    }

    if let Some(server) = server {
        rows.into_iter()
            .find(|r| r.server.eq_ignore_ascii_case(server))
            .with_context(|| format!("no user with uid={uid} on server={server}"))
    } else if rows.len() == 1 {
        Ok(rows.into_iter().next().unwrap())
    } else {
        let servers: Vec<String> = rows.iter().map(|r| r.server.clone()).collect();
        eprintln!(
            "uid={uid} exists on multiple servers ({}); using the earliest-created one. \
             Pass --server to choose.",
            servers.join(", ")
        );
        Ok(rows.into_iter().next().unwrap())
    }
}

async fn resolve_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<ResolvedUser> {
    sqlx::query_as::<_, ResolvedUser>(
        "SELECT u.id, u.uid, s.code AS server, u.nickname \
         FROM users u JOIN servers s ON s.id = u.server_id \
         WHERE u.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("user lookup by id failed")?
    .with_context(|| format!("no user found with id={id}"))
}

#[derive(Debug, Serialize)]
struct Export<'a> {
    user: &'a ResolvedUser,
    exported_at: String,
    record_count: usize,
    records: &'a [GachaRecord],
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "export_gacha=info,backend=warn".into()),
        )
        .init();

    let args = parse_args()?;

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;

    let user = if let Some(id) = args.user_id {
        resolve_by_id(&pool, id).await?
    } else {
        resolve_by_uid(&pool, args.uid.as_deref().unwrap(), args.server.as_deref()).await?
    };

    tracing::info!(
        id = %user.id,
        uid = %user.uid,
        server = %user.server,
        nickname = ?user.nickname,
        "resolved user",
    );

    let records = queries::gacha::get_all_for_user(&pool, user.id)
        .await
        .context("failed to fetch gacha records")?;
    tracing::info!(count = records.len(), "fetched gacha records");

    let out_path = args.output.unwrap_or_else(|| {
        let ts = chrono::Utc::now().timestamp();
        PathBuf::from(format!("gacha_{}_{}.json", user.uid, ts))
    });

    let export = Export {
        user: &user,
        exported_at: chrono::Utc::now().to_rfc3339(),
        record_count: records.len(),
        records: &records,
    };

    let json = serde_json::to_vec_pretty(&export).context("failed to serialize export")?;
    std::fs::write(&out_path, json)
        .with_context(|| format!("failed to write {}", out_path.display()))?;

    println!(
        "Exported {} record(s) for uid={} (id={}) to {}",
        records.len(),
        user.uid,
        user.id,
        out_path.display()
    );
    Ok(())
}
