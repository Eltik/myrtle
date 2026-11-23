use myrtle_db::repos::{credentials, users};
use myrtle_hypergryph::login::bilibili::{OTP_TTL_SECS, OtpChallenge};
use myrtle_hypergryph::login::passport::{self, PassportCredential};
use myrtle_hypergryph::login::yostar::{self, AccountPortalSession};
use myrtle_hypergryph::{AuthSession, DurableCredential, SessionKey, UpstreamError, session};
use myrtle_types::{GameUid, Server, UserId};
use serde::Serialize;

use super::jwt::{create_token, parse_role};
use super::sessions::{PORTAL_SESSION_TTL, portal_session_key};
use crate::privacy::mask_identifier;
use crate::{AppContext, AppError};

const BILIBILI_SMS_COUNTRY: &str = "86";

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub uid: String,
    pub server: String,
}

pub fn parse_server(s: &str) -> Result<Server, AppError> {
    Server::ALL
        .into_iter()
        .find(|server| server.as_str() == s)
        .ok_or_else(|| AppError::bad_request(format!("unknown server: {s}")))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn logged<T>(
    result: Result<T, UpstreamError>,
    who: &str,
    what: &'static str,
) -> Result<T, AppError> {
    result.map_err(|e| {
        tracing::warn!(who = %mask_identifier(who), error = ?e, "{what} failed");
        AppError::Upstream(e)
    })
}

#[must_use]
pub fn bilibili_sms_key(phone: &str) -> String {
    format!("bilibili_sms:{phone}")
}

pub async fn send_code(ctx: &AppContext, email: &str, server: Server) -> Result<(), AppError> {
    let sent = yostar::send_code(&ctx.upstream, email, server).await;
    logged(sent, email, "yostar send_code").map(drop)
}

pub async fn login(
    ctx: &AppContext,
    email: &str,
    code: &str,
    server: Server,
    save_credentials: bool,
) -> Result<LoginResponse, AppError> {
    let result = session::login(&ctx.upstream, email, code, server).await;
    let result = logged(result, email, "yostar login")?;
    complete_login(
        ctx,
        server,
        &result.session,
        result.portal_session.as_ref(),
        save_credentials,
    )
    .await
}

pub async fn login_bilibili(
    ctx: &AppContext,
    username: &str,
    password: &str,
    save_credentials: bool,
) -> Result<LoginResponse, AppError> {
    let session = session::login_bilibili(&ctx.upstream, username, password).await;
    let session = logged(session, username, "bilibili login")?;
    complete_login(ctx, Server::Bilibili, &session, None, save_credentials).await
}

pub async fn send_bilibili_sms(ctx: &AppContext, phone: &str) -> Result<(), AppError> {
    let challenge = session::send_bilibili_sms(&ctx.upstream, phone, BILIBILI_SMS_COUNTRY).await;
    let challenge = logged(challenge, phone, "bilibili send_otp")?;
    let ttl = std::time::Duration::from_secs(OTP_TTL_SECS);
    ctx.cache
        .set_json(&bilibili_sms_key(phone), &challenge, ttl)
        .await;
    Ok(())
}

pub async fn login_bilibili_sms(
    ctx: &AppContext,
    phone: &str,
    sms_code: &str,
    save_credentials: bool,
) -> Result<LoginResponse, AppError> {
    let key = bilibili_sms_key(phone);
    let challenge = ctx
        .cache
        .get_json::<OtpChallenge>(&key)
        .await
        .filter(|c| !c.is_expired(unix_now()))
        .ok_or_else(|| AppError::bad_request("no live code for this number; send a new one"))?;
    let session = session::login_bilibili_sms(&ctx.upstream, &challenge, sms_code).await;
    let session = logged(session, phone, "bilibili sms login")?;
    ctx.cache.invalidate(&key).await;
    complete_login(ctx, Server::Bilibili, &session, None, save_credentials).await
}

pub async fn send_code_cn(ctx: &AppContext, phone: &str) -> Result<(), AppError> {
    let sent = passport::send_phone_code(&ctx.upstream, phone).await;
    logged(sent, phone, "hypergryph passport send_phone_code")
}

pub async fn login_cn(
    ctx: &AppContext,
    phone: &str,
    password: Option<&str>,
    sms_code: Option<&str>,
    save_credentials: bool,
) -> Result<LoginResponse, AppError> {
    let credential = match (password, sms_code) {
        (Some(password), _) => PassportCredential::Password { phone, password },
        (None, Some(code)) => PassportCredential::SmsCode { phone, code },
        (None, None) => return Err(AppError::bad_request("provide either password or code")),
    };
    let session = session::login_cn(&ctx.upstream, credential).await;
    let session = logged(session, phone, "hypergryph CN login")?;
    complete_login(ctx, Server::CN, &session, None, save_credentials).await
}

pub async fn complete_login(
    ctx: &AppContext,
    server: Server,
    session: &AuthSession,
    portal: Option<&AccountPortalSession>,
    save_credentials: bool,
) -> Result<LoginResponse, AppError> {
    let uid = &*session.uid;
    let key = SessionKey::Player {
        server,
        uid: GameUid(uid.into()),
    };
    ctx.sessions
        .save(&key, session)
        .await
        .map_err(AppError::internal)?;
    if let Some(portal) = portal {
        ctx.cache
            .set_json(&portal_session_key(uid), portal, PORTAL_SESSION_TTL)
            .await;
    }

    let user = users::find_or_create(&mut *ctx.db.conn().await?, uid, server).await?;
    let id = UserId(user.id);
    persist_credentials(ctx, id, &key, session, save_credentials).await;

    let token = create_token(
        &ctx.secrets.jwt,
        id,
        uid,
        server,
        parse_role(&user.role),
        unix_now(),
    )?;
    Ok(LoginResponse {
        token,
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
    })
}

async fn persist_credentials(
    ctx: &AppContext,
    user: UserId,
    key: &SessionKey,
    session: &AuthSession,
    save: bool,
) {
    let uid = &*session.uid;
    if save {
        let Some(credential) = DurableCredential::of(session) else {
            return;
        };
        if let Err(e) = ctx.sessions.save_durable(key, &credential).await {
            tracing::warn!(uid, error = %e.detail(), "failed to persist durable game credentials; resync will need a re-login");
        }
        return;
    }
    let removed = match ctx.db.conn().await {
        Ok(mut conn) => credentials::delete(&mut conn, user).await,
        Err(e) => Err(e),
    };
    match removed {
        Ok(removed) => {
            tracing::info!(uid, removed, "login opted out of storing game credentials");
        }
        Err(e) => tracing::warn!(
            uid,
            error = %e,
            "login opted out of storing game credentials, but clearing the previous one failed"
        ),
    }
}
