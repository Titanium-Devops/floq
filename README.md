<div align="center">

<img src="assets/logo.png" alt="Floq" width="340">

# Floq

**Your team. Your agents. One workspace.**

Bring conversations, code, and review together so your team can build with AI and stay close to the work.

**3 panes: projects, the conversation, the change.**

[Website](https://www.getfloq.ai) · [Report an issue](../../issues)

</div>

---

![The Floq workspace](assets/workspace-hero.png)

## The workspace, as it is

Projects on the left, each with its own workspaces. The developer and the agent in the centre. The change, its preview and its review on the right. Nothing leaves the window.

- **Projects → Workspaces → Chats.** Each workspace is an isolated git worktree — one per stream of work. The branch is the unit of integration.
- **Agents that actually run.** Claude Code, Codex, Cursor, and OpenCode run inside the workspace, with their execution recorded — not just their prose.
- **Real change tracking.** File changes come from git itself (`+adds / −dels` per file), never from what the agent claims it did.
- **Delivery in context.** Pull-request status and merge sit beside the code — derived from `gh`, so "Ready to merge" means ready.

![Conversation and change](assets/workspace-close.png)

## How a change moves through it

1. **Select a project** — any local git repo becomes a project folder.
2. **Spin up a workspace** — a fresh worktree (city-named, like *Toronto* or *Lisbon*) off your base branch.
3. **Start a chat** — each conversation keeps its own history inside the workspace. New chats can include an earlier transcript as one-way context.
4. **Direct the agent** — Plan mode, Fast mode, and per-agent model selection. Runs appear as collapsed activity groups with real elapsed time; task lists render as live checklists.
5. **Inspect the change** — the Changes tab shows every touched file with additions and deletions straight from `git diff --numstat`.
6. **Run it** — Setup, Run, and Terminal tabs execute inside the workspace (PTY-backed), so you test exactly what the agent changed.
7. **Ship it** — review the PR header, and merge when it's actually ready.

## Built with

| Layer | Tech |
|---|---|
| Shell | Tauri 2 (Rust + WebView) |
| Backend | Rust (`floq_lib` + `workspace_runtime_core`), tokio, sqlx |
| Storage | SQLite with versioned migrations |
| Terminal | portable-pty |
| Agents | Claude Code · Codex · Cursor · OpenCode |
| Frontend | React + Vite + TypeScript, Geist |

Cross-platform: macOS and Linux.

## Getting started

```bash
pnpm install
pnpm tauri dev
```

Requirements: [Rust](https://rustup.rs), Node 18+, pnpm, git. Agent CLIs install in-app from the Agents panel (`npm i -g @anthropic-ai/claude-code`, `@openai/codex`, …). GitHub delivery uses your existing [`gh`](https://cli.github.com) login.

## Status

Pre-release. Interface spec: [INTERFACE-SPEC.md](INTERFACE-SPEC.md).

<div align="center">
<sub>Built for software teams.</sub>
</div>
