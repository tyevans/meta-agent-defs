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

pub fn run(repo: &Repository, since: Option<i64>, limit: Option<usize>) -> Result<ChurnOutput> {
    let commits = common::walk_commits(repo, since)?;

    let mut file_adds: HashMap<String, usize> = HashMap::new();
    let mut file_dels: HashMap<String, usize> = HashMap::new();
    let mut file_commits: HashMap<String, usize> = HashMap::new();
    let total_commits = commits.len();

    for commit in &commits {
        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    *file_commits.entry(path.to_string()).or_insert(0) += 1;
                }
                true
            },
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    let key = path.to_string();
                    match line.origin() {
                        '+' => *file_adds.entry(key).or_insert(0) += 1,
                        '-' => *file_dels.entry(key).or_insert(0) += 1,
                        _ => {}
                    }
                }
                true
            }),
        )?;
    }

    let all_files: std::collections::HashSet<&String> = file_adds
        .keys()
        .chain(file_dels.keys())
        .chain(file_commits.keys())
        .collect();

    let mut files: Vec<FileChurn> = all_files
        .into_iter()
        .map(|path| {
            let additions = *file_adds.get(path).unwrap_or(&0);
            let deletions = *file_dels.get(path).unwrap_or(&0);
            FileChurn {
                path: path.clone(),
                additions,
                deletions,
                total_churn: additions + deletions,
                commit_count: *file_commits.get(path).unwrap_or(&0),
            }
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
