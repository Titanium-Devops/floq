-- Floq SQLite schema (v1)
-- schema (architecture §8), adapted for local-worktree-only, cross-platform.
-- Owned by crates/floq_lib (used by sqlx::migrate!).

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS repos (
    id             TEXT PRIMARY KEY,
    remote         TEXT,
    path           TEXT,
    storage_version INTEGER NOT NULL DEFAULT 1,
    archive_script TEXT,
    run_script     TEXT,
    run_script_mode TEXT NOT NULL DEFAULT 'concurrent',
    display_order  INTEGER NOT NULL DEFAULT 0,
    hidden         INTEGER NOT NULL DEFAULT 0,
    icon           TEXT,
    project_config TEXT,
    custom_prompt_code_review TEXT,
    custom_prompt_create_pr   TEXT,
    custom_prompt_rename_branch TEXT,
    custom_prompt_general     TEXT,
    custom_prompt_fix_errors  TEXT,
    custom_prompt_resolve_merge_conflicts TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workspaces (
    id               TEXT PRIMARY KEY,
    repo_id          TEXT REFERENCES repos(id),
    active_session_id TEXT,
    unread           INTEGER NOT NULL DEFAULT 0,
    placeholder_branch_name TEXT,
    state            TEXT NOT NULL DEFAULT 'active',
    initialization_parent_branch TEXT,
    big_terminal_mode INTEGER NOT NULL DEFAULT 0,
    setup_log_path   TEXT,
    initialization_log_path TEXT,
    initialization_files_copied INTEGER,
    pinned_at        TEXT,
    linked_workspace_ids TEXT,
    notes            TEXT,
    intended_target_branch TEXT,
    archive_commit   TEXT,
    pr_title         TEXT,
    pr_description   TEXT,
    secondary_directory_name TEXT,
    linked_directory_paths TEXT,
    hosting_server_url TEXT,
    manual_status    TEXT,
    derived_status   TEXT NOT NULL DEFAULT 'in-progress',
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id               TEXT PRIMARY KEY,
    workspace_id     TEXT REFERENCES workspaces(id),
    status           TEXT,
    claude_session_id TEXT,
    agent_type       TEXT,
    title            TEXT NOT NULL DEFAULT 'Untitled',
    model            TEXT,
    permission_mode  TEXT NOT NULL DEFAULT 'default',
    unread_count     INTEGER NOT NULL DEFAULT 0,
    freshly_compacted INTEGER NOT NULL DEFAULT 0,
    context_token_count INTEGER NOT NULL DEFAULT 0,
    context_used_percent FLOAT,
    is_compacting    INTEGER NOT NULL DEFAULT 0,
    is_hidden        INTEGER NOT NULL DEFAULT 0,
    fast_mode        INTEGER NOT NULL DEFAULT 0,
    thinking_enabled INTEGER NOT NULL DEFAULT 1,
    codex_thinking_level TEXT,
    claude_effort_level TEXT,
    agent_personality TEXT,
    resume_session_at TEXT,
    last_user_message_at TEXT,
    feed_offset      INTEGER,
    queue_paused_at  TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS session_messages (
    id               TEXT PRIMARY KEY,
    session_id       TEXT REFERENCES sessions(id),
    sent_at          TEXT,
    full_message     TEXT,
    cancelled_at     TEXT,
    model            TEXT,
    sdk_message_id   TEXT,
    last_assistant_message_id TEXT,
    turn_id          TEXT,
    is_resumable_message INTEGER,
    queue_order      INTEGER,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS diff_comments (
    id               TEXT PRIMARY KEY,
    author_avatar_url TEXT,
    is_outdated      INTEGER,
    is_resolved      INTEGER,
    end_line_number  INTEGER,
    updated_at       INTEGER,
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS terminal_sessions (
    id               TEXT PRIMARY KEY,
    has_received_user_input INTEGER,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS port_forwards (
    id            TEXT PRIMARY KEY,
    workspace_id  TEXT,
    local_port    INTEGER,
    remote_port   INTEGER,
    auto_detected INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_port_forwards_local_port ON port_forwards(local_port);
CREATE UNIQUE INDEX IF NOT EXISTS idx_port_forwards_workspace_remote ON port_forwards(workspace_id, remote_port);

CREATE TABLE IF NOT EXISTS attachments (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS todo_lists (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS instance_processes (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS symlinks_pending_deletion (
    id          TEXT PRIMARY KEY
);
CREATE TABLE IF NOT EXISTS migration_rollbacks (
    version     INTEGER PRIMARY KEY,
    rollback_sql TEXT
);

CREATE TRIGGER IF NOT EXISTS update_workspaces_updated_at
AFTER UPDATE ON workspaces
BEGIN
    UPDATE workspaces SET updated_at = datetime('now') WHERE id = NEW.id;
END;
