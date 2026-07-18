mod cli;

use std::collections::HashSet;
use std::sync::Arc;

use clap::Parser;
use cli::{Cli, Commands};
use downloader::{
    download, hot_update, manifest, pipeline, resource_manifest, server::Server, types, version,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .init();

    let server: Server = cli.server.parse()?;
    let client = reqwest::Client::new();

    let ver = version::fetch_version(&client, server.version_url()).await?;
    println!(
        "Client: {}  Resources: {}",
        ver.client_version, ver.res_version
    );

    match &cli.command {
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

        Commands::Download {
            all,
            packages,
            profile,
        } => {
            run_download(
                &client,
                server,
                ver,
                &cli,
                *all,
                packages.as_deref(),
                profile.as_deref(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn run_download(
    client: &reqwest::Client,
    server: Server,
    ver: types::VersionResponse,
    cli: &Cli,
    all: bool,
    packages: Option<&str>,
    profile: Option<&str>,
) -> anyhow::Result<()> {
    let groups =
        hot_update::fetch_hot_update_list(client, server.cdn_base_url(), &ver.res_version).await?;

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

    std::fs::create_dir_all(&cli.savedir)?;
    let mut manifest = manifest::Manifest::load(&cli.savedir)?;
    let all_files: Vec<_> = selected.iter().flat_map(|g| &g.files).cloned().collect();

    // `--profile` is a comma-separated set of named profiles, OR-combined
    // (e.g. `operators,stages`). `full`/omitted keeps everything.
    let all_files: Vec<_> = match profile {
        Some("full") | None => all_files,
        Some(spec) => {
            let mut keep_fns: Vec<fn(&str) -> bool> = Vec::new();
            for p in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                keep_fns.push(match p {
                    "operators" => downloader::profile::keep_for_operators,
                    "stages" => downloader::profile::keep_for_stages,
                    _ => anyhow::bail!("unknown profile: {p} (expected: operators, stages, full)"),
                });
            }
            let kept: Vec<_> = all_files
                .iter()
                .filter(|f| keep_fns.iter().any(|keep| keep(&f.name)))
                .cloned()
                .collect();
            expand_with_dependencies(&cli.savedir, all_files, kept)
        }
    };
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
    let stats = pipeline::run(downloader, tasks, cli.savedir.clone(), &mut manifest).await?;

    println!(
        "Done: {} downloaded, {} failed, {} bytes",
        stats.downloaded, stats.failed, stats.total_bytes
    );

    Ok(())
}

/// Expand a prefix-selected set (`kept`) to include every downloadable bundle
/// reachable via transitive `allDependencies` in the resource manifest.
///
/// `all_files` is the full downloadable set (before profile filtering); it is
/// the source we map dependency bundle *names* back to concrete
/// [`types::HotFile`]s (with md5/size). Dependency names absent from
/// `all_files` (not separately downloadable) are skipped. When no resource
/// manifest `.idx` is present under `savedir`, `kept` is returned unchanged so
/// behavior matches the prior prefix-only selection.
fn expand_with_dependencies(
    savedir: &std::path::Path,
    all_files: Vec<types::HotFile>,
    kept: Vec<types::HotFile>,
) -> Vec<types::HotFile> {
    let manifest = match resource_manifest::ResourceManifest::find_and_load(savedir) {
        Ok(Some(m)) => m,
        Ok(None) => return kept,
        Err(e) => {
            eprintln!(
                "warning: could not parse resource manifest, using prefix-only selection: {e}"
            );
            return kept;
        }
    };

    let closure = manifest.dependency_closure(kept.iter().map(|f| f.name.as_str()));
    let kept_names: HashSet<String> = kept.iter().map(|f| f.name.clone()).collect();

    let mut result = kept;
    let before = result.len();
    result.extend(
        all_files
            .into_iter()
            .filter(|f| !kept_names.contains(&f.name) && closure.contains(&f.name)),
    );
    let added = result.len() - before;
    if added > 0 {
        println!("+{added} shared dependency bundle(s) added via manifest closure");
    }
    result
}
