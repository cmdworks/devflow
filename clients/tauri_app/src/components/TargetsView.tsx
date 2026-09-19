import React from "react";
import {
  FolderGit2,
  Play,
  Square,
  Zap,
  RotateCw,
  Terminal,
  Layers,
} from "lucide-react";
import type { ProjectTarget, PaneInfo, ActiveSessionInfo } from "../types";

interface TargetsViewProps {
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activeSessions: ActiveSessionInfo[];
  workspaceName: string;
  workspacePath: string;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onSwitchToTerminal: (paneId?: string) => void;
}

export const TargetsView: React.FC<TargetsViewProps> = ({
  targets,
  openPanes,
  activeSessions,
  workspaceName,
  workspacePath,
  onToggleRun,
  onReload,
  onRestart,
  onOpenTargetPane,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onSwitchToTerminal,
}) => {
  return (
    <div className="view-container">
      {/* Header Banner */}
      <div className="view-header">
        <div>
          <div className="view-title-group">
            <FolderGit2 size={22} color="#06b6d4" />
            <h1 className="view-title">Workspace Targets & Process Matrix</h1>
            <span className="view-badge">{targets.length} Targets</span>
          </div>
          <p className="view-subtitle">
            Subprojects discovered in <span className="font-mono text-cyan">{workspaceName}</span> ({workspacePath})
          </p>
        </div>

        <div className="view-actions">
          <button className="btn-primary" onClick={onRunAll} title="Run all targets in parallel">
            <Play size={13} fill="currentColor" />
            <span>Run All</span>
          </button>

          <button className="btn-glass" onClick={onReloadAll} title="Hot reload all active sessions">
            <Zap size={13} color="#f59e0b" />
            <span>Reload All</span>
          </button>

          <button className="btn-glass" onClick={onRestartAll} title="Restart all active sessions">
            <RotateCw size={13} color="#06b6d4" />
            <span>Restart All</span>
          </button>

          <button className="btn-danger" onClick={onStopAll} title="Stop all running processes">
            <Square size={13} fill="currentColor" />
            <span>Stop All</span>
          </button>
        </div>
      </div>

      {/* Target Matrix Cards */}
      <div className="section-title">
        <Layers size={15} color="#06b6d4" />
        <span>Discovered Project Targets ({targets.length})</span>
      </div>

      {targets.length === 0 ? (
        <div className="empty-card-container">
          <FolderGit2 size={32} color="#64748b" />
          <div className="empty-title">No Targets Discovered</div>
          <p className="empty-desc">
            No supported framework projects (Swift, Kotlin, Rust, Flutter, React Native, Tauri) were discovered in this directory.
          </p>
        </div>
      ) : (
        <div className="targets-cards-grid">
          {targets.map((target) => {
            const pane = openPanes.find((p) => p.targetId === target.id);
            const status = pane ? pane.status : "idle";
            const isRunning = status === "running";
            const isBuilding = status === "building";
            const isError = status === "error";

            const activeSession = activeSessions.find(
              (s) => s.session_id === target.id || s.project_name.toLowerCase() === target.name.toLowerCase()
            );

            const platformClass = (target.platform || "generic").toLowerCase();

            return (
              <div key={target.id} className={`target-card ${status}-border`}>
                <div className="target-card-top">
                  <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <div className="target-icon-wrap">
                      <FolderGit2 size={18} color="#06b6d4" />
                    </div>
                    <div>
                      <div className="target-title-row">
                        <span className="target-title">{target.name}</span>
                        {target.is_default && <span className="default-pill">PRIMARY</span>}
                      </div>
                      <div style={{ display: "flex", alignItems: "center", gap: "6px", marginTop: "2px" }}>
                        <span className={`platform-tag ${platformClass}`}>{target.platform}</span>
                        <span className="framework-tag">{target.framework}</span>
                      </div>
                    </div>
                  </div>

                  <div className="target-status-badge">
                    <span
                      className={`pulse-dot ${
                        isRunning ? "green" : isBuilding ? "yellow" : isError ? "red" : "gray"
                      }`}
                    />
                    <span style={{ fontSize: "11px", fontWeight: 600 }}>
                      {isBuilding ? "Building..." : isRunning ? "Running" : isError ? "Error" : "Idle"}
                    </span>
                  </div>
                </div>

                <div className="target-card-body">
                  <div className="card-meta-row">
                    <span className="meta-label">Path</span>
                    <span className="meta-value font-mono text-truncate" title={target.path}>
                      {target.path}
                    </span>
                  </div>

                  {activeSession && (
                    <>
                      <div className="card-meta-row">
                        <span className="meta-label">Process PID</span>
                        <span className="meta-value font-mono">{activeSession.pid}</span>
                      </div>
                      <div className="card-meta-row">
                        <span className="meta-label">Socket</span>
                        <span className="meta-value font-mono text-truncate" title={activeSession.socket_path}>
                          {activeSession.socket_path}
                        </span>
                      </div>
                    </>
                  )}
                </div>

                <div className="target-card-actions">
                  <button
                    className={`target-btn-action ${isRunning || isBuilding ? "stop-btn" : "run-btn"}`}
                    onClick={() => onToggleRun(target.id)}
                    title={isRunning || isBuilding ? "Stop Target" : "Run Target"}
                  >
                    {isRunning || isBuilding ? (
                      <>
                        <Square size={13} fill="currentColor" />
                        <span>Stop</span>
                      </>
                    ) : (
                      <>
                        <Play size={13} fill="currentColor" />
                        <span>Run</span>
                      </>
                    )}
                  </button>

                  <button
                    className="target-btn-action glass"
                    onClick={() => onReload(target.id)}
                    disabled={!isRunning}
                    title="Trigger Framework Hot Reload"
                  >
                    <Zap size={13} color="#f59e0b" />
                    <span>Reload</span>
                  </button>

                  <button
                    className="target-btn-action glass"
                    onClick={() => onRestart(target.id)}
                    disabled={!isRunning}
                    title="Restart Application Process"
                  >
                    <RotateCw size={13} color="#06b6d4" />
                    <span>Restart</span>
                  </button>

                  <button
                    className="target-btn-action terminal-jump"
                    onClick={() => {
                      onOpenTargetPane(target);
                      onSwitchToTerminal(`pane-${target.id}`);
                    }}
                    title="Open in Terminal Tab & View Live Logs"
                  >
                    <Terminal size={13} />
                    <span>Logs</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
