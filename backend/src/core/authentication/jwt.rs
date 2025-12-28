use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // User internal UUID
    pub uid: String,    // Arknights UID
    pub server: String, // Game server
    pub role: String,   // User role
    pub exp: i64,       // Expiration timestamp
    pub iat: i64,       // Issued at
}

pub fn create_token(
    secret: &str,
    user_id: Uuid,
    uid: &str,
    server: &str,
    role: &str,
    expiry_days: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(expiry_days);

    let claims = Claims {
        sub: user_id.to_string(),
        uid: uid.to_string(),
        server: server.to_string(),
        role: role.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
