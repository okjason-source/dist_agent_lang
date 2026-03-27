# DAL CEO Runtime Runbook

Operational runbook for DAL CEO host-protocol behavior.

---

## Quick triage

1. Check `agent_route_metrics` logs:
   - `route`, `policy`
   - `termination_reason`, `guard_stopped`
   - `steps_used`, `max_steps_reached`
2. Check API payload diagnostics from `/api/message` and `/api/task`:
   - `route`, `policy`, `tool_trace`
   - `termination_reason`, `guard_stopped`
   - `observability.legacy_text_protocol_enabled`
   - `observability.native_tool_calling_enabled`
3. Confirm running env toggles.

---

## Rollout/rollback toggles

- `DAL_AGENT_POLICY_DEFAULT=reply_only|auto|tool_loop`
- `DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0|1`
- `DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0|1`
- `DAL_AGENT_GUARDS_STRICT_MODE=0|1`

Optional hard caps:

- `DAL_AGENT_MAX_WALL_CLOCK_MS`
- `DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE`
- `DAL_AGENT_MAX_REPEATED_IDENTICAL_INVOCATIONS`
- `DAL_AGENT_MAX_CONSECUTIVE_NO_PROGRESS`
- `DAL_AGENT_MAX_TOTAL_TOKENS`
- `DAL_AGENT_MAX_COST_MICROUSD`

---

## Validation loop

### 1) Policy wiring

```bash
curl -s -X POST http://localhost:4040/api/message \
  -H 'Content-Type: application/json' \
  -d '{"content":"What is DAL?","sender_id":"web","policy":"reply_only"}' | python3 -m json.tool

curl -s -X POST http://localhost:4040/api/message \
  -H 'Content-Type: application/json' \
  -d '{"content":"Run echo hello","sender_id":"web","policy":"tool_loop"}' | python3 -m json.tool
```

Expect: `route`, `policy`, `tool_steps_count`, `tool_trace` (or `last_tool_names`),
`termination_reason`, `guard_stopped`.

### 2) Observability fields

```bash
curl -s -X POST http://localhost:4040/api/task \
  -H 'Content-Type: application/json' \
  -d '{"description":"List files in current directory","policy":"auto"}' | python3 -m json.tool
```

Expect nested `observability` flags and top-level diagnostics.

### 3) Guard smoke

```bash
export DAL_AGENT_GUARDS_STRICT_MODE=1
export DAL_AGENT_MAX_TOOL_CALLS_PER_TYPE=1
curl -s -X POST http://localhost:4040/api/task \
  -H 'Content-Type: application/json' \
  -d '{"description":"Run ls and then run pwd","policy":"tool_loop"}' | python3 -m json.tool
```

Expect `result` (or final text) beginning with `Stopped:`.

### 4) Metric shape

Inspect logs for:

- `agent_route_metrics`
- `termination_reason`
- `guard_stopped`

### 5) Rollback default check

```bash
export DAL_AGENT_POLICY_DEFAULT=reply_only
# restart service
curl -s -X POST http://localhost:4040/api/message \
  -H 'Content-Type: application/json' \
  -d '{"content":"Run ls","sender_id":"web"}' | python3 -m json.tool
```

Expect default `route` resolves to `reply_only` when request omits `policy`.

---

## Deployment profiles (copy/paste)

### Safe push profile

```bash
export DAL_AGENT_POLICY_DEFAULT=reply_only
export DAL_AGENT_GUARDS_STRICT_MODE=1
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
```

### Fallback profile

```bash
export DAL_AGENT_POLICY_DEFAULT=reply_only
export DAL_AGENT_GUARDS_STRICT_MODE=1
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=1
```

### Controlled ramp profile

```bash
export DAL_AGENT_POLICY_DEFAULT=auto
export DAL_AGENT_GUARDS_STRICT_MODE=0
export DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=1
export DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=0
```

---

## Interpreting termination_reason

Common values:

- `reply`
- `ask_user`
- `max_steps_reached`
- `guard_wall_clock`
- `guard_token_budget`
- `guard_cost_budget`
- `guard_per_tool_type_limit`
- `guard_repeated_identical_invocation`
- `guard_no_progress`
- `parse_fail_terminal`
- `unsupported_tool_call_terminal`

---

## Suggested alerts

- `max_steps_reached` > 2% over 15m
- any `guard_*` > 1% sustained over 15m
- `parse_fail_terminal` > 0.5% when native tool calling enabled
- p95 latency regression > 30% baseline

---

## Incident response checklist

1. Set `DAL_AGENT_POLICY_DEFAULT=reply_only`.
2. Set `DAL_AGENT_GUARDS_STRICT_MODE=1`.
3. If provider-native behavior is unstable, set:
   - `DAL_AGENT_NATIVE_TOOL_CALLS_ENABLED=0`
   - `DAL_AGENT_ENABLE_LEGACY_TEXT_JSON=1`
4. Restart service.
5. Verify API diagnostics and logs reflect expected toggles/outcomes.
6. Recover gradually to ramp profile after root-cause review.

