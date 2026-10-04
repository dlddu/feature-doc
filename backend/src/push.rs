//! Web Push: 브라우저 구독(엔드포인트·키)을 사용자별로 저장하고, 분석 단계가 끝나거나 문서가 바뀌면 그 구독으로 알린다.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hkdf::Hkdf;
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256};
use ring::rand::{SecureRandom, SystemRandom};
use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::pipeline::{self, stage_status};
use crate::state::AppState;
use crate::util::now_unix;

const RECORD_SIZE: u32 = 4096;
const TTL_SECONDS: u32 = 86_400;
const JWT_LIFETIME_SECONDS: i64 = 12 * 3600;
const SEND_TIMEOUT_SECONDS: u64 = 10;

pub const DEFAULT_SERVICES: [&str; 4] = [
    "fcm.googleapis.com",
    "push.services.mozilla.com",
    "push.apple.com",
    "notify.windows.com",
];

#[derive(Clone)]
pub struct PushConfig {
    pub private_key_pkcs8: Vec<u8>,
    pub subject: String,
    pub services: Vec<String>,
}

impl PushConfig {
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("FEATUREDOC_VAPID_PRIVATE_KEY").ok()?;
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        let Ok(private_key_pkcs8) = decode_b64(key) else {
            tracing::warn!("FEATUREDOC_VAPID_PRIVATE_KEY is not base64url — push delivery is off");
            return None;
        };
        let config = Self {
            private_key_pkcs8,
            subject: std::env::var("FEATUREDOC_VAPID_SUBJECT")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "mailto:featuredoc@localhost".into()),
            services: match std::env::var("FEATUREDOC_PUSH_SERVICES") {
                Ok(list) if !list.trim().is_empty() => list
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                _ => DEFAULT_SERVICES.iter().map(|s| s.to_string()).collect(),
            },
        };
        if config.public_key().is_none() {
            tracing::warn!("FEATUREDOC_VAPID_PRIVATE_KEY is not a P-256 PKCS#8 key — push delivery is off");
            return None;
        }
        Some(config)
    }

    fn key_pair(&self) -> Option<EcdsaKeyPair> {
        EcdsaKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            &self.private_key_pkcs8,
            &SystemRandom::new(),
        )
        .ok()
    }

    pub fn public_key(&self) -> Option<String> {
        self.key_pair()
            .map(|pair| URL_SAFE_NO_PAD.encode(pair.public_key().as_ref()))
    }

    pub fn accepts(&self, endpoint: &str) -> bool {
        let Ok(url) = url::Url::parse(endpoint) else {
            return false;
        };
        let Some(host) = url.host_str() else {
            return false;
        };
        let loopback = host == "127.0.0.1" || host == "localhost";
        if url.scheme() != "https" && !loopback {
            return false;
        }
        self.services
            .iter()
            .any(|s| host == s || host.ends_with(&format!(".{s}")))
    }
}

pub fn generate_private_key() -> String {
    let doc = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &SystemRandom::new())
        .expect("P-256 key generation");
    URL_SAFE_NO_PAD.encode(doc.as_ref())
}

fn decode_b64(value: &str) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(value.trim().trim_end_matches('='))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/push/key", get(key))
        .route("/api/push/subscription", put(subscribe))
}

#[derive(Serialize)]
struct KeyView {
    key: Option<String>,
}

async fn key(State(state): State<AppState>, CurrentUser(_user): CurrentUser) -> Json<KeyView> {
    Json(KeyView {
        key: state.config.push.as_ref().and_then(PushConfig::public_key),
    })
}

#[derive(Deserialize)]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Deserialize)]
pub struct SubscriptionReq {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

async fn subscribe(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<SubscriptionReq>,
) -> Result<StatusCode, AppError> {
    let Some(config) = state.config.push.as_ref() else {
        return Err(AppError::NotFound);
    };
    if !config.accepts(&req.endpoint) {
        return Err(AppError::BadRequest("unsupported push service".into()));
    }
    let p256dh = decode_b64(&req.keys.p256dh)
        .ok()
        .filter(|k| k.len() == 65 && k[0] == 0x04)
        .ok_or_else(|| AppError::BadRequest("invalid p256dh key".into()))?;
    let auth = decode_b64(&req.keys.auth)
        .ok()
        .filter(|a| a.len() == 16)
        .ok_or_else(|| AppError::BadRequest("invalid auth secret".into()))?;

    sqlx::query(
        "INSERT INTO push_subscriptions (endpoint, user_id, p256dh, auth, created_at) \
         VALUES (?, ?, ?, ?, ?) \
         ON CONFLICT(endpoint) DO UPDATE SET \
           user_id = excluded.user_id, p256dh = excluded.p256dh, auth = excluded.auth",
    )
    .bind(&req.endpoint)
    .bind(&user.id)
    .bind(URL_SAFE_NO_PAD.encode(&p256dh))
    .bind(URL_SAFE_NO_PAD.encode(&auth))
    .bind(now_unix())
    .execute(&state.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Message {
    pub title: String,
    pub body: String,
    pub tag: String,
}

pub fn stage_label(key: &str) -> &'static str {
    match key {
        pipeline::FETCH => "저장소 내려받기",
        pipeline::CROSS_CUTTING => "횡단 관심사 추출",
        pipeline::DISCOVERY_STRATEGY => "탐색 전략 생성",
        pipeline::FEATURE_CANDIDATES => "feature 후보 추출",
        pipeline::ACCEPTANCE_DEPENDENCIES => "인수 시나리오 생성",
        _ => "분석",
    }
}

pub fn change_body(changed: usize, open_conflicts: usize) -> Option<String> {
    if changed == 0 {
        return None;
    }
    let conflicts = if open_conflicts == 0 {
        "부딪히는 편집은 없어요.".to_string()
    } else {
        format!("부딪히는 편집이 {open_conflicts}건 있습니다.")
    };
    Some(format!("{changed}개 기능의 표현이 갱신됐어요. {conflicts}"))
}

pub fn stage_body(key: &str, status: &str) -> Option<String> {
    let label = stage_label(key);
    match status {
        stage_status::SUCCEEDED => Some(format!("{label} 단계가 끝났어요.")),
        stage_status::FAILED => Some(format!(
            "{label} 단계가 실패했어요. 앞 단계 결과는 그대로 있으니 이 단계만 다시 돌리면 됩니다."
        )),
        _ => None,
    }
}

pub fn stage_reported(state: &AppState, analysis_id: &str, key: &str, status: &str) {
    if state.config.push.is_none() {
        return;
    }
    if !matches!(status, stage_status::SUCCEEDED | stage_status::FAILED) {
        return;
    }
    let state = state.clone();
    let analysis_id = analysis_id.to_string();
    let key = key.to_string();
    let status = status.to_string();
    tokio::spawn(async move {
        if let Err(e) = notify_stage(&state, &analysis_id, &key, &status).await {
            tracing::warn!(analysis_id = %analysis_id, stage = %key, error = ?e, "push notification not sent");
        }
    });
}

pub async fn notify_stage(
    state: &AppState,
    analysis_id: &str,
    key: &str,
    status: &str,
) -> Result<(), AppError> {
    let target: Option<(String, String, String)> = sqlx::query_as(
        "SELECT user_id, repo_owner, repo_name FROM analyses WHERE id = ?",
    )
    .bind(analysis_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((user_id, owner, name)) = target else {
        return Ok(());
    };

    let rerun = crate::analysis::diff_view(state, &user_id, analysis_id).await?;
    let body = match (rerun.compared_to.is_some(), status) {
        (true, stage_status::SUCCEEDED) if key == pipeline::ACCEPTANCE_DEPENDENCIES => {
            let (open,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM feature_doc_conflicts WHERE analysis_id = ? AND status = ?",
            )
            .bind(analysis_id)
            .bind(crate::doc_conflict::status::OPEN)
            .fetch_one(&state.db)
            .await?;
            change_body(rerun.features.len(), open as usize)
        }
        (true, stage_status::SUCCEEDED) => None,
        _ => stage_body(key, status),
    };
    let Some(body) = body else {
        return Ok(());
    };
    send_to_user(
        state,
        &user_id,
        &Message {
            title: format!("{owner}/{name}"),
            body,
            tag: analysis_id.to_string(),
        },
    )
    .await
}

pub async fn send_to_user(state: &AppState, user_id: &str, message: &Message) -> Result<(), AppError> {
    let Some(config) = state.config.push.as_ref() else {
        return Ok(());
    };
    let subscriptions: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT endpoint, p256dh, auth FROM push_subscriptions WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let payload = serde_json::to_vec(message).map_err(|e| AppError::internal(e.to_string()))?;

    for (endpoint, p256dh, auth) in subscriptions {
        match deliver(state, config, &endpoint, &p256dh, &auth, &payload).await {
            Ok(status) if status == 404 || status == 410 => {
                sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = ?")
                    .bind(&endpoint)
                    .execute(&state.db)
                    .await?;
            }
            Ok(status) if !(200..300).contains(&status) => {
                tracing::warn!(status, "push service refused the message");
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "push delivery failed"),
        }
    }
    Ok(())
}

async fn deliver(
    state: &AppState,
    config: &PushConfig,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    payload: &[u8],
) -> anyhow::Result<u16> {
    if !config.accepts(endpoint) {
        anyhow::bail!("endpoint is not an accepted push service");
    }
    let ua_public = decode_b64(p256dh)?;
    let auth = decode_b64(auth)?;
    let body = encrypt(&ua_public, &auth, payload)?;
    let authorization = vapid_authorization(config, endpoint)?;
    let resp = state
        .http
        .post(endpoint)
        .timeout(std::time::Duration::from_secs(SEND_TIMEOUT_SECONDS))
        .header("TTL", TTL_SECONDS.to_string())
        .header("Content-Encoding", "aes128gcm")
        .header("Content-Type", "application/octet-stream")
        .header("Urgency", "normal")
        .header("Authorization", authorization)
        .body(body)
        .send()
        .await?;
    Ok(resp.status().as_u16())
}

pub fn encrypt(ua_public: &[u8], auth: &[u8], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
    let rng = SystemRandom::new();
    let private = EphemeralPrivateKey::generate(&ECDH_P256, &rng)
        .map_err(|_| anyhow::anyhow!("ephemeral key generation failed"))?;
    let as_public = private
        .compute_public_key()
        .map_err(|_| anyhow::anyhow!("ephemeral public key failed"))?
        .as_ref()
        .to_vec();
    let ecdh_secret = agree_ephemeral(private, &UnparsedPublicKey::new(&ECDH_P256, ua_public), |s| {
        s.to_vec()
    })
    .map_err(|_| anyhow::anyhow!("invalid subscription key"))?;
    let mut salt = [0u8; 16];
    rng.fill(&mut salt)
        .map_err(|_| anyhow::anyhow!("salt generation failed"))?;
    seal(&ecdh_secret, ua_public, &as_public, auth, &salt, plaintext)
}

pub fn seal(
    ecdh_secret: &[u8],
    ua_public: &[u8],
    as_public: &[u8],
    auth: &[u8],
    salt: &[u8; 16],
    plaintext: &[u8],
) -> anyhow::Result<Vec<u8>> {
    if plaintext.len() + 1 + 16 + 86 > RECORD_SIZE as usize {
        anyhow::bail!("push payload does not fit one record");
    }
    let mut key_info = b"WebPush: info\0".to_vec();
    key_info.extend_from_slice(ua_public);
    key_info.extend_from_slice(as_public);
    let mut ikm = [0u8; 32];
    Hkdf::<Sha256>::new(Some(auth), ecdh_secret)
        .expand(&key_info, &mut ikm)
        .map_err(|_| anyhow::anyhow!("hkdf expand failed"))?;

    let prk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let mut cek = [0u8; 16];
    prk.expand(b"Content-Encoding: aes128gcm\0", &mut cek)
        .map_err(|_| anyhow::anyhow!("hkdf expand failed"))?;
    let mut nonce = [0u8; 12];
    prk.expand(b"Content-Encoding: nonce\0", &mut nonce)
        .map_err(|_| anyhow::anyhow!("hkdf expand failed"))?;

    let mut record = plaintext.to_vec();
    record.push(0x02);
    let ciphertext = Aes128Gcm::new_from_slice(&cek)
        .map_err(|_| anyhow::anyhow!("bad content key"))?
        .encrypt(Nonce::from_slice(&nonce), record.as_slice())
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;

    let mut out = Vec::with_capacity(86 + ciphertext.len());
    out.extend_from_slice(salt);
    out.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    out.push(as_public.len() as u8);
    out.extend_from_slice(as_public);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn vapid_authorization(config: &PushConfig, endpoint: &str) -> anyhow::Result<String> {
    let url = url::Url::parse(endpoint)?;
    let audience = url.origin().ascii_serialization();
    let pair = config
        .key_pair()
        .ok_or_else(|| anyhow::anyhow!("VAPID key unreadable"))?;
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ES256"}"#);
    let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&serde_json::json!({
        "aud": audience,
        "exp": now_unix() + JWT_LIFETIME_SECONDS,
        "sub": config.subject,
    }))?);
    let signing_input = format!("{header}.{claims}");
    let signature = pair
        .sign(&SystemRandom::new(), signing_input.as_bytes())
        .map_err(|_| anyhow::anyhow!("VAPID signing failed"))?;
    Ok(format!(
        "vapid t={signing_input}.{}, k={}",
        URL_SAFE_NO_PAD.encode(signature.as_ref()),
        URL_SAFE_NO_PAD.encode(pair.public_key().as_ref())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b64(value: &str) -> Vec<u8> {
        decode_b64(&value.replace(char::is_whitespace, "")).unwrap()
    }

    #[test]
    fn rfc8291_appendix_a_vector() {
        let salt: [u8; 16] = b64("DGv6ra1nlYgDCS1FRnbzlw").try_into().unwrap();
        let sealed = seal(
            &b64("kyrL1jIIOHEzg3sM2ZWRHDRB62YACZhhSlknJ672kSs"),
            &b64("BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcx aOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4"),
            &b64("BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIg Dll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8"),
            &b64("BTBZMqHH6r4Tts7J_aSIgg"),
            &salt,
            b"When I grow up, I want to be a watermelon",
        )
        .unwrap();
        assert_eq!(
            URL_SAFE_NO_PAD.encode(sealed),
            "DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27ml\
             mlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A_yl95bQpu6cVPT\
             pK4Mqgkf1CXztLVBSt2Ks3oZwbuwXPXLWyouBWLVWGNWQexSgSxsj_Qulcy4a-fN"
        );
    }

    #[test]
    fn no_change_means_no_message() {
        assert_eq!(change_body(0, 3), None);
        assert_eq!(
            change_body(2, 0).as_deref(),
            Some("2개 기능의 표현이 갱신됐어요. 부딪히는 편집은 없어요.")
        );
        assert_eq!(
            change_body(1, 2).as_deref(),
            Some("1개 기능의 표현이 갱신됐어요. 부딪히는 편집이 2건 있습니다.")
        );
    }

    #[test]
    fn only_finished_stages_are_announced() {
        assert_eq!(stage_body(pipeline::FETCH, stage_status::RUNNING), None);
        assert_eq!(
            stage_body(pipeline::DISCOVERY_STRATEGY, stage_status::SUCCEEDED).as_deref(),
            Some("탐색 전략 생성 단계가 끝났어요.")
        );
        assert!(stage_body(pipeline::FETCH, stage_status::FAILED)
            .unwrap()
            .starts_with("저장소 내려받기 단계가 실패했어요."));
    }

    #[test]
    fn only_known_push_services_are_accepted() {
        let config = PushConfig {
            private_key_pkcs8: Vec::new(),
            subject: "mailto:a@b".into(),
            services: DEFAULT_SERVICES.iter().map(|s| s.to_string()).collect(),
        };
        assert!(config.accepts("https://fcm.googleapis.com/fcm/send/abc"));
        assert!(config.accepts("https://web.push.apple.com/QGx"));
        assert!(!config.accepts("http://fcm.googleapis.com/fcm/send/abc"));
        assert!(!config.accepts("https://evil.example/fcm.googleapis.com"));
        assert!(!config.accepts("https://notfcm.googleapis.com.evil.example/"));
        assert!(!config.accepts("not a url"));
    }

    #[test]
    fn a_generated_key_round_trips_to_a_public_key() {
        let config = PushConfig {
            private_key_pkcs8: decode_b64(&generate_private_key()).unwrap(),
            subject: "mailto:a@b".into(),
            services: Vec::new(),
        };
        let public = decode_b64(&config.public_key().unwrap()).unwrap();
        assert_eq!(public.len(), 65);
        assert_eq!(public[0], 0x04);
    }
}
