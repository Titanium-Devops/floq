// Database layer — SQLite connection + migrations (Phase 1).

use std::path::Path;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

/// Open (or create) the SQLite database at `db_path`, enable foreign keys,
/// and run the embedded migrations.
pub async fn init(db_path: &Path) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
