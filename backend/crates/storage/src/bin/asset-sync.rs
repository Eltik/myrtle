use std::io::stderr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use dotenv::from_path;
use futures::StreamExt;
use myrtle_storage::config::parse_roots;
use myrtle_storage::key::server_prefix;
use myrtle_storage::restore::{RestoreOptions, run_restore};
use myrtle_storage::sync::{SyncError, SyncOptions, run_sync};
use myrtle_storage::{DeliveryMode, ObjectStore, StorageConfig, open_store};
use serde_json::to_string;
use tokio::runtime::Builder;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

const EXIT_CONFIG: u8 = 1;
const EXIT_FAILED: u8 = 2;

#[derive(Parser)]
#[command(
    name = "asset-sync",
    about = "Mirror Myrtle's served asset roots into S3-compatible storage"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
struct EnvArgs {
    #[arg(long = "env-file", global = true)]
    env_file: Vec<PathBuf>,
}

#[derive(Args)]
struct TreeArgs {
    /// The `assets/output` directory holding one tree per server.
    #[arg(long)]
    root: PathBuf,
    /// Server directory under the root: en, cn, jp, kr.
    #[arg(long)]
    server: String,
    /// Comma list of roots, overrides `STORAGE_ROOTS`
    #[arg(long)]
    roots: Option<String>,
    /// Concurrent uploads and deletes, overrides `STORAGE_SYNC_CONCURRENCY`
    #[arg(long)]
    concurrency: Option<usize>,
    /// Concurrent hashing tasks, overrides `STORAGE_SYNC_HASH_THREADS`
    #[arg(long)]
    hash_threads: Option<usize>,
    /// Print the summary as one JSON object.
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Upload new and changed files, then delete remote keys with no local file
    Sync {
        #[command(flatten)]
        tree: TreeArgs,
        /// Don't delete when uploading
        #[arg(long)]
        no_delete: bool,
        /// Dry run
        #[arg(long)]
        dry_run: bool,
        /// Delete even when the local tree is under 90% of the remote
        #[arg(long)]
        allow_shrink: bool,
        /// Truncate files the bucket holds to 0-byte stubs, overriding
        /// `STORAGE_EVICT_LOCAL`
        #[arg(long, conflicts_with = "no_evict")]
        evict: bool,
        /// Never truncate, overriding `STORAGE_EVICT_LOCAL`
        #[arg(long)]
        no_evict: bool,
        /// Evict even when asset delivery resolves to `local`
        #[arg(long)]
        force_evict: bool,
        #[command(flatten)]
        env: EnvArgs,
    },
    /// Download every evicted stub back from the store; run before turning
    /// storage off. Works with `STORAGE_EVICT_LOCAL` unset.
    Restore {
        /// `assets/output` directory holding one tree per server.
        #[arg(long)]
        root: PathBuf,
        /// Server directory (en, cn, jp, kr)
        #[arg(long)]
        server: String,
        /// Dry run
        #[arg(long)]
        dry_run: bool,
        /// Restore only stubs under this rel-path prefix
        #[arg(long)]
        only: Vec<String>,
        /// Concurrent downloads, overrides `STORAGE_SYNC_CONCURRENCY`
        #[arg(long)]
        concurrency: Option<usize>,
        /// Print summary as JSON object
        #[arg(long)]
        json: bool,
        #[command(flatten)]
        env: EnvArgs,
    },
    /// Dry-run diff; exits 2 when the remote differs from the local tree
    Verify {
        #[command(flatten)]
        tree: TreeArgs,
        #[command(flatten)]
        env: EnvArgs,
    },
    /// List remote keys as `size etag key`
    Ls {
        #[arg(long)]
        server: Option<String>,
        /// Key prefix under the server (or under `CLOUDFLARE_PREFIX`)
        #[arg(long)]
        prefix: Option<String>,
        #[command(flatten)]
        env: EnvArgs,
    },
    /// Set the bucket's CORS rule: GET and HEAD from these origins, or from
    /// any origin (`*`) when none is given
    Cors {
        /// Allowed origin; repeatable. Default `*`.
        #[arg(long = "allow-origin")]
        allow_origin: Vec<String>,
        #[command(flatten)]
        env: EnvArgs,
    },
}

impl Command {
    fn env_files(&self) -> &[PathBuf] {
        match self {
            Self::Sync { env, .. }
            | Self::Restore { env, .. }
            | Self::Verify { env, .. }
            | Self::Ls { env, .. }
            | Self::Cors { env, .. } => &env.env_file,
        }
    }
}

fn load_env(explicit: &[PathBuf]) -> Result<(), String> {
    let files: Vec<PathBuf> = if explicit.is_empty() {
        [PathBuf::from(".env"), PathBuf::from("../.env")]
            .into_iter()
            .filter(|p| p.is_file())
            .collect()
    } else {
        if let Some(missing) = explicit.iter().find(|p| !p.is_file()) {
            return Err(format!("env file {} does not exist", missing.display()));
        }
        explicit.to_vec()
    };
    if files.is_empty() {
        info!("no env file found (.env, ../.env); using the environment only");
    }
    for path in &files {
        from_path(path).map_err(|e| format!("env file {}: {e}", path.display()))?;
        info!(path = %path.display(), "env file loaded");
    }
    Ok(())
}

fn options(config: &StorageConfig, tree: &TreeArgs) -> Result<SyncOptions, String> {
    let mut opts = SyncOptions::from_config(config, tree.root.clone(), tree.server.clone());
    if let Some(raw) = &tree.roots {
        let roots = parse_roots(raw);
        if roots.is_empty() {
            return Err(format!("--roots {raw:?} names no directory"));
        }
        opts.roots = roots;
    }
    match tree.concurrency {
        Some(0) => return Err("--concurrency must be at least 1".into()),
        Some(n) => opts.concurrency = n,
        None => {}
    }
    match tree.hash_threads {
        Some(0) => return Err("--hash-threads must be at least 1".into()),
        Some(n) => opts.hash_threads = n,
        None => {}
    }
    check_server(&tree.server)?;
    Ok(opts)
}

fn check_server(server: &str) -> Result<(), String> {
    if server.is_empty() || server.contains('/') || server.starts_with('.') {
        return Err(format!(
            "--server {server:?} is not a server directory name"
        ));
    }
    Ok(())
}

/// Refuses eviction when the backend would serve the stubs
fn delivery_guard(config: &StorageConfig) -> Result<(), String> {
    match DeliveryMode::from_env(Some(config)) {
        Ok(DeliveryMode::Local) => Err(
            "eviction refused: ASSET_DELIVERY=local, so the backend would serve the 0-byte stubs. \
             Unset ASSET_DELIVERY (or set redirect or proxy) in the env files the backend reads, \
             or pass --force-evict"
                .into(),
        ),
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "eviction refused: asset delivery does not resolve ({e}); fix it or pass --force-evict"
        )),
    }
}

#[allow(clippy::too_many_lines)]
async fn run(cli: Cli) -> u8 {
    if let Err(e) = load_env(cli.command.env_files()) {
        eprintln!("asset-sync: {e}");
        return EXIT_CONFIG;
    }
    let config = match StorageConfig::from_env() {
        Ok(Some(c)) => c,
        Ok(None) => {
            println!("asset-sync: storage disabled (STORAGE_TYPE unset), nothing to do");
            return 0;
        }
        Err(e) => {
            eprintln!("asset-sync: {e}");
            return EXIT_CONFIG;
        }
    };
    let store = match open_store(&config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("asset-sync: {e}");
            return EXIT_CONFIG;
        }
    };

    match cli.command {
        Command::Sync {
            tree,
            no_delete,
            dry_run,
            allow_shrink,
            evict,
            no_evict,
            force_evict,
            ..
        } => {
            let mut opts = match options(&config, &tree) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("asset-sync: {e}");
                    return EXIT_CONFIG;
                }
            };
            opts.delete = !no_delete;
            opts.dry_run = dry_run;
            opts.allow_shrink = allow_shrink;
            if evict {
                opts.evict.enabled = true;
            } else if no_evict {
                opts.evict.enabled = false;
            }
            if opts.evict.enabled
                && !force_evict
                && let Err(e) = delivery_guard(&config)
            {
                eprintln!("asset-sync: {e}");
                return EXIT_CONFIG;
            }
            sync_and_report(store.as_ref(), &opts, tree.json, false).await
        }
        Command::Restore {
            root,
            server,
            dry_run,
            only,
            concurrency,
            json,
            ..
        } => {
            if let Err(e) = check_server(&server) {
                eprintln!("asset-sync: {e}");
                return EXIT_CONFIG;
            }
            let concurrency = match concurrency {
                Some(0) => {
                    eprintln!("asset-sync: --concurrency must be at least 1");
                    return EXIT_CONFIG;
                }
                Some(n) => n,
                None => config.sync_concurrency,
            };
            let mut opts = RestoreOptions::new(
                root,
                server,
                config.bucket.clone(),
                config.prefix.clone(),
                concurrency,
            );
            opts.dry_run = dry_run;
            opts.only = only;
            restore_and_report(store.as_ref(), &opts, json).await
        }
        Command::Verify { tree, .. } => {
            let mut opts = match options(&config, &tree) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("asset-sync: {e}");
                    return EXIT_CONFIG;
                }
            };
            opts.dry_run = true;
            // Verify only compares; what eviction would do is not drift.
            opts.evict.enabled = false;
            sync_and_report(store.as_ref(), &opts, tree.json, true).await
        }
        Command::Ls { server, prefix, .. } => {
            let mut list_prefix = match &server {
                Some(s) => server_prefix(&config.prefix, s),
                None if config.prefix.is_empty() => String::new(),
                None => format!("{}/", config.prefix),
            };
            if let Some(p) = prefix {
                list_prefix.push_str(p.trim_start_matches('/'));
            }
            let mut listing = store.list(&list_prefix);
            let mut count = 0u64;
            while let Some(item) = listing.next().await {
                match item {
                    Ok(meta) => {
                        count += 1;
                        println!(
                            "{}\t{}\t{}",
                            meta.size,
                            meta.etag.as_deref().unwrap_or("-"),
                            meta.key
                        );
                    }
                    Err(e) => {
                        eprintln!("asset-sync: listing failed: {e}");
                        return EXIT_FAILED;
                    }
                }
            }
            eprintln!("asset-sync: {count} objects under {list_prefix:?}");
            0
        }
        Command::Cors { allow_origin, .. } => {
            let allow_origin = if allow_origin.is_empty() {
                vec!["*".to_owned()]
            } else {
                allow_origin
            };
            match store.put_bucket_cors(&allow_origin).await {
                Ok(()) => {
                    println!(
                        "asset-sync: CORS set on bucket {} for {}",
                        config.bucket,
                        allow_origin.join(", ")
                    );
                    0
                }
                Err(e) => {
                    eprintln!("asset-sync: setting CORS failed: {e}");
                    EXIT_FAILED
                }
            }
        }
    }
}

async fn sync_and_report(
    store: &dyn ObjectStore,
    opts: &SyncOptions,
    json: bool,
    verify: bool,
) -> u8 {
    let report = match run_sync(store, opts).await {
        Ok(r) => r,
        Err(e @ SyncError::MissingServerDir(_)) => {
            eprintln!("asset-sync: {e}");
            return EXIT_CONFIG;
        }
        Err(e) => {
            eprintln!("asset-sync: {e}");
            return EXIT_FAILED;
        }
    };
    if json {
        match to_string(&report) {
            Ok(line) => println!("{line}"),
            Err(e) => eprintln!("asset-sync: report not serializable: {e}"),
        }
    } else {
        println!("{}", report.summary_line());
    }
    for failure in &report.failures {
        eprintln!("asset-sync: failed {failure}");
    }
    if report.evict_failed > 0 {
        eprintln!(
            "asset-sync: {} files could not be evicted and keep their bytes (see the warnings above)",
            report.evict_failed
        );
    }
    if verify && report.drifted() {
        return EXIT_FAILED;
    }
    u8::try_from(report.exit_code()).unwrap_or(EXIT_FAILED)
}

async fn restore_and_report(store: &dyn ObjectStore, opts: &RestoreOptions, json: bool) -> u8 {
    // Restore exits 1 on any failure, so a caller has one code to test.
    let report = match run_restore(store, opts).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("asset-sync: {e}");
            return EXIT_CONFIG;
        }
    };
    if json {
        match to_string(&report) {
            Ok(line) => println!("{line}"),
            Err(e) => eprintln!("asset-sync: report not serializable: {e}"),
        }
    } else {
        println!("{}", report.summary_line());
    }
    for failure in &report.failures {
        eprintln!("asset-sync: failed {failure}");
    }
    u8::try_from(report.exit_code()).unwrap_or(EXIT_CONFIG)
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // `--help` and `--version` print to stdout and are not errors.
            let code = if e.use_stderr() { EXIT_CONFIG } else { 0 };
            let _ = e.print();
            return ExitCode::from(code);
        }
    };
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_writer(stderr)
        .init();
    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("asset-sync: cannot start the runtime: {e}");
            return ExitCode::from(EXIT_FAILED);
        }
    };
    ExitCode::from(runtime.block_on(run(cli)))
}
