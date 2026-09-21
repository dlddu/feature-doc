//! 자동 추출이 놓친 feature 의 직접 추가(AC3.2)와 그 출처 보존(AC3.4) — 라우터를 그대로
//! 돌려 본다.
//!
//! 인수 문서는 워커의 `/internal` 경로로 넣는다(형제 테스트와 같은 방침) — 손으로 행을
//! 꽂으면 픽스처가 실제 워커 산출물과 어긋날 수 있고, 그 계약이 바로 여기서 지켜야 할 것이다.
//!
//! 브라우저에서 관측되는 흐름은 `e2e/tests/sc03-03`·`sc03-04` 가 지킨다. 이 파일이 지키는
//! 것은 그 아래의 규칙이다 — 근거는 트리 안 경로뿐 · 근거 없음은 빈 초안 · 확정 전 문서
//! 불변 · 확정 후 끝에 겹침(출처 포함) · 의존성 후보의 행 적재 · 더해진 feature 도 편집
//! 대상 · 남의 분석에는 닿지 않음.
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

const WORKER: &str = "w-feature-add";
const FEATURE: &str = "payments-api/src/api/routes.rs";
/// 스텁 트리의 두 경로(`src/api/routes.rs` · `src/middleware/auth.rs`)에 든 낱말을 품은 문장.
const FOUND: &str = "routes 와 auth 미들웨어를 거치는 회원 탈퇴 기능";
/// 스텁 트리의 어느 경로에도 없는 문장.
const UNKNOWN: &str = "관리자 일괄 삭제 추가";

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

/// 큐에 든 분석 하나 — 5단계가 아직 아무것도 쓰지 않은 상태.
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

/// 인수 문서가 서 있는 분석 하나.
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

async fn draft(
    state: &AppState,
    session: &str,
    id: &str,
    request: &str,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/additions"),
            session,
            json!({ "request": request }),
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
    addition: &str,
    decision: &str,
) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/additions/{addition}/decision"),
            session,
            json!({ "decision": decision }),
        ))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

async fn list(state: &AppState, session: &str, id: &str) -> (StatusCode, serde_json::Value) {
    let resp = build_router(state.clone())
        .oneshot(get(&format!("/api/analyses/{id}/features/additions"), session))
        .await
        .unwrap();
    let status = resp.status();
    (status, json_body(resp).await)
}

fn urlencoding(value: &str) -> String {
    value.replace('/', "%2F").replace(':', "%3A")
}

#[tokio::test]
async fn a_draft_cites_only_paths_of_this_repository_and_confirming_registers_it() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9401, "adder").await;
    let id = analysis_with_document(&state, &s).await;
    assert_eq!(features(&state, &s, &id).await.unwrap().len(), 1);

    let (status, drafted) = draft(&state, &s, &id, FOUND).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(drafted["status"], "drafted");
    assert_eq!(drafted["evidenceFound"], true);
    assert!(drafted["source"].is_null(), "확정 전에는 출처가 없다");
    let key = drafted["key"].as_str().unwrap().to_string();
    assert!(key.starts_with("added:"), "{key}");

    let scenarios = drafted["scenarios"].as_array().unwrap();
    assert_eq!(scenarios.len(), 2, "두 경로를 가리켰으니 초안도 둘이다");
    for scenario in scenarios {
        let evidence = scenario["evidence"].as_str().unwrap();
        assert!(evidence.starts_with("payments-api/"), "{evidence}");
        assert!(evidence.contains("routes") || evidence.contains("auth"), "{evidence}");
    }
    let dependencies = drafted["dependencies"].as_array().unwrap();
    assert!(!dependencies.is_empty());
    for dependency in dependencies {
        assert!(featuredoc::dependencies::is_category(
            dependency["category"].as_str().unwrap()
        ));
    }

    // 확정 전에는 아무것도 바뀌지 않는다.
    assert_eq!(features(&state, &s, &id).await.unwrap().len(), 1);
    let (_, listed) = list(&state, &s, &id).await;
    assert_eq!(listed["finalCount"], 0, "결정 전에는 목록에 세지 않는다");

    let (status, confirmed) =
        decide(&state, &s, &id, drafted["id"].as_str().unwrap(), "confirm").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(confirmed["status"], "confirmed");
    assert_eq!(confirmed["source"], "user_llm", "도움받아 만든 초안의 출처");

    let after = features(&state, &s, &id).await.unwrap();
    assert_eq!(after.len(), 2, "확정했는데 feature 가 늘지 않았다");
    assert_eq!(after[0]["key"], FEATURE, "자동 feature 가 밀려났다");
    assert_eq!(after[1]["key"], key);
    assert_eq!(after[1]["name"], FOUND);
    assert_eq!(after[1]["source"], "user_llm");
    assert_eq!(after[1]["evidenceFound"], true);
    assert_eq!(after[1]["scenarios"].as_array().unwrap().len(), 2);
    assert_eq!(after[1]["scenarios"][0]["source"], "user_llm");
    assert_eq!(after[1]["location"], after[1]["scenarios"][0]["evidence"]);

    let (_, listed) = list(&state, &s, &id).await;
    assert_eq!(listed["confirmedAdditions"], 1);
    assert_eq!(listed["finalCount"], 1);

    // 확정한 의존성 후보는 의존성 화면이 읽는 행으로 실렸다.
    let resp = build_router(state.clone())
        .oneshot(get(
            &format!(
                "/api/analyses/{id}/features/dependencies?key={}",
                urlencoding(&key)
            ),
            &s,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "더해진 feature 도 feature 다");
    let traced = json_body(resp).await;
    assert_eq!(traced["status"], "succeeded");
    assert_eq!(traced["featureName"], FOUND);
    assert_eq!(traced["total"], dependencies.len());
}

#[tokio::test]
async fn nothing_in_the_tree_means_an_empty_draft_and_a_direct_source() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9402, "honest").await;
    let id = analysis_with_document(&state, &s).await;

    let (status, drafted) = draft(&state, &s, &id, UNKNOWN).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(drafted["evidenceFound"], false);
    assert_eq!(drafted["scenarios"], json!([]), "근거 없이 초안을 지어냈다");
    assert_eq!(drafted["dependencies"], json!([]));

    let (status, confirmed) =
        decide(&state, &s, &id, drafted["id"].as_str().unwrap(), "confirm").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(confirmed["source"], "user_direct", "근거 없이 더한 것은 사람이 직접 한 것");

    let after = features(&state, &s, &id).await.unwrap();
    assert_eq!(after.len(), 2);
    assert_eq!(after[1]["name"], UNKNOWN);
    assert_eq!(after[1]["source"], "user_direct");
    assert_eq!(after[1]["evidenceFound"], false);
    assert!(after[1]["location"].is_null(), "근거 없음은 위치가 없는 것이지 지어낸 위치가 아니다");
    assert_eq!(after[1]["scenarios"], json!([]));
}

/// 5단계가 아직 문서를 쓰지 않은 분석에서도 등록은 성립하고, 문서 부재의 뜻은 그대로다.
#[tokio::test]
async fn registration_does_not_need_the_automatic_document() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9403, "early").await;
    let id = analysis(&state, &s).await;
    assert!(features(&state, &s, &id).await.is_none());

    let (status, drafted) = draft(&state, &s, &id, FOUND).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = decide(&state, &s, &id, drafted["id"].as_str().unwrap(), "confirm").await;
    assert_eq!(status, StatusCode::OK);

    let (_, listed) = list(&state, &s, &id).await;
    assert_eq!(listed["finalCount"], 1);
    assert!(
        features(&state, &s, &id).await.is_none(),
        "「아직 생성 전」이 사람이 더했다는 이유로 문서가 되어서는 안 된다"
    );
}

#[tokio::test]
async fn a_cancelled_attempt_stays_out_of_the_document_and_a_decision_is_final() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9404, "undo").await;
    let id = analysis_with_document(&state, &s).await;

    let (_, drafted) = draft(&state, &s, &id, FOUND).await;
    let aid = drafted["id"].as_str().unwrap();
    let (status, cancelled) = decide(&state, &s, &id, aid, "cancel").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");
    assert!(cancelled["source"].is_null());
    assert_eq!(features(&state, &s, &id).await.unwrap().len(), 1);

    let (again, _) = decide(&state, &s, &id, aid, "confirm").await;
    assert_eq!(again, StatusCode::CONFLICT);

    let (status, _) = draft(&state, &s, &id, "   ").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "빈 문장은 트리도 모델도 부르지 않는다");
}

/// 더해진 feature 는 편집(AC3.1)의 대상이기도 하다 — 추가 겹침이 편집 겹침보다 먼저다.
#[tokio::test]
async fn an_added_feature_can_be_edited_like_any_other() {
    let (state, _path) = stub_state().await;
    let s = login_installed(&state, 9405, "editor").await;
    let id = analysis_with_document(&state, &s).await;

    let (_, drafted) = draft(&state, &s, &id, FOUND).await;
    let key = drafted["key"].as_str().unwrap().to_string();
    decide(&state, &s, &id, drafted["id"].as_str().unwrap(), "confirm").await;

    let resp = build_router(state.clone())
        .oneshot(user_post(
            &format!("/api/analyses/{id}/features/edit-proposals"),
            &s,
            json!({ "key": key, "scenarioIndex": 0, "request": "에러 케이스 1개 더 추가" }),
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
            &s,
            json!({ "decision": "approve" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let after = features(&state, &s, &id).await.unwrap();
    let added = after.iter().find(|f| f["key"] == key).unwrap();
    assert_eq!(added["scenarios"].as_array().unwrap().len(), 3, "승인한 편집이 더해진 feature 에 겹쳐지지 않았다");
}

/// AC4.7 — 남의 분석은 없는 것과 같다. 추가도 그 분석에 매인다.
#[tokio::test]
async fn another_users_analysis_cannot_be_added_to() {
    let (state, _path) = stub_state().await;
    let owner = login_installed(&state, 9406, "owner").await;
    let stranger = login_installed(&state, 9407, "stranger").await;
    let id = analysis_with_document(&state, &owner).await;

    let (status, _) = draft(&state, &stranger, &id, FOUND).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = list(&state, &stranger, &id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, drafted) = draft(&state, &owner, &id, FOUND).await;
    let (status, _) = decide(
        &state,
        &stranger,
        &id,
        drafted["id"].as_str().unwrap(),
        "confirm",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
