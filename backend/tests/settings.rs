mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

async fn login_installed(state: &AppState, github_id: i64, login: &str) -> String {
    let gh = GithubUser {
        id: github_id,
        login: login.into(),
        name: None,
        avatar_url: None,
    };
    let user = users::upsert(&state.db, &gh).await.unwrap();
    installations::upsert(
        &state.db,
        &user.id,
        &installations::NewInstallation {
            installation_id: 4242,
            account_login: Some("stub-account"),
            account_type: Some("User"),
            repository_selection: Some("selected"),
        },
    )
    .await
    .unwrap();
    session::create(&state.db, &user.id).await.unwrap()
}

fn user_req(method: &str, uri: &str, token: &str, body: Option<serde_json::Value>) -> Request<Body> {
    let b = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json");
    b.body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn language_of(state: &AppState, session: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_req("GET", "/api/settings", session, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await["llmLanguage"].as_str().unwrap().to_string()
}

async fn set_language(state: &AppState, session: &str, lang: &str) -> StatusCode {
    build_router(state.clone())
        .oneshot(user_req(
            "PUT",
            "/api/settings",
            session,
            Some(serde_json::json!({ "llmLanguage": lang })),
        ))
        .await
        .unwrap()
        .status()
}

async fn enqueue(state: &AppState, session: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(user_req(
            "POST",
            "/api/analyses",
            session,
            Some(serde_json::json!({ "repoUrl": "stub-account/payments-api" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await
}

async fn claim(state: &AppState) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal/analyses/claim")
                .header(header::AUTHORIZATION, format!("Bearer {WORKER_TOKEN}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"workerId":"w1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

#[tokio::test]
async fn settings_require_a_session() {
    let (state, _p) = stub_state().await;
    let resp = build_router(state.clone())
        .oneshot(Request::builder().uri("/api/settings").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_user_who_never_chose_reads_korean() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    assert_eq!(language_of(&state, &s).await, "ko");
}

#[tokio::test]
async fn the_choice_persists_and_only_offered_languages_are_accepted() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;

    assert_eq!(set_language(&state, &s, "en").await, StatusCode::OK);
    assert_eq!(language_of(&state, &s).await, "en");

    assert_eq!(set_language(&state, &s, "ja").await, StatusCode::BAD_REQUEST);
    assert_eq!(language_of(&state, &s).await, "en", "a refused value must not overwrite");
}

#[tokio::test]
async fn one_users_choice_does_not_reach_another() {
    let (state, _p) = stub_state().await;
    let alice = login_installed(&state, 1, "alice").await;
    let bob = login_installed(&state, 2, "bob").await;

    assert_eq!(set_language(&state, &alice, "en").await, StatusCode::OK);
    assert_eq!(language_of(&state, &bob).await, "ko");
}

/// The analysis is written in the language that was set when it was triggered —
/// changing the setting afterwards steers the next analysis, not this one.
#[tokio::test]
async fn an_analysis_keeps_the_language_it_was_triggered_with() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;

    assert_eq!(set_language(&state, &s, "en").await, StatusCode::OK);
    let first = enqueue(&state, &s).await;
    assert_eq!(first["llmLanguage"], "en");

    assert_eq!(set_language(&state, &s, "ko").await, StatusCode::OK);

    let job = claim(&state).await;
    assert_eq!(job["id"], first["id"]);
    assert_eq!(job["llmLanguage"], "en", "the worker reads the analysis, not the setting");

    let second = enqueue(&state, &s).await;
    assert_eq!(second["llmLanguage"], "ko");
}

/// A run triggered before the setting existed has no language on it, and the claim
/// says so rather than inventing one — the worker then leaves its prompt untouched.
#[tokio::test]
async fn a_run_without_a_language_is_claimed_without_one() {
    let (state, _p) = stub_state().await;
    let s = login_installed(&state, 1, "alice").await;
    let a = enqueue(&state, &s).await;
    sqlx::query("UPDATE analyses SET llm_language = NULL WHERE id = ?")
        .bind(a["id"].as_str().unwrap())
        .execute(&state.db)
        .await
        .unwrap();

    let job = claim(&state).await;
    assert!(job["llmLanguage"].is_null(), "{job}");
}
