use std::sync::{Arc, Mutex};
use std::time::Duration;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, KeyInit, Nonce};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hkdf::Hkdf;
use ring::agreement::{agree_ephemeral, EphemeralPrivateKey, UnparsedPublicKey, ECDH_P256};
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::{json, Value};
use sha2::Sha256;

use featuredoc::config::Config;
use featuredoc::push::{generate_private_key, PushConfig};
use featuredoc::state::AppState;

#[derive(Clone)]
pub struct Received {
    pub path: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

#[derive(Clone)]
pub struct PushService {
    pub origin: String,
    pub received: Arc<Mutex<Vec<Received>>>,
}

#[derive(Clone)]
struct ServiceState {
    received: Arc<Mutex<Vec<Received>>>,
    status: StatusCode,
}

async fn accept(
    State(service): State<ServiceState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    service.received.lock().unwrap().push(Received {
        path: format!("/push/{id}"),
        headers,
        body: body.to_vec(),
    });
    service.status
}

pub async fn start_service(status: StatusCode) -> PushService {
    let received = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .route("/push/{id}", post(accept))
        .with_state(ServiceState {
            received: received.clone(),
            status,
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    PushService {
        origin: format!("http://{addr}"),
        received,
    }
}

impl PushService {
    pub fn endpoint(&self, id: &str) -> String {
        format!("{}/push/{id}", self.origin)
    }

    pub fn count(&self) -> usize {
        self.received.lock().unwrap().len()
    }

    pub async fn wait_for(&self, n: usize) -> Vec<Received> {
        for _ in 0..100 {
            if self.count() >= n {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        self.received.lock().unwrap().clone()
    }
}

pub fn push_config() -> PushConfig {
    PushConfig {
        private_key_pkcs8: URL_SAFE_NO_PAD.decode(generate_private_key()).unwrap(),
        subject: "mailto:test@featuredoc.invalid".into(),
        services: vec!["127.0.0.1".into()],
    }
}

pub fn with_push(state: &AppState) -> AppState {
    let config: Config = Config {
        push: Some(push_config()),
        ..(*state.config).clone()
    };
    AppState {
        db: state.db.clone(),
        config: Arc::new(config),
        http: state.http.clone(),
    }
}

pub struct Subscriber {
    pub endpoint: String,
    private: EphemeralPrivateKey,
    public: Vec<u8>,
    auth: [u8; 16],
}

impl Subscriber {
    pub fn new(endpoint: String) -> Self {
        let rng = SystemRandom::new();
        let private = EphemeralPrivateKey::generate(&ECDH_P256, &rng).unwrap();
        let public = private.compute_public_key().unwrap().as_ref().to_vec();
        let mut auth = [0u8; 16];
        rng.fill(&mut auth).unwrap();
        Self {
            endpoint,
            private,
            public,
            auth,
        }
    }

    pub fn subscription(&self) -> Value {
        json!({
            "endpoint": self.endpoint,
            "keys": {
                "p256dh": URL_SAFE_NO_PAD.encode(&self.public),
                "auth": URL_SAFE_NO_PAD.encode(self.auth),
            },
        })
    }

    pub fn open(self, message: &[u8]) -> Value {
        let salt = &message[..16];
        let record_size = u32::from_be_bytes(message[16..20].try_into().unwrap());
        assert_eq!(record_size, 4096);
        let id_len = message[20] as usize;
        let as_public = &message[21..21 + id_len];
        let ciphertext = &message[21 + id_len..];

        let ecdh_secret = agree_ephemeral(
            self.private,
            &UnparsedPublicKey::new(&ECDH_P256, as_public),
            |s| s.to_vec(),
        )
        .unwrap();
        let mut key_info = b"WebPush: info\0".to_vec();
        key_info.extend_from_slice(&self.public);
        key_info.extend_from_slice(as_public);
        let mut ikm = [0u8; 32];
        Hkdf::<Sha256>::new(Some(&self.auth), &ecdh_secret)
            .expand(&key_info, &mut ikm)
            .unwrap();
        let prk = Hkdf::<Sha256>::new(Some(salt), &ikm);
        let mut cek = [0u8; 16];
        prk.expand(b"Content-Encoding: aes128gcm\0", &mut cek).unwrap();
        let mut nonce = [0u8; 12];
        prk.expand(b"Content-Encoding: nonce\0", &mut nonce).unwrap();
        let mut plain = Aes128Gcm::new_from_slice(&cek)
            .unwrap()
            .decrypt(Nonce::from_slice(&nonce), ciphertext)
            .unwrap();
        assert_eq!(plain.pop(), Some(0x02));
        serde_json::from_slice(&plain).unwrap()
    }
}

pub fn vapid_audience(authorization: &str) -> String {
    let token = authorization
        .strip_prefix("vapid t=")
        .and_then(|rest| rest.split(", k=").next())
        .unwrap();
    let claims = token.split('.').nth(1).unwrap();
    let claims: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(claims).unwrap()).unwrap();
    claims["aud"].as_str().unwrap().to_string()
}
