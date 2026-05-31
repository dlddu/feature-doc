//! S02 (Repositories home): list the user's connected repositories and connect a
//! new one. Connect verifies the repository is within the user's GitHub App
//! installation scope (AC1.1: "접근 범위 밖"이면 거부) before linking.
//!
//! The analysis-derived fields (status/featureCount/conflictCount/spendCents/
//! progress/step/lastAnalyzedAt) are reserved but not yet produced: connect leaves
//! them NULL / 'not_analyzed', so the screen renders the empty state until the
//! analysis pipeline lands.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::models::Repository;
use crate::state::AppState;
use crate::util::now_unix;
use crate::{github_app, installations};

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/repositories", get(list).post(connect))
}

/// The camelCase view the S02 screen renders. Analysis fields are nullable and stay
/// `null` until the pipeline fills them.
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
struct RepositoryView {
    id: String,
    owner: String,
    name: String,
    branch: String,
    status: String,
    feature_count: Option<i64>,
    conflict_count: Option<i64>,
    spend_cents: Option<i64>,
    progress: Option<i64>,
    step: Option<String>,
    last_analyzed_at: Option<i64>,
    created_at: i64,
}

impl From<Repository> for RepositoryView {
    fn from(r: Repository) -> Self {
        Self {
            id: r.id,
            owner: r.owner,
            name: r.name,
            branch: r.branch,
            status: r.status,
            feature_count: r.feature_count,
            conflict_count: r.conflict_count,
            spend_cents: r.spend_cents,
            progress: r.progress,
            step: r.step,
            last_analyzed_at: r.last_analyzed_at,
            created_at: r.created_at,
        }
    }
}

/// Lists the current user's connected repositories (AC4.7: scoped to the owner).
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<RepositoryView>>, AppError> {
    let rows = sqlx::query_as::<_, RepositoryView>(
        "SELECT id, owner, name, branch, status, feature_count, conflict_count, \
         spend_cents, progress, step, last_analyzed_at, created_at \
         FROM repositories WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(RepositoryView::from).collect()))
}

#[derive(Deserialize)]
struct ConnectReq {
    owner: String,
    name: String,
    #[serde(default)]
    branch: Option<String>,
}

/// Connects a repository: confirm the user has a GitHub App installation, confirm
/// the requested repo is within that installation's access scope, then upsert it.
/// A repo outside the scope is `Forbidden` (AC1.1). Returns the connected row (201).
async fn connect(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<ConnectReq>,
) -> Result<(StatusCode, Json<RepositoryView>), AppError> {
    let owner = req.owner.trim();
    let name = req.name.trim();
    if owner.is_empty() || name.is_empty() {
        return Err(AppError::BadRequest("owner와 name이 필요합니다".into()));
    }

    let installation = installations::get_for_user(&state.db, &user.id)
        .await?
        .ok_or(AppError::Forbidden)?;

    // The repo must be within the installation's access scope (AC1.1).
    let accessible = github_app::list_repositories(&state, installation.installation_id).await?;
    let matched = accessible
        .into_iter()
        .find(|r| r.owner == owner && r.name == name)
        .ok_or(AppError::Forbidden)?;

    let branch = req
        .branch
        .as_deref()
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .unwrap_or(&matched.default_branch);

    let row = upsert(&state.db, &user.id, owner, name, branch).await?;
    crate::audit::record(
        &state.db,
        Some(&user.id),
        "repository.connect",
        Some(&format!("{owner}/{name}")),
    )
    .await;

    Ok((StatusCode::CREATED, Json(RepositoryView::from(row))))
}

/// Links (or refreshes the branch of) a repository under `user_id`. Unique per
/// (user, owner, name). Analysis-derived columns are left at their reserved
/// defaults (NULL / 'not_analyzed'). Public so the `seed` binary can connect repos
/// directly without going through the HTTP handler.
pub async fn upsert(
    db: &sqlx::SqlitePool,
    user_id: &str,
    owner: &str,
    name: &str,
    branch: &str,
) -> Result<Repository, AppError> {
    let existing = sqlx::query_as::<_, Repository>(
        "SELECT * FROM repositories WHERE user_id = ? AND owner = ? AND name = ?",
    )
    .bind(user_id)
    .bind(owner)
    .bind(name)
    .fetch_optional(db)
    .await?;

    if let Some(mut row) = existing {
        sqlx::query("UPDATE repositories SET branch = ? WHERE id = ?")
            .bind(branch)
            .bind(&row.id)
            .execute(db)
            .await?;
        row.branch = branch.to_string();
        return Ok(row);
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    sqlx::query(
        "INSERT INTO repositories (id, user_id, owner, name, branch, created_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(owner)
    .bind(name)
    .bind(branch)
    .bind(now)
    .execute(db)
    .await?;

    Ok(Repository {
        id,
        user_id: user_id.to_string(),
        owner: owner.to_string(),
        name: name.to_string(),
        branch: branch.to_string(),
        status: "not_analyzed".to_string(),
        feature_count: None,
        conflict_count: None,
        spend_cents: None,
        progress: None,
        step: None,
        last_analyzed_at: None,
        created_at: now,
    })
}
