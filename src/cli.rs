use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wt", about = "Git worktree manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new worktree
    New {
        /// Slug for the worktree (used in directory name and branch)
        slug: String,

        /// Branch prefix segments (e.g. "chnn" or "team/chnn"), overrides config
        #[arg(long)]
        branch_prefix: Option<String>,

        /// Files to symlink from root worktree (repeatable), overrides config
        #[arg(long = "symlink-file")]
        symlink_files: Option<Vec<String>>,

        /// Print what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Open $EDITOR to author a prompt, then run claude in the new worktree
        #[arg(short = 'p', long)]
        prompt: bool,

        /// Pass --dangerously-skip-permissions to claude (requires -p)
        #[arg(long)]
        dangerously_skip_permissions: bool,
    },

    /// Print shell function for sourcing in .zshrc/.bashrc
    ShellInit,

    /// List worktree slugs
    #[command(alias = "ls")]
    List {
        /// Print what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Remove one or more worktrees
    #[command(alias = "rm")]
    Remove {
        /// Slugs to remove
        slugs: Vec<String>,

        /// Print what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Initialize a .wtconfig.toml in the root worktree
    Init {
        /// Print what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },
}
