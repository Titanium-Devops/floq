// Entity models for the SQLite data layer (Phase 1).
// Maps 1:1 to migrations/0001_initial.sql.

use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Repo {
    pub id: String,
    pub remote: Option<String>,
    pub path: Option<String>,
    pub storage_version: i64,
    pub archive_script: Option<String>,
    pub run_script: Option<String>,
    pub run_script_mode: String,
    pub display_order: i64,
    pub hidden: i64,
    pub icon: Option<String>,
    pub project_config: Option<String>,
    pub custom_prompt_code_review: Option<String>,
    pub custom_prompt_create_pr: Option<String>,
    pub custom_prompt_rename_branch: Option<String>,
    pub custom_prompt_general: Option<String>,
    pub custom_prompt_fix_errors: Option<String>,
    pub custom_prompt_resolve_merge_conflicts: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Workspace {
    pub id: String,
    pub repo_id: Option<String>,
    pub active_session_id: Option<String>,
    pub unread: i64,
    pub placeholder_branch_name: Option<String>,
    pub state: String,
    #[sqlx(skip)]
    pub worktree_path: Option<String>,
    pub initialization_parent_branch: Option<String>,
    pub big_terminal_mode: i64,
    pub setup_log_path: Option<String>,
    pub initialization_log_path: Option<String>,
    pub initialization_files_copied: Option<i64>,
    pub pinned_at: Option<String>,
    pub linked_workspace_ids: Option<String>,
    pub notes: Option<String>,
    pub intended_target_branch: Option<String>,
    pub archive_commit: Option<String>,
    pub pr_title: Option<String>,
    pub pr_description: Option<String>,
    pub secondary_directory_name: Option<String>,
    pub linked_directory_paths: Option<String>,
    pub hosting_server_url: Option<String>,
    pub manual_status: Option<String>,
    pub derived_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Session {
    pub id: String,
    pub workspace_id: Option<String>,
    pub status: Option<String>,
    pub claude_session_id: Option<String>,
    pub agent_type: Option<String>,
    pub title: String,
    pub model: Option<String>,
    pub permission_mode: String,
    pub unread_count: i64,
    pub freshly_compacted: i64,
    pub context_token_count: i64,
    pub context_used_percent: Option<f64>,
    pub is_compacting: i64,
    pub is_hidden: i64,
    pub fast_mode: i64,
    pub thinking_enabled: i64,
    pub codex_thinking_level: Option<String>,
    pub claude_effort_level: Option<String>,
    pub agent_personality: Option<String>,
    pub resume_session_at: Option<String>,
    pub last_user_message_at: Option<String>,
    pub feed_offset: Option<i64>,
    pub queue_paused_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: Option<String>,
    pub sent_at: Option<String>,
    pub full_message: Option<String>,
    pub cancelled_at: Option<String>,
    pub model: Option<String>,
    pub sdk_message_id: Option<String>,
    pub last_assistant_message_id: Option<String>,
    pub turn_id: Option<String>,
    pub is_resumable_message: Option<i64>,
    pub queue_order: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}
