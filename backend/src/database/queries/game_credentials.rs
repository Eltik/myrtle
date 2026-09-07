//! Durable Yostar credentials — the half of a game session that cannot be
//! re-derived without an email code.
//!
//! See `migrations/v012_game_credentials.sql` for why this is a table rather
//! than a cache entry, and [`crate::core::auth::credentials`] for the sealing.

use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::auth::credentials::CredentialKey;
use crate::core::hypergryph::constants::AuthSession;

/// The durable half of an `AuthSession`: exactly what `refresh_secret` needs to
/// mint a live session, and nothing else. The short-lived `secret` / `token`
/// pair is deliberately absent — it is re-derivable, so persisting it would
/// widen the blast radius of a database leak for no gain.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DurableCredential {
    pub yostar_uid: String,
    pub yostar_token: String,
}

impl DurableCredential {
    /// Extract the durable pair, or `None` when the session never carried one.
    fn from_session(session: &AuthSession) -> Option<Self> {
        if session.yostar_uid.is_empty() || session.yostar_token.is_empty() {
            return None;
        }
        Some(Self {
            yostar_uid: session.yostar_uid.to_string(),
            yostar_token: session.yostar_token.to_string(),
        })
    }
}

/// Persist (or refresh) a user's durable credentials, sealed.
///
/// A session with no durable pair is a no-op rather than an error: it means
/// this login route never produced one, and there is nothing to keep.
pub async fn store(
    pool: &PgPool,
    key: &CredentialKey,
    user_id: Uuid,
    session: &AuthSession,
) -> anyhow::Result<()> {
    let Some(credential) = DurableCredential::from_session(session) else {
        return Ok(());
    };

    let plaintext = serde_json::to_vec(&credential).context("serialize durable credential")?;
    let (nonce, ciphertext) = key.seal(user_id, &plaintext)?;

    sqlx::query(
        "INSERT INTO user_game_credentials (user_id, nonce, ciphertext)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE
           SET nonce = EXCLUDED.nonce,
               ciphertext = EXCLUDED.ciphertext,
               updated_at = NOW()",
    )
    .bind(user_id)
    .bind(nonce)
    .bind(ciphertext)
    .execute(pool)
    .await
    .context("store durable credential")?;

    Ok(())
}

/// Read a user's durable credentials back.
///
/// Returns `Ok(None)` both when no row exists and when the stored row cannot be
/// opened. A rotated `GAME_CREDENTIAL_KEY` leaves rows indistinguishable from
/// absent, and both mean the same thing to a caller: this user has to log in
/// again. The unopenable case is logged, so a botched rotation is visible.
pub async fn load(
    pool: &PgPool,
    key: &CredentialKey,
    user_id: Uuid,
) -> anyhow::Result<Option<DurableCredential>> {
    let row: Option<(Vec<u8>, Vec<u8>)> =
        sqlx::query_as("SELECT nonce, ciphertext FROM user_game_credentials WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .context("read durable credential")?;

    let Some((nonce, ciphertext)) = row else {
        return Ok(None);
    };

    match key.open(user_id, &nonce, &ciphertext) {
        Ok(plaintext) => Ok(serde_json::from_slice(&plaintext).ok()),
        Err(e) => {
            tracing::warn!(
                user_id = %user_id,
                error = %e,
                "stored game credential did not open; treating as absent"
            );
            Ok(None)
        }
    }
}

/// Forget a user's durable credentials. Returns whether a row was removed.
///
/// This is the whole of "disconnect": with the row gone we cannot reach the
/// account again until the user supplies a fresh email code.
pub async fn delete(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_game_credentials WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
