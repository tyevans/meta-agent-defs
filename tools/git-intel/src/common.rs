use anyhow::Result;
use git2::{Commit, Repository, Sort};

/// Classify a commit message into a conventional-commit type.
/// Returns one of: feat, fix, chore, docs, refactor, test, style, perf, ci, build, other.
pub fn classify_commit(message: &str) -> &'static str {
    let first_line = message.lines().next().unwrap_or("");
    let lower = first_line.to_lowercase();
    if lower.starts_with("feat") {
        "feat"
    } else if lower.starts_with("fix") {
        "fix"
    } else if lower.starts_with("chore") {
        "chore"
    } else if lower.starts_with("docs") {
        "docs"
    } else if lower.starts_with("refactor") {
        "refactor"
    } else if lower.starts_with("test") {
        "test"
    } else if lower.starts_with("style") {
        "style"
    } else if lower.starts_with("perf") {
        "perf"
    } else if lower.starts_with("ci") {
        "ci"
    } else if lower.starts_with("build") {
        "build"
    } else {
        "other"
    }
}

/// Walk commits from HEAD in time order, filtered by an optional since timestamp.
/// Returns commits newest-first.
pub fn walk_commits<'repo>(
    repo: &'repo Repository,
    since: Option<i64>,
) -> Result<Vec<Commit<'repo>>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        if let Some(since_ts) = since {
            if commit.time().seconds() < since_ts {
                break;
            }
        }

        commits.push(commit);
    }

    Ok(commits)
}
