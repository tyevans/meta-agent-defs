// Each subcommand module exposes run(&Repository, ...) -> Result<T: Serialize>.
// This uniform interface is intentional; see DESIGN-daemon.md for the
// caching/daemon design that depends on it.
pub mod churn;
pub mod common;
pub mod lifecycle;
pub mod metrics;
pub mod patterns;

use anyhow::Result;

pub fn parse_since(since: &Option<String>) -> Result<Option<i64>> {
    match since {
        Some(s) => {
            let naive = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid --since date '{}': {}", s, e))?;
            let dt = naive
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
            Ok(Some(dt.and_utc().timestamp()))
        }
        None => Ok(None),
    }
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
}
