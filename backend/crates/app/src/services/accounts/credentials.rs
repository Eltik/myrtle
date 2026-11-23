use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use uuid::Uuid;

pub const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CredentialError {
    #[error("the credential key must be 32 bytes, as hex or base64")]
    BadKey,
    #[error("failed to seal credential")]
    Seal,
    #[error("failed to open credential")]
    Open,
}

pub struct CredentialKey(XChaCha20Poly1305);

impl std::fmt::Debug for CredentialKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CredentialKey(<redacted>)")
    }
}

impl CredentialKey {
    pub fn parse(raw: &str) -> Result<Self, CredentialError> {
        let raw = raw.trim();
        let bytes = hex::decode(raw)
            .ok()
            .or_else(|| STANDARD.decode(raw).ok())
            .ok_or(CredentialError::BadKey)?;
        let key: [u8; KEY_LEN] = bytes.try_into().map_err(|_| CredentialError::BadKey)?;
        Ok(Self(XChaCha20Poly1305::new(&key.into())))
    }

    pub fn seal(
        &self,
        owner: Uuid,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), CredentialError> {
        let nonce: [u8; NONCE_LEN] = rand::random();
        let ciphertext = self
            .0
            .encrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: plaintext,
                    aad: owner.as_bytes(),
                },
            )
            .map_err(|_| CredentialError::Seal)?;
        Ok((nonce.to_vec(), ciphertext))
    }

    pub fn open(
        &self,
        owner: Uuid,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CredentialError> {
        let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| CredentialError::Open)?;
        self.0
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: ciphertext,
                    aad: owner.as_bytes(),
                },
            )
            .map_err(|_| CredentialError::Open)
    }
}
