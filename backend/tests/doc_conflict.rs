//! 자동 재분석과 사용자 편집의 충돌 처리(AC3.5) — 라우터를 그대로 돌려 본다.
//!
//! 두 분석은 같은 타깃이고, 두 번째 분석의 자동 문서는 워커가 넣는 값이라 여기서
//! 「코드가 바뀌어 같은 시나리오를 다르게 읽었다」를 문장 하나로 재현한다.
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

const WORKER: &str = "w-doc-conflict";
const FEATURE: &str = "payments-api/src/api/routes.rs";
const FIRST_THEN: &str = "저장되지 않습니다";
const REWRITTEN_THEN: &str = "저장되지 않고 만료일을 고치라고 안내합니다";

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

/// 같은 타깃의 분석 하나를 큐에 넣고 워커가 집게 한 뒤, 첫 시나리오의 `then` 이
/// `first_then` 인 인수 문서를 세운다.
async fn analysis_with_document(state: &AppState, session: &str, first_then: &str) -> String {
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
    submit_document(state, &id, first_then).await;
    id
}

async fn submit_document(state: &AppState, id: &str, first_then: &str) {
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
                            "then": first_then,
                            "evidence": FEATURE,
                            "symbol": "pay",
                            "source": "logic",
                        }, {
                            "given": "카드를 이미 등록한 사용자가",
                            "when": "같은 카드를 다시 저장하면",
                            "then": "중복으로 저장되지 않습니다",
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
}

async fn thens(state: &AppState, session: &str, id: &str) -> Vec<String> {
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
        .iter()
        .map(|s| s["then"].as_str().unwrap().to_string())
        .collect()
}

/// 첫 시나리오를 한 줄 부탁으로 고쳐 쓰고 승인한다 — 사용자의 마지막 편집.
async fn edit_first_scenario(state: &AppState, session: &str, id: &str) -> String {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals"),
            session,
            json!({ "key": FEATURE, "scenarioIndex": 0, "request": "더 분명하게" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let proposal = json_body(resp).await;
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!(
                "/api/analyses/{id}/features/edit-proposals/{}/decision",
                proposal["id"].as_str().unwrap()
            ),
            session,
            json!({ "decision": "approve" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    proposal["after"][0]["then"].as_str().unwrap().to_string()
}

async fn conflicts(state: &AppState, session: &str, id: &str) -> serde_json::Value {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/conflicts"), session))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

async fn post(
    state: &AppState,
    session: &str,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(uri, session, body))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

#[tokio::test]
async fn an_edit_whose_sentence_survived_the_reanalysis_is_carried_over() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9601, "carrier").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    let mine = edit_first_scenario(&state, &s, &first).await;

    let second = analysis_with_document(&state, &s, FIRST_THEN).await;
    assert_eq!(
        thens(&state, &s, &second).await[0],
        mine,
        "자동 결과가 그대로인데 사용자 문장이 새 분석에서 사라졌다"
    );
    assert_eq!(conflicts(&state, &s, &second).await["open"], 0);

    // 다시 저장돼도(단계 재시도) 이월은 한 번이다.
    submit_document(&state, &second, FIRST_THEN).await;
    let doc = thens(&state, &s, &second).await;
    assert_eq!(doc.len(), 2);
    assert_eq!(doc[0], mine);
}

#[tokio::test]
async fn a_rewritten_sentence_opens_a_conflict_and_overwrites_nothing() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9602, "collider").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    let mine = edit_first_scenario(&state, &s, &first).await;

    let second = analysis_with_document(&state, &s, REWRITTEN_THEN).await;

    let doc = thens(&state, &s, &second).await;
    assert_eq!(doc[0], REWRITTEN_THEN, "열린 충돌 동안 문서에는 자동 결과가 선다");
    assert!(!doc.contains(&mine), "결정 없이 사용자 문장이 얹혔다");
    assert_eq!(
        thens(&state, &s, &first).await[0],
        mine,
        "앞선 분석의 사용자 문장은 이력으로 남는다"
    );

    let listed = conflicts(&state, &s, &second).await;
    assert_eq!(listed["open"], 1);
    let conflict = &listed["conflicts"][0];
    assert_eq!(conflict["status"], "open");
    assert_eq!(conflict["featureKey"], FEATURE);
    assert_eq!(conflict["featureName"], "결제 수단 등록");
    assert_eq!(conflict["scenarioIndex"], 0);
    assert_eq!(conflict["mine"][0]["then"], mine, "사람이 고친 쪽");
    assert_eq!(conflict["auto"]["then"], REWRITTEN_THEN, "자동 결과 쪽");
    assert_eq!(conflict["before"]["then"], FIRST_THEN);
    assert_eq!(conflict["source"], "user_llm");
    assert_eq!(conflict["previousAnalysisId"], first);
    assert!(conflict["mergeProposal"].is_null());
}

#[tokio::test]
async fn keeping_mine_puts_the_readers_sentence_back_with_a_direct_source() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9603, "keeper").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    let mine = edit_first_scenario(&state, &s, &first).await;
    let second = analysis_with_document(&state, &s, REWRITTEN_THEN).await;
    let cid = conflicts(&state, &s, &second).await["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, decided) = post(
        &state,
        &s,
        &format!("/api/analyses/{second}/conflicts/{cid}/decision"),
        json!({ "decision": "mine" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decided["status"], "mine");
    assert!(decided["decidedAt"].is_i64());

    let doc = thens(&state, &s, &second).await;
    assert_eq!(doc[0], mine, "「내 문장 그대로 두기」인데 사용자 문장이 서지 않았다");
    assert_eq!(doc.len(), 2);
    assert_eq!(conflicts(&state, &s, &second).await["open"], 0);

    let sources: Vec<(String, String)> = sqlx::query_as(
        "SELECT source, before_json FROM feature_doc_edits \
          WHERE analysis_id = ? AND status = 'approved'",
    )
    .bind(&second)
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].0, "user_direct", "덮어쓰기를 막은 결정은 사람이 직접 내린 것이다");
    assert!(sources[0].1.contains(REWRITTEN_THEN), "그 결정이 무엇을 밀어냈는지도 이력이다");

    let (again, _) = post(
        &state,
        &s,
        &format!("/api/analyses/{second}/conflicts/{cid}/decision"),
        json!({ "decision": "auto" }),
    )
    .await;
    assert_eq!(again, StatusCode::CONFLICT, "결정된 충돌을 다시 결정할 수 없다");
}

#[tokio::test]
async fn taking_the_auto_result_closes_the_conflict_and_changes_nothing() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9604, "yielder").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    edit_first_scenario(&state, &s, &first).await;
    let second = analysis_with_document(&state, &s, REWRITTEN_THEN).await;
    let cid = conflicts(&state, &s, &second).await["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, decided) = post(
        &state,
        &s,
        &format!("/api/analyses/{second}/conflicts/{cid}/decision"),
        json!({ "decision": "auto" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decided["status"], "auto");
    assert_eq!(thens(&state, &s, &second).await[0], REWRITTEN_THEN);
    assert_eq!(conflicts(&state, &s, &second).await["open"], 0);
}

#[tokio::test]
async fn merging_is_a_proposal_that_only_the_reader_can_turn_into_a_decision() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9605, "merger").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    let mine = edit_first_scenario(&state, &s, &first).await;
    let second = analysis_with_document(&state, &s, REWRITTEN_THEN).await;
    let cid = conflicts(&state, &s, &second).await["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let base = format!("/api/analyses/{second}/conflicts/{cid}");

    let (status, proposed) = post(&state, &s, &format!("{base}/merge"), json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(proposed["status"], "open", "제안은 결정이 아니다");
    let merged = proposed["mergeProposal"]["after"][0]["then"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(merged.contains(&mine), "내가 보탠 내용이 합친 문장에서 빠졌다");
    assert!(merged.contains(REWRITTEN_THEN), "새 규칙이 합친 문장에 얹히지 않았다");
    assert_eq!(proposed["mergeProposal"]["status"], "proposed");
    assert_eq!(
        thens(&state, &s, &second).await[0],
        REWRITTEN_THEN,
        "승인하지 않았는데 문서가 바뀌었다"
    );

    // 같은 제안을 다시 부르면 모델을 부르지 않고 그 제안을 돌려준다.
    let (_, same) = post(&state, &s, &format!("{base}/merge"), json!({})).await;
    assert_eq!(same["mergeProposal"]["id"], proposed["mergeProposal"]["id"]);

    let (status, rejected) = post(
        &state,
        &s,
        &format!("{base}/merge-decision"),
        json!({ "decision": "reject" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rejected["status"], "open", "합친 문장을 버려도 충돌은 미해소로 남는다");
    assert!(rejected["mergeProposal"].is_null());
    assert_eq!(conflicts(&state, &s, &second).await["open"], 1);

    let (_, again) = post(&state, &s, &format!("{base}/merge"), json!({ "keepMine": true })).await;
    let second_merge = again["mergeProposal"]["after"][0]["then"].as_str().unwrap();
    assert_ne!(second_merge, merged, "버린 합친 문장이 그대로 다시 왔다");

    let (status, done) = post(
        &state,
        &s,
        &format!("{base}/merge-decision"),
        json!({ "decision": "approve" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(done["status"], "merged");
    assert_eq!(done["mergeProposal"]["status"], "approved");
    let doc = thens(&state, &s, &second).await;
    assert_eq!(doc[0], second_merge, "확정한 합친 문장이 문서에 서지 않았다");
    assert_eq!(doc.len(), 2);
    assert_eq!(conflicts(&state, &s, &second).await["open"], 0);

    let (status, _) = post(
        &state,
        &s,
        &format!("{base}/merge-decision"),
        json!({ "decision": "approve" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn an_undecided_conflict_stands_again_after_the_next_reanalysis() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9606, "leaver").await;
    let first = analysis_with_document(&state, &s, FIRST_THEN).await;
    let mine = edit_first_scenario(&state, &s, &first).await;
    let second = analysis_with_document(&state, &s, REWRITTEN_THEN).await;
    assert_eq!(conflicts(&state, &s, &second).await["open"], 1);

    let third = analysis_with_document(&state, &s, REWRITTEN_THEN).await;
    let listed = conflicts(&state, &s, &third).await;
    assert_eq!(listed["open"], 1, "결정하지 않은 충돌이 다음 재분석에서 사라졌다");
    assert_eq!(listed["conflicts"][0]["mine"][0]["then"], mine);
    assert_eq!(listed["conflicts"][0]["previousAnalysisId"], second);
    assert_eq!(thens(&state, &s, &third).await[0], REWRITTEN_THEN);
}

#[tokio::test]
async fn another_users_conflicts_are_out_of_reach() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 9607, "owner").await;
    let stranger = login_installed(&state, 9608, "stranger").await;
    let first = analysis_with_document(&state, &owner, FIRST_THEN).await;
    edit_first_scenario(&state, &owner, &first).await;
    let second = analysis_with_document(&state, &owner, REWRITTEN_THEN).await;
    let cid = conflicts(&state, &owner, &second).await["conflicts"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{second}/conflicts"), &stranger))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let (status, _) = post(
        &state,
        &stranger,
        &format!("/api/analyses/{second}/conflicts/{cid}/decision"),
        json!({ "decision": "auto" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
