//! Establish a service account session for one server.
//!
//! The game server has no anonymous mode, so impersonal reads still need a
//! logged-in account. This bin performs the one interactive step, a Yostar
//! email code, and writes a durable session that the backend then renews by
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
//!   cargo run --bin set-session -- --server bili --bili-phone 13800000000
//!   cargo run --bin set-session -- --server bili --bili-phone 13800000000 --otp 123456
//!   cargo run --bin set-session -- --server bili --bili-phone 2065550100 --country-code 1
//!   cargo run --bin set-session -- --server bili --bili-user 13800000000 --bili-pass '...'
//!   cargo run --bin set-session -- --server bili --import session.json
//!   cargo run --bin set-session -- --status
//!
//! Options:
//!   --email <addr>    Yostar account address (EN, JP, KR)
//!   --code <n>        Code from the email; with --email, performs the login
//!   --bili-phone <n>  Bilibili SMS login: the account's phone, national number.
//!                     Alone it texts a code; with --otp it logs in. Gets past
//!                     the `200007` risk control that stops password login
//!   --country-code <c> With --bili-phone: the number's country code (default 86)
//!   --otp <code>      With --bili-phone: the texted code
//!   --bili-user <u>   Bilibili account (phone, email or username); the CN
//!                     Bilibili channel, password login through the `BiliGame` SDK
//!   --bili-pass <p>   Its password (set one at bilibili.com if the account only
//!                     has SMS login; the SDK's SMS endpoint is not known)
//!   --import <path>   Adopt an existing `AuthSession` JSON instead of logging in.
//!                     For bili, `yostar_uid` / `yostar_token` are the SDK's
//!                     `uid` / `access_key` as seen in the client's login traffic.
//!   --status          Verify the stored session still refreshes
//!   --server <code>   EN | JP | KR | bili (default: BIN_SERVER, else first of SERVERS)
//!   --out <path>      Override the session file path
//!
//! Sessions are stored one file per server under `GAME_SESSION_DIR`; the
//! Bilibili channel's is `bili.json` and serves the CN game data.

use anyhow::{Context, Result, bail};
use backend::{
    app::state::default_bin_server_from_env,
    core::{
        hypergryph::{
            bilibili::{self, OtpChallenge},
            config::{GlobalConfig, init_config},
            constants::{AuthSession, Server},
            fetch::FetchError,
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
           --bili-phone <n>  Bilibili SMS login for --server bili (national number;\n\
                             alone it texts a code, with --otp it logs in)\n\
           --country-code <c> With --bili-phone: country code (default 86)\n\
           --otp <code>      With --bili-phone: the texted code\n\
           --bili-user <u>   Bilibili account for --server bili (password login)\n\
           --bili-pass <p>   Its password\n\
           --import <path>   Adopt an existing AuthSession JSON instead of logging in\n\
           --status          Verify the stored session still refreshes\n\
           --server <code>   EN | JP | KR | bili (default: BIN_SERVER, else first of SERVERS)\n\
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

    if let Some(user) = flag("--bili-user") {
        if server != Server::Bilibili {
            bail!("--bili-user needs --server bili");
        }
        let Some(pass) = flag("--bili-pass") else {
            bail!("--bili-user needs --bili-pass");
        };
        let session = session::login_bilibili(&client, &user, &pass)
            .await
            .map_err(|e| anyhow::anyhow!("bilibili login failed: {e:?}"))?;
        return persist(&out, &session, server);
    }

    if let Some(phone) = flag("--bili-phone") {
        if server != Server::Bilibili {
            bail!("--bili-phone needs --server bili");
        }
        return bili_sms(&client, &out, &phone, flag("--otp").as_deref()).await;
    }

    let Some(email) = flag("--email") else {
        print_help();
        bail!("nothing to do: pass --email, --bili-phone, --bili-user, --import or --status");
    };
    if server == Server::Bilibili {
        bail!(
            "--server bili logs in with --bili-phone, --bili-user / --bili-pass or --import, not --email"
        );
    }

    if let Some(code) = flag("--code") {
        let result = session::login(&client, &email, &code, server)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {e:?}"))?;

        return persist(&out, &result.session, server);
    }

    send_code(&client, &email, server)
        .await
        .map_err(|e| anyhow::anyhow!("send_code failed: {e:?}"))?;
    println!("Login code sent to {email}.");
    println!("Now run: cargo run --bin set-session -- --email {email} --code <code>");
    Ok(())
}

/// The Bilibili SMS login, split over two runs like `--email`. The first texts
/// the code and keeps the challenge beside the session file (the second run
/// must come from the same device ids and name its `captcha_key`); the second
/// trades the code for the durable channel token.
///
/// The u8 exchange that proves the token needs Hypergryph's CN network config,
/// which a box outside China may not reach. Then the token is stored unproven:
/// the login itself just succeeded, and the backend proves it when it adopts
/// the file.
async fn bili_sms(
    client: &reqwest::Client,
    out: &Path,
    phone: &str,
    otp: Option<&str>,
) -> Result<()> {
    let pending = out.with_extension("otp.json");

    let Some(otp) = otp else {
        let country = flag("--country-code").unwrap_or_else(|| "86".to_owned());
        let challenge = session::send_bilibili_sms(client, phone, &country)
            .await
            .map_err(|e| anyhow::anyhow!("sending the code failed: {e:?}"))?;
        write_private(&pending, &serde_json::to_vec(&challenge)?)
            .with_context(|| format!("writing {}", pending.display()))?;
        println!("Code texted to +{country} {phone}. Within 5 minutes run:");
        println!(
            "  cargo run --bin set-session -- --server bili --bili-phone {phone} --otp <code>"
        );
        return Ok(());
    };

    let raw = std::fs::read(&pending).with_context(|| {
        format!(
            "no pending code at {}; run once without --otp first",
            pending.display()
        )
    })?;
    let challenge: OtpChallenge =
        serde_json::from_slice(&raw).context("the pending code file is corrupt")?;
    if challenge.phone != phone {
        bail!(
            "the pending code was sent to {}, not {phone}",
            challenge.phone
        );
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    if challenge.is_expired(now) {
        bail!("the code has expired; run again without --otp for a new one");
    }

    let login = bilibili::login_otp(client, &challenge, otp)
        .await
        .map_err(|e| anyhow::anyhow!("bilibili SMS login failed: {e:?}"))?;
    let _ = std::fs::remove_file(&pending);

    let mut stored = AuthSession {
        seqnum: 1,
        yostar_uid: login.uid.into(),
        yostar_token: login.access_key.into(),
        ..AuthSession::default()
    };
    match session::refresh_secret(client, &mut stored, Server::Bilibili).await {
        Ok(()) => {}
        Err(e @ (FetchError::DomainNotFound(..) | FetchError::RequestFailed(_))) => {
            println!("warning: Hypergryph is unreachable from here ({e:?}).");
            println!("The durable token is stored unproven; the backend proves it on adoption.");
        }
        Err(e) => bail!("the new token was refused by Hypergryph: {e:?}"),
    }
    persist(out, &stored, Server::Bilibili)
}

/// Write `bytes` readable by the owner only.
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
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
        Err(e) => {
            let relogin = if server == Server::Bilibili {
                "--bili-user/--bili-pass"
            } else {
                "--email"
            };
            bail!("refresh failed: {e:?} - re-run with {relogin} to log in again")
        }
    }
}

fn persist(out: &Path, session: &AuthSession, server: Server) -> Result<()> {
    write_session_file(out, session)
        .with_context(|| format!("writing session to {}", out.display()))?;

    println!("Session stored at {}", out.display());
    println!("  uid    : {}", session.uid);
    println!("  server : {}", server.as_str());
    println!();
    println!("The backend will renew this itself from the durable channel token.");
    println!("Keep the file secret - it holds live game credentials.");
    Ok(())
}
