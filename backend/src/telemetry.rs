//! OpenTelemetry tracing for the API and the worker: an OTLP/HTTP exporter, W3C
//! trace-context propagation, one server span per API request, and on the worker one
//! trace per claimed job with a span per stage and per call back to the API, and in
//! either process a span per LLM call.
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
//! both Deployments, so these are the only spans either process produces.

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use opentelemetry::trace::{FutureExt, SpanKind, Status, TraceContextExt, Tracer};
use std::fmt::Display;
use std::future::Future;

use opentelemetry::{global, Context, KeyValue};
use opentelemetry_http::{HeaderExtractor, HeaderInjector};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

const SCOPE: &str = "featuredoc";

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
/// batch is flushed. `OTEL_SERVICE_NAME` / `OTEL_RESOURCE_ATTRIBUTES` override
/// `service_name`.
pub fn init(service_name: &'static str) -> anyhow::Result<Option<SdkTracerProvider>> {
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
        resource = resource.with_service_name(service_name);
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

    let tracer = global::tracer(SCOPE);
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

/// Runs one claimed job as the root of a new trace (`process analysis`), so its stages
/// and the calls they make share one trace. An `Err` marks the span failed.
pub async fn job<T, E: Display>(
    analysis_id: &str,
    repo: &str,
    stages: &[String],
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let tracer = global::tracer(SCOPE);
    let span = tracer
        .span_builder("process analysis")
        .with_kind(SpanKind::Consumer)
        .with_attributes([
            KeyValue::new("analysis.id", analysis_id.to_owned()),
            KeyValue::new("analysis.repository", repo.to_owned()),
            KeyValue::new("analysis.stages", stages.join(",")),
        ])
        .start_with_context(&tracer, &Context::new());
    finish(Context::new().with_span(span), work).await
}

/// Runs one pipeline stage as a child span named after its key.
pub async fn stage<T, E: Display>(
    key: &str,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let tracer = global::tracer(SCOPE);
    let parent = Context::current();
    let span = tracer
        .span_builder(format!("stage {key}"))
        .with_kind(SpanKind::Internal)
        .start_with_context(&tracer, &parent);
    finish(parent.with_span(span), work).await
}

async fn finish<T, E: Display>(
    cx: Context,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let result = work.with_context(cx.clone()).await;
    let span = cx.span();
    if let Err(e) = &result {
        span.set_status(Status::error(e.to_string()));
    }
    span.end();
    result
}

/// Sends `request` with a client span and a `traceparent` for it, when a span is
/// active. Outside one — the idle claim poll — it is sent untouched, so polling makes
/// no traces. The span is named with IDs in the path replaced by `{id}`.
pub async fn send(
    client: &reqwest::Client,
    request: reqwest::RequestBuilder,
) -> reqwest::Result<reqwest::Response> {
    send_traced(client, request, true).await
}

/// Like [`send`], for calls to third parties (GitHub): the span is recorded but no
/// `traceparent` leaves the cluster.
pub async fn send_external(
    client: &reqwest::Client,
    request: reqwest::RequestBuilder,
) -> reqwest::Result<reqwest::Response> {
    send_traced(client, request, false).await
}

async fn send_traced(
    client: &reqwest::Client,
    request: reqwest::RequestBuilder,
    propagate: bool,
) -> reqwest::Result<reqwest::Response> {
    let mut request = request.build()?;
    let parent = Context::current();
    if !parent.span().span_context().is_valid() {
        return client.execute(request).await;
    }

    let method = request.method().as_str().to_owned();
    let tracer = global::tracer(SCOPE);
    let span = tracer
        .span_builder(format!("{method} {}", templated(request.url().path())))
        .with_kind(SpanKind::Client)
        .with_attributes([
            KeyValue::new("http.request.method", method),
            KeyValue::new("url.path", request.url().path().to_owned()),
            KeyValue::new(
                "server.address",
                request.url().host_str().unwrap_or_default().to_owned(),
            ),
        ])
        .start_with_context(&tracer, &parent);
    let cx = parent.with_span(span);
    if propagate {
        global::get_text_map_propagator(|p| {
            p.inject_context(&cx, &mut HeaderInjector(request.headers_mut()))
        });
    }

    let result = client.execute(request).with_context(cx.clone()).await;
    let span = cx.span();
    match &result {
        Ok(response) => {
            let status = response.status();
            span.set_attribute(KeyValue::new(
                "http.response.status_code",
                i64::from(status.as_u16()),
            ));
            if !status.is_success() {
                span.set_status(Status::error(status.to_string()));
            }
        }
        Err(e) => span.set_status(Status::error(e.to_string())),
    }
    span.end();
    result
}

/// `/internal/analyses/<uuid>/stages/fetch` → `/internal/analyses/{id}/stages/fetch`,
/// and numeric IDs (`/app/installations/42/access_tokens`) the same way.
pub fn templated(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            let uuid = seg.len() == 36 && seg.matches('-').count() == 4;
            let number = !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit());
            if uuid || number {
                "{id}"
            } else {
                seg
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Runs one LLM call as a client span following the OpenTelemetry GenAI conventions
/// (`chat <model>`, provider, models, token usage). Only what [`crate::llm::Answer`]
/// already exposes is recorded — never the prompt, the response or the key — and a
/// failure carries the same one-line reason the caller gets.
pub async fn llm_call(
    provider: &str,
    request_model: &str,
    url: &str,
    work: impl Future<Output = Result<crate::llm::Answer, String>>,
) -> Result<crate::llm::Answer, String> {
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_default();
    let tracer = global::tracer(SCOPE);
    let parent = Context::current();
    let span = tracer
        .span_builder(format!("chat {request_model}"))
        .with_kind(SpanKind::Client)
        .with_attributes([
            KeyValue::new("gen_ai.operation.name", "chat"),
            KeyValue::new("gen_ai.provider.name", provider.to_owned()),
            KeyValue::new("gen_ai.request.model", request_model.to_owned()),
            KeyValue::new("server.address", host),
        ])
        .start_with_context(&tracer, &parent);
    let cx = parent.with_span(span);

    let result = work.with_context(cx.clone()).await;
    let span = cx.span();
    match &result {
        Ok(answer) => {
            span.set_attribute(KeyValue::new("gen_ai.response.model", answer.model.clone()));
            span.set_attribute(KeyValue::new(
                "gen_ai.usage.input_tokens",
                answer.input_tokens,
            ));
            span.set_attribute(KeyValue::new(
                "gen_ai.usage.output_tokens",
                answer.output_tokens,
            ));
        }
        Err(reason) => span.set_status(Status::error(reason.clone())),
    }
    span.end();
    result
}
