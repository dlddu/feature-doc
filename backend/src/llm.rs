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
//!   (HTTP 400) on current Claude models, so the schema goes in the provider's
//!   structured-output slot and the model is constrained server-side.
//! * **No sampling parameters.** Current models on both providers reject
//!   `temperature` outright — sending `temperature: 0` is a 400, not a no-op.
//!   Determinism comes from a fixed prompt over a sorted input plus a reasoning
//!   `effort`, and is *observed* rather than assumed: [`crate::analysis`] hashes
//!   the result so a re-analysis reports whether it reproduced (AC1.2's
//!   "결정적으로 재현되거나 차이가 명시된다").
//!
//! The key is a function argument, never a struct field, and never appears in an
//! error message — the same rule `llmkey` follows (AC4.3).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::Mode;

/// Model each provider's pipeline calls use. Pinned here rather than in each
/// caller so a model change is one edit.
const ANTHROPIC_MODEL: &str = "claude-opus-5";
const OPENAI_MODEL: &str = "gpt-5.6-luna";

const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const OPENAI_URL: &str = "https://api.openai.com/v1/responses";

/// Output cap. Reasoning and response text share this budget on current models,
/// so it is sized well above the document itself rather than trimmed to it.
const MAX_TOKENS: u32 = 16000;

/// How much the model may spend reasoning before it answers. Not a sampling
/// parameter — both providers take it under that name.
const EFFORT: &str = "medium";

/// Name the structured-output schema is registered under. OpenAI requires one;
/// it is an identifier, not part of the prompt.
const SCHEMA_NAME: &str = "analysis_document";

/// One model call: what to ask, the JSON shape the answer must take, and the
/// answer stub mode hands back instead of calling anyone.
pub struct Ask<'a> {
    pub system: &'a str,
    pub user: String,
    /// JSON Schema the response is constrained to. Sent to the provider verbatim,
    /// so it holds nothing but the schema — OpenAI validates it under
    /// `strict: true` and rejects keywords it does not know.
    pub schema: Value,
    /// What [`Mode::Stub`] answers with. Lives beside the schema rather than
    /// inside it so the two never travel together to a provider.
    pub stub: Value,
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

    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAI => "openai",
            Provider::Google => "google",
        }
    }

    /// The model this provider's calls are made with, or `None` where no call is
    /// implemented yet.
    pub fn default_model(self) -> Option<&'static str> {
        match self {
            Provider::Anthropic => Some(ANTHROPIC_MODEL),
            Provider::OpenAI => Some(OPENAI_MODEL),
            Provider::Google => None,
        }
    }

    /// Whether an analysis call is implemented for this provider — AC4.2's
    /// **supported-provider scope**, and the one place that answers it.
    ///
    /// Derived from [`Self::default_model`] rather than kept as its own list so the
    /// registration gate (`llmkey::register`) and the dispatch in [`ask`] cannot
    /// drift apart: a provider is registerable exactly when there is a model to call
    /// it with. Landing a provider is therefore one edit, not two.
    pub fn supports_analysis(self) -> bool {
        self.default_model().is_some()
    }
}

/// Every provider the registration vocabulary knows, supported or not. Exists so
/// tests can assert over the whole set instead of restating it and going stale.
pub const ALL_PROVIDERS: [Provider; 3] = [Provider::Anthropic, Provider::OpenAI, Provider::Google];

/// The provider a call falls back to when the job carries no usable registration.
///
/// This is the product's default engine, and it is the same answer as
/// `llmkey`'s key preference and the registration screen's initial selection —
/// all three say OpenAI, so a user who never chooses is billed the cheapest
/// supported tier ([`OPENAI_MODEL`]).
pub const DEFAULT_PROVIDER: Provider = Provider::OpenAI;

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
                Provider::OpenAI => openai(http, key, ask).await,
                // No call for this provider yet. Registration refuses these up front
                // (`llmkey::register`, AC4.2's supported scope), so reaching here means
                // a key stored before that gate existed. Failing loudly beats quietly
                // producing a stub document in a real deployment — the stage fails and
                // AC1.5's per-stage retry picks it up once the provider lands.
                other => Err(format!(
                    "{} is registered but not yet supported for analysis; register an OpenAI or Anthropic key",
                    other.as_str()
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

// ── real: OpenAI Responses API ───────────────────────────────────────────────
//
// The Responses API rather than Chat Completions: it is the endpoint that takes
// `reasoning.effort` and `text.format` together, which is exactly the pair the
// Anthropic call above uses (`output_config.effort` / `.format`). Keeping the two
// providers on matching request shapes is what lets the stage code stay unaware of
// which one answered.

#[derive(Deserialize, Default)]
struct OpenAiResponse {
    #[serde(default)]
    model: String,
    /// `completed` / `incomplete` — an early stop is a 200, so it has to be read.
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    incomplete_details: Option<OpenAiIncomplete>,
    #[serde(default)]
    output: Vec<OpenAiOutput>,
    #[serde(default)]
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiIncomplete {
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiOutput {
    /// `message`, `reasoning`, … — only `message` carries the answer.
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    content: Vec<OpenAiContent>,
}

#[derive(Deserialize)]
struct OpenAiContent {
    /// `output_text` or `refusal`.
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize, Default)]
struct OpenAiUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
}

/// The request body, built apart from the call so a test can read it without a
/// network. `instructions` is the system turn and `input` the user turn.
fn openai_body(ask: &Ask<'_>) -> Value {
    json!({
        "model": OPENAI_MODEL,
        "instructions": ask.system,
        "input": ask.user,
        // `strict` is what makes the schema binding rather than advisory; it is
        // also why `Ask::schema` may hold nothing but JSON Schema keywords.
        "text": {
            "format": {
                "type": "json_schema",
                "name": SCHEMA_NAME,
                "schema": ask.schema,
                "strict": true,
            },
        },
        "reasoning": { "effort": EFFORT },
        "max_output_tokens": MAX_TOKENS,
    })
}

/// Reads the answer out of a Responses payload, or says why there isn't one.
/// Split out of [`openai`] so every branch below is reachable from a test.
fn openai_answer(parsed: OpenAiResponse) -> Result<Answer, String> {
    let contents = || parsed.output.iter().flat_map(|o| o.content.iter());

    // A refusal is a successful HTTP response, so it has to be checked before
    // reading the text — otherwise it reads as an empty document rather than a
    // failure.
    if contents().any(|c| c.kind == "refusal") {
        return Err("LLM declined to answer this request".to_string());
    }

    // Truncation is also a 200. Its JSON is unparseable, and reporting "malformed"
    // for it would send the operator looking in the wrong place.
    if parsed.status.as_deref() == Some("incomplete") {
        let reason = parsed
            .incomplete_details
            .as_ref()
            .and_then(|d| d.reason.as_deref())
            .unwrap_or("unknown");
        // Only the reason code is surfaced, sanitised — never the payload (AC4.3).
        let reason: String = reason
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .take(40)
            .collect();
        return Err(format!("LLM stopped before finishing ({reason})"));
    }

    let text = parsed
        .output
        .iter()
        .filter(|o| o.kind == "message")
        .flat_map(|o| o.content.iter())
        .find(|c| c.kind == "output_text")
        .and_then(|c| c.text.as_deref())
        .ok_or_else(|| "LLM returned no text content".to_string())?;

    let content: Value =
        serde_json::from_str(text).map_err(|_| "LLM returned unparseable JSON".to_string())?;

    Ok(Answer {
        content,
        model: if parsed.model.is_empty() {
            OPENAI_MODEL.to_string()
        } else {
            parsed.model
        },
        input_tokens: parsed.usage.input_tokens,
        output_tokens: parsed.usage.output_tokens,
    })
}

async fn openai(http: &reqwest::Client, key: &str, ask: Ask<'_>) -> Result<Answer, String> {
    let resp = http
        .post(OPENAI_URL)
        .header("Authorization", format!("Bearer {key}"))
        .header("content-type", "application/json")
        .header("User-Agent", "featuredoc-worker/0.1")
        .json(&openai_body(&ask))
        .send()
        .await
        // Never interpolate the request or the key into the message (AC4.3).
        .map_err(|_| "LLM request failed".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("LLM rejected the request ({})", resp.status().as_u16()));
    }

    let parsed: OpenAiResponse = resp
        .json()
        .await
        .map_err(|_| "LLM returned a malformed response".to_string())?;

    openai_answer(parsed)
}

// ── stub ─────────────────────────────────────────────────────────────────────

/// The model name the stub reports, so a document written in stub mode is
/// distinguishable from a real one by inspection.
pub const STUB_MODEL: &str = "stub-model";

/// Deterministic stand-in. The caller supplies the whole answer in [`Ask::stub`],
/// which keeps this file free of any one stage's output shape while still giving
/// the e2e a fixed document to assert on.
///
/// Token counts are derived from the prompt length so the cost columns hold
/// plausible, stable numbers rather than zeros.
fn stub_answer(ask: &Ask<'_>) -> Result<Answer, String> {
    if ask.stub.is_null() {
        return Err("stub mode: caller supplied no stub answer".to_string());
    }

    Ok(Answer {
        content: ask.stub.clone(),
        model: STUB_MODEL.to_string(),
        input_tokens: (ask.system.len() + ask.user.len()) as i64 / 4,
        output_tokens: 256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an_ask() -> Ask<'static> {
        Ask {
            system: "sys",
            user: "a\nb\nc".to_string(),
            schema: json!({ "type": "object", "additionalProperties": false }),
            stub: json!({ "categories": [] }),
        }
    }

    #[tokio::test]
    async fn stub_is_deterministic_for_the_same_ask() {
        let http = reqwest::Client::new();
        let a = ask(&http, Mode::Stub, Provider::OpenAI, None, an_ask())
            .await
            .unwrap();
        let b = ask(&http, Mode::Stub, Provider::OpenAI, None, an_ask())
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
        let err = ask(&http, Mode::Real, Provider::Anthropic, None, an_ask())
            .await
            .unwrap_err();
        assert!(err.contains("no LLM key"), "{err}");
    }

    /// A provider with no call must say so instead of failing somewhere less
    /// legible. `llmkey::register` now refuses these at the door (AC4.2's supported
    /// scope), so this arm is the second line: a row stored before that gate existed
    /// still reaches here, and it fails by name rather than by accident.
    ///
    /// Asserted over [`ALL_PROVIDERS`] rather than over `Google` alone so the day
    /// Google lands, this test keeps passing on whatever is unsupported next.
    #[tokio::test]
    async fn an_unsupported_provider_is_refused_by_name() {
        let http = reqwest::Client::new();
        let unsupported: Vec<Provider> = ALL_PROVIDERS
            .into_iter()
            .filter(|p| !p.supports_analysis())
            .collect();
        assert!(
            !unsupported.is_empty(),
            "nothing unsupported left — fold this test into the registration gate"
        );
        for p in unsupported {
            let err = ask(&http, Mode::Real, p, Some("k"), an_ask())
                .await
                .unwrap_err();
            assert!(err.contains(p.as_str()), "{p:?}: {err}");
            assert!(err.contains("not yet supported"), "{p:?}: {err}");
        }
    }

    /// AC4.2 supported-provider scope: registerable **iff** callable. Both sides read
    /// this predicate, so the assertion is that it says what the dispatch does.
    #[test]
    fn the_supported_scope_is_the_set_of_implemented_calls() {
        assert!(Provider::OpenAI.supports_analysis());
        assert!(Provider::Anthropic.supports_analysis());
        assert!(!Provider::Google.supports_analysis());
        // A user who never chooses is billed the default, so the default must be
        // registerable — otherwise the fallback names a provider nobody can hold.
        assert!(DEFAULT_PROVIDER.supports_analysis());
        for p in ALL_PROVIDERS {
            assert_eq!(
                p.supports_analysis(),
                p.default_model().is_some(),
                "{p:?}: scope and model must not be listed apart"
            );
        }
    }

    #[test]
    fn provider_parses_the_registered_names() {
        assert_eq!(Provider::parse("anthropic"), Some(Provider::Anthropic));
        assert_eq!(Provider::parse("OpenAI"), Some(Provider::OpenAI));
        assert_eq!(Provider::parse("google"), Some(Provider::Google));
        assert_eq!(Provider::parse("nope"), None);
    }

    /// The product default, asserted where it is defined so a change to it is a
    /// deliberate edit to a test rather than a silent one.
    #[test]
    fn the_default_provider_and_its_model() {
        assert_eq!(DEFAULT_PROVIDER, Provider::OpenAI);
        assert_eq!(DEFAULT_PROVIDER.default_model(), Some("gpt-5.6-luna"));
        assert_eq!(
            Provider::Anthropic.default_model(),
            Some("claude-opus-5"),
            "the Anthropic path keeps its model — the default moved, it did not replace"
        );
        assert_eq!(Provider::Google.default_model(), None);
    }

    // ── OpenAI request/response shape ────────────────────────────────────────

    #[test]
    fn openai_request_carries_the_schema_and_no_sampling_parameters() {
        let body = openai_body(&an_ask());

        assert_eq!(body["model"], "gpt-5.6-luna");
        assert_eq!(body["instructions"], "sys");
        assert_eq!(body["input"], "a\nb\nc");
        assert_eq!(body["text"]["format"]["type"], "json_schema");
        assert_eq!(body["text"]["format"]["name"], SCHEMA_NAME);
        assert_eq!(body["text"]["format"]["strict"], true);
        assert_eq!(body["text"]["format"]["schema"], an_ask().schema);
        assert_eq!(body["reasoning"]["effort"], EFFORT);
        assert_eq!(body["max_output_tokens"], MAX_TOKENS);

        // Sending any of these is a 400, not a no-op (see the module header).
        for rejected in ["temperature", "top_p", "top_k"] {
            assert!(body.get(rejected).is_none(), "{rejected} must not be sent");
        }
    }

    /// The stub answer must not ride along inside the schema: OpenAI validates
    /// `text.format.schema` under `strict` and rejects keywords it does not know.
    #[test]
    fn openai_request_schema_holds_only_the_schema() {
        let body = openai_body(&an_ask());
        assert!(body["text"]["format"]["schema"]
            .get("stub_answer")
            .is_none());
    }

    fn openai_response(output: Value, status: &str, incomplete: Value) -> OpenAiResponse {
        serde_json::from_value(json!({
            "id": "resp_1",
            "object": "response",
            "status": status,
            "incomplete_details": incomplete,
            "model": "gpt-5.6-luna-2026-07-09",
            "output": output,
            "usage": { "input_tokens": 81, "output_tokens": 11, "total_tokens": 92 },
        }))
        .unwrap()
    }

    #[test]
    fn openai_answer_reads_the_message_text() {
        let parsed = openai_response(
            json!([
                { "id": "rs_1", "type": "reasoning", "summary": [] },
                {
                    "id": "msg_1",
                    "type": "message",
                    "role": "assistant",
                    "content": [{ "type": "output_text", "text": "{\"categories\":[]}" }],
                },
            ]),
            "completed",
            Value::Null,
        );

        let answer = openai_answer(parsed).unwrap();
        assert_eq!(answer.content, json!({ "categories": [] }));
        assert_eq!(answer.input_tokens, 81);
        assert_eq!(answer.output_tokens, 11);
        // The served model, not the constant — the pinned name may resolve to a
        // dated snapshot, and the document records what actually answered.
        assert_eq!(answer.model, "gpt-5.6-luna-2026-07-09");
    }

    #[test]
    fn openai_refusal_is_its_own_failure() {
        let parsed = openai_response(
            json!([{
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "refusal", "refusal": "I can't help with that." }],
            }]),
            "completed",
            Value::Null,
        );

        let err = openai_answer(parsed).unwrap_err();
        assert!(err.contains("declined"), "{err}");
    }

    #[test]
    fn openai_truncation_is_not_reported_as_malformed() {
        let parsed = openai_response(
            json!([{
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "{\"categ" }],
            }]),
            "incomplete",
            json!({ "reason": "max_output_tokens" }),
        );

        let err = openai_answer(parsed).unwrap_err();
        assert!(err.contains("stopped before finishing"), "{err}");
        assert!(err.contains("max_output_tokens"), "{err}");
    }

    #[test]
    fn openai_unparseable_text_fails_rather_than_yielding_a_partial_document() {
        let parsed = openai_response(
            json!([{
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": "not json" }],
            }]),
            "completed",
            Value::Null,
        );

        assert!(openai_answer(parsed)
            .unwrap_err()
            .contains("unparseable JSON"));
    }

    /// Reasoning-only output (no message) is not an empty document.
    #[test]
    fn openai_without_a_message_fails() {
        let parsed = openai_response(
            json!([{ "id": "rs_1", "type": "reasoning", "summary": [] }]),
            "completed",
            Value::Null,
        );

        assert!(openai_answer(parsed)
            .unwrap_err()
            .contains("no text content"));
    }
}
