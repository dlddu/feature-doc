//! Opaque server-side sessions keyed by a random cookie value.

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::User;
use crate::util::{now_unix, random_token};

/// Name of the session cookie. HttpOnly; the value is an opaque random token.
pub const SESSION_COOKIE: &str = "fd_session";

const SESSION_TTL_SECS: i64 = 60 * 60 * 24 * 30; // 30 days

/// Creates a new session for `user_id`, returning the opaque token to set as a cookie.
pub async fn create(db: &SqlitePool, user_id: &str) -> Result<String, AppError> {
    let token = random_token();
    let now = now_unix();
    sqlx::query("INSERT INTO sessions (id, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)")
        .bind(&token)
        .bind(user_id)
        .bind(now)
        .bind(now + SESSION_TTL_SECS)
        .execute(db)
        .await?;
    Ok(token)
}

/// Resolves a (non-expired) session token to its owning user, if any.
pub async fn lookup_user(db: &SqlitePool, token: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT u.id, u.github_id, u.login, u.name, u.avatar_url, u.created_at \
         FROM users u JOIN sessions s ON s.user_id = u.id \
         WHERE s.id = ? AND s.expires_at > ?",
    )
    .bind(token)
    .bind(now_unix())
    .fetch_optional(db)
    .await?;
    Ok(user)
}

/// Deletes a session (logout).
pub async fn delete(db: &SqlitePool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(token)
        .execute(db)
        .await?;
    Ok(())
}
