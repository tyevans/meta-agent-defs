use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::common;

#[derive(Serialize)]
pub struct MetricsOutput {
    pub commit_types: Vec<CommitType>,
    pub activity: Vec<ActivityBurst>,
    pub velocity: VelocityStats,
    pub total_commits: usize,
}

#[derive(Serialize)]
pub struct CommitType {
    #[serde(rename = "type")]
    pub type_name: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Serialize)]
pub struct ActivityBurst {
    pub date: String,
    pub commits: usize,
}

#[derive(Serialize)]
pub struct VelocityStats {
    pub avg_lines_per_commit: f64,
    pub max_lines_in_commit: usize,
    pub min_lines_in_commit: usize,
    pub total_lines_changed: usize,
}

fn lines_changed(repo: &Repository, commit: &git2::Commit) -> usize {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return 0,
    };

    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
        Ok(d) => d,
        Err(_) => return 0,
    };

    let stats = match diff.stats() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    stats.insertions() + stats.deletions()
}

pub fn run(repo: &Repository, since: Option<i64>, limit: Option<usize>) -> Result<MetricsOutput> {
    let commits = common::walk_commits(repo, since)?;

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut daily_counts: HashMap<String, usize> = HashMap::new();
    let mut line_counts: Vec<usize> = Vec::new();
    let total = commits.len();

    for commit in &commits {
        let message = commit.message().unwrap_or("");
        let ctype = common::classify_commit(message);
        *type_counts.entry(ctype.to_string()).or_insert(0) += 1;

        let time = commit.time().seconds();
        let dt = chrono::DateTime::from_timestamp(time, 0).unwrap_or_default();
        let date_str = dt.format("%Y-%m-%d").to_string();
        *daily_counts.entry(date_str).or_insert(0) += 1;

        let lines = lines_changed(repo, commit);
        line_counts.push(lines);
    }

    // Build commit types sorted by count descending
    let mut commit_types: Vec<CommitType> = type_counts
        .into_iter()
        .map(|(type_name, count)| CommitType {
            type_name,
            count,
            percentage: if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        })
        .collect();
    commit_types.sort_by(|a, b| b.count.cmp(&a.count));

    // Build activity bursts sorted by date descending
    let mut activity: Vec<ActivityBurst> = daily_counts
        .into_iter()
        .map(|(date, commits)| ActivityBurst { date, commits })
        .collect();
    activity.sort_by(|a, b| b.date.cmp(&a.date));

    if let Some(limit) = limit {
        activity.truncate(limit);
    }

    // Velocity stats
    let total_lines: usize = line_counts.iter().sum();
    let velocity = VelocityStats {
        avg_lines_per_commit: if total > 0 {
            total_lines as f64 / total as f64
        } else {
            0.0
        },
        max_lines_in_commit: line_counts.iter().copied().max().unwrap_or(0),
        min_lines_in_commit: line_counts.iter().copied().min().unwrap_or(0),
        total_lines_changed: total_lines,
    };

    Ok(MetricsOutput {
        commit_types,
        activity,
        velocity,
        total_commits: total,
    })
}
