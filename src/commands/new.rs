use std::path::Path;

use anyhow::{Result, bail};

use crate::config;
use crate::context::{Context, execute};
use crate::git;

pub fn run(
    slug: String,
    branch_prefix: Option<String>,
    symlink_files: Option<Vec<String>>,
    dry_run: bool,
) -> Result<()> {
    // Validate slug
    if slug.contains('/') {
        bail!("slug must not contain '/' — got '{slug}'");
    }

    let ctx = Context::new(dry_run)?;
    let cfg = config::load_config(&ctx.root_worktree)?;

    // Merge CLI overrides with config
    let prefix = match branch_prefix {
        Some(p) => Some(p.split('/').map(String::from).collect()),
        None => cfg.branch_prefix,
    };
    let symlinks = symlink_files.or(cfg.symlink_files).unwrap_or_default();

    let branch = config::format_branch_name(&prefix, &slug);
    let main_branch = git::detect_main_branch()?;

    let root_parent = ctx
        .root_worktree
        .parent()
        .expect("root worktree must have a parent directory");
    let wt_path = root_parent.join(format!("{}-{slug}", ctx.repo_name));

    if wt_path.exists() {
        bail!("directory already exists: {}", wt_path.display());
    }

    // Fetch latest from origin
    let fetch_main = main_branch.clone();
    execute(&ctx, &format!("git fetch origin {fetch_main}"), || {
        git::git(&["fetch", "origin", &fetch_main])?;
        Ok(())
    })?;

    // Create worktree
    let start_point = format!("origin/{main_branch}");
    let add_path = wt_path.clone();
    let add_branch = branch.clone();
    let add_start = start_point.clone();
    execute(
        &ctx,
        &format!(
            "git worktree add --no-track -b {add_branch} {} {add_start}",
            add_path.display()
        ),
        move || git::add_worktree(&add_path, &add_branch, &add_start),
    )?;

    // Create symlinks
    for file in &symlinks {
        let source = ctx.root_worktree.join(file);
        let target = wt_path.join(file);
        let src = source.clone();
        let tgt = target.clone();
        let file_name = file.clone();

        execute(
            &ctx,
            &format!("symlink {}", target.display()),
            move || {
                create_symlink(&src, &tgt, &file_name)
            },
        )?;
    }

    if !ctx.dry_run {
        eprintln!("Created worktree '{slug}' on branch {branch}");
        println!("{}", wt_path.display());
    }

    Ok(())
}

fn create_symlink(source: &Path, target: &Path, file_name: &str) -> Result<()> {
    if !source.exists() {
        eprintln!("warning: symlink source does not exist: {file_name}");
        return Ok(());
    }

    // Ensure parent directory of target exists
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;

    #[cfg(windows)]
    {
        if source.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)?;
        } else {
            std::os::windows::fs::symlink_file(source, target)?;
        }
    }

    Ok(())
}
