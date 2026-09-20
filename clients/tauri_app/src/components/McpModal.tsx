import React, { useState, useEffect } from "react";
import {
  Bot,
  X,
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
  Layers,
  Activity,
  RotateCw,
} from "lucide-react";
import type { McpStatusResponse, McpAccessLogEntry } from "../types";

interface McpModalProps {
  isOpen: boolean;
  onClose: () => void;
  onFetchStatus: () => Promise<McpStatusResponse>;
  onToggleServer: (enabled: boolean) => Promise<{ success: boolean; enabled: boolean }>;
  onFetchLogs: (limit?: number) => Promise<McpAccessLogEntry[]>;
}

export const McpModal: React.FC<McpModalProps> = ({
  isOpen,
  onClose,
  onFetchStatus,
  onToggleServer,
  onFetchLogs,
}) => {
  const [status, setStatus] = useState<McpStatusResponse | null>(null);
  const [logs, setLogs] = useState<McpAccessLogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isToggling, setIsToggling] = useState(false);
  const [activeTab, setActiveTab] = useState<"logs" | "tools" | "claude" | "cursor" | "antigravity" | "vscode">("logs");
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedTool, setExpandedTool] = useState<string | null>(null);
  const [expandedLogId, setExpandedLogId] = useState<string | null>(null);

  const loadStatusAndLogs = async () => {
    setIsLoading(true);
    try {
      const [statusData, logsData] = await Promise.all([
        onFetchStatus(),
        onFetchLogs(100),
      ]);
      setStatus(statusData);
      setLogs(logsData || []);
    } catch (e) {
      console.error("Failed to load MCP status or logs:", e);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      loadStatusAndLogs();
      const interval = setInterval(async () => {
        try {
          const freshLogs = await onFetchLogs(100);
          setLogs(freshLogs || []);
        } catch (_) {}
      }, 3000);
      return () => clearInterval(interval);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleToggle = async () => {
    if (!status || isToggling) return;
    setIsToggling(true);
    try {
      const targetState = !status.enabled;
      await onToggleServer(targetState);
      setStatus((prev) => (prev ? { ...prev, enabled: targetState } : null));
    } catch (e) {
      console.error("Failed to toggle MCP server:", e);
    } finally {
      setIsToggling(false);
    }
  };

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedKey(key);
    setTimeout(() => setCopiedKey(null), 1800);
  };

  const filteredTools = (status?.tools || []).filter((t) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      t.name.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q)
    );
  });

  const getActiveConfigCode = () => {
    if (!status?.configs) return "";
    switch (activeTab) {
      case "claude":
        return JSON.stringify(status.configs.claude_desktop, null, 2);
      case "cursor":
        return JSON.stringify(status.configs.cursor, null, 2);
      case "antigravity":
        return JSON.stringify(status.configs.antigravity, null, 2);
      case "vscode":
        return JSON.stringify(status.configs.vscode, null, 2);
      default:
        return "";
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content mcp-modal-box"
        onClick={(e) => e.stopPropagation()}
        style={{
          width: "740px",
          maxWidth: "94vw",
          maxHeight: "88vh",
          display: "flex",
          flexDirection: "column",
          background: "var(--bg-panel, #0f172a)",
          borderRadius: "10px",
          border: "1px solid rgba(255, 255, 255, 0.08)",
          boxShadow: "0 20px 40px rgba(0, 0, 0, 0.6)",
          overflow: "hidden",
        }}
      >
        {/* Modal Header */}
        <div
          className="modal-header"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "14px 18px",
            borderBottom: "1px solid rgba(255, 255, 255, 0.07)",
            background: "rgba(15, 23, 42, 0.8)",
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <div
              style={{
                width: "28px",
                height: "28px",
                borderRadius: "6px",
                background: "rgba(168, 85, 247, 0.15)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: "1px solid rgba(168, 85, 247, 0.3)",
              }}
            >
              <Bot size={16} color="#c084fc" />
            </div>
            <div>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <h2 style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)" }}>
                  DevFlow Model Context Protocol (MCP) Hub
                </h2>
                <span
                  style={{
                    fontSize: "10.5px",
                    fontWeight: 600,
                    padding: "2px 7px",
                    borderRadius: "10px",
                    background: status?.enabled
                      ? "rgba(16, 185, 129, 0.15)"
                      : "rgba(239, 68, 68, 0.15)",
                    color: status?.enabled ? "#10b981" : "#ef4444",
                    border: `1px solid ${
                      status?.enabled
                        ? "rgba(16, 185, 129, 0.3)"
                        : "rgba(239, 68, 68, 0.3)"
                    }`,
                    display: "flex",
                    alignItems: "center",
                    gap: "4px",
                  }}
                >
                  {status?.enabled ? <CheckCircle2 size={11} /> : <XCircle size={11} />}
                  {status?.enabled ? `ONLINE (PORT ${status.port || 9292})` : "OFFLINE"}
                </span>
              </div>
              <p style={{ fontSize: "11.5px", color: "var(--text-muted)", margin: 0, marginTop: "2px" }}>
                Connect Claude Desktop, Cursor, Antigravity, or VS Code to build, run, diagnose, and stream logs.
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              cursor: "pointer",
              padding: "4px",
              borderRadius: "4px",
            }}
          >
            <X size={16} />
          </button>
        </div>

        {/* Modal Body */}
        <div
          className="modal-body"
          style={{
            padding: "16px 18px",
            overflowY: "auto",
            display: "flex",
            flexDirection: "column",
            gap: "14px",
          }}
        >
          {/* Server Control & Quick Connect Banner */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "12px 14px",
              background: "rgba(15, 23, 42, 0.6)",
              border: "1px solid rgba(255, 255, 255, 0.06)",
              borderRadius: "8px",
            }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                <span style={{ fontSize: "12px", fontWeight: 600, color: "var(--text-primary)" }}>
                  JSON-RPC Endpoint:
                </span>
                <code
                  style={{
                    fontSize: "11px",
                    fontFamily: "monospace",
                    background: "rgba(255, 255, 255, 0.05)",
                    padding: "2px 6px",
                    borderRadius: "4px",
                    color: "#38bdf8",
                  }}
                >
                  {status?.http_endpoint || "http://localhost:9292/rpc"}
                </code>
                <button
                  onClick={() => handleCopy(status?.http_endpoint || "http://localhost:9292/rpc", "endpoint")}
                  style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    padding: "2px",
                  }}
                  title="Copy HTTP Endpoint"
                >
                  {copiedKey === "endpoint" ? <Check size={12} color="#10b981" /> : <Copy size={12} />}
                </button>
              </div>

              <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                  Auto-Connect CLI:
                </span>
                <code
                  style={{
                    fontSize: "10.5px",
                    fontFamily: "monospace",
                    color: "#c084fc",
                    background: "rgba(168, 85, 247, 0.08)",
                    padding: "1px 6px",
                    borderRadius: "3px",
                  }}
                >
                  devflow mcp connect all
                </code>
                <button
                  onClick={() => handleCopy("devflow mcp connect all", "cli_connect")}
                  style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    padding: "2px",
                  }}
                  title="Copy Auto-Connect CLI Command"
                >
                  {copiedKey === "cli_connect" ? <Check size={11} color="#10b981" /> : <Copy size={11} />}
                </button>
              </div>
            </div>

            <button
              onClick={handleToggle}
              disabled={isToggling || isLoading}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "6px",
                padding: "6px 14px",
                borderRadius: "6px",
                fontSize: "12px",
                fontWeight: 600,
                cursor: "pointer",
                transition: "all 0.15s ease",
                border: status?.enabled
                  ? "1px solid rgba(239, 68, 68, 0.3)"
                  : "1px solid rgba(16, 185, 129, 0.4)",
                background: status?.enabled
                  ? "rgba(239, 68, 68, 0.15)"
                  : "rgba(16, 185, 129, 0.18)",
                color: status?.enabled ? "#ef4444" : "#10b981",
              }}
            >
              <Power size={13} />
              <span>{isToggling ? "Updating..." : status?.enabled ? "Disable Server" : "Enable Server"}</span>
            </button>
          </div>

          {/* Navigation Tabs */}
          <div
            style={{
              display: "flex",
              gap: "6px",
              borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
              paddingBottom: "8px",
            }}
          >
            {[
              { id: "logs", label: `Activity Logs (${logs.length})`, icon: <Activity size={12} /> },
              { id: "tools", label: `MCP Tools (${status?.tools_count || 11})`, icon: <Wrench size={12} /> },
              { id: "claude", label: "Claude Desktop", icon: <Code2 size={12} /> },
              { id: "cursor", label: "Cursor", icon: <Code2 size={12} /> },
              { id: "antigravity", label: "Antigravity", icon: <Sparkles size={12} /> },
              { id: "vscode", label: "VS Code", icon: <Layers size={12} /> },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "5px",
                  padding: "5px 11px",
                  borderRadius: "5px",
                  fontSize: "11.5px",
                  fontWeight: 500,
                  cursor: "pointer",
                  border:
                    activeTab === tab.id
                      ? "1px solid rgba(168, 85, 247, 0.4)"
                      : "1px solid rgba(255, 255, 255, 0.05)",
                  background:
                    activeTab === tab.id
                      ? "rgba(168, 85, 247, 0.15)"
                      : "rgba(255, 255, 255, 0.02)",
                  color: activeTab === tab.id ? "#c084fc" : "var(--text-muted)",
                }}
              >
                {tab.icon}
                <span>{tab.label}</span>
              </button>
            ))}
          </div>

          {/* Tab 1: Activity & Traces Log Tab */}
          {activeTab === "logs" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                  <span style={{ fontSize: "12px", fontWeight: 600, color: "var(--text-secondary)" }}>
                    AI Agent Invocation History:
                  </span>
                  <span
                    style={{
                      fontSize: "10px",
                      background: "rgba(16, 185, 129, 0.15)",
                      color: "#10b981",
                      padding: "1px 6px",
                      borderRadius: "10px",
                    }}
                  >
                    Live SSE Stream
                  </span>
                </div>

                <button
                  onClick={async () => {
                    const fresh = await onFetchLogs(100);
                    setLogs(fresh || []);
                  }}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "4px",
                    background: "rgba(255, 255, 255, 0.04)",
                    border: "1px solid rgba(255, 255, 255, 0.08)",
                    borderRadius: "4px",
                    padding: "3px 8px",
                    color: "var(--text-muted)",
                    fontSize: "11px",
                    cursor: "pointer",
                  }}
                >
                  <RotateCw size={11} />
                  <span>Refresh</span>
                </button>
              </div>

              {logs.length === 0 ? (
                <div
                  style={{
                    padding: "30px 20px",
                    textAlign: "center",
                    background: "rgba(255, 255, 255, 0.02)",
                    borderRadius: "6px",
                    border: "1px dashed rgba(255, 255, 255, 0.08)",
                  }}
                >
                  <Bot size={28} color="var(--text-muted)" style={{ margin: "0 auto 8px auto" }} />
                  <div style={{ fontSize: "12.5px", fontWeight: 600, color: "var(--text-primary)" }}>
                    No AI Agent Invocations Yet
                  </div>
                  <p style={{ fontSize: "11.5px", color: "var(--text-muted)", margin: "4px 0 0 0" }}>
                    Run <code>devflow mcp connect all</code> and ask Claude or Cursor to inspect doctor, list devices, or run builds to see live traces here.
                  </p>
                </div>
              ) : (
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "6px",
                    maxHeight: "340px",
                    overflowY: "auto",
                  }}
                >
                  {logs.map((log) => {
                    const isExpanded = expandedLogId === log.id;
                    const isSuccess = log.status === "success";
                    const timeStr = log.timestamp
                      ? new Date(log.timestamp).toLocaleTimeString()
                      : "";

                    return (
                      <div
                        key={log.id}
                        style={{
                          background: "rgba(255, 255, 255, 0.02)",
                          border: "1px solid rgba(255, 255, 255, 0.06)",
                          borderRadius: "6px",
                          overflow: "hidden",
                        }}
                      >
                        <div
                          onClick={() => setExpandedLogId(isExpanded ? null : log.id)}
                          style={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                            padding: "8px 12px",
                            cursor: "pointer",
                            background: isExpanded ? "rgba(255, 255, 255, 0.04)" : "transparent",
                          }}
                        >
                          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                            {isSuccess ? (
                              <CheckCircle2 size={13} color="#10b981" />
                            ) : (
                              <XCircle size={13} color="#ef4444" />
                            )}
                            <code
                              style={{
                                fontSize: "11.5px",
                                fontWeight: 600,
                                color: "#38bdf8",
                                fontFamily: "monospace",
                              }}
                            >
                              {log.tool_name || log.method}
                            </code>
                            <span style={{ fontSize: "11.5px", color: "var(--text-secondary)" }}>
                              {log.summary}
                            </span>
                          </div>

                          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                            <span
                              style={{
                                fontSize: "10px",
                                padding: "1px 5px",
                                borderRadius: "4px",
                                background: "rgba(255, 255, 255, 0.05)",
                                color: "var(--text-muted)",
                              }}
                            >
                              {log.duration_ms}ms
                            </span>
                            <span style={{ fontSize: "10.5px", color: "var(--text-muted)" }}>
                              {timeStr}
                            </span>
                            {isExpanded ? (
                              <ChevronDown size={13} color="var(--text-muted)" />
                            ) : (
                              <ChevronRight size={13} color="var(--text-muted)" />
                            )}
                          </div>
                        </div>

                        {isExpanded && (
                          <div
                            style={{
                              padding: "10px 14px",
                              borderTop: "1px solid rgba(255, 255, 255, 0.05)",
                              background: "rgba(15, 23, 42, 0.4)",
                              fontSize: "11px",
                            }}
                          >
                            <div style={{ fontWeight: 600, color: "var(--text-muted)", marginBottom: "4px" }}>
                              Arguments:
                            </div>
                            <pre
                              style={{
                                background: "rgba(0, 0, 0, 0.3)",
                                padding: "8px",
                                borderRadius: "4px",
                                fontFamily: "monospace",
                                color: "#38bdf8",
                                overflowX: "auto",
                                margin: 0,
                              }}
                            >
                              {JSON.stringify(log.arguments || {}, null, 2)}
                            </pre>
                            {log.error_message && (
                              <div style={{ marginTop: "8px", color: "#f87171" }}>
                                <strong>Error:</strong> {log.error_message}
                              </div>
                            )}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}

          {/* Tab 2: Tools Explorer */}
          {activeTab === "tools" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "6px",
                  background: "rgba(15, 23, 42, 0.6)",
                  border: "1px solid rgba(255, 255, 255, 0.08)",
                  borderRadius: "6px",
                  padding: "4px 10px",
                }}
              >
                <Search size={13} color="var(--text-muted)" />
                <input
                  type="text"
                  placeholder="Search MCP tools by name or description..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  style={{
                    background: "transparent",
                    border: "none",
                    outline: "none",
                    color: "var(--text-primary)",
                    fontSize: "12px",
                    width: "100%",
                  }}
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery("")}
                    style={{
                      background: "transparent",
                      border: "none",
                      color: "var(--text-muted)",
                      cursor: "pointer",
                      fontSize: "11px",
                    }}
                  >
                    ✕
                  </button>
                )}
              </div>

              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "8px",
                  maxHeight: "340px",
                  overflowY: "auto",
                  paddingRight: "2px",
                }}
              >
                {filteredTools.map((tool) => {
                  const isExpanded = expandedTool === tool.name;
                  const properties = tool.inputSchema?.properties || {};
                  const required = tool.inputSchema?.required || [];

                  return (
                    <div
                      key={tool.name}
                      style={{
                        background: "rgba(255, 255, 255, 0.02)",
                        border: "1px solid rgba(255, 255, 255, 0.06)",
                        borderRadius: "6px",
                        overflow: "hidden",
                      }}
                    >
                      <div
                        onClick={() => setExpandedTool(isExpanded ? null : tool.name)}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "space-between",
                          padding: "8px 12px",
                          cursor: "pointer",
                          background: isExpanded ? "rgba(255, 255, 255, 0.04)" : "transparent",
                        }}
                      >
                        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                          {isExpanded ? (
                            <ChevronDown size={14} color="var(--text-muted)" />
                          ) : (
                            <ChevronRight size={14} color="var(--text-muted)" />
                          )}
                          <code
                            style={{
                              fontSize: "12px",
                              fontWeight: 600,
                              color: "#38bdf8",
                              fontFamily: "monospace",
                            }}
                          >
                            {tool.name}
                          </code>
                          <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                            — {tool.description}
                          </span>
                        </div>

                        <span
                          style={{
                            fontSize: "10px",
                            color: "var(--text-muted)",
                            background: "rgba(255, 255, 255, 0.05)",
                            padding: "1px 6px",
                            borderRadius: "4px",
                          }}
                        >
                          {Object.keys(properties).length} params
                        </span>
                      </div>

                      {isExpanded && (
                        <div
                          style={{
                            padding: "10px 14px",
                            borderTop: "1px solid rgba(255, 255, 255, 0.05)",
                            background: "rgba(15, 23, 42, 0.4)",
                            fontSize: "11.5px",
                          }}
                        >
                          <div style={{ fontWeight: 600, color: "var(--text-secondary)", marginBottom: "6px" }}>
                            Parameters:
                          </div>
                          {Object.keys(properties).length === 0 ? (
                            <span style={{ color: "var(--text-muted)", fontStyle: "italic" }}>
                              No input parameters required
                            </span>
                          ) : (
                            <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                              {Object.entries(properties).map(([propName, propDef]) => {
                                const isReq = required.includes(propName);
                                return (
                                  <div
                                    key={propName}
                                    style={{
                                      display: "flex",
                                      alignItems: "flex-start",
                                      gap: "8px",
                                      background: "rgba(255, 255, 255, 0.02)",
                                      padding: "4px 8px",
                                      borderRadius: "4px",
                                    }}
                                  >
                                    <code style={{ color: isReq ? "#fb923c" : "#94a3b8", fontWeight: 600 }}>
                                      {propName}
                                    </code>
                                    <span style={{ color: "var(--text-muted)", fontSize: "10.5px" }}>
                                      ({propDef.type || "string"}{isReq ? ", required" : ", optional"}):
                                    </span>
                                    <span style={{ color: "var(--text-secondary)" }}>
                                      {propDef.description || ""}
                                    </span>
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

          {/* Tab 3: Config Generators */}
          {activeTab !== "logs" && activeTab !== "tools" && (
            <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
                  Paste this configuration into your <strong>{activeTab.toUpperCase()}</strong> settings file:
                </span>
                <button
                  onClick={() => handleCopy(getActiveConfigCode(), activeTab)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "5px",
                    padding: "4px 10px",
                    background: "rgba(168, 85, 247, 0.2)",
                    border: "1px solid rgba(168, 85, 247, 0.4)",
                    borderRadius: "4px",
                    color: "#c084fc",
                    fontSize: "11px",
                    fontWeight: 600,
                    cursor: "pointer",
                  }}
                >
                  {copiedKey === activeTab ? <Check size={12} /> : <Copy size={12} />}
                  <span>{copiedKey === activeTab ? "Copied!" : "Copy Configuration"}</span>
                </button>
              </div>

              <pre
                style={{
                  background: "#080d1a",
                  border: "1px solid rgba(255, 255, 255, 0.08)",
                  borderRadius: "6px",
                  padding: "12px 14px",
                  fontSize: "12px",
                  fontFamily: "monospace",
                  color: "#34d399",
                  overflowX: "auto",
                  lineHeight: "1.4",
                  maxHeight: "300px",
                }}
              >
                {getActiveConfigCode()}
              </pre>
            </div>
          )}
        </div>

        {/* Modal Footer */}
        <div
          className="modal-footer"
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            padding: "12px 18px",
            borderTop: "1px solid rgba(255, 255, 255, 0.06)",
            background: "rgba(15, 23, 42, 0.8)",
          }}
        >
          <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
            Protocol Version: <strong>2024-11-05</strong> • Tools: <strong>11 Available</strong>
          </span>

          <button
            onClick={onClose}
            style={{
              padding: "6px 16px",
              background: "rgba(255, 255, 255, 0.08)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              borderRadius: "5px",
              color: "var(--text-primary)",
              fontSize: "12px",
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
