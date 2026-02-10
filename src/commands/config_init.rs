use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::config;
use crate::context::{Context, execute};

pub fn run(dry_run: bool) -> Result<()> {
    let ctx = Context::new(dry_run)?;
    let config_path = ctx.root_worktree.join(".wtconfig.toml");

    if config_path.exists() {
        bail!(
            ".wtconfig.toml already exists at {}",
            config_path.display()
        );
    }

    execute(&ctx, "write .wtconfig.toml", || {
        fs::write(&config_path, config::config_template())?;
        Ok(())
    })?;

    let exclude_path = ctx.root_worktree.join(".git/info/exclude");
    execute(&ctx, "add .wtconfig.toml to .git/info/exclude", || {
        add_to_exclude(&exclude_path)
    })?;

    println!("Initialized .wtconfig.toml in {}", ctx.root_worktree.display());
    Ok(())
}

fn add_to_exclude(exclude_path: &PathBuf) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = exclude_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check if already present
    if exclude_path.exists() {
        let contents = fs::read_to_string(exclude_path)?;
        if contents.lines().any(|l| l.trim() == ".wtconfig.toml") {
            return Ok(());
        }
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(exclude_path)?;
    writeln!(file, ".wtconfig.toml")?;
    Ok(())
}
