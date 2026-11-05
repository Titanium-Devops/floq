// Session + message commands (Phase 2 — data layer completion).

use crate::models::{Session, SessionMessage};
use crate::state::AppState;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn create_session(
    state: tauri::State<'_, AppState>,
    id: String,
    workspace_id: String,
    agent_type: Option<String>,
    fast_mode: Option<i64>,
    permission_mode: Option<String>,
) -> Result<Session, String> {
    let pool: &SqlitePool = &state.db;
    sqlx::query(
        "INSERT INTO sessions (id, workspace_id, agent_type, fast_mode, permission_mode) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&workspace_id)
    .bind(&agent_type)
    .bind(fast_mode.unwrap_or(0))
    .bind(permission_mode.as_deref().unwrap_or("default"))
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions(
    state: tauri::State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<Session>, String> {
    sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE workspace_id = ? ORDER BY created_at DESC",
    )
    .bind(&workspace_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn append_message(
    state: tauri::State<'_, AppState>,
    id: String,
    session_id: String,
    full_message: String,
    model: Option<String>,
) -> Result<SessionMessage, String> {
    let pool: &SqlitePool = &state.db;
    sqlx::query(
        "INSERT INTO session_messages (id, session_id, full_message, model) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&session_id)
    .bind(&full_message)
    .bind(&model)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query_as::<_, SessionMessage>("SELECT * FROM session_messages WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_messages(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<SessionMessage>, String> {
    sqlx::query_as::<_, SessionMessage>(
        "SELECT * FROM session_messages WHERE session_id = ? ORDER BY rowid ASC",
    )
    .bind(&session_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())
}
