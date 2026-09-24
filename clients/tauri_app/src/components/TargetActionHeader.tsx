import React from "react";
import {
  Play,
  Square,
  Zap,
  RotateCw,
  Package,
  Trash2,
  ExternalLink,
  Sliders,
  Globe,
  FolderOpen,
  CheckCircle2,
  Smartphone,
  AppWindow,
  PowerOff,
  Unplug,
  Loader2,
} from "lucide-react";
import type { PaneInfo, ProjectTarget, ActiveSessionInfo } from "../types";

interface TargetActionHeaderProps {
  pane: PaneInfo;
  target?: ProjectTarget;
  activeSession?: ActiveSessionInfo;
  executingAction?: string;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onExecuteAction: (targetId: string, action: string, port?: number) => void;
  onOpenDevOptions?: (targetId?: string) => void;
}

export const TargetActionHeader: React.FC<TargetActionHeaderProps> = ({
  pane,
  target,
  activeSession,
  executingAction,
  onToggleRun,
  onReload,
  onRestart,
  onExecuteAction,
  onOpenDevOptions,
}) => {
  const isCombined = pane.isCombined;
  const targetId = pane.targetId;
  const status = pane.status;
  const isRunning = status === "running";
  const isBuilding = status === "building";
  const fw = (target?.framework || pane.framework || "generic").toLowerCase();

  const isAndroid = fw.includes("kotlin") || fw.includes("android");
  const isSwift = fw.includes("swift") || fw.includes("apple") || fw.includes("xcode");
  const isRust = fw.includes("rust") || fw.includes("tauri") || fw.includes("cargo");
  const isWeb =
    fw.includes("web") ||
    fw.includes("react") ||
    fw.includes("node") ||
    fw.includes("generic") ||
    fw.includes("next");

  if (isCombined) {
    return (
      <div className="target-action-header combined-header">
        <div className="target-header-meta">
          <div className="target-header-title-wrap">
            <span className="target-header-dot green" />
            <span className="target-header-title">Combined Stream</span>
            <span className="target-header-badge universal">UNIVERSAL AGGREGATOR</span>
          </div>
          <span className="target-header-sub">
            Real-time multi-target stdout/stderr & diagnostic logs
          </span>
        </div>
      </div>
    );
  }

  const isExecuting = (action: string) => executingAction === action;

  return (
    <div className="target-action-header">
      {/* 1. Target Metadata Identity */}
      <div className="target-header-meta">
        <div className="target-header-title-wrap">
          <span
            className={`target-header-dot ${
              isRunning ? "green" : isBuilding ? "yellow" : status === "error" ? "red" : "gray"
            }`}
          />
          <span className="target-header-title">{pane.title}</span>
          <span className={`target-header-badge ${pane.platform.toLowerCase()}`}>
            {pane.platform}
          </span>
          <span className="target-header-framework">{pane.framework}</span>
        </div>

        <div className="target-header-details">
          {activeSession && activeSession.pid && (
            <span className="target-detail-pill">
              PID: <strong className="font-mono">{activeSession.pid}</strong>
            </span>
          )}
          {target?.path && (
            <span className="target-detail-pill font-mono text-truncate" title={target.path}>
              {target.path.split("/").slice(-2).join("/")}
            </span>
          )}
        </div>
      </div>

      {/* 2. Primary & Framework-Specific Action Toolbar */}
      <div className="target-header-actions">
        {/* Core Run / Stop Lifecycle */}
        <div className="action-button-group primary-lifecycle">
          <button
            className={`action-btn-pill ${isRunning || isBuilding ? "stop-btn" : "run-btn"}`}
            onClick={() => onToggleRun(targetId)}
            title={isRunning || isBuilding ? "Stop Target Process" : "Build & Run Target"}
          >
            {isRunning || isBuilding ? (
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

          {/* Hot Reload */}
          <button
            className="action-btn-pill glass"
            onClick={() => onReload(targetId)}
            disabled={!isRunning}
            title="Trigger Framework Hot Reload"
          >
            <Zap size={12} color="#f59e0b" />
            <span>Reload</span>
          </button>

          {/* Restart */}
          <button
            className="action-btn-pill glass"
            onClick={() => onRestart(targetId)}
            disabled={!isRunning}
            title="Restart Target Process"
          >
            <RotateCw size={12} color="#06b6d4" />
            <span>Restart</span>
          </button>
        </div>

        <div className="action-toolbar-divider" />

        {/* --- Framework-Specific Dedicated Tools --- */}
        <div className="action-button-group framework-tools">
          {/* 1. Android / Kotlin Suite */}
          {isAndroid && (
            <>
              <button
                className={`action-btn-pill glass-emerald ${isExecuting("reinstall_apk") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "reinstall_apk")}
                disabled={Boolean(executingAction)}
                title="Compile & Install APK on Android Device (gradle installDebug)"
              >
                {isExecuting("reinstall_apk") ? (
                  <Loader2 size={12} className="animate-spin text-emerald" />
                ) : (
                  <Package size={12} color="#10b981" />
                )}
                <span>Install APK</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("launch_app") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "launch_app")}
                disabled={Boolean(executingAction)}
                title="Launch App Activity on Device via ADB (am start)"
              >
                {isExecuting("launch_app") ? (
                  <Loader2 size={12} className="animate-spin text-emerald" />
                ) : (
                  <Smartphone size={12} color="#10b981" />
                )}
                <span>Open App</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("force_stop") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "force_stop")}
                disabled={Boolean(executingAction)}
                title="Force Stop Android Package on Device (am force-stop)"
              >
                {isExecuting("force_stop") ? (
                  <Loader2 size={12} className="animate-spin text-rose" />
                ) : (
                  <PowerOff size={12} color="#f43f5e" />
                )}
                <span>Kill App</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("clear_data") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "clear_data")}
                disabled={Boolean(executingAction)}
                title="Clear App Data & Cache (pm clear)"
              >
                {isExecuting("clear_data") ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <Trash2 size={12} color="#94a3b8" />
                )}
                <span>Clear Data</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("open_ide") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "open_ide")}
                disabled={Boolean(executingAction)}
                title="Open Project in Android Studio"
              >
                {isExecuting("open_ide") ? (
                  <Loader2 size={12} className="animate-spin text-cyan" />
                ) : (
                  <ExternalLink size={12} color="#38bdf8" />
                )}
                <span>Studio</span>
              </button>
            </>
          )}

          {/* 2. Swift / Apple Suite */}
          {isSwift && (
            <>
              <button
                className={`action-btn-pill glass-orange ${isExecuting("launch_app") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "launch_app")}
                disabled={Boolean(executingAction)}
                title="Launch Swift Binary / App Bundle"
              >
                {isExecuting("launch_app") ? (
                  <Loader2 size={12} className="animate-spin text-orange" />
                ) : (
                  <AppWindow size={12} color="#f97316" />
                )}
                <span>Open App</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("force_stop") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "force_stop")}
                disabled={Boolean(executingAction)}
                title="Terminate App Process"
              >
                {isExecuting("force_stop") ? (
                  <Loader2 size={12} className="animate-spin text-rose" />
                ) : (
                  <PowerOff size={12} color="#f43f5e" />
                )}
                <span>Close App</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("clean_cache") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "clean_cache")}
                disabled={Boolean(executingAction)}
                title="Clean Swift Package Build Cache"
              >
                {isExecuting("clean_cache") ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <Trash2 size={12} color="#94a3b8" />
                )}
                <span>Clean</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("open_ide") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "open_ide")}
                disabled={Boolean(executingAction)}
                title="Open Project in Xcode"
              >
                {isExecuting("open_ide") ? (
                  <Loader2 size={12} className="animate-spin text-cyan" />
                ) : (
                  <ExternalLink size={12} color="#38bdf8" />
                )}
                <span>Xcode</span>
              </button>
            </>
          )}

          {/* 3. Rust / Desktop Suite */}
          {isRust && (
            <>
              <button
                className={`action-btn-pill glass ${isExecuting("clippy_scan") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "clippy_scan")}
                disabled={Boolean(executingAction)}
                title="Run Cargo Clippy Linter"
              >
                {isExecuting("clippy_scan") ? (
                  <Loader2 size={12} className="animate-spin text-amber" />
                ) : (
                  <CheckCircle2 size={12} color="#f59e0b" />
                )}
                <span>Clippy</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("reveal_finder") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "reveal_finder")}
                disabled={Boolean(executingAction)}
                title="Reveal Binary Directory in macOS Finder"
              >
                {isExecuting("reveal_finder") ? (
                  <Loader2 size={12} className="animate-spin text-cyan" />
                ) : (
                  <FolderOpen size={12} color="#38bdf8" />
                )}
                <span>Finder</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("open_ide") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "open_ide")}
                disabled={Boolean(executingAction)}
                title="Open in VS Code"
              >
                {isExecuting("open_ide") ? (
                  <Loader2 size={12} className="animate-spin text-cyan" />
                ) : (
                  <ExternalLink size={12} color="#38bdf8" />
                )}
                <span>IDE</span>
              </button>
            </>
          )}

          {/* 4. Web / Node Suite */}
          {isWeb && (
            <>
              <button
                className={`action-btn-pill glass-blue ${isExecuting("open_browser") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "open_browser")}
                disabled={Boolean(executingAction)}
                title="Open in Default Web Browser"
              >
                {isExecuting("open_browser") ? (
                  <Loader2 size={12} className="animate-spin text-blue" />
                ) : (
                  <Globe size={12} color="#3b82f6" />
                )}
                <span>Browser</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("free_port") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "free_port")}
                disabled={Boolean(executingAction)}
                title="Kill Conflicting Port Listener"
              >
                {isExecuting("free_port") ? (
                  <Loader2 size={12} className="animate-spin text-rose" />
                ) : (
                  <Unplug size={12} color="#ef4444" />
                )}
                <span>Free Port</span>
              </button>

              <button
                className={`action-btn-pill glass ${isExecuting("clean_cache") ? "loading" : ""}`}
                onClick={() => onExecuteAction(targetId, "clean_cache")}
                disabled={Boolean(executingAction)}
                title="Clean Vite / Next Cache"
              >
                {isExecuting("clean_cache") ? (
                  <Loader2 size={12} className="animate-spin" />
                ) : (
                  <Trash2 size={12} color="#94a3b8" />
                )}
                <span>Clean Cache</span>
              </button>
            </>
          )}

          {/* Runner Options Modal */}
          {onOpenDevOptions && (
            <button
              className="action-btn-pill glass options-btn"
              onClick={() => onOpenDevOptions(targetId)}
              title="Configure Target Runner & Dev Server Options"
            >
              <Sliders size={12} color="#38bdf8" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
