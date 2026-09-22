//! 코드 변경 후 재분석 diff.
//!
//! 판정 규칙 자체는 `src/diff.rs` 의 단위 테스트가 지킨다 — 여기서 묻는 것은 앱
//! 전체만 답할 수 있는 것뿐이다.
//!
//! 인수 문서는 여기서만 손으로 만든다 — 답이 달라지는 지점을 테스트가 쥐고 있어야
//! 하기 때문이다. 워커가 실제로 보내는 모양은 그대로 따른다.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::config::Mode;
use featuredoc::github_api::GithubUser;
use featuredoc::llm::Provider;
use featuredoc::state::AppState;
use featuredoc::{acceptance::Subject, build_router, dependencies, installations, session, users};

const WORKER: &str = "w-diff";

/// 두 분석이 공유하는 후보 두 건. 둘 다 stub 트리가 실제로 가진 경로를 인용한다.
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

fn user_send(method: &str, uri: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
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

async fn enqueue(state: &AppState, session: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            "/api/analyses",
            session,
            json!({ "repoUrl": "stub-account/payments-api" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

async fn claim(state: &AppState, id: &str) -> Value {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "queue should hold {id}");
    let job = json_body(resp).await;
    assert_eq!(job["id"], id);
    job
}

async fn report(state: &AppState, id: &str, key: &str) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/{key}"),
            json!({ "workerId": WORKER, "status": "succeeded" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "stage report {key}");
}

async fn submit(state: &AppState, id: &str, kind: &str, content: Value) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/{kind}"),
            json!({ "workerId": WORKER, "content": content, "model": "stub-model" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "document submit {kind}");
}

async fn finish(state: &AppState, id: &str) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/finish"),
            json!({ "workerId": WORKER, "status": "awaiting_pipeline" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "finish");
}

fn candidate_doc() -> Value {
    let candidates: Vec<Value> = SIFTED
        .iter()
        .map(|(name, location)| json!({ "name": name, "location": location, "rationale": "stub" }))
        .collect();
    json!({ "candidates": candidates })
}

fn feature_doc(key: &str, name: &str, thens: &[&str]) -> Value {
    json!({
        "key": key,
        "name": name,
        "scenarios": thens
            .iter()
            .map(|then| json!({
                "given": format!("{name} 을(를) 쓰는 사용자가"),
                "when": "요청을 보내면",
                "then": then,
                "evidence": "payments-api/src/api/routes.rs",
                "source": "logic",
            }))
            .collect::<Vec<_>>(),
    })
}

async fn run_to_documented(
    state: &AppState,
    session: &str,
    id: &str,
    scenarios: &[(&str, Vec<&str>)],
) -> Vec<String> {
    claim(state, id).await;
    for key in ["fetch", "cross_cutting", "discovery_strategy"] {
        report(state, id, key).await;
    }
    submit(state, id, "cross_cutting", json!({ "categories": [] })).await;
    submit(
        state,
        id,
        "discovery_strategy",
        json!({
            "entries": [{
                "pattern": "payments-api/src/**",
                "kind": "route",
                "rationale": "stub",
                "evidence": ["payments-api/src/main.rs"],
            }],
        }),
    )
    .await;
    finish(state, id).await;

    // 읽는 순간 검토 가능한 전략이 심어진다(lazy seed). 이 GET 없이 approve 를 치면
    // `load_strategy` 가 아직 없는 행을 찾아 404 다.
    assert!(build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/discovery-strategy"), session))
        .await
        .unwrap()
        .status()
        .is_success());

    let status = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/discovery-strategy/approve"),
            session,
            json!({}),
        ))
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::OK);

    claim(state, id).await;
    report(state, id, "feature_candidates").await;
    submit(state, id, "feature_candidates", candidate_doc()).await;
    finish(state, id).await;

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

    for key in &keys {
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
    }

    claim(state, id).await;
    let features: Vec<Value> = keys
        .iter()
        .zip(SIFTED.iter())
        .map(|(key, (name, _))| {
            let thens = scenarios
                .iter()
                .find(|(subject, _)| subject == name)
                .map(|(_, thens)| thens.clone())
                .unwrap_or_default();
            feature_doc(key, name, &thens)
        })
        .collect();
    report(state, id, "acceptance_dependencies").await;
    submit(
        state,
        id,
        "acceptance_dependencies",
        json!({ "features": features }),
    )
    .await;
    finish(state, id).await;
    keys
}

/// `tree` 는 그 시점의 저장소 트리다 — 두 번째 분석은 한 걸음 나아간 트리를 본다.
async fn trace(state: &AppState, session: &str, id: &str, key: &str, tree: &[String]) {
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

    let job = claim(state, id).await;
    let requested = job["dependencyRequests"].as_array().unwrap();
    let feature = requested
        .iter()
        .find(|f| f["key"] == key)
        .expect("claim hands the requested feature over");
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
        tree,
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
    finish(state, id).await;
}

async fn read_diff(state: &AppState, session: &str, id: &str) -> Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/diff"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "read the diff");
    json_body(resp).await
}

fn first_revision() -> Vec<String> {
    featuredoc::repo_scan::stub_scan_at("payments-api", "main", featuredoc::repo_scan::Revision::First)
        .unwrap()
        .paths
}

fn second_revision() -> Vec<String> {
    featuredoc::repo_scan::stub_scan_at("payments-api", "main", featuredoc::repo_scan::Revision::Second)
        .unwrap()
        .paths
}

#[tokio::test]
async fn a_first_analysis_has_nothing_to_compare_against() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session).await;
    run_to_documented(
        &state,
        &session,
        &id,
        &[("결제 수단 등록", vec!["요청한 대로 처리됩니다"])],
    )
    .await;

    let view = read_diff(&state, &session, &id).await;
    assert!(view["comparedTo"].is_null(), "첫 분석이다: {view}");
    assert_eq!(view["features"].as_array().unwrap().len(), 0);
    assert_eq!(view["changedLineCount"], 0);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_re_analysis_lists_only_the_features_that_changed() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 2, "bob").await;

    let unchanged = ("주문 내역 내려받기", vec!["내려받기가 시작됩니다"]);
    let first = enqueue(&state, &session).await;
    run_to_documented(
        &state,
        &session,
        &first,
        &[
            ("결제 수단 등록", vec!["요청한 대로 처리됩니다"]),
            unchanged.clone(),
        ],
    )
    .await;

    let second = enqueue(&state, &session).await;
    let keys = run_to_documented(
        &state,
        &session,
        &second,
        &[
            (
                "결제 수단 등록",
                vec!["요청한 대로 처리됩니다", "만료된 카드는 저장하지 않고 알려 줍니다"],
            ),
            unchanged.clone(),
        ],
    )
    .await;

    let view = read_diff(&state, &session, &second).await;
    assert_eq!(view["comparedTo"], first, "직전 분석이 상대다: {view}");
    let features = view["features"].as_array().unwrap();
    assert_eq!(features.len(), 1, "바뀐 것은 하나뿐이다: {view}");
    assert_eq!(features[0]["key"], keys[0]);
    assert_eq!(features[0]["name"], "결제 수단 등록");
    assert_eq!(
        features[0]["location"], "payments-api/src/api/routes.rs",
        "카드는 그 기능을 찾은 자리를 밝힌다"
    );
    let lines = features[0]["scenarioLines"].as_array().unwrap();
    assert_eq!(lines.len(), 1, "더해진 한 줄만 뜬다: {view}");
    assert_eq!(lines[0]["mark"], "+");
    assert_eq!(lines[0]["text"], "만료된 카드는 저장하지 않고 알려 줍니다");
    assert_eq!(view["changedLineCount"], 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn an_added_dependency_shows_up_as_a_line() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 3, "carol").await;
    let same = [
        ("결제 수단 등록", vec!["요청한 대로 처리됩니다"]),
        ("주문 내역 내려받기", vec!["내려받기가 시작됩니다"]),
    ];

    let first = enqueue(&state, &session).await;
    let first_keys = run_to_documented(&state, &session, &first, &same).await;
    trace(&state, &session, &first, &first_keys[0], &first_revision()).await;

    let second = enqueue(&state, &session).await;
    let second_keys = run_to_documented(&state, &session, &second, &same).await;
    trace(&state, &session, &second, &second_keys[0], &second_revision()).await;
    assert_eq!(first_keys[0], second_keys[0], "feature 의 정체성은 분석을 가로지른다");

    let view = read_diff(&state, &session, &second).await;
    let features = view["features"].as_array().unwrap();
    assert_eq!(features.len(), 1, "시나리오는 그대로, 의존성만 달라졌다: {view}");
    assert_eq!(features[0]["scenarioLines"].as_array().unwrap().len(), 0);
    let lines = features[0]["dependencyLines"].as_array().unwrap();
    assert_eq!(lines.len(), 1, "새 모듈 한 줄: {view}");
    assert_eq!(lines[0]["mark"], "+");
    assert_eq!(lines[0]["category"], "logic");
    assert_eq!(lines[0]["name"], "billing · refund");

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn an_untraced_side_is_not_read_as_a_removal() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 4, "dave").await;
    let same = [
        ("결제 수단 등록", vec!["요청한 대로 처리됩니다"]),
        ("주문 내역 내려받기", vec!["내려받기가 시작됩니다"]),
    ];

    let first = enqueue(&state, &session).await;
    let first_keys = run_to_documented(&state, &session, &first, &same).await;
    trace(&state, &session, &first, &first_keys[0], &first_revision()).await;

    let second = enqueue(&state, &session).await;
    run_to_documented(&state, &session, &second, &same).await;

    let view = read_diff(&state, &session, &second).await;
    assert_eq!(view["comparedTo"], first);
    assert_eq!(
        view["features"].as_array().unwrap().len(),
        0,
        "이번에 묻지 않았을 뿐이다: {view}"
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn another_users_analysis_is_a_404() {
    let (state, path) = stub_state().await;
    let alice = login_installed(&state, 5, "alice2").await;
    let mallory = login_installed(&state, 6, "mallory").await;
    let id = enqueue(&state, &alice).await;

    let status = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/diff"), &mallory))
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NOT_FOUND);

    let _ = std::fs::remove_file(path);
}
