use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use anyhow::{Error, Result};

pub struct Context {
    pub dry_run: bool,
    pub root_worktree: PathBuf,
    pub repo_name: String,
    progress: ProgressState,
}

impl Context {
    pub fn new(dry_run: bool) -> Result<Self> {
        let root_worktree = crate::git::root_worktree_path()?;
        let repo_name = crate::git::repo_name(&root_worktree);
        Ok(Self {
            dry_run,
            root_worktree,
            repo_name,
            progress: ProgressState::new(),
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

pub fn execute_with_progress<F>(ctx: &Context, description: &str, f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if ctx.dry_run {
        println!("[dry-run] {description}");
        return Ok(());
    }

    let spinner = ctx
        .progress
        .stderr_is_tty
        .then(|| Spinner::start(description));
    let result = f();

    if let Some(spinner) = spinner {
        spinner.stop_and_clear();
    }

    match result {
        Ok(()) => {
            ctx.progress.record_completed_step(description);
            Ok(())
        }
        Err(err) => {
            print_failure_report(ctx, description, &err);
            Err(err)
        }
    }
}

struct ProgressState {
    completed_steps: Mutex<Vec<String>>,
    stderr_is_tty: bool,
}

impl ProgressState {
    fn new() -> Self {
        Self {
            completed_steps: Mutex::new(Vec::new()),
            stderr_is_tty: io::stderr().is_terminal(),
        }
    }

    fn record_completed_step(&self, description: &str) {
        let mut completed_steps = self
            .completed_steps
            .lock()
            .expect("progress mutex poisoned while recording completed step");
        completed_steps.push(description.to_string());
    }

    fn completed_steps_snapshot(&self) -> Vec<String> {
        self.completed_steps
            .lock()
            .expect("progress mutex poisoned while reading completed steps")
            .clone()
    }
}

struct Spinner {
    stop_signal: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    fn start(description: &str) -> Self {
        let stop_signal = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop_signal);
        let description = description.to_string();

        let handle = thread::spawn(move || {
            let frames = ['|', '/', '-', '\\'];
            let mut frame_idx = 0usize;

            while !worker_stop.load(Ordering::Relaxed) {
                eprint!("\r{} {}", frames[frame_idx], description);
                let _ = io::stderr().flush();
                frame_idx = (frame_idx + 1) % frames.len();
                thread::sleep(Duration::from_millis(80));
            }
        });

        Self {
            stop_signal,
            handle: Some(handle),
        }
    }

    fn stop_and_clear(mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        eprint!("\r\x1b[2K\r");
        let _ = io::stderr().flush();
    }
}

fn print_failure_report(ctx: &Context, failed_step: &str, err: &Error) {
    let completed_steps = ctx.progress.completed_steps_snapshot();
    eprintln!(
        "{}",
        format_failure_report(&completed_steps, failed_step, err)
    );
}

fn format_failure_report(completed_steps: &[String], failed_step: &str, err: &Error) -> String {
    let mut lines = Vec::new();
    lines.push("Progress before failure:".to_string());

    for step in completed_steps {
        lines.push(format!("  [done] {step}"));
    }

    lines.push(format!("  [failed] {failed_step}"));
    lines.push(format!("  [error] {err:#}"));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::format_failure_report;

    #[test]
    fn failure_report_lists_completed_and_failed_steps() {
        let report = format_failure_report(
            &[
                "git fetch origin main".to_string(),
                "git worktree add ...".to_string(),
            ],
            "symlink /tmp/project-feat/.env",
            &anyhow::anyhow!("symlink source missing"),
        );

        assert!(report.contains("[done] git fetch origin main"));
        assert!(report.contains("[done] git worktree add ..."));
        assert!(report.contains("[failed] symlink /tmp/project-feat/.env"));
        assert!(report.contains("[error] symlink source missing"));
    }

    #[test]
    fn failure_report_handles_no_completed_steps() {
        let report = format_failure_report(&[], "git fetch origin main", &anyhow::anyhow!("boom"));

        assert!(!report.contains("[done]"));
        assert!(report.contains("[failed] git fetch origin main"));
        assert!(report.contains("[error] boom"));
    }
}
