use git2::{Oid, Repository, Signature, Time};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use git_intel::{cache, churn, hotspots, lifecycle, metrics, patterns};

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
    let result = metrics::run(&repo, None, None, None).unwrap();
    assert_eq!(result.total_commits, 5);
}

#[test]
fn metrics_commit_type_counts() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None, None).unwrap();

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
    let result = metrics::run(&repo, since, None, None).unwrap();
    assert_eq!(result.total_commits, 3);
}

#[test]
fn metrics_velocity_nonzero() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None, None).unwrap();
    assert!(result.velocity.total_lines_changed > 0);
    assert!(result.velocity.avg_lines_per_commit > 0.0);
}

#[test]
fn metrics_activity_dates() {
    let (_dir, repo) = create_fixture();
    let result = metrics::run(&repo, None, None, None).unwrap();
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
    let result = churn::run(&repo, None, None, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 5);
    // Files: README.md, src/lib.rs, src/utils.rs, .gitignore
    assert_eq!(result.total_files, 4);
}

#[test]
fn churn_sorted_by_total() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None, None).unwrap();
    // Verify sorted descending by total_churn
    for w in result.files.windows(2) {
        assert!(w[0].total_churn >= w[1].total_churn);
    }
}

#[test]
fn churn_limit() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None, Some(2)).unwrap();
    assert!(result.files.len() <= 2);
    // total_files should still reflect the untruncated count
    assert_eq!(result.total_files, 4);
}

#[test]
fn churn_readme_touched_twice() {
    let (_dir, repo) = create_fixture();
    let result = churn::run(&repo, None, None, None).unwrap();
    let readme = result.files.iter().find(|f| f.path == "README.md").unwrap();
    assert_eq!(readme.commit_count, 2);
    assert!(readme.additions > 0);
}

// ---- lifecycle tests ----

#[test]
fn lifecycle_existing_file() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, None, &["README.md".to_string()]).unwrap();
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
    let result = lifecycle::run(&repo, None, None, &["src/utils.rs".to_string()]).unwrap();
    let f = &result.files[0];
    assert_eq!(f.history.len(), 1);
    assert_eq!(f.history[0].status, "created");
}

#[test]
fn lifecycle_nonexistent_file() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, None, &["nonexistent.txt".to_string()]).unwrap();
    let f = &result.files[0];
    assert!(!f.exists);
    assert!(f.current_lines.is_none());
    assert!(f.history.is_empty());
}

#[test]
fn lifecycle_modified_file_history() {
    let (_dir, repo) = create_fixture();
    let result = lifecycle::run(&repo, None, None, &["src/lib.rs".to_string()]).unwrap();
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
    let result = patterns::run(&repo, None, None, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 5);

    // Commit order (newest first): docs, fix, chore, feat, feat
    // The fix at index 1 modifies src/lib.rs, feat at index 3 (initial commit) created src/lib.rs
    // They share src/lib.rs, so fix-after-feat should detect the pair
    assert!(!result.fix_after_feat.is_empty());
    let pair = &result.fix_after_feat[0];
    assert_eq!(pair.fix_message, "fix: handle null input");
    assert!(pair.feat_message.starts_with("feat:"));
    assert!(pair.gap_commits <= 3);
    assert!(pair.shared_files.contains(&"src/lib.rs".to_string()));
}

#[test]
fn patterns_no_multi_edit_chains() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();
    // No file is touched 3+ times in our 5-commit fixture
    // README.md: 2 times, src/lib.rs: 2 times -- not enough for multi-edit
    assert!(result.multi_edit_chains.is_empty());
}

#[test]
fn patterns_limit_zero() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None, Some(0)).unwrap();
    assert!(result.fix_after_feat.is_empty());
    assert!(result.multi_edit_chains.is_empty());
    assert!(result.temporal_clusters.is_empty());
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
    let result = metrics::run(&repo, None, None, None).unwrap();

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
    let result = metrics::run(&repo, None, None, None).unwrap();

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
    let result = metrics::run(&repo, None, None, None).unwrap();

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
    let result = metrics::run(&repo, None, None, None).unwrap();

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

// ---- cache tests ----

#[test]
fn cache_miss_then_hit() {
    let (_dir, repo) = create_fixture();
    let key = cache::cache_key("metrics", None, None, None);

    // Cache should miss on first read
    assert!(cache::read_cache(&repo, &key).is_none());

    // Compute and write
    let result = metrics::run(&repo, None, None, None).unwrap();
    cache::write_cache(&repo, &key, &result).unwrap();

    // Cache should hit now
    let cached = cache::read_cache(&repo, &key);
    assert!(cached.is_some());

    // Cached JSON should deserialize to same total_commits
    let cached_json: serde_json::Value =
        serde_json::from_str(&cached.unwrap()).unwrap();
    assert_eq!(cached_json["total_commits"], 5);
}

#[test]
fn cache_invalidated_by_new_commit() {
    let (dir, repo) = create_fixture();
    let key = cache::cache_key("metrics", None, None, None);

    // Write cache
    let result = metrics::run(&repo, None, None, None).unwrap();
    cache::write_cache(&repo, &key, &result).unwrap();
    assert!(cache::read_cache(&repo, &key).is_some());

    // Make a new commit to change HEAD
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    let sig = Signature::new("Test Author", "test@test.com", &Time::new(base_epoch + 10 * day, 0))
        .expect("create sig");
    let mut index = repo.index().unwrap();
    let new_file = dir.path().join("new.txt");
    fs::write(&new_file, "new content\n").unwrap();
    index.add_path(Path::new("new.txt")).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    let parent = repo.find_commit(head).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "feat: new commit", &tree, &[&parent])
        .unwrap();

    // Cache should miss now (HEAD changed)
    assert!(cache::read_cache(&repo, &key).is_none());
}

#[test]
fn cache_separate_keys_for_since() {
    let (_dir, repo) = create_fixture();
    let key_all = cache::cache_key("metrics", None, None, None);
    let key_since = cache::cache_key("metrics", Some(1736467200), None, None);
    assert_ne!(key_all, key_since);

    // Write cache for "all"
    let result = metrics::run(&repo, None, None, None).unwrap();
    cache::write_cache(&repo, &key_all, &result).unwrap();

    // "since" key should still miss
    assert!(cache::read_cache(&repo, &key_since).is_none());
    // "all" key should hit
    assert!(cache::read_cache(&repo, &key_all).is_some());
}

// ---- hotspots tests ----

#[test]
fn hotspots_depth_0_aggregates_to_root() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 0, None).unwrap();
    assert_eq!(result.depth, 0);
    // depth=0 should aggregate everything into a single "." entry
    assert_eq!(result.total_directories, 1);
    assert_eq!(result.directories[0].path, ".");
    // file_count should be the total number of distinct files
    assert!(result.directories[0].file_count > 0);
    assert!(result.directories[0].total_churn > 0);
}

#[test]
fn hotspots_depth_1_groups_by_top_level() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();
    assert_eq!(result.depth, 1);
    assert_eq!(result.total_commits_analyzed, 5);

    // Fixture files: README.md (root), src/lib.rs, src/utils.rs (src), .gitignore (root)
    // depth=1 should give us "." (root files) and "src"
    assert_eq!(result.total_directories, 2);

    let find_dir = |name: &str| -> &hotspots::DirectoryHotspot {
        result.directories.iter().find(|d| d.path == name).unwrap()
    };

    let src = find_dir("src");
    assert_eq!(src.file_count, 2); // lib.rs + utils.rs

    let root = find_dir(".");
    assert_eq!(root.file_count, 2); // README.md + .gitignore
}

#[test]
fn hotspots_sorted_by_churn_descending() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();

    for w in result.directories.windows(2) {
        assert!(w[0].total_churn >= w[1].total_churn);
    }
}

#[test]
fn hotspots_limit_truncates() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, Some(1)).unwrap();

    assert_eq!(result.directories.len(), 1);
    // total_directories still reflects the full count
    assert_eq!(result.total_directories, 2);
}

#[test]
fn hotspots_depth_2_preserves_nested() {
    let (_dir, repo) = create_fixture();
    // At depth 2, "src/lib.rs" -> "src" (only 1 level deep), root files -> "."
    // Same as depth 1 for this fixture since src/ has no subdirs
    let result = hotspots::run(&repo, None, None, 2, None).unwrap();
    assert_eq!(result.total_directories, 2);
}

#[test]
fn hotspots_churn_sums_match() {
    let (_dir, repo) = create_fixture();
    let hotspot_result = hotspots::run(&repo, None, None, 1, None).unwrap();
    let churn_result = churn::run(&repo, None, None, None).unwrap();

    // Total additions across all hotspot dirs should equal total across all churn files
    let hotspot_adds: usize = hotspot_result.directories.iter().map(|d| d.additions).sum();
    let churn_adds: usize = churn_result.files.iter().map(|f| f.additions).sum();
    assert_eq!(hotspot_adds, churn_adds);

    let hotspot_dels: usize = hotspot_result.directories.iter().map(|d| d.deletions).sum();
    let churn_dels: usize = churn_result.files.iter().map(|f| f.deletions).sum();
    assert_eq!(hotspot_dels, churn_dels);
}

#[test]
fn hotspots_since_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Only commits 3, 4, 5 (chore: gitignore, fix: lib.rs, docs: README)
    let since = Some(base_epoch + 2 * day);
    let result = hotspots::run(&repo, since, None, 1, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 3);
}

#[test]
fn hotspots_json_output_shape() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["directories"].is_array());
    assert!(parsed["total_directories"].is_number());
    assert!(parsed["total_commits_analyzed"].is_number());
    assert!(parsed["depth"].is_number());

    // Check directory entry shape
    let first = &parsed["directories"][0];
    assert!(first["path"].is_string());
    assert!(first["additions"].is_number());
    assert!(first["deletions"].is_number());
    assert!(first["total_churn"].is_number());
    assert!(first["commit_count"].is_number());
    assert!(first["file_count"].is_number());
}

// ---- until filter tests ----

#[test]
fn metrics_until_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Until end of day 2 (base + 2*day + 86399) should include commits 1, 2, 3
    let until = Some(base_epoch + 2 * day + 86399);
    let result = metrics::run(&repo, None, until, None).unwrap();
    assert_eq!(result.total_commits, 3);
}

#[test]
fn metrics_since_and_until_range() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Range: day 2 through day 3 (commits 2, 3, 4)
    let since = Some(base_epoch + day);
    let until = Some(base_epoch + 3 * day + 86399);
    let result = metrics::run(&repo, since, until, None).unwrap();
    assert_eq!(result.total_commits, 3);
}

#[test]
fn metrics_until_before_all_commits() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    // Until before the first commit — should return 0 commits
    let until = Some(base_epoch - 1);
    let result = metrics::run(&repo, None, until, None).unwrap();
    assert_eq!(result.total_commits, 0);
}

#[test]
fn churn_until_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Until end of day 1 — only commits 1 and 2
    let until = Some(base_epoch + day + 86399);
    let result = churn::run(&repo, None, until, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 2);
}

#[test]
fn hotspots_until_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Until end of day 2 — commits 1, 2, 3
    let until = Some(base_epoch + 2 * day + 86399);
    let result = hotspots::run(&repo, None, until, 1, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 3);
}

#[test]
fn hotspots_depth_exceeds_path_depth() {
    let (_dir, repo) = create_fixture();
    // depth=10 is deeper than any path in the fixture — should still work,
    // grouping files by their full parent directory
    let result = hotspots::run(&repo, None, None, 10, None).unwrap();
    assert_eq!(result.total_directories, 2); // "." for root files, "src" for src/lib.rs
    let paths: Vec<&str> = result.directories.iter().map(|d| d.path.as_str()).collect();
    assert!(paths.contains(&"."));
    assert!(paths.contains(&"src"));
}

#[test]
fn patterns_until_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Until end of day 1 — only commits 1 and 2 (both feat, no fix)
    let until = Some(base_epoch + day + 86399);
    let result = patterns::run(&repo, None, until, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 2);
    assert!(result.fix_after_feat.is_empty());
}

#[test]
fn cache_separate_keys_for_until() {
    let (_dir, _repo) = create_fixture();
    let key_no_until = cache::cache_key("metrics", None, None, None);
    let key_with_until = cache::cache_key("metrics", None, Some(1736553600), None);
    assert_ne!(key_no_until, key_with_until);
}

#[test]
fn cache_separate_keys_for_since_and_until() {
    let (_dir, _repo) = create_fixture();
    let key_since_only = cache::cache_key("metrics", Some(1736467200), None, None);
    let key_both = cache::cache_key("metrics", Some(1736467200), Some(1736553600), None);
    assert_ne!(key_since_only, key_both);
}

// ---- temporal cluster tests ----

/// Create a fixture with 4 fix commits within 1 hour + 1 feat commit far apart.
/// Used to test temporal cluster detection.
fn create_temporal_cluster_fixture() -> (TempDir, Repository) {
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

    // Commit 1: feat far in the past
    let tree_oid = stage_files(&repo, &[("README.md", "# Temporal Test\n")]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Commits 2-5: 4 fix commits within 1 hour (15 min apart)
    let tree_oid = stage_files(&repo, &[("src/a.rs", "// fix 1\n")]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "fix: first bug");

    let tree_oid = stage_files(&repo, &[("src/b.rs", "// fix 2\n")]);
    let s = make_sig(base_epoch + day + 900); // +15 min
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "fix: second bug");

    let tree_oid = stage_files(&repo, &[("src/c.rs", "// fix 3\n")]);
    let s = make_sig(base_epoch + day + 1800); // +30 min
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "fix: third bug");

    let tree_oid = stage_files(&repo, &[("src/d.rs", "// fix 4\n")]);
    let s = make_sig(base_epoch + day + 2700); // +45 min
    let _c5 = do_commit(&repo, tree_oid, &[c4], &s, "fix: fourth bug");

    (dir, repo)
}

#[test]
fn temporal_cluster_detected() {
    let (_dir, repo) = create_temporal_cluster_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // Should detect a cluster of 4 fix commits within 1 hour
    assert!(!result.temporal_clusters.is_empty());
    let cluster = &result.temporal_clusters[0];
    assert_eq!(cluster.cluster_type, "fix");
    assert_eq!(cluster.commit_count, 4);
    assert_eq!(cluster.commits.len(), 4);
    // All 4 fix commits touch different files
    assert!(cluster.affected_files.len() >= 4);
}

#[test]
fn temporal_cluster_not_triggered_by_spread_commits() {
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();
    // Standard fixture: commits are 1 day apart, no cluster possible
    assert!(result.temporal_clusters.is_empty());
}
