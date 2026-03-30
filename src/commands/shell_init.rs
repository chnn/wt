use anyhow::Result;

pub fn run() -> Result<()> {
    print!(
        r#"wt() {{
  case "$1" in
    new)
      # Scan for -p/--prompt and --dangerously-skip-permissions
      local has_prompt=false
      local has_skip_perms=false
      local wt_args=()
      for arg in "$@"; do
        case "$arg" in
          -p|--prompt) has_prompt=true ;;
          --dangerously-skip-permissions) has_skip_perms=true ;;
          *) wt_args+=("$arg") ;;
        esac
      done

      # If -p was given, open $EDITOR before creating the worktree
      local prompt_file=""
      if $has_prompt; then
        prompt_file=$(mktemp "${{TMPDIR:-/tmp}}/wt-prompt-XXXXXX.md")
        ${{EDITOR:-vi}} "$prompt_file"
        if [[ ! -s "$prompt_file" ]] || [[ -z "$(tr -d '[:space:]' < "$prompt_file")" ]]; then
          echo "prompt is empty, aborting" >&2
          rm -f "$prompt_file"
          return 1
        fi
      fi

      # Create the worktree (flags stripped so the binary just does its job)
      local wt_path
      wt_path=$(command wt "${{wt_args[@]}}")
      local exit_code=$?
      if [[ $exit_code -ne 0 ]]; then
        [[ -n "$prompt_file" ]] && rm -f "$prompt_file"
        return $exit_code
      fi

      cd "$wt_path" || return 1

      # If a prompt was authored, launch claude
      if [[ -n "$prompt_file" ]]; then
        local prompt
        prompt=$(<"$prompt_file")
        rm -f "$prompt_file"

        local claude_args=("$prompt")
        $has_skip_perms && claude_args+=(--dangerously-skip-permissions)
        claude "${{claude_args[@]}}"
      fi
      ;;
    *)
      command wt "$@"
      ;;
  esac
}}
"#
    );
    Ok(())
}
