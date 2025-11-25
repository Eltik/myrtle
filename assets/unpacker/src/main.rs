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
        } => {
            assets_unpacker::decode_textasset::main(&input, &output, delete)?;
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
    }

    Ok(())
}
