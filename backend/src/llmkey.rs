//! LLM API key registration, listing, and revocation (AC4.2). Keys are
//! live-validated before being sealed with envelope encryption; only
//! non-sensitive identifiers (provider, fingerprint, masked tail, status) are
//! ever returned. Every query is scoped to the session user (AC4.7).

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::CurrentUser;
use crate::crypto::{self, SecretString};
use crate::db::now;
use crate::error::AppError;
use crate::llm::Provider;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterReq {
    provider: String,
    key: String,
}

#[derive(Serialize)]
pub struct KeyView {
    id: String,
    provider: String,
    fingerprint: String,
    masked: String,
    status: String,
    created_at: i64,
}

/// POST /api/llm-keys — validate, seal, store. Returns the safe view only.
pub async fn register(
    State(app): State<AppState>,
    user: CurrentUser,
    Json(req): Json<RegisterReq>,
) -> Result<Json<KeyView>, AppError> {
    let provider = Provider::parse(&req.provider)?;
    // Wrap immediately so the plaintext can never accidentally be logged.
    let secret = SecretString::new(req.key);

    // Live validation against the provider (or mock). On failure we never
    // persist anything.
    app.llm.validate(provider, &secret).await?;

    let env = crypto::seal(&app.kek, &secret)?;
    drop(secret); // plaintext gone from memory

    // Supersede any existing active key for this provider+user.
    sqlx::query(
        "UPDATE llm_keys SET status = 'revoked', revoked_at = ? \
         WHERE user_id = ? AND provider = ? AND status = 'active'",
    )
    .bind(now())
    .bind(&user.id)
    .bind(provider.as_str())
    .execute(&app.db)
    .await?;

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO llm_keys \
         (id, user_id, provider, fingerprint, masked, dek_wrapped, dek_nonce, \
          ciphertext, ciphertext_nonce, status, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(provider.as_str())
    .bind(&env.fingerprint)
    .bind(&env.masked)
    .bind(&env.dek_wrapped)
    .bind(&env.dek_nonce)
    .bind(&env.ciphertext)
    .bind(&env.ciphertext_nonce)
    .bind(now())
    .execute(&app.db)
    .await?;

    crate::audit::record(
        &app.db,
        Some(&user.id),
        "llm_key.register",
        &format!("{} fp={}", provider.as_str(), env.fingerprint),
    )
    .await;

    Ok(Json(KeyView {
        id,
        provider: provider.as_str().to_string(),
        fingerprint: env.fingerprint,
        masked: env.masked,
        status: "active".into(),
        created_at: now(),
    }))
}

/// GET /api/llm-keys — list this user's keys (identifiers only, never values).
pub async fn list(
    State(app): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<KeyView>>, AppError> {
    let rows: Vec<(String, String, String, String, String, i64)> = sqlx::query_as(
        "SELECT id, provider, fingerprint, masked, status, created_at \
         FROM llm_keys WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user.id)
    .fetch_all(&app.db)
    .await?;
    let views = rows
        .into_iter()
        .map(|(id, provider, fingerprint, masked, status, created_at)| KeyView {
            id,
            provider,
            fingerprint,
            masked,
            status,
            created_at,
        })
        .collect();
    Ok(Json(views))
}

/// DELETE /api/llm-keys/:id — revoke. Scoped to the user: another user's id
/// simply isn't found (AC4.7).
pub async fn revoke(
    State(app): State<AppState>,
    user: CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let affected = sqlx::query(
        "UPDATE llm_keys SET status = 'revoked', revoked_at = ? \
         WHERE id = ? AND user_id = ? AND status = 'active'",
    )
    .bind(now())
    .bind(&id)
    .bind(&user.id)
    .execute(&app.db)
    .await?
    .rows_affected();

    if affected == 0 {
        // Either it doesn't exist, isn't ours, or is already revoked.
        return Err(AppError::NotFound);
    }

    crate::audit::record(&app.db, Some(&user.id), "llm_key.revoke", &id).await;
    Ok(Json(json!({ "ok": true, "status": "revoked" })))
}

/// Fetch the active key for a provider and decrypt it — used by the analysis
/// pipeline at call time (AC4.3b: decrypt only just before use). If there is
/// no active key the call is blocked with the AC4.2 message.
#[allow(dead_code)]
pub async fn active_key_for(
    app: &AppState,
    user_id: &str,
    provider: Provider,
) -> Result<SecretString, AppError> {
    let row: Option<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> = sqlx::query_as(
        "SELECT dek_wrapped, dek_nonce, ciphertext, ciphertext_nonce \
         FROM llm_keys WHERE user_id = ? AND provider = ? AND status = 'active' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .fetch_optional(&app.db)
    .await?;

    let Some((dek_wrapped, dek_nonce, ciphertext, ciphertext_nonce)) = row else {
        return Err(AppError::Blocked(
            "키가 없거나 폐기되었습니다.".into(),
        ));
    };
    let env = crypto::Envelope {
        dek_wrapped,
        dek_nonce,
        ciphertext,
        ciphertext_nonce,
        fingerprint: String::new(),
        masked: String::new(),
    };
    crypto::open(&app.kek, &env).map_err(Into::into)
}
