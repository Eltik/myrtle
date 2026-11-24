pub mod commands;
pub mod env;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use commands::{
    db, gacha, gamedata, grants, hypergryph, jobs, migrate, regrade, session, stats, tier_list,
    user,
};

pub use env::Env;

#[derive(Debug, Parser)]
#[command(name = "myrtle", about = "Myrtle backend operator CLI")]
pub struct Cli {
    /// Config file; default config/myrtle.toml
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    /// Log warnings and errors only
    #[arg(long, short, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Apply pending migrations
    Migrate,
    /// Compile or publish a server's game data
    Gamedata {
        #[command(subcommand)]
        command: gamedata::Command,
    },
    /// Whole-database export, import and legacy translation
    Db {
        #[command(subcommand)]
        command: db::Command,
    },
    /// One user's data
    User {
        #[command(subcommand)]
        command: user::Command,
    },
    /// Regrade users
    Regrade(regrade::Args),
    /// Users, roles and tier-list permissions
    Grants {
        #[command(subcommand)]
        command: grants::Command,
    },
    /// Tier-list administration
    TierList {
        #[command(subcommand)]
        command: tier_list::Command,
    },
    /// Gacha maintenance
    Gacha {
        #[command(subcommand)]
        command: gacha::Command,
    },
    /// Recompute community stats now
    Stats(stats::Args),
    /// Service-account sessions
    Session {
        #[command(subcommand)]
        command: session::Command,
    },
    /// Job queue
    Jobs {
        #[command(subcommand)]
        command: jobs::Command,
    },
    /// Game-client identity
    Hypergryph {
        #[command(subcommand)]
        command: hypergryph::Command,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let command = match cli.command {
        Command::Hypergryph { command } => return hypergryph::run(command),
        command => command,
    };
    let env = Env::load(cli.config.as_deref())?;
    match command {
        Command::Migrate => migrate::run(&env).await,
        Command::Gamedata { command } => gamedata::run(&env, command).await,
        Command::Db { command } => db::run(&env, command).await,
        Command::User { command } => user::run(&env, command).await,
        Command::Regrade(args) => regrade::run(&env, args).await,
        Command::Grants { command } => grants::run(&env, command).await,
        Command::TierList { command } => tier_list::run(&env, command).await,
        Command::Gacha { command } => gacha::run(&env, command).await,
        Command::Stats(args) => stats::run(&env, args).await,
        Command::Session { command } => session::run(&env, command).await,
        Command::Jobs { command } => jobs::run(&env, command).await,
        Command::Hypergryph { .. } => unreachable!("handled before the config loads"),
    }
}
