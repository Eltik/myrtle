mod cli;

use std::sync::Arc;

use clap::Parser;
use cli::{Cli, Commands};
use downloader::{download, hot_update, manifest, pipeline, server::Server, types, version};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .init();

    let server: Server = cli.server.parse()?;
    let client = reqwest::Client::new();

    // Fetch version
    let ver = version::fetch_version(&client, server.version_url()).await?;
    println!(
        "Client: {}  Resources: {}",
        ver.client_version, ver.res_version
    );

    match cli.command {
        Commands::CheckUpdate => { /* just print version, already done */ }

        Commands::ListPacks => {
            let groups =
                hot_update::fetch_hot_update_list(&client, server.cdn_base_url(), &ver.res_version)
                    .await?;
            for g in &groups {
                println!(
                    "{:<30} {:>10}  ({} files)",
                    g.name,
                    g.total_size,
                    g.files.len()
                );
            }
        }

        Commands::Download { all, packages } => {
            let groups =
                hot_update::fetch_hot_update_list(&client, server.cdn_base_url(), &ver.res_version)
                    .await?;

            // Select which groups to download
            let selected: Vec<_> = if all {
                groups
            } else if let Some(pkgs) = packages {
                let names: Vec<&str> = pkgs.split(',').collect();
                groups
                    .into_iter()
                    .filter(|g| names.contains(&g.name.as_str()))
                    .collect()
            } else {
                anyhow::bail!("specify --all or --packages");
            };

            // Flatten to tasks, filter by manifest
            std::fs::create_dir_all(&cli.savedir)?;
            let mut manifest = manifest::Manifest::load(&cli.savedir)?;
            let all_files: Vec<_> = selected.iter().flat_map(|g| &g.files).cloned().collect();
            let tasks: Vec<_> = manifest
                .filter_needed(&all_files)
                .into_iter()
                .map(|f| types::DownloadTask {
                    filename: f.name,
                    md5: f.md5,
                    total_size: f.total_size,
                })
                .collect();

            println!(
                "{} files to download ({} skipped)",
                tasks.len(),
                all_files.len() - tasks.len()
            );

            let downloader = Arc::new(download::Downloader::new(
                server,
                ver.res_version,
                cli.threads,
            ));
            let stats = pipeline::run(downloader, tasks, cli.savedir, &mut manifest).await?;

            println!(
                "Done: {} downloaded, {} failed, {} bytes",
                stats.downloaded, stats.failed, stats.total_bytes
            );
        }
    }

    Ok(())
}
