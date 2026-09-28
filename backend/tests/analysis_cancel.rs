mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const ABSENT_ID: &str = "00000000-0000-4000-8000-000000000000";

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

fn user_post(uri: &str, token: Option<&str>, body: serde_json::Value) -> Request<Body> {
    let b = Request::builder().method("POST").uri(uri);
    let b = match token {
        Some(t) => b.header(header::COOKIE, format!("fd_session={t}")),
        None => b,
    };
    b.header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn worker_post(uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {WORKER_TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn body_bytes(resp: axum::response::Response) -> Vec<u8> {
    resp.into_body().collect().await.unwrap().to_bytes().to_vec()
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    serde_json::from_slice(&body_bytes(resp).await).unwrap()
}

async fn enqueue(state: &AppState, session: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            "/api/analyses",
            Some(session),
            serde_json::json!({ "repoUrl": "stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn stop(state: &AppState, token: Option<&str>, id: &str) -> (StatusCode, Vec<u8>) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/cancel"),
            token,
            serde_json::json!({}),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, body_bytes(resp).await)
}

async fn claim(state: &AppState, worker_id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            serde_json::json!({ "workerId": worker_id }),
        ))
        .await
        .unwrap()
}

async fn heartbeat(state: &AppState, id: &str, worker_id: &str) -> StatusCode {
    build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/heartbeat"),
            serde_json::json!({ "workerId": worker_id }),
        ))
        .await
        .unwrap()
        .status()
}

async fn report_stage(state: &AppState, id: &str, worker_id: &str, status: &str) -> StatusCode {
    build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/fetch"),
            serde_json::json!({ "workerId": worker_id, "status": status }),
        ))
        .await
        .unwrap()
        .status()
}

async fn finish(state: &AppState, id: &str, worker_id: &str) -> StatusCode {
    build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            serde_json::json!({ "workerId": worker_id, "status": "failed" }),
        ))
        .await
        .unwrap()
        .status()
}

async fn analysis_row(state: &AppState, id: &str) -> (String, Option<i64>, Option<String>) {
    sqlx::query_as::<_, (String, Option<i64>, Option<String>)>(
        "SELECT status, finished_at, claimed_by FROM analyses WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .unwrap()
}

async fn stage_status(state: &AppState, id: &str) -> String {
    sqlx::query_scalar::<_, String>(
        "SELECT status FROM analysis_stages WHERE analysis_id = ? AND key = ?",
    )
    .bind(id)
    .bind("fetch")
    .fetch_one(&state.db)
    .await
    .unwrap()
}

#[tokio::test]
async fn a_stopped_analysis_is_never_claimed_again() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "g5a").await;
    let id = enqueue(&state, &owner).await;

    let (status, body) = stop(&state, Some(&owner), &id).await;
    assert_eq!(status, StatusCode::OK);
    let view: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(view["status"], "cancelled");
    assert!(view["finishedAt"].is_i64(), "종료 시각이 없다: {view}");

    let (db_status, finished_at, claimed_by) = analysis_row(&state, &id).await;
    assert_eq!(db_status, "cancelled");
    assert!(finished_at.is_some());
    assert!(claimed_by.is_none());

    assert_eq!(
        claim(&state, "w1").await.status(),
        StatusCode::NO_CONTENT,
        "중단된 분석이 다시 집혔다 — 비용이 계속 난다"
    );

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn stopping_a_running_analysis_closes_every_paid_path() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "g5b").await;
    let id = enqueue(&state, &owner).await;

    assert_eq!(claim(&state, "w1").await.status(), StatusCode::OK);
    assert_eq!(
        report_stage(&state, &id, "w1", "running").await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(heartbeat(&state, &id, "w1").await, StatusCode::NO_CONTENT);

    assert_eq!(stop(&state, Some(&owner), &id).await.0, StatusCode::OK);

    assert_eq!(
        heartbeat(&state, &id, "w1").await,
        StatusCode::CONFLICT,
        "워커가 유료 호출 앞의 heartbeat 를 통과했다"
    );
    assert_eq!(
        report_stage(&state, &id, "w1", "succeeded").await,
        StatusCode::CONFLICT
    );
    assert_eq!(finish(&state, &id, "w1").await, StatusCode::CONFLICT);

    let (db_status, _, _) = analysis_row(&state, &id).await;
    assert_eq!(db_status, "cancelled", "워커가 중단을 덮어썼다");
    assert_eq!(
        stage_status(&state, &id).await,
        "pending",
        "멈춘 단계가 영원히 진행 중으로 남는다"
    );

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_stopped_analysis_cannot_be_stopped_again() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "g5c").await;
    let id = enqueue(&state, &owner).await;

    assert_eq!(stop(&state, Some(&owner), &id).await.0, StatusCode::OK);
    assert_eq!(
        stop(&state, Some(&owner), &id).await.0,
        StatusCode::CONFLICT
    );

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_strangers_analysis_is_indistinguishable_from_one_that_does_not_exist() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "g5d").await;
    let stranger = login_installed(&state, 2, "g5e").await;
    let id = enqueue(&state, &owner).await;

    let real = stop(&state, Some(&stranger), &id).await;
    let absent = stop(&state, Some(&stranger), ABSENT_ID).await;
    assert_eq!(real.0, StatusCode::NOT_FOUND);
    assert_eq!(real, absent, "남의 대상과 없는 대상의 응답이 갈린다");

    let (db_status, _, _) = analysis_row(&state, &id).await;
    assert_eq!(db_status, "queued", "남이 내 분석을 멈췄다");

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn an_unauthenticated_caller_cannot_stop() {
    let (state, path) = stub_state().await;
    let owner = login_installed(&state, 1, "g5f").await;
    let id = enqueue(&state, &owner).await;

    assert_eq!(stop(&state, None, &id).await.0, StatusCode::UNAUTHORIZED);
    let (db_status, _, _) = analysis_row(&state, &id).await;
    assert_eq!(db_status, "queued");

    state.db.close().await;
    let _ = std::fs::remove_file(&path);
}
