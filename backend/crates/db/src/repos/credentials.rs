use myrtle_types::UserId;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct SealedCredentialRow {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub async fn store(
    conn: &mut Conn,
    user: UserId,
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO user_game_credentials (user_id, nonce, ciphertext)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE
           SET nonce = EXCLUDED.nonce,
               ciphertext = EXCLUDED.ciphertext,
               updated_at = NOW()",
        user.0,
        nonce,
        ciphertext,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn load(conn: &mut Conn, user: UserId) -> Result<Option<SealedCredentialRow>, DbError> {
    Ok(sqlx::query_as!(
        SealedCredentialRow,
        "SELECT nonce, ciphertext FROM user_game_credentials WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn delete(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM user_game_credentials WHERE user_id = $1",
        user.0,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}
