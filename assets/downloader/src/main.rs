use arknights_downloader::downloader::{ArkAssets, Servers};
use clap::Parser;
use std::process;

/// Arknights Asset Downloader - Rust implementation with multithreading support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server to download from:
    ///   official  - CN Official (Hypergryph)
    ///   bilibili  - CN Bilibili
    ///   en        - Global/EN (Yostar)
    ///   jp        - Japan (Yostar)
    ///   kr        - Korea (Yostar)
    ///   tw        - Taiwan (Gryphline)
    #[arg(short, long, default_value = "en")]
    server: String,

    /// Directory to save downloaded assets
    #[arg(
        short = 'd',
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
        "official" | "cn" => Servers::OFFICIAL,
        "bilibili" | "bili" | "b" => Servers::BILIBILI,
        "en" | "global" | "us" => Servers::EN,
        "jp" | "japan" => Servers::JP,
        "kr" | "korea" => Servers::KR,
        "tw" | "taiwan" => Servers::TW,
        _ => {
            eprintln!(
                "Error: Invalid server type '{}'. Valid options: official, bilibili, en, jp, kr, tw",
                args.server
            );
            process::exit(1);
        }
    };

    println!("Selected server: {}", server.display_name());

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
