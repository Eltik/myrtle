//! Sealing for durable game credentials at rest.
//!
//! A user's `AuthSession` carries two kinds of secret. The `secret` / `token`
//! pair is short-lived and re-derivable: [`refresh_secret`] mints a new one on
//! demand, and losing it costs one round-trip. The `yostar_uid` /
//! `yostar_token` pair is durable and NOT re-derivable — losing it costs the
//! user a full email-code login. That pair is what `user_game_credentials`
//! persists, and it is credential material rather than cache, so it does not
//! reach the database in the clear.
//!
//! XChaCha20-Poly1305 with a fresh random 24-byte nonce per write. The owning
//! row's `user_id` is passed as additional authenticated data, so a ciphertext
//! lifted from one row fails to open under another — a row swap at the database
//! level cannot hand one account's credentials to a different account.
//!
//! [`refresh_secret`]: crate::core::hypergryph::session::refresh_secret

use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use uuid::Uuid;

/// Nonce width for XChaCha20-Poly1305. Wide enough that random nonces stay
/// collision-free without a counter.
pub const NONCE_LEN: usize = 24;

const KEY_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("GAME_CREDENTIAL_KEY must be 32 bytes, as hex or base64")]
    BadKey,
    #[error("failed to seal credential")]
    Seal,
    #[error("failed to open credential")]
    Open,
}

/// An AEAD key for credential sealing, loaded once at startup.
pub struct CredentialKey(XChaCha20Poly1305);

// Deliberate: a derived Debug would print key material into logs.
impl std::fmt::Debug for CredentialKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CredentialKey(<redacted>)")
    }
}

impl CredentialKey {
    /// Parse a 32-byte key written as 64 hex characters or as base64.
    ///
    /// Generate one with `openssl rand -hex 32`. Rotating it makes every stored
    /// credential unopenable, which costs each user one re-login before resync
    /// works again; it corrupts nothing else.
    pub fn parse(raw: &str) -> Result<Self, CredentialError> {
        let raw = raw.trim();
        let bytes = hex::decode(raw)
            .ok()
            .or_else(|| STANDARD.decode(raw).ok())
            .ok_or(CredentialError::BadKey)?;

        let key: [u8; KEY_LEN] = bytes.try_into().map_err(|_| CredentialError::BadKey)?;
        Ok(Self(XChaCha20Poly1305::new(&key.into())))
    }

    /// Seal `plaintext` for `user_id`, returning `(nonce, ciphertext)`.
    pub fn seal(
        &self,
        user_id: Uuid,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), CredentialError> {
        let nonce_bytes: [u8; NONCE_LEN] = rand::random();

        let ciphertext = self
            .0
            .encrypt(
                &XNonce::from(nonce_bytes),
                Payload {
                    msg: plaintext,
                    aad: user_id.as_bytes(),
                },
            )
            .map_err(|_| CredentialError::Seal)?;

        Ok((nonce_bytes.to_vec(), ciphertext))
    }

    /// Open a `(nonce, ciphertext)` pair that was sealed for `user_id`.
    ///
    /// Fails on a wrong key, a tampered ciphertext, or a row whose `user_id` no
    /// longer matches the one it was sealed under.
    pub fn open(
        &self,
        user_id: Uuid,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CredentialError> {
        let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| CredentialError::Open)?;

        self.0
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: ciphertext,
                    aad: user_id.as_bytes(),
                },
            )
            .map_err(|_| CredentialError::Open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> CredentialKey {
        CredentialKey::parse(&"ab".repeat(KEY_LEN)).expect("valid hex key")
    }

    #[test]
    fn round_trips() {
        let k = key();
        let user = Uuid::new_v4();
        let (nonce, ct) = k.seal(user, b"durable-token").expect("seal");
        assert_eq!(k.open(user, &nonce, &ct).expect("open"), b"durable-token");
    }

    #[test]
    fn ciphertext_does_not_open_under_another_user() {
        let k = key();
        let (nonce, ct) = k.seal(Uuid::new_v4(), b"durable-token").expect("seal");
        assert!(k.open(Uuid::new_v4(), &nonce, &ct).is_err());
    }

    #[test]
    fn rejects_keys_that_are_not_32_bytes() {
        assert!(CredentialKey::parse("deadbeef").is_err());
        assert!(CredentialKey::parse("").is_err());
    }

    #[test]
    fn accepts_base64_and_hex_spellings_of_one_key() {
        let raw = [7u8; KEY_LEN];
        let from_hex = CredentialKey::parse(&hex::encode(raw)).expect("hex");
        let from_b64 = CredentialKey::parse(&STANDARD.encode(raw)).expect("base64");

        let user = Uuid::new_v4();
        let (nonce, ct) = from_hex.seal(user, b"same key").expect("seal");
        assert_eq!(from_b64.open(user, &nonce, &ct).expect("open"), b"same key");
    }
}
