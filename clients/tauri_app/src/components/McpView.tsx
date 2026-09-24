import React, { useState, useEffect, useCallback } from "react";
import {
  Bot,
  Power,
  Copy,
  Check,
  CheckCircle2,
  XCircle,
  Code2,
  Wrench,
  Search,
  ChevronDown,
  ChevronRight,
  Sparkles,
  Activity,
  RotateCw,
  Radio,
  Layers,
  AlertCircle,
  Terminal,
  Trash2,
  X,
} from "lucide-react";
import type { McpStatusResponse, McpAccessLogEntry, McpSessionDescriptor } from "../types";
import type { FetchMcpLogsParams } from "../hooks/useDevFlowApi";
import { copyToClipboard } from "../utils/clipboard";

interface McpViewProps {
  onFetchStatus: () => Promise<McpStatusResponse>;
  onToggleServer: (enabled: boolean) => Promise<{ success: boolean; enabled: boolean }>;
  onFetchLogs: (params?: FetchMcpLogsParams | number) => Promise<McpAccessLogEntry[]>;
  onFetchSessions?: () => Promise<McpSessionDescriptor[] | string[]>;
  onFetchAgents?: () => Promise<string[]>;
  onDeleteLog?: (id: string) => Promise<{ success: boolean }>;
  onClearLogs?: (sessionId?: string) => Promise<{ success: boolean }>;
}

// Canonical fallback tool list — always shown even if API hasn't hydrated yet
const FALLBACK_TOOLS = [
  { name: "devflow_start_session", description: "Start a development session: detect project, build, install, launch, attach logs, start watcher.", inputSchema: { type: "object", properties: { project_path: { type: "string", description: "Absolute or relative path to project directory" }, target: { type: "string", description: "Device ID or name (optional)" }, framework: { type: "string", description: "Override framework adapter (optional)" } }, required: ["project_path"] } },
  { name: "devflow_stop_session", description: "Stop a development session: stop watcher, detach logs, terminate running app.", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" } }, required: ["session_id"] } },
  { name: "devflow_reload", description: "Trigger framework-aware reload (HMR/VM reload) or restart for an active session.", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" } }, required: ["session_id"] } },
  { name: "devflow_restart", description: "Trigger a full app restart (kill + relaunch) for an active session.", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" } }, required: ["session_id"] } },
  { name: "devflow_get_logs", description: "Get recent log lines and clustered crashes for a session with optional filtering.", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" }, levels: { type: "array", description: "Filter by log levels (E, W, I, D)" }, tags: { type: "array", description: "Filter by tag names" }, query: { type: "string", description: "Search query filter" }, limit: { type: "integer", description: "Max lines to return (default 100)" } }, required: ["session_id"] } },
  { name: "devflow_list_devices", description: "Return unified device list (Android devices & AVDs, iOS simulators, macOS targets) with status.", inputSchema: { type: "object", properties: {} } },
  { name: "devflow_boot_emulator", description: "Boot an Android Virtual Device (AVD) or simulator by name/id.", inputSchema: { type: "object", properties: { name: { type: "string", description: "Name or ID of the AVD/simulator to boot" } }, required: ["name"] } },
  { name: "devflow_select_device", description: "Change the active device for a session and relaunch if needed.", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" }, device_id: { type: "string", description: "Device ID to switch to" } }, required: ["session_id", "device_id"] } },
  { name: "devflow_build", description: "Perform a one-off build; returns artifact path and duration.", inputSchema: { type: "object", properties: { project_path: { type: "string", description: "Project directory path" }, config: { type: "object", description: "Optional build config overrides" } }, required: ["project_path"] } },
  { name: "devflow_doctor", description: "Run environment and project diagnostics and return structured findings with fix hints.", inputSchema: { type: "object", properties: { project_path: { type: "string", description: "Project directory path (defaults to current dir)" } } } },
  { name: "devflow_get_session_state", description: "Return the current session state (status, device, build duration, counts, last error).", inputSchema: { type: "object", properties: { session_id: { type: "string", description: "Active session ID" } }, required: ["session_id"] } },
];

// Build config JSON for a given AI host, given the current HTTP endpoint
function buildConfigs(endpoint: string) {
  return {
    claude: {
      id: "claude" as const,
      name: "Claude Desktop",
      file: "~/Library/Application Support/Claude/claude_desktop_config.json",
      json: {
        mcpServers: {
          devflow: {
            command: "devflow",
            args: ["mcp", "serve"],
          },
        },
      },
    },
    cursor: {
      id: "cursor" as const,
      name: "Cursor IDE",
      file: ".cursor/mcp.json  (workspace) or ~/.cursor/mcp.json (global)",
      json: {
        mcpServers: {
          devflow: {
            url: endpoint,
          },
        },
      },
    },
    antigravity: {
      id: "antigravity" as const,
      name: "Google Antigravity",
      file: "~/.gemini/antigravity-ide/mcp_config.json or .agents/mcp_config.json",
      json: {
        mcpServers: {
          devflow: {
            command: "devflow",
            args: ["mcp", "serve"],
          },
        },
      },
    },
    vscode: {
      id: "vscode" as const,
      name: "VS Code / Cline / Roo",
      file: ".vscode/mcp.json",
      json: {
        servers: {
          devflow: {
            type: "http",
            url: endpoint,
          },
        },
      },
    },
  };
}

type ToastKind = "success" | "error" | "info";
interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
}

let _toastId = 0;

// Agent branding helper: returns clean display name, theme color class, and icon
function getAgentBrand(agentName: string) {
  const norm = (agentName || "").toLowerCase();
  if (norm.includes("claude")) {
    return { label: "Claude", colorClass: "claude", icon: <Sparkles size={11} color="#f59e0b" /> };
  }
  if (norm.includes("cursor")) {
    return { label: "Cursor", colorClass: "cursor", icon: <Code2 size={11} color="#06b6d4" /> };
  }
  if (norm.includes("antigravity")) {
    return { label: "Antigravity", colorClass: "antigravity", icon: <Bot size={11} color="#a855f7" /> };
  }
  if (norm.includes("vscode") || norm.includes("cline") || norm.includes("roo")) {
    return { label: "VS Code", colorClass: "vscode", icon: <Terminal size={11} color="#3b82f6" /> };
  }
  if (norm.includes("cli")) {
    return { label: "CLI", colorClass: "cli", icon: <Terminal size={11} color="#10b981" /> };
  }
  return { label: agentName || "Agent", colorClass: "generic", icon: <Bot size={11} color="#94a3b8" /> };
}

export const McpView: React.FC<McpViewProps> = ({
  onFetchStatus,
  onToggleServer,
  onFetchLogs,
  onFetchSessions,
  onFetchAgents,
  onDeleteLog,
  onClearLogs,
}) => {
  const [status, setStatus] = useState<McpStatusResponse | null>(null);
  const [logs, setLogs] = useState<McpAccessLogEntry[]>([]);
  const [sessions, setSessions] = useState<McpSessionDescriptor[]>([]);
  const [agents, setAgents] = useState<string[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<string>("all");
  const [selectedSession, setSelectedSession] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<"all" | "success" | "error">("all");
  const [logSearchQuery, setLogSearchQuery] = useState<string>("");
  const [isDeletingId, setIsDeletingId] = useState<string | null>(null);
  const [isClearing, setIsClearing] = useState<boolean>(false);
  const [confirmingClear, setConfirmingClear] = useState<boolean>(false);
  const [isLogsRefreshing, setIsLogsRefreshing] = useState<boolean>(false);

  const [isLoading, setIsLoading] = useState(false);
  const [isToggling, setIsToggling] = useState(false);
  const [fetchError, setFetchError] = useState<string | null>(null);
  const [activeSubTab, setActiveSubTab] = useState<"all" | "logs" | "tools" | "claude" | "cursor" | "antigravity" | "vscode">("all");
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedTool, setExpandedTool] = useState<string | null>(null);
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback((kind: ToastKind, message: string) => {
    const id = ++_toastId;
    setToasts((prev) => [...prev, { id, kind, message }]);
    setTimeout(() => setToasts((prev) => prev.filter((t) => t.id !== id)), 3500);
  }, []);

  const loadSessions = useCallback(async () => {
    if (onFetchSessions) {
      try {
        const raw = await onFetchSessions();
        if (Array.isArray(raw)) {
          const normalized: McpSessionDescriptor[] = raw.map((item) => {
            if (typeof item === "string") {
              return {
                session_id: item,
                client: "CLI / Other",
                total_calls: 1,
                last_timestamp: "",
              };
            }
            return item;
          });
          setSessions(normalized);
        }
      } catch (_) {}
    }
  }, [onFetchSessions]);

  const loadAgents = useCallback(async () => {
    if (onFetchAgents) {
      try {
        const list = await onFetchAgents();
        if (Array.isArray(list)) {
          setAgents(list);
        }
      } catch (_) {}
    }
  }, [onFetchAgents]);

  const loadLogs = useCallback(async (isBackground = false) => {
    if (!isBackground) setIsLogsRefreshing(true);
    try {
      const params: FetchMcpLogsParams = {
        limit: 100,
        session_id: selectedSession !== "all" ? selectedSession : undefined,
        client: selectedAgent !== "all" ? selectedAgent : undefined,
        status: statusFilter !== "all" ? statusFilter : undefined,
        search: logSearchQuery.trim() || undefined,
      };
      const fetched = await onFetchLogs(params);
      setLogs(fetched || []);
    } catch (e) {
      if (!isBackground) {
        console.error("Failed to load filtered logs:", e);
      }
    } finally {
      if (!isBackground) setIsLogsRefreshing(false);
    }
  }, [onFetchLogs, selectedSession, selectedAgent, statusFilter, logSearchQuery]);

  const loadStatusAndLogs = useCallback(async () => {
    setIsLoading(true);
    setFetchError(null);
    try {
      const [statusData] = await Promise.all([
        onFetchStatus(),
        loadLogs(true),
        loadSessions(),
        loadAgents(),
      ]);
      setStatus(statusData);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setFetchError(msg);
      console.error("Failed to load MCP status or logs:", e);
    } finally {
      setIsLoading(false);
    }
  }, [onFetchStatus, loadLogs, loadSessions, loadAgents]);

  useEffect(() => {
    loadStatusAndLogs();
  }, [loadStatusAndLogs]);

  useEffect(() => {
    loadLogs(true);
  }, [loadLogs]);

  useEffect(() => {
    const handlePoll = () => {
      if (document.hidden) return;
      loadLogs(true);
      loadSessions();
      loadAgents();
    };
    const interval = setInterval(handlePoll, 5000);
    window.addEventListener("focus", handlePoll);
    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", handlePoll);
    };
  }, [loadLogs, loadSessions, loadAgents]);

  const handleDeleteLog = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    if (!onDeleteLog || isDeletingId) return;
    setIsDeletingId(id);
    try {
      const res = await onDeleteLog(id);
      if (res.success) {
        setLogs((prev) => prev.filter((item) => item.id !== id));
        showToast("info", "Log entry deleted");
        loadSessions();
      } else {
        showToast("error", "Failed to delete log entry");
      }
    } catch (_) {
      showToast("error", "Failed to delete log entry");
    } finally {
      setIsDeletingId(null);
    }
  };

  useEffect(() => {
    if (!confirmingClear) return;
    const timer = setTimeout(() => setConfirmingClear(false), 4000);
    return () => clearTimeout(timer);
  }, [confirmingClear]);

  const handleClearLogs = async () => {
    if (!onClearLogs || isClearing) return;
    if (!confirmingClear) {
      setConfirmingClear(true);
      return;
    }

    setConfirmingClear(false);
    setIsClearing(true);
    try {
      const res = await onClearLogs(selectedSession !== "all" ? selectedSession : undefined);
      if (res.success) {
        setLogs([]);
        showToast("info", "Logs cleared successfully");
        loadSessions();
      } else {
        showToast("error", "Failed to clear logs");
      }
    } catch (_) {
      showToast("error", "Failed to clear logs");
    } finally {
      setIsClearing(false);
    }
  };

  const handleToggle = async () => {
    if (isToggling) return;
    setIsToggling(true);
    const targetState = !(status?.enabled ?? false);
    try {
      const result = await onToggleServer(targetState);
      if (result.success) {
        setStatus((prev) =>
          prev ? { ...prev, enabled: result.enabled } : prev
        );
        showToast(
          "success",
          result.enabled
            ? "✅ MCP Server started — stdio & HTTP endpoints ready"
            : "⏹ MCP Server stopped"
        );
      } else {
        showToast("error", "Failed to toggle MCP server");
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      showToast("error", `Toggle failed: ${msg}`);
    } finally {
      setIsToggling(false);
    }
  };

  const handleCopy = async (text: string, key: string) => {
    if (!text || text.trim() === "" || text.trim() === "{}") {
      showToast("error", "Nothing to copy — config is empty");
      return;
    }
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopiedKey(key);
      setTimeout(() => setCopiedKey(null), 1800);
    }
  };

  // Use API tools if available and non-empty, otherwise show fallback catalog
  const toolList = (status?.tools && status.tools.length > 0) ? status.tools : FALLBACK_TOOLS;

  const filteredTools = toolList.filter((t) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      t.name.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q)
    );
  });

  // Merge distinct agents discovered from API, sessions, and logs
  const distinctAgents = React.useMemo(() => {
    const set = new Set<string>();
    agents.forEach((a) => set.add(a));
    sessions.forEach((s) => {
      if (s.client && s.client !== "unknown") set.add(s.client);
    });
    logs.forEach((l) => {
      if (l.client && l.client !== "unknown") set.add(l.client);
    });
    if (set.size === 0) {
      return ["Claude Desktop", "Cursor", "Antigravity", "VS Code", "CLI"];
    }
    return Array.from(set);
  }, [agents, sessions, logs]);

  // Filter sessions by selected agent
  const filteredSessions = React.useMemo(() => {
    if (selectedAgent === "all") return sessions;
    return sessions.filter(
      (s) => s.client.toLowerCase() === selectedAgent.toLowerCase()
    );
  }, [sessions, selectedAgent]);

  const successLogsCount = logs.filter((l) => l.status === "success").length;
  const failedLogsCount = logs.filter((l) => l.status !== "success").length;

  // Always build configs from current endpoint (fallback to 9292)
  const endpoint = status?.http_endpoint || "http://localhost:9292/rpc";
  const configs = buildConfigs(endpoint);
  const configsList = Object.values(configs).map((cfg) => ({
    ...cfg,
    // Also try to use server-supplied configs (for cursor URL etc), fall through to our hardcoded ones
    code: (() => {
      let serverJson: unknown = null;
      if (status?.configs) {
        const key = cfg.id === "claude" ? "claude_desktop" : cfg.id;
        serverJson = (status.configs as unknown as Record<string, unknown>)[key] ?? null;
      }
      const finalJson = (serverJson && typeof serverJson === "object" && Object.keys(serverJson).length > 0)
        ? serverJson
        : cfg.json;
      return JSON.stringify(finalJson, null, 2);
    })(),
    icon: cfg.id === "claude"
      ? <Sparkles size={16} color="#d97706" />
      : cfg.id === "antigravity"
      ? <Bot size={16} color="#a855f7" />
      : cfg.id === "vscode"
      ? <Code2 size={16} color="#60a5fa" />
      : <Code2 size={16} color="#38bdf8" />,
  }));

  const isEnabled = status?.enabled ?? false;

  return (
    <div className="view-container">
      {/* Toast notifications */}
      <div className="mcp-toast-stack">
        {toasts.map((t) => (
          <div key={t.id} className={`mcp-toast mcp-toast-${t.kind}`}>
            {t.kind === "success" && <CheckCircle2 size={14} />}
            {t.kind === "error" && <XCircle size={14} />}
            {t.kind === "info" && <AlertCircle size={14} />}
            <span>{t.message}</span>
          </div>
        ))}
      </div>

      {/* 1. Header */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <Bot size={22} color="#c084fc" />
            <h1 className="view-title">Model Context Protocol Hub</h1>
            <span
              className="view-badge"
              style={{
                background: isEnabled ? "rgba(16, 185, 129, 0.15)" : "rgba(148, 163, 184, 0.15)",
                color: isEnabled ? "#34d399" : "#94a3b8",
                borderColor: isEnabled ? "rgba(16, 185, 129, 0.3)" : "rgba(148, 163, 184, 0.3)",
              }}
            >
              {isEnabled ? "● Server Active" : "○ Server Idle"}
            </span>
          </div>
          <p className="view-subtitle">
            Bridge your active project targets, devices, runners, and terminal log streams directly into AI coding agents.
          </p>
        </div>

        <div className="view-actions">
          <button
            className="btn-glass"
            onClick={loadStatusAndLogs}
            disabled={isLoading}
            title="Refresh status and access logs"
          >
            <RotateCw size={14} className={isLoading ? "animate-spin" : ""} />
            <span>Refresh</span>
          </button>

          <button
            className="btn-primary"
            style={{
              background: isEnabled ? "rgba(239, 68, 68, 0.15)" : "rgba(16, 185, 129, 0.15)",
              color: isEnabled ? "#f87171" : "#34d399",
              borderColor: isEnabled ? "rgba(239, 68, 68, 0.3)" : "rgba(16, 185, 129, 0.3)",
              opacity: isToggling ? 0.6 : 1,
            }}
            onClick={handleToggle}
            disabled={isToggling}
          >
            <Power size={14} className={isToggling ? "animate-spin" : ""} />
            <span>
              {isToggling
                ? isEnabled ? "Stopping…" : "Starting…"
                : isEnabled ? "Stop MCP" : "Start MCP"}
            </span>
          </button>
        </div>
      </div>

      {/* API error banner */}
      {fetchError && (
        <div className="mcp-error-banner">
          <AlertCircle size={14} />
          <span>Could not reach backend: {fetchError}</span>
          <button className="btn-glass btn-sm" onClick={loadStatusAndLogs}>Retry</button>
        </div>
      )}

      {/* 2. Hero Strip */}
      <div className="mcp-hero-strip">
        <div className="mcp-stat-item">
          <span className="mcp-stat-label">Stdio Command</span>
          <div className="copyable-endpoint" onClick={() => handleCopy("devflow mcp serve", "cmd")}>
            <Terminal size={11} />
            <code>devflow mcp serve</code>
            {copiedKey === "cmd" ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
          </div>
        </div>

        <div className="mcp-stat-item">
          <span className="mcp-stat-label">HTTP JSON-RPC</span>
          <div
            className="copyable-endpoint"
            onClick={() => handleCopy(endpoint, "endpoint")}
          >
            <code>{endpoint}</code>
            {copiedKey === "endpoint" ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
          </div>
        </div>

        <div className="mcp-stat-item">
          <span className="mcp-stat-label">Auto-Connect CLI</span>
          <div className="copyable-endpoint" onClick={() => handleCopy("devflow mcp connect all", "connect-cmd")}>
            <code>devflow mcp connect all</code>
            {copiedKey === "connect-cmd" ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
          </div>
        </div>

        <div className="mcp-stat-item">
          <span className="mcp-stat-label">Tools Available</span>
          <div className="mcp-stat-value">
            <Wrench size={12} color="#38bdf8" />
            <span className="font-mono">{status?.tools_count ?? FALLBACK_TOOLS.length} registered</span>
          </div>
        </div>
      </div>

      {/* 3. Sub-Nav Tabs */}
      <div className="mcp-sub-nav">
        <button className={`mcp-tab-btn ${activeSubTab === "all" ? "active" : ""}`} onClick={() => setActiveSubTab("all")}>
          <Layers size={14} />
          <span>Overview</span>
        </button>
        <button className={`mcp-tab-btn ${activeSubTab === "tools" ? "active" : ""}`} onClick={() => setActiveSubTab("tools")}>
          <Wrench size={14} />
          <span>Tools ({FALLBACK_TOOLS.length})</span>
        </button>
        <button className={`mcp-tab-btn ${activeSubTab === "logs" ? "active" : ""}`} onClick={() => setActiveSubTab("logs")}>
          <Activity size={14} />
          <span>Activity</span>
          {logs.length > 0 && <span className="tab-badge">{logs.length}</span>}
        </button>
        <div className="mcp-nav-divider" />
        <button className={`mcp-tab-btn ${activeSubTab === "claude" ? "active" : ""}`} onClick={() => setActiveSubTab("claude")}>
          <Sparkles size={14} color="#d97706" />
          <span>Claude</span>
        </button>
        <button className={`mcp-tab-btn ${activeSubTab === "cursor" ? "active" : ""}`} onClick={() => setActiveSubTab("cursor")}>
          <Code2 size={14} color="#38bdf8" />
          <span>Cursor</span>
        </button>
        <button className={`mcp-tab-btn ${activeSubTab === "antigravity" ? "active" : ""}`} onClick={() => setActiveSubTab("antigravity")}>
          <Bot size={14} color="#a855f7" />
          <span>Antigravity</span>
        </button>
        <button className={`mcp-tab-btn ${activeSubTab === "vscode" ? "active" : ""}`} onClick={() => setActiveSubTab("vscode")}>
          <Code2 size={14} color="#60a5fa" />
          <span>VS Code</span>
        </button>
      </div>

      {/* 4. Tab Contents */}
      <div className="mcp-content-area">

        {/* VIEW: All Overview */}
        {activeSubTab === "all" && (
          <div className="mcp-all-overview">
            <div className="section-title">
              <Sparkles size={15} color="#c084fc" />
              <span>AI Agent Configurations — 1-Click Copy or Connect</span>
            </div>
            <div className="mcp-configs-grid">
              {configsList.map((cfg) => (
                <div key={cfg.id} className="mcp-config-card-compact">
                  <div className="cfg-card-header">
                    <div className="cfg-card-title-group">
                      {cfg.icon}
                      <span className="cfg-card-name">{cfg.name}</span>
                    </div>
                    <button
                      className="btn-glass btn-sm"
                      onClick={() => handleCopy(cfg.code, `cfg-${cfg.id}`)}
                    >
                      {copiedKey === `cfg-${cfg.id}` ? (
                        <><Check size={12} color="#10b981" /><span>Copied!</span></>
                      ) : (
                        <><Copy size={12} /><span>Copy JSON</span></>
                      )}
                    </button>
                  </div>
                  <span className="cfg-card-file truncate" title={cfg.file}>{cfg.file}</span>
                  <div className="cfg-code-preview">
                    <pre>{cfg.code}</pre>
                  </div>
                </div>
              ))}
            </div>

            {/* Tools summary in overview */}
            <div className="section-title" style={{ marginTop: "28px" }}>
              <Wrench size={15} color="#38bdf8" />
              <span>Registered Tools Catalog ({FALLBACK_TOOLS.length} Tools)</span>
            </div>
            <div className="mcp-tools-grid">
              {filteredTools.map((tool) => {
                const isExpanded = expandedTool === tool.name;
                const props = tool.inputSchema?.properties || {};
                const required = tool.inputSchema?.required || [];
                return (
                  <div key={tool.name} className={`mcp-tool-card ${isExpanded ? "expanded" : ""}`}>
                    <div className="mcp-tool-card-header" onClick={() => setExpandedTool(isExpanded ? null : tool.name)}>
                      <div className="mcp-tool-header-left">
                        <Wrench size={14} color="#38bdf8" />
                        <span className="mcp-tool-name">{tool.name}</span>
                      </div>
                      <div className="mcp-tool-header-right">
                        <span className="mcp-params-count">{Object.keys(props).length} params</span>
                        {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      </div>
                    </div>
                    <p className="mcp-tool-desc">{tool.description}</p>
                    {isExpanded && (
                      <div className="mcp-tool-params-expanded">
                        <span className="params-title">Parameters:</span>
                        {Object.keys(props).length === 0 ? (
                          <span className="no-params-hint">No input parameters required</span>
                        ) : (
                          <div className="params-table">
                            {Object.entries(props).map(([pName, pVal]) => {
                              const isReq = required.includes(pName);
                              const pv = pVal as { type?: string; description?: string };
                              return (
                                <div key={pName} className="param-row">
                                  <div className="param-name-cell">
                                    <code>{pName}</code>
                                    {isReq
                                      ? <span className="req-pill">required</span>
                                      : <span className="opt-pill">optional</span>}
                                  </div>
                                  <div className="param-info-cell">
                                    <span className="param-type font-mono">{pv.type || "any"}</span>
                                    <span className="param-desc">{pv.description}</span>
                                  </div>
                                </div>
                              );
                            })}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* VIEW: Tools Catalog Dedicated */}
        {activeSubTab === "tools" && (
          <div className="mcp-tools-panel">
            <div className="mcp-tools-filter-bar">
              <div className="search-input-wrapper">
                <Search size={14} color="#64748b" />
                <input
                  type="text"
                  placeholder="Filter tools by name or description…"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="glass-input"
                />
              </div>
              <span className="mcp-tool-count-pill">{filteredTools.length} / {FALLBACK_TOOLS.length} tools</span>
            </div>
            <div className="mcp-tools-grid">
              {filteredTools.map((tool) => {
                const isExpanded = expandedTool === tool.name;
                const props = tool.inputSchema?.properties || {};
                const required = tool.inputSchema?.required || [];
                return (
                  <div key={tool.name} className={`mcp-tool-card ${isExpanded ? "expanded" : ""}`}>
                    <div className="mcp-tool-card-header" onClick={() => setExpandedTool(isExpanded ? null : tool.name)}>
                      <div className="mcp-tool-header-left">
                        <Wrench size={14} color="#38bdf8" />
                        <span className="mcp-tool-name">{tool.name}</span>
                      </div>
                      <div className="mcp-tool-header-right">
                        <span className="mcp-params-count">{Object.keys(props).length} params</span>
                        {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                      </div>
                    </div>
                    <p className="mcp-tool-desc">{tool.description}</p>
                    {isExpanded && (
                      <div className="mcp-tool-params-expanded">
                        <span className="params-title">Parameter Schema:</span>
                        {Object.keys(props).length === 0 ? (
                          <span className="no-params-hint">No input parameters required</span>
                        ) : (
                          <div className="params-table">
                            {Object.entries(props).map(([pName, pVal]) => {
                              const isReq = required.includes(pName);
                              const pv = pVal as { type?: string; description?: string };
                              return (
                                <div key={pName} className="param-row">
                                  <div className="param-name-cell">
                                    <code>{pName}</code>
                                    {isReq
                                      ? <span className="req-pill">required</span>
                                      : <span className="opt-pill">optional</span>}
                                  </div>
                                  <div className="param-info-cell">
                                    <span className="param-type font-mono">{pv.type || "any"}</span>
                                    <span className="param-desc">{pv.description}</span>
                                  </div>
                                </div>
                              );
                            })}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* VIEW: Activity & Traces */}
        {activeSubTab === "logs" && (
          <div className="mcp-logs-panel">
            {/* Filter & Toolbar */}
            <div className="mcp-logs-toolbar">
              <div className="logs-filter-group">
                {/* Agent Filter Chips */}
                <div className="logs-agent-chips">
                  <button
                    className={`agent-chip ${selectedAgent === "all" ? "active" : ""}`}
                    onClick={() => {
                      setSelectedAgent("all");
                    }}
                    title="Show logs from all AI agents"
                  >
                    <Bot size={12} />
                    <span>All Agents</span>
                  </button>
                  {distinctAgents.map((agent) => {
                    const brand = getAgentBrand(agent);
                    const isActive = selectedAgent.toLowerCase() === agent.toLowerCase();
                    return (
                      <button
                        key={agent}
                        className={`agent-chip ${brand.colorClass} ${isActive ? "active" : ""}`}
                        onClick={() => {
                          setSelectedAgent(isActive ? "all" : agent);
                        }}
                        title={`Filter logs for ${agent}`}
                      >
                        {brand.icon}
                        <span>{agent}</span>
                      </button>
                    );
                  })}
                </div>

                {/* Session Filter */}
                <div className="logs-select-wrapper" title="Filter by agent session or target runner session">
                  <Layers size={13} color="#94a3b8" />
                  <select
                    value={selectedSession}
                    onChange={(e) => setSelectedSession(e.target.value)}
                    className="logs-filter-select"
                  >
                    <option value="all">
                      {selectedAgent !== "all"
                        ? `All ${selectedAgent} Sessions (${filteredSessions.length})`
                        : `All Sessions (${sessions.length})`}
                    </option>
                    {filteredSessions.map((sess) => {
                      const displayId =
                        sess.session_id.length > 18
                          ? `${sess.session_id.slice(0, 10)}...${sess.session_id.slice(-6)}`
                          : sess.session_id;
                      return (
                        <option key={sess.session_id} value={sess.session_id}>
                          {sess.client} • {displayId} ({sess.total_calls} calls{sess.last_tool ? ` • ${sess.last_tool}` : ""})
                        </option>
                      );
                    })}
                  </select>
                </div>

                {/* Status Chips */}
                <div className="logs-status-chips">
                  <button
                    className={`status-chip ${statusFilter === "all" ? "active" : ""}`}
                    onClick={() => setStatusFilter("all")}
                  >
                    All ({logs.length})
                  </button>
                  <button
                    className={`status-chip success ${statusFilter === "success" ? "active" : ""}`}
                    onClick={() => setStatusFilter("success")}
                  >
                    <CheckCircle2 size={11} color="#10b981" />
                    <span>Success ({successLogsCount})</span>
                  </button>
                  {failedLogsCount > 0 && (
                    <button
                      className={`status-chip error ${statusFilter === "error" ? "active" : ""}`}
                      onClick={() => setStatusFilter("error")}
                    >
                      <XCircle size={11} color="#ef4444" />
                      <span>Failed ({failedLogsCount})</span>
                    </button>
                  )}
                </div>

                {/* Search Bar */}
                <div className="logs-search-wrapper">
                  <Search size={13} color="#94a3b8" />
                  <input
                    type="text"
                    placeholder="Search logs, tools, payload..."
                    value={logSearchQuery}
                    onChange={(e) => setLogSearchQuery(e.target.value)}
                    className="logs-search-input"
                  />
                  {logSearchQuery && (
                    <button className="clear-search-btn" onClick={() => setLogSearchQuery("")}>
                      <X size={12} />
                    </button>
                  )}
                </div>
              </div>

              <div className="logs-actions-group">
                <button
                  className={`btn-ghost btn-sm ${isLogsRefreshing ? "spinning" : ""}`}
                  onClick={() => { loadLogs(); loadSessions(); }}
                  title="Refresh logs from SQLite"
                >
                  <RotateCw size={13} />
                  <span>Refresh</span>
                </button>
                {confirmingClear ? (
                  <div className="clear-confirm-group">
                    <button
                      className="btn-danger-solid btn-sm"
                      onClick={handleClearLogs}
                      disabled={isClearing}
                      title="Click to permanently clear logs"
                    >
                      <AlertCircle size={13} />
                      <span>Confirm Clear?</span>
                    </button>
                    <button
                      className="btn-ghost btn-sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setConfirmingClear(false);
                      }}
                      title="Cancel"
                    >
                      <X size={12} />
                    </button>
                  </div>
                ) : (
                  <button
                    className="btn-danger-ghost btn-sm"
                    onClick={handleClearLogs}
                    disabled={isClearing || logs.length === 0}
                    title={selectedSession !== "all" ? "Clear this session's logs" : "Clear all logs"}
                  >
                    <Trash2 size={13} />
                    <span>{selectedSession !== "all" ? "Clear Session" : "Clear All"}</span>
                  </button>
                )}
                <span className="logs-live-indicator">
                  <Radio size={12} color="#10b981" className="animate-pulse" />
                  <span>Live</span>
                </span>
              </div>
            </div>

            {/* Logs List or Empty State */}
            {logs.length === 0 ? (
              <div className="empty-card-container" style={{ minHeight: "300px" }}>
                <Activity size={32} color="#64748b" />
                <div className="empty-title">
                  {selectedSession !== "all" || statusFilter !== "all" || logSearchQuery
                    ? "No Logs Match Filter"
                    : "No MCP Invocations Yet"}
                </div>
                <p className="empty-desc">
                  {selectedSession !== "all" || statusFilter !== "all" || logSearchQuery
                    ? "Try adjusting the session selector, status filter, or search query."
                    : "When AI agents invoke DevFlow tools, live traces, durations, input parameters, and output responses will appear here."}
                </p>
                {selectedAgent !== "all" || selectedSession !== "all" || statusFilter !== "all" || logSearchQuery ? (
                  <button
                    className="btn-secondary"
                    onClick={() => {
                      setSelectedAgent("all");
                      setSelectedSession("all");
                      setStatusFilter("all");
                      setLogSearchQuery("");
                    }}
                  >
                    Reset Filters
                  </button>
                ) : (
                  <button
                    className="btn-primary"
                    onClick={() => handleCopy("devflow mcp connect all", "empty-connect")}
                  >
                    <Copy size={13} />
                    <span>{copiedKey === "empty-connect" ? "Copied!" : "Copy Auto-Connect Command"}</span>
                  </button>
                )}
              </div>
            ) : (
              <div className="mcp-logs-list">
                {logs.map((log) => {
                  const isSuccess = log.status === "success";
                  const isExpanded = expandedLogId === log.id;
                  const label = log.tool_name || log.method;
                  return (
                    <div
                      key={log.id}
                      className={`mcp-log-item ${isSuccess ? "success" : "failure"} ${isExpanded ? "expanded" : ""}`}
                      onClick={() => {
                        if (window.getSelection() && window.getSelection()!.toString().length > 0) return;
                        setExpandedLogId(isExpanded ? null : log.id);
                      }}
                    >
                      {/* Compact / Short Header Row */}
                      <div className="log-item-main">
                        <div className="log-item-left">
                          {isSuccess ? <CheckCircle2 size={16} color="#10b981" /> : <XCircle size={16} color="#ef4444" />}
                          <span className="log-tool-name">{label}</span>
                          <span
                            className={`log-client-pill client-badge-${getAgentBrand(log.client).colorClass}`}
                            title={`AI Agent: ${log.client} (Click to filter)`}
                            onClick={(e) => {
                              e.stopPropagation();
                              setSelectedAgent(selectedAgent.toLowerCase() === log.client.toLowerCase() ? "all" : log.client);
                            }}
                          >
                            {getAgentBrand(log.client).icon}
                            <span>{log.client}</span>
                          </span>
                          {log.session_id && (
                            <span
                              className="log-session-pill"
                              title={`Session: ${log.session_id} (Click to filter)`}
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedSession(log.session_id!);
                              }}
                            >
                              sess: {log.session_id.length > 12 ? `${log.session_id.slice(0, 10)}...` : log.session_id}
                            </span>
                          )}
                          <span className="log-summary-text" title={log.summary}>{log.summary}</span>
                        </div>
                        <div className="log-item-right">
                          <span className="log-duration-badge">{log.duration_ms}ms</span>
                          <span className="log-time-stamp">{new Date(log.timestamp).toLocaleTimeString()}</span>
                          <button
                            className="log-copy-btn"
                            title="Copy log entry to clipboard"
                            onClick={async (e) => {
                              e.stopPropagation();
                              const payloadStr = log.arguments ? `\nInput: ${JSON.stringify(log.arguments, null, 2)}` : "";
                              const respStr = log.response ? `\nOutput: ${JSON.stringify(log.response, null, 2)}` : "";
                              const errStr = log.error_message ? `\nError: ${log.error_message}` : "";
                              const text = `[${log.timestamp}] [${log.client}] ${label} (${log.duration_ms}ms) - ${log.status}\nSummary: ${log.summary}${errStr}${payloadStr}${respStr}`;
                              const ok = await copyToClipboard(text);
                              if (ok) {
                                showToast("success", "Log entry copied to clipboard");
                              }
                            }}
                          >
                            <Copy size={13} />
                          </button>
                          <button
                            className="log-trash-btn"
                            title="Delete this log entry"
                            onClick={(e) => handleDeleteLog(e, log.id)}
                            disabled={isDeletingId === log.id}
                          >
                            <Trash2 size={13} />
                          </button>
                          {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                        </div>
                      </div>

                      {/* Expanded Details: Dual Input / Output Panels */}
                      {isExpanded && (
                        <div className="log-expanded-details" onClick={(e) => e.stopPropagation()}>
                          {log.error_message && (
                            <div className="log-error-banner">
                              <strong>Error:</strong> {log.error_message}
                            </div>
                          )}

                          {/* Session & Target Context if present */}
                          <div className="log-context-strip">
                            <div className="context-item">
                              <span className="context-label">Agent:</span>
                              <span className={`log-client-pill client-badge-${getAgentBrand(log.client).colorClass}`}>
                                {getAgentBrand(log.client).icon}
                                <span>{log.client}</span>
                              </span>
                            </div>
                            {log.session_id && (
                              <div className="context-item">
                                <span className="context-label">Session ID:</span>
                                <code>{log.session_id}</code>
                                  <button
                                    className="btn-icon-tiny"
                                    onClick={() => handleCopy(log.session_id!, `sess-${log.id}`)}
                                    title="Copy Session ID"
                                  >
                                    {copiedKey === `sess-${log.id}` ? <Check size={11} color="#10b981" /> : <Copy size={11} />}
                                  </button>
                                </div>
                              )}
                              {log.project_path && (
                                <div className="context-item">
                                  <span className="context-label">Project:</span>
                                  <code>{log.project_path}</code>
                                </div>
                              )}
                            </div>

                          {/* Dual Payload Viewer: Input Parameters & Output Response */}
                          <div className="log-dual-payloads">
                            {/* Input Parameters */}
                            <div className="log-payload-panel">
                              <div className="log-payload-panel-header">
                                <span className="payload-panel-title">📥 Input Parameters</span>
                                {log.arguments ? (
                                  <button
                                    className="btn-copy-tiny"
                                    onClick={() => handleCopy(JSON.stringify(log.arguments, null, 2), `args-${log.id}`)}
                                  >
                                    {copiedKey === `args-${log.id}` ? (
                                      <><Check size={11} color="#10b981" /><span>Copied</span></>
                                    ) : (
                                      <><Copy size={11} /><span>Copy</span></>
                                    )}
                                  </button>
                                ) : null}
                              </div>
                              <pre className="payload-code input-code">
                                {log.arguments
                                  ? JSON.stringify(log.arguments, null, 2)
                                  : "// No arguments provided"}
                              </pre>
                            </div>

                            {/* Output Response */}
                            <div className="log-payload-panel">
                              <div className="log-payload-panel-header">
                                <span className="payload-panel-title">📤 Output Response</span>
                                {log.response ? (
                                  <button
                                    className="btn-copy-tiny"
                                    onClick={() => handleCopy(JSON.stringify(log.response, null, 2), `resp-${log.id}`)}
                                  >
                                    {copiedKey === `resp-${log.id}` ? (
                                      <><Check size={11} color="#10b981" /><span>Copied</span></>
                                    ) : (
                                      <><Copy size={11} /><span>Copy</span></>
                                    )}
                                  </button>
                                ) : null}
                              </div>
                              <pre className="payload-code output-code">
                                {log.response
                                  ? JSON.stringify(log.response, null, 2)
                                  : "// No output response recorded"}
                              </pre>
                            </div>
                          </div>

                          {/* Footer Meta */}
                          <div className="log-raw-meta">
                            <span>ID: <code>{log.id}</code></span>
                            <span>Timestamp: <code>{log.timestamp}</code></span>
                            <button
                              className="btn-delete-link"
                              onClick={(e) => handleDeleteLog(e, log.id)}
                            >
                              <Trash2 size={11} />
                              <span>Delete Entry</span>
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}

        {/* VIEWS: Individual Config Tabs (Claude, Cursor, Antigravity, VS Code) */}
        {["claude", "cursor", "antigravity", "vscode"].includes(activeSubTab) && (
          <div className="mcp-config-panel">
            {configsList
              .filter((c) => c.id === activeSubTab)
              .map((cfg) => (
                <div key={cfg.id} className="config-hero-card">
                  <div className="config-hero-header">
                    <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                      {cfg.icon}
                      <div>
                        <h3 className="config-title">{cfg.name} Integration</h3>
                        <p className="config-subtitle">Config file: {cfg.file}</p>
                      </div>
                    </div>
                    <button
                      className="btn-primary"
                      onClick={() => handleCopy(cfg.code, `config-${cfg.id}`)}
                    >
                      {copiedKey === `config-${cfg.id}` ? (
                        <><Check size={14} color="#10b981" /><span>Copied!</span></>
                      ) : (
                        <><Copy size={14} /><span>Copy Config JSON</span></>
                      )}
                    </button>
                  </div>

                  <div className="config-code-block">
                    <pre>{cfg.code}</pre>
                  </div>

                  <div className="config-instructions">
                    <h4>1-Click CLI Auto-Connect</h4>
                    <p>Run this in your terminal to auto-install into {cfg.name}:</p>
                    <div
                      className="copyable-endpoint"
                      onClick={() => handleCopy(`devflow mcp connect ${cfg.id}`, `cli-${cfg.id}`)}
                      style={{ width: "fit-content", marginTop: "6px" }}
                    >
                      <code>devflow mcp connect {cfg.id}</code>
                      {copiedKey === `cli-${cfg.id}` ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
                    </div>
                  </div>
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
};
