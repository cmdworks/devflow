import React, { useState, useEffect, useCallback } from "react";
import {
  Folder,
  FolderPlus,
  FolderOpen,
  Clipboard,
  ArrowRight,
  Trash2,
  X,
  Layers,
  Clock,
  Check,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { useDevFlowApi } from "../hooks/useDevFlowApi";
import type { KnownWorkspace } from "../types";

interface WorkspaceModalProps {
  currentPath: string;
  isOpen: boolean;
  onClose: () => void;
  onSelectWorkspace: (path: string) => void;
}

export const WorkspaceModal: React.FC<WorkspaceModalProps> = ({
  currentPath,
  isOpen,
  onClose,
  onSelectWorkspace,
}) => {
  const api = useDevFlowApi();
  const [workspaces, setWorkspaces] = useState<KnownWorkspace[]>([]);
  const [newPathInput, setNewPathInput] = useState<string>("");
  const [isListLoading, setIsListLoading] = useState<boolean>(false);
  const [isOpening, setIsOpening] = useState<boolean>(false);
  const [isPickingFolder, setIsPickingFolder] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const loadWorkspaces = useCallback(async () => {
    try {
      setIsListLoading(true);
      const list = await api.fetchWorkspaces();
      setWorkspaces(list || []);
      setErrorMessage(null);
    } catch (err: any) {
      console.error("Failed to load known workspaces:", err);
    } finally {
      setIsListLoading(false);
    }
  }, [api]);

  useEffect(() => {
    if (isOpen) {
      loadWorkspaces();
      setNewPathInput("");
      setErrorMessage(null);
    }
  }, [isOpen, loadWorkspaces]);

  if (!isOpen) return null;

  const handleAddAndOpen = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const trimmed = newPathInput.trim();
    if (!trimmed) {
      setErrorMessage("Please provide a valid workspace directory path.");
      return;
    }

    try {
      setIsOpening(true);
      setErrorMessage(null);
      await api.addWorkspace(trimmed);
      onSelectWorkspace(trimmed);
      onClose();
    } catch (err: any) {
      setErrorMessage(err?.message || "Failed to open directory. Verify path exists.");
    } finally {
      setIsOpening(false);
    }
  };

  const handleBrowseFolder = async () => {
    try {
      setIsPickingFolder(true);
      setErrorMessage(null);
      const res = await api.pickFolder();
      if (res.success && res.path) {
        setNewPathInput(res.path);
        // Automatically attempt to add and open
        try {
          setIsOpening(true);
          await api.addWorkspace(res.path);
          onSelectWorkspace(res.path);
          onClose();
        } catch (err: any) {
          setErrorMessage(err?.message || "Failed to open directory.");
        } finally {
          setIsOpening(false);
        }
      }
    } catch (err: any) {
      console.error("Browse folder error:", err);
    } finally {
      setIsPickingFolder(false);
    }
  };

  const handlePasteClipboard = async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (text) {
        setNewPathInput(text.trim());
        setErrorMessage(null);
      }
    } catch (err) {
      console.warn("Direct clipboard read unavailable:", err);
    }
  };

  const handleRemove = async (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    try {
      await api.removeWorkspace(path);
      setWorkspaces((prev) => prev.filter((w) => w.path !== path));
    } catch (err) {
      console.error("Failed to remove workspace:", err);
    }
  };

  const formatTime = (iso: string) => {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return "";
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content"
        style={{ width: "680px" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Layers size={18} color="#06b6d4" />
            <span className="modal-title">Workspace Manager</span>
          </div>
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              cursor: "pointer",
            }}
          >
            <X size={18} />
          </button>
        </div>

        <div className="modal-body" style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
          {/* Active Workspace Banner */}
          <div
            style={{
              background: "rgba(6, 182, 212, 0.08)",
              border: "1px solid rgba(6, 182, 212, 0.25)",
              borderRadius: "6px",
              padding: "10px 14px",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
              <Folder size={16} color="#06b6d4" />
              <div>
                <div style={{ fontSize: "11px", color: "var(--text-muted)", textTransform: "uppercase", letterSpacing: "0.5px" }}>
                  Active Workspace
                </div>
                <div style={{ fontSize: "13px", fontWeight: 600, color: "#fff", wordBreak: "break-all" }}>
                  {currentPath || "None loaded"}
                </div>
              </div>
            </div>
            <span
              style={{
                fontSize: "11px",
                background: "rgba(16, 185, 129, 0.2)",
                color: "#10b981",
                padding: "2px 8px",
                borderRadius: "4px",
                fontWeight: 600,
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              <Check size={12} /> Active
            </span>
          </div>

          {/* Add / Open New Workspace Form */}
          <form onSubmit={handleAddAndOpen} style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <label style={{ fontSize: "12px", fontWeight: 600, color: "var(--text-secondary)" }}>
                Add or Open Workspace Directory
              </label>
              <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                Accepts full paths, ~ tilde, or project names
              </span>
            </div>

            <div style={{ display: "flex", gap: "8px" }}>
              <div style={{ position: "relative", flex: 1 }}>
                <input
                  type="text"
                  placeholder="/Users/username/Dev/Projects/my-app"
                  value={newPathInput}
                  onChange={(e) => {
                    setNewPathInput(e.target.value);
                    if (errorMessage) setErrorMessage(null);
                  }}
                  onKeyDown={async (e) => {
                    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "v") {
                      try {
                        const text = await navigator.clipboard.readText();
                        if (text) {
                          e.preventDefault();
                          const target = e.currentTarget;
                          const start = target.selectionStart || 0;
                          const end = target.selectionEnd || 0;
                          const val = newPathInput;
                          const next = val.slice(0, start) + text.trim() + val.slice(end);
                          setNewPathInput(next);
                        }
                      } catch (_) {}
                    }
                  }}
                  style={{
                    width: "100%",
                    boxSizing: "border-box",
                    padding: "9px 12px",
                    background: "var(--bg-primary)",
                    border: errorMessage ? "1px solid #ef4444" : "1px solid var(--border-color)",
                    borderRadius: "6px",
                    color: "#fff",
                    fontSize: "13px",
                    fontFamily: "var(--font-mono, monospace)",
                    outline: "none",
                  }}
                  autoFocus
                />
              </div>

              {/* Paste Button */}
              <button
                type="button"
                onClick={handlePasteClipboard}
                title="Paste directory path from clipboard"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "5px",
                  padding: "0 12px",
                  background: "rgba(30, 41, 59, 0.8)",
                  color: "var(--text-secondary)",
                  border: "1px solid var(--border-color)",
                  borderRadius: "6px",
                  fontSize: "12px",
                  fontWeight: 500,
                  cursor: "pointer",
                  whiteSpace: "nowrap",
                }}
              >
                <Clipboard size={14} />
                <span>Paste</span>
              </button>

              {/* Browse Button */}
              <button
                type="button"
                disabled={isPickingFolder}
                onClick={handleBrowseFolder}
                title="Browse folder with macOS native dialog"
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "5px",
                  padding: "0 12px",
                  background: "rgba(6, 182, 212, 0.12)",
                  color: "#06b6d4",
                  border: "1px solid rgba(6, 182, 212, 0.35)",
                  borderRadius: "6px",
                  fontSize: "12px",
                  fontWeight: 600,
                  cursor: isPickingFolder ? "wait" : "pointer",
                  whiteSpace: "nowrap",
                }}
              >
                {isPickingFolder ? <Loader2 size={14} className="animate-spin" /> : <FolderOpen size={14} />}
                <span>Browse...</span>
              </button>

              {/* Open Submit Button */}
              <button
                type="submit"
                disabled={isOpening}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "6px",
                  padding: "0 16px",
                  background: "#0891b2",
                  color: "#fff",
                  border: "none",
                  borderRadius: "6px",
                  fontSize: "13px",
                  fontWeight: 600,
                  cursor: isOpening ? "wait" : "pointer",
                  whiteSpace: "nowrap",
                }}
              >
                {isOpening ? <Loader2 size={14} className="animate-spin" /> : <FolderPlus size={15} />}
                <span>Open</span>
              </button>
            </div>

            {errorMessage && (
              <div style={{ display: "flex", alignItems: "center", gap: "6px", color: "#ef4444", fontSize: "12px" }}>
                <AlertCircle size={13} />
                <span>{errorMessage}</span>
              </div>
            )}
          </form>

          {/* Known / Recent Workspaces List */}
          <div>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                marginBottom: "8px",
              }}
            >
              <span style={{ fontSize: "12px", fontWeight: 600, color: "var(--text-secondary)" }}>
                Recent & Known Workspaces ({workspaces.length})
              </span>
              {isListLoading && (
                <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>Refreshing...</span>
              )}
            </div>

            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "6px",
                maxHeight: "260px",
                overflowY: "auto",
              }}
            >
              {workspaces.length === 0 ? (
                <div
                  style={{
                    textAlign: "center",
                    padding: "24px",
                    color: "var(--text-muted)",
                    fontSize: "13px",
                    background: "rgba(15, 23, 42, 0.4)",
                    borderRadius: "6px",
                  }}
                >
                  No other known workspaces recorded yet. Enter a directory path above to open.
                </div>
              ) : (
                workspaces.map((ws) => {
                  const isCurrent = ws.path === currentPath || ws.name === currentPath;
                  return (
                    <div
                      key={ws.path}
                      onClick={() => {
                        onSelectWorkspace(ws.path);
                        onClose();
                      }}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        padding: "10px 14px",
                        background: isCurrent ? "rgba(6, 182, 212, 0.12)" : "rgba(30, 41, 59, 0.5)",
                        border: isCurrent
                          ? "1px solid rgba(6, 182, 212, 0.45)"
                          : "1px solid var(--border-color)",
                        borderRadius: "6px",
                        cursor: "pointer",
                        transition: "all 0.15s ease",
                      }}
                      onMouseEnter={(e) => {
                        if (!isCurrent) e.currentTarget.style.background = "rgba(30, 41, 59, 0.85)";
                      }}
                      onMouseLeave={(e) => {
                        if (!isCurrent) e.currentTarget.style.background = "rgba(30, 41, 59, 0.5)";
                      }}
                    >
                      <div style={{ display: "flex", alignItems: "center", gap: "10px", minWidth: 0 }}>
                        <Folder size={16} color={isCurrent ? "#06b6d4" : "var(--text-muted)"} />
                        <div style={{ minWidth: 0 }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                            <span style={{ fontWeight: 600, color: "#fff", fontSize: "13px" }}>
                              {ws.name}
                            </span>
                            {ws.framework && (
                              <span
                                style={{
                                  fontSize: "10px",
                                  padding: "1px 6px",
                                  borderRadius: "4px",
                                  background: "rgba(148, 163, 184, 0.15)",
                                  color: "var(--text-muted)",
                                  textTransform: "uppercase",
                                }}
                              >
                                {ws.framework}
                              </span>
                            )}
                          </div>
                          <div
                            style={{
                              fontSize: "11px",
                              color: "var(--text-muted)",
                              whiteSpace: "nowrap",
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              maxWidth: "380px",
                            }}
                            title={ws.path}
                          >
                            {ws.path}
                          </div>
                        </div>
                      </div>

                      <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                        {ws.last_opened && (
                          <div
                            style={{
                              display: "flex",
                              alignItems: "center",
                              gap: "4px",
                              fontSize: "11px",
                              color: "var(--text-muted)",
                              marginRight: "4px",
                            }}
                          >
                            <Clock size={11} />
                            <span>{formatTime(ws.last_opened)}</span>
                          </div>
                        )}

                        {isCurrent ? (
                          <span
                            style={{
                              display: "flex",
                              alignItems: "center",
                              gap: "4px",
                              padding: "4px 10px",
                              background: "rgba(16, 185, 129, 0.15)",
                              border: "1px solid rgba(16, 185, 129, 0.4)",
                              borderRadius: "4px",
                              color: "#10b981",
                              fontSize: "11.5px",
                              fontWeight: 600,
                            }}
                          >
                            <Check size={12} /> Active
                          </span>
                        ) : (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              onSelectWorkspace(ws.path);
                              onClose();
                            }}
                            style={{
                              display: "flex",
                              alignItems: "center",
                              gap: "4px",
                              padding: "4px 10px",
                              background: "rgba(6, 182, 212, 0.15)",
                              border: "1px solid rgba(6, 182, 212, 0.4)",
                              borderRadius: "4px",
                              color: "#06b6d4",
                              fontSize: "11.5px",
                              fontWeight: 600,
                              cursor: "pointer",
                              transition: "all 0.15s ease",
                            }}
                            onMouseEnter={(e) => {
                              e.currentTarget.style.background = "#0891b2";
                              e.currentTarget.style.color = "#fff";
                            }}
                            onMouseLeave={(e) => {
                              e.currentTarget.style.background = "rgba(6, 182, 212, 0.15)";
                              e.currentTarget.style.color = "#06b6d4";
                            }}
                          >
                            <span>Open</span>
                            <ArrowRight size={12} />
                          </button>
                        )}

                        <button
                          onClick={(e) => handleRemove(e, ws.path)}
                          title="Remove from recent list"
                          style={{
                            background: "transparent",
                            border: "none",
                            color: "var(--text-muted)",
                            cursor: "pointer",
                            padding: "4px",
                            borderRadius: "4px",
                          }}
                          onMouseEnter={(e) => (e.currentTarget.style.color = "#ef4444")}
                          onMouseLeave={(e) => (e.currentTarget.style.color = "var(--text-muted)")}
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
