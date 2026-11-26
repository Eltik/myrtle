use chrono::{DateTime, Utc};
use myrtle_db::repos::oauth::{AppPublicRow, AppRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthApp {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub redirect_uris: Vec<String>,
    #[ts(type = "number")]
    pub authorized_users: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AppRow> for OAuthApp {
    fn from(row: AppRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            redirect_uris: row.redirect_uris,
            authorized_users: row.authorized_users,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthAppWithSecret {
    #[serde(flatten)]
    #[ts(flatten)]
    pub app: OAuthApp,
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthAppInput {
    pub name: String,
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthAppOwner {
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthAppPublic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner: OAuthAppOwner,
    pub created_at: DateTime<Utc>,
}

impl From<AppPublicRow> for OAuthAppPublic {
    fn from(row: AppPublicRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            owner: OAuthAppOwner {
                nickname: row.owner_nickname,
                nick_number: row.owner_nick_number,
            },
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthorizeParams {
    #[serde(default)]
    #[ts(optional)]
    pub response_type: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub client_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub scope: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub state: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub code_challenge_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuthorizeCheck {
    Consent {
        app: OAuthAppPublic,
        scopes: Vec<String>,
        redirect_uri: String,
        redirect_host: String,
        already_authorized: bool,
    },
    Redirect { redirect_to: String },
}

#[derive(Debug, Clone, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthorizeDecisionInput {
    #[serde(flatten)]
    #[ts(flatten)]
    pub params: AuthorizeParams,
    pub approve: bool,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AuthorizeDecision {
    pub redirect_to: String,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OAuthAuthorization {
    pub app: OAuthAppPublic,
    pub scopes: Vec<String>,
    pub authorized_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OAuthUserInfo {
    pub id: Uuid,
    pub username: String,
    pub tag: Option<String>,
    pub uid: String,
    pub server: String,
    pub avatar_id: Option<String>,
    pub avatar_url: String,
}
