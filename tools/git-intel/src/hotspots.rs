use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::churn;

#[derive(Serialize)]
pub struct HotspotsOutput {
    pub directories: Vec<DirectoryHotspot>,
    pub total_directories: usize,
    pub total_commits_analyzed: usize,
    pub depth: usize,
}

#[derive(Serialize)]
pub struct DirectoryHotspot {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub total_churn: usize,
    pub commit_count: usize,
    pub file_count: usize,
}

/// Extract the directory prefix at the given depth from a file path.
/// depth=1: "src/lib.rs" -> "src", "README.md" -> "."
/// depth=2: "src/utils/helper.rs" -> "src/utils", "src/lib.rs" -> "src"
fn dir_prefix(path: &str, depth: usize) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= depth {
        // File is at or above the requested depth — use parent dir or "."
        if parts.len() == 1 {
            ".".to_string()
        } else {
            parts[..parts.len() - 1].join("/")
        }
    } else {
        parts[..depth].join("/")
    }
}

/// Aggregate file-level churn into directory-level hotspots.
///
/// Reuses `churn::run` for diff traversal, then groups by directory prefix
/// at the specified depth.
pub fn run(
    repo: &Repository,
    since: Option<i64>,
    depth: usize,
    limit: Option<usize>,
) -> Result<HotspotsOutput> {
    let churn_output = churn::run(repo, since, None)?;
    let total_commits = churn_output.total_commits_analyzed;

    let mut dir_map: HashMap<String, DirAccumulator> = HashMap::new();

    for file in &churn_output.files {
        let prefix = dir_prefix(&file.path, depth);
        let entry = dir_map.entry(prefix).or_insert_with(DirAccumulator::default);
        entry.additions += file.additions;
        entry.deletions += file.deletions;
        entry.file_count += 1;
        // Track unique commits per directory via max (approximation: sum would
        // overcount when multiple files in same dir change in one commit).
        // For a correct count we'd need per-commit tracking, but commit_count
        // on the file already double-counts across files. We use the max
        // file commit_count as a lower bound, which is still useful.
        if file.commit_count > entry.max_commit_count {
            entry.max_commit_count = file.commit_count;
        }
        entry.sum_commit_count += file.commit_count;
    }

    let mut directories: Vec<DirectoryHotspot> = dir_map
        .into_iter()
        .map(|(path, acc)| DirectoryHotspot {
            path,
            additions: acc.additions,
            deletions: acc.deletions,
            total_churn: acc.additions + acc.deletions,
            // Use sum of file commit counts (may overcount, but gives useful
            // relative ranking — directories with more active files rank higher)
            commit_count: acc.sum_commit_count,
            file_count: acc.file_count,
        })
        .collect();

    directories.sort_by(|a, b| b.total_churn.cmp(&a.total_churn));

    let total_directories = directories.len();

    if let Some(limit) = limit {
        directories.truncate(limit);
    }

    Ok(HotspotsOutput {
        directories,
        total_directories,
        total_commits_analyzed: total_commits,
        depth,
    })
}

#[derive(Default)]
struct DirAccumulator {
    additions: usize,
    deletions: usize,
    file_count: usize,
    max_commit_count: usize,
    sum_commit_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_prefix_depth_1() {
        assert_eq!(dir_prefix("src/lib.rs", 1), "src");
        assert_eq!(dir_prefix("README.md", 1), ".");
        assert_eq!(dir_prefix(".gitignore", 1), ".");
    }

    #[test]
    fn dir_prefix_depth_2() {
        assert_eq!(dir_prefix("src/utils/helper.rs", 2), "src/utils");
        assert_eq!(dir_prefix("src/lib.rs", 2), "src");
        assert_eq!(dir_prefix("README.md", 2), ".");
    }

    #[test]
    fn dir_prefix_depth_3() {
        assert_eq!(dir_prefix("a/b/c/d.rs", 3), "a/b/c");
        assert_eq!(dir_prefix("a/b/c.rs", 3), "a/b");
        assert_eq!(dir_prefix("a/b.rs", 3), "a");
        assert_eq!(dir_prefix("a.rs", 3), ".");
    }
}
