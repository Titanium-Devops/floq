// Repo + workspace commands (Phase 1 slice).

use crate::models::{Repo, Workspace};
use crate::state::AppState;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn create_repo(
    state: tauri::State<'_, AppState>,
    id: String,
    remote: Option<String>,
    path: Option<String>,
) -> Result<Repo, String> {
    let pool: &SqlitePool = &state.db;
    sqlx::query("INSERT INTO repos (id, remote, path) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&remote)
        .bind(&path)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Repo>("SELECT * FROM repos WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_repos(state: tauri::State<'_, AppState>) -> Result<Vec<Repo>, String> {
    sqlx::query_as::<_, Repo>("SELECT * FROM repos ORDER BY display_order ASC, created_at ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_workspace(
    state: tauri::State<'_, AppState>,
    id: String,
    repo_id: String,
    branch: Option<String>,
) -> Result<Workspace, String> {
    let pool: &SqlitePool = &state.db;
    sqlx::query("INSERT INTO workspaces (id, repo_id, placeholder_branch_name) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&repo_id)
        .bind(&branch)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // If the repo has a local checkout, create the git worktree on disk.
    let repo_root: Option<(Option<String>,)> = sqlx::query_as("SELECT path FROM repos WHERE id = ?")
        .bind(&repo_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    if let Some((Some(root),)) = repo_root {
        let base = branch.clone().unwrap_or_else(|| "main".to_string());
        let root_path = std::path::PathBuf::from(&root);
        let worktree_path =
            workspace_runtime_core::worktree::worktree_path_for(&root_path, &id);
        let wt_branch = format!("floq-{id}");
        let root2 = root_path.clone();
        let wt = worktree_path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            workspace_runtime_core::worktree::create(&root2, &wt, &wt_branch, &base)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    }

    sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_workspaces(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Workspace>, String> {
    let mut rows = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces ORDER BY created_at ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    for w in rows.iter_mut() {
        if let Some(rid) = w.repo_id.clone() {
            let repo: Option<(Option<String>,)> =
                sqlx::query_as("SELECT path FROM repos WHERE id = ?")
                    .bind(&rid)
                    .fetch_optional(&state.db)
                    .await
                    .map_err(|e| e.to_string())?;
            if let Some((Some(root),)) = repo {
                w.worktree_path = Some(
                    workspace_runtime_core::worktree::worktree_path_for(
                        std::path::Path::new(&root),
                        &w.id,
                    )
                    .to_string_lossy()
                    .into_owned(),
                );
            }
        }
    }
    Ok(rows)
}
