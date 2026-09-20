import React from "react";
import {
  LayoutDashboard,
  Zap,
  Terminal,
  Radio,
  Plus,
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
  onOpenCombinedPane,
}) => {
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
      </div>
    );
  }

  const runningTargetsCount = activeSessions.length;

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

      {/* 2. Workspace View Tabs */}
      <div className="secondary-nav-section">
        <div className="section-label">WORKSPACE VIEWS</div>

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
          <span>Targets & Matrix</span>
          {runningTargetsCount > 0 && (
            <span className="nav-running-badge">{runningTargetsCount}</span>
          )}
        </button>

        <button
          className={`secondary-nav-item ${activeSection === "terminal" ? "active" : ""}`}
          onClick={() => onSelectSection("terminal")}
        >
          <Terminal size={14} />
          <span>Terminal Logs</span>
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
                  title={`Open terminal pane for ${target.name}`}
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

                  <button
                    className="btn-quick-open-pane"
                    title="Open in Terminal Tab"
                    onClick={(e) => {
                      e.stopPropagation();
                      onOpenTargetPane(target);
                      onSelectSection("terminal");
                    }}
                  >
                    <Plus size={12} />
                  </button>
                </div>
              );
            })
          )}
        </div>

        <button
          className="btn-combined-stream"
          onClick={() => {
            onOpenCombinedPane();
            onSelectSection("terminal");
          }}
          title="Aggregated Multi-Target Live Stream"
        >
          <Radio size={13} color="#06b6d4" />
          <span>Combined Stream</span>
        </button>
      </div>
    </aside>
  );
};
