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
static HTTP_AUTH_REJECTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_RATE_LIMIT_REJECTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_ACTIVE: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_ACTIVE: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_CONNECTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_CONNECTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_RESUME_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_RESUME_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_GAP_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_GAP_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_REPLAY_EVICTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_REPLAY_EVICTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_LAGGED_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_LAGGED_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_RUN_STREAM_RECV_CLOSED_TOTAL: AtomicU64 = AtomicU64::new(0);
static IDE_SSE_EVENTS_STREAM_RECV_CLOSED_TOTAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdeSseEndpoint {
    Run,
    Events,
}

impl IdeSseEndpoint {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Events => "events",
        }
    }
}

pub struct IdeSseActiveGuard {
    endpoint: IdeSseEndpoint,
}

impl Drop for IdeSseActiveGuard {
    fn drop(&mut self) {
        match self.endpoint {
            IdeSseEndpoint::Run => {
                IDE_SSE_RUN_STREAM_ACTIVE.fetch_sub(1, Ordering::Relaxed);
            }
            IdeSseEndpoint::Events => {
                IDE_SSE_EVENTS_STREAM_ACTIVE.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }
}

pub fn ide_sse_stream_open(endpoint: IdeSseEndpoint) -> IdeSseActiveGuard {
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_STREAM_ACTIVE.fetch_add(1, Ordering::Relaxed);
            IDE_SSE_RUN_STREAM_CONNECTIONS_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_STREAM_ACTIVE.fetch_add(1, Ordering::Relaxed);
            IDE_SSE_EVENTS_STREAM_CONNECTIONS_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
    }
    IdeSseActiveGuard { endpoint }
}

pub fn ide_sse_resume(endpoint: IdeSseEndpoint) {
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_STREAM_RESUME_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_STREAM_RESUME_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub fn ide_sse_gap(endpoint: IdeSseEndpoint) {
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_STREAM_GAP_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_STREAM_GAP_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub fn ide_sse_replay_evictions(endpoint: IdeSseEndpoint, count: u64) {
    if count == 0 {
        return;
    }
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_REPLAY_EVICTIONS_TOTAL.fetch_add(count, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_REPLAY_EVICTIONS_TOTAL.fetch_add(count, Ordering::Relaxed);
        }
    }
}

pub fn ide_sse_lagged(endpoint: IdeSseEndpoint, dropped: u64) {
    if dropped == 0 {
        return;
    }
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_STREAM_LAGGED_TOTAL.fetch_add(dropped, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_STREAM_LAGGED_TOTAL.fetch_add(dropped, Ordering::Relaxed);
        }
    }
}

pub fn ide_sse_recv_closed(endpoint: IdeSseEndpoint) {
    match endpoint {
        IdeSseEndpoint::Run => {
            IDE_SSE_RUN_STREAM_RECV_CLOSED_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
        IdeSseEndpoint::Events => {
            IDE_SSE_EVENTS_STREAM_RECV_CLOSED_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
    }
}

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
    let auth_reject = HTTP_AUTH_REJECTIONS_TOTAL.load(Ordering::Relaxed);
    let rate_reject = HTTP_RATE_LIMIT_REJECTIONS_TOTAL.load(Ordering::Relaxed);
    let run_active = IDE_SSE_RUN_STREAM_ACTIVE.load(Ordering::Relaxed);
    let events_active = IDE_SSE_EVENTS_STREAM_ACTIVE.load(Ordering::Relaxed);
    let run_connections = IDE_SSE_RUN_STREAM_CONNECTIONS_TOTAL.load(Ordering::Relaxed);
    let events_connections = IDE_SSE_EVENTS_STREAM_CONNECTIONS_TOTAL.load(Ordering::Relaxed);
    let run_resume = IDE_SSE_RUN_STREAM_RESUME_TOTAL.load(Ordering::Relaxed);
    let events_resume = IDE_SSE_EVENTS_STREAM_RESUME_TOTAL.load(Ordering::Relaxed);
    let run_gap = IDE_SSE_RUN_STREAM_GAP_TOTAL.load(Ordering::Relaxed);
    let events_gap = IDE_SSE_EVENTS_STREAM_GAP_TOTAL.load(Ordering::Relaxed);
    let run_evict = IDE_SSE_RUN_REPLAY_EVICTIONS_TOTAL.load(Ordering::Relaxed);
    let events_evict = IDE_SSE_EVENTS_REPLAY_EVICTIONS_TOTAL.load(Ordering::Relaxed);
    let run_lagged = IDE_SSE_RUN_STREAM_LAGGED_TOTAL.load(Ordering::Relaxed);
    let events_lagged = IDE_SSE_EVENTS_STREAM_LAGGED_TOTAL.load(Ordering::Relaxed);
    let run_closed = IDE_SSE_RUN_STREAM_RECV_CLOSED_TOTAL.load(Ordering::Relaxed);
    let events_closed = IDE_SSE_EVENTS_STREAM_RECV_CLOSED_TOTAL.load(Ordering::Relaxed);
    format!(
        "# HELP dal_http_requests_total Total HTTP responses recorded by dal observability middleware.\n\
         # TYPE dal_http_requests_total counter\n\
         dal_http_requests_total {n}\n\
         # HELP dal_http_responses_4xx_total HTTP responses with 4xx status.\n\
         # TYPE dal_http_responses_4xx_total counter\n\
         dal_http_responses_4xx_total {x4}\n\
         # HELP dal_http_responses_5xx_total HTTP responses with 5xx status.\n\
         # TYPE dal_http_responses_5xx_total counter\n\
         dal_http_responses_5xx_total {x5}\n\
         # HELP dal_http_auth_rejections_total HTTP auth rejection responses (401/403).\n\
         # TYPE dal_http_auth_rejections_total counter\n\
         dal_http_auth_rejections_total {auth_reject}\n\
         # HELP dal_http_rate_limit_rejections_total HTTP rate-limit rejection responses (429).\n\
         # TYPE dal_http_rate_limit_rejections_total counter\n\
         dal_http_rate_limit_rejections_total {rate_reject}\n\
         # HELP dal_ide_sse_run_stream_active Active IDE run SSE streams.\n\
         # TYPE dal_ide_sse_run_stream_active gauge\n\
         dal_ide_sse_run_stream_active {run_active}\n\
         # HELP dal_ide_sse_events_stream_active Active IDE activity SSE streams.\n\
         # TYPE dal_ide_sse_events_stream_active gauge\n\
         dal_ide_sse_events_stream_active {events_active}\n\
         # HELP dal_ide_sse_run_stream_connections_total Total IDE run SSE stream connections.\n\
         # TYPE dal_ide_sse_run_stream_connections_total counter\n\
         dal_ide_sse_run_stream_connections_total {run_connections}\n\
         # HELP dal_ide_sse_events_stream_connections_total Total IDE activity SSE stream connections.\n\
         # TYPE dal_ide_sse_events_stream_connections_total counter\n\
         dal_ide_sse_events_stream_connections_total {events_connections}\n\
         # HELP dal_ide_sse_run_stream_resume_total Total IDE run SSE stream resume attempts.\n\
         # TYPE dal_ide_sse_run_stream_resume_total counter\n\
         dal_ide_sse_run_stream_resume_total {run_resume}\n\
         # HELP dal_ide_sse_events_stream_resume_total Total IDE activity SSE stream resume attempts.\n\
         # TYPE dal_ide_sse_events_stream_resume_total counter\n\
         dal_ide_sse_events_stream_resume_total {events_resume}\n\
         # HELP dal_ide_sse_run_stream_gap_total IDE run SSE replay gap events.\n\
         # TYPE dal_ide_sse_run_stream_gap_total counter\n\
         dal_ide_sse_run_stream_gap_total {run_gap}\n\
         # HELP dal_ide_sse_events_stream_gap_total IDE activity SSE replay gap events.\n\
         # TYPE dal_ide_sse_events_stream_gap_total counter\n\
         dal_ide_sse_events_stream_gap_total {events_gap}\n\
         # HELP dal_ide_sse_run_replay_evictions_total IDE run replay buffer evictions.\n\
         # TYPE dal_ide_sse_run_replay_evictions_total counter\n\
         dal_ide_sse_run_replay_evictions_total {run_evict}\n\
         # HELP dal_ide_sse_events_replay_evictions_total IDE activity replay buffer evictions.\n\
         # TYPE dal_ide_sse_events_replay_evictions_total counter\n\
         dal_ide_sse_events_replay_evictions_total {events_evict}\n\
         # HELP dal_ide_sse_run_stream_lagged_total IDE run SSE lagged dropped event count.\n\
         # TYPE dal_ide_sse_run_stream_lagged_total counter\n\
         dal_ide_sse_run_stream_lagged_total {run_lagged}\n\
         # HELP dal_ide_sse_events_stream_lagged_total IDE activity SSE lagged dropped event count.\n\
         # TYPE dal_ide_sse_events_stream_lagged_total counter\n\
         dal_ide_sse_events_stream_lagged_total {events_lagged}\n\
         # HELP dal_ide_sse_run_stream_recv_closed_total IDE run SSE receive closed errors.\n\
         # TYPE dal_ide_sse_run_stream_recv_closed_total counter\n\
         dal_ide_sse_run_stream_recv_closed_total {run_closed}\n\
         # HELP dal_ide_sse_events_stream_recv_closed_total IDE activity SSE receive closed errors.\n\
         # TYPE dal_ide_sse_events_stream_recv_closed_total counter\n\
         dal_ide_sse_events_stream_recv_closed_total {events_closed}\n"
    )
}

fn record_http_response(status: u16) {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    if (400..500).contains(&status) {
        HTTP_STATUS_4XX.fetch_add(1, Ordering::Relaxed);
        if status == 401 || status == 403 {
            HTTP_AUTH_REJECTIONS_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
        if status == 429 {
            HTTP_RATE_LIMIT_REJECTIONS_TOTAL.fetch_add(1, Ordering::Relaxed);
        }
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
        assert!(s.contains("dal_http_auth_rejections_total"));
        assert!(s.contains("dal_http_rate_limit_rejections_total"));
        assert!(s.contains("dal_ide_sse_run_stream_active"));
        assert!(s.contains("dal_ide_sse_events_stream_active"));
        assert!(s.contains("dal_ide_sse_run_stream_connections_total"));
        assert!(s.contains("dal_ide_sse_events_stream_connections_total"));
        assert!(s.contains("dal_ide_sse_run_stream_resume_total"));
        assert!(s.contains("dal_ide_sse_events_stream_resume_total"));
        assert!(s.contains("dal_ide_sse_run_stream_gap_total"));
        assert!(s.contains("dal_ide_sse_events_stream_gap_total"));
        assert!(s.contains("dal_ide_sse_run_replay_evictions_total"));
        assert!(s.contains("dal_ide_sse_events_replay_evictions_total"));
        assert!(s.contains("dal_ide_sse_run_stream_lagged_total"));
        assert!(s.contains("dal_ide_sse_events_stream_lagged_total"));
        assert!(s.contains("dal_ide_sse_run_stream_recv_closed_total"));
        assert!(s.contains("dal_ide_sse_events_stream_recv_closed_total"));
    }
}
