//! Seeding the backend's own game accounts: a human logs one in once per server, the backend renews it.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use myrtle_hypergryph::login::bilibili::{self, OtpChallenge};
use myrtle_hypergryph::login::yostar;
use myrtle_hypergryph::{AuthSession, UpstreamError, session};
use myrtle_types::Server;

use super::sessions::Sessions;
use crate::{AppContext, AppError};

/// How the account signs in; a Bilibili login needs the `bili` server, a Yostar one any other
pub enum ServiceLogin<'a> {
    Yostar {
        email: &'a str,
        code: &'a str,
    },
    Bilibili {
        username: &'a str,
        password: &'a str,
    },
    /// The challenge [`send_service_sms`] returned, within its five minutes
    BilibiliSms {
        challenge: &'a OtpChallenge,
        code: &'a str,
    },
    /// A session produced elsewhere; it must still refresh
    Import(AuthSession),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSessionStatus {
    pub uid: String,
    pub seqnum: u32,
}

/// What became of one server's session file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOutcome {
    Imported,
    /// A stored session was kept; `force` replaces it
    Existing,
    /// The reason names a class, never the file's text
    Failed(String),
}

/// A session file is a few hundred bytes
const MAX_SESSION_FILE: u64 = 64 * 1024;

fn sessions(ctx: &AppContext) -> Sessions {
    Sessions::new(
        ctx.db.clone(),
        ctx.cache.clone(),
        Arc::clone(&ctx.secrets.credentials),
    )
}

fn store_error(e: &myrtle_hypergryph::SessionStoreError) -> AppError {
    AppError::internal(format!("session store {}: {}", e.kind(), e.detail()))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn has_credential(session: &AuthSession) -> bool {
    !session.yostar_uid.is_empty() && !session.yostar_token.is_empty()
}

/// Emails the login code for [`ServiceLogin::Yostar`]
pub async fn send_service_code(
    ctx: &AppContext,
    server: Server,
    email: &str,
) -> Result<(), AppError> {
    if server == Server::Bilibili {
        return Err(AppError::bad_request(
            "the bili server logs in with Bilibili, not an email code",
        ));
    }
    yostar::send_code(&ctx.upstream, email, server).await?;
    Ok(())
}

/// Texts the code for [`ServiceLogin::BilibiliSms`]; the caller keeps the challenge
pub async fn send_service_sms(
    ctx: &AppContext,
    phone: &str,
    country_code: &str,
) -> Result<OtpChallenge, AppError> {
    Ok(session::send_bilibili_sms(&ctx.upstream, phone, country_code).await?)
}

/// Signs the account in and replaces the server's stored session
pub async fn set_service_session(
    ctx: &AppContext,
    server: Server,
    login: ServiceLogin<'_>,
) -> Result<(), AppError> {
    let bilibili = matches!(
        login,
        ServiceLogin::Bilibili { .. } | ServiceLogin::BilibiliSms { .. }
    );
    if bilibili != (server == Server::Bilibili) && !matches!(login, ServiceLogin::Import(_)) {
        return Err(AppError::bad_request(if bilibili {
            "a Bilibili login needs the bili server"
        } else {
            "the bili server logs in with Bilibili, not an email code"
        }));
    }
    let session = match login {
        ServiceLogin::Yostar { email, code } => {
            session::login(&ctx.upstream, email, code, server)
                .await?
                .session
        }
        ServiceLogin::Bilibili { username, password } => {
            session::login_bilibili(&ctx.upstream, username, password).await?
        }
        ServiceLogin::BilibiliSms { challenge, code } => sms_session(ctx, challenge, code).await?,
        ServiceLogin::Import(mut imported) => {
            if !has_credential(&imported) {
                return Err(AppError::bad_request(
                    "the session has no durable token (yostar_uid / yostar_token), so it cannot be refreshed unattended",
                ));
            }
            session::refresh_secret(&ctx.upstream, &mut imported, server).await?;
            imported
        }
    };
    sessions(ctx)
        .seed(server, &session, true)
        .await
        .map(drop)
        .map_err(|e| store_error(&e))
}

/// Hypergryph's CN config may be out of reach from the box; the token is then stored unproven
async fn sms_session(
    ctx: &AppContext,
    challenge: &OtpChallenge,
    code: &str,
) -> Result<AuthSession, AppError> {
    if challenge.is_expired(unix_now()) {
        return Err(AppError::bad_request(
            "the code has expired; send a new one",
        ));
    }
    let login = bilibili::login_otp(&ctx.upstream, challenge, code).await?;
    let mut session = AuthSession {
        seqnum: 1,
        yostar_uid: login.uid.into(),
        yostar_token: login.access_key.into(),
        ..AuthSession::default()
    };
    match session::refresh_secret(&ctx.upstream, &mut session, Server::Bilibili).await {
        Ok(())
        | Err(
            UpstreamError::DomainNotFound { .. }
            | UpstreamError::Transport(_)
            | UpstreamError::Timeout(_),
        ) => Ok(session),
        Err(e) => Err(e.into()),
    }
}

/// Proves the stored session still refreshes and keeps the fresh secret, unless a seed landed meanwhile
pub async fn service_session_status(
    ctx: &AppContext,
    server: Server,
) -> Result<ServiceSessionStatus, AppError> {
    let store = sessions(ctx);
    let mut stored = store
        .service_session(server)
        .await
        .map_err(|e| store_error(&e))?
        .ok_or_else(|| AppError::NotFound(Some(format!("no session for {}", server.as_str()))))?;
    session::refresh_secret(&ctx.upstream, &mut stored, server).await?;
    store
        .persist_service(server, &stored)
        .await
        .map_err(|e| store_error(&e))?;
    Ok(ServiceSessionStatus {
        uid: stored.uid.to_string(),
        seqnum: stored.seqnum,
    })
}

/// Seals each `{server}.json` session file in `dir`, a stored session kept unless `force`; a bad file costs only its server
pub async fn import_service_sessions(
    ctx: &AppContext,
    dir: &Path,
    force: bool,
) -> Vec<(Server, ImportOutcome)> {
    let store = sessions(ctx);
    let mut outcomes = Vec::new();
    for server in Server::ALL {
        let path = dir.join(format!("{}.json", server.as_str()));
        let outcome = match read_session_file(&path) {
            Ok(None) => continue,
            Ok(Some(session)) => match store.seed(server, &session, force).await {
                Ok(true) => ImportOutcome::Imported,
                Ok(false) => ImportOutcome::Existing,
                Err(e) => ImportOutcome::Failed(format!("not stored ({})", e.kind())),
            },
            Err(reason) => ImportOutcome::Failed(reason),
        };
        if let ImportOutcome::Failed(reason) = &outcome {
            tracing::warn!(path = %path.display(), reason, "service session file skipped");
        }
        outcomes.push((server, outcome));
    }
    outcomes
}

/// `None` when absent; a symlink, an oversized file or one without a durable token is refused
fn read_session_file(path: &Path) -> Result<Option<AuthSession>, String> {
    let unreadable = |e: std::io::Error| format!("unreadable ({:?})", e.kind());
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(unreadable(e)),
    };
    if !meta.is_file() {
        return Err("not a regular file".to_owned());
    }
    let mut text = String::new();
    std::fs::File::open(path)
        .and_then(|f| f.take(MAX_SESSION_FILE + 1).read_to_string(&mut text))
        .map_err(unreadable)?;
    if text.len() as u64 > MAX_SESSION_FILE {
        return Err(format!("larger than {MAX_SESSION_FILE} bytes"));
    }
    let session: AuthSession = serde_json::from_str(&text).map_err(|e| {
        format!(
            "not a session ({:?} at line {}, column {})",
            e.classify(),
            e.line(),
            e.column()
        )
    })?;
    if !has_credential(&session) {
        return Err("no durable token".to_owned());
    }
    Ok(Some(session))
}
