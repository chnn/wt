use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub is_bare: bool,
}

pub fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn root_worktree_path() -> Result<PathBuf> {
    let output = git(&["worktree", "list", "--porcelain"])?;
    let entries = parse_worktree_list(&output);
    entries
        .into_iter()
        .find(|e| !e.is_bare)
        .map(|e| e.path)
        .or_else(|| {
            // Fallback: use git rev-parse --show-toplevel
            git(&["rev-parse", "--show-toplevel"])
                .ok()
                .map(PathBuf::from)
        })
        .context("could not determine root worktree path")
}

pub fn repo_name(root: &Path) -> String {
    root.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn detect_main_branch() -> Result<String> {
    // Try symbolic-ref first
    if let Ok(refname) = git(&["symbolic-ref", "refs/remotes/origin/HEAD"]) {
        if let Some(branch) = refname.strip_prefix("refs/remotes/origin/") {
            return Ok(branch.to_string());
        }
    }

    // Fallback: check common branch names
    for name in &["main", "master", "trunk"] {
        if git(&["rev-parse", "--verify", &format!("origin/{name}")]).is_ok() {
            return Ok(name.to_string());
        }
    }

    bail!(
        "could not detect main branch: no origin/HEAD and none of main/master/trunk found at origin"
    )
}

pub fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = git(&["worktree", "list", "--porcelain"])?;
    Ok(parse_worktree_list(&output))
}

pub fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    let mut entries = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;

    for line in output.lines() {
        if let Some(path_str) = line.strip_prefix("worktree ") {
            // Save previous entry if exists
            if let Some(path) = current_path.take() {
                entries.push(WorktreeInfo {
                    path,
                    branch: current_branch.take(),
                    is_bare,
                });
            }
            current_path = Some(PathBuf::from(path_str));
            current_branch = None;
            is_bare = false;
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            current_branch = Some(
                branch_ref
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch_ref)
                    .to_string(),
            );
        } else if line == "bare" {
            is_bare = true;
        }
    }

    // Don't forget the last entry
    if let Some(path) = current_path {
        entries.push(WorktreeInfo {
            path,
            branch: current_branch,
            is_bare,
        });
    }

    entries
}

pub fn add_worktree(path: &Path, branch: &str, start_point: &str) -> Result<()> {
    let path_str = path.to_string_lossy();
    git(&["worktree", "add", "--no-track", "-b", branch, &path_str, start_point])?;
    Ok(())
}

pub fn remove_worktree(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    git(&["worktree", "remove", &path_str])?;
    Ok(())
}

pub fn delete_branch(branch: &str) -> Result<()> {
    git(&["branch", "-D", branch])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_worktree_list_basic() {
        let output = "\
worktree /home/user/project
HEAD abc123
branch refs/heads/main
\n\
worktree /home/user/project-feat
HEAD def456
branch refs/heads/chnn/feat
";
        let entries = parse_worktree_list(output);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("/home/user/project"));
        assert_eq!(entries[0].branch.as_deref(), Some("main"));
        assert!(!entries[0].is_bare);
        assert_eq!(entries[1].path, PathBuf::from("/home/user/project-feat"));
        assert_eq!(entries[1].branch.as_deref(), Some("chnn/feat"));
    }

    #[test]
    fn test_parse_worktree_list_bare() {
        let output = "\
worktree /home/user/project.git
bare
";
        let entries = parse_worktree_list(output);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_bare);
    }

    #[test]
    fn test_parse_worktree_list_detached() {
        let output = "\
worktree /home/user/project
HEAD abc123
branch refs/heads/main
\n\
worktree /home/user/project-detached
HEAD def456
detached
";
        let entries = parse_worktree_list(output);
        assert_eq!(entries.len(), 2);
        assert!(entries[1].branch.is_none());
    }

    #[test]
    fn test_repo_name() {
        assert_eq!(repo_name(Path::new("/home/user/my-project")), "my-project");
        assert_eq!(repo_name(Path::new("/foo/bar")), "bar");
    }
}
