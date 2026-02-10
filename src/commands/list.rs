use anyhow::Result;

use crate::context::Context;
use crate::git;

pub fn run(dry_run: bool) -> Result<()> {
    let ctx = Context::new(dry_run)?;
    let worktrees = git::list_worktrees()?;
    let prefix = format!("{}-", ctx.repo_name);

    for wt in &worktrees {
        let dir_name = match wt.path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => continue,
        };

        if let Some(slug) = dir_name.strip_prefix(&prefix) {
            println!("{slug}");
        }
    }

    Ok(())
}
