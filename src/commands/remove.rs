use anyhow::{Result, bail};

use crate::context::{Context, execute_with_progress};
use crate::git;

pub fn run(slugs: Vec<String>, dry_run: bool) -> Result<()> {
    let ctx = Context::new(dry_run)?;
    let worktrees = git::list_worktrees()?;

    let root_parent = ctx
        .root_worktree
        .parent()
        .expect("root worktree must have a parent directory");

    let mut errors: Vec<String> = Vec::new();

    for slug in &slugs {
        if let Err(e) = remove_one(&ctx, &worktrees, root_parent, slug) {
            errors.push(format!("{slug}: {e}"));
        }
    }

    if !errors.is_empty() {
        let msg = errors.join("\n  ");
        bail!("some worktrees could not be removed:\n  {msg}");
    }

    Ok(())
}

fn remove_one(
    ctx: &Context,
    worktrees: &[git::WorktreeInfo],
    root_parent: &std::path::Path,
    slug: &str,
) -> Result<()> {
    let expected_path = root_parent.join(format!("{}-{slug}", ctx.repo_name));

    let wt = worktrees
        .iter()
        .find(|w| w.path == expected_path)
        .ok_or_else(|| anyhow::anyhow!("no worktree found at {}", expected_path.display()))?;

    let branch = wt.branch.clone();

    let path = expected_path.clone();
    execute_with_progress(
        ctx,
        &format!("git worktree remove {}", path.display()),
        move || git::remove_worktree(&path),
    )?;

    if let Some(branch) = branch {
        let b = branch.clone();
        execute_with_progress(ctx, &format!("git branch -D {b}"), move || {
            git::delete_branch(&branch)
        })?;
    }

    if !ctx.dry_run {
        println!("Removed worktree '{slug}'");
    }

    Ok(())
}
