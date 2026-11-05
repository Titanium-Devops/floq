// Agent + binary commands (Phase 3): request a binary download and run an agent.

use crate::binary_manager::{self, AgentKind};
use crate::state::AppState;
use tauri::Manager;

#[tauri::command]
pub async fn request_agent_binary_download(
    app: tauri::AppHandle,
    agent_type: String,
    url: String,
    sha256: Option<String>,
) -> Result<String, String> {
    let kind =
        AgentKind::from_str(&agent_type).ok_or_else(|| format!("unknown agent type: {agent_type}"))?;
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("agents");
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;

    let dest = dir.join(kind.binary());
    let tmp = dir.join(format!("{}.download", kind.binary()));
    binary_manager::download_and_verify(&url, &tmp, sha256.as_deref()).await?;

    if url.ends_with(".gz") {
        binary_manager::decompress_gzip(&tmp, &dest)?;
        let _ = tokio::fs::remove_file(&tmp).await;
    } else {
        tokio::fs::rename(&tmp, &dest).await.map_err(|e| e.to_string())?;
    }
    binary_manager::make_executable(&dest)?;
    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn run_agent(
    state: tauri::State<'_, AppState>,
    agent_type: String,
    prompt: String,
    cwd: Option<String>,
    session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let kind =
        AgentKind::from_str(&agent_type).ok_or_else(|| format!("unknown agent type: {agent_type}"))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    // Persist the user prompt as a message.
    if let Some(sid) = &session_id {
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, full_message, model) VALUES (?, ?, ?, ?)",
        )
        .bind(format!("msg-{now}"))
        .bind(sid)
        .bind(&prompt)
        .bind("user")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    let mut cmd = tokio::process::Command::new(kind.binary());
    for arg in kind.prompt_args(&prompt) {
        cmd.arg(arg);
    }
    if let (Some(sid), AgentKind::Claude) = (&session_id, kind) {
        cmd.arg("--resume").arg(sid);
    }
    if let Some(dir) = &cwd {
        cmd.current_dir(dir);
    }
    let started = std::time::Instant::now();
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    let elapsed = started.elapsed().as_secs();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    // Record the run itself (rendered as a collapsed activity group in the UI).
    if let Some(sid) = &session_id {
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, full_message, model) VALUES (?, ?, ?, ?)",
        )
        .bind(format!("msg-{now}-run"))
        .bind(sid)
        .bind(format!("Agent run · {elapsed}s"))
        .bind("run")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Persist the agent reply as a message.
    if let Some(sid) = &session_id {
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, full_message, model) VALUES (?, ?, ?, ?)",
        )
        .bind(format!("msg-{now}-r"))
        .bind(sid)
        .bind(&stdout)
        .bind(kind.binary())
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(serde_json::json!({
        "agent": kind.binary(),
        "stdout": stdout,
        "stderr": String::from_utf8_lossy(&output.stderr),
        "status": output.status.code(),
    }))
}

#[tauri::command]
pub async fn list_agents() -> Result<Vec<serde_json::Value>, String> {
    let mut out = Vec::new();
    for name in ["claude", "codex", "cursor-agent", "opencode"] {
        let kind = AgentKind::from_str(name).ok_or_else(|| format!("unknown agent: {name}"))?;
        let installed = tokio::process::Command::new("which")
            .arg(name)
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);
        out.push(serde_json::json!({
            "name": name,
            "binary": kind.binary(),
            "installed": installed,
            "install_command": kind.install_command(),
        }));
    }
    Ok(out)
}

#[tauri::command]
pub async fn install_agent(agent_type: String) -> Result<serde_json::Value, String> {
    let kind =
        AgentKind::from_str(&agent_type).ok_or_else(|| format!("unknown agent type: {agent_type}"))?;
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(kind.install_command())
        .output()
        .await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "status": output.status.code(),
    }))
}
