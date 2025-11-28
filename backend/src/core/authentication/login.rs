use std::sync::Arc;

use reqwest::Client;
use tokio::sync::RwLock;

use crate::core::authentication::{
    config::GlobalConfig,
    constants::{AuthSession, FetchError, Server},
    get::{get_secret, get_u8_token},
    yostar::{request_token, submit_auth},
};
use crate::events::{ConfigEvent, EventEmitter};

pub async fn login(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    events: &Arc<EventEmitter>,
    email: &str,
    code: &str,
    server: Server,
    session: Option<AuthSession>,
) -> Result<AuthSession, FetchError> {
    let yostar_data = submit_auth(client, email, code, server).await?;
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

    Ok(session)
}
