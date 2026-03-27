//! Wall-clock and duration helpers for DAL (no LLM). Use with [`crate::stdlib::scatter`]
//! for **relative** (`scatter::after_ms`) vs **absolute** (`scatter::after_at_unix_ms`) scheduling.
//!
//! All Unix times are **milliseconds** since `1970-01-01T00:00:00Z` as `i64` (DAL `Int`).
//! RFC3339 strings include timezone offset or `Z`.

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

/// Current time as Unix milliseconds.
pub fn unix_ms_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Parse an RFC3339 / ISO8601 instant (e.g. `2026-03-26T15:30:00Z`, `2026-03-26T10:30:00-05:00`) to Unix ms.
pub fn parse_rfc3339_unix_ms(s: &str) -> Result<i64, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("parse_rfc3339_unix_ms: empty string".into());
    }
    let dt =
        DateTime::parse_from_rfc3339(t).map_err(|e| format!("parse_rfc3339_unix_ms: {}", e))?;
    Ok(dt.timestamp_millis())
}

/// Parse RFC3339, or a **naive** UTC datetime (e.g. `2026-03-26T15:30:00`).
pub fn parse_rfc3339_or_naive_utc_unix_ms(s: &str) -> Result<i64, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("parse_rfc3339_or_naive_utc_unix_ms: empty string".into());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(t) {
        return Ok(dt.timestamp_millis());
    }
    let naive = NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M:%S"))
        .map_err(|e| format!("parse_rfc3339_or_naive_utc_unix_ms: {}", e))?;
    Ok(Utc.from_utc_datetime(&naive).timestamp_millis())
}

/// Non-negative milliseconds from **now** until `target_unix_ms` (0 if already past).
pub fn delay_ms_until_unix_ms(target_unix_ms: i64) -> i64 {
    let now = unix_ms_now();
    (target_unix_ms - now).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_ms_now_sane() {
        let n = unix_ms_now();
        assert!(n > 1_700_000_000_000); // ~2023+
    }

    #[test]
    fn parse_rfc3339_z() {
        let ms = parse_rfc3339_unix_ms("1970-01-01T00:00:00.000Z").unwrap();
        assert_eq!(ms, 0);
    }

    #[test]
    fn delay_non_negative() {
        assert_eq!(delay_ms_until_unix_ms(unix_ms_now() - 1000), 0);
    }
}
