// Workspace changes / PR status / dev-run — all derived from real evidence
// (git + gh in the workspace worktree), never from agent prose.

use crate::state::AppState;
use std::path::PathBuf;

async fn worktree_of(state: &AppState, workspace_id: &str) -> Result<PathBuf, String> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT r.path FROM workspaces w JOIN repos r ON w.repo_id = r.id WHERE w.id = ?")
            .bind(workspace_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    match row {
        Some((Some(root),)) => Ok(workspace_runtime_core::worktree::worktree_path_for(
            std::path::Path::new(&root),
            workspace_id,
        )),
        _ => Err(format!("workspace {workspace_id} has no repo path")),
    }
}

fn git(dir: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .arg("-C").arg(dir)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}

/// Changed files with real +/- counts: `git diff --numstat HEAD` plus untracked files.
#[tauri::command]
pub async fn workspace_changes(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<serde_json::Value, String> {
    let wt = worktree_of(&state, &workspace_id).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut rows: Vec<serde_json::Value> = Vec::new();
        if let Ok(out) = git(&wt, &["diff", "--numstat", "HEAD"]) {
            for line in out.lines().filter(|l| !l.is_empty()) {
                let mut it = line.split('\t');
                let (added, deleted, path) = (
                    it.next().unwrap_or("0"), it.next().unwrap_or("0"), it.next().unwrap_or(""),
                );
                rows.push(serde_json::json!({
                    "path": path, "added": added, "deleted": deleted, "status": "M",
                }));
            }
        }
        if let Ok(out) = git(&wt, &["status", "--porcelain"]) {
            for line in out.lines() {
                let path = line.get(3..).unwrap_or("").trim();
                if !path.is_empty() && line.starts_with("??") && !rows.iter().any(|r| r["path"] == path) {
                    rows.push(serde_json::json!({
                        "path": path, "added": "new", "deleted": "0", "status": "A",
                    }));
                }
            }
        }
        Ok::<_, String>(serde_json::json!({ "changes": rows }))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// PR state from `gh` in the worktree (absent → null pr).
#[tauri::command]
pub async fn pr_status(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<serde_json::Value, String> {
    let wt = worktree_of(&state, &workspace_id).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("gh")
            .current_dir(&wt)
            .args(["pr", "view", "--json", "number,state,mergeable,url,title,isDraft"])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Ok(serde_json::json!({ "pr": null }));
        }
        let pr: serde_json::Value = serde_json::from_slice(&out.stdout)
            .map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "pr": pr,
            "ready": pr.get("mergeable").and_then(|m| m.as_str()) == Some("MERGEABLE")
                && pr.get("isDraft").and_then(|d| d.as_bool()) != Some(true),
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Merge the workspace PR via gh (user's gh auth; explicit button action).
#[tauri::command]
pub async fn merge_pr(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<serde_json::Value, String> {
    let wt = worktree_of(&state, &workspace_id).await?;
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("gh")
            .current_dir(&wt)
            .args(["pr", "merge", "--squash", "--delete-branch"])
            .output()
            .map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "ok": out.status.success(),
            "stdout": String::from_utf8_lossy(&out.stdout),
            "stderr": String::from_utf8_lossy(&out.stderr),
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Run a command (e.g. the dev script) in a PTY inside the workspace.
#[tauri::command]
pub async fn run_in_workspace(
    state: tauri::State<'_, AppState>,
    terminal_id: String,
    workspace_id: String,
    command: String,
) -> Result<(), String> {
    let wt = worktree_of(&state, &workspace_id).await?;
    let session = tauri::async_runtime::spawn_blocking(move || {
        workspace_runtime_core::pty::PtySession::spawn(&command, Some(&wt))
    })
    .await
    .map_err(|e| e.to_string())??;
    let mut terminals = state.terminals.lock().await;
    terminals.insert(terminal_id, session);
    Ok(())
}
