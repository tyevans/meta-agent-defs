use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::common;

#[derive(Serialize)]
pub struct ChurnOutput {
    pub files: Vec<FileChurn>,
    pub total_files: usize,
    pub total_commits_analyzed: usize,
}

#[derive(Serialize)]
pub struct FileChurn {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub total_churn: usize,
    pub commit_count: usize,
}

struct FileChurnAccumulator {
    additions: usize,
    deletions: usize,
    commit_count: usize,
}

pub fn run(repo: &Repository, since: Option<i64>, until: Option<i64>, limit: Option<usize>) -> Result<ChurnOutput> {
    let commits = common::walk_commits(repo, since, until)?;

    let mut accum: HashMap<String, FileChurnAccumulator> = HashMap::new();
    let total_commits = commits.len();

    for commit in &commits {
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        // Pass 1: count which files are touched in this commit
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    accum
                        .entry(path.to_string())
                        .or_insert_with(|| FileChurnAccumulator {
                            additions: 0,
                            deletions: 0,
                            commit_count: 0,
                        })
                        .commit_count += 1;
                }
                true
            },
            None,
            None,
            None,
        )?;

        // Pass 2: count line-level additions and deletions
        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    let entry = accum
                        .entry(path.to_string())
                        .or_insert_with(|| FileChurnAccumulator {
                            additions: 0,
                            deletions: 0,
                            commit_count: 0,
                        });
                    match line.origin() {
                        '+' => entry.additions += 1,
                        '-' => entry.deletions += 1,
                        _ => {}
                    }
                }
                true
            }),
        )?;
    }

    let mut files: Vec<FileChurn> = accum
        .into_iter()
        .map(|(path, acc)| FileChurn {
            path,
            additions: acc.additions,
            deletions: acc.deletions,
            total_churn: acc.additions + acc.deletions,
            commit_count: acc.commit_count,
        })
        .collect();

    files.sort_by(|a, b| b.total_churn.cmp(&a.total_churn));

    let total_files = files.len();

    if let Some(limit) = limit {
        files.truncate(limit);
    }

    Ok(ChurnOutput {
        files,
        total_files,
        total_commits_analyzed: total_commits,
    })
}
