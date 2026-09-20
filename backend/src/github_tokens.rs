//! The user's GitHub OAuth token, kept for one purpose only: confirming at setup
//! time that they really can reach the installation they are linking.

use sqlx::SqlitePool;

use crate::crypto::{self, Envelope};
use crate::error::AppError;
use crate::util::now_unix;

pub async fn store(
    db: &SqlitePool,
    kek: &[u8; 32],
    user_id: &str,
    token: &str,
) -> Result<(), AppError> {
    let env = crypto::seal(kek, token.as_bytes())?;
    sqlx::query(
        "INSERT INTO github_tokens (user_id, ciphertext, nonce, wrapped_dek, dek_nonce, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?) \
         ON CONFLICT(user_id) DO UPDATE SET \
           ciphertext = excluded.ciphertext, nonce = excluded.nonce, \
           wrapped_dek = excluded.wrapped_dek, dek_nonce = excluded.dek_nonce, \
           updated_at = excluded.updated_at",
    )
    .bind(user_id)
    .bind(env.ciphertext.as_slice())
    .bind(env.nonce.as_slice())
    .bind(env.wrapped_dek.as_slice())
    .bind(env.dek_nonce.as_slice())
    .bind(now_unix())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn load(
    db: &SqlitePool,
    kek: &[u8; 32],
    user_id: &str,
) -> Result<Option<String>, AppError> {
    let row = sqlx::query_as::<_, (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)>(
        "SELECT ciphertext, nonce, wrapped_dek, dek_nonce FROM github_tokens WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    let Some((ciphertext, nonce, wrapped_dek, dek_nonce)) = row else {
        return Ok(None);
    };
    let plaintext = crypto::open(
        kek,
        &Envelope {
            ciphertext,
            nonce,
            wrapped_dek,
            dek_nonce,
        },
    )?;
    let token =
        String::from_utf8(plaintext).map_err(|_| AppError::internal("github token: invalid utf8"))?;
    Ok(Some(token))
}
