use std::process::Command;

use assert_cmd::Command as AssertCommand;
use tempfile::TempDir;

/// Set up a test environment: a bare "remote" repo and a local clone.
/// Returns (temp_dir, local_clone_path).
/// The local clone has an initial commit on "main" and origin/HEAD set.
fn setup_repos() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let bare_path = tmp.path().join("remote.git");
    let local_path = tmp.path().join("myproject");

    // Create bare repo
    Command::new("git")
        .args(["init", "--bare"])
        .arg(&bare_path)
        .output()
        .unwrap();

    // Set default branch to main in bare repo
    Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(&bare_path)
        .output()
        .unwrap();

    // Clone it
    Command::new("git")
        .args(["clone"])
        .arg(&bare_path)
        .arg(&local_path)
        .output()
        .unwrap();

    // Ensure local default branch is main regardless of system git defaults.
    Command::new("git")
        .args(["checkout", "-B", "main"])
        .current_dir(&local_path)
        .output()
        .unwrap();

    // Configure user in local clone
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(&local_path)
        .output()
        .unwrap();

    // Create initial commit on main
    std::fs::write(local_path.join("README.md"), "# Test\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&local_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "--no-verify", "-m", "initial commit"])
        .current_dir(&local_path)
        .output()
        .unwrap();

    // Push main to remote
    Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(&local_path)
        .output()
        .unwrap();

    // Set origin/HEAD so detect_main_branch works
    Command::new("git")
        .args(["remote", "set-head", "origin", "main"])
        .current_dir(&local_path)
        .output()
        .unwrap();

    (tmp, local_path)
}

fn wt_cmd(dir: &std::path::Path) -> AssertCommand {
    let mut cmd = AssertCommand::cargo_bin("wt").unwrap();
    cmd.current_dir(dir);
    cmd
}

#[test]
fn test_help() {
    AssertCommand::cargo_bin("wt")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Git worktree manager"));
}

#[test]
fn test_config_init() {
    let (_tmp, local) = setup_repos();

    // Should succeed
    wt_cmd(&local)
        .args(["config", "init"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Initialized .wtconfig.toml"));

    // Config file should exist
    assert!(local.join(".wtconfig.toml").exists());

    // .git/info/exclude should contain .wtconfig.toml
    let exclude = std::fs::read_to_string(local.join(".git/info/exclude")).unwrap();
    assert!(exclude.contains(".wtconfig.toml"));

    // Running again should fail (already exists)
    wt_cmd(&local)
        .args(["config", "init"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}

#[test]
fn test_config_init_dry_run() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["config", "init", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("[dry-run]"));

    // File should NOT have been created
    assert!(!local.join(".wtconfig.toml").exists());
}

#[test]
fn test_new_and_list_and_remove() {
    let (_tmp, local) = setup_repos();

    // Create a worktree
    wt_cmd(&local)
        .args(["new", "feat"])
        .assert()
        .success()
        .stderr(predicates::str::contains("Created worktree 'feat'"));

    // Worktree directory should exist
    let wt_path = _tmp.path().join("myproject-feat");
    assert!(wt_path.exists());
    assert!(wt_path.join("README.md").exists());

    // List should show the slug
    wt_cmd(&local)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feat"));

    // ls alias should also work
    wt_cmd(&local)
        .args(["ls"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feat"));

    // Remove it
    wt_cmd(&local)
        .args(["rm", "feat"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Removed worktree 'feat'"));

    // Directory should be gone
    assert!(!wt_path.exists());

    // List should be empty
    wt_cmd(&local).args(["ls"]).assert().success().stdout("");
}

#[test]
fn test_new_with_branch_prefix() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "feat", "--branch-prefix", "chnn"])
        .assert()
        .success()
        .stderr(predicates::str::contains("branch chnn/feat"));

    let wt_path = _tmp.path().join("myproject-feat");
    assert!(wt_path.exists());
}

#[test]
fn test_new_with_multi_segment_prefix() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "feat", "--branch-prefix", "team/chnn"])
        .assert()
        .success()
        .stderr(predicates::str::contains("branch team/chnn/feat"));
}

#[test]
fn test_new_with_config_prefix() {
    let (_tmp, local) = setup_repos();

    std::fs::write(local.join(".wtconfig.toml"), "branch_prefix = [\"chnn\"]\n").unwrap();

    wt_cmd(&local)
        .args(["new", "feat"])
        .assert()
        .success()
        .stderr(predicates::str::contains("branch chnn/feat"));
}

#[test]
fn test_new_cli_prefix_overrides_config() {
    let (_tmp, local) = setup_repos();

    std::fs::write(
        local.join(".wtconfig.toml"),
        "branch_prefix = [\"from-config\"]\n",
    )
    .unwrap();

    wt_cmd(&local)
        .args(["new", "feat", "--branch-prefix", "from-cli"])
        .assert()
        .success()
        .stderr(predicates::str::contains("branch from-cli/feat"));
}

#[test]
fn test_new_runs_post_create_commands_from_config() {
    let (_tmp, local) = setup_repos();

    std::fs::write(
        local.join(".wtconfig.toml"),
        "post_create_commands = [\"git rev-parse --show-toplevel > .wt-post-create-path\"]\n",
    )
    .unwrap();

    wt_cmd(&local).args(["new", "feat"]).assert().success();

    let wt_path = _tmp.path().join("myproject-feat");
    let marker = wt_path.join(".wt-post-create-path");
    assert!(marker.exists());

    let recorded_path = std::fs::read_to_string(&marker).unwrap();
    assert_eq!(
        std::path::PathBuf::from(recorded_path.trim())
            .canonicalize()
            .unwrap(),
        wt_path.canonicalize().unwrap()
    );
}

#[test]
fn test_new_rejects_slug_with_slash() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "bad/slug"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("must not contain '/'"));
}

#[test]
fn test_new_rejects_existing_directory() {
    let (_tmp, local) = setup_repos();

    // Create the directory manually
    let wt_path = _tmp.path().join("myproject-taken");
    std::fs::create_dir_all(&wt_path).unwrap();

    wt_cmd(&local)
        .args(["new", "taken"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already exists"));
}

#[test]
fn test_new_dry_run() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "feat", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("[dry-run]"));

    // Should NOT have created the worktree
    let wt_path = _tmp.path().join("myproject-feat");
    assert!(!wt_path.exists());
}

#[test]
fn test_new_with_symlinks() {
    let (_tmp, local) = setup_repos();

    // Create a file to symlink
    std::fs::write(local.join(".env"), "SECRET=123\n").unwrap();

    wt_cmd(&local)
        .args(["new", "feat", "--symlink-file", ".env"])
        .assert()
        .success();

    let wt_path = _tmp.path().join("myproject-feat");
    let symlink = wt_path.join(".env");
    assert!(symlink.exists());
    assert!(symlink.is_symlink());

    // Symlink should point to root worktree's .env
    // Use canonicalize to handle macOS /var -> /private/var symlink
    let target = std::fs::read_link(&symlink).unwrap();
    assert_eq!(
        target.canonicalize().unwrap(),
        local.join(".env").canonicalize().unwrap()
    );
}

#[test]
fn test_new_symlink_warns_on_missing_source() {
    let (_tmp, local) = setup_repos();

    // Don't create .env — it should warn but not fail
    wt_cmd(&local)
        .args(["new", "feat", "--symlink-file", ".env"])
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "warning: symlink source does not exist",
        ));
}

#[test]
fn test_remove_nonexistent() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["rm", "no-such-slug"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no worktree found"));
}

#[test]
fn test_remove_multiple() {
    let (_tmp, local) = setup_repos();

    // Create two worktrees
    wt_cmd(&local).args(["new", "one"]).assert().success();
    wt_cmd(&local).args(["new", "two"]).assert().success();

    // Remove both at once
    wt_cmd(&local).args(["rm", "one", "two"]).assert().success();

    // Both should be gone
    assert!(!_tmp.path().join("myproject-one").exists());
    assert!(!_tmp.path().join("myproject-two").exists());
}

#[test]
fn test_list_from_linked_worktree() {
    let (_tmp, local) = setup_repos();

    // Create a worktree
    wt_cmd(&local).args(["new", "feat"]).assert().success();

    let wt_path = _tmp.path().join("myproject-feat");

    // Run ls from inside the linked worktree
    wt_cmd(&wt_path)
        .args(["ls"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feat"));
}

#[test]
fn test_new_accepts_prompt_flag() {
    // The binary accepts -p (for --help visibility) but the flag is handled
    // by the shell wrapper. The binary just creates the worktree as normal.
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "feat", "-p"])
        .assert()
        .success()
        .stderr(predicates::str::contains("Created worktree 'feat'"));

    let wt_path = _tmp.path().join("myproject-feat");
    assert!(wt_path.exists());
}

#[test]
fn test_new_accepts_dangerously_skip_permissions_flag() {
    let (_tmp, local) = setup_repos();

    wt_cmd(&local)
        .args(["new", "feat", "-p", "--dangerously-skip-permissions"])
        .assert()
        .success()
        .stderr(predicates::str::contains("Created worktree 'feat'"));
}

#[test]
fn test_shell_init_outputs_function() {
    let output = AssertCommand::cargo_bin("wt")
        .unwrap()
        .arg("shell-init")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    assert!(stdout.contains("wt()"));
    assert!(stdout.contains("command wt"));
    assert!(stdout.contains("claude"));
    assert!(stdout.contains("EDITOR"));
    assert!(stdout.contains("--dangerously-skip-permissions"));
}
