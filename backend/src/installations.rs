//! Persistence for GitHub App installations linked to a user.

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::Installation;
use crate::util::now_unix;

pub struct NewInstallation<'a> {
    pub installation_id: i64,
    pub account_login: Option<&'a str>,
    pub account_type: Option<&'a str>,
    pub repository_selection: Option<&'a str>,
}

/// Links (or refreshes) an installation under `user_id`. Unique per (user, installation).
pub async fn upsert(
    db: &SqlitePool,
    user_id: &str,
    n: &NewInstallation<'_>,
) -> Result<Installation, AppError> {
    let existing = sqlx::query_as::<_, Installation>(
        "SELECT * FROM installations WHERE user_id = ? AND installation_id = ?",
    )
    .bind(user_id)
    .bind(n.installation_id)
    .fetch_optional(db)
    .await?;

    if let Some(mut row) = existing {
        sqlx::query(
            "UPDATE installations SET account_login = ?, account_type = ?, repository_selection = ? WHERE id = ?",
        )
        .bind(n.account_login)
        .bind(n.account_type)
        .bind(n.repository_selection)
        .bind(&row.id)
        .execute(db)
        .await?;
        row.account_login = n.account_login.map(str::to_string);
        row.account_type = n.account_type.map(str::to_string);
        row.repository_selection = n.repository_selection.map(str::to_string);
        return Ok(row);
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    sqlx::query(
        "INSERT INTO installations \
         (id, user_id, installation_id, account_login, account_type, repository_selection, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(n.installation_id)
    .bind(n.account_login)
    .bind(n.account_type)
    .bind(n.repository_selection)
    .bind(now)
    .execute(db)
    .await?;

    Ok(Installation {
        id,
        user_id: user_id.to_string(),
        installation_id: n.installation_id,
        account_login: n.account_login.map(str::to_string),
        account_type: n.account_type.map(str::to_string),
        repository_selection: n.repository_selection.map(str::to_string),
        created_at: now,
    })
}

/// The user's most recent installation, if any.
pub async fn get_for_user(
    db: &SqlitePool,
    user_id: &str,
) -> Result<Option<Installation>, AppError> {
    let row = sqlx::query_as::<_, Installation>(
        "SELECT * FROM installations WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;
    Ok(row)
}
