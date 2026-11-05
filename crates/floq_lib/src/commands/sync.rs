// start_remote_file_sync / stop / trigger / status / resolve / reject / cleanup
// (architecture §14.3, git-backed).

#[tauri::command]
pub async fn start_remote_file_sync(workspace_id: String) -> Result<(), String> {
    // TODO(Phase 4): git ls-files / check-attr baseline sync.
    let _ = workspace_id;
    Err("not implemented".into())
}
