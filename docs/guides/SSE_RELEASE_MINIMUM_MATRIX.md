# SSE Release Minimum Passing Matrix

Minimum gates required before promoting SSE changes to release stages.

## CI-required gates

All of the following must pass in CI:

1. `Code Quality Check` job (`cargo check`, fmt, clippy)
2. `Test Suite` job (default + optional feature tests)
3. `Streaming Reliability Gates` job:
   - `cargo test --test ide_sse_phase0_tests`
   - `cargo test mcp_bridge_transport_tests --bin dal`
   - exact schema drift guard execution:
     - `sse_envelope_schema_drift_guard_for_run_and_activity_streams`

## Manual/ops-required gate before production promotion

Run long soak harness and capture results:

- `scripts/soak/run_ide_sse_soak.sh 1800`
- Expected:
  - soak test passes for full duration
  - no replay `gap` assertion failures
  - reconnect windows continue advancing `id` cursor

## Promotion decision checklist

- Stage 0 (local/dev): all CI-required gates pass.
- Stage 1 (internal/staging): CI-required gates pass + one successful 30-minute soak evidence run.
- Stage 2 (canary): same as Stage 1, plus no critical stream regression alerts.
- Stage 3 (GA): maintain green CI and soak evidence for the release candidate.
