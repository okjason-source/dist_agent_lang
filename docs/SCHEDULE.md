# `schedule::` — unified time + scatter for agents

The **`schedule::`** stdlib module wraps [`time::`](TIME.md) and [`scatter::`](SCATTER.md) behind **one namespace** so DAL code and LLM-generated code can express “when” without juggling two modules.

**Semantics:** identical to Scatter — **process-local** timers only. For **durable** jobs across process restarts, use an application store (e.g. COO `POST /api/schedule`).

## Functions (DAL)

| Call | Meaning |
|------|---------|
| `schedule::once_after_ms(id, delay_ms)` | One-shot after a delay from now. |
| `schedule::once_at_unix_ms(id, unix_ms)` | One-shot at wall-clock Unix ms. |
| `schedule::once_at_rfc3339(id, rfc3339)` | Parse RFC3339 / naive UTC (see `time::parse_rfc3339_or_naive_utc_unix_ms`), then one-shot. |
| `schedule::every_seconds(id, n)` | Repeat every `n` seconds (`n` &gt; 0). |
| `schedule::every_minutes(id, n)` | Repeat every `n` minutes. |
| `schedule::every_hours(id, n)` | Repeat every `n` hours. |
| `schedule::series_interval_unix_ms(prefix, start_unix_ms, interval_ms, count)` | `count` one-shots at `start + i * interval` for `i in 0..count`. Ids: `prefix_0` … `prefix_{count-1}`. Max `count` = 10_000. |
| `schedule::series_interval_from_now_ms(prefix, interval_ms, count)` | Same with `start = time::unix_ms_now()`. |
| `schedule::cancel(id)` | Remove a job (delegates to `scatter::cancel`). |
| `schedule::pending()` | Drain due ids (delegates to `scatter::pending`). |
| `schedule::peek_pending()` | Non-draining snapshot. |
| `schedule::scheduled_count()` | Jobs in the heap (delegates to `scatter::scheduled_count`). |
| `schedule::next_due_ms()` | Ms until next fire, or null. |

**Argument order:** job **`id` first**, then time parameters — easier to read than `scatter::after_ms(ms, id)` when authoring for agents.

## Example: recurring cadence

```dal
schedule::every_minutes("heartbeat", 15);
```

## Example: six posts spaced by two hours (from now)

Two hours = `2 * 60 * 60 * 1000` ms:

```dal
let ids = schedule::series_interval_from_now_ms("xpost", 7200000, 6);
```

Dispatch in a route or loop with `schedule::pending()` and match on `xpost_0`, …

## Relationship to `scatter::`

`schedule::` does not add a second timer implementation; it **delegates** to `scatter::`. Prefer **`schedule::`** for new agent-facing DAL; use **`scatter::`** directly when you need the raw `(milliseconds, id)` argument order or minimal surface area.

See also: [`TIME.md`](TIME.md), [`SCATTER.md`](SCATTER.md), `examples/schedule_stdlib.dal`.
