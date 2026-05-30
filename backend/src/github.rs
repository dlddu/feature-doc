//! GitHub App installation surface: install URL, post-install setup callback, and
//! the connection status the S01 screen renders.

use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::auth::CurrentUser;
use crate::config::Mode;
use crate::error::AppError;
use crate::state::AppState;
use crate::{cookies, github_app, installations, util};

const SETUP_STATE_COOKIE: &str = "fd_setup_state";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/github/install-url", get(install_url))
        .route("/api/github/setup", get(setup))
        .route("/api/github/connection", get(connection))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallUrlView {
    url: String,
}

/// Returns where to send the user to install the App. Real mode points at GitHub's
/// installation page; stub mode points back at our own setup callback so the
/// browser e2e completes the loop without GitHub.
async fn install_url(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentUser(user): CurrentUser,
) -> Result<(CookieJar, Json<InstallUrlView>), AppError> {
    let nonce = util::random_token();
    let jar = jar.add(cookies::make(&state, SETUP_STATE_COOKIE, nonce.clone()));

    let url = match state.config.mode {
        Mode::Stub => {
            let iid = stub_installation_id(user.github_id);
            format!("/api/github/setup?installation_id={iid}&setup_action=install&state={nonce}")
        }
        Mode::Real => {
            let mut u = url::Url::parse(&format!(
                "{}/apps/{}/installations/new",
                state.config.github.web_base, state.config.github.app_slug
            ))
            .map_err(|_| AppError::internal("invalid web_base"))?;
            u.query_pairs_mut().append_pair("state", &nonce);
            u.to_string()
        }
    };

    Ok((jar, Json(InstallUrlView { url })))
}

#[derive(Deserialize)]
struct SetupParams {
    installation_id: i64,
    #[serde(default)]
    state: Option<String>,
}

/// GitHub App "Setup URL" callback after the user installs/selects repositories.
/// Links the installation to the current user, then returns to the SPA.
async fn setup(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentUser(user): CurrentUser,
    Query(params): Query<SetupParams>,
) -> Result<(CookieJar, Redirect), AppError> {
    // If we issued a setup-state cookie, require the echoed value to match.
    if let Some(expected) = jar.get(SETUP_STATE_COOKIE).map(|c| c.value().to_string()) {
        if params.state.as_deref() != Some(expected.as_str()) {
            return Err(AppError::BadRequest("invalid setup state".into()));
        }
    }

    let info = github_app::fetch_installation(&state, params.installation_id).await?;
    installations::upsert(
        &state.db,
        &user.id,
        &installations::NewInstallation {
            installation_id: params.installation_id,
            account_login: info.account_login.as_deref(),
            account_type: info.account_type.as_deref(),
            repository_selection: info.repository_selection.as_deref(),
        },
    )
    .await?;

    let jar = jar.remove(cookies::removal(SETUP_STATE_COOKIE));
    Ok((jar, Redirect::to("/")))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountView {
    login: Option<String>,
    account_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionView {
    installed: bool,
    account: Option<AccountView>,
    repository_selection: Option<String>,
    repository_count: Option<i64>,
    /// The read-only scopes the App requests — always present so the screen can
    /// show them before installation too.
    permissions: Vec<String>,
}

/// Reports the current user's installation state for the S01 screen.
async fn connection(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<ConnectionView>, AppError> {
    let permissions = github_app::REQUESTED_PERMISSIONS
        .iter()
        .map(|s| s.to_string())
        .collect();

    match installations::get_for_user(&state.db, &user.id).await? {
        None => Ok(Json(ConnectionView {
            installed: false,
            account: None,
            repository_selection: None,
            repository_count: None,
            permissions,
        })),
        Some(inst) => {
            let repository_count =
                github_app::repository_count(&state, inst.installation_id).await;
            Ok(Json(ConnectionView {
                installed: true,
                account: Some(AccountView {
                    login: inst.account_login,
                    account_type: inst.account_type,
                }),
                repository_selection: inst.repository_selection,
                repository_count,
                permissions,
            }))
        }
    }
}

/// Deterministic per-user stub installation id (distinct users → distinct ids).
fn stub_installation_id(github_id: i64) -> i64 {
    10_000 + github_id.rem_euclid(90_000)
}
