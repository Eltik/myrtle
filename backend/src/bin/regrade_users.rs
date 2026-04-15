//! Regrade every user in the database.
//!
//! Loads game data once, then walks the `users` table with keyset pagination
//! and recomputes each user's grade concurrently (bounded by `--concurrency`),
//! upserting the result into `user_scores`.
//!
//! Usage:
//!   cargo run --release --bin regrade-users
//!   cargo run --release --bin regrade-users -- --concurrency 16
//!   cargo run --release --bin regrade-users -- --uid 123456789          # single user
//!   cargo run --release --bin regrade-users -- --server en              # filter by server
//!   cargo run --release --bin regrade-users -- --dry-run                # compute, don't write
//!   cargo run --release --bin regrade-users -- --fail-fast               # stop on first error

use anyhow::{Context, Result};
use backend::{
    core::{
        gamedata::init_game_data, grade::calculate::calculate_user_grade,
        hypergryph::constants::Server,
    },
    database::{models::score::UserScore, queries::score::update_score},
};
use chrono::Utc;
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;
use uuid::Uuid;

struct Args {
    concurrency: usize,
    page_size: i64,
    uid: Option<String>,
    server: Option<i16>,
    dry_run: bool,
    fail_fast: bool,
    quiet: bool,
    progress_every: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            concurrency: 8,
            page_size: 500,
            uid: None,
            server: None,
            dry_run: false,
            fail_fast: false,
            quiet: false,
            progress_every: 50,
        }
    }
}

fn parse_args() -> Result<Args> {
    let mut a = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--concurrency" | "-j" => {
                a.concurrency = it
                    .next()
                    .context("--concurrency requires a value")?
                    .parse()?;
            }
            "--page-size" => {
                a.page_size = it.next().context("--page-size requires a value")?.parse()?;
            }
            "--uid" => {
                a.uid = Some(it.next().context("--uid requires a value")?);
            }
            "--server" => {
                let code = it.next().context("--server requires a code")?;
                let srv = Server::all()
                    .iter()
                    .find(|s| s.as_str().eq_ignore_ascii_case(&code))
                    .copied()
                    .with_context(|| format!("unknown server code: {code}"))?;
                a.server = Some(srv.index() as i16);
            }
            "--dry-run" => a.dry_run = true,
            "--fail-fast" => a.fail_fast = true,
            "--quiet" | "-q" => a.quiet = true,
            "--progress-every" => {
                a.progress_every = it
                    .next()
                    .context("--progress-every requires a value")?
                    .parse()?;
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }
    if a.concurrency == 0 {
        anyhow::bail!("--concurrency must be >= 1");
    }
    Ok(a)
}

fn print_usage() {
    eprintln!(
        "Usage: regrade-users [options]\n\
         \n\
         Recomputes user_scores for every user using the current grading logic.\n\
         \n\
         Options:\n\
           -j, --concurrency <N>     Max users graded in parallel (default 8)\n\
               --page-size <N>       Users fetched per keyset page (default 500)\n\
               --uid <UID>           Regrade a single user by Arknights UID\n\
               --server <code>       Restrict to one server (en|jp|kr|cn|bili|tw)\n\
               --dry-run             Compute grades but do not write to user_scores\n\
               --fail-fast           Abort on the first per-user failure\n\
           -q, --quiet               Suppress per-user log lines\n\
               --progress-every <N>  Log running totals every N users (default 50)\n\
           -h, --help                Show this help"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "regrade_users=info,backend=warn".into()),
        )
        .init();

    let args = parse_args()?;

    // Game data (expensive — load once, share).
    let data_dir =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());

    tracing::info!("loading game data...");
    let t0 = Instant::now();
    let game_data = init_game_data(Path::new(&data_dir), Path::new(&assets_dir))
        .context("failed to load game data")?;
    tracing::info!(
        operators = game_data.operators.len(),
        elapsed_s = format!("{:.2}", t0.elapsed().as_secs_f64()),
        "game data loaded",
    );
    let game_data = Arc::new(game_data);

    // Dedicated pool sized to comfortably hold concurrency * (fanout of grade queries).
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let max_conns = std::cmp::max(20, (args.concurrency as u32) * 3);
    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(600))
        .connect(&database_url)
        .await
        .context("failed to connect to database")?;
    tracing::info!(max_conns, "connected to database");

    // Total count (for ETA) — best-effort; skipped on error.
    let total = count_targets(&pool, &args).await.unwrap_or(0);
    tracing::info!(total, "users to regrade");

    let sem = Arc::new(Semaphore::new(args.concurrency));
    let successes = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));
    let processed = Arc::new(AtomicU64::new(0));
    let inflight = Arc::new(AtomicUsize::new(0));
    let abort = Arc::new(AtomicBool::new(false));
    let run_start = Instant::now();

    // Ctrl-C: stop dispatching new work. In-flight tasks finish naturally.
    {
        let abort = abort.clone();
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                tracing::warn!("received SIGINT, draining in-flight tasks (Ctrl-C again to force)");
                abort.store(true, Ordering::SeqCst);
                if tokio::signal::ctrl_c().await.is_ok() {
                    std::process::exit(130);
                }
            }
        });
    }

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    let mut cursor: Option<Uuid> = None;
    let mut dispatched: u64 = 0;
    let mut stopped_early = false;

    'outer: loop {
        if abort.load(Ordering::Relaxed) {
            stopped_early = true;
            break;
        }
        let page = fetch_page(&pool, &args, cursor, args.page_size).await?;
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|(id, _)| *id);

        for (user_id, uid) in page {
            if abort.load(Ordering::Relaxed) {
                stopped_early = true;
                break 'outer;
            }

            let permit = sem.clone().acquire_owned().await.expect("semaphore closed");
            inflight.fetch_add(1, Ordering::Relaxed);

            let pool = pool.clone();
            let game_data = game_data.clone();
            let successes = successes.clone();
            let failures = failures.clone();
            let processed = processed.clone();
            let inflight = inflight.clone();
            let abort = abort.clone();
            let dry_run = args.dry_run;
            let fail_fast = args.fail_fast;
            let quiet = args.quiet;
            let progress_every = args.progress_every;

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let started = Instant::now();
                let result = regrade_one(&pool, user_id, &game_data, dry_run).await;
                let elapsed_ms = started.elapsed().as_millis();

                let n = processed.fetch_add(1, Ordering::Relaxed) + 1;
                match result {
                    Ok(grade) => {
                        successes.fetch_add(1, Ordering::Relaxed);
                        if !quiet {
                            tracing::info!(
                                uid = %uid,
                                user_id = %user_id,
                                grade = %grade,
                                ms = elapsed_ms as u64,
                                "regraded"
                            );
                        }
                    }
                    Err(e) => {
                        failures.fetch_add(1, Ordering::Relaxed);
                        tracing::error!(uid = %uid, user_id = %user_id, error = %e, "regrade failed");
                        if fail_fast {
                            abort.store(true, Ordering::SeqCst);
                        }
                    }
                }

                if progress_every > 0 && n.is_multiple_of(progress_every) {
                    let s = successes.load(Ordering::Relaxed);
                    let f = failures.load(Ordering::Relaxed);
                    tracing::info!(
                        processed = n,
                        successes = s,
                        failures = f,
                        in_flight = inflight.load(Ordering::Relaxed),
                        "progress"
                    );
                }
                inflight.fetch_sub(1, Ordering::Relaxed);
            });
            handles.push(handle);
            dispatched += 1;
        }
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = run_start.elapsed();
    let s = successes.load(Ordering::Relaxed);
    let f = failures.load(Ordering::Relaxed);
    tracing::info!(
        dispatched,
        successes = s,
        failures = f,
        stopped_early,
        dry_run = args.dry_run,
        elapsed_s = format!("{:.2}", elapsed.as_secs_f64()),
        rate_per_s = format!("{:.2}", s as f64 / elapsed.as_secs_f64().max(0.001)),
        "regrade complete"
    );

    if f > 0 && args.fail_fast {
        std::process::exit(1);
    }
    Ok(())
}

async fn regrade_one(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &backend::core::gamedata::types::GameData,
    dry_run: bool,
) -> Result<String> {
    let grade = calculate_user_grade(pool, user_id, game_data)
        .await
        .with_context(|| format!("calculate_user_grade({user_id})"))?;

    let overall = grade.overall.clone();

    if !dry_run {
        let score = UserScore {
            user_id,
            total_score: grade.total_score,
            operator_score: grade.operator_grade,
            stage_score: grade.stage_grade,
            roguelike_score: grade.roguelike_grade,
            sandbox_score: grade.sandbox_grade,
            medal_score: grade.medal_grade,
            base_score: grade.base_grade,
            skin_score: 0.0,
            grade: Some(grade.overall),
            calculated_at: Utc::now(),
        };
        update_score(pool, &score)
            .await
            .with_context(|| format!("update_score({user_id})"))?;
    }

    Ok(overall)
}

async fn count_targets(pool: &PgPool, args: &Args) -> Result<i64> {
    if let Some(uid) = &args.uid {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE uid = $1")
            .bind(uid)
            .fetch_one(pool)
            .await?;
        return Ok(n);
    }
    if let Some(sid) = args.server {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE server_id = $1")
            .bind(sid)
            .fetch_one(pool)
            .await?;
        return Ok(n);
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(n)
}

/// Keyset page of `(id, uid)` strictly greater than `after`, bounded to `limit`.
async fn fetch_page(
    pool: &PgPool,
    args: &Args,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<(Uuid, String)>> {
    // Single-user mode: short-circuit after first call.
    if let Some(uid) = &args.uid {
        if after.is_some() {
            return Ok(Vec::new());
        }
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, uid FROM users WHERE uid = $1 ORDER BY id")
                .bind(uid)
                .fetch_all(pool)
                .await?;
        return Ok(rows);
    }

    let rows: Vec<(Uuid, String)> = match (after, args.server) {
        (None, None) => {
            sqlx::query_as("SELECT id, uid FROM users ORDER BY id LIMIT $1")
                .bind(limit)
                .fetch_all(pool)
                .await?
        }
        (Some(cursor), None) => {
            sqlx::query_as("SELECT id, uid FROM users WHERE id > $1 ORDER BY id LIMIT $2")
                .bind(cursor)
                .bind(limit)
                .fetch_all(pool)
                .await?
        }
        (None, Some(sid)) => {
            sqlx::query_as("SELECT id, uid FROM users WHERE server_id = $1 ORDER BY id LIMIT $2")
                .bind(sid)
                .bind(limit)
                .fetch_all(pool)
                .await?
        }
        (Some(cursor), Some(sid)) => {
            sqlx::query_as(
                "SELECT id, uid FROM users WHERE server_id = $1 AND id > $2 ORDER BY id LIMIT $3",
            )
            .bind(sid)
            .bind(cursor)
            .bind(limit)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(rows)
}
