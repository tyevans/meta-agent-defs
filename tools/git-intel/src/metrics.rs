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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ticket_refs: Vec<TicketRef>,
}

#[derive(Serialize)]
pub struct TicketRef {
    pub ticket: String,
    pub count: usize,
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

pub fn run(repo: &Repository, since: Option<i64>, until: Option<i64>, limit: Option<usize>) -> Result<MetricsOutput> {
    run_impl(repo, since, until, limit, &mut |msg, parents| {
        common::classify_commit_with_parents(msg, parents).to_string()
    })
}

/// Run metrics with an optional ML classifier for enriched commit classification.
#[cfg(feature = "ml")]
pub fn run_with_ml(
    repo: &Repository,
    since: Option<i64>,
    until: Option<i64>,
    limit: Option<usize>,
    mut ml: Option<&mut crate::ml::MlClassifier>,
) -> Result<MetricsOutput> {
    run_impl(repo, since, until, limit, &mut |msg, parents| {
        common::classify_commit_with_ml(msg, parents, ml.as_mut().map(|m| &mut **m))
    })
}

fn run_impl(
    repo: &Repository,
    since: Option<i64>,
    until: Option<i64>,
    limit: Option<usize>,
    classify: &mut dyn FnMut(&str, usize) -> String,
) -> Result<MetricsOutput> {
    let commits = common::walk_commits(repo, since, until)?;

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut daily_counts: HashMap<String, usize> = HashMap::new();
    let mut ticket_counts: HashMap<String, usize> = HashMap::new();
    let mut line_counts: Vec<usize> = Vec::new();
    let mut total = 0usize;

    for result in commits {
        let commit = result?;
        total += 1;
        let message = commit.message().unwrap_or("");
        let ctype = classify(message, commit.parent_count());
        *type_counts.entry(ctype).or_insert(0) += 1;

        let time = commit.time().seconds();
        let dt = match chrono::DateTime::from_timestamp(time, 0) {
            Some(dt) => dt,
            None => {
                eprintln!(
                    "warning: commit {} has invalid timestamp {}, falling back to epoch 0",
                    commit.id(),
                    time
                );
                chrono::DateTime::default()
            }
        };
        let date_str = dt.format("%Y-%m-%d").to_string();
        *daily_counts.entry(date_str).or_insert(0) += 1;

        if let Some(ticket) = common::extract_ticket_ref(message) {
            *ticket_counts.entry(ticket).or_insert(0) += 1;
        }

        let lines = lines_changed(repo, &commit);
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

    if let Some(limit) = limit {
        commit_types.truncate(limit);
    }

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

    // Build ticket refs sorted by count descending
    let mut ticket_refs: Vec<TicketRef> = ticket_counts
        .into_iter()
        .map(|(ticket, count)| TicketRef { ticket, count })
        .collect();
    ticket_refs.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.ticket.cmp(&b.ticket)));

    if let Some(limit) = limit {
        ticket_refs.truncate(limit);
    }

    Ok(MetricsOutput {
        commit_types,
        activity,
        velocity,
        total_commits: total,
        ticket_refs,
    })
}
