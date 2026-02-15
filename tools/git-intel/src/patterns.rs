use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::common;
use crate::signals::{Signal, SignalKind};

#[derive(Serialize)]
pub struct PatternsOutput {
    pub fix_after_feat: Vec<FixAfterFeat>,
    pub multi_edit_chains: Vec<MultiEditChain>,
    pub directory_chains: Vec<DirectoryChain>,
    pub temporal_clusters: Vec<TemporalCluster>,
    pub total_commits_analyzed: usize,
    pub signals: Vec<Signal>,
}

#[derive(Serialize)]
pub struct FixAfterFeat {
    pub feat_commit: String,
    pub feat_date: String,
    pub feat_message: String,
    pub fix_commit: String,
    pub fix_date: String,
    pub fix_message: String,
    pub gap_commits: usize,
    pub shared_files: Vec<String>,
}

#[derive(Serialize)]
pub struct MultiEditChain {
    pub path: String,
    pub edit_count: usize,
    pub total_churn: usize,
    pub type_distribution: HashMap<String, usize>,
    pub commits: Vec<ChainCommit>,
}

#[derive(Serialize, Clone)]
pub struct ChainCommit {
    pub commit: String,
    pub date: String,
    pub message: String,
    pub commit_type: String,
}

#[derive(Serialize)]
pub struct DirectoryChain {
    pub path: String,
    pub total_edit_count: usize,
    pub total_churn: usize,
    pub files: Vec<String>,
}

#[derive(Serialize)]
pub struct TemporalCluster {
    pub cluster_type: String,
    pub start_time: String,
    pub end_time: String,
    pub commit_count: usize,
    pub commits: Vec<ChainCommit>,
    pub affected_files: Vec<String>,
}

struct CommitInfo {
    oid: String,
    date: String,
    message: String,
    commit_type: String,
    timestamp: i64,
    files_touched: Vec<String>,
    file_churn: HashMap<String, usize>,
}

pub fn run(repo: &Repository, since: Option<i64>, until: Option<i64>, limit: Option<usize>) -> Result<PatternsOutput> {
    run_impl(repo, since, until, limit, &mut |msg, parents| {
        common::classify_commit_with_parents(msg, parents).to_string()
    })
}

/// Run patterns with an optional ML classifier for enriched commit classification.
#[cfg(feature = "ml")]
pub fn run_with_ml(
    repo: &Repository,
    since: Option<i64>,
    until: Option<i64>,
    limit: Option<usize>,
    mut ml: Option<&mut crate::ml::MlClassifier>,
) -> Result<PatternsOutput> {
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
) -> Result<PatternsOutput> {
    let commits_iter = common::walk_commits(repo, since, until)?;

    let mut commits_info: Vec<CommitInfo> = Vec::new();

    for result in commits_iter {
        let commit = result?;
        let message = commit.message().unwrap_or("").to_string();
        let first_line = message.lines().next().unwrap_or("").to_string();
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

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut files = Vec::new();
        let mut file_churn: HashMap<String, usize> = HashMap::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(p) = delta.new_file().path().and_then(|p| p.to_str()) {
                    files.push(p.to_string());
                }
                true
            },
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(p) = delta.new_file().path().and_then(|p| p.to_str()) {
                    match line.origin() {
                        '+' | '-' => {
                            *file_churn.entry(p.to_string()).or_insert(0) += 1;
                        }
                        _ => {}
                    }
                }
                true
            }),
        )?;

        commits_info.push(CommitInfo {
            oid: common::short_hash(&commit.id()),
            date: dt.format("%Y-%m-%d").to_string(),
            message: first_line,
            commit_type: classify(&message, commit.parent_count()),
            timestamp: time,
            files_touched: files,
            file_churn,
        });
    }

    let total_commits_analyzed = commits_info.len();

    // 1. Fix-after-feat: find fix commits that follow feat commits within 5 commits
    //    Requires file overlap: at least one file must be touched by both the feat and fix.
    let mut fix_after_feat = Vec::new();
    let mut signals = Vec::new();
    for (i, ci) in commits_info.iter().enumerate() {
        if ci.commit_type == "fix" {
            let fix_files: std::collections::HashSet<&String> = ci.files_touched.iter().collect();
            for gap in 1..=5 {
                if i + gap >= commits_info.len() {
                    break;
                }
                let older = &commits_info[i + gap];
                if older.commit_type == "feat" || older.commit_type == "refactor" {
                    let shared: Vec<String> = older
                        .files_touched
                        .iter()
                        .filter(|f| fix_files.contains(f))
                        .cloned()
                        .collect();
                    if !shared.is_empty() {
                        // Backward compatibility: only add to fix_after_feat for "feat" type
                        if older.commit_type == "feat" {
                            fix_after_feat.push(FixAfterFeat {
                                feat_commit: older.oid.clone(),
                                feat_date: older.date.clone(),
                                feat_message: older.message.clone(),
                                fix_commit: ci.oid.clone(),
                                fix_date: ci.date.clone(),
                                fix_message: ci.message.clone(),
                                gap_commits: gap - 1,
                                shared_files: shared.clone(),
                            });
                        }

                        // Generate signal for both feat and refactor
                        let kind = if older.commit_type == "feat" {
                            SignalKind::FixAfterFeat
                        } else {
                            SignalKind::FixAfterRefactor
                        };

                        let severity = 1.0 / (gap as f64 + 1.0) * (shared.len().min(5) as f64 / 5.0);

                        signals.push(Signal {
                            kind,
                            severity,
                            message: format!(
                                "Fix {} likely caused by {} {} ({} shared files, {} commits apart)",
                                ci.oid, older.commit_type, older.oid, shared.len(), gap - 1
                            ),
                            commits: vec![older.oid.clone(), ci.oid.clone()],
                            files: shared,
                        });
                        break;
                    }
                }
            }
        }
    }

    if let Some(limit) = limit {
        fix_after_feat.truncate(limit);
    }

    // 2. Multi-edit chains: files touched 3+ times with >100 total lines changed
    let mut file_history: HashMap<String, Vec<ChainCommit>> = HashMap::new();
    let mut file_total_churn: HashMap<String, usize> = HashMap::new();
    let mut file_type_dist: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for ci in &commits_info {
        for f in &ci.files_touched {
            file_history
                .entry(f.clone())
                .or_default()
                .push(ChainCommit {
                    commit: ci.oid.clone(),
                    date: ci.date.clone(),
                    message: ci.message.clone(),
                    commit_type: ci.commit_type.clone(),
                });
            *file_total_churn.entry(f.clone()).or_insert(0) +=
                ci.file_churn.get(f).copied().unwrap_or(0);
            *file_type_dist
                .entry(f.clone())
                .or_default()
                .entry(ci.commit_type.clone())
                .or_insert(0) += 1;
        }
    }

    let mut multi_edit_chains: Vec<MultiEditChain> = file_history
        .into_iter()
        .filter(|(path, edits)| {
            edits.len() >= 3
                && file_total_churn.get(path).copied().unwrap_or(0) > 100
        })
        .map(|(path, commits)| {
            let total_churn = file_total_churn.get(&path).copied().unwrap_or(0);
            let type_distribution = file_type_dist.remove(&path).unwrap_or_default();
            MultiEditChain {
                edit_count: commits.len(),
                total_churn,
                type_distribution,
                commits,
                path,
            }
        })
        .collect();
    multi_edit_chains.sort_by(|a, b| b.edit_count.cmp(&a.edit_count));

    // Cap at top 10 unless --limit is specified and smaller
    let chain_cap = match limit {
        Some(l) if l < 10 => l,
        _ => 10,
    };
    multi_edit_chains.truncate(chain_cap);

    // 2b. Directory chains: aggregate file edit counts by parent directory (depth 1)
    let mut dir_edit_count: HashMap<String, usize> = HashMap::new();
    let mut dir_churn: HashMap<String, usize> = HashMap::new();
    let mut dir_files: HashMap<String, Vec<String>> = HashMap::new();
    for ci in &commits_info {
        // Track unique (dir, file) pairs per commit to count edits per directory correctly:
        // each commit touching a file in that directory counts as one edit for the directory.
        let mut seen_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
        for f in &ci.files_touched {
            let dir = common::dir_prefix(f, 1);
            if seen_dirs.insert(dir.clone()) {
                *dir_edit_count.entry(dir.clone()).or_insert(0) += 1;
            }
            *dir_churn.entry(dir.clone()).or_insert(0) +=
                ci.file_churn.get(f).copied().unwrap_or(0);
            let files = dir_files.entry(dir).or_default();
            if !files.contains(f) {
                files.push(f.clone());
            }
        }
    }

    let mut directory_chains: Vec<DirectoryChain> = dir_edit_count
        .into_iter()
        .filter(|(_dir, count)| *count >= 3)
        .map(|(dir, count)| {
            let total_churn = dir_churn.get(&dir).copied().unwrap_or(0);
            let mut files = dir_files.remove(&dir).unwrap_or_default();
            files.sort();
            DirectoryChain {
                path: dir,
                total_edit_count: count,
                total_churn,
                files,
            }
        })
        .collect();
    directory_chains.sort_by(|a, b| b.total_churn.cmp(&a.total_churn));

    let dir_chain_cap = match limit {
        Some(l) if l < 10 => l,
        _ => 10,
    };
    directory_chains.truncate(dir_chain_cap);

    // 3. Temporal clusters: 3+ commits of the same type within a 1-hour window
    let mut temporal_clusters = Vec::new();
    {
        // Group commits by type
        let mut by_type: HashMap<String, Vec<&CommitInfo>> = HashMap::new();
        for ci in &commits_info {
            by_type.entry(ci.commit_type.clone()).or_default().push(ci);
        }

        for (ctype, mut type_commits) in by_type {
            // Sort by timestamp ascending for clustering
            type_commits.sort_by_key(|c| c.timestamp);

            let mut i = 0;
            while i < type_commits.len() {
                let start_ts = type_commits[i].timestamp;
                let mut j = i + 1;
                while j < type_commits.len()
                    && type_commits[j].timestamp - start_ts <= 3600
                {
                    j += 1;
                }
                let cluster_size = j - i;
                if cluster_size >= 3 {
                    let cluster_commits: Vec<&CommitInfo> =
                        type_commits[i..j].to_vec();
                    let start_dt =
                        chrono::DateTime::from_timestamp(cluster_commits[0].timestamp, 0)
                            .unwrap_or_default();
                    let end_dt = chrono::DateTime::from_timestamp(
                        cluster_commits.last().unwrap().timestamp,
                        0,
                    )
                    .unwrap_or_default();

                    let mut affected: Vec<String> = cluster_commits
                        .iter()
                        .flat_map(|c| c.files_touched.iter().cloned())
                        .collect();
                    affected.sort();
                    affected.dedup();

                    let commits_vec: Vec<ChainCommit> = cluster_commits
                        .iter()
                        .map(|c| ChainCommit {
                            commit: c.oid.clone(),
                            date: c.date.clone(),
                            message: c.message.clone(),
                            commit_type: c.commit_type.clone(),
                        })
                        .collect();

                    temporal_clusters.push(TemporalCluster {
                        cluster_type: ctype.clone(),
                        start_time: start_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        end_time: end_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        commit_count: cluster_size,
                        commits: commits_vec,
                        affected_files: affected,
                    });
                }
                // Move past this cluster (don't re-use commits)
                i = j;
            }
        }
    }
    temporal_clusters.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    if let Some(limit) = limit {
        temporal_clusters.truncate(limit);
    }

    Ok(PatternsOutput {
        fix_after_feat,
        multi_edit_chains,
        directory_chains,
        temporal_clusters,
        total_commits_analyzed,
        signals,
    })
}
