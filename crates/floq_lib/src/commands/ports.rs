// start_port_forward / stop_port_forward / list_port_forwards /
// resolve_port_forward_token / reject_port_forward_token (architecture §14.2).

#[tauri::command]
pub async fn start_port_forward(
    workspace_id: String,
    local_port: u16,
    remote_port: u16,
) -> Result<(), String> {
    // TODO(Phase 4): SSH gate + HTTP/2 tunnel.
    let _ = (workspace_id, local_port, remote_port);
    Err("not implemented".into())
}
