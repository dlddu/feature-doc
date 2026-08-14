//! The LLM call boundary (AC1.2~AC1.4).
//!
//! Stage 1 (`fetch`) needed no model; every stage after it does. This module is the
//! one place that talks to a provider, so the pipeline stages stay about *what* to
//! ask and this file owns *how* to ask it.
//!
//! It mirrors the `Mode` split the rest of the codebase already uses (`repo_scan`,
//! `github_app`, `llmkey`): `Stub` answers deterministically from the input so the
//! kind e2e and unit tests never reach the network, `Real` calls the provider with
//! the user's own key.
//!
//! Two deliberate choices about the request shape:
//!
//! * **Structured outputs, not a prefill.** The answer has to come back as a fixed
//!   JSON shape. Prefilling the assistant turn to force that shape is rejected
//!   (HTTP 400) on current Claude models, so the schema goes in `output_config`
//!   and the model is constrained server-side.
//! * **No `temperature`.** Current Claude models reject sampling parameters
//!   outright — sending `temperature: 0` is a 400, not a no-op. Determinism comes
//!   from a fixed prompt over a sorted input plus `effort`, and is *observed*
//!   rather than assumed: [`crate::analysis`] hashes the result so a re-analysis
//!   reports whether it reproduced (AC1.2's "결정적으로 재현되거나 차이가 명시된다").
//!
//! The key is a function argument, never a struct field, and never appears in an
//! error message — the same rule `llmkey` follows (AC4.3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Mode;

/// Anthropic model used for pipeline stages. Pinned here rather than in each
/// caller so a model change is one edit.
const ANTHROPIC_MODEL: &str = "claude-opus-5";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Output cap. Thinking and response text share this budget on current models, so
/// it is sized well above the document itself rather than trimmed to it.
const MAX_TOKENS: u32 = 16000;

/// One model call: what to ask, and the JSON shape the answer must take.
pub struct Ask<'a> {
    pub system: &'a str,
    pub user: String,
    /// JSON Schema the response is constrained to (`output_config.format`).
    pub schema: Value,
}

/// What a call produced. Token counts are recorded for the cost accounting the
/// roadmap places in a later slice (AC4.6) — persisted now so the numbers exist
/// when the screen for them lands.
#[derive(Debug, Clone)]
pub struct Answer {
    pub content: Value,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

/// The providers a key can be registered under (`llmkey::Provider`), as seen from
/// the call side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Google,
}

impl Provider {
    pub fn parse(s: &str) -> Option<Provider> {
        match s.to_ascii_lowercase().as_str() {
            "anthropic" => Some(Provider::Anthropic),
            "openai" => Some(Provider::OpenAI),
            "google" => Some(Provider::Google),
            _ => None,
        }
    }
}

/// Runs one call and returns the parsed JSON answer.
///
/// `Err` is an operator/user-facing one-liner: it names what failed, never the key
/// and never the raw provider response.
pub async fn ask(
    http: &reqwest::Client,
    mode: Mode,
    provider: Provider,
    key: Option<&str>,
    ask: Ask<'_>,
) -> Result<Answer, String> {
    match mode {
        Mode::Stub => stub_answer(&ask),
        Mode::Real => {
            let key = key.ok_or_else(|| "no LLM key available for this analysis".to_string())?;
            match provider {
                Provider::Anthropic => anthropic(http, key, ask).await,
                // Only the Anthropic call is implemented in this slice. Failing
                // loudly beats quietly producing a stub document in a real
                // deployment — the stage fails and AC1.5's per-stage retry picks
                // it up once the provider lands.
                other => Err(format!(
                    "{} is registered but not yet supported for analysis; register an Anthropic key",
                    match other {
                        Provider::OpenAI => "openai",
                        Provider::Google => "google",
                        Provider::Anthropic => unreachable!(),
                    }
                )),
            }
        }
    }
}

// ── real: Anthropic Messages API ─────────────────────────────────────────────

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    model: String,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    content: Vec<AnthropicBlock>,
    #[serde(default)]
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize, Default)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Value>,
    output_config: Value,
}

async fn anthropic(http: &reqwest::Client, key: &str, ask: Ask<'_>) -> Result<Answer, String> {
    let body = AnthropicRequest {
        model: ANTHROPIC_MODEL,
        max_tokens: MAX_TOKENS,
        system: ask.system,
        messages: vec![json!({ "role": "user", "content": ask.user })],
        // `effort` bounds how much the model spends before answering; the schema
        // constrains the answer's shape. Neither is a sampling parameter.
        output_config: json!({
            "effort": "medium",
            "format": { "type": "json_schema", "schema": ask.schema },
        }),
    };

    let resp = http
        .post(ANTHROPIC_URL)
        .header("x-api-key", key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .header("User-Agent", "featuredoc-worker/0.1")
        .json(&body)
        .send()
        .await
        // Never interpolate the request or the key into the message (AC4.3).
        .map_err(|_| "LLM request failed".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("LLM rejected the request ({})", resp.status().as_u16()));
    }

    let parsed: AnthropicResponse = resp
        .json()
        .await
        .map_err(|_| "LLM returned a malformed response".to_string())?;

    // A refusal is a successful HTTP response with an empty or partial body, so
    // it has to be checked before reading `content` — otherwise it reads as an
    // empty document rather than a failure.
    if parsed.stop_reason.as_deref() == Some("refusal") {
        return Err("LLM declined to answer this request".to_string());
    }

    let text = parsed
        .content
        .iter()
        .find(|b| b.kind == "text")
        .and_then(|b| b.text.as_deref())
        .ok_or_else(|| "LLM returned no text content".to_string())?;

    let content: Value =
        serde_json::from_str(text).map_err(|_| "LLM returned unparseable JSON".to_string())?;

    Ok(Answer {
        content,
        model: if parsed.model.is_empty() {
            ANTHROPIC_MODEL.to_string()
        } else {
            parsed.model
        },
        input_tokens: parsed.usage.input_tokens,
        output_tokens: parsed.usage.output_tokens,
    })
}

// ── stub ─────────────────────────────────────────────────────────────────────

/// The model name the stub reports, so a document written in stub mode is
/// distinguishable from a real one by inspection.
pub const STUB_MODEL: &str = "stub-model";

/// Deterministic stand-in. The caller supplies the whole answer under
/// `"stub_answer"` in the schema, which keeps this file free of any one stage's
/// output shape while still giving the e2e a fixed document to assert on.
///
/// Token counts are derived from the prompt length so the cost columns hold
/// plausible, stable numbers rather than zeros.
fn stub_answer(ask: &Ask<'_>) -> Result<Answer, String> {
    let content = ask
        .schema
        .get("stub_answer")
        .cloned()
        .ok_or_else(|| "stub mode: caller supplied no stub answer".to_string())?;

    Ok(Answer {
        content,
        model: STUB_MODEL.to_string(),
        input_tokens: (ask.system.len() + ask.user.len()) as i64 / 4,
        output_tokens: 256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_with_stub() -> Value {
        json!({
            "type": "object",
            "stub_answer": { "categories": [] },
        })
    }

    #[tokio::test]
    async fn stub_is_deterministic_for_the_same_ask() {
        let http = reqwest::Client::new();
        let mk = || Ask {
            system: "sys",
            user: "a\nb\nc".to_string(),
            schema: schema_with_stub(),
        };
        let a = ask(&http, Mode::Stub, Provider::Anthropic, None, mk())
            .await
            .unwrap();
        let b = ask(&http, Mode::Stub, Provider::Anthropic, None, mk())
            .await
            .unwrap();
        assert_eq!(a.content, b.content);
        assert_eq!(a.input_tokens, b.input_tokens);
        assert_eq!(a.model, STUB_MODEL);
    }

    /// Real mode with no key must fail rather than silently fall through to the
    /// stub — a stub document in a real deployment is the "gate behind a fake"
    /// shape the pipeline is meant to avoid.
    #[tokio::test]
    async fn real_mode_without_a_key_fails() {
        let http = reqwest::Client::new();
        let err = ask(
            &http,
            Mode::Real,
            Provider::Anthropic,
            None,
            Ask {
                system: "sys",
                user: "u".to_string(),
                schema: schema_with_stub(),
            },
        )
        .await
        .unwrap_err();
        assert!(err.contains("no LLM key"), "{err}");
    }

    #[test]
    fn provider_parses_the_registered_names() {
        assert_eq!(Provider::parse("anthropic"), Some(Provider::Anthropic));
        assert_eq!(Provider::parse("OpenAI"), Some(Provider::OpenAI));
        assert_eq!(Provider::parse("google"), Some(Provider::Google));
        assert_eq!(Provider::parse("nope"), None);
    }
}
