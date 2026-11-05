import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import "./styles.css";
import wordmark from "./assets/floq-wordmark.png";

// ---- types ----
interface Repo { id: string; remote: string | null; path: string | null; }
interface Workspace {
  id: string; repo_id: string | null; state: string;
  derived_status: string; worktree_path: string | null;
}
interface Session {
  id: string; workspace_id: string | null; agent_type: string | null;
  title: string; fast_mode: number; permission_mode: string;
}
interface SessionMessage { id: string; full_message: string | null; model: string | null; created_at: string; }
interface ChangeRow { path: string; added: string; deleted: string; status: string; }
interface PrInfo { number: number; state: string; url: string; title: string; }
interface ShellResult { stdout: string; stderr: string; status: number | null; }
interface TreeNode { name: string; path: string; is_dir: boolean; children: TreeNode[]; }

const CITIES = ["Lisbon", "Kyoto", "Oslo", "Hanoi", "Austin", "Prague", "Taipei",
  "Marseille", "Toronto", "Nairobi", "Seville", "Cusco", "Denver", "Bali"];
const pick = () => CITIES[Math.floor(Math.random() * CITIES.length)];
const agentLabel: Record<string, string> = {
  claude: "Claude Code", codex: "Codex", cursor: "Cursor", opencode: "OpenCode",
};
const splitPath = (p: string) => {
  const i = p.lastIndexOf("/");
  return i === -1 ? { dir: "", file: p } : { dir: p.slice(0, i + 1), file: p.slice(i + 1) };
};

// ---- message body renderers ----
const Checklist: React.FC<{ text: string }> = ({ text }) => {
  const rows = text.split("\n").map(l => {
    const m = l.match(/^\s*[-*]\s+\[( |x|X)\]\s*(.*)$/);
    return m ? { done: m[1].toLowerCase() === "x", task: m[2] } : null;
  }).filter(Boolean) as { done: boolean; task: string }[];
  if (rows.length === 0) return null;
  const firstOpen = rows.findIndex(r => !r.done);
  return (
    <div className="checklist">
      {rows.map((r, i) => (
        <div key={i} className="row">
          <span className={"box " + (r.done ? "done" : i === firstOpen ? "active" : "")}>
            {r.done ? "✔" : i === firstOpen ? "◐" : "○"}
          </span>
          <span className={"task" + (r.done ? " done" : "")}>{r.task}</span>
        </div>
      ))}
    </div>
  );
};

const MessageBody: React.FC<{ text: string }> = ({ text }) => {
  const segments = text.split("```");
  return (
    <>
      {segments.map((seg, i) =>
        i % 2 === 1
          ? <pre key={i} className="codeblock">{seg.replace(/^\w*\n/, "")}</pre>
          : <React.Fragment key={i}>
              {i > 0 ? null : null}
              <Checklist text={seg} />
              {seg.split("\n").filter(l => !/^\s*[-*]\s+\[( |x|X)\]/.test(l)).join("\n")}
            </React.Fragment>
      )}
    </>
  );
};

const FileTree: React.FC<{ node: TreeNode; depth: number; expanded: Set<string>; toggle: (p: string) => void }> =
({ node, depth, expanded, toggle }) => {
  if (!node.is_dir) {
    return <div className="tree-row" style={{ paddingLeft: depth * 12 + 8 }} title={node.path}>📄 {node.name}</div>;
  }
  const open = expanded.has(node.path);
  return (
    <>
      <div className="tree-row dir" style={{ paddingLeft: depth * 12 + 8 }}
           onClick={() => toggle(node.path)} title={node.path}>
        <span className="chev">{open ? "▾" : "▸"}</span> 📁 {node.name}
      </div>
      {open && (node.children || []).map(c => (
        <FileTree key={c.path} node={c} depth={depth + 1} expanded={expanded} toggle={toggle} />
      ))}
    </>
  );
};

function App() {
  const [repos, setRepos] = useState<Repo[]>([]);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [wsChanges, setWsChanges] = useState<Record<string, { a: number; d: number }>>({});
  const [openProjects, setOpenProjects] = useState<Set<string>>(new Set());
  const [sessions, setSessions] = useState<Session[]>([]);
  const [messages, setMessages] = useState<SessionMessage[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState("");
  const [selectedSession, setSelectedSession] = useState("");
  const [contextChips, setContextChips] = useState<Set<string>>(new Set());
  const [agent, setAgent] = useState("claude");
  const [planMode, setPlanMode] = useState(false);
  const [fastMode, setFastMode] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [expandedRuns, setExpandedRuns] = useState<Set<string>>(new Set());
  const [changes, setChanges] = useState<ChangeRow[]>([]);
  const [pr, setPr] = useState<PrInfo | null>(null);
  const [prReady, setPrReady] = useState(false);
  const [rightTab, setRightTab] = useState<"files" | "changes" | "checks" | "review">("changes");
  const [fileTree, setFileTree] = useState<TreeNode | null>(null);
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());
  const [execTab, setExecTab] = useState<"setup" | "run" | "terminal">("run");
  const [cmd, setCmd] = useState("");
  const [cmdResult, setCmdResult] = useState<ShellResult | null>(null);
  const [setupCmd, setSetupCmd] = useState("pnpm install");
  const [execTerm, setExecTerm] = useState<{ id: string; output: string } | null>(null);
  const [termId, setTermId] = useState("");
  const [termOutput, setTermOutput] = useState("");
  const [termInput, setTermInput] = useState("");
  const [online, setOnline] = useState<boolean | null>(null);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");

  const ws = workspaces.find(w => w.id === selectedWorkspace);
  const wsPath = ws?.worktree_path ?? null;
  const selRepo = repos.find(r => r.id === ws?.repo_id);

  const refreshRepos = () => invoke<Repo[]>("list_repos").then(setRepos).catch(e => setError(String(e)));
  const refreshWorkspaces = () =>
    invoke<Workspace[]>("list_workspaces").then(all => {
      setWorkspaces(all);
      // per-workspace change summaries (real git numstat)
      all.forEach(w => {
        if (!w.worktree_path) return;
        invoke<{ changes: ChangeRow[] }>("workspace_changes", { workspaceId: w.id })
          .then(r => {
            const a = r.changes.reduce((s, c) => s + (c.added === "new" ? 0 : parseInt(c.added) || 0), 0);
            const d = r.changes.reduce((s, c) => s + (parseInt(c.deleted) || 0), 0);
            setWsChanges(prev => ({ ...prev, [w.id]: { a, d } }));
          }).catch(() => {});
      });
    }).catch(e => setError(String(e)));
  const refreshSessions = (wsId: string) => {
    setSelectedWorkspace(wsId);
    invoke<Session[]>("list_sessions", { workspaceId: wsId }).then(setSessions).catch(e => setError(String(e)));
  };
  const refreshMessages = (sid: string) => {
    setSelectedSession(sid);
    invoke<SessionMessage[]>("list_messages", { sessionId: sid }).then(setMessages).catch(e => setError(String(e)));
  };
  const refreshChanges = (wsId: string) =>
    invoke<{ changes: ChangeRow[] }>("workspace_changes", { workspaceId: wsId })
      .then(r => setChanges(r.changes)).catch(() => setChanges([]));
  const refreshPr = (wsId: string) =>
    invoke<{ pr: PrInfo | null; ready: boolean }>("pr_status", { workspaceId: wsId })
      .then(r => { setPr(r.pr); setPrReady(r.ready); }).catch(() => setPr(null));
  const refreshTree = (path: string) =>
    invoke<TreeNode>("list_file_tree", { directoryPath: path, maxDepth: 3 })
      .then(t => { setFileTree(t); setExpandedDirs(new Set([t.path])); }).catch(() => setFileTree(null));

  const selectWorkspace = (wsId: string) => {
    setChanges([]); setPr(null); setFileTree(null); setExecTerm(null);
    setContextChips(new Set());
    refreshSessions(wsId); refreshChanges(wsId); refreshPr(wsId);
    const w = workspaces.find(x => x.id === wsId);
    if (w?.worktree_path) refreshTree(w.worktree_path);
  };

  useEffect(() => {
    invoke<{ online: boolean }>("network_status").then(r => setOnline(r.online)).catch(() => setOnline(false));
    refreshRepos();
  }, []);
  useEffect(() => { refreshWorkspaces(); }, [repos.length]);

  // ---- actions ----
  const addRepo = async () => {
    try {
      const path = await open({ directory: true }) as string | null;
      if (!path) return;
      const id = path.split("/").filter(Boolean).pop() || "repo";
      setStatus(`Adding ${id}…`);
      await invoke("create_repo", { id, remote: null, path });
      setStatus(""); refreshRepos();
    } catch (e) { setError(`Add repo failed: ${e}`); setStatus(""); }
  };
  const newWorkspace = async (repoId: string) => {
    try {
      const id = pick();
      setStatus(`Creating worktree ${id}…`);
      await invoke("create_workspace", { id, repoId, branch: "main" });
      setStatus(""); refreshWorkspaces();
    } catch (e) { setError(`Create workspace failed: ${e}`); setStatus(""); }
  };
  const newSession = async () => {
    if (!selectedWorkspace) return;
    try {
      const id = crypto.randomUUID();
      await invoke("create_session", {
        id, workspaceId: selectedWorkspace, agentType: agent,
        fastMode: fastMode ? 1 : 0, permissionMode: planMode ? "plan" : "default",
      });
      refreshSessions(selectedWorkspace);
    } catch (e) { setError(`New session failed: ${e}`); }
  };
  const send = async () => {
    if (!selectedSession || !prompt.trim()) return;
    await invoke("append_message", { id: crypto.randomUUID(), sessionId: selectedSession, fullMessage: prompt, model: "user" });
    setPrompt(""); refreshMessages(selectedSession);
  };
  const runAgentNow = async () => {
    if (!prompt.trim()) return;
    // Transcript inclusion: explicit one-way snapshot into the first run of an empty chat.
    let finalPrompt = prompt;
    if (messages.length === 0 && contextChips.size > 0) {
      for (const sid of contextChips) {
        const tx = await invoke<SessionMessage[]>("list_messages", { sessionId: sid }).catch(() => []);
        const text = tx.map(m => `${m.model === "user" ? "user" : "agent"}: ${m.full_message}`).join("\n");
        finalPrompt = `Context (snapshot from another chat):\n${text}\n\nTask: ${finalPrompt}`;
      }
    }
    setStatus(`${agentLabel[agent]} is working…`);
    try {
      await invoke("run_agent", {
        agentType: agent, prompt: finalPrompt, cwd: wsPath, sessionId: selectedSession || null,
      });
      if (selectedSession) refreshMessages(selectedSession);
      if (selectedWorkspace) { refreshChanges(selectedWorkspace); refreshWorkspaces(); }
    } catch (e) { setError(String(e)); }
    setStatus("");
  };
  const merge = async () => {
    setStatus("Merging PR…");
    const r = await invoke<{ ok: boolean; stdout: string; stderr: string }>("merge_pr", { workspaceId: selectedWorkspace });
    setStatus(r.ok ? "Merged" : (r.stderr || "merge failed").slice(0, 80));
    refreshPr(selectedWorkspace); refreshChanges(selectedWorkspace);
  };
  const runCmd = async () => {
    setCmdResult(await invoke<ShellResult>("execute_shell_command", { command: cmd, cwd: wsPath }));
  };
  const startExec = async (command: string) => {
    if (!selectedWorkspace) return;
    const id = crypto.randomUUID();
    try {
      await invoke("run_in_workspace", { terminalId: id, workspaceId: selectedWorkspace, command });
      setExecTerm({ id, output: "" });
    } catch (e) { setError(`Start failed: ${e}`); }
  };
  const openTerm = async () => {
    const id = crypto.randomUUID();
    await invoke("spawn_terminal", { id, cwd: wsPath });
    setTermId(id); setTermOutput("");
  };
  const sendTerm = async () => {
    await invoke("write_terminal", { id: termId, data: termInput + "\r" });
    setTermInput("");
  };

  useEffect(() => {
    if (!execTerm) return;
    const t = setInterval(async () => {
      try {
        const c = await invoke<string>("read_terminal", { id: execTerm.id });
        if (c) setExecTerm(p => p ? { ...p, output: p.output + c } : p);
      } catch { /* exited */ }
    }, 400);
    return () => clearInterval(t);
  }, [execTerm?.id]);
  useEffect(() => {
    if (!termId) return;
    const t = setInterval(async () => {
      try {
        const c = await invoke<string>("read_terminal", { id: termId });
        if (c) setTermOutput(p => p + c);
      } catch { /* closed */ }
    }, 400);
    return () => clearInterval(t);
  }, [termId]);

  const notify = async () => {
    let granted = await isPermissionGranted();
    if (!granted) granted = (await requestPermission()) === "granted";
    if (granted) sendNotification({ title: "Floq", body: "Notifications are on." });
  };
  const toggleRun = (id: string) =>
    setExpandedRuns(prev => { const n = new Set(prev); n.has(id) ? n.delete(id) : n.add(id); return n; });
  const toggleDir = (p: string) =>
    setExpandedDirs(prev => { const n = new Set(prev); n.has(p) ? n.delete(p) : n.add(p); return n; });
  const toggleProject = (id: string) =>
    setOpenProjects(prev => { const n = new Set(prev); n.has(id) ? n.delete(id) : n.add(id); return n; });
  const toggleChip = (id: string) =>
    setContextChips(prev => { const n = new Set(prev); n.has(id) ? n.delete(id) : n.add(id); return n; });

  const sessionTabTitle = (s: Session) =>
    s.title && s.title !== "Untitled" ? s.title : (agentLabel[s.agent_type || ""] || "Session");
  const others = sessions.filter(s => s.id !== selectedSession);

  return (
    <div className="app">
      {/* ================= LEFT ================= */}
      <aside className="sidebar">
        <div className="logo"><img src={wordmark} alt="Floq" className="logo-wordmark" /></div>

        <div className="section" style={{ flex: 1 }}>
          <div className="section-title">Projects</div>
          {repos.length === 0 && <div className="empty-hint">No projects yet</div>}
          {repos.map(r => {
            const open = openProjects.has(r.id) || repos.length <= 2;
            const myWs = workspaces.filter(w => w.repo_id === r.id);
            return (
              <div key={r.id} className="proj-group">
                <div className="proj-head" onClick={() => toggleProject(r.id)}>
                  <span className="chev">{open ? "▾" : "▸"}</span>
                  <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>{r.id}</span>
                  <span className="plus" title="New workspace"
                        onClick={e => { e.stopPropagation(); newWorkspace(r.id); }}>+</span>
                </div>
                {open && myWs.map(w => {
                  const ch = wsChanges[w.id];
                  return (
                    <div key={w.id} className={"ws-row" + (selectedWorkspace === w.id ? " active" : "")}
                         onClick={() => selectWorkspace(w.id)} title={w.worktree_path || w.id}>
                      <span className="st" style={{ background: w.state === "active" ? "var(--green)" : "var(--faint)" }} />
                      <span className="name">{w.id}</span>
                      {ch && (ch.a > 0 || ch.d > 0) && <>
                        <span className="add">+{ch.a}</span><span className="del">−{ch.d}</span>
                      </>}
                    </div>
                  );
                })}
              </div>
            );
          })}
          <button className="add" onClick={addRepo}>+ Add project folder</button>
        </div>

        <div className="section">
          <div className="item" onClick={notify}>🔔 Notifications</div>
          <div className="empty-hint" style={{ padding: "2px 9px 6px" }}>
            {online === null ? "…" : online ? "🟢 online" : "🔴 offline"}
          </div>
        </div>
      </aside>

      {/* ================= CENTER ================= */}
      <main className="main">
        <div className="topbar" style={{ flexDirection: "column", alignItems: "stretch", gap: 0, padding: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10, padding: "8px 14px" }}>
            <div className="breadcrumb">
              <span className="proj">{selRepo ? selRepo.id : "Floq"}</span>
              {ws && <><span className="sep">›</span><span className="wsname">{ws.id}</span></>}
            </div>
            {status && <span className="status-line">{status}</span>}
            {wsPath && <span className="path" title={wsPath}>{wsPath}</span>}
          </div>
          {ws && (
            <div className="tabs" style={{ padding: "0 10px" }}>
              {sessions.map(s => (
                <button key={s.id} className={"tab" + (selectedSession === s.id ? " active" : "")}
                        onClick={() => refreshMessages(s.id)}>
                  {sessionTabTitle(s)}
                  {s.permission_mode === "plan" ? " ◇" : ""}{s.fast_mode ? " ⚡" : ""}
                </button>
              ))}
              <button className="tab" onClick={newSession} title="New chat">+</button>
            </div>
          )}
        </div>

        <div className="transcript">
          {!ws ? (
            <div className="empty-state">
              <div className="big">🌊</div>
              <div><b>Pick a workspace</b></div>
              <div className="hint">Projects → workspaces → chats. Each workspace is an isolated git worktree.</div>
            </div>
          ) : !selectedSession ? (
            <div className="empty-state">
              <div className="big">💬</div>
              <div><b>Open a chat</b> (or press the + tab)</div>
            </div>
          ) : messages.length === 0 ? (
            <div className="empty-state">
              <div className="big">✨</div>
              <div><b>New chat in {ws.id}</b></div>
              {others.length > 0 && <>
                <div className="hint">Add chat transcripts (one-way snapshot into the first run):</div>
                <div className="chips">
                  {others.map(s => (
                    <span key={s.id} className={"chip-sel" + (contextChips.has(s.id) ? " on" : "")}
                          onClick={() => toggleChip(s.id)}>
                      {sessionTabTitle(s)}
                    </span>
                  ))}
                </div>
              </>}
            </div>
          ) : messages.map(m => {
            if (m.model === "run") {
              const open = expandedRuns.has(m.id);
              return (
                <React.Fragment key={m.id}>
                  <div className="run-group" onClick={() => toggleRun(m.id)}>
                    <span className="chev">{open ? "▾" : "▸"}</span> ⚙ {m.full_message}
                  </div>
                  {open && <div className="run-detail">Execution record for this run (output in the reply below).</div>}
                </React.Fragment>
              );
            }
            return (
              <div key={m.id} className={"msg " + (m.model === "user" ? "user" : "agent")}>
                <div className="meta">{m.model === "user" ? "you" : (agentLabel[m.model || ""] || m.model)} · {m.created_at}</div>
                <div className="body"><MessageBody text={m.full_message || ""} /></div>
              </div>
            );
          })}
        </div>

        <div className="composer">
          <select value={agent} onChange={e => setAgent(e.target.value)}>
            <option value="claude">Claude Code</option>
            <option value="codex">Codex</option>
            <option value="cursor">Cursor</option>
            <option value="opencode">OpenCode</option>
          </select>
          <button className={"btn" + (planMode ? " primary" : "")} onClick={() => setPlanMode(v => !v)}>Plan</button>
          <button className={"btn" + (fastMode ? " primary" : "")} onClick={() => setFastMode(v => !v)}>Fast</button>
          <input className="grow" value={prompt} onChange={e => setPrompt(e.target.value)}
                 placeholder={selectedSession ? "Ask anything…" : "Create a chat first (+ tab)"}
                 onKeyDown={e => e.key === "Enter" && send()} />
          <button className="btn" onClick={send} disabled={!selectedSession}>Send</button>
          <button className="btn primary" onClick={runAgentNow} disabled={!selectedSession}>↑ Run agent</button>
        </div>
        {error && <div className="err">{error}</div>}
      </main>

      {/* ================= RIGHT ================= */}
      <aside className="right">
        <div className="pr-header">
          {pr ? (
            <>
              <span className="num">PR #{pr.number}</span>
              <span className="state">{prReady ? "● Ready to merge" : pr.state}</span>
              <span className="spacer" />
              <button className="btn merge" onClick={merge} disabled={!prReady}>Merge</button>
            </>
          ) : (
            <><span className="num" style={{ color: "#756f6c" }}>No pull request</span><span className="spacer" /></>
          )}
        </div>

        <div className="panel-tabs">
          {(["files", "changes", "checks", "review"] as const).map(t => (
            <button key={t} className={"tab" + (rightTab === t ? " active" : "")} onClick={() => setRightTab(t)}>
              {t === "files" ? "All files" : t[0].toUpperCase() + t.slice(1)}
              {t === "changes" && changes.length > 0 && <span className="badge-count">{changes.length}</span>}
            </button>
          ))}
        </div>

        <div className="change-list">
          {rightTab === "files" && (fileTree
            ? <FileTree node={fileTree} depth={0} expanded={expandedDirs} toggle={toggleDir} />
            : <div className="empty-hint">{ws ? "No tree (not a git worktree?)" : "Pick a workspace"}</div>)}
          {rightTab === "changes" && (changes.length === 0
            ? <div className="empty-hint">No changes vs HEAD — run an agent</div>
            : changes.map(c => {
                const { dir, file } = splitPath(c.path);
                return (
                  <div key={c.path} className="change-row" title={c.path}>
                    <span className="dir">{dir}</span><span className="file">{file}</span>
                    <span className="add">+{c.added}</span><span className="del">−{c.deleted}</span>
                  </div>
                );
              }))}
          {rightTab === "checks" && <div className="empty-hint">Checks via `gh pr checks` — next pass</div>}
          {rightTab === "review" && <div className="empty-hint">Review — next pass</div>}
        </div>

        <div className="exec">
          <div className="panel-tabs">
            {(["setup", "run", "terminal"] as const).map(t => (
              <button key={t} className={"tab" + (execTab === t ? " active" : "")}
                      onClick={() => setExecTab(t)}>{t[0].toUpperCase() + t.slice(1)}</button>
            ))}
          </div>
          {execTab === "setup" && (
            execTerm ? <div className="term">{execTerm.output || "running…"}</div> : (
              <div className="exec-cmd" style={{ flexDirection: "column", gap: 8 }}>
                <div className="empty-hint">Prepare the workspace environment</div>
                <div style={{ display: "flex", gap: 8 }}>
                  <input className="grow" value={setupCmd} onChange={e => setSetupCmd(e.target.value)} />
                  <button className="btn" onClick={() => startExec(setupCmd)} disabled={!ws}>Run setup</button>
                </div>
              </div>
            )
          )}
          {execTab === "run" && (
            execTerm ? <div className="term">{execTerm.output || "starting…"}</div> : (
              <div className="run-idle">
                <div className="play">▶</div>
                <button className="btn merge" onClick={() => startExec("pnpm dev")} disabled={!ws}>Start Dev</button>
                <kbd>⌘R</kbd>
                <div>Test your changes here</div>
              </div>
            )
          )}
          {execTab === "terminal" && (termId ? (
            <>
              <div className="term">{termOutput || "\u00a0"}</div>
              <div style={{ display: "flex", gap: 8, padding: "0 8px 8px" }}>
                <input className="grow" value={termInput} onChange={e => setTermInput(e.target.value)}
                       placeholder="type and Enter…" onKeyDown={e => e.key === "Enter" && sendTerm()} />
              </div>
            </>
          ) : (
            <div className="run-idle">
              <button className="btn" onClick={openTerm}>Open terminal{ws ? " in workspace" : ""}</button>
            </div>
          ))}
        </div>
      </aside>
    </div>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
