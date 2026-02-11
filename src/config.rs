use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct WtConfig {
    pub branch_prefix: Option<Vec<String>>,
    pub symlink_files: Option<Vec<String>>,
    pub post_create_commands: Option<Vec<String>>,
}

pub fn load_config(root_worktree: &Path) -> Result<WtConfig> {
    let config_path = root_worktree.join(".wtconfig.toml");
    if !config_path.exists() {
        return Ok(WtConfig::default());
    }
    let contents =
        std::fs::read_to_string(&config_path).context("failed to read .wtconfig.toml")?;
    let config: WtConfig = toml::from_str(&contents).context("failed to parse .wtconfig.toml")?;
    Ok(config)
}

pub fn config_template() -> &'static str {
    "\
# wt — Git Worktree Manager configuration
#
# Branch prefix segments, joined with '/'.
# Example: [\"chnn\"] produces branches like \"chnn/my-feature\"
# Example: [\"team\", \"chnn\"] produces \"team/chnn/my-feature\"
#
# branch_prefix = [\"chnn\"]

# Files to symlink from the root worktree into new worktrees.
# Useful for editor configs, environment files, etc.
#
# symlink_files = [\".env\", \".idea\"]

# Shell commands to run in each new worktree after creation.
# Commands run in order and stop on the first failure.
#
# post_create_commands = [\"npm install\", \"npm run build\"]
"
}

pub fn format_branch_name(prefix: &Option<Vec<String>>, slug: &str) -> String {
    match prefix {
        Some(segments) if !segments.is_empty() => {
            let mut parts = segments.clone();
            parts.push(slug.to_string());
            parts.join("/")
        }
        _ => slug.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_branch_name_no_prefix() {
        assert_eq!(format_branch_name(&None, "feat"), "feat");
        assert_eq!(format_branch_name(&Some(vec![]), "feat"), "feat");
    }

    #[test]
    fn test_format_branch_name_single_prefix() {
        let prefix = Some(vec!["chnn".to_string()]);
        assert_eq!(format_branch_name(&prefix, "feat"), "chnn/feat");
    }

    #[test]
    fn test_format_branch_name_multi_prefix() {
        let prefix = Some(vec!["team".to_string(), "chnn".to_string()]);
        assert_eq!(format_branch_name(&prefix, "feat"), "team/chnn/feat");
    }

    #[test]
    fn test_load_config_missing_file() {
        let dir = std::env::temp_dir().join("wt-test-no-config");
        std::fs::create_dir_all(&dir).unwrap();
        let config = load_config(&dir).unwrap();
        assert!(config.branch_prefix.is_none());
        assert!(config.symlink_files.is_none());
        assert!(config.post_create_commands.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_config_partial() {
        let dir = std::env::temp_dir().join("wt-test-partial-config");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".wtconfig.toml"), "branch_prefix = [\"chnn\"]\n").unwrap();
        let config = load_config(&dir).unwrap();
        assert_eq!(config.branch_prefix, Some(vec!["chnn".to_string()]));
        assert!(config.symlink_files.is_none());
        assert!(config.post_create_commands.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_config_full() {
        let dir = std::env::temp_dir().join("wt-test-full-config");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(".wtconfig.toml"),
            "branch_prefix = [\"team\", \"chnn\"]\nsymlink_files = [\".env\", \".idea\"]\npost_create_commands = [\"npm install\", \"npm test\"]\n",
        )
        .unwrap();
        let config = load_config(&dir).unwrap();
        assert_eq!(
            config.branch_prefix,
            Some(vec!["team".to_string(), "chnn".to_string()])
        );
        assert_eq!(
            config.symlink_files,
            Some(vec![".env".to_string(), ".idea".to_string()])
        );
        assert_eq!(
            config.post_create_commands,
            Some(vec!["npm install".to_string(), "npm test".to_string()])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
