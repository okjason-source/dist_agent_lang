# IDE SSE Operations Runbook

This runbook covers production operations for IDE SSE streams served by `dal ide serve`.

## Scope

- Endpoints: `GET /api/run/stream/:job_id`, `GET /api/events/stream`
- Metrics source: `GET /metrics` when `DAL_METRICS=1`
- Logs/traces: `tracing` target `dal_stream` (enable with `RUST_LOG=dal_stream=info`)

## Key indicators

- Availability:
  - `dal_ide_sse_run_stream_active`
  - `dal_ide_sse_events_stream_active`
- Connection churn:
  - `dal_ide_sse_run_stream_connections_total`
  - `dal_ide_sse_events_stream_connections_total`
  - `dal_ide_sse_run_stream_resume_total`
  - `dal_ide_sse_events_stream_resume_total`
- Reliability:
  - `dal_ide_sse_run_stream_gap_total`
  - `dal_ide_sse_events_stream_gap_total`
  - `dal_ide_sse_run_stream_lagged_total`
  - `dal_ide_sse_events_stream_lagged_total`
  - `dal_ide_sse_run_replay_evictions_total`
  - `dal_ide_sse_events_replay_evictions_total`
- Upstream closure/error:
  - `dal_ide_sse_run_stream_recv_closed_total`
  - `dal_ide_sse_events_stream_recv_closed_total`

## Error taxonomy

Use `error_code` in `dal_stream` logs:

- `channel_lagged`: receiver lagged and dropped events (recoverable, stream stays open).
- `channel_closed`: upstream broadcast closed (non-recoverable, stream closes).
- `replay_window_exceeded`: resume token older than retained replay window (gap event emitted, recoverable).

## Threat model and deployment policy

- Local-only development (`DAL_SERVE_SECURITY_PRESET=legacy`):
  - Prioritizes compatibility and local velocity.
  - Permissive CORS can remain enabled (`DAL_IDE_CORS_ALLOW_ANY=1`).
  - Stream auth token is optional.
- Hosted/internal environments (`balanced` / `strict`):
  - Prefer explicit CORS origin (`DAL_IDE_CORS_ALLOW_ORIGIN=https://your-ide.example`).
  - Enable stream auth token (`DAL_IDE_SSE_AUTH_TOKEN`) or same-origin session auth.
  - Keep abuse controls enabled (`DAL_IDE_SSE_MAX_STREAMS_PER_CLIENT`, `DAL_IDE_SSE_MAX_ESTABLISH_PER_MINUTE`, idle/lifetime caps).
- Health/metrics policy:
  - `/health` and `/metrics` are exempt from stream-token checks.
  - Restrict these endpoints at reverse proxy/network layer for hosted deployments.

## Incident: stream outage

Symptoms:

- Frontend shows disconnected/reconnecting.
- Active stream gauges drop to zero unexpectedly.
- `channel_closed` errors spike.

Immediate actions:

1. Confirm IDE process health and endpoint availability (`/health`, `/metrics`).
2. Check recent `dal_stream` lifecycle logs for `phase=close` and `reason`.
3. Validate replay settings:
   - `DAL_IDE_SSE_STRUCTURED=1`
   - `DAL_IDE_SSE_REPLAY=1`
   - `DAL_IDE_SSE_REPLAY_CAP` sized for expected reconnect windows
4. Restart service if channel closure is caused by upstream crash or rolling restart mismatch.

Recovery validation:

- Active gauges return to non-zero during client traffic.
- Connection totals increase with stable close rate.
- Frontend run/events indicators converge to `connected` or `idle` as expected.

## Incident: reconnect storm or backpressure

Symptoms:

- Resume counters and gap counters climb rapidly.
- Lagged counters increase continuously.
- Elevated client reconnect attempts from UI telemetry.

Immediate actions:

1. Increase `DAL_IDE_SSE_REPLAY_CAP` to absorb reconnect bursts.
2. Reduce event fanout pressure (temporarily lower noisy activity emitters if possible).
3. Check host CPU/memory saturation and network health.
4. If lag is run-stream specific, inspect long-running job output rates.

Stabilization:

- Keep alerting on lagged and gap deltas until rates normalize.
- Capture a before/after snapshot of counters for postmortem.

## Rollback procedure (toggle checklist)

If a rollout degrades stream reliability, revert to compatibility mode:

1. Set `DAL_IDE_SSE_STRUCTURED=0`.
2. Set `DAL_IDE_SSE_REPLAY=0`.
3. Restart IDE server.
4. Verify legacy stream behavior with a run from the IDE.
5. Confirm no rapid growth in recv-closed counters.
6. Re-enable flags only after root cause is understood.

## Alerting suggestions

- Page: sustained growth in `*_recv_closed_total` combined with low active gauges.
- Ticket: rising `*_lagged_total` or `*_gap_total` above normal baseline.
- Warn: replay evictions increasing faster than expected traffic growth.

## Quick validation commands

Use these before rollout or after policy/config changes.

### SSE policy stability matrix

From repo root:

```bash
scripts/policy/check_sse_policy_matrix.sh
```

### MCP `http-stream` lifecycle integration

From repo root:

```bash
cd COO/mcp
# or, from dist_agent_lang/: cd ../COO/mcp
npm ci
npm test
```

Expected: both tests pass (`initialize/list/delete lifecycle`, `session idle gc expires sessions`).
