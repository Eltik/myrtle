use reqwest::Client;
use serde::Deserialize;

use crate::api::CONFIG_TIMEOUT;

/// Whether a backend service account's durable token still refreshes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountState {
    Ok,
    Refused,
    /// Not used yet since the backend started. A state this bot does not know reads the same.
    #[serde(other)]
    Unknown,
}

impl AccountState {
    /// The form `service_account_announced.state` stores.
    #[must_use]
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Refused => "refused",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "ok" => Self::Ok,
            "refused" => Self::Refused,
            _ => Self::Unknown,
        }
    }
}

/// One account from `GET /api/service-accounts`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AccountStatus {
    /// The backend's server code: `en`, `jp`, `kr`, `cn` or `bili`.
    pub server: String,
    pub state: AccountState,
    /// Unix seconds of the first refusal in the current outage.
    #[serde(default)]
    pub refused_since: Option<i64>,
    /// Unix seconds of the last refresh or request the game accepted.
    #[serde(default)]
    pub last_ok_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceAccountsResponse {
    pub accounts: Vec<AccountStatus>,
}

/// Fetch every service account's state. Any error, a non-2xx answer included, means the backend
/// could not be asked, never that an account is down.
pub async fn service_accounts(
    client: &Client,
    base_url: &str,
) -> Result<ServiceAccountsResponse, reqwest::Error> {
    let url = format!("{}/api/service-accounts", base_url.trim_end_matches('/'));
    client
        .get(&url)
        .timeout(CONFIG_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json::<ServiceAccountsResponse>()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_backend_shape() {
        let body = r#"{"accounts": [
            {"server": "bili", "state": "refused", "refused_since": 1762000000, "last_ok_at": null},
            {"server": "en", "state": "ok", "refused_since": null, "last_ok_at": 1762000100},
            {"server": "jp", "state": "something_new", "refused_since": null, "last_ok_at": null}
        ]}"#;
        let parsed: ServiceAccountsResponse = serde_json::from_str(body).unwrap();
        assert_eq!(
            parsed.accounts[0],
            AccountStatus {
                server: "bili".into(),
                state: AccountState::Refused,
                refused_since: Some(1_762_000_000),
                last_ok_at: None,
            }
        );
        assert_eq!(parsed.accounts[1].state, AccountState::Ok);
        assert_eq!(parsed.accounts[2].state, AccountState::Unknown);
    }

    #[test]
    fn db_strings_round_trip() {
        for state in [
            AccountState::Ok,
            AccountState::Refused,
            AccountState::Unknown,
        ] {
            assert_eq!(AccountState::from_db_str(state.as_db_str()), state);
        }
    }
}
