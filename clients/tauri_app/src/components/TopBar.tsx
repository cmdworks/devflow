import React from "react";
import {
  Play,
  Zap,
  RotateCw,
  Square,
  ChevronRight,
  PanelLeft,
  Sidebar as SidebarIcon,
  Search,
  X,
  Loader2,
  Sliders,
} from "lucide-react";
import type { ViewSection } from "../types";

interface TopBarProps {
  workspaceName: string;
  activeSection: ViewSection;
  isPrimarySidebarCollapsed: boolean;
  isSecondarySidebarCollapsed: boolean;
  searchQuery?: string;
  levelFilter?: string;
  targetsCount?: number;
  runningTargetsCount?: number;
  allTargetsRunning?: boolean;
  anyTargetRunning?: boolean;
  isStartingAll?: boolean;
  isStoppingAll?: boolean;
  onChangeSearch?: (query: string) => void;
  onSelectLevel?: (level: string) => void;
  onTogglePrimarySidebar: () => void;
  onToggleSecondarySidebar: () => void;
  onSelectSection: (section: ViewSection) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onOpenDevOptions?: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  workspaceName,
  activeSection,
  isPrimarySidebarCollapsed,
  isSecondarySidebarCollapsed,
  searchQuery = "",
  levelFilter = "ALL",
  targetsCount = 0,
  runningTargetsCount: _runningTargetsCount = 0,
  allTargetsRunning = false,
  anyTargetRunning = false,
  isStartingAll = false,
  isStoppingAll = false,
  onChangeSearch,
  onSelectLevel,
  onTogglePrimarySidebar,
  onToggleSecondarySidebar,
  onSelectSection,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onOpenDevOptions,
}) => {
  const sectionLabelMap: Record<ViewSection, string> = {
    overview: "Overview",
    targets: "Targets & Matrix",
    terminal: "Terminal Logs",
    devices: "Devices & Emulators",
    doctor: "Doctor Diagnostics",
    mcp: "MCP Hub",
    settings: "Settings",
  };

  return (
    <header className="top-bar">
      {/* 1. Left: Sidebar Toggles & Brand & Breadcrumb */}
      <div className="top-bar-left">
        <div className="sidebar-toggles-group">
          <button
            className={`btn-top-icon-toggle ${!isPrimarySidebarCollapsed ? "active" : ""}`}
            onClick={onTogglePrimarySidebar}
            title={isPrimarySidebarCollapsed ? "Expand Workspaces Rail" : "Collapse Workspaces Rail"}
          >
            <PanelLeft size={14} />
          </button>

          <button
            className={`btn-top-icon-toggle ${!isSecondarySidebarCollapsed ? "active" : ""}`}
            onClick={onToggleSecondarySidebar}
            title={isSecondarySidebarCollapsed ? "Expand Workspace Nav" : "Collapse Workspace Nav"}
          >
            <SidebarIcon size={14} />
          </button>
        </div>

        <div className="app-brand" onClick={() => onSelectSection("overview")} style={{ cursor: "pointer" }}>
          <img src="/image/logo/icon.svg" alt="DevFlow" style={{ width: "16px", height: "16px", objectFit: "contain" }} />
          <span className="brand-name">DevFlow</span>
          <span className="brand-badge">PRO</span>
        </div>

        <div className="top-bar-breadcrumb">
          <span className="breadcrumb-sep">
            <ChevronRight size={12} />
          </span>
          <span
            className="breadcrumb-item workspace-name"
            onClick={() => onSelectSection("overview")}
            title="Jump to Workspace Overview"
          >
            {workspaceName}
          </span>
          <span className="breadcrumb-sep">
            <ChevronRight size={12} />
          </span>
          <span className="breadcrumb-item current-section">
            {sectionLabelMap[activeSection] || "Terminal"}
          </span>
        </div>
      </div>

      {/* 2. Center: Global Process Batch Lifecycle Controls */}
      <div className="top-bar-center">
        <button
          className={`btn-top-action primary ${isStartingAll ? "loading" : ""}`}
          onClick={onRunAll}
          disabled={isStartingAll || allTargetsRunning || targetsCount === 0}
          title={
            isStartingAll
              ? "Starting workspace targets..."
              : allTargetsRunning
              ? "All targets are already running"
              : targetsCount === 0
              ? "No targets found in workspace"
              : "Run all workspace targets"
          }
        >
          {isStartingAll ? (
            <Loader2 size={12} className="animate-spin" />
          ) : (
            <Play size={12} fill="currentColor" />
          )}
          <span>{isStartingAll ? "Starting..." : "Run All"}</span>
        </button>

        <button
          className="btn-top-action"
          onClick={onReloadAll}
          disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
          title={anyTargetRunning ? "Hot reload all active running sessions" : "No active sessions to reload"}
        >
          <Zap size={12} color={anyTargetRunning ? "#f59e0b" : "var(--text-muted)"} />
          <span>Reload</span>
        </button>

        <button
          className="btn-top-action"
          onClick={onRestartAll}
          disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
          title={anyTargetRunning ? "Restart all active apps" : "No active sessions to restart"}
        >
          <RotateCw size={12} color={anyTargetRunning ? "#06b6d4" : "var(--text-muted)"} />
          <span>Restart</span>
        </button>

        <button
          className={`btn-top-action danger ${isStoppingAll ? "loading" : ""}`}
          onClick={onStopAll}
          disabled={isStoppingAll || !anyTargetRunning}
          title={
            isStoppingAll
              ? "Stopping running processes..."
              : !anyTargetRunning
              ? "No running processes to stop"
              : "Stop all running processes"
          }
        >
          {isStoppingAll ? (
            <Loader2 size={12} className="animate-spin" />
          ) : (
            <Square size={12} fill="currentColor" />
          )}
          <span>{isStoppingAll ? "Stopping..." : "Stop All"}</span>
        </button>

        {onOpenDevOptions && (
          <button
            className="btn-top-action dev-options"
            onClick={onOpenDevOptions}
            title="Configure Dev Server, Clean Build, Release Mode, or Force Terminate"
          >
            <Sliders size={12} color="#38bdf8" />
            <span>Options</span>
          </button>
        )}
      </div>

      {/* 3. Right: Terminal Log Search & Level Filters */}
      <div className="top-bar-right">
        {activeSection === "terminal" && onChangeSearch && onSelectLevel ? (
          <div className="top-bar-search-group">
            <div className="top-bar-search-box">
              <Search size={11} color="var(--text-muted)" />
              <input
                type="text"
                className="top-bar-search-input"
                placeholder="Search logs..."
                value={searchQuery}
                onChange={(e) => onChangeSearch(e.target.value)}
              />
              {searchQuery && (
                <button className="btn-search-clear" onClick={() => onChangeSearch("")} title="Clear search">
                  <X size={10} />
                </button>
              )}
            </div>

            <div className="top-bar-level-chips">
              {[
                { label: "ALL", value: "ALL" },
                { label: "ERR", value: "E" },
                { label: "WRN", value: "W" },
                { label: "INF", value: "I" },
              ].map(({ label, value }) => {
                const isActive =
                  (value === "ALL" && (!levelFilter || levelFilter === "ALL" || levelFilter === "*")) ||
                  levelFilter === value ||
                  (value === "E" && (levelFilter === "ERR" || levelFilter === "ERROR")) ||
                  (value === "W" && (levelFilter === "WRN" || levelFilter === "WARN")) ||
                  (value === "I" && (levelFilter === "INF" || levelFilter === "INFO"));

                return (
                  <button
                    key={label}
                    className={`chip-level-btn ${isActive ? "active" : ""} ${label.toLowerCase()}`}
                    onClick={() => onSelectLevel(value)}
                  >
                    {label}
                  </button>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="top-bar-meta-hint">
            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
              {targetsCount} Target{targetsCount === 1 ? "" : "s"} Discovered
            </span>
          </div>
        )}
      </div>
    </header>
  );
};
