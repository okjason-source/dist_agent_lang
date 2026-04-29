# MCP Transport Parity Matrix

Status: Implemented parity contract + remote `http-stream` session lifecycle.

## Scope

- CLI transport selector: `DAL_MCP_TRANSPORT=stdio|http-stream`
- DAL stdlib namespace: `mcp::*`
- Bridge script: `COO/mcp/src/server.js` (or `../COO/mcp/src/server.js` from `dist_agent_lang/`)

## Tool parity matrix (user-visible)

| Tool name | Endpoint | Request body | Response shape | Stdio | http-stream |
|---|---|---|---|---|---|
| `dal_agent_message` | `POST /api/message` | `{ content, sender_id?, policy?, include_rag? }` | `{ httpStatus, body }` (JSON text payload) | Yes | Yes |
| `dal_agent_task` | `POST /api/task` | `{ description, policy?, include_rag? }` | `{ httpStatus, body }` | Yes | Yes |
| `dal_agent_run` | `POST /api/agents/run` | `{ role, task, include_rag? }` | `{ httpStatus, body }` | Yes | Yes |
| `dal_agent_workflow` | `POST /api/workflow` | `{ workflow, input, include_rag? }` | `{ httpStatus, body }` | Yes | Yes |

## Error class parity

- Transport selection:
  - Unsupported value -> deterministic config error (`use stdio or http-stream`).
- Invocation:
  - Unsupported tool name -> deterministic `mcp::invoke unsupported tool`.
  - HTTP failure -> non-success `status` with parsed `body` / `body_text`.
- Bridge lifecycle:
  - Missing bridge script or `node_modules` -> deterministic startup error.
  - Stopped bridge id -> `false` / `null` status response.

## Retry/backoff guidance

- Client policy:
  - Use exponential backoff with jitter for bridge start retries (1s, 2s, 4s, cap 8s + jitter).
  - Retry only idempotent invocations automatically.
- Server policy:
  - Preserve idempotency by sending `Idempotency-Key` on `/api/message` and `/api/task` when callers may retry.

## Timeout and cancellation semantics

- `mcp::invoke` uses bounded HTTP timeout (30s) and returns structured failure on timeout.
- `mcp::bridge_stop` is the cancellation primitive for a running bridge process.
- Workflow/run endpoints should be treated as long-running; callers should avoid aggressive retry loops.
- `http-stream` bridge mode includes session lifecycle management:
  - connect via `POST /mcp` initialize;
  - heartbeat via periodic server logging notifications;
  - close via `DELETE /mcp` (or idle session GC).

## Known differences and rationale

- Stdio remains the default for maximal local compatibility.
- `http-stream` mode requires an MCP HTTP client that sends `Accept: application/json, text/event-stream` and uses `mcp-session-id`.
- Multi-tenant isolation is deployment-level (separate bridge/server instances), not in-process bridge tenancy.
