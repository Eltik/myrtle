use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        } => {
            assets_unpacker::decode_textasset::main(&input, &output, delete, manifest.as_deref())?;
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
    }

    Ok(())
}
