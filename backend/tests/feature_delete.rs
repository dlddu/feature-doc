//! feature 문서의 삭제와 보존(AC3.3) — 라우터를 그대로 돌려 본다.
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const WORKER: &str = "w-feature-delete";
const FEATURE: &str = "payments-api/src/api/routes.rs";
const WHY: &str = "결제 수단 등록은 이 제품의 기능이 아니라 결제 대행사 화면이다";

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

fn get(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .body(Body::empty())
        .unwrap()
}

fn user_post(uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json")
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

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn analysis(state: &AppState, session: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            "/api/analyses",
            session,
            json!({ "repoUrl": "stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn analysis_with_document(state: &AppState, session: &str) -> String {
    let id = analysis(state, session).await;

    let claimed = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(claimed.status(), StatusCode::OK, "queue should hold a job");

    let submitted = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/acceptance_dependencies"),
            json!({
                "workerId": WORKER,
                "model": "stub-model",
                "content": {
                    "features": [{
                        "key": FEATURE,
                        "name": "결제 수단 등록",
                        "location": FEATURE,
                        "symbol": "pay",
                        "scenarios": [{
                            "given": "카드가 없는 사용자가",
                            "when": "만료된 카드를 저장하면",
                            "then": "저장되지 않습니다",
                            "evidence": FEATURE,
                            "symbol": "pay",
                            "source": "logic",
                        }],
                        "contradictions": [],
                    }],
                },
            }),
        ))
        .await
        .unwrap();
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT, "document submit");
    id
}

async fn features(state: &AppState, session: &str, id: &str) -> Option<Vec<serde_json::Value>> {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/documents/acceptance-dependencies"),
            session,
        ))
        .await
        .unwrap();
    if resp.status() == StatusCode::NOT_FOUND {
        return None;
    }
    assert_eq!(resp.status(), StatusCode::OK);
    Some(
        json_body(resp).await["content"]["features"]
            .as_array()
            .unwrap()
            .clone(),
    )
}


async fn delete(
    state: &AppState,
    session: &str,
    id: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(&format!("/api/analyses/{id}/features/deletions"), session, body))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn restore(
    state: &AppState,
    session: &str,
    id: &str,
    deletion: &str,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/deletions/{deletion}/restore"),
            session,
            json!({}),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn archive(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/features/deletions"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

async fn candidates(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/candidates"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

#[tokio::test]
async fn deleting_moves_the_feature_to_the_archive_and_restoring_brings_it_back() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9501, "remover").await;
    let id = analysis_with_document(&state, &s).await;
    assert_eq!(features(&state, &s, &id).await.unwrap().len(), 1);

    let (status, deleted) = delete(&state, &s, &id, json!({ "key": FEATURE, "reason": WHY })).await;
    assert_eq!(status, StatusCode::OK, "{deleted}");
    assert_eq!(deleted["key"], FEATURE);
    assert_eq!(deleted["name"], "결제 수단 등록");
    assert_eq!(deleted["reason"], WHY);
    assert_eq!(deleted["restorable"], true);
    assert!(deleted["restoredAt"].is_null());
    let deleted_at = deleted["deletedAt"].as_i64().unwrap();
    let until = deleted["restoreUntil"].as_i64().unwrap();
    assert_eq!(
        until - deleted_at,
        featuredoc::feature_delete::RETENTION_DAYS * 24 * 60 * 60,
        "되돌릴 수 있는 기한은 지운 시점의 사실이다"
    );

    let after = features(&state, &s, &id).await;
    assert_eq!(after, Some(vec![]), "지운 feature 가 문서에서 가려지지 않았다");
    let doc: (String,) = sqlx::query_as(
        "SELECT content FROM analysis_documents WHERE analysis_id = ? AND kind = 'acceptance_dependencies'",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert!(doc.0.contains(FEATURE), "자동 문서를 고쳤다 — 재현성의 기준값이 움직인다");

    let listed = archive(&state, &s, &id).await;
    assert_eq!(listed["retentionDays"], featuredoc::feature_delete::RETENTION_DAYS);
    assert_eq!(listed["deletions"].as_array().unwrap().len(), 1);
    assert_eq!(listed["deletions"][0]["id"], deleted["id"]);

    let (status, _) = delete(&state, &s, &id, json!({ "key": FEATURE })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, restored) = restore(&state, &s, &id, deleted["id"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{restored}");
    assert!(restored["restoredAt"].is_i64());
    assert_eq!(restored["restorable"], false);
    assert_eq!(features(&state, &s, &id).await.unwrap().len(), 1, "복구했는데 문서에 돌아오지 않았다");
    assert!(archive(&state, &s, &id).await["deletions"].as_array().unwrap().is_empty());

    let (status, _) = restore(&state, &s, &id, deleted["id"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::CONFLICT, "두 번 복구");

    let (status, again) = delete(&state, &s, &id, json!({ "key": FEATURE })).await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(again["id"], deleted["id"]);
    assert!(again["reason"].is_null(), "사유는 선택이다");
    let rows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM feature_deletions WHERE analysis_id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(rows.0, 2, "되돌린 행을 지웠다");
}

#[tokio::test]
async fn a_deleted_feature_rediscovered_by_the_next_analysis_is_marked_not_reactivated() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9502, "rediscoverer").await;
    let first = analysis_with_document(&state, &s).await;
    let (status, deleted) = delete(&state, &s, &first, json!({ "key": FEATURE, "reason": WHY })).await;
    assert_eq!(status, StatusCode::OK);

    let second = analysis(&state, &s).await;
    let claimed = build_router(state.clone())
        .oneshot(worker_post("/internal/analyses/claim", json!({ "workerId": WORKER })))
        .await
        .unwrap();
    assert_eq!(claimed.status(), StatusCode::OK);
    let submitted = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{second}/documents/feature_candidates"),
            json!({
                "workerId": WORKER,
                "model": "stub-model",
                "content": { "candidates": [
                    { "name": "결제 수단 등록", "location": FEATURE, "rationale": "stub" },
                    { "name": "다른 기능", "location": "payments-api/src/api/other.rs", "rationale": "stub" },
                ]},
            }),
        ))
        .await
        .unwrap();
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT);

    let listed = candidates(&state, &s, &second).await;
    let items = listed["candidates"].as_array().unwrap();
    let carried = items
        .iter()
        .find(|c| c["key"] == FEATURE)
        .expect("같은 자리의 후보");
    assert_eq!(carried["decision"], "undecided", "사용자 확인 없이 다시 활성화됐다");
    assert_eq!(carried["previouslyDeleted"]["reason"], WHY);
    assert_eq!(carried["previouslyDeleted"]["analysisId"], first);
    assert!(carried["previouslyRejected"].is_null());
    let other = items.iter().find(|c| c["location"] == "payments-api/src/api/other.rs").unwrap();
    assert!(other["previouslyDeleted"].is_null(), "지운 적 없는 자리에 표시가 붙었다");
    assert_eq!(listed["undecided"], 2);

    let (status, _) = restore(&state, &s, &first, deleted["id"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    let listed = candidates(&state, &s, &second).await;
    let carried = listed["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["key"] == FEATURE)
        .unwrap();
    assert!(carried["previouslyDeleted"].is_null());
}

#[tokio::test]
async fn another_users_analysis_is_out_of_reach() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 9503, "owner").await;
    let stranger = login_installed(&state, 9504, "stranger").await;
    let id = analysis_with_document(&state, &owner).await;

    let (status, _) = delete(&state, &stranger, &id, json!({ "key": FEATURE })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/features/deletions"), &stranger))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let (status, deleted) = delete(&state, &owner, &id, json!({ "key": FEATURE })).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = restore(&state, &stranger, &id, deleted["id"].as_str().unwrap()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(features(&state, &owner, &id).await, Some(vec![]));

    let (status, _) = delete(&state, &owner, &id, json!({ "key": "nope/never.rs" })).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "문서에 없는 feature");
    let (status, _) = delete(&state, &owner, &id, json!({ "key": "  " })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
