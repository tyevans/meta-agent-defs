use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use git_intel::{cache, churn, hotspots, lifecycle, metrics, parse_since, patterns};

#[derive(Parser)]
#[command(name = "git-intel", about = "Git history analyzer — JSON output for hooks and skills")]
struct Cli {
    /// Path to the git repository (default: current directory)
    #[arg(long, default_value = ".", global = true)]
    repo: PathBuf,

    /// Limit history to commits after this date (YYYY-MM-DD)
    #[arg(long, global = true)]
    since: Option<String>,

    /// Maximum number of output items (for churn, metrics)
    #[arg(long, global = true)]
    limit: Option<usize>,

    /// Bypass cache and always recompute
    #[arg(long, global = true)]
    no_cache: bool,

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
    /// Detect fix-after-feat sequences, multi-edit chains, convergence
    Patterns {
        /// Maximum number of convergence pairs to output (default: 50)
        #[arg(long, default_value_t = patterns::DEFAULT_CONVERGENCE_LIMIT)]
        convergence_limit: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let since_epoch = parse_since(&cli.since)?;

    let repo = git2::Repository::discover(&cli.repo)
        .map_err(|e| anyhow::anyhow!("Could not open repo at {:?}: {}", cli.repo, e))?;

    match cli.command {
        Commands::Metrics => {
            let key = cache::cache_key("metrics", since_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = metrics::run(&repo, since_epoch, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Churn => {
            let key = cache::cache_key("churn", since_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = churn::run(&repo, since_epoch, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Lifecycle { files } => {
            let key = cache::cache_key("lifecycle", since_epoch, Some(&files));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = lifecycle::run(&repo, since_epoch, &files)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Hotspots { depth } => {
            let extra = &[format!("{}", depth)];
            let key = cache::cache_key("hotspots", since_epoch, Some(extra));
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = hotspots::run(&repo, since_epoch, depth, cli.limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Patterns { convergence_limit } => {
            let key = cache::cache_key("patterns", since_epoch, None);
            if !cli.no_cache {
                if let Some(cached) = cache::read_cache(&repo, &key) {
                    println!("{}", cached);
                    return Ok(());
                }
            }
            let result = patterns::run_with_convergence_limit(&repo, since_epoch, cli.limit, convergence_limit)?;
            let _ = cache::write_cache(&repo, &key, &result);
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
