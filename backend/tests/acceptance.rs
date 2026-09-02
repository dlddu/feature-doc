//! Acceptance scenarios for a confirmed feature (AC2.1 · AC2.2 · AC2.3).
//!
//! The generation itself is unit-tested inside `src/acceptance.rs` — this file is
//! about the two things only the whole app can answer:
//!
//!   * **the gate** — stage 5 is not offered until the reviewer has confirmed a
//!     feature, and approving one is what re-queues the analysis so the stage gets a
//!     turn. Rejecting confirms nothing, so it re-queues nothing.
//!   * **coverage** — a reviewer decides candidates one at a time, so confirming a
//!     second feature *after* the stage ran must re-open it. "The stage succeeded"
//!     is the wrong predicate here and this is where that shows.
//!
//! Documents are written through the worker's own `/internal` route and produced by
//! `acceptance::derive` in stub mode, not hand-written — a fixture would drift from
//! what a worker actually submits.

mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use common::{stub_state, WORKER_TOKEN};
use featuredoc::acceptance::{self, Subject};
use featuredoc::config::Mode;
use featuredoc::github_api::GithubUser;
use featuredoc::llm::Provider;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, repo_scan, session, users};

const WORKER: &str = "w-acceptance";

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

/// One claim attempt. `None` is `204` — the queue is empty, which is itself an
/// assertion several tests here make.
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

fn offered(job: &serde_json::Value) -> Vec<String> {
    job["executableStages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
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

async fn status_of(state: &AppState, id: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT status FROM analyses WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .unwrap()
}

/// The two candidates every test below sifts. Both cite paths the stub tree has, so
/// stage 5's evidence check has something real to accept.
const SIFTED: [(&str, &str); 2] = [
    ("결제 수단 등록", "payments-api/src/api/routes.rs"),
    ("주문 내역 내려받기", "payments-api/src/domain/model.rs"),
];

fn candidate_doc() -> serde_json::Value {
    let candidates: Vec<serde_json::Value> = SIFTED
        .iter()
        .map(|(name, location)| {
            json!({ "name": name, "location": location, "rationale": "stub" })
        })
        .collect();
    json!({ "candidates": candidates })
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

/// The tree stage 1 measured — the same list stage 5 reads, so the test's evidence
/// expectations and the stage's own validation come from one source.
fn tree() -> Vec<String> {
    repo_scan::stub_scan("payments-api", "main").unwrap().paths
}

/// Walks an analysis to "stage 4 ran and the reviewer has a list to sift".
async fn run_to_candidates(state: &AppState, session: &str, id: &str) {
    let job = claim(state).await;
    assert_eq!(job["id"], id, "claimed a different job than the test meant to");
    for key in ["fetch", "cross_cutting", "discovery_strategy"] {
        report(state, id, key, "succeeded").await;
    }
    submit(state, id, "cross_cutting", json!({ "categories": [] })).await;
    submit(state, id, "discovery_strategy", strategy_doc()).await;
    finish(state, id, "awaiting_pipeline").await;

    // Reading materialises the reviewable strategy (AC1.3's lazy seed); approving is
    // what re-queues the job so stage 4 gets its turn.
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
    assert_eq!(resp.status(), StatusCode::OK, "approve the strategy");

    let job = claim(state).await;
    assert_eq!(job["id"], id);
    report(state, id, "feature_candidates", "succeeded").await;
    submit(state, id, "feature_candidates", candidate_doc()).await;
    finish(state, id, "awaiting_pipeline").await;
}

async fn candidates(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/candidates"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "candidate list");
    json_body(resp).await
}

async fn decide(
    state: &AppState,
    session: &str,
    id: &str,
    key: &str,
    body: serde_json::Value,
) -> StatusCode {
    let mut body = body;
    body["key"] = json!(key);
    build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/candidates/decision"),
            session,
            body,
        ))
        .await
        .unwrap()
        .status()
}

/// Runs stage 5 the way the worker does: with the candidates the claim handed over.
async fn run_stage_five(state: &AppState, id: &str, job: &serde_json::Value) -> serde_json::Value {
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
        "stub-account",
        "payments-api",
        "main",
        &tree(),
        &subjects,
    )
    .await
    .expect("stage 5 produces a document in stub mode");

    report(state, id, "acceptance_dependencies", "succeeded").await;
    submit(state, id, "acceptance_dependencies", answer.content.clone()).await;
    finish(state, id, "awaiting_pipeline").await;
    answer.content
}

async fn document(state: &AppState, session: &str, id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(get(
            &format!("/api/analyses/{id}/documents/acceptance-dependencies"),
            session,
        ))
        .await
        .unwrap()
}

/// AC2.1 is about a **confirmed** feature, so an unreviewed candidate list is not
/// something to write scenarios from. The gate is the queue's, not the worker's.
#[tokio::test]
async fn stage_five_is_withheld_until_a_feature_is_confirmed() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_to_candidates(&state, &session, &id).await;

    // Nothing decided yet: the analysis has come to rest and the queue is empty, so
    // no amount of claiming would run stage 5.
    assert_eq!(status_of(&state, &id).await, "awaiting_pipeline");
    assert!(
        try_claim(&state).await.is_none(),
        "an analysis waiting on the reviewer is not claimable work"
    );

    // Confirming one feature opens it — and hands over that feature, not the list.
    let key = candidates(&state, &session, &id).await["candidates"][0]["key"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        decide(&state, &session, &id, &key, json!({ "decision": "approve" })).await,
        StatusCode::OK
    );
    assert_eq!(
        status_of(&state, &id).await,
        "queued",
        "approving a candidate re-queues the analysis"
    );

    let job = claim(&state).await;
    let stages = offered(&job);
    assert!(stages.iter().any(|s| s == "acceptance_dependencies"), "{stages:?}");
    assert!(
        !stages.iter().any(|s| s == "cross_cutting" || s == "feature_candidates"),
        "a stage that already succeeded must not be re-offered: {stages:?}"
    );
    let handed = job["approvedCandidates"].as_array().unwrap();
    assert_eq!(handed.len(), 1, "only the confirmed feature is handed over");
    assert_eq!(handed[0]["key"], key.as_str());

    let _ = std::fs::remove_file(&path);
}

/// A rejection confirms nothing, so it opens nothing. Without this an analysis whose
/// every candidate was rejected would be re-queued for a stage that can never run.
#[tokio::test]
async fn rejecting_a_candidate_does_not_requeue_the_analysis() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 2, "bob").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_to_candidates(&state, &session, &id).await;

    let key = candidates(&state, &session, &id).await["candidates"][0]["key"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        decide(
            &state,
            &session,
            &id,
            &key,
            json!({ "decision": "reject", "reason": "내부 도구라 사용자 기능이 아님" })
        )
        .await,
        StatusCode::OK
    );
    assert_eq!(status_of(&state, &id).await, "awaiting_pipeline");

    let _ = std::fs::remove_file(&path);
}

/// The document is one per analysis, so the reviewer confirming a *second* feature
/// after the stage ran has to re-open it. "Has the stage succeeded" would answer no
/// here and the second feature would never get a document.
#[tokio::test]
async fn confirming_a_second_feature_reopens_the_stage() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 3, "carol").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_to_candidates(&state, &session, &id).await;

    let list = candidates(&state, &session, &id).await;
    let first = list["candidates"][0]["key"].as_str().unwrap().to_string();
    let second = list["candidates"][1]["key"].as_str().unwrap().to_string();

    decide(&state, &session, &id, &first, json!({ "decision": "approve" })).await;
    let job = claim(&state).await;
    let doc = run_stage_five(&state, &id, &job).await;
    let documented = acceptance::features(&doc);
    assert_eq!(documented.len(), 1);
    assert_eq!(documented[0].key, first);

    // Everything confirmed is documented, so there is nothing left to do.
    assert_eq!(status_of(&state, &id).await, "awaiting_pipeline");
    assert!(try_claim(&state).await.is_none(), "no work left to claim");

    // …until a second feature is confirmed.
    decide(&state, &session, &id, &second, json!({ "decision": "approve" })).await;
    assert_eq!(status_of(&state, &id).await, "queued");
    let job = claim(&state).await;
    let stages = offered(&job);
    assert!(stages.iter().any(|s| s == "acceptance_dependencies"), "{stages:?}");
    assert_eq!(job["approvedCandidates"].as_array().unwrap().len(), 2);

    let doc = run_stage_five(&state, &id, &job).await;
    let documented = acceptance::features(&doc);
    assert_eq!(documented.len(), 2, "the rerun covers both confirmed features");

    let _ = std::fs::remove_file(&path);
}

/// What the reviewer reads on S08, and who may read it (AC4.7).
#[tokio::test]
async fn the_document_is_readable_by_its_owner_and_nobody_else() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 4, "dave").await;
    let stranger = login_installed(&state, 5, "erin").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_to_candidates(&state, &session, &id).await;

    // Before the stage has run, "not there yet" is a 404 — the same distinction the
    // cross-cutting document draws between "has not run" and "found nothing".
    assert_eq!(
        document(&state, &session, &id).await.status(),
        StatusCode::NOT_FOUND
    );

    let key = candidates(&state, &session, &id).await["candidates"][0]["key"]
        .as_str()
        .unwrap()
        .to_string();
    decide(&state, &session, &id, &key, json!({ "decision": "approve" })).await;
    let job = claim(&state).await;
    run_stage_five(&state, &id, &job).await;

    let resp = document(&state, &session, &id).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let features = body["content"]["features"].as_array().unwrap();
    assert_eq!(features.len(), 1);
    let feature = &features[0];
    assert_eq!(feature["key"], key.as_str());
    assert!(feature["name"].as_str().is_some_and(|n| !n.is_empty()));

    // AC2.1: every criterion carries the code location it came from, and that
    // location is a path this analysis actually saw.
    let scenarios = feature["scenarios"].as_array().unwrap();
    assert!(!scenarios.is_empty());
    for scenario in scenarios {
        for field in ["given", "when", "then", "evidence", "source"] {
            assert!(
                scenario[field].as_str().is_some_and(|s| !s.is_empty()),
                "scenario is missing `{field}`: {scenario}"
            );
        }
        let evidence = scenario["evidence"].as_str().unwrap();
        assert!(
            tree().iter().any(|p| p == evidence),
            "scenario cites a path outside the analyzed tree: {evidence}"
        );
    }

    // AC2.2: the test pass added at least one criterion of its own, and every
    // criterion it contributed cites a test file.
    let from_tests: Vec<&serde_json::Value> =
        scenarios.iter().filter(|s| s["source"] == "test").collect();
    assert!(
        !from_tests.is_empty(),
        "the stub tree has test files, so 보강 must have happened"
    );
    for scenario in from_tests {
        assert!(acceptance::is_test_path(scenario["evidence"].as_str().unwrap()));
    }

    // …and a disagreement is kept out of the scenario list, with both sides named.
    let clashes = feature["contradictions"].as_array().unwrap();
    assert!(!clashes.is_empty(), "the stub disagrees with itself once");
    for clash in clashes {
        assert_ne!(clash["codeSays"], clash["testSays"]);
        assert!(acceptance::is_test_path(clash["testEvidence"].as_str().unwrap()));
        assert!(!scenarios.iter().any(|s| s["then"] == clash["testSays"]));
    }

    // Someone else's analysis id is a 404, not a 403 — the API never confirms that
    // an id exists.
    assert_eq!(
        document(&state, &stranger, &id).await.status(),
        StatusCode::NOT_FOUND
    );

    let _ = std::fs::remove_file(&path);
}
