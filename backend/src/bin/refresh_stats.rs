//! Force a refresh of any statistic that a background job would normally own.
//!
//! Every poller and cron job in `core` keeps its scheduling to itself and
//! exposes the work as `refresh_once`. This walks the registry in
//! `core::refresh` and runs whichever ones you name, so a box with
//! `DISABLE_BACKGROUND_JOBS=1` can still bring its aggregates current without
//! also starting the asset watcher, the DPS poller and the rest of the cron set.
//!
//! Adding a statistic here is a registry entry, not an edit to this file.
//!
//! Usage:
//!   cargo run --release --bin refresh-stats -- --list
//!   cargo run --release --bin refresh-stats -- --all
//!   cargo run --release --bin refresh-stats -- --all --include-heavy
//!   cargo run --release --bin refresh-stats -- operator-ownership trending
//!   cargo run --release --bin refresh-stats -- --all --concurrency 3

use anyhow::{Context, Result, bail};
use backend::app::cache::store::CacheStore;
use backend::app::state::{AppConfig, AppState, load_server_map};
use backend::core::hypergryph::config::GlobalConfig;
use backend::core::hypergryph::{config, loaders};
use backend::core::refresh::{self, Cost, RefreshTask};
use backend::core::service_account::ServiceAccounts;
use dotenv::dotenv;
use std::time::Instant;
use tokio::sync::Semaphore;

struct Args {
    names: Vec<String>,
    all: bool,
    include_heavy: bool,
    list: bool,
    concurrency: usize,
}

fn parse_args() -> Result<Args> {
    let mut args = Args {
        names: Vec::new(),
        all: false,
        include_heavy: false,
        list: false,
        // Sequential by default. Every task shares one Postgres pool, so
        // fanning them out trades a little wall clock for contention on the
        // same tables; opt in when you know the box can take it.
        concurrency: 1,
    };

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--all" | "-a" => args.all = true,
            "--include-heavy" => args.include_heavy = true,
            "--list" | "-l" => args.list = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--concurrency" | "-j" => {
                let raw = it.next().context("--concurrency needs a value")?;
                args.concurrency = raw.parse().context("--concurrency must be a number")?;
                if args.concurrency == 0 {
                    bail!("--concurrency must be at least 1");
                }
            }
            other if other.starts_with('-') => bail!("unknown flag {other} (try --help)"),
            other => args.names.push(other.to_owned()),
        }
    }

    Ok(args)
}

fn print_help() {
    println!("Force-refresh statistics normally driven by a poller or cron job.\n");
    println!("USAGE:");
    println!("  refresh-stats [--all [--include-heavy]] [TASK...] [-j N]\n");
    println!("FLAGS:");
    println!("  -a, --all             Run every cheap task");
    println!("      --include-heavy   With --all, also run the heavy ones");
    println!("  -l, --list            Show the tasks and exit");
    println!("  -j, --concurrency N   Run up to N at once (default 1)\n");
    print_tasks();
}

fn print_tasks() {
    let width = refresh::TASKS
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    println!("TASKS:");
    for task in refresh::TASKS {
        let tag = match task.cost {
            Cost::Cheap => "      ",
            Cost::Heavy => " HEAVY",
        };
        println!(
            "  {:<width$}{tag}  {}",
            task.name,
            task.about,
            width = width
        );
    }
    println!("\nHEAVY tasks scale with the user count or make network calls, so");
    println!("--all skips them unless --include-heavy is given.");
}

/// Build the same state the server builds. Tasks reach for game data, the cache
/// and the service accounts, so a cut-down context would refresh a different
/// thing than the job does.
async fn build_state() -> Result<AppState> {
    let config = AppConfig::from_env();
    let servers = load_server_map(&config, |_| ());
    let default_server = config.default_server;

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let db = backend::database::init(&database_url)
        .await
        .context("failed to initialize database")?;

    // Attach to the SAME cache the server uses, when one is configured.
    //
    // An in-memory cache here would be worse than useless: several tasks finish
    // by dropping cached responses under a prefix, and against a private cache
    // that call invalidates nothing. A running server would keep serving the
    // pre-refresh answer until its TTL lapsed, an hour for the ownership key,
    // which defeats the point of forcing a refresh at all. Invalidation is by
    // prefix and therefore targeted, so sharing the cache evicts exactly the
    // entries this run made stale and nothing else.
    //
    // Falling back to memory when Redis is absent or unreachable keeps the tool
    // usable on a box with no cache; the refresh still lands in Postgres, and
    // only the eviction is lost.
    let cache = match std::env::var("REDIS_URL") {
        Ok(url) => match redis::Client::open(url) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(conn) => CacheStore::new_redis(conn),
                Err(e) => {
                    eprintln!(
                        "warning: Redis unreachable ({e}); the running server will keep serving cached values until their TTL lapses"
                    );
                    CacheStore::new_memory()
                }
            },
            Err(e) => {
                eprintln!("warning: invalid REDIS_URL ({e}); falling back to an in-memory cache");
                CacheStore::new_memory()
            }
        },
        Err(_) => CacheStore::new_memory(),
    };

    let http_client = reqwest::Client::new();
    config::init_config(GlobalConfig::new());
    loaders::init(&http_client).await;
    let service_accounts = ServiceAccounts::load(&config.servers);

    Ok(AppState::new(
        db,
        cache,
        servers,
        default_server,
        config,
        http_client,
        service_accounts,
    ))
}

fn select(args: &Args) -> Result<Vec<&'static RefreshTask>> {
    if args.all {
        if !args.names.is_empty() {
            bail!("pass either --all or task names, not both");
        }
        return Ok(if args.include_heavy {
            refresh::TASKS.iter().collect()
        } else {
            refresh::default_set().collect()
        });
    }

    if args.names.is_empty() {
        bail!("nothing to do: name a task, or pass --all (try --list)");
    }

    args.names
        .iter()
        .map(|name| {
            refresh::find(name).ok_or_else(|| anyhow::anyhow!("unknown task {name} (try --list)"))
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,backend=info".into()),
        )
        .init();

    let args = parse_args()?;
    if args.list {
        print_tasks();
        return Ok(());
    }

    let selected = select(&args)?;
    let state = build_state().await?;

    println!(
        "running {} task(s) with concurrency {}\n",
        selected.len(),
        args.concurrency
    );

    let sem = std::sync::Arc::new(Semaphore::new(args.concurrency));
    let started = Instant::now();

    // One failure must not cancel the rest: these are independent aggregates
    // and a half-refreshed set is strictly better than an aborted one. Failures
    // are collected and reported together, and decide the exit code.
    let mut handles = Vec::with_capacity(selected.len());
    for task in selected {
        let permit = sem.clone().acquire_owned().await?;
        let state = state.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let began = Instant::now();
            let outcome = (task.run)(&state).await;
            (task.name, began.elapsed(), outcome)
        }));
    }

    let mut failed = 0usize;
    for handle in handles {
        let (name, elapsed, outcome) = handle.await.context("a refresh task panicked")?;
        match outcome {
            Ok(summary) => println!("  ok    {name} ({:.2}s): {summary}", elapsed.as_secs_f64()),
            Err(e) => {
                failed += 1;
                // `{e:#}` prints the anyhow context chain, which is where the
                // actual cause lives.
                println!("  FAIL  {name} ({:.2}s): {e:#}", elapsed.as_secs_f64());
            }
        }
    }

    println!("\nfinished in {:.2}s", started.elapsed().as_secs_f64());
    if failed > 0 {
        bail!("{failed} task(s) failed");
    }
    Ok(())
}
