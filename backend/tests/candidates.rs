//! Feature candidate extraction and review (AC1.4).
//!
//! AC1.4's verification method names four actions and one memory, and each has a
//! test here:
//!   · 후보 목록으로 **제시**된다 — reading materialises the list from what stage 4
//!     produced, and says "not there yet" before the stage has run.
//!   · **승인 / 거부** — a rejection without a reason is refused outright; AC1.4
//!     requires the reason, and a reason written later is a reason nobody wrote.
//!   · **병합 / 이름 변경** — merging keeps the folded rows (visible, reversible) and
//!     renaming does not move the candidate's identity.
//!   · **거부된 후보의 사유는 다음 분석 시 참고될 수 있도록 기록된다** — test/01
//!     시나리오 7, asserted across two analyses of the same target.
//!
//! Plus the two queue properties without which stage 4 could never run at all:
//! approving a strategy re-queues the analysis, and the claim that follows offers
//! **only** the stages that have not already succeeded — otherwise stages 2-3 would
//! re-run their LLM calls and replace the very proposal the reviewer approved.
//!
//! Documents are written through the worker's own `/internal` route, not by
//! inserting rows — a hand-written fixture could drift from what a worker submits.

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

const WORKER: &str = "w-candidates";

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

async fn claim(state: &AppState) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            "/internal/analyses/claim",
            json!({ "workerId": WORKER }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "queue should hold a job");
    json_body(resp).await
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

fn strategy_doc(patterns: &[&str]) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = patterns
        .iter()
        .map(|p| {
            json!({
                "pattern": p,
                "kind": "route",
                "rationale": "stub",
                "evidence": ["src/main.rs"],
            })
        })
        .collect();
    json!({ "entries": entries })
}

fn candidate_doc(items: &[(&str, &str, &str)]) -> serde_json::Value {
    let candidates: Vec<serde_json::Value> = items
        .iter()
        .map(|(name, location, rationale)| {
            json!({ "name": name, "location": location, "rationale": rationale })
        })
        .collect();
    json!({ "candidates": candidates })
}

/// Walks an analysis to "stage 3 succeeded, awaiting the reviewer" — the state every
/// test below starts from, driven through the same routes a worker uses.
async fn run_through_stage_three(state: &AppState, id: &str, patterns: &[&str]) {
    let job = claim(state).await;
    assert_eq!(job["id"], id, "claimed a different job than the test meant to");
    for key in ["fetch", "cross_cutting", "discovery_strategy"] {
        report(state, id, key, "succeeded").await;
    }
    submit(state, id, "cross_cutting", json!({ "categories": [] })).await;
    submit(state, id, "discovery_strategy", strategy_doc(patterns)).await;
    finish(state, id, "awaiting_pipeline").await;
}

async fn approve(state: &AppState, session: &str, id: &str) {
    // The strategy row is materialised on first read (AC1.3's lazy seed), so a
    // reviewer always reads before approving and so does this helper.
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/discovery-strategy"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "strategy read before approval");

    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/discovery-strategy/approve"),
            session,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "approve");
}

/// Runs stage 4 the way the worker does, on the claim that follows approval.
async fn run_stage_four(state: &AppState, id: &str, items: &[(&str, &str, &str)]) {
    let job = claim(state).await;
    assert_eq!(job["id"], id);
    report(state, id, "feature_candidates", "succeeded").await;
    submit(state, id, "feature_candidates", candidate_doc(items)).await;
    finish(state, id, "awaiting_pipeline").await;
}

async fn list(state: &AppState, session: &str, id: &str) -> serde_json::Value {
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
) -> axum::response::Response {
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
}

fn candidate<'a>(view: &'a serde_json::Value, key: &str) -> &'a serde_json::Value {
    view["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["key"] == key)
        .unwrap_or_else(|| panic!("no candidate {key} in {view}"))
}

async fn status_of(state: &AppState, id: &str) -> String {
    sqlx::query_scalar::<_, String>("SELECT status FROM analyses WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .unwrap()
}

const SIFTED: [(&str, &str, &str); 2] = [
    ("비밀번호 재설정", "src/routes/auth.ts", "메일 템플릿과 만료 토큰 검증"),
    ("내부 디버그 콘솔", "src/routes/admin.ts", "관리자 세션에서만 열린다"),
];

#[tokio::test]
async fn a_candidate_list_exists_only_once_the_stage_has_extracted_one() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9401, "sifter").await;
    let id = enqueue(&state, &session, "payments-api").await;

    // Not a 404 and not an error: "아직 추출 전" 은 화면이 그릴 수 있는 상태다.
    let before = list(&state, &session, &id).await;
    assert_eq!(before["extracted"], false);
    assert_eq!(before["undecided"], 0);

    run_through_stage_three(&state, &id, &["src/routes/**"]).await;
    approve(&state, &session, &id).await;
    run_stage_four(&state, &id, &SIFTED).await;

    let after = list(&state, &session, &id).await;
    assert_eq!(after["extracted"], true);
    assert_eq!(after["undecided"], 2);
    let first = candidate(&after, "src/routes/auth.ts");
    assert_eq!(first["name"], "비밀번호 재설정");
    assert_eq!(first["decision"], "undecided");
    // AC1.4: 발견된 위치와 추정 근거가 함께 기록된다.
    assert_eq!(first["location"], "src/routes/auth.ts");
    assert_eq!(first["rationale"], "메일 템플릿과 만료 토큰 검증");
}

#[tokio::test]
async fn a_rejection_without_a_reason_is_refused_and_stores_nothing() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9402, "sifter").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_through_stage_three(&state, &id, &["src/routes/**"]).await;
    approve(&state, &session, &id).await;
    run_stage_four(&state, &id, &SIFTED).await;

    for body in [json!({ "decision": "reject" }), json!({ "decision": "reject", "reason": "   " })] {
        let resp = decide(&state, &session, &id, "src/routes/admin.ts", body).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
    let untouched = list(&state, &session, &id).await;
    assert_eq!(candidate(&untouched, "src/routes/admin.ts")["decision"], "undecided");
    assert_eq!(untouched["undecided"], 2);

    let resp = decide(
        &state,
        &session,
        &id,
        "src/routes/admin.ts",
        json!({ "decision": "reject", "reason": "내부 도구라 사용자 기능이 아님" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let after = json_body(resp).await;
    let rejected = candidate(&after, "src/routes/admin.ts");
    assert_eq!(rejected["decision"], "rejected");
    assert_eq!(rejected["rejectReason"], "내부 도구라 사용자 기능이 아님");
    assert_eq!(after["undecided"], 1);
}

#[tokio::test]
async fn renaming_keeps_the_candidates_identity_and_merging_keeps_the_folded_row() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9403, "sifter").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_through_stage_three(&state, &id, &["src/routes/**"]).await;
    approve(&state, &session, &id).await;
    run_stage_four(&state, &id, &SIFTED).await;

    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/candidates/rename"),
            &session,
            json!({ "key": "src/routes/auth.ts", "name": "비밀번호 찾기" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let renamed = json_body(resp).await;
    let entry = candidate(&renamed, "src/routes/auth.ts");
    assert_eq!(entry["name"], "비밀번호 찾기");
    // The key did not move: identity is where the candidate was found, which is what
    // lets the *next* analysis still recognise a renamed candidate.
    assert_eq!(entry["key"], "src/routes/auth.ts");

    let resp = build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/candidates/merge"),
            &session,
            json!({ "into": "src/routes/auth.ts", "keys": ["src/routes/admin.ts"] }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let merged = json_body(resp).await;
    // Kept, not deleted — the merge stays visible and reversible.
    assert_eq!(
        candidate(&merged, "src/routes/admin.ts")["mergedInto"],
        "src/routes/auth.ts"
    );
    // …and it stops being a thing to decide.
    assert_eq!(merged["undecided"], 1);
}

#[tokio::test]
async fn a_rejected_candidate_comes_back_flagged_in_the_next_analysis_of_the_same_target() {
    // test/01 시나리오 7. The flag is information, never an automatic decision: the
    // mockup is explicit that the reviewer decides again.
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9404, "sifter").await;

    let first = enqueue(&state, &session, "payments-api").await;
    run_through_stage_three(&state, &first, &["src/routes/**"]).await;
    approve(&state, &session, &first).await;
    run_stage_four(&state, &first, &SIFTED).await;
    let resp = decide(
        &state,
        &session,
        &first,
        "src/routes/admin.ts",
        json!({ "decision": "reject", "reason": "내부 도구라 사용자 기능이 아님" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // `created_at` is unix *seconds*, so two analyses triggered together tie; the
    // carry-over query breaks the tie on rowid, and back-dating the first one here
    // exercises the ordering clause rather than the tie-break.
    sqlx::query("UPDATE analyses SET created_at = created_at - 100 WHERE id = ?")
        .bind(&first)
        .execute(&state.db)
        .await
        .unwrap();

    let second = enqueue(&state, &session, "payments-api").await;
    run_through_stage_three(&state, &second, &["src/routes/**"]).await;
    approve(&state, &session, &second).await;
    run_stage_four(&state, &second, &SIFTED).await;

    let view = list(&state, &session, &second).await;
    let carried = candidate(&view, "src/routes/admin.ts");
    assert_eq!(carried["previouslyRejected"]["reason"], "내부 도구라 사용자 기능이 아님");
    assert_eq!(carried["previouslyRejected"]["analysisId"], first.as_str());
    // Not re-decided for the user.
    assert_eq!(carried["decision"], "undecided");
    assert_eq!(view["undecided"], 2);
    // A candidate nobody rejected carries nothing.
    assert!(candidate(&view, "src/routes/auth.ts")["previouslyRejected"].is_null());
}

#[tokio::test]
async fn approving_requeues_the_job_and_the_next_claim_offers_only_what_is_left() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9405, "sifter").await;
    let id = enqueue(&state, &session, "payments-api").await;
    run_through_stage_three(&state, &id, &["src/routes/**", "src/jobs/**"]).await;

    // Before approval the job is parked: stage 4 is withheld (AC1.3's gate) and
    // nothing would ever hand it back to a worker.
    assert_eq!(status_of(&state, &id).await, "awaiting_pipeline");

    approve(&state, &session, &id).await;
    assert_eq!(
        status_of(&state, &id).await,
        "queued",
        "approval must re-queue, or stage 4 opens for a job no claim can reach"
    );

    let job = claim(&state).await;
    let offered: Vec<&str> = job["executableStages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert_eq!(
        offered,
        vec!["fetch", "feature_candidates"],
        "stages 2-3 already succeeded; re-running them would spend the owner's LLM \
         budget again and replace the proposal the reviewer approved"
    );
    assert_eq!(
        job["approvedPatterns"],
        json!(["src/routes/**", "src/jobs/**"]),
        "stage 4's input rides on the claim that opened it"
    );
}

#[tokio::test]
async fn an_approval_that_lands_mid_run_still_gets_its_turn() {
    // The half `approve_strategy` cannot do: it refuses to re-queue a job under a
    // live lease (`retry_stage`'s invariant), so `finish` has to notice.
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9406, "sifter").await;
    let id = enqueue(&state, &session, "payments-api").await;

    claim(&state).await;
    for key in ["fetch", "cross_cutting", "discovery_strategy"] {
        report(&state, &id, key, "succeeded").await;
    }
    submit(&state, &id, "discovery_strategy", strategy_doc(&["src/routes/**"])).await;

    // Approval arrives while the worker still holds the lease.
    approve(&state, &session, &id).await;
    assert_eq!(status_of(&state, &id).await, "running", "the lease holder is untouched");

    finish(&state, &id, "awaiting_pipeline").await;
    assert_eq!(
        status_of(&state, &id).await,
        "queued",
        "an approval that arrived mid-run must not strand stage 4"
    );
}

#[tokio::test]
async fn another_users_candidates_are_not_readable_or_decidable() {
    let (state, _dir) = stub_state().await;
    let owner = login_installed(&state, 9407, "owner").await;
    let stranger = login_installed(&state, 9408, "stranger").await;
    let id = enqueue(&state, &owner, "payments-api").await;
    run_through_stage_three(&state, &id, &["src/routes/**"]).await;
    approve(&state, &owner, &id).await;
    run_stage_four(&state, &id, &SIFTED).await;

    // 404, not 403 — the API never confirms that someone else's id exists (AC4.7).
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/candidates"), &stranger))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let resp = decide(
        &state,
        &stranger,
        &id,
        "src/routes/auth.ts",
        json!({ "decision": "approve" }),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
