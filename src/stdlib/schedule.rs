//! **`schedule::`** — ergonomic scheduling on top of [`crate::stdlib::time`] and [`crate::stdlib::scatter`].
//!
//! Use this module when you want **one namespace** for “when does work fire?” instead of splitting
//! between `time::` (parse / compare) and `scatter::` (arm timers).
//!
//! **Semantics:** identical to Scatter — **process-local** only. Jobs exist while the interpreter
//! process runs and something drains [`pending`]. For **durable** tasks across restarts, use an
//! application store (e.g. CEO `POST /api/schedule`).

use crate::runtime::values::Value;

const MS_PER_SEC: i64 = 1_000;
const MS_PER_MIN: i64 = 60_000;
const MS_PER_HOUR: i64 = 3_600_000;

/// Max one-shots in [`series_interval_unix_ms`] / [`series_interval_from_now_ms`] (safety cap).
pub const MAX_SERIES_COUNT: i64 = 10_000;

/// One-shot after `delay_ms` from now (delegates to [`scatter::after_ms`]).
pub fn once_after_ms(id: &str, delay_ms: i64) -> Result<(), String> {
    crate::stdlib::scatter::after_ms(delay_ms, id)
}

/// One-shot at wall-clock Unix ms (delegates to [`scatter::after_at_unix_ms`]).
pub fn once_at_unix_ms(id: &str, unix_ms: i64) -> Result<(), String> {
    crate::stdlib::scatter::after_at_unix_ms(unix_ms, id)
}

/// Parse RFC3339 / naive UTC (see [`time::parse_rfc3339_or_naive_utc_unix_ms`]) then schedule once.
pub fn once_at_rfc3339(id: &str, rfc3339: &str) -> Result<(), String> {
    let ms = crate::stdlib::time::parse_rfc3339_or_naive_utc_unix_ms(rfc3339)?;
    once_at_unix_ms(id, ms)
}

/// Repeating every `seconds` (> 0).
pub fn every_seconds(id: &str, seconds: i64) -> Result<(), String> {
    if seconds <= 0 {
        return Err("schedule::every_seconds: interval must be > 0".into());
    }
    let ms = seconds
        .checked_mul(MS_PER_SEC)
        .ok_or_else(|| "schedule::every_seconds: overflow".to_string())?;
    crate::stdlib::scatter::every_ms(ms, id)
}

/// Repeating every `minutes` (> 0).
pub fn every_minutes(id: &str, minutes: i64) -> Result<(), String> {
    if minutes <= 0 {
        return Err("schedule::every_minutes: interval must be > 0".into());
    }
    let ms = minutes
        .checked_mul(MS_PER_MIN)
        .ok_or_else(|| "schedule::every_minutes: overflow".to_string())?;
    crate::stdlib::scatter::every_ms(ms, id)
}

/// Repeating every `hours` (> 0).
pub fn every_hours(id: &str, hours: i64) -> Result<(), String> {
    if hours <= 0 {
        return Err("schedule::every_hours: interval must be > 0".into());
    }
    let ms = hours
        .checked_mul(MS_PER_HOUR)
        .ok_or_else(|| "schedule::every_hours: overflow".to_string())?;
    crate::stdlib::scatter::every_ms(ms, id)
}

/// `count` one-shots at `start_unix_ms + i * interval_ms` for `i in 0..count`.
/// Job ids: `{prefix}_0` … `{prefix}_{count-1}`.
pub fn series_interval_unix_ms(
    prefix: &str,
    start_unix_ms: i64,
    interval_ms: i64,
    count: i64,
) -> Result<Vec<String>, String> {
    if prefix.is_empty() {
        return Err("schedule::series_interval_unix_ms: prefix must not be empty".into());
    }
    if count <= 0 {
        return Err("schedule::series_interval_unix_ms: count must be > 0".into());
    }
    if count > MAX_SERIES_COUNT {
        return Err(format!(
            "schedule::series_interval_unix_ms: count must be <= {}",
            MAX_SERIES_COUNT
        ));
    }
    if interval_ms < 0 {
        return Err("schedule::series_interval_unix_ms: interval_ms must be >= 0".into());
    }
    let mut ids = Vec::with_capacity(count as usize);
    for i in 0..count {
        let id = format!("{}_{}", prefix, i);
        let step = i
            .checked_mul(interval_ms)
            .ok_or_else(|| "schedule::series_interval_unix_ms: overflow".to_string())?;
        let t = start_unix_ms
            .checked_add(step)
            .ok_or_else(|| "schedule::series_interval_unix_ms: overflow".to_string())?;
        crate::stdlib::scatter::after_at_unix_ms(t, &id)?;
        ids.push(id);
    }
    Ok(ids)
}

/// Same as [`series_interval_unix_ms`] with `start_unix_ms = time::unix_ms_now()`.
pub fn series_interval_from_now_ms(
    prefix: &str,
    interval_ms: i64,
    count: i64,
) -> Result<Vec<String>, String> {
    series_interval_unix_ms(
        prefix,
        crate::stdlib::time::unix_ms_now(),
        interval_ms,
        count,
    )
}

pub fn cancel(id: &str) -> bool {
    crate::stdlib::scatter::cancel(id)
}

pub fn pending() -> Vec<String> {
    crate::stdlib::scatter::pending()
}

pub fn peek_pending() -> Vec<String> {
    crate::stdlib::scatter::peek_pending()
}

pub fn scheduled_count() -> usize {
    crate::stdlib::scatter::scheduled_count()
}

pub fn next_due_ms() -> Option<u64> {
    crate::stdlib::scatter::next_due_ms()
}

pub fn pending_value() -> Value {
    crate::stdlib::scatter::pending_value()
}

pub fn peek_pending_value() -> Value {
    crate::stdlib::scatter::peek_pending_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::scatter;

    #[test]
    fn series_ids_distinct() {
        let _lock = scatter::test_lock();
        scatter::reset_for_test();
        let ids = series_interval_unix_ms("p", 1_700_000_000_000, 3_600_000, 3).unwrap();
        assert_eq!(ids, vec!["p_0", "p_1", "p_2"]);
        scatter::reset_for_test();
    }

    #[test]
    fn series_rejects_bad_count() {
        let _lock = scatter::test_lock();
        scatter::reset_for_test();
        assert!(series_interval_unix_ms("p", 0, 1, 0).is_err());
        assert!(series_interval_unix_ms("", 0, 1, 1).is_err());
        scatter::reset_for_test();
    }

    #[test]
    fn every_minutes_delegates() {
        let _lock = scatter::test_lock();
        scatter::reset_for_test();
        every_minutes("m", 60).unwrap();
        assert!(scatter::scheduled_count() >= 1);
        scatter::reset_for_test();
    }
}
