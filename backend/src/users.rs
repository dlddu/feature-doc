//! User persistence: upsert from a GitHub identity.

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::github_api::GithubUser;
use crate::models::User;
use crate::util::now_unix;

/// Inserts a new user or refreshes the profile of an existing one, keyed by `github_id`.
pub async fn upsert(db: &SqlitePool, gh: &GithubUser) -> Result<User, AppError> {
    let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE github_id = ?")
        .bind(gh.id)
        .fetch_optional(db)
        .await?;

    if let Some(mut user) = existing {
        sqlx::query("UPDATE users SET login = ?, name = ?, avatar_url = ? WHERE id = ?")
            .bind(&gh.login)
            .bind(&gh.name)
            .bind(&gh.avatar_url)
            .bind(&user.id)
            .execute(db)
            .await?;
        user.login = gh.login.clone();
        user.name = gh.name.clone();
        user.avatar_url = gh.avatar_url.clone();
        return Ok(user);
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    sqlx::query(
        "INSERT INTO users (id, github_id, login, name, avatar_url, created_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(gh.id)
    .bind(&gh.login)
    .bind(&gh.name)
    .bind(&gh.avatar_url)
    .bind(now)
    .execute(db)
    .await?;

    Ok(User {
        id,
        github_id: gh.id,
        login: gh.login.clone(),
        name: gh.name.clone(),
        avatar_url: gh.avatar_url.clone(),
        created_at: now,
    })
}
