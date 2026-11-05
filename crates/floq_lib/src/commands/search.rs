// grep_workspace / fuzzy_match_files / list_file_tree / list_workspace_artifacts
// (architecture §6: file_search). Simple, dependency-light implementations.

use std::path::Path;

#[tauri::command]
pub async fn grep_workspace(
    workspace_path: String,
    query: String,
    case_sensitive: Option<bool>,
    max_results: Option<usize>,
) -> Result<Vec<String>, String> {
    let mut cmd = tokio::process::Command::new("grep");
    cmd.arg("-rIn"); // recursive, ignore binary, line numbers
    if !case_sensitive.unwrap_or(false) {
        cmd.arg("-i");
    }
    cmd.arg("--").arg(&query).arg(&workspace_path);
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    let max = max_results.unwrap_or(200).max(1);
    if lines.len() > max {
        lines.truncate(max);
    }
    Ok(lines)
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<i32> {
    let q: Vec<char> = query.chars().collect();
    let mut qi = 0;
    let mut score = 0i32;
    let mut streak = 0i32;
    for ch in candidate.chars() {
        if qi < q.len() && ch == q[qi] {
            qi += 1;
            streak += 1;
            score += streak * streak;
        } else {
            streak = 0;
        }
    }
    if qi == q.len() {
        Some(score)
    } else {
        None
    }
}

#[tauri::command]
pub async fn fuzzy_match_files(files: Vec<String>, query: String) -> Result<Vec<String>, String> {
    let mut scored: Vec<(i32, String)> = files
        .into_iter()
        .filter_map(|f| fuzzy_score(&query, &f).map(|s| (s, f)))
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(scored.into_iter().map(|(_, f)| f).collect())
}

#[tauri::command]
pub async fn list_file_tree(
    directory_path: String,
    max_depth: Option<usize>,
) -> Result<serde_json::Value, String> {
    let max = max_depth.unwrap_or(4).min(8);
    tauri::async_runtime::spawn_blocking(move || {
        fn walk(path: &Path, depth: usize, max: usize) -> serde_json::Value {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            let mut node = serde_json::json!({
                "name": name,
                "path": path.to_string_lossy().into_owned(),
                "is_dir": path.is_dir(),
                "children": [],
            });
            if path.is_dir() && depth < max {
                if let Ok(entries) = std::fs::read_dir(path) {
                    let mut children = Vec::new();
                    for e in entries.flatten().take(500) {
                        children.push(walk(&e.path(), depth + 1, max));
                    }
                    node["children"] = serde_json::json!(children);
                }
            }
            node
        }
        Ok::<_, String>(walk(Path::new(&directory_path), 0, max))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_workspace_artifacts(
    workspace_path: String,
    cursor: Option<usize>,
) -> Result<serde_json::Value, String> {
    let skip = cursor.unwrap_or(0);
    tauri::async_runtime::spawn_blocking(move || {
        let mut paths = Vec::new();
        fn walk(dir: &Path, out: &mut Vec<String>) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(&p, out);
                    } else {
                        out.push(p.to_string_lossy().into_owned());
                    }
                }
            }
        }
        walk(Path::new(&workspace_path), &mut paths);
        paths.sort();
        let page: Vec<String> = paths.into_iter().skip(skip).take(200).collect();
        Ok::<_, String>(serde_json::json!({
            "items": page,
            "next_cursor": if page.is_empty() { None } else { Some(skip + 200) },
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}
