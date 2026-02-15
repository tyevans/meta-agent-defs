use anyhow::Result;
use chrono::Utc;
use git2::Repository;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::{churn, metrics};

#[derive(Serialize)]
pub struct TrendsOutput {
    pub windows: Vec<WindowData>,
    pub window_count: usize,
    pub window_size_days: u32,
    pub deltas: Deltas,
    pub dormant_files: Vec<String>,
}

#[derive(Serialize)]
pub struct WindowData {
    pub index: usize,
    pub label: String,
    pub since: String,
    pub until: String,
    pub total_commits: usize,
    pub type_distribution: HashMap<String, usize>,
    pub velocity: f64,
    pub top_churn_files: Vec<String>,
}

#[derive(Serialize)]
pub struct Deltas {
    pub commit_trend: String,
    pub fix_rate_trend: String,
}

/// Compute the trend label by comparing two values.
/// "stable" if within 10%, otherwise "increasing" or "decreasing".
fn trend_label(latest: f64, previous: f64) -> &'static str {
    if previous == 0.0 && latest == 0.0 {
        return "stable";
    }
    if previous == 0.0 {
        return "increasing";
    }
    let ratio = (latest - previous) / previous;
    if ratio > 0.1 {
        "increasing"
    } else if ratio < -0.1 {
        "decreasing"
    } else {
        "stable"
    }
}

pub fn run(
    repo: &Repository,
    window_count: usize,
    window_size_days: u32,
    churn_limit: usize,
) -> Result<TrendsOutput> {
    let now = Utc::now().timestamp();
    let window_secs = window_size_days as i64 * 86400;

    let mut windows = Vec::with_capacity(window_count);
    let mut file_sets: Vec<HashSet<String>> = Vec::with_capacity(window_count);

    for i in 0..window_count {
        let until_epoch = now - (i as i64 * window_secs);
        let since_epoch = until_epoch - window_secs;

        let since_date = chrono::DateTime::from_timestamp(since_epoch, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d")
            .to_string();
        let until_date = chrono::DateTime::from_timestamp(until_epoch, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d")
            .to_string();

        let label = format!("{} to {}", since_date, until_date);

        // Get metrics for this window
        let metrics_result = metrics::run(repo, Some(since_epoch), Some(until_epoch), None)?;

        // Build type distribution from commit_types
        let mut type_distribution = HashMap::new();
        for ct in &metrics_result.commit_types {
            type_distribution.insert(ct.type_name.clone(), ct.count);
        }

        let total_commits = metrics_result.total_commits;
        let velocity = total_commits as f64 / window_size_days as f64;

        // Get churn files for this window (no limit, so we get the full file set)
        let churn_result = churn::run(repo, Some(since_epoch), Some(until_epoch), None)?;
        let all_files: Vec<String> = churn_result.files.into_iter().map(|f| f.path).collect();
        let top_churn_files: Vec<String> = all_files.iter().take(churn_limit).cloned().collect();
        let active_file_set: HashSet<String> = all_files.into_iter().collect();

        windows.push(WindowData {
            index: i,
            label,
            since: since_date,
            until: until_date,
            total_commits,
            type_distribution,
            velocity,
            top_churn_files,
        });

        file_sets.push(active_file_set);
    }

    // Compute deltas comparing window[0] (latest) vs window[1] (previous)
    let deltas = if windows.len() >= 2 {
        let latest = &windows[0];
        let previous = &windows[1];

        let commit_trend = trend_label(latest.total_commits as f64, previous.total_commits as f64).to_string();

        let latest_fixes = *latest.type_distribution.get("fix").unwrap_or(&0) as f64;
        let latest_total = latest.total_commits as f64;
        let latest_fix_rate = if latest_total > 0.0 { latest_fixes / latest_total } else { 0.0 };

        let prev_fixes = *previous.type_distribution.get("fix").unwrap_or(&0) as f64;
        let prev_total = previous.total_commits as f64;
        let prev_fix_rate = if prev_total > 0.0 { prev_fixes / prev_total } else { 0.0 };

        let fix_rate_trend = trend_label(latest_fix_rate, prev_fix_rate).to_string();

        Deltas {
            commit_trend,
            fix_rate_trend,
        }
    } else {
        Deltas {
            commit_trend: "stable".to_string(),
            fix_rate_trend: "stable".to_string(),
        }
    };

    // Compute dormant files: active in any older window but not in the newest (index 0)
    let dormant_files = compute_dormant(&file_sets);

    Ok(TrendsOutput {
        windows,
        window_count,
        window_size_days,
        deltas,
        dormant_files,
    })
}

/// Given file sets per window (index 0 = newest), compute dormant files.
/// Exposed for unit testing.
fn compute_dormant(file_sets: &[HashSet<String>]) -> Vec<String> {
    if file_sets.len() < 2 {
        return Vec::new();
    }
    let newest = &file_sets[0];
    let older_union: HashSet<String> = file_sets[1..]
        .iter()
        .flat_map(|s| s.iter().cloned())
        .collect();
    let mut dormant: Vec<String> = older_union
        .into_iter()
        .filter(|f| !newest.contains(f))
        .collect();
    dormant.sort();
    dormant
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trend_label_stable_both_zero() {
        assert_eq!(trend_label(0.0, 0.0), "stable");
    }

    #[test]
    fn trend_label_increasing_from_zero() {
        assert_eq!(trend_label(5.0, 0.0), "increasing");
    }

    #[test]
    fn trend_label_increasing() {
        assert_eq!(trend_label(12.0, 10.0), "increasing");
    }

    #[test]
    fn trend_label_decreasing() {
        assert_eq!(trend_label(8.0, 10.0), "decreasing");
    }

    #[test]
    fn trend_label_stable_within_threshold() {
        // 10% of 10 is 1, so 10.5 is within threshold
        assert_eq!(trend_label(10.5, 10.0), "stable");
    }

    #[test]
    fn dormant_single_window_returns_empty() {
        let sets = vec![HashSet::from(["a.rs".to_string()])];
        assert_eq!(compute_dormant(&sets), Vec::<String>::new());
    }

    #[test]
    fn dormant_no_windows_returns_empty() {
        let sets: Vec<HashSet<String>> = vec![];
        assert_eq!(compute_dormant(&sets), Vec::<String>::new());
    }

    #[test]
    fn dormant_file_in_older_not_newest() {
        let newest = HashSet::from(["a.rs".to_string()]);
        let older = HashSet::from(["a.rs".to_string(), "b.rs".to_string()]);
        let sets = vec![newest, older];
        assert_eq!(compute_dormant(&sets), vec!["b.rs".to_string()]);
    }

    #[test]
    fn dormant_file_active_in_both_not_dormant() {
        let newest = HashSet::from(["a.rs".to_string(), "b.rs".to_string()]);
        let older = HashSet::from(["a.rs".to_string(), "b.rs".to_string()]);
        let sets = vec![newest, older];
        assert_eq!(compute_dormant(&sets), Vec::<String>::new());
    }

    #[test]
    fn dormant_union_across_multiple_older_windows() {
        let newest = HashSet::from(["a.rs".to_string()]);
        let older1 = HashSet::from(["b.rs".to_string()]);
        let older2 = HashSet::from(["c.rs".to_string()]);
        let sets = vec![newest, older1, older2];
        assert_eq!(compute_dormant(&sets), vec!["b.rs".to_string(), "c.rs".to_string()]);
    }

    #[test]
    fn dormant_output_sorted_alphabetically() {
        let newest = HashSet::new();
        let older = HashSet::from([
            "z.rs".to_string(),
            "a.rs".to_string(),
            "m.rs".to_string(),
        ]);
        let sets = vec![newest, older];
        assert_eq!(
            compute_dormant(&sets),
            vec!["a.rs".to_string(), "m.rs".to_string(), "z.rs".to_string()]
        );
    }

    #[test]
    fn dormant_new_file_only_in_newest_not_flagged() {
        let newest = HashSet::from(["new.rs".to_string(), "a.rs".to_string()]);
        let older = HashSet::from(["a.rs".to_string()]);
        let sets = vec![newest, older];
        assert_eq!(compute_dormant(&sets), Vec::<String>::new());
    }
}
