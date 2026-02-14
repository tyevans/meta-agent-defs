use git2::{Oid, Repository, Signature, Time};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use git_intel::{churn, lifecycle, metrics, patterns};

/// Create a temporary git repo with a controlled commit history.
///
/// Commits (newest to oldest):
///   5. "docs: update README"        -- modifies README.md
///   4. "fix: handle null input"     -- modifies src/lib.rs
///   3. "chore: add gitignore"       -- creates .gitignore
///   2. "feat: add helper module"    -- creates src/utils.rs
///   1. "feat: initial commit"       -- creates README.md, src/lib.rs
///
/// All commits have controlled timestamps 1 day apart starting at
/// 2026-01-10 00:00:00 UTC (epoch 1736467200).
fn create_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    // Base epoch: 2026-01-10 00:00:00 UTC
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

    // Helper closures cannot borrow `dir` and `repo` simultaneously with
    // the commit calls below, so we use a function-like macro approach.
    // Instead, just inline the work using only Oid values (no lingering borrows).

    let make_sig = |epoch: i64| -> Signature<'static> {
        Signature::new("Test Author", "test@test.com", &Time::new(epoch, 0))
            .expect("create signature")
    };

    // Write files to disk and stage them, returning the tree Oid.
    let stage_files = |repo: &Repository, files: &[(&str, &str)]| -> Oid {
        let mut index = repo.index().expect("get index");
        for (path, content) in files {
            let full_path = dir.path().join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).expect("create dirs");
            }
            fs::write(&full_path, content).expect("write file");
            index.add_path(Path::new(path)).expect("add to index");
        }
        index.write().expect("write index");
        index.write_tree().expect("write tree")
    };

    // Commit helper: creates a commit and returns its Oid.
    // Uses a block scope so Tree/Commit borrows don't escape.
    let do_commit = |repo: &Repository,
                     tree_oid: Oid,
                     parent_oids: &[Oid],
                     sig: &Signature,
                     message: &str|
     -> Oid {
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parents: Vec<git2::Commit> = parent_oids
            .iter()
            .map(|oid| repo.find_commit(*oid).expect("find parent"))
            .collect();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), sig, sig, message, &tree, &parent_refs)
            .expect("create commit")
    };

    // Commit 1: feat: initial commit (README.md + src/lib.rs)
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# My Project\n\nA test project.\n"),
        ("src/lib.rs", "pub fn hello() {\n    println!(\"hello\");\n}\n"),
    ]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Commit 2: feat: add helper module (src/utils.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/utils.rs", "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"),
    ]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "feat: add helper module");

    // Commit 3: chore: add gitignore (.gitignore)
    let tree_oid = stage_files(&repo, &[
        (".gitignore", "target/\n*.tmp\n"),
    ]);
    let s = make_sig(base_epoch + 2 * day);
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "chore: add gitignore");

    // Commit 4: fix: handle null input (modifies src/lib.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/lib.rs", "pub fn hello() {\n    println!(\"hello\");\n}\n\npub fn safe_hello(name: &str) {\n    if !name.is_empty() {\n        println!(\"hello {}\", name);\n    }\n}\n"),
    ]);
    let s = make_sig(base_epoch + 3 * day);
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "fix: handle null input");

    // Commit 5: docs: update README (modifies README.md)
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# My Project\n\nA test project with utilities.\n\n## Usage\n\nSee src/lib.rs for examples.\n"),
    ]);
    let s = make_sig(base_epoch + 4 * day);
    let _c5 = do_commit(&repo, tree_oid, &[c4], &s, "docs: update README");

    (dir, repo)
}

// ---- metrics tests ----

#[test]
fn metrics_total_commits() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None).unwrap();
    assert_eq!(result.total_commits, 5);
}

#[test]
fn metrics_commit_type_counts() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None).unwrap();

    let find_type = |name: &str| -> usize {
        result
            .commit_types
            .iter()
            .find(|ct| ct.type_name == name)
            .map(|ct| ct.count)
            .unwrap_or(0)
    };

    assert_eq!(find_type("feat"), 2);
    assert_eq!(find_type("fix"), 1);
    assert_eq!(find_type("chore"), 1);
    assert_eq!(find_type("docs"), 1);
    assert_eq!(find_type("other"), 0);
}

#[test]
fn metrics_since_filter() {
    let (_dir, repo) = create_fixture();
    // Filter to only commits on or after day 3 (base + 2*day)
    // That should include commits 3, 4, 5
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    let since = Some(base_epoch + 2 * day);
    let result = metrics::run(&repo, since, None).unwrap();
    assert_eq!(result.total_commits, 3);
}

#[test]
fn metrics_velocity_nonzero() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None).unwrap();
    assert!(result.velocity.total_lines_changed > 0);
    assert!(result.velocity.avg_lines_per_commit > 0.0);
}

#[test]
fn metrics_activity_dates() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None).unwrap();
    // 5 commits on 5 different days means 5 activity bursts
    assert_eq!(result.activity.len(), 5);
    for burst in &result.activity {
        assert_eq!(burst.commits, 1);
    }
}

// ---- churn tests ----

#[test]
fn churn_file_counts() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 5);
    // Files: README.md, src/lib.rs, src/utils.rs, .gitignore
    assert_eq!(result.total_files, 4);
}

#[test]
fn churn_sorted_by_total() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None).unwrap();
    // Verify sorted descending by total_churn
    for w in result.files.windows(2) {
        assert!(w[0].total_churn >= w[1].total_churn);
    }
}

#[test]
fn churn_limit() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, Some(2)).unwrap();
    assert!(result.files.len() <= 2);
    // total_files should still reflect the untruncated count
    assert_eq!(result.total_files, 4);
}

#[test]
fn churn_readme_touched_twice() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None).unwrap();
    let readme = result.files.iter().find(|f| f.path == "README.md").unwrap();
    assert_eq!(readme.commit_count, 2);
    assert!(readme.additions > 0);
}

// ---- lifecycle tests ----

#[test]
fn lifecycle_existing_file() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, &["README.md".to_string()]).unwrap();
    assert_eq!(result.files.len(), 1);
    let f = &result.files[0];
    assert!(f.exists);
    assert!(f.current_lines.is_some());
    // README.md was touched in commits 1 and 5
    assert_eq!(f.history.len(), 2);
}

#[test]
fn lifecycle_created_status() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, &["src/utils.rs".to_string()]).unwrap();
    let f = &result.files[0];
    assert_eq!(f.history.len(), 1);
    assert_eq!(f.history[0].status, "created");
}

#[test]
fn lifecycle_nonexistent_file() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, &["nonexistent.txt".to_string()]).unwrap();
    let f = &result.files[0];
    assert!(!f.exists);
    assert!(f.current_lines.is_none());
    assert!(f.history.is_empty());
}

#[test]
fn lifecycle_modified_file_history() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, &["src/lib.rs".to_string()]).unwrap();
    let f = &result.files[0];
    assert!(f.exists);
    // src/lib.rs was created in commit 1, modified in commit 4
    assert_eq!(f.history.len(), 2);

    // History is newest-first (from walk_commits sorting)
    // Commit 4 (fix) should show additions > 0
    assert!(f.history[0].additions > 0);

    // Commit 1 (initial) should be "created"
    assert_eq!(f.history[1].status, "created");
}

// ---- patterns tests ----

#[test]
fn patterns_fix_after_feat() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 5);

    // Commit order (newest first): docs, fix, chore, feat, feat
    // The fix at index 1 should find feat at index 3 (gap of 1 commit: chore)
    assert!(!result.fix_after_feat.is_empty());
    let pair = &result.fix_after_feat[0];
    assert_eq!(pair.fix_message, "fix: handle null input");
    assert!(pair.feat_message.starts_with("feat:"));
    // gap_commits between fix (idx 1) and feat (idx 3) = 3 - 1 - 1 = 1
    assert!(pair.gap_commits <= 3);
}

#[test]
fn patterns_no_multi_edit_chains() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None).unwrap();
    // No file is touched 3+ times in our 5-commit fixture
    // README.md: 2 times, src/lib.rs: 2 times -- not enough for multi-edit
    assert!(result.multi_edit_chains.is_empty());
}

#[test]
fn patterns_convergence_well_formed() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None).unwrap();
    // Convergence pairs are files with similar byte sizes (within 10%, min 100 bytes)
    // Our fixture has small files, some may be below 100 byte threshold
    // Just verify the field is populated and well-formed
    for pair in &result.convergence {
        assert!(pair.bytes_ratio >= 0.90);
        assert!(pair.bytes_ratio <= 1.0);
        assert!(pair.bytes_a >= 100);
        assert!(pair.bytes_b >= 100);
    }
}

#[test]
fn patterns_limit_zero() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, Some(0)).unwrap();
    assert!(result.fix_after_feat.is_empty());
    assert!(result.multi_edit_chains.is_empty());
    assert!(result.convergence.is_empty());
}
