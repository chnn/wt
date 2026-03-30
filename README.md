A small Rust CLI that automates the tedious parts of creating Git working trees.

## Install

```
cargo install --git https://github.com/chnn/wt
```

## Usage

```
wt help
```

## Shell Integration

Add this to your `.zshrc` or `.bashrc`:

```bash
eval "$(command wt shell-init)"
```

This wraps `wt new` so that it automatically `cd`s into the new worktree.

With `-p`, it opens `$EDITOR` for a prompt before creating the worktree. After the worktree is created, `wt` will start an interactive `claude` session with that prompt running.

```bash
wt new my-feature -p
wt new my-feature -p --dangerously-skip-permissions
```
