//! Standalone application-command deployer.
//!
//! Usage:
//!   cargo run --bin commands -- deploy global          # overwrite the global command set
//!   cargo run --bin commands -- deploy guild <id>      # overwrite one guild's commands
//!   cargo run --bin commands -- reset  global          # clear all global commands
//!   cargo run --bin commands -- reset  guild <id>      # clear one guild's commands
//!   cargo run --bin commands -- reset  guild all       # clear commands in every guild the bot is in

use std::env;

use discord::cmds;
use discord::types::Error;
use poise::builtins::create_application_commands;
use serenity::all::{Command, CreateCommand, GuildId};
use serenity::http::{GuildPagination, Http};

const USAGE: &str = "\
usage: commands <deploy|reset> <global|guild> [<guild_id>|all]
  deploy global          overwrite the global command set
  deploy guild <id>      overwrite a single guild's commands
  reset  global          clear all global commands
  reset  guild <id>      clear a single guild's commands
  reset  guild all       clear commands in every guild the bot is in";

fn builders() -> Vec<CreateCommand> {
    create_application_commands(&cmds::all())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

enum Op {
    DeployGlobal,
    DeployGuild(GuildId),
    ResetGlobal,
    ResetGuild(GuildId),
    ResetGuildAll,
}

fn parse_args() -> Option<Op> {
    let args: Vec<String> = env::args().skip(1).collect();
    let parts: Vec<&str> = args.iter().map(String::as_str).collect();
    match parts.as_slice() {
        ["deploy", "global"] => Some(Op::DeployGlobal),
        ["deploy", "guild", id] => Some(Op::DeployGuild(parse_guild(id)?)),
        ["reset", "global"] => Some(Op::ResetGlobal),
        ["reset", "guild", "all"] => Some(Op::ResetGuildAll),
        ["reset", "guild", id] => Some(Op::ResetGuild(parse_guild(id)?)),
        _ => None,
    }
}

async fn run() -> Result<(), Error> {
    let Some(op) = parse_args() else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };

    let token = env::var("DISCORD_TOKEN")?;
    let http = Http::new(&token);
    let app = http.get_current_application_info().await?;
    http.set_application_id(app.id);

    match op {
        Op::DeployGlobal => {
            let b = builders();
            let n = b.len();
            Command::set_global_commands(&http, b).await?;
            println!("OK: deployed {n} global command(s). Global propagation can take up to ~1h.");
        }
        Op::DeployGuild(gid) => {
            let b = builders();
            let n = b.len();
            gid.set_commands(&http, b).await?;
            println!("OK: deployed {n} command(s) to guild {gid} (effective instantly).");
        }
        Op::ResetGlobal => {
            Command::set_global_commands(&http, vec![]).await?;
            println!("OK: cleared all global commands.");
        }
        Op::ResetGuild(gid) => {
            gid.set_commands(&http, vec![]).await?;
            println!("OK: cleared commands in guild {gid}.");
        }
        Op::ResetGuildAll => {
            let guilds = all_guilds(&http).await?;
            for gid in &guilds {
                gid.set_commands(&http, vec![]).await?;
                println!("  cleared guild {gid}");
            }
            println!("OK: cleared commands in {} guild(s).", guilds.len());
        }
    }

    Ok(())
}

fn parse_guild(raw: &str) -> Option<GuildId> {
    raw.parse::<u64>().ok().map(GuildId::new)
}

/// Page through every guild the bot is a member of.
async fn all_guilds(http: &Http) -> Result<Vec<GuildId>, Error> {
    let mut out = Vec::new();
    let mut after: Option<GuildId> = None;
    loop {
        let batch = http
            .get_guilds(after.map(GuildPagination::After), Some(200))
            .await?;
        let len = batch.len();
        if let Some(last) = batch.last() {
            after = Some(last.id);
        }
        out.extend(batch.into_iter().map(|g| g.id));
        if len < 200 {
            break;
        }
    }
    Ok(out)
}
