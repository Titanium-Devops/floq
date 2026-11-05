// settings get/set (architecture §8: settings table).

use crate::state::AppState;

#[tauri::command]
pub async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(&key)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.0))
}

#[tauri::command]
pub async fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(&key)
    .bind(&value)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
