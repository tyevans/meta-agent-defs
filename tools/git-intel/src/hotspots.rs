use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::churn;
use crate::common;

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
    pub type_distribution: HashMap<String, usize>,
}

/// Delegate to common::dir_prefix (moved there for reuse by authors subcommand).
fn dir_prefix(path: &str, depth: usize) -> String {
    common::dir_prefix(path, depth)
}

/// Aggregate file-level churn into directory-level hotspots.
///
/// Reuses `churn::run` for diff traversal (additions/deletions/file counts),
/// then walks commits separately to build per-directory commit type distributions
/// using `classify_commit_with_parents`.
pub fn run(
    repo: &Repository,
    since: Option<i64>,
    until: Option<i64>,
    depth: usize,
    limit: Option<usize>,
) -> Result<HotspotsOutput> {
    let churn_output = churn::run(repo, since, until, None)?;
    let total_commits = churn_output.total_commits_analyzed;

    // Phase 1: aggregate churn data by directory
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

    // Phase 2: walk commits to build per-directory type distributions
    let commits = common::walk_commits(repo, since, until)?;
    // Map: dir_prefix -> (commit_type -> count)
    let mut type_dist: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for result in commits {
        let commit = result?;
        let message = commit.message().unwrap_or("");
        let parent_count = commit.parent_count();
        let commit_type = common::classify_commit_with_parents(message, parent_count);

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        // Collect unique directory prefixes touched by this commit
        let mut dirs_touched: HashSet<String> = HashSet::new();
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    dirs_touched.insert(dir_prefix(path, depth));
                }
                true
            },
            None,
            None,
            None,
        )?;

        for dir in dirs_touched {
            *type_dist
                .entry(dir)
                .or_default()
                .entry(commit_type.to_string())
                .or_insert(0) += 1;
        }
    }

    let mut directories: Vec<DirectoryHotspot> = dir_map
        .into_iter()
        .map(|(path, acc)| {
            let td = type_dist.remove(&path).unwrap_or_default();
            DirectoryHotspot {
                path,
                additions: acc.additions,
                deletions: acc.deletions,
                total_churn: acc.additions + acc.deletions,
                // Use sum of file commit counts (may overcount, but gives useful
                // relative ranking — directories with more active files rank higher)
                commit_count: acc.sum_commit_count,
                file_count: acc.file_count,
                type_distribution: td,
            }
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
    fn dir_prefix_depth_0_aggregates_to_root() {
        assert_eq!(dir_prefix("src/lib.rs", 0), ".");
        assert_eq!(dir_prefix("a/b/c/d.rs", 0), ".");
        assert_eq!(dir_prefix("README.md", 0), ".");
    }

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
