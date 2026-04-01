# Scatter — runtime “time to do work”

Scatter is a **process-local** timer registry in the dist_agent_lang stdlib (`scatter::`). It answers *when* something is due; your DAL code decides *what* to run by matching job ids. For a **single namespace** that combines wall-clock helpers + these timers (better for agents), see [`schedule::`](SCHEDULE.md).

**Internals:** a **schedule** map (per job id: next fire + once vs repeat), a **min-heap** of `(next fire time, id)` with lazy removal of stale entries when ids are rescheduled or cancelled, and a **due queue** (`VecDeque`) of ids ready to handle. A worker thread **sleeps until the next due time** or until a **notify** from `after_ms` / `every_ms` / `cancel`, then runs **process_due** to move fired ids into the due queue.

## Environment

| Variable | Default | Meaning |
|----------|---------|---------|
| `SCATTER_TICK_MS` | `50` | Minimum sleep when many jobs fire at the same instant (avoids a hot loop). Clamped to `1`…`3600000`. |

## API (DAL)

| Call | Meaning |
|------|---------|
| `scatter::after_ms(ms, id)` | **Relative:** one shot after `ms` milliseconds from now. Returns the id string. |
| `scatter::after_at_unix_ms(unix_ms, id)` | **Absolute:** one shot at wall-clock Unix ms (same as `after_ms(delay, id)` where `delay = time::delay_ms_until_unix_ms(unix_ms)`). See [`TIME.md`](TIME.md). |
| `scatter::every_ms(ms, id)` | Repeats every `ms` ms (`ms` must be &gt; 0). Returns the id string. |
| `scatter::cancel(id)` | Remove a scheduled job. Returns whether it existed. |
| `scatter::pending()` | Drain the **due queue**: array of job ids (FIFO) since last call. **Use this for dispatch.** |
| `scatter::peek_pending()` | Copy due ids **without** draining (debug / dashboards only). |
| `scatter::scheduled_count()` | Entries still in the **schedule** (not ids sitting in the due queue). |
| `scatter::next_due_ms()` | Ms until the next heap entry fires, or null if none (`0` if already due). |

Duplicate `id` replaces an existing schedule entry.

**Wall-clock:** use [`time::`](TIME.md) (`parse_rfc3339_unix_ms`, `unix_ms_now`, `delay_ms_until_unix_ms`) for deterministic parsing; the **app** maps LLM output to those inputs.

## Pattern

1. Register jobs with `after_ms` / `after_at_unix_ms` / `every_ms`.
2. On each turn of your control loop (e.g. HTTP handler, CLI loop, or a route dedicated to polling), call `scatter::pending()` and branch on each id to run work.

## Developer experience

Scatter is **process-local**: timers run only while the interpreter process stays alive. That shapes how you demo and ship scheduling:

| Goal | What to do |
|------|------------|
| **Try `time::` + `scatter::` in one shot** | `dal run` a script that calls `time::unix_ms_now()`, `scatter::after_ms(...)`, etc. The process may exit **before** a short timer fires; that is expected. |
| **See `pending()` actually drain** | Use **`dal serve`** with a long-lived process and poll in a handler (e.g. `GET /health` that calls `scatter::pending()`), or any loop that keeps calling `pending()` while the VM runs. |
| **Wall-clock deadlines** | Use [`TIME.md`](TIME.md) (`parse_rfc3339_unix_ms`, `delay_ms_until_unix_ms`) and `scatter::after_at_unix_ms` or `after_ms(delay, id)`. |
| **Natural-language → schedule** | Your app or agent turns chat into RFC3339 or Unix ms; the runtime only sees typed values. Optional: **recommendations → human approval → host** arms `time::` / `scatter::`. |

**Copy-paste example:** `examples/scatter_time_scheduling.dal` — `dal run` for logs, or `dal serve … --port 4040` and `curl http://127.0.0.1:4040/health` (call again after the delay to see ids in `due_this_poll`).

## Fleets

**Fleets** name groups of agent ids and optional deploy tasks (see `fleet::` and `docs/FLEET_DEPLOYMENT.md`). Scatter does **not** deliver timers to other processes.

Sensible composition:

- Use Scatter in the **coordinator** process for cadence (e.g. `"fleet:tick"`, `"reconcile"`).
- In the handler for that id, call `fleet::run(...)`, `fleet::deploy(...)`, or your own orchestration.

So Scatter “fits” fleets as **clock + policy in one VM**; cross-node fleet execution stays a separate concern (HTTP, queues, etc.).
