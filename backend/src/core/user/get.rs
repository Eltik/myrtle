use std::sync::Arc;

use reqwest::Client;
use tokio::sync::RwLock;

use super::types::{User, UserResponse};
use crate::core::authentication::{
    auth_request,
    config::GlobalConfig,
    constants::{AuthSession, FetchError, Server},
};

pub async fn get(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    session: &mut AuthSession,
    server: Server,
) -> Result<Option<User>, FetchError> {
    let body = serde_json::json!({ "platform": 1 });

    let response = auth_request(
        client,
        config,
        "account/syncData",
        Some(body),
        session,
        server,
    )
    .await?;

    // DEBUG
    //let text = response.text().await.map_err(FetchError::RequestFailed)?;
    //eprintln!("Raw response: {}", text);

    let data: UserResponse = response.json().await.map_err(FetchError::RequestFailed)?;

    //let data: UserResponse = serde_json::from_str(&text).map_err(|e| FetchError::ParseError(format!("JSON parse error: {}", e)))?;

    Ok(data.user.map(format_user))
}

// TODO: When local data is processed, add formatting logic here
fn format_user(user: User) -> User {
    // Future: Add operator static data, trust calculations, etc.
    user
}
