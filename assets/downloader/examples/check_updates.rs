/// Example: Version Update Checker
///
/// This example demonstrates how to use the version checking functionality
/// to monitor for Arknights game and asset updates.
///
/// Usage:
///   cargo run --example check_updates
///
use arknights_downloader::downloader::{ArkAssets, Servers, UpdateStatus};

fn main() {
    println!("=== Arknights Version Update Checker ===\n");

    // Configuration
    let server = Servers::OFFICIAL; // or Servers::BILIBILI
    let savedir = "./ArkAssets";

    println!("Checking for updates...\n");

    match ArkAssets::check_for_updates(server, savedir) {
        Ok(status) => {
            match status {
                UpdateStatus::NoUpdate => {
                    println!("✓ No updates available - you're on the latest version!");

                    // You can still load cached info to display current version
                    if let Some(cached) = ArkAssets::load_version_cache(server, savedir) {
                        println!("\nCurrent Versions:");
                        println!("  Resource Version: {}", cached.resVersion);
                        println!("  Client Version: {}", cached.clientVersion);
                        println!("  Last Checked: {}", cached.lastChecked);
                    }
                }

                UpdateStatus::FirstCheck => {
                    println!("✓ First version check - baseline established!");
                    println!("  Version cache created at: {}/version_cache_official.json", savedir);
                    println!("  Run this again later to detect updates.");

                    if let Some(cached) = ArkAssets::load_version_cache(server, savedir) {
                        println!("\nCurrent Versions:");
                        println!("  Resource Version: {}", cached.resVersion);
                        println!("  Client Version: {}", cached.clientVersion);
                    }
                }

                UpdateStatus::ResourceUpdate { old_version, new_version } => {
                    println!("🔄 Resource Update Detected!");
                    println!("  Old Resource Version: {}", old_version);
                    println!("  New Resource Version: {}", new_version);
                    println!("\n⚠️  Asset files have been updated - you should re-download assets.");

                    // Automatically download
                    let assets = ArkAssets::new(server).unwrap();
                    assets.download(savedir).unwrap();
                }

                UpdateStatus::ClientUpdate { old_version, new_version } => {
                    println!("🔄 Client Update Detected!");
                    println!("  Old Client Version: {}", old_version);
                    println!("  New Client Version: {}", new_version);
                    println!("\n⚠️  Game client has been updated.");
                }

                UpdateStatus::BothUpdated {
                    old_res_version,
                    new_res_version,
                    old_client_version,
                    new_client_version,
                } => {
                    println!("🔄 Major Update Detected - Both Resource and Client Updated!");
                    println!("\nResource Update:");
                    println!("  Old Version: {}", old_res_version);
                    println!("  New Version: {}", new_res_version);
                    println!("\nClient Update:");
                    println!("  Old Version: {}", old_client_version);
                    println!("  New Version: {}", new_client_version);
                    println!("\n⚠️  Major game update - recommend full re-download.");

                    // Automatically download
                    let assets = ArkAssets::new(server).unwrap();
                    assets.download(savedir).unwrap();
                }
            }
        }

        Err(e) => {
            eprintln!("❌ Error checking for updates: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Version check complete ===");
}
