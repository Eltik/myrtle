use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // User internal ID
    pub uid: String,    // Arknights UID
    pub server: String, // Game server
    pub role: String,   // User role
    pub exp: u64,       // Expiration timestamp
    pub iat: u64,       // Issued at
}

pub fn create_token(
    secret: &str,
    user_id: &str,
    uid: &str,
    server: &str,
    role: &str,
    expiry_days: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let claims = Claims {
        sub: user_id.to_owned(),
        uid: uid.to_owned(),
        server: server.to_owned(),
        role: role.to_owned(),
        exp: now + expiry_days * 86400,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|t| t.claims)
}
