use arknights_downloader::downloader::{ArkAssets, Servers};
use clap::Parser;
use std::process;

/// Arknights Asset Downloader - Rust implementation with multithreading support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server to download from (official or bilibili)
    #[arg(short, long, default_value = "official")]
    server: String,

    /// Directory to save downloaded assets
    #[arg(
        short,
        long,
        default_value = "/Users/eltik/Documents/Coding/Arknights-Downloader/ArkAssets"
    )]
    savedir: String,

    /// Specific packages to download (comma-separated, e.g., "lpack/lcom,gamedata")
    /// If not specified, interactive mode will be used
    #[arg(short, long)]
    packages: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Parse server type
    let server = match args.server.to_lowercase().as_str() {
        "official" => Servers::OFFICIAL,
        "bilibili" => Servers::BILIBILI,
        _ => {
            eprintln!(
                "Error: Invalid server type '{}'. Must be 'official' or 'bilibili'",
                args.server
            );
            process::exit(1);
        }
    };

    // Create ArkAssets instance
    let assets = match ArkAssets::new(server) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error initializing ArkAssets: {}", e);
            process::exit(1);
        }
    };

    // Run download
    let result = if let Some(pkg_list) = args.packages {
        // Direct package download mode (like Python line 437 commented example)
        let packages: Vec<String> = pkg_list.split(',').map(|s| s.trim().to_string()).collect();
        assets.download_fromlist(&packages, &args.savedir, 6)
    } else {
        // Interactive mode (like Python line 436)
        assets.download(&args.savedir)
    };

    // Handle result
    if let Err(e) = result {
        eprintln!("Error during download: {}", e);
        process::exit(1);
    }

    println!("Download completed successfully!");
}
