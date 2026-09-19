import React, { useState, useRef, useEffect } from "react";
import {
  Folder,
  FolderGit2,
  Smartphone,
  Radio,
  Plus,
  FolderOpen,
  Clipboard,
  ChevronDown,
  ChevronUp,
  Activity,
  Terminal,
  Layers,
  Check,
} from "lucide-react";
import type { ProjectTarget, Device, PaneInfo, KnownWorkspace } from "../types";

export type ViewMode = "terminal" | "devices" | "doctor" | "targets";

interface SidebarProps {
  workspaceName: string;
  workspacePath: string;
  knownWorkspaces: KnownWorkspace[];
  activeView: ViewMode;
  targets: ProjectTarget[];
  devices: Device[];
  openPanes: PaneInfo[];
  onSelectView: (view: ViewMode) => void;
  onSelectWorkspace: (path: string) => void;
  onBrowseWorkspace: () => void;
  onPasteWorkspace: () => void;
  onOpenWorkspaceManager: () => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  workspaceName,
  workspacePath,
  knownWorkspaces,
  activeView,
  targets,
  devices,
  openPanes,
  onSelectView,
  onSelectWorkspace,
  onBrowseWorkspace,
  onPasteWorkspace,
  onOpenWorkspaceManager,
  onOpenTargetPane,
  onOpenCombinedPane,
}) => {
  const [isWorkspaceDropdownOpen, setIsWorkspaceDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement | null>(null);

  // Close dropdown on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsWorkspaceDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const onlineDevicesCount = devices.filter(
    (d) => d.state === "Connected" || d.state === "connected" || d.state === "Booted" || d.online
  ).length;

  return (
    <aside className="sidebar">
      {/* 1. Top Section: Active Workspace Card & Switcher */}
      <div className="sidebar-workspace-container" ref={dropdownRef}>
        <div
          className="sidebar-workspace-card"
          onClick={() => setIsWorkspaceDropdownOpen(!isWorkspaceDropdownOpen)}
          title="Click to switch or manage workspace"
        >
          <div className="workspace-card-header">
            <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
              <div className="workspace-icon-box">
                <Folder size={15} color="#06b6d4" />
              </div>
              <div style={{ minWidth: 0 }}>
                <div className="workspace-name-text text-truncate">{workspaceName || "Workspace"}</div>
                <div className="workspace-path-snippet text-truncate" title={workspacePath}>
                  {workspacePath ? `...${workspacePath.slice(-22)}` : "No workspace loaded"}
                </div>
              </div>
            </div>

            <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
              {isWorkspaceDropdownOpen ? (
                <ChevronUp size={14} color="var(--text-muted)" />
              ) : (
                <ChevronDown size={14} color="var(--text-muted)" />
              )}
            </div>
          </div>
        </div>

        {/* Quick Switch Dropdown */}
        {isWorkspaceDropdownOpen && (
          <div className="sidebar-workspace-dropdown">
            <div className="dropdown-header-row">
              <span className="dropdown-title">Recent Workspaces</span>
              <button
                className="dropdown-action-link"
                onClick={(e) => {
                  e.stopPropagation();
                  setIsWorkspaceDropdownOpen(false);
                  onOpenWorkspaceManager();
                }}
              >
                Manage All
              </button>
            </div>

            <div className="dropdown-list">
              {knownWorkspaces.length === 0 ? (
                <div className="dropdown-empty">No other recent projects</div>
              ) : (
                knownWorkspaces.map((ws) => {
                  const isCurrent = ws.path === workspacePath || ws.name === workspacePath;
                  return (
                    <div
                      key={ws.path}
                      className={`dropdown-item ${isCurrent ? "active-item" : ""}`}
                      onClick={() => {
                        onSelectWorkspace(ws.path);
                        setIsWorkspaceDropdownOpen(false);
                      }}
                      title={ws.path}
                    >
                      <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
                        <Folder size={13} color={isCurrent ? "#06b6d4" : "var(--text-muted)"} />
                        <span className="dropdown-item-name text-truncate">{ws.name}</span>
                        {ws.framework && <span className="dropdown-framework-badge">{ws.framework}</span>}
                      </div>

                      {isCurrent && <Check size={13} color="#10b981" />}
                    </div>
                  );
                })
              )}
            </div>

            <div className="dropdown-footer-actions">
              <button
                className="dropdown-btn-action"
                onClick={(e) => {
                  e.stopPropagation();
                  setIsWorkspaceDropdownOpen(false);
                  onBrowseWorkspace();
                }}
                title="Browse directory with macOS native Finder dialog"
              >
                <FolderOpen size={13} color="#06b6d4" />
                <span>Browse...</span>
              </button>

              <button
                className="dropdown-btn-action"
                onClick={(e) => {
                  e.stopPropagation();
                  setIsWorkspaceDropdownOpen(false);
                  onPasteWorkspace();
                }}
                title="Paste directory from clipboard"
              >
                <Clipboard size={13} />
                <span>Paste</span>
              </button>
            </div>
          </div>
        )}
      </div>

      {/* 2. Main Navigation Mode Selector */}
      <div className="sidebar-nav-section">
        <button
          className={`nav-mode-btn ${activeView === "terminal" ? "active" : ""}`}
          onClick={() => onSelectView("terminal")}
        >
          <Terminal size={15} color={activeView === "terminal" ? "#06b6d4" : "#94a3b8"} />
          <span>Terminal Logs</span>
          {openPanes.length > 0 && <span className="nav-badge">{openPanes.length}</span>}
        </button>

        <button
          className={`nav-mode-btn ${activeView === "targets" ? "active" : ""}`}
          onClick={() => onSelectView("targets")}
        >
          <Layers size={15} color={activeView === "targets" ? "#06b6d4" : "#94a3b8"} />
          <span>Targets & Sessions</span>
          <span className="nav-badge">{targets.length}</span>
        </button>

        <button
          className={`nav-mode-btn ${activeView === "devices" ? "active" : ""}`}
          onClick={() => onSelectView("devices")}
        >
          <Smartphone size={15} color={activeView === "devices" ? "#38bdf8" : "#94a3b8"} />
          <span>Devices & Emulators</span>
          <span className="nav-badge">{devices.length}</span>
        </button>

        <button
          className={`nav-mode-btn ${activeView === "doctor" ? "active" : ""}`}
          onClick={() => onSelectView("doctor")}
        >
          <Activity size={15} color={activeView === "doctor" ? "#10b981" : "#94a3b8"} />
          <span>Doctor Diagnostics</span>
        </button>
      </div>

      {/* 3. Sub-Projects / Targets Section */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <FolderGit2 size={13} color="#06b6d4" />
            <span>Workspace Targets</span>
          </div>
          <span className="target-badge-count">{targets.length}</span>
        </div>

        {targets.length === 0 ? (
          <div style={{ color: "var(--text-muted)", fontSize: "11px", padding: "6px 0" }}>
            No recognized targets
          </div>
        ) : (
          targets.map((target) => {
            const isOpen = openPanes.some((p) => p.targetId === target.id);
            const activePane = openPanes.find((p) => p.targetId === target.id);
            const status = activePane ? activePane.status : "idle";
            const platformClass = (target.platform || "generic").toLowerCase();

            return (
              <div
                key={target.id}
                className={`subproject-card ${isOpen ? "active-in-pane" : ""}`}
                onClick={() => {
                  onOpenTargetPane(target);
                  onSelectView("terminal");
                }}
                title={`Open logs for ${target.name}`}
              >
                <div className="subproject-info">
                  <div className="subproject-title">{target.name}</div>
                  <div className="subproject-meta">
                    <span className={`platform-tag ${platformClass}`}>{target.platform}</span>
                    <span
                      className={`pulse-dot ${
                        status === "running"
                          ? "green"
                          : status === "building"
                          ? "yellow"
                          : status === "error"
                          ? "red"
                          : "gray"
                      }`}
                    />
                    <span>{target.framework}</span>
                  </div>
                </div>

                <button
                  className="subproject-action-btn"
                  style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    fontSize: "13px",
                  }}
                  title="Open Tab"
                >
                  <Plus size={14} />
                </button>
              </div>
            );
          })
        )}

        <button
          className="sidebar-btn-combine"
          onClick={() => {
            onOpenCombinedPane();
            onSelectView("terminal");
          }}
        >
          <Radio size={13} />
          <span>Combined Stream</span>
        </button>
      </div>

      {/* 4. Footer: Live Device Status Indicator */}
      <div className="sidebar-footer" onClick={() => onSelectView("devices")}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <span className={`pulse-dot ${onlineDevicesCount > 0 ? "green" : "yellow"}`} />
          <span style={{ fontSize: "11.5px", fontWeight: 500, color: "var(--text-secondary)" }}>
            {onlineDevicesCount} Device{onlineDevicesCount === 1 ? "" : "s"} Ready
          </span>
        </div>
        <Smartphone size={14} color="#38bdf8" />
      </div>
    </aside>
  );
};
