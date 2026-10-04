mod common;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use http_body_util::BodyExt;
use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_FIXED};
use serde_json::{json, Value};
use tower::ServiceExt;

use common::push::{start_service, vapid_audience, with_push, Subscriber};
use common::{stub_state, WORKER_TOKEN};
use featuredoc::github_api::GithubUser;
use featuredoc::state::AppState;
use featuredoc::{build_router, installations, session, users};

const WORKER: &str = "w-push";

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

async fn read_key(state: &AppState, session: &str) -> Value {
    let resp = build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri("/api/push/key")
                .header(header::COOKIE, format!("fd_session={session}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    json_body(resp).await
}

async fn subscribe(state: &AppState, session: &str, subscription: Value) -> StatusCode {
    build_router(state.clone())
        .oneshot(user_send("PUT", "/api/push/subscription", session, subscription))
        .await
        .unwrap()
        .status()
}

async fn subscriptions(state: &AppState) -> i64 {
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM push_subscriptions")
        .fetch_one(&state.db)
        .await
        .unwrap();
    n
}

async fn enqueue_and_claim(state: &AppState, session: &str) -> String {
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
    let id = json_body(resp).await["id"].as_str().unwrap().to_string();
    let resp = build_router(state.clone())
        .oneshot(worker_post("/internal/analyses/claim", json!({ "workerId": WORKER })))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    id
}

async fn report(state: &AppState, id: &str, key: &str, status: &str) {
    let resp = build_router(state.clone())
        .oneshot(worker_post(
            &format!("/internal/analyses/{id}/stages/{key}"),
            json!({ "workerId": WORKER, "status": status }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn without_a_vapid_key_there_is_no_key_and_no_subscription() {
    let (state, path) = stub_state().await;
    let session = login_installed(&state, 1, "alice").await;

    assert!(read_key(&state, &session).await["key"].is_null());
    let service = start_service(StatusCode::CREATED).await;
    let subscriber = Subscriber::new(service.endpoint("a"));
    assert_eq!(
        subscribe(&state, &session, subscriber.subscription()).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(subscriptions(&state).await, 0);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_subscription_is_accepted_only_for_a_known_push_service() {
    let (state, path) = stub_state().await;
    let state = with_push(&state);
    let session = login_installed(&state, 2, "bob").await;

    let key = read_key(&state, &session).await;
    let key = URL_SAFE_NO_PAD.decode(key["key"].as_str().unwrap()).unwrap();
    assert_eq!(key.len(), 65);

    let elsewhere = Subscriber::new("https://push.example.invalid/x".into());
    assert_eq!(
        subscribe(&state, &session, elsewhere.subscription()).await,
        StatusCode::BAD_REQUEST
    );
    let mut broken = elsewhere.subscription();
    broken["endpoint"] = json!("http://127.0.0.1:1/push/b");
    broken["keys"]["auth"] = json!("AAAA");
    assert_eq!(subscribe(&state, &session, broken).await, StatusCode::BAD_REQUEST);

    let service = start_service(StatusCode::CREATED).await;
    let subscriber = Subscriber::new(service.endpoint("c"));
    assert_eq!(
        subscribe(&state, &session, subscriber.subscription()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        subscribe(&state, &session, subscriber.subscription()).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(subscriptions(&state).await, 1);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_finished_stage_reaches_the_owner_as_an_encrypted_push() {
    let (state, path) = stub_state().await;
    let state = with_push(&state);
    let session = login_installed(&state, 3, "carol").await;
    let service = start_service(StatusCode::CREATED).await;
    let subscriber = Subscriber::new(service.endpoint("carol"));
    assert_eq!(
        subscribe(&state, &session, subscriber.subscription()).await,
        StatusCode::NO_CONTENT
    );

    let id = enqueue_and_claim(&state, &session).await;
    report(&state, &id, "fetch", "running").await;
    report(&state, &id, "fetch", "succeeded").await;

    let received = service.wait_for(1).await;
    assert_eq!(received.len(), 1, "running is not announced, succeeded is");
    let message = &received[0];
    assert_eq!(message.path, "/push/carol");
    assert_eq!(message.headers["content-encoding"], "aes128gcm");
    assert_eq!(message.headers["ttl"], "86400");

    let authorization = message.headers["authorization"].to_str().unwrap();
    assert_eq!(vapid_audience(authorization), service.origin);
    let (token, k) = authorization
        .strip_prefix("vapid t=")
        .unwrap()
        .split_once(", k=")
        .unwrap();
    let (signed, signature) = token.rsplit_once('.').unwrap();
    UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, URL_SAFE_NO_PAD.decode(k).unwrap())
        .verify(signed.as_bytes(), &URL_SAFE_NO_PAD.decode(signature).unwrap())
        .expect("the VAPID token is signed by the advertised key");
    assert_eq!(
        URL_SAFE_NO_PAD.encode(URL_SAFE_NO_PAD.decode(k).unwrap()),
        read_key(&state, &session).await["key"].as_str().unwrap()
    );

    let payload = subscriber.open(&message.body);
    assert_eq!(
        payload,
        json!({
            "title": "stub-account/payments-api",
            "body": "저장소 내려받기 단계가 끝났어요.",
            "tag": id,
        })
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_failed_stage_says_only_that_stage_needs_a_retry() {
    let (state, path) = stub_state().await;
    let state = with_push(&state);
    let session = login_installed(&state, 4, "dave").await;
    let service = start_service(StatusCode::CREATED).await;
    let subscriber = Subscriber::new(service.endpoint("dave"));
    subscribe(&state, &session, subscriber.subscription()).await;

    let id = enqueue_and_claim(&state, &session).await;
    report(&state, &id, "fetch", "failed").await;

    let received = service.wait_for(1).await;
    assert_eq!(received.len(), 1);
    assert_eq!(
        subscriber.open(&received[0].body)["body"],
        "저장소 내려받기 단계가 실패했어요. 앞 단계 결과는 그대로 있으니 이 단계만 다시 돌리면 됩니다."
    );

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn a_gone_subscription_is_forgotten() {
    let (state, path) = stub_state().await;
    let state = with_push(&state);
    let session = login_installed(&state, 5, "erin").await;
    let service = start_service(StatusCode::GONE).await;
    let subscriber = Subscriber::new(service.endpoint("erin"));
    subscribe(&state, &session, subscriber.subscription()).await;
    assert_eq!(subscriptions(&state).await, 1);

    let id = enqueue_and_claim(&state, &session).await;
    featuredoc::push::notify_stage(&state, &id, "fetch", "succeeded")
        .await
        .unwrap();

    assert_eq!(service.count(), 1);
    assert_eq!(subscriptions(&state).await, 0);

    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn another_user_s_analysis_is_not_pushed_to_me() {
    let (state, path) = stub_state().await;
    let state = with_push(&state);
    let mine = login_installed(&state, 6, "frank").await;
    let theirs = login_installed(&state, 7, "grace").await;
    let service = start_service(StatusCode::CREATED).await;
    let subscriber = Subscriber::new(service.endpoint("frank"));
    subscribe(&state, &mine, subscriber.subscription()).await;

    let id = enqueue_and_claim(&state, &theirs).await;
    featuredoc::push::notify_stage(&state, &id, "fetch", "succeeded")
        .await
        .unwrap();
    assert_eq!(service.count(), 0);

    let _ = std::fs::remove_file(path);
}
