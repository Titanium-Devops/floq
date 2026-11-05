// keychain — real OS keychain via `keyring` (macOS Keychain / Linux Secret
// Service), falling back to the settings table when no backend is available.

use crate::state::AppState;

fn namespaced(key: &str) -> String {
    format!("secret:{key}")
}

fn keyring_set(key: &str, value: &str) -> bool {
    keyring::Entry::new("floq", key)
        .and_then(|e| e.set_secret(value.as_bytes()))
        .is_ok()
}

fn keyring_get(key: &str) -> Option<String> {
    keyring::Entry::new("floq", key)
        .and_then(|e| e.get_secret())
        .ok()
        .and_then(|v| String::from_utf8(v).ok())
}

fn keyring_delete(key: &str) -> bool {
    keyring::Entry::new("floq", key)
        .and_then(|e| e.delete_credential())
        .is_ok()
}

#[tauri::command]
pub async fn set_secret(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    if keyring_set(&key, &value) {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
    )
    .bind(namespaced(&key))
    .bind(value)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_secret(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    if let Some(v) = keyring_get(&key) {
        return Ok(Some(v));
    }
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(namespaced(&key))
        .fetch_optional(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.map(|r| r.0))
}

#[tauri::command]
pub async fn delete_secret(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<(), String> {
    keyring_delete(&key);
    sqlx::query("DELETE FROM settings WHERE key = ?")
        .bind(namespaced(&key))
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
