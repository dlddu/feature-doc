//! OpenTelemetry tracing for the API: an OTLP/HTTP exporter, W3C trace-context
//! propagation and one server span per request.
//!
//! Tracing is opt-in. With `OTEL_EXPORTER_OTLP_ENDPOINT` (or
//! `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`) unset, [`init`] installs nothing and the
//! global tracer stays a no-op, so tests, the kind e2e and local runs export nothing.
//!
//! Why an SDK when OBI (eBPF) already traces this namespace: OBI looks for the
//! `traceparent` header only in the first 1024 bytes of a request. Traefik writes the
//! request line, Host and User-Agent first and the rest alphabetically, so a browser's
//! Cookie, Referer and Sec-* headers come before Traceparent and can push it out of that
//! window; those requests then started a trace of their own instead of continuing
//! Traefik's. Reading the parsed headers here has no such limit. OBI is told to skip
//! this Deployment, so this is the only source of the API's server spans.

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::trace::{FutureExt, SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{global, KeyValue};
use opentelemetry_http::HeaderExtractor;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

const SERVICE_NAME: &str = "featuredoc";

/// Paths that get no span: the kubelet probe and the worker's idle claim poll, which
/// would otherwise be most of the traffic and carry no request worth following.
const SKIP_PATHS: &[&str] = &["/hello", "/internal/analyses/claim"];

/// Whether an OTLP endpoint is configured, reading variables through `var`.
pub fn enabled(var: impl Fn(&str) -> Option<String>) -> bool {
    [
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
    ]
    .iter()
    .any(|k| var(k).is_some_and(|v| !v.is_empty()))
}

/// Installs the global tracer provider and propagator when an endpoint is configured.
/// Keep the returned provider and call `shutdown` on it before exiting so the last
/// batch is flushed. `OTEL_SERVICE_NAME` / `OTEL_RESOURCE_ATTRIBUTES` override the
/// compiled-in service name.
pub fn init() -> anyhow::Result<Option<SdkTracerProvider>> {
    let var = |k: &str| std::env::var(k).ok();
    if !enabled(var) {
        return Ok(None);
    }
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()?;

    let mut resource = Resource::builder();
    let named_by_env = var("OTEL_SERVICE_NAME").is_some()
        || var("OTEL_RESOURCE_ATTRIBUTES").is_some_and(|a| a.contains("service.name="));
    if !named_by_env {
        resource = resource.with_service_name(SERVICE_NAME);
    }

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.build())
        .build();
    global::set_tracer_provider(provider.clone());
    global::set_text_map_propagator(TraceContextPropagator::new());
    Ok(Some(provider))
}

/// Axum middleware: one server span per request, continuing the caller's trace when
/// the request carries `traceparent`. The span is named after the matched route
/// (`GET /api/analyses/{id}`) so IDs in the path do not make every name unique;
/// requests the SPA fallback answers have no route and are named by method alone.
pub async fn trace_request(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if SKIP_PATHS.contains(&path.as_str()) {
        return next.run(req).await;
    }

    let parent = global::get_text_map_propagator(|p| p.extract(&HeaderExtractor(req.headers())));
    let method = req.method().as_str().to_owned();
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_owned());

    let mut attributes = vec![
        KeyValue::new("http.request.method", method.clone()),
        KeyValue::new("url.path", path),
    ];
    let name = match &route {
        Some(r) => {
            attributes.push(KeyValue::new("http.route", r.clone()));
            format!("{method} {r}")
        }
        None => method,
    };

    let tracer = global::tracer(SERVICE_NAME);
    let span = tracer
        .span_builder(name)
        .with_kind(SpanKind::Server)
        .with_attributes(attributes)
        .start_with_context(&tracer, &parent);
    let cx = parent.with_span(span);

    let response = next.run(req).with_context(cx.clone()).await;

    let span = cx.span();
    let status = response.status();
    span.set_attribute(KeyValue::new(
        "http.response.status_code",
        i64::from(status.as_u16()),
    ));
    if status.is_server_error() {
        span.set_status(Status::error(status.to_string()));
    }
    span.end();
    response
}
