use anyhow::Result;
use git2::Repository;
use serde::Serialize;
use std::collections::HashMap;

use crate::common;

#[derive(Serialize)]
pub struct AuthorsOutput {
    pub directories: Vec<DirectoryAuthors>,
    pub total_authors: usize,
    pub total_commits_analyzed: usize,
    pub depth: usize,
}

#[derive(Serialize)]
pub struct DirectoryAuthors {
    pub path: String,
    pub authors: Vec<AuthorStats>,
    pub top_contributor: String,
    pub bus_factor: usize,
    pub total_commits: usize,
}

#[derive(Serialize, Clone)]
pub struct AuthorStats {
    pub name: String,
    pub email: String,
    pub commits: usize,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

/// Per-author accumulator keyed by email.
#[derive(Default)]
struct AuthorAccum {
    name: String,
    email: String,
    commits: usize,
    lines_added: usize,
    lines_deleted: usize,
}

/// Per-directory accumulator.
#[derive(Default)]
struct DirAccum {
    /// Keyed by author email for dedup.
    authors: HashMap<String, AuthorAccum>,
    total_commits: usize,
}

/// Calculate bus factor: minimum number of authors whose commits exceed 50% of total.
/// Authors are sorted by commits descending; accumulate until >50%.
fn bus_factor(authors: &[AuthorStats], total_commits: usize) -> usize {
    if total_commits == 0 || authors.is_empty() {
        return 0;
    }
    let threshold = total_commits as f64 * 0.5;
    let mut accumulated = 0usize;
    for (i, author) in authors.iter().enumerate() {
        accumulated += author.commits;
        if accumulated as f64 > threshold {
            return i + 1;
        }
    }
    authors.len()
}

pub fn run(
    repo: &Repository,
    since: Option<i64>,
    until: Option<i64>,
    depth: usize,
    limit: Option<usize>,
) -> Result<AuthorsOutput> {
    let commits = common::walk_commits(repo, since, until)?;
    let mut total_commits_analyzed = 0usize;

    let mut dir_map: HashMap<String, DirAccum> = HashMap::new();
    // Track all unique authors globally (by email).
    let mut global_authors: HashMap<String, ()> = HashMap::new();
    let mailmap = common::load_mailmap(repo);

    for result in commits {
        let commit = result?;
        total_commits_analyzed += 1;
        let author_sig = commit.author();
        let (author_name, author_email) = common::resolve_author(mailmap.as_ref(), &author_sig);
        global_authors.entry(author_email.clone()).or_insert(());

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        // Collect per-file line stats for this commit, grouped by directory.
        let mut dir_lines: HashMap<String, (usize, usize)> = HashMap::new();
        // Track which directories this commit touches.
        let mut dirs_touched: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Single pass: file callback to find directories, line callback for add/del.
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    let prefix = common::dir_prefix(path, depth);
                    dirs_touched.insert(prefix);
                }
                true
            },
            None,
            None,
            None,
        )?;

        // Second pass for line-level stats.
        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut |delta, _hunk, line| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    let prefix = common::dir_prefix(path, depth);
                    let entry = dir_lines.entry(prefix).or_insert((0, 0));
                    match line.origin() {
                        '+' => entry.0 += 1,
                        '-' => entry.1 += 1,
                        _ => {}
                    }
                }
                true
            }),
        )?;

        // Accumulate into dir_map.
        for dir in &dirs_touched {
            let dir_acc = dir_map.entry(dir.clone()).or_default();
            dir_acc.total_commits += 1;

            let author_acc = dir_acc
                .authors
                .entry(author_email.clone())
                .or_insert_with(|| AuthorAccum {
                    name: author_name.clone(),
                    email: author_email.clone(),
                    ..Default::default()
                });
            author_acc.commits += 1;

            if let Some(&(added, deleted)) = dir_lines.get(dir) {
                author_acc.lines_added += added;
                author_acc.lines_deleted += deleted;
            }
        }
    }

    let total_authors = global_authors.len();

    let mut directories: Vec<DirectoryAuthors> = dir_map
        .into_iter()
        .map(|(path, acc)| {
            let mut authors: Vec<AuthorStats> = acc
                .authors
                .into_values()
                .map(|a| AuthorStats {
                    name: a.name,
                    email: a.email,
                    commits: a.commits,
                    lines_added: a.lines_added,
                    lines_deleted: a.lines_deleted,
                })
                .collect();
            authors.sort_by(|a, b| b.commits.cmp(&a.commits));

            let top_contributor = authors
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_default();
            let bf = bus_factor(&authors, acc.total_commits);

            DirectoryAuthors {
                path,
                authors,
                top_contributor,
                bus_factor: bf,
                total_commits: acc.total_commits,
            }
        })
        .collect();

    // Sort by total_commits descending for consistent output.
    directories.sort_by(|a, b| b.total_commits.cmp(&a.total_commits));

    if let Some(limit) = limit {
        directories.truncate(limit);
    }

    Ok(AuthorsOutput {
        directories,
        total_authors,
        total_commits_analyzed,
        depth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_factor_single_author() {
        let authors = vec![AuthorStats {
            name: "Alice".into(),
            email: "alice@test.com".into(),
            commits: 10,
            lines_added: 100,
            lines_deleted: 20,
        }];
        assert_eq!(bus_factor(&authors, 10), 1);
    }

    #[test]
    fn bus_factor_two_equal_authors() {
        let authors = vec![
            AuthorStats {
                name: "Alice".into(),
                email: "alice@test.com".into(),
                commits: 5,
                lines_added: 50,
                lines_deleted: 10,
            },
            AuthorStats {
                name: "Bob".into(),
                email: "bob@test.com".into(),
                commits: 5,
                lines_added: 50,
                lines_deleted: 10,
            },
        ];
        // 5/10 = 50%, need >50%, so first author alone is not enough
        // 10/10 = 100% > 50%, so bus_factor = 2
        assert_eq!(bus_factor(&authors, 10), 2);
    }

    #[test]
    fn bus_factor_dominant_author() {
        let authors = vec![
            AuthorStats {
                name: "Alice".into(),
                email: "alice@test.com".into(),
                commits: 8,
                lines_added: 80,
                lines_deleted: 10,
            },
            AuthorStats {
                name: "Bob".into(),
                email: "bob@test.com".into(),
                commits: 2,
                lines_added: 20,
                lines_deleted: 5,
            },
        ];
        // 8/10 = 80% > 50%, so bus_factor = 1
        assert_eq!(bus_factor(&authors, 10), 1);
    }

    #[test]
    fn bus_factor_empty() {
        let authors: Vec<AuthorStats> = vec![];
        assert_eq!(bus_factor(&authors, 0), 0);
    }
}
