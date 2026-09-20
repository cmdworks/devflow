import React from "react";
import {
  LayoutDashboard,
  Zap,
  Terminal,
  ChevronLeft,
  ChevronRight,
  FolderGit2,
} from "lucide-react";
import type { ProjectTarget, PaneInfo, ViewSection, ActiveSessionInfo } from "../types";

interface SecondarySidebarProps {
  workspaceName: string;
  workspacePath: string;
  activeSection: ViewSection;
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activeSessions: ActiveSessionInfo[];
  isCollapsed: boolean;
  onToggleCollapse: () => void;
  onSelectSection: (section: ViewSection) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
}

export const SecondarySidebar: React.FC<SecondarySidebarProps> = ({
  workspaceName,
  activeSection,
  targets,
  openPanes,
  activeSessions,
  isCollapsed,
  onToggleCollapse,
  onSelectSection,
  onOpenTargetPane,
}) => {
  const runningTargetsCount = activeSessions.length;

  if (isCollapsed) {
    return (
      <div className="secondary-sidebar-collapsed">
        <button
          className="btn-toggle-subsidebar"
          onClick={onToggleCollapse}
          title="Expand Workspace Navigation"
        >
          <ChevronRight size={14} />
        </button>

        <div className="secondary-collapsed-nav">
          <button
            className={`secondary-icon-btn ${activeSection === "overview" ? "active" : ""}`}
            onClick={() => onSelectSection("overview")}
            title="Overview"
          >
            <LayoutDashboard size={14} />
          </button>

          <button
            className={`secondary-icon-btn ${activeSection === "targets" ? "active" : ""}`}
            onClick={() => onSelectSection("targets")}
            title={`Targets (${targets.length})`}
          >
            <Zap size={14} />
            {runningTargetsCount > 0 && <span className="icon-pulse-badge" />}
          </button>

          <button
            className={`secondary-icon-btn ${activeSection === "terminal" ? "active" : ""}`}
            onClick={() => onSelectSection("terminal")}
            title="Terminal Logs"
          >
            <Terminal size={14} />
          </button>
        </div>
      </div>
    );
  }

  return (
    <aside className="secondary-sidebar">
      {/* 1. Header: Active Workspace Header & Collapse Action */}
      <div className="secondary-sidebar-header">
        <div className="secondary-ws-info">
          <FolderGit2 size={13} color="#06b6d4" />
          <span className="secondary-ws-name" title={workspaceName}>
            {workspaceName}
          </span>
        </div>
        <button
          className="btn-toggle-subsidebar"
          onClick={onToggleCollapse}
          title="Collapse Sidebar"
        >
          <ChevronLeft size={14} />
        </button>
      </div>

      {/* 2. Workspace View Navigation */}
      <div className="secondary-nav-section">
        <button
          className={`secondary-nav-item ${activeSection === "overview" ? "active" : ""}`}
          onClick={() => onSelectSection("overview")}
        >
          <LayoutDashboard size={14} />
          <span>Overview</span>
        </button>

        <button
          className={`secondary-nav-item ${activeSection === "targets" ? "active" : ""}`}
          onClick={() => onSelectSection("targets")}
        >
          <Zap size={14} />
          <span>Targets</span>
          {runningTargetsCount > 0 && (
            <span className="nav-running-badge">{runningTargetsCount}</span>
          )}
        </button>

        <button
          className={`secondary-nav-item ${activeSection === "terminal" ? "active" : ""}`}
          onClick={() => onSelectSection("terminal")}
        >
          <Terminal size={14} />
          <span>Terminal</span>
          <span className="nav-stream-dot" />
        </button>
      </div>

      {/* 3. Subprojects & Targets Quick Jump List */}
      <div className="secondary-targets-section">
        <div className="section-label">SUBPROJECTS ({targets.length})</div>

        <div className="secondary-targets-list">
          {targets.length === 0 ? (
            <div className="secondary-empty-text">No targets discovered</div>
          ) : (
            targets.map((target) => {
              const isOpen = openPanes.some((p) => p.targetId === target.id);
              const activePane = openPanes.find((p) => p.targetId === target.id);
              const status = activePane ? activePane.status : "idle";
              const platformClass = (target.platform || "generic").toLowerCase();

              return (
                <div
                  key={target.id}
                  className={`secondary-target-card ${isOpen ? "open" : ""}`}
                  onClick={() => {
                    onOpenTargetPane(target);
                    onSelectSection("terminal");
                  }}
                  title={`Open / focus terminal pane for ${target.name}`}
                >
                  <div className="target-card-main">
                    <div className="target-card-name">{target.name}</div>
                    <div className="target-card-meta">
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
                </div>
              );
            })
          )}
        </div>
      </div>
    </aside>
  );
};

