import React, { useState, useRef, useEffect } from "react";
import {
  FolderGit2,
  Play,
  Square,
  Zap,
  RotateCw,
  Terminal,
  Layers,
  Loader2,
  Sliders,
  Package,
  Trash2,
  Globe,
  FolderOpen,
  CheckCircle2,
  Smartphone,
  AppWindow,
  PowerOff,
  Unplug,
  ExternalLink,
  ChevronDown,
  Wrench,
} from "lucide-react";
import type { ProjectTarget, PaneInfo, ActiveSessionInfo } from "../types";

interface TargetsViewProps {
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activeSessions: ActiveSessionInfo[];
  workspaceName: string;
  workspacePath: string;
  isStartingAll?: boolean;
  isStoppingAll?: boolean;
  allTargetsRunning?: boolean;
  anyTargetRunning?: boolean;
  executingActions?: Record<string, string>;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onExecuteAction?: (targetId: string, action: string, port?: number) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onOpenDevOptions?: (targetId?: string) => void;
  onSwitchToTerminal: (paneId?: string) => void;
}

export const TargetsView: React.FC<TargetsViewProps> = ({
  targets,
  openPanes,
  activeSessions,
  workspaceName,
  workspacePath,
  isStartingAll = false,
  isStoppingAll = false,
  allTargetsRunning = false,
  anyTargetRunning = false,
  executingActions = {},
  onToggleRun,
  onReload,
  onRestart,
  onExecuteAction = () => {},
  onOpenTargetPane,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onOpenDevOptions,
  onSwitchToTerminal,
}) => {
  const [openToolsMenuTargetId, setOpenToolsMenuTargetId] = useState<string | null>(null);
  const toolsMenuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (toolsMenuRef.current && !toolsMenuRef.current.contains(e.target as Node)) {
        setOpenToolsMenuTargetId(null);
      }
    };
    if (openToolsMenuTargetId) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [openToolsMenuTargetId]);

  return (
    <div className="view-container">
      {/* 1. Header & Batch Actions */}
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
          <button
            className={`btn-primary ${isStartingAll ? "loading" : ""}`}
            onClick={onRunAll}
            disabled={isStartingAll || allTargetsRunning || targets.length === 0}
            title={
              isStartingAll
                ? "Starting workspace targets..."
                : allTargetsRunning
                ? "All targets are already running"
                : targets.length === 0
                ? "No targets found in workspace"
                : "Run all targets in parallel"
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
            className="btn-glass"
            onClick={onReloadAll}
            disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
            title={anyTargetRunning ? "Hot reload all active sessions" : "No active sessions to reload"}
          >
            <Zap size={13} color="#f59e0b" />
            <span>Reload All</span>
          </button>

          <button
            className="btn-glass"
            onClick={onRestartAll}
            disabled={!anyTargetRunning || isStartingAll || isStoppingAll}
            title={anyTargetRunning ? "Restart all active target processes" : "No active sessions to restart"}
          >
            <RotateCw size={13} color="#06b6d4" />
            <span>Restart All</span>
          </button>

          <button
            className={`btn-glass ${isStoppingAll ? "loading" : ""}`}
            onClick={onStopAll}
            disabled={!anyTargetRunning || isStoppingAll || isStartingAll}
            title={anyTargetRunning ? "Stop all running target processes" : "No running targets to stop"}
          >
            {isStoppingAll ? (
              <Loader2 size={13} className="animate-spin" />
            ) : (
              <Square size={13} fill="currentColor" />
            )}
            <span>{isStoppingAll ? "Stopping..." : "Stop All"}</span>
          </button>

          {onOpenDevOptions && (
            <button
              className="btn-glass dev-options-trigger"
              onClick={() => onOpenDevOptions()}
              title="Configure Dev Server & Runner Profiles"
            >
              <Sliders size={13} color="#38bdf8" />
              <span>Dev Options</span>
            </button>
          )}
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
            const fw = (target.framework || "").toLowerCase();
            const isAndroid = fw.includes("kotlin") || fw.includes("android");
            const isSwift = fw.includes("swift") || fw.includes("apple") || fw.includes("xcode");
            const isRust = fw.includes("rust") || fw.includes("tauri") || fw.includes("cargo");
            const isWeb =
              fw.includes("web") ||
              fw.includes("react") ||
              fw.includes("node") ||
              fw.includes("generic") ||
              fw.includes("next");

            const isCurrentExecuting = executingActions[target.id];
            const isToolsMenuOpen = openToolsMenuTargetId === target.id;

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

                {/* Primary Card Controls */}
                <div className="target-card-actions">
                  <button
                    className={`target-btn-action ${isRunning || isBuilding ? "stop-btn" : "run-btn"}`}
                    onClick={() => onToggleRun(target.id)}
                    title={isRunning || isBuilding ? "Stop Target Process" : "Build & Run Target"}
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

                  {/* Clean Tools Dropdown Popover */}
                  <div className="tools-dropdown-anchor" style={{ position: "relative" }}>
                    <button
                      className={`target-btn-action glass ${isToolsMenuOpen ? "active" : ""}`}
                      onClick={() => setOpenToolsMenuTargetId(isToolsMenuOpen ? null : target.id)}
                      title="Framework & Device Tools"
                    >
                      {isCurrentExecuting ? (
                        <Loader2 size={13} className="animate-spin text-cyan" />
                      ) : (
                        <Wrench size={13} color="#38bdf8" />
                      )}
                      <span>Tools</span>
                      <ChevronDown size={11} />
                    </button>

                    {isToolsMenuOpen && (
                      <div ref={toolsMenuRef} className="target-tools-popover">
                        <div className="tools-popover-header">
                          <span>{target.framework} Actions</span>
                        </div>

                        {isAndroid && (
                          <>
                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "reinstall_apk");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Package size={13} color="#10b981" />
                              <div className="popover-action-text">
                                <strong>Install APK</strong>
                                <span>Compile & push to device via Gradle</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "launch_app");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Smartphone size={13} color="#10b981" />
                              <div className="popover-action-text">
                                <strong>Launch App Activity</strong>
                                <span>Start launcher activity on device</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item danger"
                              onClick={() => {
                                onExecuteAction(target.id, "force_stop");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <PowerOff size={13} color="#f43f5e" />
                              <div className="popover-action-text">
                                <strong>Kill App Process</strong>
                                <span>am force-stop package</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "clear_data");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Trash2 size={13} color="#94a3b8" />
                              <div className="popover-action-text">
                                <strong>Clear App Data & Cache</strong>
                                <span>pm clear package state</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "open_ide");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <ExternalLink size={13} color="#38bdf8" />
                              <div className="popover-action-text">
                                <strong>Open in Android Studio</strong>
                                <span>Launch project in Studio</span>
                              </div>
                            </button>
                          </>
                        )}

                        {isSwift && (
                          <>
                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "launch_app");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <AppWindow size={13} color="#f97316" />
                              <div className="popover-action-text">
                                <strong>Launch Swift App</strong>
                                <span>Run compiled binary / app</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item danger"
                              onClick={() => {
                                onExecuteAction(target.id, "force_stop");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <PowerOff size={13} color="#f43f5e" />
                              <div className="popover-action-text">
                                <strong>Terminate App</strong>
                                <span>Kill running Swift process</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "clean_cache");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Trash2 size={13} color="#94a3b8" />
                              <div className="popover-action-text">
                                <strong>Clean Build Cache</strong>
                                <span>swift package clean</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "open_ide");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <ExternalLink size={13} color="#38bdf8" />
                              <div className="popover-action-text">
                                <strong>Open in Xcode</strong>
                                <span>Launch project in Xcode IDE</span>
                              </div>
                            </button>
                          </>
                        )}

                        {isRust && (
                          <>
                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "clippy_scan");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <CheckCircle2 size={13} color="#f59e0b" />
                              <div className="popover-action-text">
                                <strong>Cargo Clippy</strong>
                                <span>Run Rust linter inspection</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "reveal_finder");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <FolderOpen size={13} color="#38bdf8" />
                              <div className="popover-action-text">
                                <strong>Reveal in Finder</strong>
                                <span>Open project in macOS Finder</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "open_ide");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <ExternalLink size={13} color="#38bdf8" />
                              <div className="popover-action-text">
                                <strong>Open in IDE</strong>
                                <span>Launch in VS Code / Cursor</span>
                              </div>
                            </button>
                          </>
                        )}

                        {isWeb && (
                          <>
                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "open_browser");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Globe size={13} color="#3b82f6" />
                              <div className="popover-action-text">
                                <strong>Open in Browser</strong>
                                <span>Launch default web browser</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item danger"
                              onClick={() => {
                                onExecuteAction(target.id, "free_port");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Unplug size={13} color="#ef4444" />
                              <div className="popover-action-text">
                                <strong>Free Conflicting Port</strong>
                                <span>Kill stuck dev server port listener</span>
                              </div>
                            </button>

                            <button
                              className="popover-action-item"
                              onClick={() => {
                                onExecuteAction(target.id, "clean_cache");
                                setOpenToolsMenuTargetId(null);
                              }}
                            >
                              <Trash2 size={13} color="#94a3b8" />
                              <div className="popover-action-text">
                                <strong>Clean Dev Cache</strong>
                                <span>Purge Vite / Next.js cache</span>
                              </div>
                            </button>
                          </>
                        )}
                      </div>
                    )}
                  </div>

                  {onOpenDevOptions && (
                    <button
                      className="target-btn-action glass"
                      onClick={() => onOpenDevOptions(target.id)}
                      title="Configure Runner / Dev Server Options"
                    >
                      <Sliders size={13} color="#38bdf8" />
                      <span>Options</span>
                    </button>
                  )}

                  <button
                    className="target-btn-action terminal-jump"
                    onClick={() => {
                      onOpenTargetPane(target);
                      onSwitchToTerminal(`pane-${target.id}`);
                    }}
                    title="Open in Live Cockpit & View Logs"
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
