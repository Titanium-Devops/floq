// raw_websocket_open / close / ping / send_text (architecture §9b).

#[tauri::command]
pub async fn raw_websocket_open(url: String, headers: Option<Vec<String>>, token: Option<String>) -> Result<(), String> {
    // TODO(Phase 4): named-connection RawWebSocketManager.
    let _ = (url, headers, token);
    Err("not implemented".into())
}
