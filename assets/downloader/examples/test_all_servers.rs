/// Example: Test All Server Endpoints
///
/// This example tests that version fetching works for all supported servers.
///
/// Usage:
///   cargo run --example test_all_servers
///
use arknights_downloader::downloader::{ArkAssets, Servers};

fn main() {
    println!("=== Testing All Server Endpoints ===\n");

    let servers = [
        (Servers::EN, "Global/EN"),
        (Servers::JP, "Japan"),
        (Servers::KR, "Korea"),
        (Servers::TW, "Taiwan"),
        (Servers::OFFICIAL, "CN Official"),
        (Servers::BILIBILI, "CN Bilibili"),
    ];

    let mut success = 0;
    let mut failed = 0;

    for (server, name) in servers {
        print!("{:<15} - ", name);
        match ArkAssets::get_version(server) {
            Ok((res, client)) => {
                println!("✓ res: {}, client: {}", res, client);
                success += 1;
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n=== Results ===");
    println!("Success: {}", success);
    println!("Failed: {}", failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
