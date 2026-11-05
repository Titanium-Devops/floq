// workspace_runtime_core — PTY + workspace (git worktree) runtime.
// Mirrors the `workspace_runtime_core` crate found in the Floq binary
// (e.g. `workspace_runtime_core::pty::PtyManager`).

pub mod pty;
pub mod worktree;
pub mod artifacts;
