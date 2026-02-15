use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::common;

#[derive(Serialize)]
pub struct PatternsOutput {
    pub fix_after_feat: Vec<FixAfterFeat>,
    pub multi_edit_chains: Vec<MultiEditChain>,
    pub convergence: Vec<ConvergencePair>,
    pub convergence_truncated: bool,
    pub convergence_limit: usize,
    pub total_commits_analyzed: usize,
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
}

#[derive(Serialize)]
pub struct MultiEditChain {
    pub path: String,
    pub edit_count: usize,
    pub commits: Vec<ChainCommit>,
}

#[derive(Serialize)]
pub struct ChainCommit {
    pub commit: String,
    pub date: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ConvergencePair {
    pub file_a: String,
    pub file_b: String,
    pub bytes_a: usize,
    pub bytes_b: usize,
    pub bytes_diff: usize,
    pub bytes_ratio: f64,
}

struct CommitInfo {
    oid: String,
    date: String,
    message: String,
    commit_type: &'static str,
    files_touched: Vec<String>,
}

/// Default cap for convergence pairs to prevent output explosion.
pub const DEFAULT_CONVERGENCE_LIMIT: usize = 50;

/// Minimum file size in bytes for convergence scanning.
pub const MIN_CONVERGENCE_BYTES: usize = 500;

pub fn run(repo: &Repository, since: Option<i64>, limit: Option<usize>) -> Result<PatternsOutput> {
    run_with_convergence_limit(repo, since, limit, DEFAULT_CONVERGENCE_LIMIT)
}

pub fn run_with_convergence_limit(
    repo: &Repository,
    since: Option<i64>,
    limit: Option<usize>,
    convergence_limit: usize,
) -> Result<PatternsOutput> {
    let commits = common::walk_commits(repo, since)?;

    let mut commits_info: Vec<CommitInfo> = Vec::new();

    for commit in &commits {
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
        diff.foreach(
            &mut |delta, _| {
                if let Some(p) = delta.new_file().path().and_then(|p| p.to_str()) {
                    files.push(p.to_string());
                }
                true
            },
            None,
            None,
            None,
        )?;

        commits_info.push(CommitInfo {
            oid: common::short_hash(&commit.id()),
            date: dt.format("%Y-%m-%d").to_string(),
            message: first_line,
            commit_type: common::classify_commit_with_parents(&message, commit.parent_count()),
            files_touched: files,
        });
    }

    let total_commits_analyzed = commits_info.len();

    // 1. Fix-after-feat: find fix commits that follow feat commits within 5 commits
    let mut fix_after_feat = Vec::new();
    for (i, ci) in commits_info.iter().enumerate() {
        if ci.commit_type == "fix" {
            // Look at subsequent entries (older commits, since sorted TIME desc)
            // Actually we want: feat happened recently, then fix came after
            // In our list, index 0 is newest. So a fix at index i, look for feats at i+1..i+6
            for gap in 1..=5 {
                if i + gap >= commits_info.len() {
                    break;
                }
                let older = &commits_info[i + gap];
                if older.commit_type == "feat" {
                    fix_after_feat.push(FixAfterFeat {
                        feat_commit: older.oid.clone(),
                        feat_date: older.date.clone(),
                        feat_message: older.message.clone(),
                        fix_commit: ci.oid.clone(),
                        fix_date: ci.date.clone(),
                        fix_message: ci.message.clone(),
                        gap_commits: gap - 1,
                    });
                    break;
                }
            }
        }
    }

    if let Some(limit) = limit {
        fix_after_feat.truncate(limit);
    }

    // 2. Multi-edit chains: files touched 3+ times in the analyzed window
    let mut file_history: HashMap<String, Vec<ChainCommit>> = HashMap::new();
    for ci in &commits_info {
        for f in &ci.files_touched {
            file_history
                .entry(f.clone())
                .or_default()
                .push(ChainCommit {
                    commit: ci.oid.clone(),
                    date: ci.date.clone(),
                    message: ci.message.clone(),
                });
        }
    }

    let mut multi_edit_chains: Vec<MultiEditChain> = file_history
        .into_iter()
        .filter(|(_, edits)| edits.len() >= 3)
        .map(|(path, commits)| MultiEditChain {
            path,
            edit_count: commits.len(),
            commits,
        })
        .collect();
    multi_edit_chains.sort_by(|a, b| b.edit_count.cmp(&a.edit_count));

    if let Some(limit) = limit {
        multi_edit_chains.truncate(limit);
    }

    // 3. Convergence: files at HEAD with similar sizes (within 10%)
    let mut convergence = Vec::new();
    let head = repo.head()?.peel_to_commit()?;
    let head_tree = head.tree()?;

    // Get sizes of tracked files
    let mut file_sizes: Vec<(String, usize)> = Vec::new();
    head_tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() == Some(git2::ObjectType::Blob) {
            let path = if dir.is_empty() {
                entry.name().unwrap_or("").to_string()
            } else {
                format!("{}{}", dir, entry.name().unwrap_or(""))
            };
            if let Ok(blob) = repo.find_blob(entry.id()) {
                file_sizes.push((path, blob.size()));
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Find pairs with similar sizes (within 10% of each other, min 500 bytes).
    // Sort by size then scan adjacent entries — O(n log n) instead of O(n^2).
    file_sizes.retain(|&(_, size)| size >= MIN_CONVERGENCE_BYTES);
    file_sizes.sort_by_key(|&(_, size)| size);

    for i in 0..file_sizes.len() {
        let (ref pa, sa) = file_sizes[i];
        for j in (i + 1)..file_sizes.len() {
            let (ref pb, sb) = file_sizes[j];
            // Since the list is sorted, sb >= sa. Once the ratio drops below
            // 0.90, all subsequent entries will also be too large.
            let ratio = sa as f64 / sb as f64;
            if ratio < 0.90 {
                break;
            }
            let diff = sb - sa;
            convergence.push(ConvergencePair {
                file_a: pa.clone(),
                file_b: pb.clone(),
                bytes_a: sa,
                bytes_b: sb,
                bytes_diff: diff,
                bytes_ratio: ratio,
            });
        }
    }
    convergence.sort_by(|a, b| b.bytes_ratio.partial_cmp(&a.bytes_ratio).unwrap());

    // Apply convergence-specific limit (--convergence-limit flag, default 50).
    // The generic --limit flag is applied first if present and smaller.
    let effective_limit = match limit {
        Some(l) if l < convergence_limit => l,
        _ => convergence_limit,
    };
    let convergence_truncated = convergence.len() > effective_limit;
    convergence.truncate(effective_limit);

    Ok(PatternsOutput {
        fix_after_feat,
        multi_edit_chains,
        convergence,
        convergence_truncated,
        convergence_limit,
        total_commits_analyzed,
    })
}
