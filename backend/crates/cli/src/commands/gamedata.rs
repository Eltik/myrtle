use clap::Subcommand;
use myrtle_app::context::GameData;
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_db::repos::data_generations::Generation;
use myrtle_types::Server;

use crate::Env;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check that a server's tables compile and list warnings
    Compile {
        /// Server code; default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
    /// Bump a server's data generation and queue the follow-up jobs
    Publish {
        /// Server code; default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Compile { server } => compile(env, env.server(server.as_deref())?).await,
        Command::Publish { server } => {
            let server = env.server(server.as_deref())?.data_server();
            let ctx = env.ctx(GameData::None).await?;
            let generation = ctx.announce_update(server, None).await?;
            println!("{} published at generation {generation}", server.as_str());
            Ok(())
        }
    }
}

async fn compile(env: &Env, server: Server) -> anyhow::Result<()> {
    let registry = SnapshotRegistry::new(&env.config);
    let server = server.data_server();
    let default = registry.default_server();
    let stored = Generation::default();
    if server != default {
        tracing::info!(
            server = default.as_str(),
            "loading the default server to align on"
        );
        registry.activate_at(default, &stored).await?;
    }
    tracing::info!(server = server.as_str(), "compiling game data");
    registry.activate_at(server, &stored).await?;
    report(&*registry.require(server)?);
    Ok(())
}

fn report(snapshot: &Snapshot) {
    let model = &snapshot.model;
    let meta = &model.meta;
    println!("server     {}", meta.server.as_str());
    println!("version    {}", meta.game_version.client);
    println!(
        "tables     {} operators, {} skills, {} stages",
        model.operators.len(),
        model.skills.len(),
        model.stages.len()
    );
    println!("skipped    {}", meta.skipped_tables.len());
    for skipped in &meta.skipped_tables {
        println!("  {} since {}", skipped.table, skipped.since);
    }
    println!("warnings   {}", meta.table_warnings.len());
    for warning in &meta.table_warnings {
        println!("  {warning}");
    }
}
