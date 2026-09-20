import React, { useState } from "react";
import {
  Copy,
  Check,
  Play,
  Square,
  Zap,
  RotateCw,
  Terminal,
  FolderOpen,
  Layers,
  Smartphone,
  Loader2,
  Sliders,
} from "lucide-react";
import type { ProjectTarget, PaneInfo, ActiveSessionInfo, Device } from "../types";

interface WorkspaceOverviewViewProps {
  workspaceName: string;
  workspacePath: string;
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activeSessions: ActiveSessionInfo[];
  devices: Device[];
  isStartingAll?: boolean;
  isStoppingAll?: boolean;
  allTargetsRunning?: boolean;
  anyTargetRunning?: boolean;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onToggleRunTarget: (targetId: string) => void;
  onReloadTarget: (targetId: string) => void;
  onRestartTarget: (targetId: string) => void;
  onOpenDevOptions?: (targetId?: string) => void;
  onSwitchToTerminal: () => void;
  onSwitchToTargets?: () => void;
}

export const WorkspaceOverviewView: React.FC<WorkspaceOverviewViewProps> = ({
  workspaceName,
  workspacePath,
  targets,
  openPanes,
  activeSessions: _activeSessions,
  devices,
  isStartingAll = false,
  isStoppingAll = false,
  allTargetsRunning = false,
  anyTargetRunning = false,
  onOpenTargetPane,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onToggleRunTarget,
  onReloadTarget,
  onRestartTarget,
  onOpenDevOptions,
  onSwitchToTerminal,
  onSwitchToTargets,
}) => {
  const [copiedPath, setCopiedPath] = useState<boolean>(false);

  const handleCopyPath = async () => {
    try {
      await navigator.clipboard.writeText(workspacePath);
      setCopiedPath(true);
      setTimeout(() => setCopiedPath(false), 2000);
    } catch (_) {}
  };

  const runningCount = openPanes.filter(
    (p) => !p.isCombined && (p.status === "running" || p.status === "building")
  ).length;
  const onlineDevicesCount = devices.filter((d) => d.online || d.state === "device" || d.state === "booted").length;

  return (
    <div className="overview-view-wrapper">
      {/* 1. Header Banner & Workspace Identity */}
      <div className="overview-hero-card">
        <div className="overview-hero-left">
          <div className="overview-avatar-icon">
            <Layers size={22} color="#06b6d4" />
          </div>
          <div className="overview-hero-meta">
            <div className="overview-title-row">
              <h2 className="overview-title">{workspaceName}</h2>
              <span className="overview-badge-platform">
                {targets[0]?.platform || "Universal"}
              </span>
              <span className="overview-badge-targets">
                {targets.length} Target{targets.length === 1 ? "" : "s"}
              </span>
            </div>
            <div className="overview-path-row">
              <span className="overview-path-text" title={workspacePath}>
                {workspacePath}
              </span>
              <button
                className="btn-overview-copy"
                onClick={handleCopyPath}
                title="Copy full directory path"
              >
                {copiedPath ? <Check size={11} color="#10b981" /> : <Copy size={11} />}
                <span>{copiedPath ? "Copied" : "Copy"}</span>
              </button>
            </div>
          </div>
        </div>

        {/* Global Batch Process Actions */}
        <div className="overview-hero-actions">
          <button
            className={`btn-hero-action primary ${isStartingAll ? "loading" : ""}`}
            onClick={onRunAll}
            disabled={isStartingAll || allTargetsRunning || targets.length === 0}
            title={
              isStartingAll
                ? "Starting workspace targets..."
                : allTargetsRunning
                ? "All targets are already running"
                : targets.length === 0
                ? "No targets found in workspace"
                : "Run all workspace targets"
            }
          >
            {isStartingAll ? (
              <Loader2 size={13} className="animate-spin" />
            ) : (
              <Play size={13} fill="currentColor" />
            )}
            <span>{isStartingAll ? "Starting..." : "Run All"}</span>
          </button>

          <button
            className="btn-hero-action"
            onClick={onReloadAll}
            disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
            title={anyTargetRunning ? "Hot reload all active running sessions" : "No active sessions to reload"}
          >
            <Zap size={13} color={anyTargetRunning ? "#f59e0b" : "var(--text-muted)"} />
            <span>Reload</span>
          </button>

          <button
            className="btn-hero-action"
            onClick={onRestartAll}
            disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
            title={anyTargetRunning ? "Restart all active apps" : "No active sessions to restart"}
          >
            <RotateCw size={13} color={anyTargetRunning ? "#06b6d4" : "var(--text-muted)"} />
            <span>Restart</span>
          </button>

          <button
            className={`btn-hero-action danger ${isStoppingAll ? "loading" : ""}`}
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
              <Loader2 size={13} className="animate-spin" />
            ) : (
              <Square size={13} fill="currentColor" />
            )}
            <span>{isStoppingAll ? "Stopping..." : "Stop All"}</span>
          </button>
        </div>
      </div>

      {/* 2. Quick Stat Counters */}
      <div className="overview-stats-grid">
        <div
          className="stat-card"
          onClick={onSwitchToTargets || onSwitchToTerminal}
          style={{ cursor: "pointer" }}
        >
          <div className="stat-label">Discovered Targets</div>
          <div className="stat-value">{targets.length}</div>
          <div className="stat-meta">
            <Layers size={12} color="#06b6d4" />
            <span>Click to view targets matrix</span>
          </div>
        </div>

        <div
          className="stat-card"
          onClick={onSwitchToTerminal}
          style={{ cursor: "pointer" }}
        >
          <div className="stat-label">Active Processes</div>
          <div className="stat-value" style={{ color: runningCount > 0 ? "#10b981" : "inherit" }}>
            {runningCount}
          </div>
          <div className="stat-meta">
            <span className={`pulse-dot ${runningCount > 0 ? "green" : "gray"}`} />
            <span>{runningCount > 0 ? "Running in background (view logs)" : "All idle"}</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-label">Connected Devices</div>
          <div className="stat-value">{onlineDevicesCount}</div>
          <div className="stat-meta">
            <Smartphone size={12} color="#38bdf8" />
            <span>Host & connected targets</span>
          </div>
        </div>
      </div>

      {/* 3. Discovered Subprojects & Targets Table / Card Matrix */}
      <div className="overview-section">
        <div className="section-header-row">
          <div>
            <h2 className="section-heading">Workspace Targets</h2>
            <p className="section-subheading">
              Subprojects and mobile/native components discovered in this directory.
            </p>
          </div>
          <button className="btn-terminal-jump" onClick={onSwitchToTerminal}>
            <Terminal size={13} />
            <span>Open Terminal Tabs</span>
          </button>
        </div>

        <div className="overview-targets-grid">
          {targets.map((target) => {
            const pane = openPanes.find((p) => p.targetId === target.id);
            const status = pane ? pane.status : "idle";
            const isRunning = status === "running" || status === "building";
            const platformClass = (target.platform || "generic").toLowerCase();

            return (
              <div key={target.id} className="overview-target-card">
                <div className="overview-target-header">
                  <div>
                    <div className="overview-target-name">{target.name}</div>
                    <div className="overview-target-badges">
                      <span className={`platform-tag ${platformClass}`}>{target.platform}</span>
                      <span className="framework-tag">{target.framework}</span>
                    </div>
                  </div>
                  <span
                    className={`status-pill ${
                      status === "running"
                        ? "running"
                        : status === "building"
                        ? "building"
                        : status === "error"
                        ? "error"
                        : "idle"
                    }`}
                  >
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
                    <span>{status.toUpperCase()}</span>
                  </span>
                </div>

                <div className="overview-target-path" title={target.path}>
                  <FolderOpen size={11} color="var(--text-muted)" />
                  <span>{target.path}</span>
                </div>

                <div className="overview-target-actions">
                  <button
                    className={`btn-target-run ${isRunning ? "active" : ""}`}
                    onClick={() => onToggleRunTarget(target.id)}
                    title={isRunning ? "Stop Target" : "Run Target"}
                  >
                    {isRunning ? (
                      <>
                        <Square size={12} fill="currentColor" />
                        <span>Stop</span>
                      </>
                    ) : (
                      <>
                        <Play size={12} fill="currentColor" />
                        <span>Run</span>
                      </>
                    )}
                  </button>

                  <button
                    className="btn-target-subaction"
                    onClick={() => onReloadTarget(target.id)}
                    title="Hot Reload Target"
                  >
                    <Zap size={12} color="#f59e0b" />
                    <span>Reload</span>
                  </button>

                  <button
                    className="btn-target-subaction"
                    onClick={() => onRestartTarget(target.id)}
                    title="Restart Target Process"
                  >
                    <RotateCw size={12} color="#06b6d4" />
                    <span>Restart</span>
                  </button>

                  {onOpenDevOptions && (
                    <button
                      className="btn-target-subaction"
                      onClick={() => onOpenDevOptions(target.id)}
                      title="Runner & Dev Server Options"
                    >
                      <Sliders size={12} color="#38bdf8" />
                      <span>Options</span>
                    </button>
                  )}

                  <button
                    className="btn-target-subaction primary-link"
                    onClick={() => {
                      onOpenTargetPane(target);
                      onSwitchToTerminal();
                    }}
                    title="View Logs in Terminal Tab"
                  >
                    <Terminal size={12} />
                    <span>Logs</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
