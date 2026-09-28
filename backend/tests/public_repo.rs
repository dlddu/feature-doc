mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{real_state, stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::github_app::{self, PublicRepo};
use featuredoc::state::AppState;
use featuredoc::{build_router, github_tokens, installations, session, users};

const PUBLIC: &str = "github.com/stub-public/oss-lib";

async fn login(state: &AppState, id: i64, login: &str, installed: bool) -> (String, String) {
    let gh = GithubUser {
        id,
        login: login.into(),
        name: None,
        avatar_url: None,
    };
    let user = users::upsert(&state.db, &gh).await.unwrap();
    if installed {
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
    }
    let session = session::create(&state.db, &user.id).await.unwrap();
    (user.id, session)
}

fn post_json(uri: &str, session: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={session}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn preflight(state: &AppState, session: &str, repo: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/analyses/preflight",
            session,
            serde_json::json!({ "repoUrl": repo }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

async fn create(state: &AppState, session: &str, repo: &str) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(post_json(
            "/api/analyses",
            session,
            serde_json::json!({ "repoUrl": repo }),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn queued(state: &AppState, user_id: &str) -> i64 {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM analyses WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    row.0
}

#[tokio::test]
async fn a_public_repository_outside_the_installation_is_estimated_and_queued() {
    let (state, path) = stub_state().await;
    let (user_id, s) = login(&state, 1, "pub-alice", true).await;

    let est = preflight(&state, &s, PUBLIC).await;
    assert_eq!(est["hasAccess"], true);
    assert_eq!(est["publicRepo"], true);
    assert_eq!(est["authExpired"], false);
    assert_eq!(est["fullName"], "stub-public/oss-lib");
    assert_eq!(est["branch"], "main");
    assert!(est["estLlmCalls"].as_i64().unwrap() > 0);
    assert!(est["estCostCents"].as_i64().unwrap() > 0);

    let (status, body) = create(&state, &s, PUBLIC).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "queued");
    assert_eq!(body["repoOwner"], "stub-public");
    assert_eq!(body["estLlmCalls"], est["estLlmCalls"]);

    let row: (i64,) = sqlx::query_as("SELECT installation_id FROM analyses WHERE user_id = ?")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(row.0, 4242);

    let granted = preflight(&state, &s, "stub-account/payments-api").await;
    assert_eq!(granted["hasAccess"], true);
    assert_eq!(granted["publicRepo"], false);

    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/analyses")
                .header(header::COOKIE, format!("fd_session={s}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let list = json_body(resp).await;
    assert_eq!(list[0]["repoName"], "oss-lib");
    assert_eq!(list[0]["publicRepo"], true);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_private_repository_outside_the_installation_is_still_refused() {
    let (state, path) = stub_state().await;
    let (user_id, s) = login(&state, 2, "pub-bob", true).await;

    let est = preflight(&state, &s, "github.com/someone-else/private-repo").await;
    assert_eq!(est["hasAccess"], false);
    assert_eq!(est["publicRepo"], false);
    assert_eq!(est["authExpired"], false);

    let (status, body) = create(&state, &s, "github.com/someone-else/private-repo").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("이 저장소에 접근할 수 없습니다"));
    assert_eq!(queued(&state, &user_id).await, 0);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_revoked_authorization_is_not_queued_and_asks_for_sign_in_again() {
    let (state, path) = stub_state().await;
    let (user_id, s) = login(&state, 3, "pub-revoked", true).await;
    github_tokens::store(&state.db, &state.config.kek, &user_id, "gho_stale_secret")
        .await
        .unwrap();
    std::env::set_var("FEATUREDOC_STUB_USER_AUTH_REVOKED", "pub-revoked");

    let est = preflight(&state, &s, PUBLIC).await;
    let (status, body) = create(&state, &s, PUBLIC).await;
    let granted = preflight(&state, &s, "stub-account/payments-api").await;
    std::env::remove_var("FEATUREDOC_STUB_USER_AUTH_REVOKED");

    assert_eq!(est["hasAccess"], false);
    assert_eq!(est["authExpired"], true);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let message = body["error"].as_str().unwrap();
    assert!(message.contains("다시 로그인하면"));
    assert!(!message.contains("gho_stale_secret"));
    assert_eq!(queued(&state, &user_id).await, 0);
    assert_eq!(granted["hasAccess"], true);
    assert_eq!(granted["authExpired"], false);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn without_the_app_installed_a_public_repository_gets_the_install_guidance() {
    let (state, path) = stub_state().await;
    let (user_id, s) = login(&state, 4, "pub-carol", false).await;

    let est = preflight(&state, &s, PUBLIC).await;
    assert_eq!(est["hasAccess"], false);
    assert_eq!(est["publicRepo"], false);

    let (status, body) = create(&state, &s, PUBLIC).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("GitHub App이 아직 설치되지 않았습니다"));
    assert_eq!(queued(&state, &user_id).await, 0);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn the_worker_reads_a_public_repository_with_the_user_authorization() {
    let (state, path) = stub_state().await;
    let (user_id, s) = login(&state, 5, "pub-dave", true).await;
    github_tokens::store(&state.db, &state.config.kek, &user_id, "gho_user_grant")
        .await
        .unwrap();
    let (status, body) = create(&state, &s, PUBLIC).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().unwrap().to_string();

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
    let job = json_body(resp).await;
    assert_eq!(job["id"], id.as_str());
    assert_eq!(job["repoName"], "oss-lib");
    assert_eq!(job["installationToken"], "gho_user_grant");

    let row: (String, Option<String>) =
        sqlx::query_as("SELECT status, error FROM analyses WHERE id = ?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(row, ("running".to_string(), None));
    let _ = std::fs::remove_file(&path);
}

async fn fake_github() -> String {
    async fn repo(
        axum::extract::Path((owner, name)): axum::extract::Path<(String, String)>,
        headers: axum::http::HeaderMap,
    ) -> axum::response::Response {
        use axum::response::IntoResponse;
        let auth = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if auth != "Bearer gho_valid" {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        let private = match (owner.as_str(), name.as_str()) {
            ("octo", "public") => false,
            ("octo", "secret") => true,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };
        axum::Json(serde_json::json!({
            "name": name,
            "owner": { "login": owner },
            "default_branch": "trunk",
            "size": 3000,
            "private": private
        }))
        .into_response()
    }
    let app = axum::Router::new().route("/repos/{owner}/{name}", axum::routing::get(repo));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn the_real_lookup_reads_public_metadata_with_the_user_authorization() {
    let api = fake_github().await;
    let (state, path) = real_state(&api).await;
    let (user_id, _) = login(&state, 6, "pub-erin", true).await;

    assert!(matches!(
        github_app::lookup_public_repository(&state, &user_id, "octo", "public")
            .await
            .unwrap(),
        PublicRepo::AuthExpired
    ));

    github_tokens::store(&state.db, &state.config.kek, &user_id, "gho_valid")
        .await
        .unwrap();
    match github_app::lookup_public_repository(&state, &user_id, "octo", "public")
        .await
        .unwrap()
    {
        PublicRepo::Found(repo) => {
            assert_eq!(repo.full_name, "octo/public");
            assert_eq!(repo.default_branch, "trunk");
            assert_eq!(repo.size_kb, 3000);
        }
        _ => panic!("a public repository is found"),
    }
    assert!(matches!(
        github_app::lookup_public_repository(&state, &user_id, "octo", "secret")
            .await
            .unwrap(),
        PublicRepo::NotPublic
    ));
    assert!(matches!(
        github_app::lookup_public_repository(&state, &user_id, "octo", "missing")
            .await
            .unwrap(),
        PublicRepo::NotPublic
    ));

    github_tokens::store(&state.db, &state.config.kek, &user_id, "gho_revoked")
        .await
        .unwrap();
    assert!(matches!(
        github_app::lookup_public_repository(&state, &user_id, "octo", "public")
            .await
            .unwrap(),
        PublicRepo::AuthExpired
    ));
    let _ = std::fs::remove_file(&path);
}
