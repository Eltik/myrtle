/// Example: Automated Production Monitor
///
/// This example shows how to use the version checker in a production environment
/// for automated monitoring and downloads.
///
/// Typical use cases:
/// - Scheduled cron jobs (e.g., check every hour)
/// - CI/CD pipelines
/// - Discord/Slack bots that notify on updates
/// - Auto-update services
///
/// Usage:
///   cargo run --example automated_monitor
///
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};
use std::process;

fn main() {
    // Parse environment variables for production deployment
    // Supported values: official, bilibili, en, jp, kr, tw
    let server = match std::env::var("ARK_SERVER")
        .as_deref()
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Ok("official") | Ok("cn") => Servers::OFFICIAL,
        Ok("bilibili") | Ok("bili") | Ok("b") => Servers::BILIBILI,
        Ok("jp") | Ok("japan") => Servers::JP,
        Ok("kr") | Ok("korea") => Servers::KR,
        Ok("tw") | Ok("taiwan") => Servers::TW,
        _ => Servers::EN, // Default to EN (Global)
    };

    let savedir = std::env::var("ARK_SAVEDIR").unwrap_or_else(|_| "./ArkAssets".to_string());

    let auto_download = std::env::var("ARK_AUTO_DOWNLOAD")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    println!("[Monitor] Arknights Update Monitor Starting...");
    println!("[Monitor] Server: {}", server.display_name());
    println!("[Monitor] Save Directory: {}", savedir);
    println!("[Monitor] Auto-download: {}", auto_download);
    println!();

    // Check for updates
    match ArkAssets::check_for_updates(server, &savedir) {
        Ok(status) => {
            match status {
                UpdateStatus::NoUpdate => {
                    println!("[Monitor] No updates detected.");
                    process::exit(0); // Exit code 0 = no updates
                }

                UpdateStatus::FirstCheck => {
                    println!("[Monitor] First check - version baseline established.");
                    process::exit(2); // Exit code 2 = first check
                }

                UpdateStatus::ResourceUpdate {
                    old_version,
                    new_version,
                } => {
                    println!("[Monitor] ⚠️  RESOURCE UPDATE DETECTED!");
                    println!("[Monitor]   Old: {}", old_version);
                    println!("[Monitor]   New: {}", new_version);

                    if auto_download {
                        println!("[Monitor] Auto-download enabled - starting download...");
                        trigger_download(server, &savedir);
                    } else {
                        println!(
                            "[Monitor] Auto-download disabled - manual intervention required."
                        );
                    }

                    process::exit(10); // Exit code 10 = resource update
                }

                UpdateStatus::ClientUpdate {
                    old_version,
                    new_version,
                } => {
                    println!("[Monitor] ⚠️  CLIENT UPDATE DETECTED!");
                    println!("[Monitor]   Old: {}", old_version);
                    println!("[Monitor]   New: {}", new_version);

                    process::exit(11); // Exit code 11 = client update
                }

                UpdateStatus::BothUpdated {
                    old_res_version,
                    new_res_version,
                    ..
                } => {
                    println!("[Monitor] ⚠️  MAJOR UPDATE DETECTED (Client + Resources)!");
                    println!(
                        "[Monitor]   Resource: {} -> {}",
                        old_res_version, new_res_version
                    );

                    if auto_download {
                        println!("[Monitor] Auto-download enabled - starting full download...");
                        trigger_download(server, &savedir);
                    } else {
                        println!(
                            "[Monitor] Auto-download disabled - manual intervention required."
                        );
                    }

                    process::exit(12); // Exit code 12 = both updated
                }
            }
        }

        Err(e) => {
            eprintln!("[Monitor] ❌ ERROR: {}", e);
            process::exit(1); // Exit code 1 = error
        }
    }
}

/// Trigger automatic download of all assets
fn trigger_download(server: Servers, savedir: &str) {
    match ArkAssets::new(server) {
        Ok(assets) => {
            println!("[Download] Initializing asset downloader...");

            // Download all packages automatically
            // In production, you might want to download specific packages only
            let all_keys: Vec<String> = assets.hot_update_list.keys().cloned().collect();

            match assets.download_fromlist(&all_keys, savedir, 6) {
                Ok(_) => {
                    println!("[Download] ✓ Download completed successfully!");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("[Download] ❌ Download failed: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("[Download] ❌ Failed to initialize downloader: {}", e);
            process::exit(1);
        }
    }
}
