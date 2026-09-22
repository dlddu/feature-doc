//! Feature 단위 종단 의존성과 그 구조화 적재를 앱 전체로 묻는다.
//!
//! 문서는 손으로 쓰지 않고 `dependencies::derive` 가 stub 모드로 만든 것을 워커의
//! `/internal` 라우트로 넣는다 — 픽스처는 워커가 실제로 보내는 것과 갈린다.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::acceptance::{self, Subject};
use featuredoc::config::Mode;
use featuredoc::dependencies;
use featuredoc::github_api::GithubUser;
use featuredoc::llm::Provider;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, repo_scan, session, users};

const WORKER: &str = "w-dependencies";

/// The two candidates every test here confirms. Both cite paths the stub tree has.
const SIFTED: [(&str, &str); 2] = [
    ("결제 수단 등록", "payments-api/src/api/routes.rs"),
    ("주문 내역 내려받기", "payments-api/src/domain/model.rs"),
];

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

fn user_send(method: &str, uri: &str, token: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
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

async fn enqueue(state: &AppState, session: &str, repo: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            "/api/analyses",
            session,
            json!({ "repoUrl": format!("stub-account/{repo}") }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn try_claim(state: &AppState) -> Option<serde_json::Value> {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    if resp.status() == StatusCode::NO_CONTENT {
        return None;
    }
    assert_eq!(resp.status(), StatusCode::OK);
    Some(json_body(resp).await)
}

async fn claim(state: &AppState) -> serde_json::Value {
    try_claim(state).await.expect("queue should hold a job")
}

async fn report(state: &AppState, id: &str, key: &str, status: &str) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/{key}"),
            json!({ "workerId": WORKER, "status": status }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "stage report {key}");
}

async fn submit(state: &AppState, id: &str, kind: &str, content: serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/{kind}"),
            json!({ "workerId": WORKER, "content": content, "model": "stub-model" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "document submit {kind}");
}

async fn finish(state: &AppState, id: &str, status: &str) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            json!({ "workerId": WORKER, "status": status }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "finish");
}

fn tree() -> Vec<String> {
    repo_scan::stub_scan("payments-api", "main").unwrap().paths
}

fn strategy_doc() -> serde_json::Value {
    json!({
        "entries": [{
            "pattern": "payments-api/src/**",
            "kind": "route",
            "rationale": "stub",
            "evidence": ["payments-api/src/main.rs"],
        }],
    })
}

fn candidate_doc() -> serde_json::Value {
    let candidates: Vec<serde_json::Value> = SIFTED
        .iter()
        .map(|(name, location)| {
            json!({ "name": name, "location": location, "rationale": "stub" })
        })
        .collect();
    json!({ "candidates": candidates })
}

/// Walks an analysis to the state every dependency test starts from: candidates confirmed.
async fn run_to_confirmed(state: &AppState, session: &str, id: &str, approve: usize) -> Vec<String> {
    let job = claim(state).await;
    assert_eq!(job["id"], id);
    for key in ["fetch", "cross_cutting", "discovery_strategy"] {
        report(state, id, key, "succeeded").await;
    }
    submit(state, id, "cross_cutting", json!({ "categories": [] })).await;
    submit(state, id, "discovery_strategy", strategy_doc()).await;
    finish(state, id, "awaiting_pipeline").await;

    assert!(build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/discovery-strategy"), session))
        .await
        .unwrap()
        .status()
        .is_success());
    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/discovery-strategy/approve"),
            session,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let job = claim(state).await;
    assert_eq!(job["id"], id);
    report(state, id, "feature_candidates", "succeeded").await;
    submit(state, id, "feature_candidates", candidate_doc()).await;
    finish(state, id, "awaiting_pipeline").await;

    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/candidates"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let listed = json_body(resp).await;
    let keys: Vec<String> = listed["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["key"].as_str().unwrap().to_string())
        .collect();

    let mut approved = Vec::new();
    for key in keys.into_iter().take(approve) {
        let status = build_router(state.clone())
            .oneshot(user_send(
                "POST",
                &format!("/api/analyses/{id}/candidates/decision"),
                session,
                json!({ "key": key, "decision": "approve" }),
            ))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::OK, "approve {key}");
        approved.push(key);
    }

    // Run stage 5 with the document a worker would actually submit, so the stage's
    // coverage predicate comes to rest.
    let job = claim(state).await;
    assert_eq!(job["id"], id);
    let subjects: Vec<Subject> = job["approvedCandidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| Subject {
            key: c["key"].as_str().unwrap().to_string(),
            name: c["name"].as_str().unwrap().to_string(),
            location: c["location"].as_str().unwrap().to_string(),
            symbol: c["symbol"].as_str().map(str::to_string),
        })
        .collect();
    let answer = acceptance::derive(
        &reqwest::Client::new(),
        Mode::Stub,
        Provider::Anthropic,
        None,
        None,
        "stub-account",
        "payments-api",
        "main",
        &tree(),
        &subjects,
    )
    .await
    .expect("stage 5 produces a document in stub mode");
    report(state, id, "acceptance_dependencies", "succeeded").await;
    submit(state, id, "acceptance_dependencies", answer.content).await;
    finish(state, id, "awaiting_pipeline").await;
    approved
}

async fn request_trace(state: &AppState, session: &str, id: &str, key: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/features/dependencies"),
            session,
            json!({ "key": key }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "request the trace");
    json_body(resp).await
}

async fn read_trace(state: &AppState, session: &str, id: &str, key: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/analyses/{id}/features/dependencies?key={}",
                urlencoding(key)
            ),
            session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "read the trace");
    json_body(resp).await
}

/// Percent-encoding, byte by byte. Dependency names carry non-ASCII (`domain · model`),
/// and encoding by `char` would emit the code point instead of its UTF-8 bytes.
fn urlencoding(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Runs the trace the way the worker does: with the feature the claim handed over.
async fn run_trace(state: &AppState, id: &str, job: &serde_json::Value, index: usize) {
    let requested = job["dependencyRequests"].as_array().unwrap();
    let feature = &requested[index];
    let subject = Subject {
        key: feature["key"].as_str().unwrap().to_string(),
        name: feature["name"].as_str().unwrap().to_string(),
        location: feature["location"].as_str().unwrap().to_string(),
        symbol: feature["symbol"].as_str().map(str::to_string),
    };
    let answer = dependencies::derive(
        &reqwest::Client::new(),
        Mode::Stub,
        Provider::Anthropic,
        None,
        None,
        "stub-account",
        "payments-api",
        "main",
        &tree(),
        &subject,
    )
    .await
    .expect("the trace produces items in stub mode");

    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/dependencies"),
            json!({
                "workerId": WORKER,
                "featureKey": subject.key,
                "status": "succeeded",
                "content": answer.content,
                "model": "stub-model",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "dependency submit");
}

fn items_of(view: &serde_json::Value) -> Vec<(String, String, Option<String>)> {
    view["categories"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["items"].as_array().unwrap().iter())
        .map(|item| {
            (
                item["category"].as_str().unwrap().to_string(),
                item["name"].as_str().unwrap().to_string(),
                item["evidence"].as_str().map(str::to_string),
            )
        })
        .collect()
}

#[tokio::test]
async fn nothing_is_traced_until_someone_asks() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    let approved = run_to_confirmed(&state, &session, &id, 1).await;

    assert!(try_claim(&state).await.is_none(), "queue should be at rest");

    let view = read_trace(&state, &session, &id, &approved[0]).await;
    assert!(view["status"].is_null(), "아직 요청 전이다: {view}");
    assert_eq!(view["total"], 0);
    // 일곱 칸은 언제나 그려진다 — 빈 분류도 정보다.
    assert_eq!(view["categories"].as_array().unwrap().len(), 7);

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_request_requeues_and_the_trace_cites_the_scanned_tree() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    let approved = run_to_confirmed(&state, &session, &id, 1).await;

    let requested = request_trace(&state, &session, &id, &approved[0]).await;
    assert_eq!(requested["status"], "queued");

    let job = claim(&state).await;
    assert_eq!(job["id"], id);
    let carried = job["dependencyRequests"].as_array().unwrap();
    assert_eq!(carried.len(), 1, "claim 이 요청된 feature 를 싣는다: {job}");
    assert_eq!(carried[0]["key"].as_str().unwrap(), approved[0]);
    let stages: Vec<&str> = job["executableStages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert!(!stages.iter().any(|s| s.contains("dependenc")), "{stages:?}");

    run_trace(&state, &id, &job, 0).await;
    finish(&state, &id, "awaiting_pipeline").await;

    let view = read_trace(&state, &session, &id, &approved[0]).await;
    assert_eq!(view["status"], "succeeded");
    let items = items_of(&view);
    assert!(!items.is_empty(), "{view}");
    assert_eq!(items.len() as u64, view["total"].as_u64().unwrap());

    let scanned = tree();
    let mut categories: Vec<String> = Vec::new();
    for (category, _, evidence) in &items {
        assert!(dependencies::is_category(category), "{category}");
        if let Some(evidence) = evidence {
            assert!(
                scanned.iter().any(|p| p == evidence),
                "근거는 이 분석이 본 경로여야 한다: {evidence}"
            );
        }
        if !categories.contains(category) {
            categories.push(category.clone());
        }
    }
    assert!(
        categories.len() >= 5,
        "다층 의존성은 분류별로 갈려야 한다: {categories:?}"
    );

    assert!(try_claim(&state).await.is_none(), "the request is settled");

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn the_reverse_query_finds_every_feature_that_depends_on_the_same_thing() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    let approved = run_to_confirmed(&state, &session, &id, 2).await;
    assert_eq!(approved.len(), 2);

    for key in &approved {
        request_trace(&state, &session, &id, key).await;
    }
    let job = claim(&state).await;
    assert_eq!(job["dependencyRequests"].as_array().unwrap().len(), 2);
    run_trace(&state, &id, &job, 0).await;
    run_trace(&state, &id, &job, 1).await;
    finish(&state, &id, "awaiting_pipeline").await;

    // 공유 항목이 어느 이름인지는 트리가 정하지, 이 테스트가 정하지 않는다.
    let first = items_of(&read_trace(&state, &session, &id, &approved[0]).await);
    let second = items_of(&read_trace(&state, &session, &id, &approved[1]).await);
    let shared = first
        .iter()
        .find(|(category, name, _)| {
            second
                .iter()
                .any(|(c, n, _)| c == category && n == name)
        })
        .expect("두 feature 가 공유하는 의존성이 있어야 한다");

    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/dependencies/features?category={}&name={}",
                urlencoding(&shared.0),
                urlencoding(&shared.1)
            ),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let found = json_body(resp).await;
    let features = found["features"].as_array().unwrap();
    assert_eq!(features.len(), 2, "{found}");
    for feature in features {
        assert!(approved.contains(&feature["featureKey"].as_str().unwrap().to_string()));
        assert_eq!(feature["repoName"], "payments-api");
    }

    let resp = build_router(state.clone())
        .oneshot(get(
            "/api/dependencies/features?category=wishful&name=whatever",
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let stranger = login_installed(&state, 2, "bob").await;
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/dependencies/features?category={}&name={}",
                urlencoding(&shared.0),
                urlencoding(&shared.1)
            ),
            &stranger,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(json_body(resp).await["features"]
        .as_array()
        .unwrap()
        .is_empty());

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn export_hands_back_a_file_something_else_can_read() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    let approved = run_to_confirmed(&state, &session, &id, 1).await;

    request_trace(&state, &session, &id, &approved[0]).await;
    let job = claim(&state).await;
    run_trace(&state, &id, &job, 0).await;
    finish(&state, &id, "awaiting_pipeline").await;

    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/dependencies/export"),
            &session,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let disposition = resp
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .expect("export is an attachment")
        .to_str()
        .unwrap()
        .to_string();
    assert!(disposition.starts_with("attachment;"), "{disposition}");

    let exported = json_body(resp).await;
    assert_eq!(exported["repository"], "stub-account/payments-api");
    assert_eq!(
        exported["categories"].as_array().unwrap().len(),
        dependencies::CATEGORIES.len()
    );
    let features = exported["features"].as_array().unwrap();
    assert_eq!(features.len(), 1, "{exported}");
    assert_eq!(features[0]["key"].as_str().unwrap(), approved[0]);
    assert!(!features[0]["dependencies"].as_array().unwrap().is_empty());

    // 남의 분석은 존재하지 않는다.
    let stranger = login_installed(&state, 2, "bob").await;
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/dependencies/export"),
            &stranger,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn a_failed_retrace_keeps_the_previous_answer() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    let approved = run_to_confirmed(&state, &session, &id, 1).await;

    request_trace(&state, &session, &id, &approved[0]).await;
    let job = claim(&state).await;
    run_trace(&state, &id, &job, 0).await;
    finish(&state, &id, "awaiting_pipeline").await;
    let before = items_of(&read_trace(&state, &session, &id, &approved[0]).await);
    assert!(!before.is_empty());

    request_trace(&state, &session, &id, &approved[0]).await;
    let job = claim(&state).await;
    assert_eq!(job["dependencyRequests"].as_array().unwrap().len(), 1);
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/dependencies"),
            json!({
                "workerId": WORKER,
                "featureKey": approved[0],
                "status": "failed",
                "error": "no LLM key available for this analysis",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    finish(&state, &id, "awaiting_pipeline").await;

    let view = read_trace(&state, &session, &id, &approved[0]).await;
    assert_eq!(view["status"], "failed");
    assert_eq!(view["error"], "no LLM key available for this analysis");
    assert_eq!(items_of(&view), before, "직전 답이 살아남는다");

    // 사유 없는 실패는 받지 않는다 — 사유 없는 거부를 받지 않는 것과 같은 이유다.
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/dependencies"),
            json!({
                "workerId": WORKER,
                "featureKey": approved[0],
                "status": "failed",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let _ = std::fs::remove_file(&path);
}
