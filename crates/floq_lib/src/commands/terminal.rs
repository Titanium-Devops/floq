// Terminal commands — spawn / write / read / resize / kill a PTY session
// (workspace_runtime_core::pty::PtySession), kept in managed state.

use crate::state::AppState;
use std::path::PathBuf;

#[tauri::command]
pub async fn spawn_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
    cwd: Option<String>,
) -> Result<(), String> {
    let cwd_owned = cwd.map(PathBuf::from);
    let session = tauri::async_runtime::spawn_blocking(move || {
        workspace_runtime_core::pty::PtySession::spawn_shell(cwd_owned.as_deref())
    })
    .await
    .map_err(|e| e.to_string())??;
    let mut terminals = state.terminals.lock().await;
    terminals.insert(id, session);
    Ok(())
}

#[tauri::command]
pub async fn write_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
    data: String,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    match terminals.get_mut(&id) {
        Some(s) => s.write(data.as_bytes()),
        None => Err(format!("no terminal {id}")),
    }
}

#[tauri::command]
pub async fn read_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let mut terminals = state.terminals.lock().await;
    match terminals.get_mut(&id) {
        Some(s) => {
            let mut buf = [0u8; 8192];
            let n = s.read(&mut buf)?;
            Ok(String::from_utf8_lossy(&buf[..n]).into_owned())
        }
        None => Err(format!("no terminal {id}")),
    }
}

#[tauri::command]
pub async fn resize_terminal(
    state: tauri::State<'_, AppState>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    match terminals.get_mut(&id) {
        Some(s) => s.resize(rows, cols),
        None => Err(format!("no terminal {id}")),
    }
}

#[tauri::command]
pub async fn kill_terminal(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut terminals = state.terminals.lock().await;
    if let Some(mut s) = terminals.remove(&id) {
        s.kill()?;
    }
    Ok(())
}
