// app_logger — cross-platform log persistence + retrieval (architecture §6).

use tauri::Manager;

#[tauri::command]
pub async fn append_log(app: tauri::AppHandle, record: String) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let path = dir.join("floq.log");
    let mut f = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| e.to_string())?;
    f.write_all(format!("{record}\n").as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn read_logs(app: tauri::AppHandle, limit: Option<usize>) -> Result<String, String> {
    let dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
    let path = dir.join("floq.log");
    match tokio::fs::read_to_string(&path).await {
        Ok(s) => {
            let lines: Vec<&str> = s.lines().collect();
            let n = limit.unwrap_or(200).min(lines.len());
            Ok(lines[lines.len() - n..].join("\n"))
        }
        Err(_) => Ok(String::new()),
    }
}
