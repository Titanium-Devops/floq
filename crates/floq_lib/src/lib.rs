// floq_lib — the backend core. Module layout mirrors the reconstructed
// `floq_lib` crate (architecture §6/§14). Each module is a stub to fill in
// during the build phases in BUILD-PLAN.md.

pub mod state;
pub mod db;
pub mod models;
pub mod commands;
pub mod raw_websocket;
pub mod tcp_tunnel;
pub mod port_forward;
pub mod binary_manager;
pub mod file_sync;
pub mod pty;
pub mod file_search;
pub mod native_http;
pub mod on_device_speech;
pub mod app_logger;
pub mod app_actions;
pub mod sidecar;
pub mod updater;
pub mod notifications;
pub mod keychain;
pub mod network_change;
