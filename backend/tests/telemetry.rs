//! Server spans from the telemetry middleware, through the real router.

mod common;

use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use opentelemetry::trace::{SpanKind, TraceId};
use opentelemetry::{global, Value};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider, SpanData};
use tower::ServiceExt;

use featuredoc::{build_router, telemetry};

fn exporter() -> &'static InMemorySpanExporter {
    static EXPORTER: OnceLock<InMemorySpanExporter> = OnceLock::new();
    EXPORTER.get_or_init(|| {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        global::set_tracer_provider(provider);
        global::set_text_map_propagator(TraceContextPropagator::new());
        exporter
    })
}

fn spans_in(trace_id: &str) -> Vec<SpanData> {
    let id = TraceId::from_hex(trace_id).unwrap();
    exporter()
        .get_finished_spans()
        .unwrap()
        .into_iter()
        .filter(|s| s.span_context.trace_id() == id)
        .collect()
}

fn attr(span: &SpanData, key: &str) -> Option<Value> {
    span.attributes
        .iter()
        .find(|kv| kv.key.as_str() == key)
        .map(|kv| kv.value.clone())
}

async fn send(path: &str, trace_id: &str, cookie: &str) -> StatusCode {
    exporter();
    let (state, _db) = common::stub_state().await;
    let req = Request::get(path)
        .header("cookie", cookie)
        .header(
            "referer",
            format!("https://feature-doc.example/{}", "r".repeat(300)),
        )
        .header("traceparent", format!("00-{trace_id}-00f067aa0ba902b7-01"))
        .body(Body::empty())
        .unwrap();
    build_router(state).oneshot(req).await.unwrap().status()
}

// The reason the middleware exists: a traceparent behind long headers (as Traefik
// writes them, alphabetically) must still be continued.
#[tokio::test]
async fn continues_trace_behind_long_headers() {
    const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    let cookie = format!("fd_session={}; other={}", "a".repeat(64), "b".repeat(1500));
    let status = send("/api/analyses/42", TRACE, &cookie).await;

    let spans = spans_in(TRACE);
    assert_eq!(spans.len(), 1, "spans: {spans:?}");
    let span = &spans[0];
    assert_eq!(span.name, "GET /api/analyses/{id}");
    assert_eq!(span.span_kind, SpanKind::Server);
    assert_eq!(span.parent_span_id.to_string(), "00f067aa0ba902b7");
    assert_eq!(attr(span, "http.route"), Some("/api/analyses/{id}".into()));
    assert_eq!(attr(span, "url.path"), Some("/api/analyses/42".into()));
    assert_eq!(
        attr(span, "http.response.status_code"),
        Some(i64::from(status.as_u16()).into())
    );
}

#[tokio::test]
async fn fallback_is_named_by_method() {
    const TRACE: &str = "5bf92f3577b34da6a3ce929d0e0e4737";
    send("/some/spa/page", TRACE, "").await;

    let spans = spans_in(TRACE);
    assert_eq!(spans.len(), 1, "spans: {spans:?}");
    assert_eq!(spans[0].name, "GET");
    assert_eq!(attr(&spans[0], "http.route"), None);
}

#[tokio::test]
async fn probe_and_claim_poll_get_no_span() {
    const PROBE: &str = "6bf92f3577b34da6a3ce929d0e0e4738";
    const CLAIM: &str = "7bf92f3577b34da6a3ce929d0e0e4739";
    send("/hello", PROBE, "").await;
    send("/internal/analyses/claim", CLAIM, "").await;

    assert!(spans_in(PROBE).is_empty());
    assert!(spans_in(CLAIM).is_empty());
}

#[test]
fn enabled_only_with_an_endpoint() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert!(!telemetry::enabled(env(&[])));
    assert!(!telemetry::enabled(env(&[(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        ""
    )])));
    assert!(telemetry::enabled(env(&[(
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "http://alloy:4318"
    )])));
    assert!(telemetry::enabled(env(&[(
        "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
        "http://alloy:4318/v1/traces"
    )])));
}
