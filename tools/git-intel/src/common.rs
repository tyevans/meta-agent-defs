use anyhow::Result;
use git2::{Commit, Mailmap, Oid, Repository, Signature, Sort};

/// Return the first 7 characters of a git object ID (standard short hash).
pub fn short_hash(oid: &Oid) -> String {
    oid.to_string()[..7].to_string()
}

/// Load the repository's mailmap, returning `None` if unavailable.
///
/// This is a convenience wrapper around `Repository::mailmap()` that
/// converts errors (e.g. no `.mailmap` file) into `None`.
pub fn load_mailmap(repo: &Repository) -> Option<Mailmap> {
    repo.mailmap().ok()
}

/// Resolve an author signature through the mailmap, returning the
/// canonical (name, email) pair.
///
/// If `mailmap` is `None` or resolution fails, the original name and
/// email from the signature are returned unchanged.
pub fn resolve_author(mailmap: Option<&Mailmap>, sig: &Signature) -> (String, String) {
    if let Some(mm) = mailmap {
        if let Ok(resolved) = mm.resolve_signature(sig) {
            return (
                resolved.name().unwrap_or("unknown").to_string(),
                resolved.email().unwrap_or("unknown").to_string(),
            );
        }
    }
    (
        sig.name().unwrap_or("unknown").to_string(),
        sig.email().unwrap_or("unknown").to_string(),
    )
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
/// Returns one of: release, revert, feat, fix, chore, docs, refactor, test, style, perf, ci, build, other.
///
/// Priority order: revert > release > conventional prefix > NL heuristics > "other".
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

    // Release/version detection: "v1.2.3", "Release 2.0", "Bump version to 1.5"
    if lower.starts_with('v') {
        if lower.len() > 1 && lower.as_bytes()[1].is_ascii_digit() {
            return "release";
        }
    }
    if lower.contains("release") || lower.contains("bump version") {
        return "release";
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

/// Classify a commit message with an optional ML fallback.
/// When the rule-based chain returns "other" and an ML classifier is provided,
/// the ML model is consulted. If it returns a label above the confidence threshold,
/// that label is used instead of "other".
///
/// The chain becomes: merge > revert > release > conventional > NL heuristics > [ML] > "other"
#[cfg(feature = "ml")]
pub fn classify_commit_with_ml(message: &str, parent_count: usize, ml: Option<&mut crate::ml::MlClassifier>) -> String {
    let rule_label = classify_commit_with_parents(message, parent_count);
    if rule_label != "other" {
        return rule_label.to_string();
    }
    if let Some(classifier) = ml {
        if let Some((label, _confidence)) = classifier.classify(message) {
            return label;
        }
    }
    "other".to_string()
}

/// Non-ML version that returns String for uniform API when ml feature is disabled.
#[cfg(not(feature = "ml"))]
pub fn classify_commit_with_ml_disabled(message: &str, parent_count: usize) -> String {
    classify_commit_with_parents(message, parent_count).to_string()
}

/// Extract a ticket reference from a commit message.
/// Returns the first match found using this priority order:
/// 1. Bracketed JIRA-style: `[PROJ-123]`
/// 2. JIRA-style: `PROJ-123` (2+ uppercase letters, dash, 1+ digits)
/// 3. "Fixes #N" / "Closes #N" patterns
/// 4. Bare GitHub issue: `#N`
///
/// Returns `None` if no ticket reference is found.
pub fn extract_ticket_ref(message: &str) -> Option<String> {
    let first_line = message.lines().next().unwrap_or("");

    // 1. Bracketed JIRA-style: [PROJ-123]
    if let Some(start) = first_line.find('[') {
        if let Some(end) = first_line[start..].find(']') {
            let inner = &first_line[start + 1..start + end];
            if is_jira_ref(inner) {
                return Some(inner.to_string());
            }
        }
    }

    // 2. Unbracketed JIRA-style: scan for UPPERCASE-DIGITS pattern
    if let Some(ticket) = find_jira_ref(first_line) {
        return Some(ticket);
    }

    // 3. "Fixes #N" / "Closes #N" / "Fixed #N" / "Closed #N" patterns
    let lower = first_line.to_lowercase();
    for keyword in &["fixes #", "fixed #", "closes #", "closed #"] {
        if let Some(pos) = lower.find(keyword) {
            let num_start = pos + keyword.len();
            if let Some(num) = extract_digits(&first_line[num_start..]) {
                return Some(format!("#{}", num));
            }
        }
    }

    // 4. Bare GitHub issue: #N
    if let Some(pos) = first_line.find('#') {
        if let Some(num) = extract_digits(&first_line[pos + 1..]) {
            return Some(format!("#{}", num));
        }
    }

    None
}

/// Check if a string matches JIRA-style: 2+ uppercase letters, dash, 1+ digits.
fn is_jira_ref(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;

    // Need at least 2 uppercase letters
    while i < bytes.len() && bytes[i].is_ascii_uppercase() {
        i += 1;
    }
    if i < 2 {
        return false;
    }

    // Dash
    if i >= bytes.len() || bytes[i] != b'-' {
        return false;
    }
    i += 1;

    // At least 1 digit
    let digit_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digit_start {
        return false;
    }

    // Must consume the entire string
    i == bytes.len()
}

/// Find the first JIRA-style reference in text (word-boundary aware).
fn find_jira_ref(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for start of uppercase sequence
        if bytes[i].is_ascii_uppercase() {
            // Check word boundary: must be at start or preceded by non-alphanumeric
            if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-') {
                i += 1;
                continue;
            }

            let start = i;
            // Count uppercase letters
            while i < len && bytes[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_count = i - start;

            // Need 2+ uppercase, then dash, then 1+ digits
            if upper_count >= 2 && i < len && bytes[i] == b'-' {
                i += 1; // skip dash
                let digit_start = i;
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i > digit_start {
                    // Check trailing boundary: end of string or non-alphanumeric
                    if i >= len || !bytes[i].is_ascii_alphanumeric() {
                        return Some(text[start..i].to_string());
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

/// Extract a run of digits from the start of a string.
fn extract_digits(s: &str) -> Option<String> {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

/// Iterator over commits from HEAD in time order, filtered by optional time bounds.
/// Yields commits newest-first, allowing callers to process and discard incrementally
/// instead of collecting all commits into memory upfront.
pub struct CommitIter<'repo> {
    repo: &'repo Repository,
    revwalk: git2::Revwalk<'repo>,
    since: Option<i64>,
    until: Option<i64>,
    done: bool,
}

impl<'repo> Iterator for CommitIter<'repo> {
    type Item = Result<Commit<'repo>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            let oid = match self.revwalk.next()? {
                Ok(oid) => oid,
                Err(e) => return Some(Err(e.into())),
            };
            let commit = match self.repo.find_commit(oid) {
                Ok(c) => c,
                Err(e) => return Some(Err(e.into())),
            };
            let ts = commit.time().seconds();

            // Skip commits newer than the until bound
            if let Some(until_ts) = self.until {
                if ts > until_ts {
                    continue;
                }
            }

            // Stop once we pass the since bound (commits are time-sorted descending)
            if let Some(since_ts) = self.since {
                if ts < since_ts {
                    self.done = true;
                    return None;
                }
            }

            return Some(Ok(commit));
        }
    }
}

/// Walk commits from HEAD in time order, filtered by optional time bounds.
/// Returns an iterator yielding commits newest-first.
///
/// Callers that need random access (e.g. patterns) should collect locally.
/// Callers that only need a single pass can stream, keeping memory proportional
/// to active processing rather than total commit count.
///
/// - `since`: lower bound (inclusive) — commits before this timestamp are excluded
/// - `until`: upper bound (inclusive) — commits after this timestamp are skipped
pub fn walk_commits<'repo>(
    repo: &'repo Repository,
    since: Option<i64>,
    until: Option<i64>,
) -> Result<CommitIter<'repo>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;

    Ok(CommitIter {
        repo,
        revwalk,
        since,
        until,
        done: false,
    })
}

/// Extract the directory prefix at the given depth from a file path.
/// depth=1: "src/lib.rs" -> "src", "README.md" -> "."
/// depth=2: "src/utils/helper.rs" -> "src/utils", "src/lib.rs" -> "src"
pub fn dir_prefix(path: &str, depth: usize) -> String {
    if depth == 0 {
        return ".".to_string();
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_prefix_depth_0() {
        assert_eq!(dir_prefix("src/lib.rs", 0), ".");
        assert_eq!(dir_prefix("README.md", 0), ".");
    }

    #[test]
    fn dir_prefix_depth_1() {
        assert_eq!(dir_prefix("src/lib.rs", 1), "src");
        assert_eq!(dir_prefix("README.md", 1), ".");
    }

    #[test]
    fn dir_prefix_depth_2() {
        assert_eq!(dir_prefix("src/utils/helper.rs", 2), "src/utils");
        assert_eq!(dir_prefix("src/lib.rs", 2), "src");
        assert_eq!(dir_prefix("README.md", 2), ".");
    }

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

    // ---- release/version commit classification tests ----

    #[test]
    fn classify_release_semver_tag() {
        assert_eq!(classify_commit("v1.2.3"), "release");
        assert_eq!(classify_commit("v0.1.0"), "release");
        assert_eq!(classify_commit("v0.1.0-beta"), "release");
        assert_eq!(classify_commit("v2.0.0-rc.1"), "release");
    }

    #[test]
    fn classify_release_semver_mixed_case() {
        assert_eq!(classify_commit("V1.2.3"), "release");
        assert_eq!(classify_commit("V0.1.0-beta"), "release");
    }

    #[test]
    fn classify_release_keyword() {
        assert_eq!(classify_commit("Release 2.0"), "release");
        assert_eq!(classify_commit("release: v1.0"), "release");
        assert_eq!(classify_commit("RELEASE 3.0"), "release");
        assert_eq!(classify_commit("release v4.5.1"), "release");
    }

    #[test]
    fn classify_release_bump_version() {
        assert_eq!(classify_commit("Bump version to 1.5"), "release");
        assert_eq!(classify_commit("bump version to 2.0.0"), "release");
        assert_eq!(classify_commit("Bump version"), "release");
    }

    #[test]
    fn classify_release_not_false_positive() {
        // "version" alone or "v" without digit should not match
        assert_eq!(classify_commit("very important change"), "other");
        assert_eq!(classify_commit("various updates"), "other");
    }

    #[test]
    fn classify_release_with_parents() {
        // Release with 1 parent should be "release"
        assert_eq!(classify_commit_with_parents("v1.2.3", 1), "release");
        // Merge commit with release message should still be "merge"
        assert_eq!(classify_commit_with_parents("v1.2.3", 2), "merge");
    }

    // ---- ticket reference extraction tests ----

    #[test]
    fn ticket_jira_style_colon() {
        assert_eq!(extract_ticket_ref("JIRA-123: fix login"), Some("JIRA-123".to_string()));
    }

    #[test]
    fn ticket_jira_style_space() {
        assert_eq!(extract_ticket_ref("PROJ-456 update auth"), Some("PROJ-456".to_string()));
    }

    #[test]
    fn ticket_jira_two_letter_prefix() {
        assert_eq!(extract_ticket_ref("FO-1 minimal prefix"), Some("FO-1".to_string()));
    }

    #[test]
    fn ticket_bracketed_jira() {
        assert_eq!(extract_ticket_ref("[PROJ-456] update auth"), Some("PROJ-456".to_string()));
    }

    #[test]
    fn ticket_bracketed_takes_priority() {
        // Bracketed should be found even if unbracketed appears first in text
        assert_eq!(extract_ticket_ref("fix [CORE-99] for JIRA-123"), Some("CORE-99".to_string()));
    }

    #[test]
    fn ticket_github_issue_bare() {
        assert_eq!(extract_ticket_ref("fix typo #789"), Some("#789".to_string()));
    }

    #[test]
    fn ticket_fixes_pattern() {
        assert_eq!(extract_ticket_ref("Fixes #42 — null check"), Some("#42".to_string()));
    }

    #[test]
    fn ticket_closes_pattern() {
        assert_eq!(extract_ticket_ref("Closes #100"), Some("#100".to_string()));
    }

    #[test]
    fn ticket_fixed_pattern() {
        assert_eq!(extract_ticket_ref("Fixed #55 in auth"), Some("#55".to_string()));
    }

    #[test]
    fn ticket_no_reference() {
        assert_eq!(extract_ticket_ref("just a regular commit message"), None);
    }

    #[test]
    fn ticket_empty_message() {
        assert_eq!(extract_ticket_ref(""), None);
    }

    #[test]
    fn ticket_multiline_uses_first_line() {
        assert_eq!(
            extract_ticket_ref("JIRA-100: first line\n\nBody mentions JIRA-200"),
            Some("JIRA-100".to_string())
        );
    }

    #[test]
    fn ticket_single_letter_prefix_no_match() {
        // Single uppercase letter + dash + digits should NOT match (need 2+ letters)
        assert_eq!(extract_ticket_ref("A-123 something"), None);
    }

    #[test]
    fn ticket_jira_mid_sentence() {
        assert_eq!(
            extract_ticket_ref("fix: resolve CORE-42 auth issue"),
            Some("CORE-42".to_string())
        );
    }

    #[test]
    fn ticket_multiple_returns_first_jira() {
        // Multiple JIRA refs: first one wins
        assert_eq!(
            extract_ticket_ref("PROJ-1 and PROJ-2 changes"),
            Some("PROJ-1".to_string())
        );
    }

    #[test]
    fn ticket_hash_without_digits_no_match() {
        assert_eq!(extract_ticket_ref("use # for comments"), None);
    }
}
