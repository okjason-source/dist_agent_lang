# Spec: IDE SSE + MCP Bridge Production Readiness

Status: Draft checklist/spec hybrid  
Owner: OKJason  
Last updated: 2026-04-09

## Goal

Deliver production-grade streaming for:

- `dal ide serve` SSE endpoints (reliable, secure, observable).
- MCP bridge transport with both local stdio and remote HTTP streaming support.

This document is intentionally a living checklist first, and a final spec artifact second.

---

## Scope

In scope:

- SSE hardening in IDE backend.
- MCP bridge dual transport (`stdio` + `http-stream`).
- Security defaults, auth, CORS, and limits for streaming paths.
- Test coverage (unit/integration/soak) and rollout controls.
- Documentation and operator runbooks.

Out of scope (for this phase):

- Replacing WebSocket-based LSP transport.
- Rewriting non-streaming DAL HTTP endpoints unless required for parity.

---

## Current baseline (as of 2026-04-09)

- SSE exists in `src/ide/server.rs`:
  - `GET /api/run/stream/:job_id`
  - `GET /api/events/stream`
- MCP bridge command exists in `src/main.rs` with `stdio` and `http-stream` transport selection (`dal mcp-bridge`).
- Dedicated SSE regression suite exists in `tests/ide_sse_phase0_tests.rs`.
- Explicit remote MCP HTTP transport contract is present, but full remote stream runtime parity remains open.

---

## Production acceptance criteria

- [x] No silent event loss for normal reconnect scenarios.
- [x] Clear stream contract (event schema, terminal semantics, reconnect semantics).
- [x] MCP behavior parity across stdio and HTTP streaming transports.
- [x] Secure-by-default profile with explicit dev override.
- [x] Metrics/logging/alerts available for stream health and failures.
- [ ] Documented rollout + rollback path validated in staging.

---

## Workstream A: IDE SSE reliability

### A1. Event contract and framing

- [x] Define SSE event schema (`id`, `type`, `timestamp`, `payload`, `version`).
- [x] Define terminal event semantics (`done`, `cancelled`, `error`).
- [x] Define recoverable error/gap event semantics.
- [x] Add compatibility note for existing consumers.

### A2. Replay and resume support

- [x] Implement bounded replay buffer for job stream (`/api/run/stream/:job_id`).
- [x] Implement bounded replay buffer for global events stream (`/api/events/stream`).
- [x] Support `Last-Event-ID` resume behavior.
- [x] Emit deterministic `id` values (monotonic per stream).

### A3. Stream lifecycle correctness

- [x] Ensure completed jobs are removed from in-memory registry after retention window.
- [x] Ensure stop/cancel/natural completion emit explicit terminal events.
- [x] Ensure lagged consumers do not fail silently.
- [x] Ensure keepalive cadence is configurable and documented.

### A4. Backpressure and limits

- [x] Define ring buffer capacity and retention policy.
- [x] Cap per-message size and truncate with explicit metadata when needed.
- [x] Add per-client stream concurrency limit.
- [x] Add rate limit for stream establishment/reconnect bursts.

---

## Workstream B: MCP bridge dual transport

### B1. Transport abstraction

- [x] Introduce explicit transport selection: `stdio` (default) and `http-stream`.
- [x] Add CLI flags/env mapping (for example `--transport`, `DAL_MCP_TRANSPORT`).
- [x] Keep current stdio behavior as fully backward compatible default.

### B2. HTTP streaming transport

- [x] Implement remote HTTP streaming session lifecycle (connect, heartbeat, close).
- [x] Define request/response envelope parity with stdio mode.
- [x] Add retry/backoff guidance for remote clients.
- [x] Add timeout/cancellation semantics.

### B3. MCP parity guarantees

- [x] Produce parity matrix for supported tools and error classes.
- [x] Ensure identical user-visible tool behavior across transports.
- [x] Validate idempotency/retry handling where applicable.
- [x] Document known differences (if any) and reasons.

Reference: `docs/guides/MCP_TRANSPORT_PARITY.md`

---

## Workstream C: Security hardening

### C1. CORS and auth policy

- [x] Tighten IDE SSE CORS defaults for production profile.
- [x] Keep permissive local dev mode explicit and opt-in.
- [x] Add optional token/JWT protection for stream endpoints.
- [x] Define auth exemption policy for health/metrics endpoints.

### C2. Abuse resistance

- [x] Rate-limit stream connection attempts.
- [x] Add max stream lifetime and idle timeout controls.
- [x] Add payload/body/header size limits for relevant paths.
- [x] Validate behavior under reconnect storms.

### C3. Threat model and policy docs

- [x] Write threat model notes for local-only vs remote deployments.
- [x] Define recommended deployment presets (`legacy`, `balanced`, `strict`).
- [x] Document secure defaults and emergency rollback toggles.

---

## Workstream D: Tests and quality gates

### D1. Automated tests

- [x] Add SSE contract tests (headers, keepalive, event format).
- [x] Add replay/resume tests (`Last-Event-ID`).
- [x] Add job lifecycle cleanup tests.
- [x] Add lag/backpressure tests.
- [x] Add MCP stdio vs HTTP parity tests.

### D2. Soak and fault tests

- [x] Long-lived stream soak test (>= 30 min). (Validated via `scripts/soak/run_ide_sse_soak.sh 1800` — pass, ~1802.59s)
- [x] Network interruption/reconnect test.
- [x] Burst reconnect test.
- [x] Forced process kill/cancel path test.

### D3. CI gates

- [x] Add/extend CI jobs to run new streaming test suites.
- [x] Define minimum passing matrix before release. (See `docs/guides/SSE_RELEASE_MINIMUM_MATRIX.md`)
- [x] Add regression guard for event schema drift.

---

## Workstream E: Observability and operations

### E1. Metrics

- [x] Active stream gauges by endpoint.
- [x] Reconnect counters and success/failure rates.
- [x] Dropped/lagged event counters.
- [x] Buffer utilization gauges and eviction counters.
- [x] Auth/rate-limit rejection counters.

### E2. Logging and tracing

- [x] Structured logs with `job_id`, `stream_id`, `client_id` correlation.
- [x] Consistent error taxonomy for stream failures.
- [x] Trace key stream lifecycle spans.

### E3. Runbooks

- [x] On-call playbook for stream outage.
- [x] Playbook for reconnect storm and backpressure incidents.
- [x] Rollback procedure with toggle checklist.

---

## Workstream F: dal-ide required changes

Purpose: keep the frontend client compatible while backend SSE/MCP contracts evolve.

### F1. Stream client compatibility (`dal-ide/src/main.js`)

- [x] Add dual parser support for legacy raw `onmessage` payloads and structured event envelopes.
- [x] Preserve compatibility with current keepalive behavior while supporting new heartbeat framing.
- [x] Support explicit terminal outcomes (`done`, `cancelled`, `error`) in addition to legacy `"[DONE]"`.
- [x] Ensure run/output UI state transitions are driven by terminal event semantics, not socket close alone.

### F2. Reconnect and resume behavior

- [x] Implement reconnect flow that can leverage `Last-Event-ID` (or equivalent replay token) when available.
- [x] Track last processed event id separately for run stream vs activity stream.
- [x] Handle gap/lag notifications with user-visible status and safe fallback behavior.
- [x] Add exponential backoff/jitter for repeated reconnect failures.

### F3. Auth model for browser stream clients

- [x] Decide browser-compatible auth strategy for SSE (`cookie/session` vs query token vs fetch-stream migration).
- [x] If token-based auth is required, avoid relying on custom headers with native `EventSource`.
- [x] Add frontend handling for auth failures (401/403) with actionable UX and retry guidance.
- [x] Document local-dev versus hosted auth wiring in `dal-ide` docs.

### F4. UX and operator visibility

- [x] Add UI indicator for stream status (`connected`, `reconnecting`, `degraded`, `disconnected`).
- [x] Surface stream reliability warnings (event loss risk, replay window exceeded).
- [x] Add debug panel/log hooks for stream metadata in development builds.
- [x] Ensure stop/run controls remain consistent during reconnect windows.

### F5. dal-ide tests and docs

- [x] Add frontend tests for event parsing compatibility (legacy + new schema).
- [x] Add tests for reconnect/resume and terminal state handling.
- [x] Update `dal-ide/IDE design/API_REFERENCE.md` with final stream contract.
- [x] Update `dal-ide/README.md` with hosted deployment/auth notes.

---

## Rollout plan checklist

### Stage 0: Local/dev

- [x] Ship behind flags with compatibility defaults.
- [x] Validate baseline behavior unchanged for existing clients.

### Stage 1: Internal/staging

- [x] Enable replay/resume.
- [x] Enable security profile candidate.
- [x] Run soak tests and capture metrics baselines.

### Stage 2: Canary production

- [ ] Enable for small traffic slice.
- [ ] Monitor error budgets and rollback triggers.
- [ ] Confirm MCP transport parity in real workflows.

### Stage 3: General availability

- [ ] Promote secure defaults.
- [ ] Publish final docs and migration notes.
- [ ] Archive this checklist into final implementation spec.

---

## File-level implementation map

Primary expected touch points:

- `src/ide/server.rs`
- `src/ide/run_backend.rs`
- `src/main.rs`
- `tests/` (new SSE and MCP parity suites)
- `docs/CONFIG.md`
- `docs/guides/AGENT_SETUP_AND_USAGE.md` (or equivalent integration guide)
- MCP bridge script path resolved by `dal mcp-bridge` (for example `COO/mcp/src/server.js`, or `../COO/mcp/src/server.js` from `dist_agent_lang/`, when present)

---

## Open questions (resolve before implementation lock)

- [x] Which auth primitive is canonical for IDE SSE in hosted mode (JWT vs API token)?
- [x] Required replay window and retention limits per stream type?
- [x] Should terminal events be explicit `event:` names or only `data.type`?
- [x] Required remote MCP HTTP transport compatibility target/version?
- [x] Do we support multi-tenant stream isolation now or defer?

---

## Decision log

Use this section to record non-obvious decisions and rationale.

- [x] Decision 1: Hosted SSE uses cookie/session auth as canonical browser path; `DAL_IDE_SSE_AUTH_TOKEN` + `access_token` query is the compatibility fallback for EventSource constraints.
- [x] Decision 2: Replay/retention defaults: `DAL_IDE_SSE_REPLAY_CAP=512`, `DAL_IDE_SSE_JOB_RETENTION_SECS=120`; override per deployment SLO.
- [x] Decision 3: Terminal semantics are conveyed in `data.type` (`done|cancelled|error`) with default `message` framing for EventSource compatibility.
- [x] Decision 4: MCP remote compatibility target is Streamable HTTP MCP over `/mcp` (SDK family `@modelcontextprotocol/sdk` 1.28+ behavior baseline).
- [x] Decision 5: Multi-tenant stream isolation is deferred; current scope is single-tenant per bridge process, with deployment-level isolation for tenants.

---

## Progress tracker

Legend: `[ ]` not started, `[-]` in progress, `[x]` complete, `[!]` blocked

- [x] Phase 0: Contract + flags
- [x] Phase 1: IDE SSE reliability
- [x] Phase 2: Security hardening
- [x] Phase 3: MCP dual transport
- [x] Phase 4: Tests + CI gates
- [x] Phase 5: Observability + runbooks
- [ ] Phase 6: Staged rollout

---

## Spec-sheet conversion checklist

When all items above are complete, convert this doc into final spec format:

- [ ] Replace checklist status with final implementation status summary.
- [ ] Add final architecture diagram(s).
- [ ] Add endpoint/protocol reference appendix.
- [ ] Add performance and reliability results from soak tests.
- [ ] Add migration and rollback guide.
- [ ] Mark `Status: Final`.
