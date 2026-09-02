//! Discovery strategy review, edit and approval (AC1.3).
//!
//! AC1.3's verification method has three clauses, and each has a test here:
//!   · 사용자는 자동 생성된 전략을 **검토**할 수 있다 — reading it materialises the
//!     draft from what stage 3 proposed, and says so before the stage has run.
//!   · **수정**할 수 있다 — deleting and adding are one write of the whole list, and
//!     provenance (generated vs. the reviewer's own) survives the round trip.
//!   · **승인된 전략만 다음 단계의 입력이 된다** — the queue withholds the next stage
//!     until approval. That is the one clause that would otherwise be a promise in
//!     prose, so it is asserted against `/internal/analyses/claim` directly.
//!
//! Plus the carry-over the S06 mockup promises in as many words ("여기서 보탠 항목은
//! 다음 분석에서도 그대로 참조됩니다"): a copy that a screen renders and nothing
//! enforces is a copy that will quietly become false.
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

const WORKER: &str = "w-strategy";

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

/// Returns a claimed job to the queue, the way AC1.5's retry does — the tests below
/// claim to observe what the queue offers, and a held lease would hide the next one.
async fn requeue(state: &AppState, id: &str) {
    sqlx::query("UPDATE analyses SET status = 'queued', lease_expires_at = NULL WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .unwrap();
}

/// The proposal stage 3 stores, submitted the way the worker submits it — which
/// means claiming the job first, because `/internal` writes are lease-guarded.
async fn propose(state: &AppState, id: &str, patterns: &[&str]) {
    let job = claim(state).await;
    assert_eq!(job["id"], id, "claimed a different job than the test meant to");
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
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/documents/discovery_strategy"),
            json!({
                "workerId": WORKER,
                "content": { "entries": entries },
                "model": "stub-model",
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT, "document submit");
}

async fn read(state: &AppState, session: &str, id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/discovery-strategy"), session))
        .await
        .unwrap()
}

async fn put(
    state: &AppState,
    session: &str,
    id: &str,
    patterns: &[&str],
) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(user_send(
            "PUT",
            &format!("/api/analyses/{id}/discovery-strategy/entries"),
            session,
            json!({ "patterns": patterns }),
        ))
        .await
        .unwrap()
}

async fn approve(state: &AppState, session: &str, id: &str) -> axum::response::Response {
    build_router(state.clone())
        .oneshot(user_send(
            "POST",
            &format!("/api/analyses/{id}/discovery-strategy/approve"),
            session,
            json!({}),
        ))
        .await
        .unwrap()
}

fn patterns_of(view: &serde_json::Value) -> Vec<String> {
    view["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["pattern"].as_str().unwrap().to_string())
        .collect()
}

/// Claiming is what a worker does; the assertion is about what the queue *offers*.
async fn offered_stages(state: &AppState) -> Vec<String> {
    claim(state).await["executableStages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_strategy_is_only_readable_once_the_stage_has_proposed_one() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9301, "reviewer").await;
    let id = enqueue(&state, &session, "payments-api").await;

    // 404, not an empty list: "아직 제안되지 않았다" 와 "제안했는데 비었다" 는 다른 상태다.
    assert_eq!(read(&state, &session, &id).await.status(), StatusCode::NOT_FOUND);

    propose(&state, &id, &["src/routes/**", "src/pages/**"]).await;
    let view = json_body(read(&state, &session, &id).await).await;
    assert_eq!(patterns_of(&view), ["src/routes/**", "src/pages/**"]);
    assert_eq!(view["approved"], false);
    assert!(view["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["source"] == "generated"));
}

#[tokio::test]
async fn the_reviewer_deletes_and_adds_and_the_provenance_survives() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9302, "reviewer").await;
    let id = enqueue(&state, &session, "payments-api").await;
    propose(&state, &id, &["src/routes/**", "src/pages/**"]).await;
    read(&state, &session, &id).await; // materialise the draft

    // One deletion and one addition, sent as the list the screen is showing.
    let view = json_body(put(&state, &session, &id, &["src/routes/**", "cmd/admin-cli"]).await).await;
    assert_eq!(patterns_of(&view), ["src/routes/**", "cmd/admin-cli"]);
    assert_eq!(view["entries"][0]["source"], "generated");
    assert_eq!(view["entries"][1]["source"], "user", "보탠 항목은 사용자 것이다");

    // The edit is server state, so a fresh read is the same list.
    let reread = json_body(read(&state, &session, &id).await).await;
    assert_eq!(patterns_of(&reread), patterns_of(&view));

    // Blank and duplicate entries are not a list the reviewer meant to make.
    let deduped =
        json_body(put(&state, &session, &id, &["src/routes/**", "  ", "src/routes/**"]).await).await;
    assert_eq!(patterns_of(&deduped), ["src/routes/**"]);
}

/// AC1.3's gate, asserted where it is enforced rather than where it is described.
#[tokio::test]
async fn the_next_stage_is_withheld_from_the_queue_until_the_strategy_is_approved() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9303, "reviewer").await;
    let id = enqueue(&state, &session, "payments-api").await;
    propose(&state, &id, &["src/routes/**"]).await;
    read(&state, &session, &id).await;

    requeue(&state, &id).await;
    let before = offered_stages(&state).await;
    assert!(before.contains(&"discovery_strategy".to_string()));
    assert!(
        !before.contains(&"feature_candidates".to_string()),
        "승인 전에는 다음 단계가 열리면 안 된다: {before:?}"
    );

    assert_eq!(approve(&state, &session, &id).await.status(), StatusCode::OK);

    // The claim above left the job `running` under a lease; re-queue it so the same
    // job is claimable again and the only difference is the approval.
    requeue(&state, &id).await;
    let after = offered_stages(&state).await;
    assert!(
        after.contains(&"feature_candidates".to_string()),
        "승인 후에는 다음 단계가 열려야 한다: {after:?}"
    );
}

#[tokio::test]
async fn an_approved_strategy_is_frozen_and_an_empty_one_cannot_be_approved() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9304, "reviewer").await;
    let id = enqueue(&state, &session, "payments-api").await;
    propose(&state, &id, &["src/routes/**"]).await;
    read(&state, &session, &id).await;

    // An empty strategy would scan nothing and read as "이 저장소에는 기능이 없다".
    assert_eq!(put(&state, &session, &id, &[]).await.status(), StatusCode::OK);
    assert_eq!(
        approve(&state, &session, &id).await.status(),
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        put(&state, &session, &id, &["src/routes/**"]).await.status(),
        StatusCode::OK
    );
    assert_eq!(approve(&state, &session, &id).await.status(), StatusCode::OK);

    // Frozen afterwards: the approved list is the pipeline's input, and letting it
    // change underneath the next stage would make "이 후보는 어느 전략에서 나왔나"
    // unanswerable.
    assert_eq!(
        put(&state, &session, &id, &["something/else"]).await.status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        approve(&state, &session, &id).await.status(),
        StatusCode::CONFLICT
    );
}

/// The S06 mockup tells the reviewer their additions carry forward. This is that
/// sentence, enforced.
#[tokio::test]
async fn the_reviewers_own_entries_carry_into_the_next_analysis_of_the_same_target() {
    let (state, _dir) = stub_state().await;
    let session = login_installed(&state, 9305, "reviewer").await;

    let first = enqueue(&state, &session, "payments-api").await;
    propose(&state, &first, &["src/routes/**"]).await;
    read(&state, &session, &first).await;
    put(&state, &session, &first, &["src/routes/**", "cmd/admin-cli"]).await;
    assert_eq!(approve(&state, &session, &first).await.status(), StatusCode::OK);

    // A second analysis of the same repository proposes something different; the
    // reviewer's own entry is still there, and the earlier *generated* one is not.
    let second = enqueue(&state, &session, "payments-api").await;
    propose(&state, &second, &["src/handlers/**"]).await;
    let view = json_body(read(&state, &session, &second).await).await;
    assert_eq!(patterns_of(&view), ["src/handlers/**", "cmd/admin-cli"]);
    assert_eq!(view["entries"][1]["source"], "user");

    // A different repository does not inherit it — the promise is per target.
    let other = enqueue(&state, &session, "checkout-web").await;
    propose(&state, &other, &["src/handlers/**"]).await;
    let other_view = json_body(read(&state, &session, &other).await).await;
    assert_eq!(patterns_of(&other_view), ["src/handlers/**"]);
}

/// Owner scope (AC4.7): another user's analysis is 404, not 403 — the API does not
/// confirm that the id exists.
#[tokio::test]
async fn another_users_strategy_is_not_readable_or_writable() {
    let (state, _dir) = stub_state().await;
    let owner = login_installed(&state, 9306, "owner").await;
    let stranger = login_installed(&state, 9307, "stranger").await;
    let id = enqueue(&state, &owner, "payments-api").await;
    propose(&state, &id, &["src/routes/**"]).await;
    read(&state, &owner, &id).await;

    assert_eq!(read(&state, &stranger, &id).await.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        put(&state, &stranger, &id, &["mine/**"]).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        approve(&state, &stranger, &id).await.status(),
        StatusCode::NOT_FOUND
    );

    // …and nothing the stranger sent landed.
    let view = json_body(read(&state, &owner, &id).await).await;
    assert_eq!(patterns_of(&view), ["src/routes/**"]);
    assert_eq!(view["approved"], false);
}
