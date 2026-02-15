// Each subcommand module exposes run(&Repository, ...) -> Result<T: Serialize>.
// This uniform interface is intentional; see DESIGN-daemon.md for the
// caching/daemon design that depends on it.
pub mod cache;
pub mod churn;
pub mod common;
pub mod hotspots;
pub mod lifecycle;
pub mod metrics;
#[cfg(feature = "ml")]
pub mod ml;
pub mod patterns;

use anyhow::Result;

/// Parse an optional YYYY-MM-DD date string into a Unix timestamp at 00:00:00 UTC.
/// Used for both --since and --until flags.
fn parse_date(value: &Option<String>, flag_name: &str) -> Result<Option<i64>> {
    match value {
        Some(s) => {
            let naive = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid --{} date '{}': {}", flag_name, s, e))?;
            let dt = naive
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            Ok(Some(dt.and_utc().timestamp()))
        }
        None => Ok(None),
    }
}

pub fn parse_since(since: &Option<String>) -> Result<Option<i64>> {
    parse_date(since, "since")
}

pub fn parse_until(until: &Option<String>) -> Result<Option<i64>> {
    // For --until, we want end-of-day (23:59:59) so the entire day is included
    match until {
        Some(s) => {
            let naive = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid --until date '{}': {}", s, e))?;
            let dt = naive
                .and_hms_opt(23, 59, 59)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            Ok(Some(dt.and_utc().timestamp()))
        }
        None => Ok(None),
    }
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
}
