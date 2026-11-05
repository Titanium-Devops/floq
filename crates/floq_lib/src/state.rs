// Managed application state. Expand with subsystem singletons as they land.

use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::process::Child;
use tokio::sync::Mutex;
use workspace_runtime_core::pty::PtySession;

pub struct AppState {
    pub db: SqlitePool,
    pub cancellable_shell: Mutex<HashMap<String, Child>>,
    pub terminals: Mutex<HashMap<String, PtySession>>,
}
