# `time::` — wall-clock helpers (language)

Deterministic **Unix millisecond** time and **RFC3339** parsing. No LLM: apps and tools turn natural language into these values, then call [`scatter`](SCATTER.md) or the ergonomic facade [`schedule`](SCHEDULE.md) (same semantics, one namespace).

## Functions (DAL)

| Call | Returns |
|------|---------|
| `time::unix_ms_now()` | Current time as Unix ms (`Int`). |
| `time::delay_ms_until_unix_ms(target_unix_ms)` | Non-negative ms from now until `target` (0 if past). |
| `time::parse_rfc3339_unix_ms(s)` | Parse RFC3339 instant → Unix ms (e.g. `2026-03-26T15:30:00Z`). |
| `time::parse_rfc3339_or_naive_utc_unix_ms(s)` | RFC3339, or naive UTC `YYYY-MM-DDTHH:MM:SS` / `YYYY-MM-DD HH:MM:SS`. |

## Relative vs absolute scheduling

**Relative** (from now):

```dal
scatter::after_ms(300000, "reminder");  // 5 minutes
```

**Absolute** (wall clock as Unix ms):

```dal
let t = time::parse_rfc3339_unix_ms("2026-12-31T23:59:59Z");
scatter::after_at_unix_ms(t, "new_year");
```

Or compose:

```dal
let t = time::parse_rfc3339_or_naive_utc_unix_ms("2026-06-01T09:00:00");
let d = time::delay_ms_until_unix_ms(t);
scatter::after_ms(d, "morning");
```

## App-level responsibility

Interpreting user chat (“remind me Tuesday”) → structured **RFC3339** or **Unix ms** (via a tool or post-processing) is your **agent or application** layer; `time::` + `scatter::` only execute **typed** inputs.

For **developers** trying scheduling see [`SCATTER.md`](SCATTER.md) and `examples/scatter_time_scheduling.dal` (`dal run` vs `dal serve` + poll `scatter::pending()`).
