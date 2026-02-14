use anyhow::Result;
use git2::Repository;
use serde::Serialize;

use crate::common;

#[derive(Serialize)]
pub struct LifecycleOutput {
    pub files: Vec<FileLifecycle>,
}

#[derive(Serialize)]
pub struct FileLifecycle {
    pub path: String,
    pub exists: bool,
    pub current_lines: Option<usize>,
    pub history: Vec<FileSnapshot>,
}

#[derive(Serialize)]
pub struct FileSnapshot {
    pub commit: String,
    pub date: String,
    pub message: String,
    pub lines: Option<usize>,
    pub additions: usize,
    pub deletions: usize,
    pub net_change: i64,
    pub status: String,
}

fn count_blob_lines(repo: &Repository, tree: &git2::Tree, path: &str) -> Option<usize> {
    let entry = tree.get_path(std::path::Path::new(path)).ok()?;
    let blob = repo.find_blob(entry.id()).ok()?;
    let content = std::str::from_utf8(blob.content()).ok()?;
    Some(content.lines().count())
}

pub fn run(repo: &Repository, since: Option<i64>, files: &[String]) -> Result<LifecycleOutput> {
    let commits = common::walk_commits(repo, since)?;

    let mut result_files = Vec::new();

    for file_path in files {
        let mut history = Vec::new();

        for commit in &commits {
            let tree = commit.tree()?;
            let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

            // Check if this commit touches the file
            let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

            let mut touches_file = false;
            let mut adds = 0usize;
            let mut dels = 0usize;

            diff.foreach(
                &mut |delta, _| {
                    if let Some(p) = delta.new_file().path().and_then(|p| p.to_str()) {
                        if p == file_path.as_str() {
                            touches_file = true;
                        }
                    }
                    if let Some(p) = delta.old_file().path().and_then(|p| p.to_str()) {
                        if p == file_path.as_str() {
                            touches_file = true;
                        }
                    }
                    true
                },
                None,
                None,
                Some(&mut |delta, _hunk, line| {
                    let is_target = delta
                        .new_file()
                        .path()
                        .and_then(|p| p.to_str())
                        .map(|p| p == file_path.as_str())
                        .unwrap_or(false);
                    if is_target {
                        match line.origin() {
                            '+' => adds += 1,
                            '-' => dels += 1,
                            _ => {}
                        }
                    }
                    true
                }),
            )?;

            if !touches_file {
                continue;
            }

            let lines = count_blob_lines(repo, &tree, file_path);
            let time = commit.time().seconds();
            let dt = chrono::DateTime::from_timestamp(time, 0).unwrap_or_default();

            let in_parent = parent_tree
                .as_ref()
                .and_then(|pt| pt.get_path(std::path::Path::new(file_path.as_str())).ok())
                .is_some();
            let status = if lines.is_some() && !in_parent {
                "created"
            } else if lines.is_none() {
                "deleted"
            } else if adds > 0 && dels > 0 {
                "modified"
            } else if adds > 0 {
                "grown"
            } else if dels > 0 {
                "shrunk"
            } else {
                "touched"
            };

            history.push(FileSnapshot {
                commit: commit.id().to_string()[..8].to_string(),
                date: dt.format("%Y-%m-%d").to_string(),
                message: commit
                    .message()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                lines,
                additions: adds,
                deletions: dels,
                net_change: adds as i64 - dels as i64,
                status: status.to_string(),
            });
        }

        // Current state
        let head = repo.head()?.peel_to_commit()?;
        let head_tree = head.tree()?;
        let current_lines = count_blob_lines(repo, &head_tree, file_path);
        let exists = current_lines.is_some();

        result_files.push(FileLifecycle {
            path: file_path.clone(),
            exists,
            current_lines,
            history,
        });
    }

    Ok(LifecycleOutput {
        files: result_files,
    })
}
