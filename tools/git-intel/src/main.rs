use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use git_intel::{churn, lifecycle, metrics, parse_since, patterns};

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
            let result = metrics::run(&repo, since_epoch, cli.limit)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Churn => {
            let result = churn::run(&repo, since_epoch, cli.limit)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Lifecycle { files } => {
            let result = lifecycle::run(&repo, since_epoch, &files)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::Patterns { convergence_limit } => {
            let result = patterns::run_with_convergence_limit(&repo, since_epoch, cli.limit, convergence_limit)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}
