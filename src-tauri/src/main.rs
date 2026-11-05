#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Floq bootstrap — mirrors the reconstructed `_main -> main -> run` chain
// (architecture §3). Registers plugins, initializes SQLite, manages state,
// and registers commands.

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() {
    tauri::Builder::default()
        // --- plugins (Floq's set, architecture §4) ---
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        // --- DB init + managed state (Phase 1) ---
        .setup(|app| {
            let db_path = app.path().app_data_dir()?.join("floq.db");
            let pool = tauri::async_runtime::block_on(floq_lib::db::init(&db_path))?;
            app.manage(floq_lib::state::AppState {
                db: pool,
                cancellable_shell: Default::default(),
                terminals: Default::default(),
            });
            Ok(())
        })
        // --- commands (architecture §7/§14.1; add handlers as implemented) ---
        .invoke_handler(tauri::generate_handler![
            floq_lib::commands::repos::create_repo,
            floq_lib::commands::repos::list_repos,
            floq_lib::commands::repos::create_workspace,
            floq_lib::commands::repos::list_workspaces,
            floq_lib::commands::sessions::create_session,
            floq_lib::commands::sessions::list_sessions,
            floq_lib::commands::sessions::append_message,
            floq_lib::commands::sessions::list_messages,
            floq_lib::commands::settings::get_setting,
            floq_lib::commands::settings::set_setting,
            floq_lib::commands::terminal::spawn_terminal,
            floq_lib::commands::terminal::write_terminal,
            floq_lib::commands::terminal::read_terminal,
            floq_lib::commands::terminal::resize_terminal,
            floq_lib::commands::terminal::kill_terminal,
            floq_lib::commands::changes::workspace_changes,
            floq_lib::commands::changes::pr_status,
            floq_lib::commands::changes::merge_pr,
            floq_lib::commands::changes::run_in_workspace,
            floq_lib::app_logger::append_log,
            floq_lib::app_logger::read_logs,
            floq_lib::keychain::set_secret,
            floq_lib::keychain::get_secret,
            floq_lib::keychain::delete_secret,
            floq_lib::commands::shell::execute_shell_command,
            floq_lib::commands::shell::execute_shell_command_cancellable,
            floq_lib::commands::shell::cancel_shell_command,
            floq_lib::commands::search::grep_workspace,
            floq_lib::commands::search::fuzzy_match_files,
            floq_lib::commands::search::list_file_tree,
            floq_lib::commands::search::list_workspace_artifacts,
            floq_lib::commands::binaries::request_agent_binary_download,
            floq_lib::commands::binaries::run_agent,
            floq_lib::commands::binaries::list_agents,
            floq_lib::commands::binaries::install_agent,
            floq_lib::network_change::network_status,
            floq_lib::commands::files::write_file_async,
            floq_lib::commands::files::get_file_metadata,
            floq_lib::commands::ws::raw_websocket_open,
            floq_lib::commands::ports::start_port_forward,
            floq_lib::commands::sync::start_remote_file_sync,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
