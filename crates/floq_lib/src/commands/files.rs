// write_file_async / get_file_metadata (architecture §7).

#[tauri::command]
pub async fn write_file_async(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content.as_bytes())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_metadata(path: String) -> Result<serde_json::Value, String> {
    let meta = tokio::fs::metadata(&path).await.map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "path": path,
        "is_file": meta.is_file(),
        "is_dir": meta.is_dir(),
        "is_symlink": meta.file_type().is_symlink(),
        "size": meta.len(),
    }))
}
