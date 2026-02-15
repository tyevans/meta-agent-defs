use anyhow::Result;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    head_commit: String,
    computed_at: String,
    result: T,
}

/// Return the cache directory: `.git/git-intel-cache/`
fn cache_dir(repo: &Repository) -> Result<PathBuf> {
    let git_dir = repo.path(); // .git/
    Ok(git_dir.join("git-intel-cache"))
}

/// Get the current HEAD commit hash as a hex string.
fn head_oid(repo: &Repository) -> Result<String> {
    let head = repo
        .head()
        .map_err(|e| anyhow::anyhow!("Cannot read HEAD: {}", e))?;
    let oid = head
        .target()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not a direct reference"))?;
    Ok(oid.to_string())
}

/// Build cache key filename from subcommand name and args.
///
/// - metrics/churn/patterns: `{subcommand}-{since_label}-{until_label}.json`
/// - lifecycle: `lifecycle-{hash(files)}-{since_label}-{until_label}.json`
pub fn cache_key(subcommand: &str, since: Option<i64>, until: Option<i64>, files: Option<&[String]>) -> String {
    let since_label = match since {
        Some(ts) => format!("{}", ts),
        None => "all".to_string(),
    };
    let until_label = match until {
        Some(ts) => format!("{}", ts),
        None => "all".to_string(),
    };

    match files {
        Some(f) => {
            let mut hasher = DefaultHasher::new();
            let mut sorted: Vec<&String> = f.iter().collect();
            sorted.sort();
            for file in sorted {
                file.hash(&mut hasher);
            }
            let h = hasher.finish();
            format!("{}-{:x}-{}-{}.json", subcommand, h, since_label, until_label)
        }
        None => format!("{}-{}-{}.json", subcommand, since_label, until_label),
    }
}

/// Try to read a cached result. Returns `Some(json_string)` on cache hit,
/// `None` on miss (file missing, HEAD mismatch, or parse error).
pub fn read_cache(repo: &Repository, key: &str) -> Option<String> {
    let dir = cache_dir(repo).ok()?;
    let path = dir.join(key);
    let data = fs::read_to_string(&path).ok()?;

    // Parse just enough to check head_commit
    let entry: serde_json::Value = serde_json::from_str(&data).ok()?;
    let cached_head = entry.get("head_commit")?.as_str()?;
    let current_head = head_oid(repo).ok()?;

    if cached_head == current_head {
        // Extract just the result field and pretty-print it
        let result = entry.get("result")?;
        serde_json::to_string_pretty(result).ok()
    } else {
        None
    }
}

/// Write a computed result to cache.
pub fn write_cache<T: Serialize>(repo: &Repository, key: &str, result: &T) -> Result<()> {
    let dir = cache_dir(repo)?;
    fs::create_dir_all(&dir)?;

    let current_head = head_oid(repo)?;
    let now = chrono::Utc::now().to_rfc3339();

    let entry = CacheEntry {
        head_commit: current_head,
        computed_at: now,
        result,
    };

    let json = serde_json::to_string_pretty(&entry)?;
    let path = dir.join(key);
    fs::write(&path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_metrics_all() {
        assert_eq!(cache_key("metrics", None, None, None), "metrics-all-all.json");
    }

    #[test]
    fn cache_key_metrics_since() {
        let key = cache_key("metrics", Some(1768435200), None, None);
        assert_eq!(key, "metrics-1768435200-all.json");
    }

    #[test]
    fn cache_key_metrics_until() {
        let key = cache_key("metrics", None, Some(1768521599), None);
        assert_eq!(key, "metrics-all-1768521599.json");
    }

    #[test]
    fn cache_key_metrics_since_until() {
        let key = cache_key("metrics", Some(1768435200), Some(1768521599), None);
        assert_eq!(key, "metrics-1768435200-1768521599.json");
    }

    #[test]
    fn cache_key_lifecycle_files() {
        let files = vec!["src/main.rs".to_string(), "Cargo.toml".to_string()];
        let key = cache_key("lifecycle", None, None, Some(&files));
        assert!(key.starts_with("lifecycle-"));
        assert!(key.ends_with("-all-all.json"));
    }

    #[test]
    fn cache_key_lifecycle_order_independent() {
        let files_a = vec!["a.rs".to_string(), "b.rs".to_string()];
        let files_b = vec!["b.rs".to_string(), "a.rs".to_string()];
        assert_eq!(
            cache_key("lifecycle", None, None, Some(&files_a)),
            cache_key("lifecycle", None, None, Some(&files_b))
        );
    }
}
