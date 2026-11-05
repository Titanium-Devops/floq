// Tauri command handlers (the IPC surface, architecture §7/§14.1).
// Grouped by subsystem; each function is a `#[tauri::command]` stub.

pub mod shell;
pub mod files;
pub mod ws;
pub mod ports;
pub mod sync;
pub mod search;
pub mod speech;
pub mod binaries;
pub mod repos;
pub mod sessions;
pub mod settings;
pub mod terminal;
pub mod changes;
