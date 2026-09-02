//! LLM API key registration, live validation, listing (identifiers only), and
//! revocation (AC4.2). Keys are envelope-encrypted at rest; the plaintext is held
//! only transiently during validation/sealing and decrypted just-in-time at use.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::auth::CurrentUser;
use crate::config::Mode;
use crate::crypto::{self, Envelope};
use crate::error::AppError;
use crate::state::AppState;
use crate::util::now_unix;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/llm-keys", post(register).get(list))
        .route("/api/llm-keys/preflight", get(preflight))
        .route("/api/llm-keys/{id}", delete(revoke))
}

// ── provider ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Provider {
    Anthropic,
    OpenAI,
    Google,
}

impl Provider {
    fn parse(s: &str) -> Option<Provider> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic" => Some(Provider::Anthropic),
            "openai" => Some(Provider::OpenAI),
            "google" => Some(Provider::Google),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAI => "openai",
            Provider::Google => "google",
        }
    }

    /// The provider's public key prefix (not secret) — used for stub validation
    /// and for masking display.
    fn prefix(self) -> &'static str {
        match self {
            Provider::Anthropic => "sk-ant-",
            Provider::OpenAI => "sk-",
            Provider::Google => "AIza",
        }
    }

    /// The call-side view of this provider.
    ///
    /// The two enums stay separate — this one is the registration vocabulary, the
    /// other is the call vocabulary — but the registration *scope* has to follow the
    /// calls (AC4.2), so this bridge is the only place they meet and
    /// [`crate::llm::Provider::supports_analysis`] is the only predicate consulted.
    fn llm(self) -> crate::llm::Provider {
        match self {
            Provider::Anthropic => crate::llm::Provider::Anthropic,
            Provider::OpenAI => crate::llm::Provider::OpenAI,
            Provider::Google => crate::llm::Provider::Google,
        }
    }
}

// ── views / rows ──────────────────────────────────────────────────────────────

/// What the API exposes for a key — identifiers only, never the secret or its bytes.
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LlmKeyView {
    pub id: String,
    pub provider: String,
    pub fingerprint: String,
    pub masked: String,
    pub status: String,
    pub created_at: i64,
}

/// The sealed columns needed to decrypt a key just-in-time at use.
#[derive(sqlx::FromRow)]
struct SealedKey {
    provider: String,
    fingerprint: String,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    wrapped_dek: Vec<u8>,
    dek_nonce: Vec<u8>,
}

// ── handlers ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterReq {
    provider: String,
    key: String,
}

/// Registers a key: validate provider → check it is in the supported scope →
/// live-validate the key → envelope-encrypt → store ciphertext only. Returns the
/// identifier view (201).
async fn register(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<RegisterReq>,
) -> Result<(StatusCode, Json<LlmKeyView>), AppError> {
    let provider = Provider::parse(&req.provider)
        .ok_or_else(|| AppError::BadRequest("지원하지 않는 provider입니다".into()))?;

    // AC4.2 지원 제공자 범위. 분석 호출이 구현되지 않은 제공자의 키는 받지 않는다 —
    // 보관해도 호출도 비용 추정도 할 수 없고, 그 사이 [`ACTIVE_KEY_SQL`]이 등록 순서만으로
    // 그 키를 "활성 키"로 골라 **이미 동작하던 분석을 조용히 깨뜨린다**(Anthropic 키를 쓰던
    // 사용자가 미지원 키를 나중에 등록하는 경우). 쓸 수 없는 자격증명을 보관하지 않는 편이
    // AC4.3 과도 맞다. 거부는 사용자가 아직 아무것도 맡기지 않은 이 지점에서 일어난다.
    if !provider.llm().supports_analysis() {
        return Err(AppError::BadRequest(format!(
            "{}는 아직 분석 호출을 지원하지 않아 키를 등록할 수 없습니다. OpenAI 또는 Anthropic 키를 등록해 주세요",
            provider.as_str()
        )));
    }

    let key = req.key.trim();
    if key.is_empty() {
        return Err(AppError::BadRequest("API 키가 비어 있습니다".into()));
    }

    validate_key(&state, provider, key).await?;

    let envelope = crypto::seal(&state.config.kek, key.as_bytes())?;
    let fingerprint = fingerprint(key);
    let masked = mask(provider, key);

    let id = uuid::Uuid::new_v4().to_string();
    let now = now_unix();
    sqlx::query(
        "INSERT INTO llm_keys \
         (id, user_id, provider, fingerprint, masked, ciphertext, nonce, wrapped_dek, dek_nonce, status, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(provider.as_str())
    .bind(&fingerprint)
    .bind(&masked)
    .bind(envelope.ciphertext.as_slice())
    .bind(envelope.nonce.as_slice())
    .bind(envelope.wrapped_dek.as_slice())
    .bind(envelope.dek_nonce.as_slice())
    .bind(now)
    .execute(&state.db)
    .await?;
    crate::audit::record(
        &state.db,
        Some(&user.id),
        "llm_key.register",
        Some(provider.as_str()),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(LlmKeyView {
            id,
            provider: provider.as_str().to_string(),
            fingerprint,
            masked,
            status: "active".to_string(),
            created_at: now,
        }),
    ))
}

/// Lists the user's keys — identifiers only.
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<LlmKeyView>>, AppError> {
    let keys = sqlx::query_as::<_, LlmKeyView>(
        "SELECT id, provider, fingerprint, masked, status, created_at \
         FROM llm_keys WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(keys))
}

/// Revokes one of the user's keys. Scoped to the owner — another user's id is a 404
/// (AC4.7), and new calls are blocked thereafter.
async fn revoke(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query(
        "UPDATE llm_keys SET status = 'revoked', revoked_at = ? \
         WHERE id = ? AND user_id = ? AND status = 'active'",
    )
    .bind(now_unix())
    .bind(&id)
    .bind(&user.id)
    .execute(&state.db)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    crate::audit::record(&state.db, Some(&user.id), "llm_key.revoke", Some(&id)).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Which of a user's active keys a call uses.
///
/// A user may hold one key per provider, so "the active key" needs a rule. It is
/// **OpenAI first, then most recently registered** — the same default the worker
/// falls back to and the registration screen starts on ([`crate::llm::DEFAULT_PROVIDER`]),
/// so all three agree on which provider a user who never chose is billed for.
/// Users with a single active key are unaffected.
///
/// Shared by [`preflight`] and [`active_key_for_user`] on purpose: preflight tells
/// the user which key an analysis will use, so it has to resolve the same one.
///
/// The ranking is deliberately blind to whether a provider is callable — it does not
/// have to know, because [`register`] refuses keys outside AC4.2's supported scope, so
/// every row it can rank is usable. Widening registration without widening the calls
/// would put that invariant back at risk (a newer unsupported key would outrank a
/// working one; `a_refused_provider_cannot_displace_a_working_active_key` pins it).
const ACTIVE_KEY_SQL: &str = "SELECT provider, fingerprint, ciphertext, nonce, wrapped_dek, dek_nonce \
     FROM llm_keys WHERE user_id = ? AND status = 'active' \
     ORDER BY (provider = 'openai') DESC, created_at DESC LIMIT 1";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreflightView {
    provider: String,
    fingerprint: String,
}

/// Confirms the user has a usable key before an LLM-backed action (the seam S03's
/// "분석 시작" preflight will call). Decrypts just-in-time to prove usability, then
/// drops the plaintext; never returns or logs the key. No active key → blocked with
/// "키가 없거나 폐기되었습니다" (test#4). Note: full per-call delegation (test#3)
/// lands with the analysis pipeline.
async fn preflight(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<PreflightView>, AppError> {
    let sealed = sqlx::query_as::<_, SealedKey>(ACTIVE_KEY_SQL)
        .bind(&user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("키가 없거나 폐기되었습니다".into()))?;

    let provider = sealed.provider.clone();
    let fingerprint = sealed.fingerprint.clone();

    // Just-in-time decryption: prove the key is recoverable, then immediately drop it.
    let plaintext = crypto::open(
        &state.config.kek,
        &Envelope {
            ciphertext: sealed.ciphertext,
            nonce: sealed.nonce,
            wrapped_dek: sealed.wrapped_dek,
            dek_nonce: sealed.dek_nonce,
        },
    )?;
    drop(plaintext);

    Ok(Json(PreflightView {
        provider,
        fingerprint,
    }))
}

/// The user's active key, decrypted for a single use.
///
/// Used by the queue when it hands a claimed job to the worker (`worker_api`): the
/// worker owns no database and cannot decrypt anything itself, so the API unseals
/// the key just-in-time and passes it over the already-authenticated `/internal`
/// channel. The plaintext is never stored, never logged, and never returned on a
/// user-facing route — the same handling as the installation token beside it (AC4.1/AC4.3).
///
/// `Ok(None)` means the user has no active key: that is a stage failure with a clear
/// reason, not an error here.
pub async fn active_key_for_user(
    state: &AppState,
    user_id: &str,
) -> Result<Option<(String, String)>, AppError> {
    let sealed = sqlx::query_as::<_, SealedKey>(ACTIVE_KEY_SQL)
        .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(sealed) = sealed else {
        return Ok(None);
    };
    let provider = sealed.provider.clone();
    let plaintext = crypto::open(
        &state.config.kek,
        &Envelope {
            ciphertext: sealed.ciphertext,
            nonce: sealed.nonce,
            wrapped_dek: sealed.wrapped_dek,
            dek_nonce: sealed.dek_nonce,
        },
    )?;
    let key = String::from_utf8(plaintext)
        .map_err(|_| AppError::BadRequest("저장된 키를 읽을 수 없습니다".into()))?;
    Ok(Some((provider, key)))
}

// ── validation + display helpers ─────────────────────────────────────────────

/// Live-validates a key with its provider. Stub mode applies a deterministic shape
/// check; real mode makes a lightweight authenticated request. Failures never echo
/// the key.
async fn validate_key(state: &AppState, provider: Provider, key: &str) -> Result<(), AppError> {
    match state.config.mode {
        Mode::Stub => {
            if key.len() >= 20 && key.starts_with(provider.prefix()) {
                Ok(())
            } else {
                Err(AppError::BadRequest("API 키 검증에 실패했습니다".into()))
            }
        }
        Mode::Real => {
            let req = match provider {
                Provider::Anthropic => state
                    .http
                    .get("https://api.anthropic.com/v1/models")
                    .header("x-api-key", key)
                    .header("anthropic-version", "2023-06-01"),
                Provider::OpenAI => state
                    .http
                    .get("https://api.openai.com/v1/models")
                    .header("Authorization", format!("Bearer {key}")),
                Provider::Google => state.http.get(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={key}"
                )),
            };
            let resp = req
                .header("User-Agent", "featuredoc/0.1")
                .send()
                .await
                .map_err(|_| AppError::BadRequest("API 키 검증 요청에 실패했습니다".into()))?;
            if resp.status().is_success() {
                Ok(())
            } else {
                Err(AppError::BadRequest("API 키 검증에 실패했습니다".into()))
            }
        }
    }
}

/// Non-reversible identifier for a key (first 64 bits of its SHA-256, hex).
fn fingerprint(key: &str) -> String {
    let digest = Sha256::digest(key.as_bytes());
    hex::encode(&digest[..8])
}

/// Display-only mask: reveal just the provider's public prefix, bullet the rest.
/// Never reveals any secret portion of the key.
fn mask(provider: Provider, key: &str) -> String {
    let head = if key.starts_with(provider.prefix()) {
        provider.prefix().to_string()
    } else {
        key.chars().take(3).collect::<String>()
    };
    format!("{head}{}", "\u{2022}".repeat(12))
}
