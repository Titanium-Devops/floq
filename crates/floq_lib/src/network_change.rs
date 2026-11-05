// network_change — reachability check (architecture §6).

#[tauri::command]
pub async fn network_status() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    let result = client
        .head("https://api.github.com")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    Ok(serde_json::json!({
        "online": result.is_ok(),
        "latency_ms": start.elapsed().as_millis(),
    }))
}
