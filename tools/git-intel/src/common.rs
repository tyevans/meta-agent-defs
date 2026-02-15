use anyhow::Result;
use git2::{Commit, Oid, Repository, Sort};

/// Return the first 7 characters of a git object ID (standard short hash).
pub fn short_hash(oid: &Oid) -> String {
    oid.to_string()[..7].to_string()
}

/// Classify a commit by its message and parent count.
/// Merge commits (2+ parents) are always classified as "merge" regardless of message.
/// Otherwise returns one of: feat, fix, chore, docs, refactor, test, style, perf, ci, build, other.
///
/// Matches conventional commit prefixes (`type:`, `type(`, `type!`) and also
/// accepts the bare type word followed by whitespace or end-of-string
/// (e.g. "fix typo" matches "fix").
pub fn classify_commit_with_parents(message: &str, parent_count: usize) -> &'static str {
    if parent_count >= 2 {
        return "merge";
    }
    classify_commit(message)
}

/// Classify a commit message into a conventional-commit type.
/// Returns one of: revert, feat, fix, chore, docs, refactor, test, style, perf, ci, build, other.
///
/// Priority order: revert > conventional prefix > NL heuristics > "other".
///
/// Revert commits are detected first (before conventional commit parsing):
/// - Git's default format: `Revert "..."`
/// - Conventional commit style: `revert:` or `revert(`
///
/// Then matches conventional commit prefixes (`type:`, `type(`, `type!`) and also
/// accepts the bare type word followed by whitespace or end-of-string
/// (e.g. "fix typo" matches "fix").
///
/// If no conventional prefix matches, natural-language heuristics are applied:
/// - Past tense: "Fixed ..." → fix, "Added ..." → feat
/// - Compound words: "bugfix", "bug fix", "hotfix" → fix
/// - GitHub auto-close: "Fixes #", "Fixed #", "Closes #" anywhere → fix
///
/// Note: This does not detect merge commits. Use `classify_commit_with_parents`
/// when parent count is available.
pub fn classify_commit(message: &str) -> &'static str {
    let first_line = message.lines().next().unwrap_or("");
    let lower = first_line.to_lowercase();

    // Revert detection: git default format `Revert "..."` and conventional `revert:` / `revert(`
    if lower.starts_with("revert \"") || lower.starts_with("revert: ") || lower.starts_with("revert:") || lower.starts_with("revert(") {
        return "revert";
    }

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

    // Natural-language heuristics (fallback before "other")
    // Past tense variants at start of message
    if lower.starts_with("fixed ") || lower.starts_with("fixed:") {
        return "fix";
    }
    if lower.starts_with("added ") || lower.starts_with("added:") {
        return "feat";
    }

    // Compound words at start of message
    if lower.starts_with("bugfix") || lower.starts_with("bug fix") {
        return "fix";
    }
    if lower.starts_with("hotfix") {
        return "fix";
    }

    // GitHub auto-close patterns anywhere in the message
    if lower.contains("fixes #") || lower.contains("fixed #") || lower.contains("closes #") {
        return "fix";
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

    // ---- natural-language heuristic tests ----

    #[test]
    fn classify_nl_past_tense_fixed() {
        assert_eq!(classify_commit("Fixed null pointer in auth"), "fix");
        assert_eq!(classify_commit("fixed bug in parser"), "fix");
        assert_eq!(classify_commit("Fixed: handle edge case"), "fix");
    }

    #[test]
    fn classify_nl_past_tense_added() {
        assert_eq!(classify_commit("Added support for dark mode"), "feat");
        assert_eq!(classify_commit("added new endpoint"), "feat");
        assert_eq!(classify_commit("Added: user profile page"), "feat");
    }

    #[test]
    fn classify_nl_github_auto_close() {
        assert_eq!(classify_commit("Fixes #123"), "fix");
        assert_eq!(classify_commit("fixes #456 — null check"), "fix");
        assert_eq!(classify_commit("Fixed #789"), "fix");
        assert_eq!(classify_commit("Closes #42"), "fix");
        assert_eq!(classify_commit("Update parser, closes #99"), "fix");
    }

    #[test]
    fn classify_nl_bugfix_compound() {
        assert_eq!(classify_commit("Bugfix: handle null"), "fix");
        assert_eq!(classify_commit("bugfix for login"), "fix");
        assert_eq!(classify_commit("Bug fix: race condition"), "fix");
        assert_eq!(classify_commit("bug fix in auth module"), "fix");
    }

    #[test]
    fn classify_nl_hotfix() {
        assert_eq!(classify_commit("hotfix: emergency patch"), "fix");
        assert_eq!(classify_commit("Hotfix for prod crash"), "fix");
    }

    #[test]
    fn classify_nl_conventional_still_wins() {
        // Conventional prefix should still match before NL heuristics
        assert_eq!(classify_commit("fix: proper conventional"), "fix");
        assert_eq!(classify_commit("feat: proper conventional"), "feat");
    }

    #[test]
    fn classify_nl_strict_edge_cases() {
        // "fixing" should still be "other" (not caught by NL heuristics either)
        assert_eq!(classify_commit("fixing: something"), "other");
        // "feature" should still be "other"
        assert_eq!(classify_commit("feature: something"), "other");
    }

    // ---- revert commit classification tests ----

    #[test]
    fn classify_revert_git_default_format() {
        // Git's default revert message: Revert "original commit message"
        assert_eq!(classify_commit("Revert \"feat: add login\""), "revert");
        assert_eq!(classify_commit("Revert \"fix: null pointer\""), "revert");
    }

    #[test]
    fn classify_revert_conventional_colon() {
        // Conventional commit style: revert: description
        assert_eq!(classify_commit("revert: undo login feature"), "revert");
        assert_eq!(classify_commit("Revert: undo login feature"), "revert");
        assert_eq!(classify_commit("REVERT: undo login feature"), "revert");
    }

    #[test]
    fn classify_revert_conventional_scope() {
        // Conventional commit with scope: revert(scope): description
        assert_eq!(classify_commit("revert(auth): undo oauth"), "revert");
        assert_eq!(classify_commit("Revert(auth): undo oauth"), "revert");
    }

    #[test]
    fn classify_revert_before_conventional_types() {
        // Revert should be detected before conventional type matching
        // (not misclassified as "other")
        assert_eq!(classify_commit("Revert \"feat: something\""), "revert");
    }

    #[test]
    fn classify_revert_with_parents_normal_commit() {
        // Revert with 1 parent should still be "revert" (not merge)
        assert_eq!(classify_commit_with_parents("Revert \"feat: add login\"", 1), "revert");
        assert_eq!(classify_commit_with_parents("revert: undo feature", 1), "revert");
    }

    #[test]
    fn classify_revert_merge_wins_over_revert() {
        // A merge commit with revert-like message should still be "merge"
        assert_eq!(classify_commit_with_parents("Revert \"feat: add login\"", 2), "merge");
    }

    // ---- merge commit classification tests ----

    #[test]
    fn classify_merge_commit_two_parents() {
        // A merge commit (2 parents) should always return "merge"
        assert_eq!(classify_commit_with_parents("Merge branch 'feature'", 2), "merge");
    }

    #[test]
    fn classify_merge_commit_overrides_conventional_type() {
        // Even if the message looks like a conventional commit, merge wins
        assert_eq!(classify_commit_with_parents("feat: merged feature branch", 2), "merge");
        assert_eq!(classify_commit_with_parents("fix: merge fix branch", 2), "merge");
    }

    #[test]
    fn classify_merge_commit_three_parents() {
        // Octopus merge (3+ parents) should also be "merge"
        assert_eq!(classify_commit_with_parents("Merge branches 'a' and 'b'", 3), "merge");
    }

    #[test]
    fn classify_non_merge_with_parents_one() {
        // Normal commit with 1 parent falls through to message classification
        assert_eq!(classify_commit_with_parents("feat: add login", 1), "feat");
        assert_eq!(classify_commit_with_parents("fix: null pointer", 1), "fix");
    }

    #[test]
    fn classify_root_commit_with_parents_zero() {
        // Root commit (0 parents) falls through to message classification
        assert_eq!(classify_commit_with_parents("feat: initial commit", 0), "feat");
        assert_eq!(classify_commit_with_parents("initial commit", 0), "other");
    }
}
