# Floq Interface Specification v2
Reference: product interface study (v2). ● observed · ○ proposed · ? unresolved

## 1. Information architecture
**App → Project → Workspace → Chat → Messages/agent activity.** Dev tools sit alongside chats inside the workspace. Floq maps: Project=`repos` row, Workspace=worktree row (`workspaces`), Chat=`sessions` row. Isolation mechanism: Floq uses git worktrees (our explicit choice; ● our implementation, ○ vs. reference which is unspecified).

## 2. Navigation (sidebar)
● Breadcrumb-free sidebar with: Dashboard/History (○ placeholder this pass), Pinned (○ later), **Projects** as collapsible groups containing workspace rows.
Workspace row: name, status marker (Floq defines: green dot = active worktree exists ● ours), **+adds/−dels** (○ from `git diff --numstat HEAD` per workspace — real evidence), derived status text. `+` per project creates a workspace; project `+` adds a project.
? Unread semantics (we defer), per-row avatars (multiplayer — out of scope).

## 3. Workspace header + chat tabs
● Breadcrumb `Project › workspace` + overflow placeholder; chat tab strip (`Untitled`…, `+` new). Selecting a chat sets the submission target — never cross-wires (state is keyed by session id ○ enforced).
? Pencil icon on tabs = rename (deferred).

## 4. New-chat empty state + transcript selection ●
Empty chat shows "New chat in <workspace>" + **"Add chat transcripts:"** chips of the workspace's other chats.
○ Floq semantics (explicit, snapshot): selecting chip(s) prepends that transcript's messages as context to the first agent run in the new chat; the run records "included transcript: X". Not live-linked, not synchronized — stated in UI copy. ? Attachments/tool-output inclusion: excluded v1.

## 5. Messages: checklists & tool events
● Agent replies render markdown task lists (`- [ ]`) as live checklists (pending ○ / active = first unchecked ○ / done ✔) — derived only from agent-recorded events, never prose claims.
● Collapsed "Agent run · Ns" groups (real elapsed time recorded by backend).
○ Individual inline tool rows (Grep "4 matches", Glob path, Read "819 lines") — requires streaming agent output (`claude -p --output-format stream-json`); **next pass**, spec'd but not built tonight.
● Code blocks in replies render monospaced in dark container with preserved indentation (no syntax coloring yet ○).

## 6. Right panel
● **All files** = real repo tree from the worktree (incl. dotfiles) via `list_file_tree` (depth-capped). ● **Changes N** = `git diff --numstat` + untracked. Checks ○ = `gh pr checks` (next pass). Review ○ placeholder. File search icon ○ later.
? Change-count baseline = HEAD of the worktree branch (our definition, stated in tooltip).

## 7. Execution: Setup · Run · Terminal ●
- **Setup**: run an install command in the workspace PTY (default `pnpm install`, editable).
- **Run**: run dev script (`pnpm dev`) in workspace PTY.
- **Terminal**: interactive shell in the worktree (clean env: TERM set, npm_config_prefix stripped ●).
Each is a distinct PTY session owned by the workspace; switching workspaces stops showing the previous one's output ○.

## 8. Delivery
● PR header via `gh pr view` (number/state/ready) + Merge button → `gh pr merge --squash`. State derived from gh, never agent prose ●. Absent → "No pull request".
? Merge strategy UI, branch protections → defined by GitHub, surfaced by gh.

## 9. Working state ●
Header status line names the active process ("Claude Code is working…", "Merging PR…"), scoped per action — no ambiguous global spinner.

## 10. Context switching & concurrency rules ○ (our explicit rules)
- Workspace switch reloads chats, changes, PR, and stops displaying prior exec output (PTYs keep running in background; terminals are per-workspace keyed).
- Chats within a workspace share the worktree filesystem. Concurrent agent runs in the same workspace are allowed but uncoordinated (v1) — flagged as a known limitation; the UI disables "Run agent" while a run is active **in that chat**.
- Transcript inclusion is a one-way snapshot at first send.

## 11. Unresolved (do not guess)
- Multiplayer presence & sharing links
- Checks tab details beyond gh
- Tab rename; workspace archive flows
- Whether Setup should auto-detect package manager
