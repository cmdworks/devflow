import React from "react";
import {
  LayoutDashboard,
  Zap,
  ChevronLeft,
  ChevronRight,
  FolderGit2,
  Radio,
} from "lucide-react";
import type { ProjectTarget, PaneInfo, ViewSection, ActiveSessionInfo } from "../types";

interface SecondarySidebarProps {
  workspaceName: string;
  workspacePath: string;
  activeSection: ViewSection;
  activePaneId?: string;
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activeSessions?: ActiveSessionInfo[];
  isCollapsed: boolean;
  onToggleCollapse: () => void;
  onSelectSection: (section: ViewSection) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
}

export const SecondarySidebar: React.FC<SecondarySidebarProps> = ({
  workspaceName,
  activeSection,
  activePaneId,
  targets,
  openPanes,
  isCollapsed,
  onToggleCollapse,
  onSelectSection,
  onOpenTargetPane,
  onOpenCombinedPane,
}) => {
  const runningTargetsCount = openPanes.filter(
    (p) => !p.isCombined && (p.status === "running" || p.status === "building")
  ).length;

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
            title="Overview Dashboard"
          >
            <LayoutDashboard size={14} />
          </button>

          <button
            className={`secondary-icon-btn ${activeSection === "targets" ? "active" : ""}`}
            onClick={() => onSelectSection("targets")}
            title={`Targets Matrix (${targets.length})`}
          >
            <Zap size={14} />
            {runningTargetsCount > 0 && <span className="icon-pulse-badge" />}
          </button>

          <button
            className={`secondary-icon-btn ${
              activeSection === "terminal" && activePaneId === "pane-combined" ? "active" : ""
            }`}
            onClick={() => {
              onOpenCombinedPane();
              onSelectSection("terminal");
            }}
            title="Combined Live Stream"
          >
            <Radio size={14} color="#06b6d4" />
          </button>

          <div className="collapsed-divider" />

          {targets.map((target) => {
            const pane = openPanes.find((p) => p.targetId === target.id);
            const status = pane ? pane.status : "idle";
            const isActive = activeSection === "terminal" && activePaneId === `pane-${target.id}`;

            return (
              <button
                key={target.id}
                className={`secondary-icon-btn ${isActive ? "active" : ""}`}
                onClick={() => {
                  onOpenTargetPane(target);
                  onSelectSection("terminal");
                }}
                title={`${target.name} [${target.framework}] (${status})`}
              >
                <span
                  className={`collapsed-status-dot ${
                    status === "running"
                      ? "green"
                      : status === "building"
                      ? "yellow"
                      : status === "error"
                      ? "red"
                      : "gray"
                  }`}
                />
              </button>
            );
          })}
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

      {/* 2. Top Views Navigation */}
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
          <span>Targets Matrix</span>
          <span className="nav-count-pill">{targets.length}</span>
        </button>
      </div>

      {/* 3. Stream & Subprojects Cockpit List */}
      <div className="secondary-targets-section">
        <div className="section-label">LIVE COCKPIT & LOGS</div>

        {/* Combined All Stream Entry */}
        <div
          className={`secondary-target-card combined-entry ${
            activeSection === "terminal" && activePaneId === "pane-combined" ? "active" : ""
          }`}
          onClick={() => {
            onOpenCombinedPane();
            onSelectSection("terminal");
          }}
          title="Open Combined Stream of all active targets"
        >
          <div className="target-card-main">
            <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
              <Radio size={13} color="#06b6d4" />
              <div className="target-card-name" style={{ fontWeight: 600 }}>
                All Streams (Combined)
              </div>
            </div>
            <div className="target-card-meta">
              <span className="platform-tag universal">universal</span>
              <span className="running-count-tag">
                {runningTargetsCount} running
              </span>
            </div>
          </div>
        </div>

        <div className="section-label" style={{ marginTop: "12px" }}>
          SUBPROJECTS ({targets.length})
        </div>

        <div className="secondary-targets-list">
          {targets.length === 0 ? (
            <div className="secondary-empty-text">No targets discovered</div>
          ) : (
            targets.map((target) => {
              const activePane = openPanes.find((p) => p.targetId === target.id);
              const status = activePane ? activePane.status : "idle";
              const platformClass = (target.platform || "generic").toLowerCase();
              const isSelected =
                activeSection === "terminal" && activePaneId === `pane-${target.id}`;

              return (
                <div
                  key={target.id}
                  className={`secondary-target-card ${isSelected ? "active" : ""}`}
                  onClick={() => {
                    onOpenTargetPane(target);
                    onSelectSection("terminal");
                  }}
                  title={`Open / focus live cockpit & logs for ${target.name}`}
                >
                  <div className="target-card-main">
                    <div className="target-card-name-row">
                      <span className="target-card-name">{target.name}</span>
                      {target.is_default && <span className="default-pill mini">MAIN</span>}
                    </div>
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
                      <span className="framework-tag-mini">{target.framework}</span>
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

