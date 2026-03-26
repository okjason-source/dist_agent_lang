//! Process-wide tracing (`tracing` + `tracing-subscriber`) and lightweight HTTP counters
//! for operations (Prometheus text when `DAL_METRICS=1`).
//!
//! **Startup:** call [`init_tracing`] once (the `dal` binary does this at process start).
//! **Env:** `RUST_LOG`, `DAL_LOG_FORMAT=json`, `DAL_METRICS=1` — see `docs/CONFIG.md` in the repo.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;
use std::time::Instant;

static TRACING_INIT: Once = Once::new();

static HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_STATUS_4XX: AtomicU64 = AtomicU64::new(0);
static HTTP_STATUS_5XX: AtomicU64 = AtomicU64::new(0);

/// Initialize `tracing` subscriber and bridge `log::` macros into tracing (`tracing-log`).
/// Safe to call multiple times; only the first call has effect.
pub fn init_tracing() {
    TRACING_INIT.call_once(|| {
        if try_init_tracing_inner().is_err() {
            // Another subscriber may already be installed (e.g. tests) — continue without panic.
        }
    });
}

fn try_init_tracing_inner() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let _ = tracing_log::LogTracer::init();

    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("warn,dist_agent_lang=info,tower_http=info,security=warn,dal_http=info")
    });

    let fmt_layer = if std::env::var("DAL_LOG_FORMAT").unwrap_or_default() == "json" {
        fmt::layer().json().boxed()
    } else {
        fmt::layer().boxed()
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()?;
    Ok(())
}

/// Prometheus exposition format (counters only). Intended for `/metrics` when enabled.
pub fn prometheus_metrics_text() -> String {
    let n = HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed);
    let x4 = HTTP_STATUS_4XX.load(Ordering::Relaxed);
    let x5 = HTTP_STATUS_5XX.load(Ordering::Relaxed);
    format!(
        "# HELP dal_http_requests_total Total HTTP responses recorded by dal observability middleware.\n\
         # TYPE dal_http_requests_total counter\n\
         dal_http_requests_total {n}\n\
         # HELP dal_http_responses_4xx_total HTTP responses with 4xx status.\n\
         # TYPE dal_http_responses_4xx_total counter\n\
         dal_http_responses_4xx_total {x4}\n\
         # HELP dal_http_responses_5xx_total HTTP responses with 5xx status.\n\
         # TYPE dal_http_responses_5xx_total counter\n\
         dal_http_responses_5xx_total {x5}\n"
    )
}

fn record_http_response(status: u16) {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    if (400..500).contains(&status) {
        HTTP_STATUS_4XX.fetch_add(1, Ordering::Relaxed);
    } else if status >= 500 {
        HTTP_STATUS_5XX.fetch_add(1, Ordering::Relaxed);
    }
}

/// Axum handler: returns Prometheus text if `DAL_METRICS=1`, else 404.
pub async fn metrics_http_response() -> impl IntoResponse {
    if std::env::var("DAL_METRICS").ok().as_deref() != Some("1") {
        return (StatusCode::NOT_FOUND, Body::empty()).into_response();
    }
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        prometheus_metrics_text(),
    )
        .into_response()
}

/// Logs one structured `tracing` event per response and updates counters (when enabled).
pub async fn http_observability_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_owned();

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let ms = start.elapsed().as_millis() as u64;

    record_http_response(status);

    tracing::info!(
        target: "dal_http",
        method = %method,
        path = %path,
        status = status,
        latency_ms = ms,
        "http_request"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_text_includes_series_names() {
        let s = prometheus_metrics_text();
        assert!(s.contains("dal_http_requests_total"));
        assert!(s.contains("dal_http_responses_4xx_total"));
        assert!(s.contains("dal_http_responses_5xx_total"));
    }
}
