use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::Subcommand;
use myrtle_app::context::GameData;
use myrtle_app::services::accounts::service_account::{
    ImportOutcome, ServiceLogin, import_service_sessions, send_service_code, send_service_sms,
    service_session_status, set_service_session,
};
use myrtle_hypergryph::login::bilibili::OtpChallenge;
use myrtle_hypergryph::{AuthSession, DeviceIds};
use myrtle_types::Server;

use crate::Env;

pub const PENDING_OTP: &str = "myrtle-bili-otp.json";

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Email a Yostar login code
    SendCode {
        #[arg(long)]
        email: String,
        /// en, jp or kr; default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
    /// Login with OTP
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        code: String,
        /// en, jp or kr; default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
    /// Bilibili password login for bili server
    Bili {
        /// Phone, email or username
        #[arg(long)]
        user: String,
        #[arg(long)]
        pass: String,
    },
    /// Bilibili SMS login
    Sms {
        /// National number without a country code
        #[arg(long)]
        phone: String,
        /// Default `86`
        #[arg(long, conflicts_with = "otp")]
        country_code: Option<String>,
        /// The texted code
        #[arg(long)]
        otp: Option<String>,
    },
    /// Import an `AuthSession` JSON file
    Import {
        path: PathBuf,
        /// Default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
    /// Import every <server>.json session file in a directory
    ImportDir {
        dir: PathBuf,
        /// Replace sessions already stored
        #[arg(long)]
        force: bool,
    },
    /// Refresh stored session to prove it still works
    Status {
        /// Default `[cli] default_server`
        #[arg(long)]
        server: Option<String>,
    },
}

pub async fn run(env: &Env, command: Command) -> anyhow::Result<()> {
    if needs_device_ids(&command) {
        let hypergryph = &env.config.hypergryph;
        require_device_ids(&DeviceIds {
            device_id: hypergryph.device_id.clone(),
            device_id2: hypergryph.device_id2.clone(),
            device_id3: hypergryph.device_id3.clone(),
        })?;
    }
    let ctx = env.ctx(GameData::None).await?;
    match command {
        Command::SendCode { email, server } => {
            let server = env.server(server.as_deref())?;
            send_service_code(&ctx, server, &email).await?;
            println!("code sent to {email}; now run:");
            println!(
                "  myrtle session login --server {} --email {email} --code <code>",
                server.as_str()
            );
        }
        Command::Login {
            email,
            code,
            server,
        } => {
            let server = env.server(server.as_deref())?;
            let login = ServiceLogin::Yostar {
                email: &email,
                code: &code,
            };
            set_service_session(&ctx, server, login).await?;
            stored(server);
        }
        Command::Bili { user, pass } => {
            let login = ServiceLogin::Bilibili {
                username: &user,
                password: &pass,
            };
            set_service_session(&ctx, Server::Bilibili, login).await?;
            stored(Server::Bilibili);
        }
        Command::Sms {
            phone,
            country_code,
            otp: None,
        } => {
            let country_code = country_code.as_deref().unwrap_or("86");
            let challenge = send_service_sms(&ctx, &phone, country_code).await?;
            let pending = Path::new(PENDING_OTP);
            write_pending(pending, &challenge)?;
            println!("code texted to +{country_code} {phone}; within 5 minutes run:");
            println!("  myrtle session sms --phone {phone} --otp <code>");
        }
        Command::Sms {
            phone,
            otp: Some(otp),
            ..
        } => {
            let pending = Path::new(PENDING_OTP);
            let challenge = read_pending(pending)?;
            check_pending(&challenge, &phone, unix_now())?;
            let login = ServiceLogin::BilibiliSms {
                challenge: &challenge,
                code: &otp,
            };
            set_service_session(&ctx, Server::Bilibili, login).await?;
            let _ = std::fs::remove_file(pending);
            stored(Server::Bilibili);
        }
        Command::Import { path, server } => {
            let server = env.server(server.as_deref())?;
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let session: AuthSession = serde_json::from_str(&text)
                .with_context(|| format!("{} is not an AuthSession", path.display()))?;
            set_service_session(&ctx, server, ServiceLogin::Import(session)).await?;
            stored(server);
        }
        Command::ImportDir { dir, force } => import_dir(&ctx, &dir, force).await?,
        Command::Status { server } => {
            let server = env.server(server.as_deref())?;
            let status = service_session_status(&ctx, server).await?;
            println!("server  {}", server.as_str());
            println!("uid     {}", status.uid);
            println!("seqnum  {}", status.seqnum);
            println!("refresh ok");
        }
    }
    Ok(())
}

async fn import_dir(ctx: &myrtle_app::AppContext, dir: &Path, force: bool) -> anyhow::Result<()> {
    let outcomes = import_service_sessions(ctx, dir, force).await;
    if outcomes.is_empty() {
        bail!("no <server>.json session files in {}", dir.display());
    }
    let mut failed = 0;
    for (server, outcome) in &outcomes {
        match outcome {
            ImportOutcome::Imported => println!("{:<5} imported", server.as_str()),
            ImportOutcome::Existing => {
                println!(
                    "{:<5} kept the stored session; --force replaces it",
                    server.as_str()
                );
            }
            ImportOutcome::Failed(reason) => {
                failed += 1;
                println!("{:<5} failed: {reason}", server.as_str());
            }
        }
    }
    if failed > 0 {
        bail!("{failed} of {} session files failed", outcomes.len());
    }
    Ok(())
}

fn stored(server: Server) {
    println!(
        "{} session stored; the backend renews it from the durable token",
        server.as_str()
    );
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub const fn needs_device_ids(command: &Command) -> bool {
    match command {
        Command::Login { .. }
        | Command::Bili { .. }
        | Command::Import { .. }
        | Command::Status { .. } => true,
        Command::Sms { otp, .. } => otp.is_some(),
        Command::SendCode { .. } | Command::ImportDir { .. } => false,
    }
}

pub fn require_device_ids(ids: &DeviceIds) -> anyhow::Result<()> {
    if !ids.is_complete() {
        bail!(
            "[hypergryph] device_id, device_id2 and device_id3 must be set, or every login looks like a new device; generate them with `myrtle hypergryph device-ids`"
        );
    }
    Ok(())
}

pub fn write_pending(path: &Path, challenge: &OtpChallenge) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(challenge)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let written = options.open(path).and_then(|mut file| {
        #[cfg(unix)]
        file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        file.write_all(&bytes)
    });
    written.with_context(|| format!("writing {}", path.display()))
}

pub fn read_pending(path: &Path) -> anyhow::Result<OtpChallenge> {
    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "no pending code at {}; run without --otp first",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("the pending code file {} is corrupt", path.display()))
}

pub fn check_pending(challenge: &OtpChallenge, phone: &str, now: u64) -> anyhow::Result<()> {
    if challenge.phone != phone {
        bail!(
            "the pending code was sent to {}, not {phone}",
            challenge.phone
        );
    }
    if challenge.is_expired(now) {
        bail!("the code has expired; run again without --otp for a new one");
    }
    Ok(())
}
