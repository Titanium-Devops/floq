// git worktree lifecycle — create from a base branch, remove, list.
// Uses the `git` CLI (git is a prerequisite for local workspaces).

use std::path::Path;
use std::process::Command;

fn run_git(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))
}

fn check(output: std::process::Output) -> Result<String, String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

fn s(p: &Path) -> Result<&str, String> {
    p.to_str().ok_or_else(|| "non-utf8 path".to_string())
}

/// Canonical on-disk location of a workspace's worktree: a sibling
/// `floq-worktrees/<workspace_id>` directory next to the repo root.
pub fn worktree_path_for(repo_root: &Path, workspace_id: &str) -> std::path::PathBuf {
    repo_root
        .parent()
        .unwrap_or_else(|| std::path::Path::new("/"))
        .join("floq-worktrees")
        .join(workspace_id)
}

/// Create a worktree at `worktree_path` for `branch`, branching off
/// `origin/<base_branch>` when a remote exists, else the local `<base_branch>`.
/// Fetch is best-effort: local-only repos have no `origin`.
pub fn create(repo_root: &Path, worktree_path: &Path, branch: &str, base_branch: &str) -> Result<(), String> {
    let repo = s(repo_root)?;
    if let Ok(out) = run_git(&["-C", repo, "fetch", "origin"]) {
        let _ = check(out); // best-effort
    }
    let wt = s(worktree_path)?;
    let remote = run_git(&["-C", repo, "worktree", "add", "-b", branch, wt, &format!("origin/{base_branch}")])?;
    if remote.status.success() {
        return Ok(());
    }
    let local = run_git(&["-C", repo, "worktree", "add", "-b", branch, wt, base_branch])?;
    check(local)?;
    Ok(())
}

pub fn remove(repo_root: &Path, worktree_path: &Path) -> Result<(), String> {
    let out = run_git(&["-C", s(repo_root)?, "worktree", "remove", "--force", s(worktree_path)?])?;
    check(out)?;
    Ok(())
}

pub fn list(repo_root: &Path) -> Result<Vec<String>, String> {
    let out = run_git(&["-C", s(repo_root)?, "worktree", "list", "--porcelain"])?;
    let stdout = check(out)?;
    Ok(stdout
        .lines()
        .filter(|l| l.starts_with("worktree "))
        .map(|l| l.trim_start_matches("worktree ").to_string())
        .collect())
}
