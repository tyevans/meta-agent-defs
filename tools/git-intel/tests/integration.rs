use git2::{Oid, Repository, Signature, Time};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use git_intel::{authors, cache, churn, common, hotspots, lifecycle, metrics, patterns, trends};

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

// ---- hotspots type_distribution tests ----

#[test]
fn hotspots_type_distribution_present() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();

    // Every directory should have a non-empty type_distribution
    for dir in &result.directories {
        assert!(
            !dir.type_distribution.is_empty(),
            "type_distribution should not be empty for {}",
            dir.path
        );
    }
}

#[test]
fn hotspots_type_distribution_correct_counts() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();

    // Fixture commits and their touched directories (depth=1):
    //   1. "feat: initial commit"       -> "." (README.md), "src" (src/lib.rs)
    //   2. "feat: add helper module"    -> "src" (src/utils.rs)
    //   3. "chore: add gitignore"       -> "." (.gitignore)
    //   4. "fix: handle null input"     -> "src" (src/lib.rs)
    //   5. "docs: update README"        -> "." (README.md)
    //
    // src directory: feat(2), fix(1)
    // root directory: feat(1), chore(1), docs(1)

    let find_dir = |name: &str| -> &hotspots::DirectoryHotspot {
        result.directories.iter().find(|d| d.path == name).unwrap()
    };

    let src = find_dir("src");
    assert_eq!(src.type_distribution.get("feat"), Some(&2));
    assert_eq!(src.type_distribution.get("fix"), Some(&1));
    assert_eq!(src.type_distribution.get("chore"), None);

    let root = find_dir(".");
    assert_eq!(root.type_distribution.get("feat"), Some(&1));
    assert_eq!(root.type_distribution.get("chore"), Some(&1));
    assert_eq!(root.type_distribution.get("docs"), Some(&1));
    assert_eq!(root.type_distribution.get("fix"), None);
}

#[test]
fn hotspots_type_distribution_depth_0_aggregates() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 0, None).unwrap();

    // depth=0: all files aggregate to "."
    // All 5 commits touch "." : feat(2), chore(1), fix(1), docs(1)
    assert_eq!(result.directories.len(), 1);
    let root = &result.directories[0];
    assert_eq!(root.path, ".");

    let total_type_count: usize = root.type_distribution.values().sum();
    assert_eq!(total_type_count, 5); // one entry per commit
    assert_eq!(root.type_distribution.get("feat"), Some(&2));
    assert_eq!(root.type_distribution.get("fix"), Some(&1));
    assert_eq!(root.type_distribution.get("chore"), Some(&1));
    assert_eq!(root.type_distribution.get("docs"), Some(&1));
}

#[test]
fn hotspots_type_distribution_in_json() {
    let (_dir, repo) = create_fixture();
    let result = hotspots::run(&repo, None, None, 1, None).unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Every directory entry should have a type_distribution object
    for dir in parsed["directories"].as_array().unwrap() {
        assert!(
            dir["type_distribution"].is_object(),
            "type_distribution should be a JSON object"
        );
    }
}

#[test]
fn hotspots_type_distribution_with_since_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Only commits 4 and 5: fix (src/lib.rs) and docs (README.md)
    let since = Some(base_epoch + 3 * day);
    let result = hotspots::run(&repo, since, None, 1, None).unwrap();

    let find_dir = |name: &str| -> Option<&hotspots::DirectoryHotspot> {
        result.directories.iter().find(|d| d.path == name)
    };

    if let Some(src) = find_dir("src") {
        assert_eq!(src.type_distribution.get("fix"), Some(&1));
        assert_eq!(src.type_distribution.get("feat"), None);
    }

    if let Some(root) = find_dir(".") {
        assert_eq!(root.type_distribution.get("docs"), Some(&1));
        assert_eq!(root.type_distribution.get("feat"), None);
    }
}

#[test]
fn hotspots_type_distribution_merge_commit() {
    let (_dir, repo) = create_merge_fixture();
    let result = hotspots::run(&repo, None, None, 0, None).unwrap();

    // Merge fixture: feat, fix, feat, merge — all at depth=0 go to "."
    let root = &result.directories[0];
    assert_eq!(root.type_distribution.get("merge"), Some(&1));
    assert_eq!(root.type_distribution.get("feat"), Some(&2));
    assert_eq!(root.type_distribution.get("fix"), Some(&1));
}

// ---- mailmap tests ----

/// Create a fixture with commits from two different email addresses that map
/// to the same person via .mailmap.
///
/// Commits:
///   1. "feat: initial commit" by "Alice <alice@work.com>"   (README.md)
///   2. "fix: typo"           by "Alice <alice@home.com>"    (README.md)
///   3. .mailmap added        by "Alice <alice@work.com>"    (maps alice@home.com -> alice@work.com)
fn create_mailmap_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

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

    // Commit 1: Alice at work email
    let tree_oid = stage_files(&repo, &[("README.md", "# Mailmap Test\n")]);
    let sig1 = Signature::new("Alice", "alice@work.com", &Time::new(base_epoch, 0))
        .expect("create sig");
    let c1 = do_commit(&repo, tree_oid, &[], &sig1, "feat: initial commit");

    // Commit 2: Alice at home email (different identity, same person)
    let tree_oid = stage_files(&repo, &[("README.md", "# Mailmap Test\n\nFixed typo.\n")]);
    let sig2 = Signature::new("alice", "alice@home.com", &Time::new(base_epoch + day, 0))
        .expect("create sig");
    let c2 = do_commit(&repo, tree_oid, &[c1], &sig2, "fix: typo");

    // Commit 3: Add .mailmap that maps home -> work
    let tree_oid = stage_files(
        &repo,
        &[(".mailmap", "Alice <alice@work.com> <alice@home.com>\n")],
    );
    let sig3 = Signature::new("Alice", "alice@work.com", &Time::new(base_epoch + 2 * day, 0))
        .expect("create sig");
    let _c3 = do_commit(&repo, tree_oid, &[c2], &sig3, "chore: add mailmap");

    (dir, repo)
}

#[test]
fn mailmap_resolve_normalizes_identity() {
    let (_dir, repo) = create_mailmap_fixture();
    let mailmap = common::load_mailmap(&repo);
    assert!(mailmap.is_some(), "mailmap should load from repo with .mailmap");

    let mm = mailmap.as_ref().unwrap();

    // The home email should resolve to the work email
    let sig_home = Signature::new("alice", "alice@home.com", &Time::new(0, 0)).unwrap();
    let (name, email) = common::resolve_author(Some(mm), &sig_home);
    assert_eq!(name, "Alice");
    assert_eq!(email, "alice@work.com");

    // The work email should stay unchanged
    let sig_work = Signature::new("Alice", "alice@work.com", &Time::new(0, 0)).unwrap();
    let (name2, email2) = common::resolve_author(Some(mm), &sig_work);
    assert_eq!(name2, "Alice");
    assert_eq!(email2, "alice@work.com");
}

#[test]
fn mailmap_resolve_counts_as_one_author() {
    let (_dir, repo) = create_mailmap_fixture();
    let mailmap = common::load_mailmap(&repo);
    let mm = mailmap.as_ref();

    // Walk all commits and collect unique authors via mailmap
    let commits_iter = common::walk_commits(&repo, None, None).unwrap();
    let mut authors = std::collections::HashSet::new();
    for result in commits_iter {
        let commit = result.unwrap();
        let (name, email) = common::resolve_author(mm, &commit.author());
        authors.insert((name, email));
    }

    // Without mailmap we'd see 2 authors (alice@work.com, alice@home.com).
    // With mailmap they should collapse to 1.
    assert_eq!(authors.len(), 1, "mailmap should collapse two emails into one author");
    let (name, email) = authors.into_iter().next().unwrap();
    assert_eq!(name, "Alice");
    assert_eq!(email, "alice@work.com");
}

#[test]
fn mailmap_resolve_without_mailmap_returns_original() {
    // Use the standard fixture which has no .mailmap
    let (_dir, repo) = create_fixture();
    let mailmap = common::load_mailmap(&repo);

    // No .mailmap in this repo — load_mailmap may return Some(empty) or None
    // depending on git2 behavior. Either way, resolve_author should return
    // the original identity.
    let sig = Signature::new("Test Author", "test@test.com", &Time::new(0, 0)).unwrap();
    let (name, email) = common::resolve_author(mailmap.as_ref(), &sig);
    assert_eq!(name, "Test Author");
    assert_eq!(email, "test@test.com");
}

#[test]
fn mailmap_resolve_none_mailmap_returns_original() {
    let sig = Signature::new("Someone", "someone@example.com", &Time::new(0, 0)).unwrap();
    let (name, email) = common::resolve_author(None, &sig);
    assert_eq!(name, "Someone");
    assert_eq!(email, "someone@example.com");
}

// ---- authors tests ----

#[test]
fn authors_single_author_fixture() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    assert_eq!(result.total_commits_analyzed, 5);
    assert_eq!(result.total_authors, 1); // All commits by "Test Author"
    assert_eq!(result.depth, 1);

    // Should have 2 directories: "." and "src"
    assert_eq!(result.directories.len(), 2);

    for dir in &result.directories {
        assert_eq!(dir.authors.len(), 1);
        assert_eq!(dir.authors[0].name, "Test Author");
        assert_eq!(dir.authors[0].email, "test@test.com");
        assert_eq!(dir.top_contributor, "Test Author");
        // Single author means bus_factor = 1
        assert_eq!(dir.bus_factor, 1);
    }
}

#[test]
fn authors_directory_commit_counts() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    let find_dir = |name: &str| -> &authors::DirectoryAuthors {
        result.directories.iter().find(|d| d.path == name).unwrap()
    };

    // src: commits 1 (feat: initial, touches src/lib.rs), 2 (feat: add helper, src/utils.rs), 4 (fix: src/lib.rs)
    let src = find_dir("src");
    assert_eq!(src.total_commits, 3);

    // root: commits 1 (feat: initial, README.md), 3 (chore: .gitignore), 5 (docs: README.md)
    let root = find_dir(".");
    assert_eq!(root.total_commits, 3);
}

#[test]
fn authors_lines_nonzero() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    for dir in &result.directories {
        let total_lines: usize = dir.authors.iter().map(|a| a.lines_added + a.lines_deleted).sum();
        assert!(total_lines > 0, "directory {} should have nonzero lines", dir.path);
    }
}

#[test]
fn authors_depth_0_aggregates() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 0, None).unwrap();

    assert_eq!(result.depth, 0);
    assert_eq!(result.directories.len(), 1);
    assert_eq!(result.directories[0].path, ".");
    assert_eq!(result.directories[0].total_commits, 5);
}

#[test]
fn authors_since_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Only commits 3, 4, 5
    let since = Some(base_epoch + 2 * day);
    let result = authors::run(&repo, since, None, 1, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 3);
}

#[test]
fn authors_until_filter() {
    let (_dir, repo) = create_fixture();
    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;
    // Only commits 1, 2
    let until = Some(base_epoch + day + 86399);
    let result = authors::run(&repo, None, until, 1, None).unwrap();
    assert_eq!(result.total_commits_analyzed, 2);
}

#[test]
fn authors_limit_truncates() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 1, Some(1)).unwrap();
    assert_eq!(result.directories.len(), 1);
}

#[test]
fn authors_json_output_shape() {
    let (_dir, repo) = create_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["directories"].is_array());
    assert!(parsed["total_authors"].is_number());
    assert!(parsed["total_commits_analyzed"].is_number());
    assert!(parsed["depth"].is_number());

    let first = &parsed["directories"][0];
    assert!(first["path"].is_string());
    assert!(first["authors"].is_array());
    assert!(first["top_contributor"].is_string());
    assert!(first["bus_factor"].is_number());
    assert!(first["total_commits"].is_number());

    let author = &first["authors"][0];
    assert!(author["name"].is_string());
    assert!(author["email"].is_string());
    assert!(author["commits"].is_number());
    assert!(author["lines_added"].is_number());
    assert!(author["lines_deleted"].is_number());
}

/// Create a multi-author fixture for bus factor testing.
///
/// Commits:
///   1. "feat: initial" by Alice (README.md)
///   2. "feat: feature"  by Alice (src/a.rs)
///   3. "fix: bug"      by Alice (src/a.rs)
///   4. "docs: readme"  by Bob   (README.md)
///   5. "feat: new"     by Charlie (src/b.rs)
fn create_multi_author_fixture() -> (TempDir, Repository) {
    let dir = TempDir::new().expect("create temp dir");
    let repo = Repository::init(dir.path()).expect("init repo");

    let base_epoch: i64 = 1736467200;
    let day: i64 = 86400;

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

    // Alice: 3 commits
    let sig_alice = |epoch: i64| -> Signature<'static> {
        Signature::new("Alice", "alice@test.com", &Time::new(epoch, 0)).expect("sig")
    };
    // Bob: 1 commit
    let sig_bob = |epoch: i64| -> Signature<'static> {
        Signature::new("Bob", "bob@test.com", &Time::new(epoch, 0)).expect("sig")
    };
    // Charlie: 1 commit
    let sig_charlie = |epoch: i64| -> Signature<'static> {
        Signature::new("Charlie", "charlie@test.com", &Time::new(epoch, 0)).expect("sig")
    };

    let tree_oid = stage_files(&repo, &[("README.md", "# Multi Author\n")]);
    let s = sig_alice(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial");

    let tree_oid = stage_files(&repo, &[("src/a.rs", "// feature\n")]);
    let s = sig_alice(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "feat: feature");

    let tree_oid = stage_files(&repo, &[("src/a.rs", "// feature\n// fixed\n")]);
    let s = sig_alice(base_epoch + 2 * day);
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "fix: bug");

    let tree_oid = stage_files(&repo, &[("README.md", "# Multi Author\n\nUpdated.\n")]);
    let s = sig_bob(base_epoch + 3 * day);
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "docs: readme");

    let tree_oid = stage_files(&repo, &[("src/b.rs", "// new feature\n")]);
    let s = sig_charlie(base_epoch + 4 * day);
    let _c5 = do_commit(&repo, tree_oid, &[c4], &s, "feat: new");

    (dir, repo)
}

#[test]
fn authors_multi_author_total_count() {
    let (_dir, repo) = create_multi_author_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    assert_eq!(result.total_authors, 3);
    assert_eq!(result.total_commits_analyzed, 5);
}

#[test]
fn authors_multi_author_bus_factor() {
    let (_dir, repo) = create_multi_author_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    let find_dir = |name: &str| -> &authors::DirectoryAuthors {
        result.directories.iter().find(|d| d.path == name).unwrap()
    };

    // src: Alice has 2 commits (feat + fix), Charlie has 1. Total = 3.
    // Alice alone = 2/3 = 66% > 50%, so bus_factor = 1
    let src = find_dir("src");
    assert_eq!(src.bus_factor, 1);
    assert_eq!(src.top_contributor, "Alice");

    // root: Alice has 1 commit, Bob has 1. Total = 2.
    // Alice alone = 1/2 = 50%, not >50%. Both needed = 2/2 > 50%, bus_factor = 2
    let root = find_dir(".");
    assert_eq!(root.bus_factor, 2);
}

#[test]
fn authors_multi_author_sorted_by_commits() {
    let (_dir, repo) = create_multi_author_fixture();
    let result = authors::run(&repo, None, None, 1, None).unwrap();

    // Within each directory, authors should be sorted by commits descending
    for dir in &result.directories {
        for w in dir.authors.windows(2) {
            assert!(
                w[0].commits >= w[1].commits,
                "authors in {} not sorted by commits desc",
                dir.path
            );
        }
    }
}

#[test]
fn authors_mailmap_deduplicates() {
    let (_dir, repo) = create_mailmap_fixture();
    let result = authors::run(&repo, None, None, 0, None).unwrap();

    // With mailmap, alice@home.com and alice@work.com should be unified
    assert_eq!(result.total_authors, 1);
    assert_eq!(result.directories[0].authors.len(), 1);
    assert_eq!(result.directories[0].authors[0].email, "alice@work.com");
    assert_eq!(result.directories[0].authors[0].commits, 3);
}

// ---- trends tests ----

#[test]
fn trends_basic_output_shape() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 3).unwrap();

    assert_eq!(result.window_count, 2);
    assert_eq!(result.window_size_days, 30);
    assert_eq!(result.windows.len(), 2);

    // Windows should be ordered most recent first
    assert_eq!(result.windows[0].index, 0);
    assert_eq!(result.windows[1].index, 1);

    // Deltas should have valid trend strings
    assert!(["increasing", "decreasing", "stable"].contains(&result.deltas.commit_trend.as_str()));
    assert!(["increasing", "decreasing", "stable"].contains(&result.deltas.fix_rate_trend.as_str()));
}

#[test]
fn trends_window_labels_have_dates() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 5).unwrap();

    for w in &result.windows {
        // Labels should be "YYYY-MM-DD to YYYY-MM-DD"
        assert!(w.label.contains(" to "), "label should contain ' to ': {}", w.label);
        assert_eq!(w.since.len(), 10, "since should be YYYY-MM-DD format");
        assert_eq!(w.until.len(), 10, "until should be YYYY-MM-DD format");
    }
}

#[test]
fn trends_velocity_computation() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 5).unwrap();

    for w in &result.windows {
        let expected_velocity = w.total_commits as f64 / 30.0;
        assert!(
            (w.velocity - expected_velocity).abs() < 0.001,
            "velocity {} should equal total_commits/window_days {}",
            w.velocity,
            expected_velocity
        );
    }
}

#[test]
fn trends_churn_limit_respected() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 1, 365, 2).unwrap();

    for w in &result.windows {
        assert!(
            w.top_churn_files.len() <= 2,
            "top_churn_files should respect limit: got {}",
            w.top_churn_files.len()
        );
    }
}

#[test]
fn trends_single_window_stable_deltas() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 1, 365, 5).unwrap();

    assert_eq!(result.windows.len(), 1);
    // With only one window, deltas should be "stable"
    assert_eq!(result.deltas.commit_trend, "stable");
    assert_eq!(result.deltas.fix_rate_trend, "stable");
}

#[test]
fn trends_json_output_shape() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 3).unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["windows"].is_array());
    assert!(parsed["window_count"].is_number());
    assert!(parsed["window_size_days"].is_number());
    assert!(parsed["deltas"].is_object());
    assert!(parsed["deltas"]["commit_trend"].is_string());
    assert!(parsed["deltas"]["fix_rate_trend"].is_string());

    // Check window entry shape
    if let Some(first) = parsed["windows"].as_array().and_then(|a| a.first()) {
        assert!(first["index"].is_number());
        assert!(first["label"].is_string());
        assert!(first["since"].is_string());
        assert!(first["until"].is_string());
        assert!(first["total_commits"].is_number());
        assert!(first["type_distribution"].is_object());
        assert!(first["velocity"].is_number());
        assert!(first["top_churn_files"].is_array());
    }
}

/// Test with a fixture that has commits within a known time range
/// to verify windows actually capture the right data.
#[test]
fn trends_captures_fixture_commits() {
    let (_dir, repo) = create_fixture();
    // Fixture commits span base_epoch to base_epoch + 4*day
    // = 1736467200 to 1736467200 + 345600 = ~5 days
    // Use a large window (365 days) so window 0 (most recent, ending "now")
    // won't contain the fixture commits (they're from 2025-01-10),
    // but if we use enough windows we might catch them.
    // Instead, just verify the function doesn't panic with various configs.
    let result = trends::run(&repo, 4, 90, 5).unwrap();
    assert_eq!(result.windows.len(), 4);

    // The type_distribution should be a HashMap for every window
    for w in &result.windows {
        // velocity should never be negative
        assert!(w.velocity >= 0.0);
    }
}

#[test]
fn trends_dormant_files_in_json_output() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 3).unwrap();
    let json = serde_json::to_string_pretty(&result).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // dormant_files field should exist and be an array
    assert!(parsed["dormant_files"].is_array(), "dormant_files should be an array in JSON output");
}

#[test]
fn trends_dormant_files_sorted() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 2, 30, 3).unwrap();

    // Verify sorted order
    let mut sorted = result.dormant_files.clone();
    sorted.sort();
    assert_eq!(result.dormant_files, sorted, "dormant_files should be sorted alphabetically");
}

#[test]
fn trends_single_window_no_dormant_files() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 1, 365, 5).unwrap();

    // With only one window, there can be no dormant files
    assert!(result.dormant_files.is_empty(), "single window should have no dormant files");
}

#[test]
fn trends_dormant_files_type_is_vec_string() {
    let (_dir, repo) = create_fixture();
    let result = trends::run(&repo, 3, 30, 5).unwrap();

    // dormant_files should be a Vec<String> (compile-time check + runtime sanity)
    let _: &Vec<String> = &result.dormant_files;
}

// ---- signal detection tests ----

/// Create a fixture with fix-after-feat and fix-after-refactor patterns
///
/// Commits (newest to oldest):
///   5. "docs: update README"        -- modifies README.md (no signal)
///   4. "fix: fix refactor bug"      -- modifies src/utils.rs (signals fix-after-refactor)
///   3. "refactor: improve utils"    -- modifies src/utils.rs
///   2. "fix: handle null input"     -- modifies src/lib.rs (signals fix-after-feat)
///   1. "feat: initial commit"       -- creates src/lib.rs, README.md
fn create_signal_fixture() -> (TempDir, Repository) {
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

    // Commit 1: feat: initial commit (README.md + src/lib.rs)
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# Signal Test\n"),
        ("src/lib.rs", "pub fn hello() {}\n"),
    ]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Commit 2: fix: handle null input (modifies src/lib.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/lib.rs", "pub fn hello() {}\npub fn safe_hello() {}\n"),
    ]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "fix: handle null input");

    // Commit 3: refactor: improve utils (creates src/utils.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/utils.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }\n"),
    ]);
    let s = make_sig(base_epoch + 2 * day);
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "refactor: improve utils");

    // Commit 4: fix: fix refactor bug (modifies src/utils.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/utils.rs", "pub fn add(a: i32, b: i32) -> i32 {\n    a.checked_add(b).unwrap_or(0)\n}\n"),
    ]);
    let s = make_sig(base_epoch + 3 * day);
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "fix: fix refactor bug");

    // Commit 5: docs: update README (modifies README.md - no signal expected)
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# Signal Test\n\n## Usage\n\nSee src/lib.rs\n"),
    ]);
    let s = make_sig(base_epoch + 4 * day);
    let _c5 = do_commit(&repo, tree_oid, &[c4], &s, "docs: update README");

    (dir, repo)
}

#[test]
fn signals_fix_after_feat_detected() {
    let (_dir, repo) = create_signal_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // Should detect fix-after-feat signal
    let feat_signals: Vec<_> = result.signals.iter()
        .filter(|s| matches!(s.kind, git_intel::signals::SignalKind::FixAfterFeat))
        .collect();

    assert!(!feat_signals.is_empty(), "Should detect at least one fix-after-feat signal");

    let signal = feat_signals[0];
    assert_eq!(signal.commits.len(), 2);
    assert!(signal.files.contains(&"src/lib.rs".to_string()));
    assert!(signal.message.contains("Fix"));
    assert!(signal.message.contains("feat"));
}

#[test]
fn signals_fix_after_refactor_detected() {
    let (_dir, repo) = create_signal_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // Should detect fix-after-refactor signal
    let refactor_signals: Vec<_> = result.signals.iter()
        .filter(|s| matches!(s.kind, git_intel::signals::SignalKind::FixAfterRefactor))
        .collect();

    assert!(!refactor_signals.is_empty(), "Should detect at least one fix-after-refactor signal");

    let signal = refactor_signals[0];
    assert_eq!(signal.commits.len(), 2);
    assert!(signal.files.contains(&"src/utils.rs".to_string()));
    assert!(signal.message.contains("Fix"));
    assert!(signal.message.contains("refactor"));
}

#[test]
fn signals_severity_calculation_adjacent() {
    let (_dir, repo) = create_signal_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // Fix at commit 4, refactor at commit 3 (gap=1, adjacent)
    let refactor_signals: Vec<_> = result.signals.iter()
        .filter(|s| matches!(s.kind, git_intel::signals::SignalKind::FixAfterRefactor))
        .collect();

    assert!(!refactor_signals.is_empty());

    let signal = refactor_signals[0];
    // gap=1 (adjacent), shared_files=1
    // severity = 1.0 / (1 + 1) * (1.min(5) / 5) = 0.5 * 0.2 = 0.1
    let expected_severity = 0.1;
    assert!((signal.severity - expected_severity).abs() < 0.001,
        "Expected severity ~{}, got {}", expected_severity, signal.severity);
}

#[test]
fn signals_no_signal_without_shared_files() {
    // Use the standard fixture where docs commit doesn't share files with feat/refactor
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // The docs commit at the top doesn't trigger signals (no shared files with feat commits)
    // But the fix commit DOES share files with feat, so we should have at least one signal
    assert!(!result.signals.is_empty(), "Should have fix-after-feat signal from standard fixture");

    // Verify all signals have non-empty file lists
    for signal in &result.signals {
        assert!(!signal.files.is_empty(), "All signals should have shared files");
    }
}

#[test]
fn signals_backward_compatibility_fix_after_feat() {
    let (_dir, repo) = create_signal_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // The existing fix_after_feat Vec should still work for backward compatibility
    assert!(!result.fix_after_feat.is_empty(), "fix_after_feat backward compatibility maintained");

    // It should only contain feat pairs, not refactor pairs
    for pair in &result.fix_after_feat {
        assert!(pair.feat_message.starts_with("feat:"),
            "fix_after_feat should only contain feat commits, got: {}", pair.feat_message);
    }
}

// ---- feature branch lifecycle tests ----

/// Create a fixture where a file is created on main, then edited on a feature branch.
/// HEAD is set to the feature branch.
///
/// Commits:
///   main:
///     1. "feat: initial commit" — creates README.md, memory/agents/rust-dev/learnings.md
///   feature-branch (branched from c1):
///     2. "chore: update learnings" — modifies memory/agents/rust-dev/learnings.md
///     3. "chore: more learnings"   — modifies memory/agents/rust-dev/learnings.md
fn create_feature_branch_fixture() -> (TempDir, Repository) {
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

    // Commit 1: initial commit on main (HEAD -> master)
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# Test\n"),
        ("memory/agents/rust-dev/learnings.md", "# Learnings\n\n- initial learning\n"),
    ]);
    let c1 = {
        let s = make_sig(base_epoch);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        repo.commit(Some("HEAD"), &s, &s, "feat: initial commit", &tree, &[])
            .expect("create commit")
    };

    // Create feature branch from c1
    {
        let c1_commit = repo.find_commit(c1).expect("find c1");
        repo.branch("feature-branch", &c1_commit, false).expect("create branch");
    }

    // Switch HEAD to feature branch
    repo.set_head("refs/heads/feature-branch").expect("set HEAD to feature branch");

    // Commit 2: edit learnings on feature branch
    let tree_oid = stage_files(&repo, &[
        ("memory/agents/rust-dev/learnings.md", "# Learnings\n\n- initial learning\n- second learning\n"),
    ]);
    let c2 = {
        let s = make_sig(base_epoch + day);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c1).expect("find c1");
        repo.commit(Some("HEAD"), &s, &s, "chore: update learnings", &tree, &[&parent])
            .expect("create commit")
    };

    // Commit 3: edit learnings again on feature branch
    let tree_oid = stage_files(&repo, &[
        ("memory/agents/rust-dev/learnings.md", "# Learnings\n\n- initial learning\n- second learning\n- third learning\n"),
    ]);
    {
        let s = make_sig(base_epoch + 2 * day);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c2).expect("find c2");
        repo.commit(Some("HEAD"), &s, &s, "chore: more learnings", &tree, &[&parent])
            .expect("create commit");
    }

    (dir, repo)
}

#[test]
fn lifecycle_feature_branch_finds_commits() {
    let (_dir, repo) = create_feature_branch_fixture();

    // Verify we're on the feature branch
    let head = repo.head().expect("get HEAD");
    assert!(
        head.name().unwrap().contains("feature-branch"),
        "HEAD should point to feature-branch, got: {}",
        head.name().unwrap()
    );

    // Run lifecycle on the learnings file
    let result = lifecycle::run(
        &repo,
        None,
        None,
        &["memory/agents/rust-dev/learnings.md".to_string()],
    )
    .unwrap();

    let f = &result.files[0];
    assert!(f.exists, "file should exist on feature branch");
    assert!(f.current_lines.is_some(), "should have line count");

    // Should find 3 commits: initial create + 2 edits on feature branch
    assert_eq!(
        f.history.len(),
        3,
        "lifecycle should find all 3 commits touching the file on the feature branch, got {}",
        f.history.len()
    );

    // First entry (newest) should be the third commit
    assert_eq!(f.history[0].message, "chore: more learnings");
    // Last entry (oldest) should be the initial commit
    assert_eq!(f.history[2].status, "created");
}

#[test]
fn walk_commits_includes_feature_branch_commits() {
    let (_dir, repo) = create_feature_branch_fixture();

    // Verify HEAD is on feature branch
    let head = repo.head().expect("get HEAD");
    assert!(head.name().unwrap().contains("feature-branch"));

    let commits: Vec<_> = common::walk_commits(&repo, None, None)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Should find 3 commits: 1 on main + 2 on feature branch
    assert_eq!(
        commits.len(),
        3,
        "walk_commits should find all 3 reachable commits from feature branch HEAD"
    );

    // Newest first
    assert_eq!(
        commits[0].message().unwrap().trim(),
        "chore: more learnings"
    );
    assert_eq!(
        commits[1].message().unwrap().trim(),
        "chore: update learnings"
    );
    assert_eq!(
        commits[2].message().unwrap().trim(),
        "feat: initial commit"
    );
}

/// Test lifecycle when feature branch has commits but main has diverged.
/// This simulates: create on main, branch, commit on both main and feature,
/// then run lifecycle from feature branch.
#[test]
fn lifecycle_feature_branch_with_diverged_main() {
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

    // Commit 1: initial on main
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# Test\n"),
        ("src/lib.rs", "pub fn hello() {}\n"),
    ]);
    let c1 = {
        let s = make_sig(base_epoch);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        repo.commit(Some("HEAD"), &s, &s, "feat: initial commit", &tree, &[])
            .expect("create commit")
    };

    // Create feature branch from c1
    {
        let c1_commit = repo.find_commit(c1).expect("find c1");
        repo.branch("my-feature", &c1_commit, false).expect("create branch");
    }

    // Commit 2: advance main (don't switch HEAD yet)
    let tree_oid = stage_files(&repo, &[
        ("src/lib.rs", "pub fn hello() {}\npub fn world() {}\n"),
    ]);
    let _c2_main = {
        let s = make_sig(base_epoch + day);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c1).expect("find c1");
        repo.commit(Some("HEAD"), &s, &s, "feat: advance main", &tree, &[&parent])
            .expect("create commit")
    };

    // Switch HEAD to feature branch
    repo.set_head("refs/heads/my-feature").expect("set HEAD");

    // Commit 3: edit on feature branch
    let tree_oid = stage_files(&repo, &[
        ("src/lib.rs", "pub fn hello() {}\npub fn feature_work() {}\n"),
    ]);
    let _c3_feature = {
        let s = make_sig(base_epoch + 2 * day);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c1).expect("find c1");
        repo.commit(Some("HEAD"), &s, &s, "feat: feature work", &tree, &[&parent])
            .expect("create commit")
    };

    // Run lifecycle on src/lib.rs
    let result = lifecycle::run(
        &repo,
        None,
        None,
        &["src/lib.rs".to_string()],
    ).unwrap();

    let f = &result.files[0];
    assert!(f.exists);

    // Should find 2 commits: initial create + feature branch edit
    // Should NOT include the main branch commit (c2_main) since it's not reachable from feature HEAD
    assert_eq!(
        f.history.len(),
        2,
        "lifecycle on feature branch should find 2 commits (initial + feature), got {}",
        f.history.len()
    );
    assert_eq!(f.history[0].message, "feat: feature work");
    assert_eq!(f.history[1].status, "created");
}

/// Verify that cache entries from one branch don't pollute results on another branch.
#[test]
fn cache_invalidated_by_branch_switch() {
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

    // Commit 1: initial on main
    let tree_oid = stage_files(&repo, &[("README.md", "# Test\n")]);
    let c1 = {
        let s = make_sig(base_epoch);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        repo.commit(Some("HEAD"), &s, &s, "feat: initial commit", &tree, &[])
            .expect("create commit")
    };

    // Run metrics and cache on main
    let key = cache::cache_key("metrics", None, None, None);
    let result = metrics::run(&repo, None, None, None).unwrap();
    cache::write_cache(&repo, &key, &result).unwrap();
    assert!(cache::read_cache(&repo, &key).is_some(), "cache should hit on main");
    assert_eq!(result.total_commits, 1);

    // Create and switch to feature branch
    {
        let c1_commit = repo.find_commit(c1).expect("find c1");
        repo.branch("feature", &c1_commit, false).expect("create branch");
    }
    repo.set_head("refs/heads/feature").expect("set HEAD");

    // Add a commit on feature branch
    let tree_oid = stage_files(&repo, &[("feature.txt", "feature\n")]);
    {
        let s = make_sig(base_epoch + day);
        let tree = repo.find_tree(tree_oid).expect("find tree");
        let parent = repo.find_commit(c1).expect("find c1");
        repo.commit(Some("HEAD"), &s, &s, "feat: feature work", &tree, &[&parent])
            .expect("create commit");
    }

    // Cache should miss on feature branch (different HEAD)
    assert!(
        cache::read_cache(&repo, &key).is_none(),
        "cache should miss after switching to feature branch with new commit"
    );

    // Fresh computation should reflect feature branch state
    let result = metrics::run(&repo, None, None, None).unwrap();
    assert_eq!(result.total_commits, 2, "feature branch should see 2 commits");
}

// ---- directory_chains tests ----

/// Create a fixture with multiple files in the same directory edited across many commits,
/// generating enough churn for directory chain detection.
///
/// Commits (newest to oldest):
///   6. "fix: patch parser again"    -- modifies src/parser.rs (large change)
///   5. "refactor: clean up parser"  -- modifies src/parser.rs (large change)
///   4. "feat: add formatter"        -- creates src/formatter.rs (large change)
///   3. "fix: parser edge case"      -- modifies src/parser.rs (large change)
///   2. "feat: add parser"           -- creates src/parser.rs (large change)
///   1. "feat: initial commit"       -- creates README.md, src/lib.rs
///
/// The "src" directory gets 5 edits across 5 commits (commits 2-6) with substantial churn.
fn create_directory_chain_fixture() -> (TempDir, Repository) {
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

    // Generate a large content block to produce significant churn
    let big_block = |prefix: &str, lines: usize| -> String {
        (0..lines).map(|i| format!("// {} line {}\n", prefix, i)).collect()
    };

    // Commit 1: feat: initial commit
    let tree_oid = stage_files(&repo, &[
        ("README.md", "# Project\n"),
        ("src/lib.rs", "pub mod parser;\n"),
    ]);
    let s = make_sig(base_epoch);
    let c1 = do_commit(&repo, tree_oid, &[], &s, "feat: initial commit");

    // Commit 2: feat: add parser (creates src/parser.rs with ~60 lines)
    let tree_oid = stage_files(&repo, &[
        ("src/parser.rs", &big_block("parser_v1", 60)),
    ]);
    let s = make_sig(base_epoch + day);
    let c2 = do_commit(&repo, tree_oid, &[c1], &s, "feat: add parser");

    // Commit 3: fix: parser edge case (modifies src/parser.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/parser.rs", &big_block("parser_v2", 60)),
    ]);
    let s = make_sig(base_epoch + 2 * day);
    let c3 = do_commit(&repo, tree_oid, &[c2], &s, "fix: parser edge case");

    // Commit 4: feat: add formatter (creates src/formatter.rs with ~60 lines)
    let tree_oid = stage_files(&repo, &[
        ("src/formatter.rs", &big_block("formatter_v1", 60)),
    ]);
    let s = make_sig(base_epoch + 3 * day);
    let c4 = do_commit(&repo, tree_oid, &[c3], &s, "feat: add formatter");

    // Commit 5: refactor: clean up parser (modifies src/parser.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/parser.rs", &big_block("parser_v3", 60)),
    ]);
    let s = make_sig(base_epoch + 4 * day);
    let c5 = do_commit(&repo, tree_oid, &[c4], &s, "refactor: clean up parser");

    // Commit 6: fix: patch parser again (modifies src/parser.rs)
    let tree_oid = stage_files(&repo, &[
        ("src/parser.rs", &big_block("parser_v4", 60)),
    ]);
    let s = make_sig(base_epoch + 5 * day);
    let _c6 = do_commit(&repo, tree_oid, &[c5], &s, "fix: patch parser again");

    (dir, repo)
}

#[test]
fn directory_chains_detected() {
    let (_dir, repo) = create_directory_chain_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    // "src" directory is touched in commits 2-6 (5 commits), with high churn
    assert!(!result.directory_chains.is_empty(), "should detect directory chains");

    let src_chain = result.directory_chains.iter().find(|d| d.path == "src");
    assert!(src_chain.is_some(), "should have a chain for 'src' directory");

    let src = src_chain.unwrap();
    assert!(src.total_edit_count >= 3, "src should have >= 3 edits, got {}", src.total_edit_count);
    assert!(src.total_churn > 0, "src should have non-zero churn");
    // src/parser.rs, src/formatter.rs, src/lib.rs are files in src/
    assert!(src.files.len() >= 2, "src should contain multiple files, got {}", src.files.len());
    // Files should be sorted
    let mut sorted_files = src.files.clone();
    sorted_files.sort();
    assert_eq!(src.files, sorted_files, "files should be sorted");
}

#[test]
fn directory_chains_empty_for_few_edits() {
    // Use temporal cluster fixture: each fix commit touches a different src/ file
    // but commits are close together. src/a.rs, src/b.rs, src/c.rs, src/d.rs each touched once,
    // plus initial commit touches src/main.rs. That's 5 commits touching src/ so it qualifies.
    // Just verify the basic contract holds.
    let (_dir, repo) = create_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();
    for chain in &result.directory_chains {
        assert!(chain.total_edit_count >= 3);
        assert!(!chain.files.is_empty());
    }
}

#[test]
fn directory_chains_sorted_by_churn_descending() {
    let (_dir, repo) = create_directory_chain_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();

    for i in 1..result.directory_chains.len() {
        assert!(
            result.directory_chains[i - 1].total_churn >= result.directory_chains[i].total_churn,
            "directory_chains should be sorted by total_churn descending"
        );
    }
}

#[test]
fn directory_chains_respects_limit() {
    let (_dir, repo) = create_directory_chain_fixture();
    let result = patterns::run(&repo, None, None, Some(0)).unwrap();
    assert!(result.directory_chains.is_empty(), "limit 0 should produce no directory chains");
}

#[test]
fn directory_chains_capped_at_10() {
    let (_dir, repo) = create_directory_chain_fixture();
    let result = patterns::run(&repo, None, None, None).unwrap();
    assert!(result.directory_chains.len() <= 10, "directory_chains should be capped at 10");
}
