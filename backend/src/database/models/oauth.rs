//! Rows and wire types of the OAuth 2.0 provider (`v046_oauth`).
//!
//! Two families. The management types (`OAuthApp`, `AuthorizeCheck`,
//! `OAuthAuthorization`, ...) are this site's own API, read by its web client
//! with a site token. The protocol types (`OAuthTokenResponse`,
//! `OAuthUserInfo`, `OAuthErrorBody`) are what a third party's server reads,
//! and their field names are fixed by RFC 6749 and RFC 7009 rather than chosen
//! here.

use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;
use utoipa::ToSchema;

/// `oauth_apps`, as its owner reads it. The secret is not here: it is shown
/// once, by the call that minted it ([`OAuthAppWithSecret`]).
#[derive(Debug, Clone, Serialize, TS, ToSchema, sqlx::FromRow)]
#[ts(export)]
pub struct OAuthApp {
    /// The application's `client_id`.
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Where a player may be sent back to. An authorization request must name
    /// one of these exactly.
    pub redirect_uris: Vec<String>,
    /// Players who currently have this application authorized.
    #[ts(type = "number")]
    pub authorized_users: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An application together with its client secret, returned by the two calls
/// that mint one: creating the application and rotating its secret. Only the
/// secret's hash is stored, so this response is the one time it can be read.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct OAuthAppWithSecret {
    #[serde(flatten)]
    #[ts(flatten)]
    pub app: OAuthApp,
    #[schema(example = "myrtle_cs_3q2-7wX0s0mZ0f1J9bq0vK1yQ4mB8sN5tU6vW7xY8z0")]
    pub client_secret: String,
}

/// The editable fields of an application. Creating one and updating one take
/// the same body; an update replaces all three.
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct OAuthAppInput {
    /// 1 to 64 characters. Shown to players on the consent screen.
    #[schema(example = "miowawa.de")]
    pub name: String,
    /// Up to 280 characters, shown under the name on the consent screen.
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
    /// 1 to 10 absolute `https` URLs (`http` is accepted for `localhost`,
    /// `127.0.0.1` and `[::1]`), without a fragment.
    #[schema(example = json!(["https://miowawa.de/auth/myrtle/callback"]))]
    pub redirect_uris: Vec<String>,
}

/// The account that registered an application, as the consent screen names it.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct OAuthAppOwner {
    pub nickname: Option<String>,
    /// The digits after the `#` in the in-game name.
    pub nick_number: Option<String>,
}

/// An application as anyone but its owner sees it: on the consent screen and
/// in a player's list of authorized applications.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct OAuthAppPublic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub owner: OAuthAppOwner,
    pub created_at: DateTime<Utc>,
}

/// `oauth_apps` joined to its owner, the row behind [`OAuthAppPublic`].
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthAppPublicRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub redirect_uris: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub owner_nickname: Option<String>,
    pub owner_nick_number: Option<String>,
}

impl OAuthAppPublicRow {
    pub fn into_public(self) -> OAuthAppPublic {
        OAuthAppPublic {
            id: self.id,
            name: self.name,
            description: self.description,
            owner: OAuthAppOwner {
                nickname: self.owner_nickname,
                nick_number: self.owner_nick_number,
            },
            created_at: self.created_at,
        }
    }
}

/// An authorization request as the browser carried it to the consent page:
/// the query parameters of RFC 6749 4.1.1 and RFC 7636 4.3, all optional here
/// so that a missing one is answered with the protocol's own error rather
/// than a deserialization failure.
#[derive(Debug, Clone, Default, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AuthorizeParams {
    /// Must be `code`.
    #[serde(default)]
    #[ts(optional)]
    pub response_type: Option<String>,
    /// The application's id.
    #[serde(default)]
    #[ts(optional)]
    pub client_id: Option<String>,
    /// One of the application's registered redirect URIs, exactly. May be
    /// left out when the application registered only one.
    #[serde(default)]
    #[ts(optional)]
    pub redirect_uri: Option<String>,
    /// Space-separated scopes. Defaults to `identify`, the only one.
    #[serde(default)]
    #[ts(optional)]
    pub scope: Option<String>,
    /// The application's own opaque value, returned to it unchanged.
    #[serde(default)]
    #[ts(optional)]
    pub state: Option<String>,
    /// PKCE: the base64url SHA-256 of the application's code verifier.
    #[serde(default)]
    #[ts(optional)]
    pub code_challenge: Option<String>,
    /// PKCE: must be `S256` when a challenge is sent.
    #[serde(default)]
    #[ts(optional)]
    pub code_challenge_method: Option<String>,
}

/// What the consent page should do with an authorization request.
///
/// A request naming an unknown application or an unregistered redirect URI is
/// not one of these: it is a 400, shown to the player, because there is no
/// trusted place to send them. Every other problem is the application's to
/// hear about, so it comes back as `redirect`.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AuthorizeCheck {
    /// The request is valid: ask the player.
    Consent {
        app: OAuthAppPublic,
        /// What the application is asking for.
        scopes: Vec<String>,
        /// Where the player goes after deciding.
        redirect_uri: String,
        /// The host of `redirect_uri` (with its port when it has one), for the
        /// screen to name.
        redirect_host: String,
        /// The player has authorized this application before.
        already_authorized: bool,
    },
    /// The request is invalid in a way the application is told about: send
    /// the browser to `redirect_to`, which carries the `error`.
    Redirect { redirect_to: String },
}

/// The player's answer on the consent page.
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[ts(export)]
pub struct AuthorizeDecisionInput {
    /// The authorization request, as the page received it.
    #[serde(flatten)]
    #[ts(flatten)]
    pub params: AuthorizeParams,
    /// `true` to authorize, `false` to cancel.
    pub approve: bool,
}

/// Where to send the browser after the player decided: the application's
/// redirect URI carrying either a `code` or `error=access_denied`, and the
/// request's `state` either way.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct AuthorizeDecision {
    pub redirect_to: String,
}

/// One application the caller has authorized.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct OAuthAuthorization {
    pub app: OAuthAppPublic,
    pub scopes: Vec<String>,
    pub authorized_at: DateTime<Utc>,
    /// The last time the application exchanged a code or refreshed a token.
    pub last_used_at: Option<DateTime<Utc>>,
}

/// `oauth_grants` joined to its application and that application's owner.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthAuthorizationRow {
    #[sqlx(flatten)]
    pub app: OAuthAppPublicRow,
    pub scope: String,
    pub authorized_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// A spent authorization code: what the token it buys is bound to.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConsumedCode {
    pub grant_id: Uuid,
    pub redirect_uri: String,
    pub scope: String,
    pub code_challenge: Option<String>,
}

/// Who an access token speaks for.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccessTokenOwner {
    pub user_id: Uuid,
    pub scope: String,
}

/// The columns of `users` that `/oauth/userinfo` returns.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthIdentityRow {
    pub id: Uuid,
    pub uid: String,
    /// `servers.code`.
    pub server: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
}

/// A successful token response (RFC 6749 5.1).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OAuthTokenResponse {
    #[schema(example = "myrtle_at_3q2-7wX0s0mZ0f1J9bq0vK1yQ4mB8sN5tU6vW7xY8z0")]
    pub access_token: String,
    /// Always `Bearer`.
    #[schema(example = "Bearer")]
    pub token_type: &'static str,
    /// Seconds until the access token expires.
    #[schema(example = 604_800)]
    pub expires_in: i64,
    /// Trades for a new pair at the token endpoint. Single use: each refresh
    /// returns a new one and retires this one.
    #[schema(example = "myrtle_rt_9bq0vK1yQ4mB8sN5tU6vW7xY8z03q2-7wX0s0mZ0f1J")]
    pub refresh_token: String,
    /// The granted scopes, space-separated.
    #[schema(example = "identify")]
    pub scope: String,
}

/// The identity behind an access token with the `identify` scope.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OAuthUserInfo {
    /// The myrtle.moe account id. Stable for the life of the account, and the
    /// value to key a connected account on.
    pub id: Uuid,
    /// The player's in-game nickname. Not unique, and it changes when they
    /// rename in game and sync again. The game uid stands in for an account
    /// whose nickname has not synced yet.
    #[schema(example = "Doctor")]
    pub username: String,
    /// The digits after the `#` in the in-game name, when the game gave one.
    #[schema(example = "1234")]
    pub tag: Option<String>,
    /// The player's in-game uid on `server`. `https://myrtle.moe/user/{uid}`
    /// is their profile page, which only opens when they made it public.
    #[schema(example = "12345678")]
    pub uid: String,
    /// The game server the account is on, lowercase: `en`, `jp`, `kr`, `cn`,
    /// `bili` or `tw`.
    #[schema(example = "en")]
    pub server: String,
    /// The id of the avatar the player picked in game; `null` when they never
    /// set one, in which case `avatar_url` is the default.
    #[schema(example = "char_002_amiya")]
    pub avatar_id: Option<String>,
    /// A square PNG of that avatar.
    #[schema(example = "https://api.myrtle.moe/api/avatar/char_002_amiya")]
    pub avatar_url: String,
}

/// The error body of the token, revocation and userinfo endpoints
/// (RFC 6749 5.2). These three speak the OAuth error format instead of this
/// API's usual envelope, because the caller is an OAuth client library.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OAuthErrorBody {
    /// `invalid_request`, `invalid_client`, `invalid_grant`,
    /// `unsupported_grant_type`, `invalid_scope`, `invalid_token`,
    /// `server_error` or `temporarily_unavailable`.
    #[schema(example = "invalid_grant")]
    pub error: &'static str,
    #[schema(example = "the authorization code is invalid, expired or already used")]
    pub error_description: String,
}
