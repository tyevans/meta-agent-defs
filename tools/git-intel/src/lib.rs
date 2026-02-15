// Each subcommand module exposes run(&Repository, ...) -> Result<T: Serialize>.
// This uniform interface is intentional; see DESIGN-daemon.md for the
// caching/daemon design that depends on it.
pub mod authors;
pub mod cache;
pub mod churn;
pub mod common;
pub mod hotspots;
pub mod lifecycle;
pub mod metrics;
#[cfg(feature = "ml")]
pub mod ml;
pub mod patterns;
pub mod signals;
pub mod trends;

use anyhow::Result;
use chrono::{Datelike, Months, NaiveDate, Utc};

/// Try to parse a relative date string (e.g. "30d", "4w", "6m", "1y").
/// Returns the resolved NaiveDate (UTC) or None if the string is not a relative format.
fn try_parse_relative(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.len() < 2 {
        return None;
    }
    let (num_part, unit) = s.split_at(s.len() - 1);
    let n: u32 = num_part.parse().ok()?;
    let today = Utc::now().date_naive();

    match unit {
        "d" => today.checked_sub_signed(chrono::Duration::days(n as i64)),
        "w" => today.checked_sub_signed(chrono::Duration::weeks(n as i64)),
        "m" => today.checked_sub_months(Months::new(n)),
        "y" => {
            // Subtract N years: handle leap year edge case (Feb 29 -> Feb 28)
            let target_year = today.year() - n as i32;
            NaiveDate::from_ymd_opt(target_year, today.month(), today.day())
                .or_else(|| NaiveDate::from_ymd_opt(target_year, today.month(), today.day() - 1))
        }
        _ => None,
    }
}

/// Parse an optional date string into a Unix timestamp at 00:00:00 UTC.
/// Accepts YYYY-MM-DD (ISO) or relative formats: Nd, Nw, Nm, Ny.
fn parse_date_start_of_day(value: &Option<String>, flag_name: &str) -> Result<Option<i64>> {
    match value {
        Some(s) => {
            let naive = if let Some(date) = try_parse_relative(s) {
                date
            } else {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| anyhow::anyhow!("Invalid --{} date '{}': {} (expected YYYY-MM-DD or relative like 30d, 4w, 6m, 1y)", flag_name, s, e))?
            };
            let dt = naive
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            Ok(Some(dt.and_utc().timestamp()))
        }
        None => Ok(None),
    }
}

/// Parse an optional date string into a Unix timestamp at 23:59:59 UTC (end of day).
/// Accepts YYYY-MM-DD (ISO) or relative formats: Nd, Nw, Nm, Ny.
fn parse_date_end_of_day(value: &Option<String>, flag_name: &str) -> Result<Option<i64>> {
    match value {
        Some(s) => {
            let naive = if let Some(date) = try_parse_relative(s) {
                date
            } else {
                NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| anyhow::anyhow!("Invalid --{} date '{}': {} (expected YYYY-MM-DD or relative like 30d, 4w, 6m, 1y)", flag_name, s, e))?
            };
            let dt = naive
                .and_hms_opt(23, 59, 59)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            Ok(Some(dt.and_utc().timestamp()))
        }
        None => Ok(None),
    }
}

pub fn parse_since(since: &Option<String>) -> Result<Option<i64>> {
    parse_date_start_of_day(since, "since")
}

pub fn parse_until(until: &Option<String>) -> Result<Option<i64>> {
    parse_date_end_of_day(until, "until")
}

/// Validate that --since is not after --until. Returns an error if the range is inverted.
pub fn validate_range(since: Option<i64>, until: Option<i64>) -> Result<()> {
    if let (Some(s), Some(u)) = (since, until) {
        if s > u {
            anyhow::bail!("--since date is after --until date (empty range)");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- ISO date tests (existing behavior preserved) ----

    #[test]
    fn parse_since_valid_date() {
        let result = parse_since(&Some("2026-01-15".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        // 2026-01-15 00:00:00 UTC
        assert_eq!(ts, 1768435200);
    }

    #[test]
    fn parse_since_none() {
        let result = parse_since(&None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_since_invalid_date() {
        let result = parse_since(&Some("not-a-date".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn parse_since_edge_date() {
        let result = parse_since(&Some("2000-01-01".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        // 2000-01-01 00:00:00 UTC = 946684800
        assert_eq!(ts, 946684800);
    }

    #[test]
    fn parse_until_valid_date() {
        let result = parse_until(&Some("2026-01-15".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        // 2026-01-15 23:59:59 UTC = 1768435200 + 86399
        assert_eq!(ts, 1768435200 + 86399);
    }

    #[test]
    fn parse_until_none() {
        let result = parse_until(&None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_until_invalid_date() {
        let result = parse_until(&Some("garbage".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn validate_range_valid() {
        assert!(validate_range(Some(100), Some(200)).is_ok());
    }

    #[test]
    fn validate_range_both_none() {
        assert!(validate_range(None, None).is_ok());
    }

    #[test]
    fn validate_range_one_none() {
        assert!(validate_range(Some(100), None).is_ok());
        assert!(validate_range(None, Some(200)).is_ok());
    }

    #[test]
    fn validate_range_inverted() {
        let result = validate_range(Some(200), Some(100));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("after"));
    }

    #[test]
    fn validate_range_equal() {
        // Same timestamp is valid (single-second window)
        assert!(validate_range(Some(100), Some(100)).is_ok());
    }

    // ---- relative date parsing tests ----

    #[test]
    fn try_parse_relative_days() {
        let date = try_parse_relative("30d").unwrap();
        let expected = Utc::now().date_naive() - chrono::Duration::days(30);
        assert_eq!(date, expected);
    }

    #[test]
    fn try_parse_relative_weeks() {
        let date = try_parse_relative("4w").unwrap();
        let expected = Utc::now().date_naive() - chrono::Duration::weeks(4);
        assert_eq!(date, expected);
    }

    #[test]
    fn try_parse_relative_months() {
        let date = try_parse_relative("6m").unwrap();
        let today = Utc::now().date_naive();
        let expected = today.checked_sub_months(Months::new(6)).unwrap();
        assert_eq!(date, expected);
    }

    #[test]
    fn try_parse_relative_years() {
        let date = try_parse_relative("1y").unwrap();
        let today = Utc::now().date_naive();
        let expected_year = today.year() - 1;
        assert_eq!(date.year(), expected_year);
        // month/day should match (unless leap year edge case)
        assert_eq!(date.month(), today.month());
    }

    #[test]
    fn try_parse_relative_zero_days() {
        // 0d = today
        let date = try_parse_relative("0d").unwrap();
        assert_eq!(date, Utc::now().date_naive());
    }

    #[test]
    fn try_parse_relative_invalid_unit() {
        assert!(try_parse_relative("30x").is_none());
    }

    #[test]
    fn try_parse_relative_no_number() {
        assert!(try_parse_relative("d").is_none());
    }

    #[test]
    fn try_parse_relative_empty() {
        assert!(try_parse_relative("").is_none());
    }

    #[test]
    fn try_parse_relative_iso_not_matched() {
        // ISO dates should not match relative format
        assert!(try_parse_relative("2026-01-15").is_none());
    }

    #[test]
    fn parse_since_relative_30d() {
        let result = parse_since(&Some("30d".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        let expected_date = Utc::now().date_naive() - chrono::Duration::days(30);
        let expected_ts = expected_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        assert_eq!(ts, expected_ts);
    }

    #[test]
    fn parse_since_relative_1y() {
        let result = parse_since(&Some("1y".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        // Should be roughly 365 days ago (start of day)
        let now = Utc::now().timestamp();
        let roughly_one_year = 365 * 86400;
        assert!((now - ts - roughly_one_year).unsigned_abs() < 2 * 86400);
    }

    #[test]
    fn parse_until_relative_3m() {
        let result = parse_until(&Some("3m".to_string())).unwrap();
        assert!(result.is_some());
        let ts = result.unwrap();
        let expected_date = Utc::now().date_naive().checked_sub_months(Months::new(3)).unwrap();
        let expected_ts = expected_date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();
        assert_eq!(ts, expected_ts);
    }

    #[test]
    fn parse_since_until_relative_range() {
        // --since 6m --until 3m should produce a valid range
        let since = parse_since(&Some("6m".to_string())).unwrap().unwrap();
        let until = parse_until(&Some("3m".to_string())).unwrap().unwrap();
        assert!(since < until, "6m ago should be before 3m ago");
        assert!(validate_range(Some(since), Some(until)).is_ok());
    }

    #[test]
    fn parse_relative_resolves_to_absolute_epoch() {
        // Two calls to parse_since("30d") in the same second should return the same value.
        // This confirms it resolves to a date (not a floating offset).
        let ts1 = parse_since(&Some("30d".to_string())).unwrap().unwrap();
        let ts2 = parse_since(&Some("30d".to_string())).unwrap().unwrap();
        assert_eq!(ts1, ts2);
    }

    #[test]
    fn parse_since_invalid_still_errors() {
        // A string that is neither ISO nor relative should error
        let result = parse_since(&Some("yesterday".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn parse_since_relative_large_number() {
        // 365d should work fine
        let result = parse_since(&Some("365d".to_string())).unwrap();
        assert!(result.is_some());
    }
}
