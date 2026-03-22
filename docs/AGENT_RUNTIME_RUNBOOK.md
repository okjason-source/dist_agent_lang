# Agent Runtime Runbook

Operational runbook for host-protocol agent runtime (project baseline).

---

## Quick triage

1. Check `agent_route_metrics` logs for:
   - `termination_reason`
   - `route`, `policy`
   - `steps_used`, `max_steps_reached`
   - `guard_stopped`
2. Check API diagnostics payloads (`agent_run` / `respond_with_tools_result`):
   - `route`, `policy`, `tool_trace`
   - `termination_reason`, `guard_stopped`
   - `observability.legacy_text_protocol_enabled`
   - `observability.native_tool_calling_enabled`
3. Confirm environment toggles in the running process.

---

## Rollback / stabilization toggles

Use these env vars for safe rollback without code changes:

- `DAL_AGENT_POLICY_DEFAULT=reply_only`
  - Fastest way to reduce tool execution risk globally.
- `DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0`
  - Disables provider-native tool calling and falls back to legacy text path behavior.
- `DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=1`
  - Enables legacy JSON-in-text parser mode.
- `DAL_AGENT_GUARDS_STRICT_MODE=1`
  - Tightens default safety limits.

Optional tighter hard caps:

- `DAL_AGENT_MAX_WALL_CLOCK_MS`
- `DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE`
- `DAL_AGENT_MAX_REPEATED_IDENTICAL_INVOCATIONS`
- `DAL_AGENT_MAX_CONSECUTIVE_NO_PROGRESS`
- `DAL_AGENT_MAX_TOTAL_TOKENS`
- `DAL_AGENT_MAX_COST_MICROUSD`

---

## Deployment profiles (copy/paste)

Use one of these profiles before restart.

### Safe push profile (recommended first deploy)

```bash
export DAL_AGENT_POLICY_DEFAULT=reply_only
export DAL_AGENT_GUARDS_STRICT_MODE=1
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
```

### Fallback profile (if native/provider behavior degrades)

```bash
export DAL_AGENT_POLICY_DEFAULT=reply_only
export DAL_AGENT_GUARDS_STRICT_MODE=1
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=1
```

### Controlled ramp profile (after stability confirmed)

```bash
export DAL_AGENT_POLICY_DEFAULT=auto
export DAL_AGENT_GUARDS_STRICT_MODE=0
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
```

---

## Interpreting termination reasons

Common `termination_reason` values in `agent_route_metrics`:

- `reply` — Normal completion with final answer.
- `ask_user` — Model requested user input.
- `max_steps_reached` — Tool loop hit configured step cap.
- `guard_wall_clock` — Exceeded wall-clock limit.
- `guard_token_budget` — Exceeded token budget.
- `guard_cost_budget` — Exceeded cost budget.
- `guard_per_tool_type_limit` — One tool exceeded per-type cap.
- `guard_repeated_identical_invocation` — Same tool+args repeated too many times.
- `guard_no_progress` — Same tool+args and same result repeatedly.
- `parse_fail_terminal` — Legacy parser terminal fallback.
- `unsupported_tool_call_terminal` — Model emitted unsupported tool call name.

---

## Suggested alert thresholds

- `max_steps_reached` > 2% of turns over 15m
- Any `guard_*` termination > 1% sustained over 15m
- `parse_fail_terminal` > 0.5% when native tool calling is enabled
- p95 latency regression > 30% baseline

---

## Incident response checklist

1. Set `DAL_AGENT_POLICY_DEFAULT=reply_only`.
2. Set `DAL_AGENT_GUARDS_STRICT_MODE=1`.
3. If native provider behavior is unstable: set `DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0` and `DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=1`.
4. Restart service and verify `observability` fields reflect toggles.
5. Confirm `termination_reason` distribution returns to baseline.
6. Re-enable features gradually (`auto` policy, native tool calling) after root cause review.

---

## Runbook validation checklist (5 minutes)

Use this quick loop after deploy or during an incident to verify controls are live.

1. **Policy wiring check**
   - Send one `/api/message` request with `"policy":"reply_only"` and one with `"policy":"tool_loop"`.
   - Confirm response fields include `route`, `tool_steps_count`, and `last_tool_names`.
   - Example:
     ```bash
     curl -s -X POST http://localhost:4040/api/message \
       -H 'Content-Type: application/json' \
       -d '{"content":"What is DAL?","sender_id":"web","policy":"reply_only"}' | python3 -m json.tool

     curl -s -X POST http://localhost:4040/api/message \
       -H 'Content-Type: application/json' \
       -d '{"content":"Run echo hello","sender_id":"web","policy":"tool_loop"}' | python3 -m json.tool
     ```
2. **Observability field check**
   - Call a path that uses `ai::agent_run` (or legacy-compatible `ai::respond_with_tools_result`).
   - Confirm diagnostics include:
     - `route`, `policy`, `tool_trace`
     - `termination_reason`, `guard_stopped`
     - `observability.legacy_text_protocol_enabled`
     - `observability.native_tool_calling_enabled`
   - Example:
     ```bash
     curl -s -X POST http://localhost:4040/api/task \
       -H 'Content-Type: application/json' \
       -d '{"description":"List files in current directory","policy":"auto"}' | python3 -m json.tool
     ```
3. **Guard trigger smoke check**
   - Temporarily set strict caps (very low) for one test run:
     - e.g. `DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE=1` or very low wall clock.
   - Trigger a tool-using prompt and confirm stop message begins with `Stopped:`.
   - Example:
     ```bash
     export DAL_AGENT_GUARDS_STRICT_MODE=1
     export DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE=1
     curl -s -X POST http://localhost:4040/api/task \
       -H 'Content-Type: application/json' \
       -d '{"description":"Run ls and then run pwd","policy":"tool_loop"}' | python3 -m json.tool
     ```
4. **Metric shape check**
   - Inspect recent `agent_route_metrics` logs.
   - Confirm `termination_reason` and `guard_stopped` fields are present.
   - Example:
     ```bash
     # Adjust log path for your environment/runtime logger.
     rg "agent_route_metrics|termination_reason|guard_stopped" ./ -g "*.log"
     ```
5. **Rollback toggle check**
   - Set `DAL_AGENT_POLICY_DEFAULT=reply_only` and restart.
   - Confirm default requests route to `reply_only`.
   - Restore prior values after validation.
   - Example:
     ```bash
     export DAL_AGENT_POLICY_DEFAULT=reply_only
     # restart service here
     curl -s -X POST http://localhost:4040/api/message \
       -H 'Content-Type: application/json' \
       -d '{"content":"Run ls","sender_id":"web"}' | python3 -m json.tool
     ```

