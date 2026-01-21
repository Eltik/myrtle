use std::sync::Arc;

use reqwest::Client;
use tokio::sync::RwLock;

use crate::core::authentication::{
    config::GlobalConfig,
    constants::{AuthSession, FetchError, Server},
    get::{get_secret, get_u8_token},
    yostar::{AccountPortalSession, account_portal_login, request_token, submit_auth},
};
use crate::events::{ConfigEvent, EventEmitter};

pub struct LoginResult {
    pub session: AuthSession,
    pub yostar_email: String,
    pub portal_session: Option<AccountPortalSession>,
}

pub async fn login(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
    email: &str,
    code: &str,
    server: Server,
    session: Option<AuthSession>,
) -> Result<LoginResult, FetchError> {
    let yostar_data = submit_auth(client, email, code, server).await?;

    // Get YSSID cookies from account portal (only for Yostar servers)
    let portal_session = if server.yostar_domain().is_some() {
        match account_portal_login(client, email, &yostar_data.token).await {
            Ok(session) => Some(session),
            Err(e) => {
                eprintln!("Warning: Failed to get YSSID cookies: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    let token_data = request_token(client, email, &yostar_data.token, server).await?;

    let mut session = session.unwrap_or_else(|| AuthSession::new(None, None, None, None));

    let data = get_u8_token(client, config, &token_data.uid, &token_data.token, server)
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))?;

    session.uid = data.uid;

    let u8_token = data.token;
    let secret = get_secret(client, config, events, &session.uid, &u8_token, server)
        .await
        .map_err(|e| FetchError::ParseError(e.to_string()))?;

    session.token = u8_token;
    session.secret = secret;

    events.emit(ConfigEvent::AuthLoginSuccess(session.clone()));

    Ok(LoginResult {
        session,
        yostar_email: email.to_string(),
        portal_session,
    })
}
