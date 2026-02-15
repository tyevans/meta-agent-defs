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
    // Convergence pairs are files with similar byte sizes (within 10%, min 500 bytes)
    // Our fixture has small files, all below 500 byte threshold
    // So convergence should be empty
    assert!(result.convergence.is_empty());
    assert!(!result.convergence_truncated);
    assert_eq!(result.convergence_limit, patterns::DEFAULT_CONVERGENCE_LIMIT);
    for pair in &result.convergence {
        assert!(pair.bytes_ratio >= 0.90);
        assert!(pair.bytes_ratio <= 1.0);
        assert!(pair.bytes_a >= patterns::MIN_CONVERGENCE_BYTES);
        assert!(pair.bytes_b >= patterns::MIN_CONVERGENCE_BYTES);
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

// ---- merge commit tests ----

/// Create a fixture with a merge commit:
///   1. "feat: initial commit" on main (README.md)
///   2. "feat: feature work" on a side branch (feature.txt)
///   3. "fix: main fix" on main (hotfix.txt)
///   4. Merge commit joining main and side branch
fn create_merge_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

    let make_sig = |epoch: i64| -> Signature<'static> {
        Signature::new("Test Author", "test@test.com", &Time::new(epoch, 0))
            .expect("create signature")
    };

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

    // Commit 1: initial commit on main
    let tree_oid = stage_files(&repo, &[("README.md", "# Merge Test\n")]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Create side branch from c1
    repo.branch(
        "feature-branch",
        &repo.find_commit(c1).expect("find c1"),
        false,
    )
    .expect("create branch");

    // Commit 2: work on main (hotfix)
    let tree_oid = stage_files(&repo, &[("hotfix.txt", "hotfix content\n")]);
    let s = make_sig(base_epoch + day);
    let c2_main = do_commit(&repo, tree_oid, &[c1], &s, "fix: main hotfix");

    // Commit 3: work on feature branch (check out feature-branch, commit there)
    // We build the tree on top of c1's tree (the branch point)
    let tree_oid = stage_files(&repo, &[("feature.txt", "feature content\n")]);
    let s = make_sig(base_epoch + day);
    // This commit's parent is c1 (the branch point), not c2_main
    let c3_feature = {
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c1).expect("find c1");
        // Don't update HEAD — this is on the side branch
        repo.commit(
            Some("refs/heads/feature-branch"),
            &s,
            &s,
            "feat: feature work",
            &tree,
            &[&parent],
        )
        .expect("create feature commit")
    };

    // Commit 4: merge commit (two parents: c2_main and c3_feature)
    // Build a merged tree containing all files
    let tree_oid = stage_files(
        &repo,
        &[
            ("feature.txt", "feature content\n"),
            ("hotfix.txt", "hotfix content\n"),
        ],
    );
    let s = make_sig(base_epoch + 2 * day);
    let _c4_merge = do_commit(
        &repo,
        tree_oid,
        &[c2_main, c3_feature],
        &s,
        "Merge branch 'feature-branch'",
    );

    (dir, repo)
}

#[test]
fn metrics_merge_commit_detected() {
    let (_dir, repo) = create_merge_fixture();
    let result = metrics::run(&repo, None, None).unwrap();

    let find_type = |name: &str| -> usize {
        result
            .commit_types
            .iter()
            .find(|ct| ct.type_name == name)
            .map(|ct| ct.count)
            .unwrap_or(0)
    };

    // 4 commits total: feat, fix, feat, merge
    assert_eq!(result.total_commits, 4);
    assert_eq!(find_type("merge"), 1);
    assert_eq!(find_type("feat"), 2);
    assert_eq!(find_type("fix"), 1);
}

#[test]
fn metrics_merge_overrides_message_type() {
    // The merge commit message doesn't start with a conventional type,
    // but even if it did, parent_count >= 2 should classify it as "merge"
    let (_dir, repo) = create_merge_fixture();
    let result = metrics::run(&repo, None, None).unwrap();

    let find_type = |name: &str| -> usize {
        result
            .commit_types
            .iter()
            .find(|ct| ct.type_name == name)
            .map(|ct| ct.count)
            .unwrap_or(0)
    };

    // "other" should be 0 — the merge commit is "merge", not "other"
    assert_eq!(find_type("other"), 0);
    assert_eq!(find_type("merge"), 1);
}

// ---- revert commit tests ----

/// Create a fixture with a revert commit:
///   1. "feat: initial commit" (README.md)
///   2. "feat: add feature" (feature.txt)
///   3. "Revert \"feat: add feature\"" (removes feature.txt content)
fn create_revert_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

    let make_sig = |epoch: i64| -> Signature<'static> {
        Signature::new("Test Author", "test@test.com", &Time::new(epoch, 0))
            .expect("create signature")
    };

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

    // Commit 1: feat: initial commit
    let tree_oid = stage_files(&repo, &[("README.md", "# Revert Test\n")]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Commit 2: feat: add feature
    let tree_oid = stage_files(&repo, &[("feature.txt", "feature content\n")]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "feat: add feature");

    // Commit 3: Revert "feat: add feature" (simulated by overwriting feature.txt)
    let tree_oid = stage_files(&repo, &[("feature.txt", "")]);
    let s = make_sig(base_epoch + 2 * day);
    let _c3 = do_commit(
        &repo,
        tree_oid,
        &[c2],
        &s,
        "Revert \"feat: add feature\"",
    );

    (dir, repo)
}

#[test]
fn metrics_revert_commit_detected() {
    let (_dir, repo) = create_revert_fixture();
    let result = metrics::run(&repo, None, None).unwrap();

    let find_type = |name: &str| -> usize {
        result
            .commit_types
            .iter()
            .find(|ct| ct.type_name == name)
            .map(|ct| ct.count)
            .unwrap_or(0)
    };

    // 3 commits: feat, feat, revert
    assert_eq!(result.total_commits, 3);
    assert_eq!(find_type("revert"), 1);
    assert_eq!(find_type("feat"), 2);
    assert_eq!(find_type("other"), 0);
}

// ---- convergence limit tests ----

/// Create a fixture with many similarly-sized files (>500 bytes each) to
/// produce enough convergence pairs for truncation testing.
/// Creates 15 files of ~510-520 bytes each (all within 10% of each other),
/// which produces C(15,2) = 105 pairs.
fn create_convergence_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;

    let make_sig = |epoch: i64| -> Signature<'static> {
        Signature::new("Test Author", "test@test.com", &Time::new(epoch, 0))
            .expect("create signature")
    };

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

    // Generate 15 files, each ~600+ bytes (all within 10% of each other).
    // This produces C(15,2) = 105 convergence pairs.
    let mut files: Vec<(String, String)> = Vec::new();
    for i in 0..15 {
        let name = format!("src/module_{:02}.rs", i);
        // Pad base content to well over 500 bytes, with small per-file variation
        let padding_line = format!("// padding line for module {} to ensure file exceeds minimum size threshold\n", i);
        let content = format!(
            "// Module {i} — auto-generated test fixture\n\
             // This file exists to test convergence pair detection.\n\
             {pad}\
             {pad}\
             pub struct Handler{i} {{\n\
             {body}\
             }}\n\
             \n\
             impl Handler{i} {{\n\
             {methods}\
             }}\n",
            i = i,
            pad = padding_line,
            body = "    pub name: String,\n    pub value: i64,\n    pub enabled: bool,\n    pub description: String,\n    pub metadata: Vec<String>,\n    pub tags: Vec<String>,\n    pub priority: u32,\n",
            methods = "    pub fn new() -> Self {\n\
                        Self { name: String::new(), value: 0, enabled: false, description: String::new(), metadata: Vec::new(), tags: Vec::new(), priority: 0 }\n\
                    }\n\
                    \n\
                    pub fn process(&self) -> Result<(), String> {\n\
                        if self.enabled { Ok(()) } else { Err(\"disabled\".to_string()) }\n\
                    }\n",
        );
        files.push((name, content));
    }

    // Also add a tiny file (<500 bytes) that should be excluded
    files.push(("tiny.txt".to_string(), "hello\n".to_string()));

    let file_refs: Vec<(&str, &str)> = files.iter().map(|(n, c)| (n.as_str(), c.as_str())).collect();
    let tree_oid = stage_files(&repo, &file_refs);
    let s = make_sig(base_epoch);
    {
        let tree = repo.find_tree(tree_oid).expect("find tree");
        repo.commit(Some("HEAD"), &s, &s, "feat: initial commit", &tree, &[])
            .expect("create commit");
    }

    (dir, repo)
}

#[test]
fn convergence_default_limit() {
    let (_dir, repo) = create_convergence_fixture();
    let result = patterns::run(&repo, None, None).unwrap();

    // 15 similar files produce C(15,2) = 105 pairs, default limit = 50
    assert_eq!(result.convergence_limit, 50);
    assert!(result.convergence.len() <= 50);
    assert!(result.convergence_truncated);
}

#[test]
fn convergence_custom_limit() {
    let (_dir, repo) = create_convergence_fixture();
    let result = patterns::run_with_convergence_limit(&repo, None, None, 10).unwrap();

    assert_eq!(result.convergence_limit, 10);
    assert!(result.convergence.len() <= 10);
    assert!(result.convergence_truncated);
}

#[test]
fn convergence_truncated_false_when_under_limit() {
    let (_dir, repo) = create_convergence_fixture();
    // Set limit higher than total possible pairs (105)
    let result = patterns::run_with_convergence_limit(&repo, None, None, 200).unwrap();

    assert_eq!(result.convergence_limit, 200);
    assert!(!result.convergence_truncated);
}

#[test]
fn convergence_min_file_size_excludes_small_files() {
    let (_dir, repo) = create_convergence_fixture();
    let result = patterns::run_with_convergence_limit(&repo, None, None, 200).unwrap();

    // The tiny.txt file (6 bytes) should be excluded from all pairs
    for pair in &result.convergence {
        assert_ne!(pair.file_a, "tiny.txt");
        assert_ne!(pair.file_b, "tiny.txt");
        assert!(pair.bytes_a >= patterns::MIN_CONVERGENCE_BYTES);
        assert!(pair.bytes_b >= patterns::MIN_CONVERGENCE_BYTES);
    }
}

#[test]
fn convergence_generic_limit_overrides_when_smaller() {
    let (_dir, repo) = create_convergence_fixture();
    // generic --limit=5 is smaller than convergence_limit=50
    let result = patterns::run_with_convergence_limit(&repo, None, Some(5), 50).unwrap();

    // convergence_limit field shows the configured value, but effective limit is 5
    assert_eq!(result.convergence_limit, 50);
    assert!(result.convergence.len() <= 5);
    assert!(result.convergence_truncated);
}

// ---- natural-language heuristic integration tests ----

/// Create a fixture with NL-style commit messages (no conventional prefixes):
///   1. "initial setup" — other (root commit)
///   2. "Added support for dark mode" — feat (NL heuristic)
///   3. "Fixed null pointer in auth" — fix (NL heuristic)
///   4. "Bugfix: handle edge case" — fix (NL heuristic)
///   5. "hotfix: emergency patch" — fix (NL heuristic)
///   6. "Update parser, closes #42" — fix (NL heuristic, GitHub auto-close)
fn create_nl_heuristic_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

    let make_sig = |epoch: i64| -> Signature<'static> {
        Signature::new("Test Author", "test@test.com", &Time::new(epoch, 0))
            .expect("create signature")
    };

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

    let tree_oid = stage_files(&repo, &[("README.md", "# NL Test\n")]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "initial setup");

    let tree_oid = stage_files(&repo, &[("dark-mode.rs", "// dark mode\n")]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "Added support for dark mode");

    let tree_oid = stage_files(&repo, &[("auth.rs", "// fixed null\n")]);
    let s = make_sig(base_epoch + 2 * day);
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "Fixed null pointer in auth");

    let tree_oid = stage_files(&repo, &[("edge.rs", "// bugfix\n")]);
    let s = make_sig(base_epoch + 3 * day);
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "Bugfix: handle edge case");

    let tree_oid = stage_files(&repo, &[("hotfix.rs", "// hotfix\n")]);
    let s = make_sig(base_epoch + 4 * day);
    let c5 = do_commit(&repo, tree_oid, &[c4], &s, "hotfix: emergency patch");

    let tree_oid = stage_files(&repo, &[("parser.rs", "// parser fix\n")]);
    let s = make_sig(base_epoch + 5 * day);
    let _c6 = do_commit(&repo, tree_oid, &[c5], &s, "Update parser, closes #42");

    (dir, repo)
}

#[test]
fn metrics_nl_heuristic_classification() {
    let (_dir, repo) = create_nl_heuristic_fixture();
    let result = metrics::run(&repo, None, None).unwrap();

    let find_type = |name: &str| -> usize {
        result
            .commit_types
            .iter()
            .find(|ct| ct.type_name == name)
            .map(|ct| ct.count)
            .unwrap_or(0)
    };

    // 6 commits: other, feat, fix, fix, fix, fix
    assert_eq!(result.total_commits, 6);
    assert_eq!(find_type("feat"), 1);   // "Added support for dark mode"
    assert_eq!(find_type("fix"), 4);    // Fixed, Bugfix, hotfix, closes #42
    assert_eq!(find_type("other"), 1);  // "initial setup"
}
