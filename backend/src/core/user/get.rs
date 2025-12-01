use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use tokio::sync::RwLock;

use super::types::{User, UserResponse};
use crate::core::{authentication::{
    auth_request,
    config::GlobalConfig,
    constants::{AuthSession, FetchError, Server},
}, local::types::GameData};

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
pub fn format_user(
    user: User,
    game_data: &GameData,  // Pre-loaded reference
) -> FormattedUser {
    // O(1) inventory enrichment
    let enriched_inventory: HashMap<String, EnrichedItem> = user.inventory
        .into_iter()
        .filter_map(|(id, amount)| {
            game_data.materials.get(&id).map(|item| {
                (id, EnrichedItem {
                    base: item.clone(),
                    amount,
                })
            })
        })
        .collect();

    // O(1) character enrichment (already pre-computed!)
    let enriched_chars: HashMap<String, EnrichedCharacter> = user.troop.chars
        .into_iter()
        .map(|(inst_id, char)| {
            let static_data = game_data.operators.get(&char.char_id).cloned();
            let trust = game_data.calculate_trust(char.favor_point);

            (inst_id, EnrichedCharacter {
                base: char,
                static_data,
                trust,
            })
        })
        .collect();

    FormattedUser {
        inventory: enriched_inventory,
        troop: EnrichedTroop { chars: enriched_chars, ..user.troop },
        ..user
    }
}
