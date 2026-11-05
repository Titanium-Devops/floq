// execute_shell_command (+ cancellable variant + cancel) with a registry.

use crate::state::AppState;
use tokio::process::Command;

#[tauri::command]
pub async fn execute_shell_command(
    command: String,
    cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&command);
    if let Some(dir) = &cwd {
        cmd.current_dir(dir);
    }
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "status": output.status.code(),
    }))
}

#[tauri::command]
pub async fn execute_shell_command_cancellable(
    state: tauri::State<'_, AppState>,
    id: String,
    command: String,
    cwd: Option<String>,
) -> Result<(), String> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&command);
    if let Some(dir) = &cwd {
        cmd.current_dir(dir);
    }
    let child = cmd.spawn().map_err(|e| e.to_string())?;
    let mut registry = state.cancellable_shell.lock().await;
    registry.insert(id, child);
    Ok(())
}

#[tauri::command]
pub async fn cancel_shell_command(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut registry = state.cancellable_shell.lock().await;
    if let Some(mut child) = registry.remove(&id) {
        let _ = child.kill().await;
    }
    Ok(())
}
