use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use git_intel::{authors, cache, churn, hotspots, lifecycle, metrics, parse_since, parse_until, patterns, trends, validate_range};

#[derive(Parser)]
#[command(name = "git-intel", about = "Git history analyzer — JSON output for hooks and skills")]
struct Cli {
    /// Path to the git repository (default: current directory)
    #[arg(long, default_value = ".", global = true)]
    repo: PathBuf,

    /// Limit history to commits after this date (YYYY-MM-DD or relative: 30d, 4w, 6m, 1y)
    #[arg(long, global = true)]
    since: Option<String>,

    /// Limit history to commits before this date (YYYY-MM-DD or relative: 30d, 4w, 6m, 1y, inclusive)
    #[arg(long, global = true)]
    until: Option<String>,

    /// Maximum number of output items (for churn, metrics)
    #[arg(long, global = true)]
    limit: Option<usize>,

    /// Bypass cache and always recompute
    #[arg(long, global = true)]
    no_cache: bool,

    /// Enable ML-based commit classification (requires --model-dir)
    #[cfg(feature = "ml")]
    #[arg(long, global = true)]
    ml: bool,

    /// Path to the ONNX model directory (must contain model.onnx, tokenizer.json, label_mapping.json)
    #[cfg(feature = "ml")]
    #[arg(long, global = true)]
    model_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Commit type distribution, activity bursts, velocity
    Metrics,
    /// File volatility ranking by lines added+removed
    Churn,
    /// Track specific files across commits (growth/shrinkage/survival)
    Lifecycle {
        /// File paths to track
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// Directory-level churn aggregation (group file churn by path prefix)
    Hotspots {
        /// Directory depth for grouping (1 = top-level dirs, 2 = two levels, etc.)
        #[arg(long, default_value_t = 1)]
        depth: usize,
    },
    /// Detect fix-after-feat sequences, multi-edit chains, temporal clusters
    Patterns,
    /// Per-directory ownership analysis: top contributor, bus factor, author stats
    Authors {
        /// Directory depth for grouping (1 = top-level dirs, 2 = two levels, etc.)
        #[arg(long, default_value_t = 1)]
        depth: usize,
    },
    /// Multi-window temporal comparison: how metrics change over time
    Trends {
        /// Number of time windows to compare (default: 4)
        #[arg(long, default_value_t = 4)]
        windows: usize,
        /// Size of each window in days (default: 90)
        #[arg(long, default_value_t = 90)]
        window_size: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let since_epoch = parse_since(&cli.since)?;
    let until_epoch = parse_until(&cli.until)?;
    validate_range(since_epoch, until_epoch)?;

    let repo = git2::Repository::discover(&cli.repo)
        .map_err(|e| anyhow::anyhow!("Could not open repo at {:?}: {}", cli.repo, e))?;

    // Load ML classifier if requested
    #[cfg(feature = "ml")]
    let mut ml_classifier = if cli.ml {
        let model_dir = cli.model_dir.as_ref().ok_or_else(|| {
            anyhow::anyhow!("--ml requires --model-dir <path> pointing to a directory with model.onnx, tokenizer.json, and label_mapping.json")
        })?;
        Some(git_intel::ml::MlClassifier::load(model_dir)?)
    } else {
        None
    };

    #[cfg(feature = "ml")]
    let ml_ref = ml_classifier.as_mut();

    match cli.command {
        Commands::Metrics => {
            let key = cache::cache_key("metrics", since_epoch, until_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            #[cfg(feature = "ml")]
            let result = metrics::run_with_ml(&repo, since_epoch, until_epoch, cli.limit, ml_ref)?;
            #[cfg(not(feature = "ml"))]
            let result = metrics::run(&repo, since_epoch, until_epoch, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Churn => {
            let key = cache::cache_key("churn", since_epoch, until_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = churn::run(&repo, since_epoch, until_epoch, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Lifecycle { files } => {
            let key = cache::cache_key("lifecycle", since_epoch, until_epoch, Some(&files));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = lifecycle::run(&repo, since_epoch, until_epoch, &files)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Hotspots { depth } => {
            let extra = &[format!("{}", depth)];
            let key = cache::cache_key("hotspots", since_epoch, until_epoch, Some(extra));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = hotspots::run(&repo, since_epoch, until_epoch, depth, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Authors { depth } => {
            let extra = &[format!("{}", depth)];
            let key = cache::cache_key("authors", since_epoch, until_epoch, Some(extra));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = authors::run(&repo, since_epoch, until_epoch, depth, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Trends { windows, window_size } => {
            let churn_limit = cli.limit.unwrap_or(5);
            let extra = &[format!("{}", windows), format!("{}", window_size), format!("{}", churn_limit)];
            let key = cache::cache_key("trends", since_epoch, until_epoch, Some(extra));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = trends::run(&repo, windows, window_size, churn_limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Patterns => {
            let key = cache::cache_key("patterns", since_epoch, until_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            #[cfg(feature = "ml")]
            let result = patterns::run_with_ml(&repo, since_epoch, until_epoch, cli.limit, ml_ref)?;
            #[cfg(not(feature = "ml"))]
            let result = patterns::run(&repo, since_epoch, until_epoch, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
