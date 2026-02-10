use std::path::PathBuf;

use anyhow::Result;

pub struct Context {
    pub dry_run: bool,
    pub root_worktree: PathBuf,
    pub repo_name: String,
}

impl Context {
    pub fn new(dry_run: bool) -> Result<Self> {
        let root_worktree = crate::git::root_worktree_path()?;
        let repo_name = crate::git::repo_name(&root_worktree);
        Ok(Self {
            dry_run,
            root_worktree,
            repo_name,
        })
    }
}

pub fn execute<F>(ctx: &Context, description: &str, f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if ctx.dry_run {
        println!("[dry-run] {description}");
        Ok(())
    } else {
        f()
    }
}
