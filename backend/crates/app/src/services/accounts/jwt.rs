use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use myrtle_types::{Role, Server, UserId};
use serde::{Deserialize, Serialize};

use crate::AppError;

pub const TOKEN_DAYS: u64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// `users.id`
    pub sub: String,
    /// The game account uid
    pub uid: String,
    pub server: String,
    pub role: String,
    pub exp: u64,
    pub iat: u64,
}

impl Claims {
    pub fn user_id(&self) -> Result<UserId, AppError> {
        self.sub
            .parse()
            .map(UserId)
            .map_err(|_| AppError::Unauthorized)
    }

    /// Exact codes only, as [`super::login::parse_server`]
    #[must_use]
    pub fn server(&self) -> Option<Server> {
        super::login::parse_server(&self.server).ok()
    }
}

/// The stored spelling of a role, as `users.role` and the token carry it
#[must_use]
pub const fn role_name(role: Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Translator => "translator",
        Role::TierListEditor => "tier_list_editor",
        Role::TierListAdmin => "tier_list_admin",
        Role::SuperAdmin => "super_admin",
    }
}

/// An unknown spelling reads as a plain user
#[must_use]
pub fn parse_role(s: &str) -> Role {
    [
        Role::Translator,
        Role::TierListEditor,
        Role::TierListAdmin,
        Role::SuperAdmin,
    ]
    .into_iter()
    .find(|role| role_name(*role) == s)
    .unwrap_or(Role::User)
}

pub fn create_token(
    secret: &str,
    user: UserId,
    uid: &str,
    server: Server,
    role: Role,
    now: u64,
) -> Result<String, AppError> {
    let claims = Claims {
        sub: user.0.to_string(),
        uid: uid.to_owned(),
        server: server.as_str().to_owned(),
        role: role_name(role).to_owned(),
        exp: now + TOKEN_DAYS * 86_400,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(AppError::internal)
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|t| t.claims)
    .map_err(|_| AppError::Unauthorized)
}
