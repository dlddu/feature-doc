//! Per-user preferences that shape what the pipeline produces rather than who may
//! run it. Today that is one value: the language LLM-written prose comes back in.
//!
//! The setting is read once, when an analysis is triggered, and copied onto that
//! analysis (`analysis::create`). Changing it therefore steers the *next* analysis
//! and never rewrites one already under way — see the worker's `language_for`.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::llm::Language;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/settings", get(read).put(update))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    /// `"ko"` or `"en"` — the codes [`Language::parse`] accepts.
    pub llm_language: String,
}

/// The user's output language. A value the code no longer recognises reads as the
/// default instead of failing the screen: the row is the user's, the vocabulary is
/// the code's, and the code is the one that moved.
pub async fn llm_language(db: &SqlitePool, user_id: &str) -> Result<Language, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT llm_language FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(row
        .and_then(|(l,)| Language::parse(&l))
        .unwrap_or(crate::llm::DEFAULT_LANGUAGE))
}

/// The language one analysis was fixed to when it was triggered — what every LLM
/// call made *for* that analysis writes in, including the ones the API makes
/// directly (edit proposals, manual feature drafts), so their prose matches the
/// document it lands in. `None` for an analysis triggered before the setting
/// existed: the caller then adds no instruction.
pub async fn analysis_language(
    db: &SqlitePool,
    analysis_id: &str,
) -> Result<Option<Language>, AppError> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT llm_language FROM analyses WHERE id = ?")
            .bind(analysis_id)
            .fetch_optional(db)
            .await?;
    Ok(row.and_then(|(l,)| l).as_deref().and_then(Language::parse))
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<SettingsView>, AppError> {
    let lang = llm_language(&state.db, &user.id).await?;
    Ok(Json(SettingsView {
        llm_language: lang.as_str().to_string(),
    }))
}

async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<SettingsView>,
) -> Result<Json<SettingsView>, AppError> {
    let lang = Language::parse(&req.llm_language)
        .ok_or_else(|| AppError::BadRequest("지원하지 않는 출력 언어입니다 — ko 또는 en".into()))?;
    sqlx::query("UPDATE users SET llm_language = ? WHERE id = ?")
        .bind(lang.as_str())
        .bind(&user.id)
        .execute(&state.db)
        .await?;
    Ok(Json(SettingsView {
        llm_language: lang.as_str().to_string(),
    }))
}
