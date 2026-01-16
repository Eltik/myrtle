use arknights_downloader::downloader::{ArkAssets, Servers};
use arknights_downloader::s3_bridge::{S3Client, S3Config, S3SyncWorkflow};
use arknights_downloader::s3_manifest::TrackedS3Sync;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

/// Arknights Asset Downloader - Rust implementation with multithreading support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Server to download from:
    ///   official  - CN Official (Hypergryph)
    ///   bilibili  - CN Bilibili
    ///   en        - Global/EN (Yostar)
    ///   jp        - Japan (Yostar)
    ///   kr        - Korea (Yostar)
    ///   tw        - Taiwan (Gryphline)
    #[arg(short, long, default_value = "en", global = true)]
    server: String,

    /// Directory to save downloaded assets
    #[arg(
        short = 'd',
        long,
        default_value = "/Users/eltik/Documents/Coding/Arknights-Downloader/ArkAssets",
        global = true
    )]
    savedir: String,

    /// Specific packages to download (comma-separated, e.g., "lpack/lcom,gamedata")
    /// If not specified, interactive mode will be used
    #[arg(short, long, global = true)]
    packages: Option<String>,

    /// Number of threads to use for parallel processing.
    /// Lower values reduce CPU usage and heat. Default uses half of available cores.
    /// Use 1 for minimal CPU usage (sequential processing).
    #[arg(short = 't', long, default_value = None, global = true)]
    threads: Option<usize>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Download assets from game server (default behavior)
    Download {
        /// Server to download from
        #[arg(short, long)]
        server: Option<String>,

        /// Directory to save downloaded assets
        #[arg(short = 'd', long)]
        savedir: Option<String>,

        /// Specific packages to download (comma-separated)
        #[arg(short, long)]
        packages: Option<String>,

        /// Number of threads
        #[arg(short = 't', long)]
        threads: Option<usize>,
    },

    /// Sync downloaded assets to S3-compatible storage
    ///
    /// Uses environment variables for S3 configuration:
    /// S3_ENDPOINT, S3_BUCKET, S3_REGION, S3_ACCESS_KEY, S3_SECRET_KEY, S3_PATH_STYLE
    S3Sync {
        /// Local directory to sync (default: savedir)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// S3 prefix for uploaded files
        #[arg(short = 'p', long, default_value = "")]
        prefix: String,

        /// Number of concurrent uploads
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,
    },

    /// Download and sync to S3 in one operation
    ///
    /// Downloads assets from game server, then uploads to S3
    S3Download {
        /// Server to download from
        #[arg(short, long)]
        server: Option<String>,

        /// Specific packages to download (comma-separated)
        #[arg(short, long)]
        packages: Option<String>,

        /// S3 prefix for uploaded files
        #[arg(short = 'o', long, default_value = "")]
        output_prefix: String,

        /// Number of concurrent S3 uploads
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,

        /// Number of download threads
        #[arg(short = 't', long)]
        threads: Option<usize>,
    },

    /// List files in S3 bucket
    S3List {
        /// Prefix filter for listing
        #[arg(short = 'p', long)]
        prefix: Option<String>,

        /// Show manifest statistics
        #[arg(long)]
        stats: bool,
    },

    /// Upload local directory to S3
    S3Upload {
        /// Local directory to upload
        #[arg(short, long)]
        input: PathBuf,

        /// S3 prefix for uploaded files
        #[arg(short = 'p', long, default_value = "")]
        prefix: String,

        /// Number of concurrent uploads
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,
    },
}

/// Parse server string to Servers enum
fn parse_server(server_str: &str) -> Result<Servers, String> {
    match server_str.to_lowercase().as_str() {
        "official" | "cn" => Ok(Servers::OFFICIAL),
        "bilibili" | "bili" | "b" => Ok(Servers::BILIBILI),
        "en" | "global" | "us" => Ok(Servers::EN),
        "jp" | "japan" => Ok(Servers::JP),
        "kr" | "korea" => Ok(Servers::KR),
        "tw" | "taiwan" => Ok(Servers::TW),
        _ => Err(format!(
            "Invalid server type '{}'. Valid options: official, bilibili, en, jp, kr, tw",
            server_str
        )),
    }
}

/// Configure and initialize thread pool
fn setup_thread_pool(threads: Option<usize>) -> usize {
    let num_threads = threads.unwrap_or_else(|| {
        let cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        (cores / 2).max(1)
    });

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap_or_else(|e| eprintln!("Warning: Could not set thread pool size: {}", e));

    num_threads
}

/// Run the download operation
fn run_download(
    server: Servers,
    savedir: &str,
    packages: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let assets = ArkAssets::new(server)?;

    if let Some(pkg_list) = packages {
        let packages: Vec<String> = pkg_list.split(',').map(|s| s.trim().to_string()).collect();
        assets.download_fromlist(&packages, savedir, 6)?;
    } else {
        assets.download(savedir)?;
    }

    Ok(())
}

fn main() {
    // Initialize logger
    env_logger::init();

    let args = Args::parse();

    // Handle commands
    match args.command {
        // No subcommand - default to download behavior
        None => {
            let num_threads = setup_thread_pool(args.threads);
            println!("Using {} threads for parallel processing", num_threads);

            let server = match parse_server(&args.server) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };

            println!("Selected server: {}", server.display_name());

            if let Err(e) = run_download(server, &args.savedir, args.packages) {
                eprintln!("Error during download: {}", e);
                process::exit(1);
            }

            println!("Download completed successfully!");
        }

        Some(Commands::Download {
            server,
            savedir,
            packages,
            threads,
        }) => {
            let num_threads = setup_thread_pool(threads.or(args.threads));
            println!("Using {} threads for parallel processing", num_threads);

            let server_str = server.as_ref().unwrap_or(&args.server);
            let server = match parse_server(server_str) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };

            let savedir = savedir.as_ref().unwrap_or(&args.savedir);
            println!("Selected server: {}", server.display_name());

            if let Err(e) = run_download(server, savedir, packages.or(args.packages)) {
                eprintln!("Error during download: {}", e);
                process::exit(1);
            }

            println!("Download completed successfully!");
        }

        Some(Commands::S3Sync {
            input,
            prefix,
            concurrency,
        }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = match S3Config::from_env() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                };

                let client = match S3Client::new(config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error creating S3 client: {}", e);
                        process::exit(1);
                    }
                };

                let input_dir = input.unwrap_or_else(|| PathBuf::from(&args.savedir));
                let server_str = &args.server;

                let mut sync = match TrackedS3Sync::new(client, concurrency, None).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error initializing S3 sync: {}", e);
                        process::exit(1);
                    }
                };

                println!("Syncing {} to S3...", input_dir.display());

                match sync
                    .sync_directory_tracked(&input_dir, &prefix, server_str)
                    .await
                {
                    Ok(stats) => {
                        println!("Sync complete: {}", stats);
                        sync.print_stats();
                    }
                    Err(e) => {
                        eprintln!("Error during S3 sync: {}", e);
                        process::exit(1);
                    }
                }
            });
        }

        Some(Commands::S3Download {
            server,
            packages,
            output_prefix,
            concurrency,
            threads,
        }) => {
            // First, download to temp directory
            let temp_dir = tempfile::tempdir().unwrap();
            let temp_path = temp_dir.path().to_string_lossy().to_string();

            let num_threads = setup_thread_pool(threads.or(args.threads));
            println!("Using {} threads for parallel processing", num_threads);

            let server_str = server.as_ref().unwrap_or(&args.server);
            let server_enum = match parse_server(server_str) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            };

            println!("Selected server: {}", server_enum.display_name());
            println!("Downloading to temporary directory...");

            if let Err(e) = run_download(server_enum, &temp_path, packages.or(args.packages)) {
                eprintln!("Error during download: {}", e);
                process::exit(1);
            }

            println!("Download complete. Uploading to S3...");

            // Then upload to S3
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = match S3Config::from_env() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                };

                let client = match S3Client::new(config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error creating S3 client: {}", e);
                        process::exit(1);
                    }
                };

                let mut sync = match TrackedS3Sync::new(client, concurrency, None).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error initializing S3 sync: {}", e);
                        process::exit(1);
                    }
                };

                // Update version info in manifest
                sync.manifest_mut()
                    .update_server_version(server_str, "unknown", "unknown");

                match sync
                    .sync_directory_tracked(temp_dir.path(), &output_prefix, server_str)
                    .await
                {
                    Ok(stats) => {
                        println!("S3 upload complete: {}", stats);
                        sync.print_stats();
                    }
                    Err(e) => {
                        eprintln!("Error during S3 upload: {}", e);
                        process::exit(1);
                    }
                }
            });

            println!("Download and S3 sync completed successfully!");
        }

        Some(Commands::S3List { prefix, stats }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = match S3Config::from_env() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                };

                let client = match S3Client::new(config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error creating S3 client: {}", e);
                        process::exit(1);
                    }
                };

                let workflow = S3SyncWorkflow::new(client, 1);

                match workflow.list_objects(prefix.as_deref()).await {
                    Ok(objects) => {
                        println!("Files in S3 bucket:");
                        for obj in &objects {
                            println!("  {}", obj);
                        }
                        println!("\nTotal: {} files", objects.len());
                    }
                    Err(e) => {
                        eprintln!("Error listing S3 objects: {}", e);
                        process::exit(1);
                    }
                }

                if stats {
                    let manifest = arknights_downloader::s3_manifest::S3DownloadManifest::load(
                        workflow.client(),
                        None,
                    )
                    .await
                    .unwrap_or_default();
                    println!("\n{}", manifest.stats());
                }
            });
        }

        Some(Commands::S3Upload {
            input,
            prefix,
            concurrency,
        }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = match S3Config::from_env() {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                };

                let client = match S3Client::new(config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error creating S3 client: {}", e);
                        process::exit(1);
                    }
                };

                let batch_ops = arknights_downloader::s3_bridge::S3BatchOperations::new(
                    Arc::new(client),
                    concurrency,
                );

                println!("Uploading {} to S3 prefix '{}'...", input.display(), prefix);

                match batch_ops.upload_directory(&input, &prefix).await {
                    Ok(stats) => {
                        println!("Upload complete: {}", stats);
                    }
                    Err(e) => {
                        eprintln!("Error during upload: {}", e);
                        process::exit(1);
                    }
                }
            });
        }
    }
}
