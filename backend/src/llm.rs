//! LLM provider model and live key validation (AC4.2). Validation goes through
//! the `LlmValidator` trait so CI/e2e run a deterministic double while staging
//! makes a real lightweight probe against the provider.

use async_trait::async_trait;
use std::sync::Arc;

use crate::config::Mode;
use crate::crypto::SecretString;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Google,
}

impl Provider {
    pub fn parse(s: &str) -> Result<Provider, AppError> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic" => Ok(Provider::Anthropic),
            "openai" => Ok(Provider::OpenAI),
            "google" => Ok(Provider::Google),
            other => Err(AppError::BadRequest(format!("unknown provider: {other}"))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAI => "openai",
            Provider::Google => "google",
        }
    }

    /// Expected key prefix, used by the mock validator and as a cheap sanity
    /// check before any network call.
    fn expected_prefix(&self) -> &'static str {
        match self {
            Provider::Anthropic => "sk-ant-",
            Provider::OpenAI => "sk-",
            Provider::Google => "AIza",
        }
    }
}

#[async_trait]
pub trait LlmValidator: Send + Sync {
    /// Returns Ok(()) if the key is currently accepted by the provider.
    async fn validate(&self, provider: Provider, key: &SecretString) -> Result<(), AppError>;
}

pub type SharedValidator = Arc<dyn LlmValidator>;

// ----------------------------------------------------------------------------
// Mock — deterministic. Accepts well-formed keys, rejects anything containing
// "invalid" so tests can exercise the failure path.
// ----------------------------------------------------------------------------

pub struct MockValidator;

#[async_trait]
impl LlmValidator for MockValidator {
    async fn validate(&self, provider: Provider, key: &SecretString) -> Result<(), AppError> {
        let k = key.expose();
        if !k.starts_with(provider.expected_prefix()) || k.contains("invalid") || k.len() < 12 {
            return Err(AppError::BadRequest(
                "키 검증에 실패했습니다. 올바른 API Key인지 확인해 주세요.".into(),
            ));
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Real — a single cheap authenticated request per provider.
// ----------------------------------------------------------------------------

pub struct RealValidator {
    http: reqwest::Client,
}

impl RealValidator {
    pub fn new(http: reqwest::Client) -> Self {
        RealValidator { http }
    }
}

#[async_trait]
impl LlmValidator for RealValidator {
    async fn validate(&self, provider: Provider, key: &SecretString) -> Result<(), AppError> {
        if !key.expose().starts_with(provider.expected_prefix()) {
            return Err(AppError::BadRequest(
                "키 형식이 올바르지 않습니다.".into(),
            ));
        }
        let req = match provider {
            Provider::Anthropic => self
                .http
                .get("https://api.anthropic.com/v1/models")
                .header("x-api-key", key.expose())
                .header("anthropic-version", "2023-06-01"),
            Provider::OpenAI => self
                .http
                .get("https://api.openai.com/v1/models")
                .bearer_auth(key.expose()),
            Provider::Google => self.http.get(format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                key.expose()
            )),
        };
        let resp = req.send().await.map_err(|e| {
            tracing::error!(error = %e, provider = provider.as_str(), "llm validation request failed");
            AppError::ProviderRejected
        })?;
        if resp.status().is_success() {
            Ok(())
        } else if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
            Err(AppError::BadRequest(
                "키 검증에 실패했습니다. 올바른 API Key인지 확인해 주세요.".into(),
            ))
        } else {
            Err(AppError::ProviderRejected)
        }
    }
}

pub fn build(mode: Mode, http: reqwest::Client) -> SharedValidator {
    match mode {
        Mode::Mock => Arc::new(MockValidator),
        Mode::Real => Arc::new(RealValidator::new(http)),
    }
}
