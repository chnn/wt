mod cli;
mod commands;
mod config;
mod context;
mod git;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands, ConfigAction};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            slug,
            branch_prefix,
            symlink_files,
            dry_run,
        } => commands::new::run(slug, branch_prefix, symlink_files, dry_run),

        Commands::List { dry_run } => commands::list::run(dry_run),

        Commands::Remove { slugs, dry_run } => commands::remove::run(slugs, dry_run),

        Commands::Config { action } => match action {
            ConfigAction::Init { dry_run } => commands::config_init::run(dry_run),
        },
    }
}
