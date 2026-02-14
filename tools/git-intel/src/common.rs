use anyhow::Result;
use git2::{Commit, Oid, Repository, Sort};

/// Return the first 7 characters of a git object ID (standard short hash).
pub fn short_hash(oid: &Oid) -> String {
    oid.to_string()[..7].to_string()
}

/// Classify a commit message into a conventional-commit type.
/// Returns one of: feat, fix, chore, docs, refactor, test, style, perf, ci, build, other.
///
/// Matches conventional commit prefixes (`type:`, `type(`, `type!`) and also
/// accepts the bare type word followed by whitespace or end-of-string
/// (e.g. "fix typo" matches "fix").
pub fn classify_commit(message: &str) -> &'static str {
    let first_line = message.lines().next().unwrap_or("");
    let lower = first_line.to_lowercase();

    let types: &[(&str, &str)] = &[
        ("feat", "feat"),
        ("fix", "fix"),
        ("chore", "chore"),
        ("docs", "docs"),
        ("refactor", "refactor"),
        ("test", "test"),
        ("style", "style"),
        ("perf", "perf"),
        ("ci", "ci"),
        ("build", "build"),
    ];

    for &(prefix, label) in types {
        if lower.starts_with(prefix) {
            // Check what follows the prefix: must be ':', '(', '!', whitespace, or end-of-string
            let rest = &lower[prefix.len()..];
            if rest.is_empty()
                || rest.starts_with(':')
                || rest.starts_with('(')
                || rest.starts_with('!')
                || rest.starts_with(char::is_whitespace)
            {
                return label;
            }
        }
    }

    "other"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_standard_types() {
        assert_eq!(classify_commit("feat: add login"), "feat");
        assert_eq!(classify_commit("fix: null pointer"), "fix");
        assert_eq!(classify_commit("chore: bump deps"), "chore");
        assert_eq!(classify_commit("docs: update README"), "docs");
        assert_eq!(classify_commit("refactor: extract method"), "refactor");
        assert_eq!(classify_commit("test: add unit tests"), "test");
        assert_eq!(classify_commit("style: format code"), "style");
        assert_eq!(classify_commit("perf: optimize query"), "perf");
        assert_eq!(classify_commit("ci: add workflow"), "ci");
        assert_eq!(classify_commit("build: update Makefile"), "build");
    }

    #[test]
    fn classify_mixed_case() {
        assert_eq!(classify_commit("Feat: uppercase"), "feat");
        assert_eq!(classify_commit("FIX: all caps"), "fix");
        assert_eq!(classify_commit("CHORE: shouting"), "chore");
    }

    #[test]
    fn classify_with_scope() {
        // starts_with("feat") matches "feat(auth)" since "feat(auth)" starts with "feat"
        assert_eq!(classify_commit("feat(auth): add oauth"), "feat");
        assert_eq!(classify_commit("fix(db): connection leak"), "fix");
    }

    #[test]
    fn classify_empty_string() {
        assert_eq!(classify_commit(""), "other");
    }

    #[test]
    fn classify_multiline_uses_first_line() {
        assert_eq!(
            classify_commit("feat: first line\n\nThis is a body with fix: mentions"),
            "feat"
        );
    }

    #[test]
    fn classify_prefix_matching_edge_cases() {
        // "fixing" is not "fix:" — should be "other" with strict matching
        assert_eq!(classify_commit("fixing: something"), "other");
        // "feature" is not "feat:" — should be "other"
        assert_eq!(classify_commit("feature: something"), "other");
        // "testing" is not "test:" — should be "other"
        assert_eq!(classify_commit("testing: something"), "other");
    }

    #[test]
    fn classify_bare_type_with_space() {
        // Bare type word followed by space (non-conventional but common)
        assert_eq!(classify_commit("fix typo"), "fix");
        assert_eq!(classify_commit("feat something new"), "feat");
        assert_eq!(classify_commit("test add coverage"), "test");
    }

    #[test]
    fn classify_breaking_change_indicator() {
        assert_eq!(classify_commit("feat!: breaking change"), "feat");
        assert_eq!(classify_commit("fix!: breaking fix"), "fix");
    }

    #[test]
    fn classify_type_alone() {
        // Just the type word with nothing after
        assert_eq!(classify_commit("fix"), "fix");
        assert_eq!(classify_commit("feat"), "feat");
    }

    #[test]
    fn classify_unrecognized() {
        assert_eq!(classify_commit("release: v1.0"), "other");
        assert_eq!(classify_commit("random commit message"), "other");
        assert_eq!(classify_commit("WIP"), "other");
    }
}
