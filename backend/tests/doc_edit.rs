//! LLM 보조 수정(AC3.1)과 그 변경의 출처 보존(AC3.4) — 라우터를 그대로 돌려 본다.
//!
//! 인수 문서는 워커의 `/internal` 경로로 넣는다(형제 테스트와 같은 방침) — 손으로 행을
//! 꽂으면 픽스처가 실제 워커 산출물과 어긋날 수 있고, 그 계약이 바로 여기서 지켜야 할 것이다.
//!
//! 브라우저에서 관측되는 3탭 흐름은 `e2e/tests/sc03-01`·`sc03-02` 가 지킨다. 이 파일이
//! 지키는 것은 그 아래의 규칙이다 — 승인 전 문서 불변 · 승인 후 겹쳐 읽기 · 거부의 기록과
//! 회피 · 남의 분석에는 닿지 않음.
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

const WORKER: &str = "w-doc-edit";
const FEATURE: &str = "payments-api/src/api/routes.rs";

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

/// 인수 문서가 서 있는 분석 하나. 편집은 그 문서 위에서만 일어난다.
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
    assert_eq!(submitted.status(), StatusCode::NO_CONTENT, "document submit");
    id
}

async fn scenarios(state: &AppState, session: &str, id: &str) -> Vec<serde_json::Value> {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/documents/acceptance-dependencies"),
            session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await["content"]["features"][0]["scenarios"]
        .as_array()
        .unwrap()
        .clone()
}

async fn propose(
    state: &AppState,
    session: &str,
    id: &str,
    request: &str,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals"),
            session,
            json!({ "key": FEATURE, "scenarioIndex": 0, "request": request }),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn decide(
    state: &AppState,
    session: &str,
    id: &str,
    proposal: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals/{proposal}/decision"),
            session,
            body,
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn context(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/analyses/{id}/features/edit-context?key={}&scenario=0",
                urlencoding(FEATURE)
            ),
            session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

/// feature key 는 경로 구분자를 품는다 — 쿼리로 실어 보내므로 그만큼은 감싸 준다.
fn urlencoding(value: &str) -> String {
    value.replace('/', "%2F")
}

#[tokio::test]
async fn approving_a_proposal_is_what_changes_the_document() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9301, "editor").await;
    let id = analysis_with_document(&state, &s).await;
    let before = scenarios(&state, &s, &id).await;
    assert_eq!(before.len(), 1);

    let (status, proposal) = propose(&state, &s, &id, "에러 케이스 1개 더 추가").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(proposal["status"], "proposed");
    assert_eq!(proposal["source"], "user_llm");
    assert_eq!(proposal["after"].as_array().unwrap().len(), 2);
    assert_eq!(
        scenarios(&state, &s, &id).await.len(),
        1,
        "승인하지 않았는데 문서가 이미 바뀌었다"
    );

    let (status, decided) = decide(
        &state,
        &s,
        &id,
        proposal["id"].as_str().unwrap(),
        json!({ "decision": "approve" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decided["status"], "approved");

    let after = scenarios(&state, &s, &id).await;
    assert_eq!(after.len(), 2, "승인했는데 시나리오가 늘지 않았다");
    assert_eq!(
        after[0]["then"], before[0]["then"],
        "「추가」인데 원래 시나리오가 밀려났다"
    );
    // 근거는 물려받는다 — 사람이 고친 것은 문장이지 그 문장이 어디서 왔는지가 아니다.
    assert_eq!(after[1]["evidence"], before[0]["evidence"]);
}

#[tokio::test]
async fn a_rejected_proposal_is_recorded_and_the_next_one_avoids_it() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9302, "rejecter").await;
    let id = analysis_with_document(&state, &s).await;

    let (_, first) = propose(&state, &s, &id, "더 분명하게").await;
    let rejected_text = first["after"][0]["then"].as_str().unwrap().to_string();

    // 사유 없는 거부는 받지 않는다 — 그러면 다음 제안이 무엇을 피할지 알 수 없다.
    let (status, _) = decide(
        &state,
        &s,
        &id,
        first["id"].as_str().unwrap(),
        json!({ "decision": "reject" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = decide(
        &state,
        &s,
        &id,
        first["id"].as_str().unwrap(),
        json!({ "decision": "reject", "reason": "오류 코드를 덧붙인 표현은 필요 없어요" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        scenarios(&state, &s, &id).await.len(),
        1,
        "거부했는데 문서가 바뀌었다"
    );

    // 사람에게 먼저 알린다.
    let ctx = context(&state, &s, &id).await;
    assert_eq!(ctx["rejectedCount"], 1);
    assert_eq!(ctx["rejectedReason"], "오류 코드를 덧붙인 표현은 필요 없어요");

    let (_, second) = propose(&state, &s, &id, "더 분명하게").await;
    assert_ne!(
        second["after"][0]["then"].as_str().unwrap(),
        rejected_text,
        "거부한 제안이 그대로 다시 왔다"
    );
}

#[tokio::test]
async fn a_decided_proposal_cannot_be_decided_again() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9303, "twice").await;
    let id = analysis_with_document(&state, &s).await;

    let (_, proposal) = propose(&state, &s, &id, "더 분명하게").await;
    let pid = proposal["id"].as_str().unwrap();
    let (first, _) = decide(&state, &s, &id, pid, json!({ "decision": "approve" })).await;
    assert_eq!(first, StatusCode::OK);
    let (again, _) = decide(&state, &s, &id, pid, json!({ "decision": "approve" })).await;
    assert_eq!(again, StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_empty_request_never_reaches_the_model() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9304, "blank").await;
    let id = analysis_with_document(&state, &s).await;

    let (status, _) = propose(&state, &s, &id, "   ").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// AC4.7 — 남의 분석은 없는 것과 같다. 제안도 그 분석에 매인다.
#[tokio::test]
async fn another_users_analysis_is_not_editable() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 9305, "owner").await;
    let stranger = login_installed(&state, 9306, "stranger").await;
    let id = analysis_with_document(&state, &owner).await;

    let (status, _) = propose(&state, &stranger, &id, "더 분명하게").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, proposal) = propose(&state, &owner, &id, "더 분명하게").await;
    let (status, _) = decide(
        &state,
        &stranger,
        &id,
        proposal["id"].as_str().unwrap(),
        json!({ "decision": "approve" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
