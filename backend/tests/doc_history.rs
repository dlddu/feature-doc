//! 변경 이력 조회와 임의 시점 복원 — 라우터를 그대로 돌려 본다.
mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const WORKER: &str = "w-doc-history";
const FEATURE: &str = "payments-api/src/api/routes.rs";
const ADD: &str = "에러 케이스 1개 더 추가";

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

fn user_post(uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::COOKIE, format!("fd_session={token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn worker_post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {WORKER_TOKEN}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn encoded(value: &str) -> String {
    value.replace('/', "%2F")
}

async fn analysis_with_document(state: &AppState, session: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            "/api/analyses",
            session,
            json!({ "repoUrl": "stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let claimed = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(claimed.status(), StatusCode::OK);

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
                        "location": "payments-api/src/api/routes.rs",
                        "symbol": "pay",
                        "scenarios": [{
                            "given": "카드가 없는 사용자가",
                            "when": "만료된 카드를 저장하면",
                            "then": "저장되지 않습니다",
                            "evidence": "payments-api/src/api/routes.rs",
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
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT);
    id
}

/// 제안 → 승인. 승인된 편집 id 를 돌려준다.
async fn approved_edit(state: &AppState, session: &str, id: &str, request: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals"),
            session,
            json!({ "key": FEATURE, "scenarioIndex": 0, "request": request }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "제안");
    let proposal = json_body(resp).await["id"].as_str().unwrap().to_string();

    let decided = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals/{proposal}/decision"),
            session,
            json!({ "decision": "approve" }),
        ))
        .await
        .unwrap();
    assert_eq!(decided.status(), StatusCode::OK, "승인");
    proposal
}

async fn history(state: &AppState, session: &str, id: &str) -> Value {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/features/{}/history", encoded(FEATURE)),
            session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "이력");
    json_body(resp).await
}

async fn preview(state: &AppState, session: &str, id: &str, entry: &str) -> (StatusCode, Value) {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/analyses/{id}/features/{}/history/{entry}",
                encoded(FEATURE)
            ),
            session,
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn restore(state: &AppState, session: &str, id: &str, entry: &str) -> (StatusCode, Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!(
                "/api/analyses/{id}/features/{}/history/{entry}/restore",
                encoded(FEATURE)
            ),
            session,
            json!({}),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

fn thens(view: &Value) -> Vec<String> {
    view["scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["then"].as_str().unwrap().to_string())
        .collect()
}

fn entries(view: &Value) -> &Vec<Value> {
    view["entries"].as_array().unwrap()
}

#[tokio::test]
async fn history_starts_at_the_automatic_baseline_and_names_the_source_of_every_change() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9401, "reader").await;
    let id = analysis_with_document(&state, &s).await;

    let first = history(&state, &s, &id).await;
    assert_eq!(entries(&first).len(), 1, "편집 전에는 기준선 하나뿐이다");
    assert_eq!(entries(&first)[0]["kind"], "auto");
    assert_eq!(entries(&first)[0]["source"], "auto");
    assert_eq!(entries(&first)[0]["current"], true);

    approved_edit(&state, &s, &id, ADD).await;
    let after = history(&state, &s, &id).await;
    assert_eq!(entries(&after).len(), 2);
    assert_eq!(entries(&after)[1]["kind"], "edit");
    assert_eq!(
        entries(&after)[1]["source"], "user_llm",
        "LLM 보조 수정의 출처가 다른 값으로 적혔다"
    );
    assert_eq!(entries(&after)[1]["request"], ADD);
    assert_eq!(entries(&after)[1]["current"], true);
    assert_eq!(entries(&after)[1]["standing"], true);
    assert_eq!(thens(&after).len(), 2, "승인된 편집이 문서에 서지 않았다");
}

#[tokio::test]
async fn restoring_a_point_puts_the_document_back_and_is_itself_a_change() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9402, "restorer").await;
    let id = analysis_with_document(&state, &s).await;

    let baseline = thens(&history(&state, &s, &id).await);
    let first = approved_edit(&state, &s, &id, ADD).await;
    let mid = thens(&history(&state, &s, &id).await);
    approved_edit(&state, &s, &id, "더 분명하게").await;
    let latest = thens(&history(&state, &s, &id).await);
    assert_ne!(mid, latest);

    let (status, view) = preview(&state, &s, &id, &first).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["isCurrent"], false);
    assert_eq!(thens(&view), mid, "미리보기가 그 시점의 문장이 아니다");
    assert!(!view["lines"].as_array().unwrap().is_empty());

    let (status, after) = restore(&state, &s, &id, &first).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thens(&after), mid, "고른 시점으로 정확히 복원되지 않았다");

    let list = entries(&after);
    assert_eq!(list.len(), 4, "복원이 이력에 남지 않았다");
    assert_eq!(list[3]["kind"], "restore");
    assert_eq!(list[3]["source"], "user_direct");
    assert_eq!(list[3]["restoredTo"], first);
    assert_eq!(list[3]["current"], true);
    assert_eq!(
        list[2]["standing"], false,
        "잘려 나간 편집이 아직 문서에 서 있다"
    );
    assert_eq!(list[2]["kind"], "edit", "잘렸다고 이력에서 지우지 않는다");

    let (status, back) = restore(&state, &s, &id, "auto").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thens(&back), baseline);
}

#[tokio::test]
async fn restoring_the_point_the_document_already_stands_at_is_refused() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9403, "stayer").await;
    let id = analysis_with_document(&state, &s).await;
    let (status, _) = restore(&state, &s, &id, "auto").await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "되돌릴 것이 없는 복원이 이력을 늘렸다"
    );

    let edit = approved_edit(&state, &s, &id, ADD).await;
    let (status, _) = restore(&state, &s, &id, &edit).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(entries(&history(&state, &s, &id).await).len(), 2);
}

#[tokio::test]
async fn an_edit_made_after_a_restore_stands_on_top_of_it() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9404, "continuer").await;
    let id = analysis_with_document(&state, &s).await;

    let first = approved_edit(&state, &s, &id, ADD).await;
    approved_edit(&state, &s, &id, "더 분명하게").await;
    let (status, _) = restore(&state, &s, &id, &first).await;
    assert_eq!(status, StatusCode::OK);

    let after_restore = approved_edit(&state, &s, &id, "한 번 더 다듬어").await;
    let view = history(&state, &s, &id).await;
    let list = entries(&view);
    let standing: Vec<&str> = list
        .iter()
        .filter(|e| e["standing"] == true && e["kind"] == "edit")
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        standing,
        vec![first.as_str(), after_restore.as_str()],
        "복원 뒤의 편집이 함께 서지 않았다"
    );
    assert_eq!(list.last().unwrap()["id"], after_restore);
}

#[tokio::test]
async fn a_restore_is_itself_a_point_that_can_be_restored_to() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9405, "undoer").await;
    let id = analysis_with_document(&state, &s).await;

    let first = approved_edit(&state, &s, &id, ADD).await;
    let second = approved_edit(&state, &s, &id, "더 분명하게").await;
    let latest = thens(&history(&state, &s, &id).await);

    let (_, after) = restore(&state, &s, &id, &first).await;
    let undone = entries(&after).last().unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ne!(thens(&after), latest);

    let (status, again) = restore(&state, &s, &id, &second).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(thens(&again), latest, "복원을 되돌리지 못했다");
    assert_ne!(undone, "");
    assert_eq!(entries(&again).len(), 5);

    let (status, view) = preview(&state, &s, &id, &undone).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(view["isCurrent"], false);
}

#[tokio::test]
async fn another_users_history_is_not_found_and_neither_is_an_unknown_point() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 9406, "owner").await;
    let other = login_installed(&state, 9407, "other").await;
    let id = analysis_with_document(&state, &owner).await;

    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/features/{}/history", encoded(FEATURE)),
            &other,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let (status, _) = preview(&state, &owner, &id, "no-such-point").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = restore(&state, &owner, &id, "no-such-point").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
