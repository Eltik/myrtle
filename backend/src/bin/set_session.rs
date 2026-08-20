//! Establish a service account session for one server.
//!
//! The game server has no anonymous mode, so impersonal reads still need a
//! logged-in account. This bin performs the one interactive step — a Yostar
//! email code — and writes a durable session that the backend then renews by
//! itself. Run it once per server you want covered.
//!
//! Point it at a **dedicated account**. Only one `secret` is live per account,
//! so sharing one with a real player would sign them out of the game whenever
//! the backend refreshes.
//!
//! Usage:
//!   cargo run --bin set-session -- --email account@example.com
//!   cargo run --bin set-session -- --email account@example.com --code 123456
//!   cargo run --bin set-session -- --server jp --email account@example.com
//!   cargo run --bin set-session -- --status
//!
//! Options:
//!   --email <addr>    Yostar account address
//!   --code <n>        Code from the email; with --email, performs the login
//!   --import <path>   Adopt an existing `AuthSession` JSON instead of logging in
//!   --status          Verify the stored session still refreshes
//!   --server <code>   EN | JP | KR (default: BIN_SERVER, else first of SERVERS)
//!   --out <path>      Override the session file path
//!
//! Sessions are stored one file per server under `GAME_SESSION_DIR`.

use anyhow::{Context, Result, bail};
use backend::{
    app::state::default_bin_server_from_env,
    core::{
        hypergryph::{
            config::{GlobalConfig, init_config},
            constants::{AuthSession, Server},
            loaders, session,
            yostar::send_code,
        },
        service_account::{session_path, write_session_file},
    },
};
use dotenv::dotenv;
use std::path::{Path, PathBuf};

fn flag(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn has_flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

fn print_help() {
    eprintln!(
        "Usage: set-session [--email <addr>] [--code <n>] [--import <path>] [--status]\n\
         \n\
         Establishes the backend's game service account session.\n\
         \n\
         Options:\n\
           --email <addr>    Yostar account address (alone: sends a login code)\n\
           --code <n>        Code from the email; combine with --email to log in\n\
           --import <path>   Adopt an existing AuthSession JSON instead of logging in\n\
           --status          Verify the stored session still refreshes\n\
           --server <code>   EN | JP | KR (default: BIN_SERVER, else first of SERVERS)\n\
           --out <path>      Override the session file path\n\
           -h, --help        Show this message\n\
         \n\
         Sessions are stored one file per server under GAME_SESSION_DIR.\n\
         Use a DEDICATED account: only one secret is live per account, so the\n\
         backend refreshing would sign a real player out of the game."
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    if has_flag("-h") || has_flag("--help") {
        print_help();
        return Ok(());
    }

    let server = match flag("--server") {
        Some(code) => {
            Server::parse(&code).with_context(|| format!("unknown server code {code:?}"))?
        }
        None => default_bin_server_from_env(),
    };
    let out: PathBuf = flag("--out").map_or_else(|| session_path(server), PathBuf::from);

    // Device ids, network config and the live client version. Without this the
    // game server rejects the login as an out-of-date client.
    let client = reqwest::Client::new();
    init_config(GlobalConfig::new());
    loaders::init(&client).await;

    if has_flag("--status") {
        return status(&client, server, &out).await;
    }

    if let Some(path) = flag("--import") {
        return import(&client, server, &out, &path).await;
    }

    let Some(email) = flag("--email") else {
        print_help();
        bail!("nothing to do: pass --email, --import or --status");
    };

    // Step 2: exchange the code for a durable session.
    if let Some(code) = flag("--code") {
        let result = session::login(&client, &email, &code, server)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {e:?}"))?;

        return persist(&out, &result.session, server);
    }

    // Step 1: ask Yostar to email a code.
    send_code(&client, &email, server)
        .await
        .map_err(|e| anyhow::anyhow!("send_code failed: {e:?}"))?;
    println!("Login code sent to {email}.");
    println!("Now run: cargo run --bin set-session -- --email {email} --code <code>");
    Ok(())
}

/// Adopt a session produced elsewhere, saving an OTP round-trip when usable
/// credentials already exist.
async fn import(client: &reqwest::Client, server: Server, out: &Path, from: &str) -> Result<()> {
    let raw = std::fs::read_to_string(from).with_context(|| format!("reading {from}"))?;
    let mut imported: AuthSession =
        serde_json::from_str(&raw).with_context(|| format!("parsing {from} as an AuthSession"))?;

    if imported.yostar_uid.is_empty() || imported.yostar_token.is_empty() {
        bail!(
            "{from} has no durable Yostar token (yostar_uid / yostar_token), so it cannot be \
             refreshed unattended. Log in with --email instead."
        );
    }

    // Prove the durable token still works before adopting it.
    session::refresh_secret(client, &mut imported, server)
        .await
        .map_err(|e| anyhow::anyhow!("imported session could not be refreshed: {e:?}"))?;

    persist(out, &imported, server)
}

async fn status(client: &reqwest::Client, server: Server, out: &Path) -> Result<()> {
    let raw =
        std::fs::read_to_string(out).with_context(|| format!("no session at {}", out.display()))?;
    let mut stored: AuthSession =
        serde_json::from_str(&raw).context("stored session is corrupt")?;

    println!("file    : {}", out.display());
    println!("uid     : {}", stored.uid);
    println!("server  : {}", server.as_str());
    println!("seqnum  : {}", stored.seqnum);
    println!(
        "durable : {}",
        if stored.yostar_token.is_empty() {
            "NO - cannot refresh unattended"
        } else {
            "yes"
        }
    );

    match session::refresh_secret(client, &mut stored, server).await {
        Ok(()) => {
            println!("refresh : ok");
            // Keep the freshly minted secret rather than leaving a stale one on
            // disk; --status is idempotent apart from this.
            write_session_file(out, &stored).context("persisting refreshed session")?;
            Ok(())
        }
        Err(e) => bail!("refresh failed: {e:?} - re-run with --email to log in again"),
    }
}

fn persist(out: &Path, session: &AuthSession, server: Server) -> Result<()> {
    write_session_file(out, session)
        .with_context(|| format!("writing session to {}", out.display()))?;

    println!("Session stored at {}", out.display());
    println!("  uid    : {}", session.uid);
    println!("  server : {}", server.as_str());
    println!();
    println!("The backend will renew this itself from the durable Yostar token.");
    println!("Keep the file secret - it holds live game credentials.");
    Ok(())
}
