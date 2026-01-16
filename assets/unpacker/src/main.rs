use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Re-export S3 types for CLI use
use assets_unpacker::s3_bridge::{ExtractOptions, S3Config, S3Workflow};
use assets_unpacker::s3_manifest::TrackedS3Workflow;

#[derive(Parser)]
#[command(name = "assets-unpacker")]
#[command(version = "0.1.0")]
#[command(about = "Extract and process Arknights game assets")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract assets from AB files
    Extract {
        /// Input directory or file
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: PathBuf,

        /// Delete existing output directory
        #[arg(short, long, default_value = "false")]
        delete: bool,

        /// Extract images
        #[arg(long, default_value = "true")]
        image: bool,

        /// Extract text assets
        #[arg(long, default_value = "true")]
        text: bool,

        /// Extract audio
        #[arg(long, default_value = "true")]
        audio: bool,

        /// Extract Spine models (organizes .skel/.atlas into type subdirectories)
        #[arg(long, default_value = "true")]
        spine: bool,

        /// Merge alpha textures into RGBA instead of saving separately
        #[arg(long, default_value = "false")]
        merge_alpha: bool,

        /// Group by source file
        #[arg(long, default_value = "true")]
        group: bool,

        #[arg(long, default_value = "false")]
        force: bool,

        /// Number of parallel threads (1=sequential, 2-4=moderate, 8+=fast but high memory)
        #[arg(short = 'j', long, default_value = "1")]
        threads: usize,

        /// Skip files larger than this size in MB (default: 0 = no skip)
        /// Use to speed up initial extraction by deferring huge files like token.ab
        #[arg(long, default_value = "0")]
        skip_large_mb: u64,
    },

    /// Combine RGB and Alpha images
    Combine {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        /// Delete existing output directory
        #[arg(short, long, default_value = "false")]
        delete: bool,
    },

    /// Decode FlatBuffers/AES encrypted TextAssets
    Decode {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,

        /// Delete existing output directory
        #[arg(short, long, default_value = "false")]
        delete: bool,

        /// Path to resource manifest (.idx file) for proper output paths
        /// If not specified, will try to find it in ArkAssets folder
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Also reorganize plain text files (story, etc.) to proper paths
        #[arg(long)]
        reorganize: bool,
    },

    /// ArkModels workflow (extract Spine models)
    Models {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },

    /// ArkVoice workflow (extract and merge audio)
    Voice {
        srcdir: PathBuf,
        destdir: PathBuf,
        #[arg(long, default_value = "false")]
        merge: bool,
        #[arg(long, default_value = "false")]
        is_custom: bool,
    },

    /// Extract individual portraits from SpritePacker (charportraits)
    Portraits {
        /// Input directory containing charportraits pack*.ab files
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for extracted portraits
        #[arg(short, long)]
        output: PathBuf,

        /// Delete existing output directory
        #[arg(short, long, default_value = "false")]
        delete: bool,
    },

    /// Reorganize extracted files from hash folders to proper game paths
    Reorganize {
        /// Source directory (containing anon/ folder with hash-based subfolders)
        #[arg(short, long)]
        input: PathBuf,

        /// Destination directory (default: same as input for in-place)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Path to resource manifest (.idx file)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },

    /// Extract assets from S3-compatible storage
    ///
    /// Downloads asset bundles from S3, processes them locally, and uploads
    /// extracted assets back to S3. Uses environment variables for configuration:
    ///
    /// S3_ENDPOINT, S3_INPUT_BUCKET, S3_OUTPUT_BUCKET, S3_REGION,
    /// S3_ACCESS_KEY, S3_SECRET_KEY, S3_PATH_STYLE
    S3Extract {
        /// Prefix filter for input files (e.g., "assets/" to only process files in assets/)
        #[arg(short = 'p', long)]
        input_prefix: Option<String>,

        /// Output prefix for extracted assets (e.g., "extracted/")
        #[arg(short, long, default_value = "")]
        output_prefix: String,

        /// Extract images
        #[arg(long, default_value = "true")]
        image: bool,

        /// Extract text assets
        #[arg(long, default_value = "true")]
        text: bool,

        /// Extract audio
        #[arg(long, default_value = "true")]
        audio: bool,

        /// Extract Spine models
        #[arg(long, default_value = "true")]
        spine: bool,

        /// Merge alpha textures into RGBA
        #[arg(long, default_value = "false")]
        merge_alpha: bool,

        /// Group by source file
        #[arg(long, default_value = "true")]
        group: bool,

        /// Force re-extraction even if already processed
        #[arg(long, default_value = "false")]
        force: bool,

        /// Number of concurrent S3 uploads (default: 4)
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,
    },

    /// List asset bundles in S3 input bucket
    S3List {
        /// Prefix filter for listing
        #[arg(short = 'p', long)]
        prefix: Option<String>,

        /// Show manifest statistics
        #[arg(long)]
        stats: bool,
    },

    /// Download files from S3 to local directory
    S3Download {
        /// S3 key or prefix to download
        #[arg(short, long)]
        key: String,

        /// Local output directory
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Upload local directory to S3
    S3Upload {
        /// Local input directory
        #[arg(short, long)]
        input: PathBuf,

        /// S3 output prefix
        #[arg(short = 'p', long, default_value = "")]
        prefix: String,

        /// Number of concurrent uploads
        #[arg(short = 'c', long, default_value = "4")]
        concurrency: usize,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract {
            input,
            output,
            delete,
            image,
            text,
            audio,
            spine,
            merge_alpha,
            group,
            force,
            threads,
            skip_large_mb,
        } => {
            assets_unpacker::resolve_ab::main(
                &input,
                &output,
                delete,
                image,
                text,
                audio,
                spine,
                merge_alpha,
                group,
                force,
                threads,
                skip_large_mb,
            )?;
        }
        Commands::Combine {
            input,
            output,
            delete,
        } => {
            assets_unpacker::combine_rgb::main(&input, &output, delete)?;
        }
        Commands::Decode {
            input,
            output,
            delete,
            manifest,
            reorganize,
        } => {
            assets_unpacker::decode_textasset::main(&input, &output, delete, manifest.as_deref())?;

            // Also reorganize plain text files if enabled
            if reorganize {
                println!("\nReorganizing plain text files...");
                assets_unpacker::reorganize::main(&input, Some(&output), manifest.as_deref())?;
            }
        }
        Commands::Models { input, output } => {
            // TODO: Implement properly
            log::warn!("Models command not yet fully implemented");
            let src_dirs: Vec<&std::path::Path> = vec![input.as_path()];
            let dest_dirs: Vec<&std::path::Path> = vec![output.as_path()];
            assets_unpacker::collect_models::main(&src_dirs, &dest_dirs)?;
        }
        Commands::Voice {
            srcdir,
            destdir,
            merge,
            is_custom: _,
        } => {
            assets_unpacker::collect_voice::main(&srcdir, &destdir, merge)?;
        }
        Commands::Portraits {
            input,
            output,
            delete,
        } => {
            use std::cell::RefCell;
            use std::rc::Rc;
            use unity_rs::helpers::import_helper::FileSource;
            use unity_rs::Environment;
            use walkdir::WalkDir;

            if delete && output.exists() {
                std::fs::remove_dir_all(&output)?;
            }
            std::fs::create_dir_all(&output)?;

            let mut total_extracted = 0usize;

            // Find all pack*.ab files in the input directory
            let files: Vec<_> = if input.is_file() {
                vec![input.clone()]
            } else {
                WalkDir::new(&input)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let name = e.file_name().to_string_lossy();
                        name.starts_with("pack") && name.ends_with(".ab")
                    })
                    .map(|e| e.path().to_path_buf())
                    .collect()
            };

            println!("Found {} pack files to process", files.len());

            for file in files {
                println!("Processing: {}", file.display());

                let mut env = Environment::new();
                env.load_file(
                    FileSource::Path(file.to_string_lossy().to_string()),
                    None,
                    false,
                );

                let env_rc = Rc::new(RefCell::new(env));
                if Environment::set_environment_references(&env_rc).is_err() {
                    log::warn!(
                        "Failed to set environment references for {}",
                        file.display()
                    );
                    continue;
                }

                match assets_unpacker::resolve_ab::extract_sprite_packer(&env_rc, &output) {
                    Ok(count) => {
                        println!("  Extracted {} portraits", count);
                        total_extracted += count;
                    }
                    Err(e) => {
                        log::warn!("Failed to extract from {}: {}", file.display(), e);
                    }
                }
            }

            println!("\nTotal portraits extracted: {}", total_extracted);
        }
        Commands::Reorganize {
            input,
            output,
            manifest,
        } => {
            assets_unpacker::reorganize::main(&input, output.as_deref(), manifest.as_deref())?;
        }

        // S3 Commands
        Commands::S3Extract {
            input_prefix,
            output_prefix,
            image,
            text,
            audio,
            spine,
            merge_alpha,
            group,
            force,
            concurrency,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = S3Config::from_env()?;
                let client = assets_unpacker::s3_bridge::S3Client::new(config)?;

                let mut workflow = TrackedS3Workflow::new(client, concurrency, None).await?;

                // List all asset bundles
                let bundles = workflow
                    .workflow()
                    .list_input_bundles(input_prefix.as_deref())
                    .await?;

                println!("Found {} asset bundles to process", bundles.len());

                if bundles.is_empty() {
                    return Ok::<_, anyhow::Error>(());
                }

                let options = ExtractOptions {
                    extract_images: image,
                    extract_text: text,
                    extract_audio: audio,
                    extract_spine: spine,
                    merge_alpha,
                    group_by_source: group,
                    force,
                    concurrency,
                };

                // Process with tracking
                let stats = workflow
                    .process_with_tracking(
                        &bundles,
                        &output_prefix,
                        force,
                        |input_path, output_path| {
                            // Call the existing extraction logic
                            assets_unpacker::resolve_ab::main(
                                input_path,
                                output_path,
                                false, // delete
                                options.extract_images,
                                options.extract_text,
                                options.extract_audio,
                                options.extract_spine,
                                options.merge_alpha,
                                options.group_by_source,
                                options.force,
                                1, // threads (sequential for S3 workflow)
                                0, // skip_large_mb
                            )?;

                            // Count output files as assets
                            let count = walkdir::WalkDir::new(output_path)
                                .into_iter()
                                .filter_map(|e| e.ok())
                                .filter(|e| e.file_type().is_file())
                                .count();

                            Ok(count)
                        },
                    )
                    .await?;

                println!("\nExtraction complete: {}", stats);
                workflow.print_stats();

                Ok(())
            })?;
        }

        Commands::S3List { prefix, stats } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = S3Config::from_env()?;
                let client = assets_unpacker::s3_bridge::S3Client::new(config)?;

                // List asset bundles
                let workflow = S3Workflow::new(client, 1);
                let bundles = workflow.list_input_bundles(prefix.as_deref()).await?;

                println!("Asset bundles in input bucket:");
                for bundle in &bundles {
                    println!("  {}", bundle);
                }
                println!("\nTotal: {} files", bundles.len());

                // Show manifest stats if requested
                if stats {
                    let manifest = assets_unpacker::s3_manifest::S3Manifest::load(
                        workflow.client(),
                        None,
                    )
                    .await?;
                    println!("\n{}", manifest.stats());
                }

                Ok::<_, anyhow::Error>(())
            })?;
        }

        Commands::S3Download { key, output } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = S3Config::from_env()?;
                let client = assets_unpacker::s3_bridge::S3Client::new(config)?;

                std::fs::create_dir_all(&output)?;

                // Check if it's a single file or a prefix
                let keys = client.list_input_objects(Some(&key)).await?;

                if keys.is_empty() {
                    // Try as exact key
                    let local_path = output.join(
                        std::path::Path::new(&key)
                            .file_name()
                            .unwrap_or_else(|| std::ffi::OsStr::new("file")),
                    );
                    println!("Downloading: {} -> {}", key, local_path.display());
                    client.download_file(&key, &local_path).await?;
                } else {
                    // Download all matching files
                    println!("Downloading {} files...", keys.len());
                    for k in &keys {
                        let relative = k.strip_prefix(&key).unwrap_or(k);
                        let local_path = output.join(relative);
                        println!("  {} -> {}", k, local_path.display());
                        client.download_file(k, &local_path).await?;
                    }
                }

                println!("Download complete.");
                Ok::<_, anyhow::Error>(())
            })?;
        }

        Commands::S3Upload {
            input,
            prefix,
            concurrency,
        } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let config = S3Config::from_env()?;
                let client = assets_unpacker::s3_bridge::S3Client::new(config)?;

                let batch_ops = assets_unpacker::s3_bridge::S3BatchOperations::new(
                    std::sync::Arc::new(client),
                    concurrency,
                );

                println!("Uploading {} to S3 prefix '{}'...", input.display(), prefix);
                let stats = batch_ops.upload_directory(&input, &prefix).await?;

                println!("Upload complete: {}", stats);
                Ok::<_, anyhow::Error>(())
            })?;
        }
    }

    Ok(())
}
