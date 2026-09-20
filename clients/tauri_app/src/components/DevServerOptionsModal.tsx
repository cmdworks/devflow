import React, { useState } from "react";
import {
  Sliders,
  X,
  Play,
  Zap,
  RotateCw,
  Square,
  Flame,
  CheckCircle2,
  Cpu,
} from "lucide-react";
import type { ProjectTarget } from "../types";

export interface RunnerOptionsConfig {
  mode: "dev" | "clean" | "release" | "test" | "force-kill";
  port?: string;
  customArgs?: string;
  openLogs?: boolean;
}

interface DevServerOptionsModalProps {
  isOpen: boolean;
  onClose: () => void;
  targets: ProjectTarget[];
  initialTargetId?: string;
  onLaunchWithOptions: (targetId: string, options: RunnerOptionsConfig) => Promise<void>;
  onStopTarget: (targetId: string, force?: boolean) => Promise<void>;
}

export const DevServerOptionsModal: React.FC<DevServerOptionsModalProps> = ({
  isOpen,
  onClose,
  targets,
  initialTargetId,
  onLaunchWithOptions,
  onStopTarget,
}) => {
  const [selectedTargetId, setSelectedTargetId] = useState<string>(
    initialTargetId || targets[0]?.id || "all"
  );
  const [runnerMode, setRunnerMode] = useState<RunnerOptionsConfig["mode"]>("dev");
  const [customPort, setCustomPort] = useState<string>("");
  const [customArgs, setCustomArgs] = useState<string>("");
  const [isProcessing, setIsProcessing] = useState(false);

  if (!isOpen) return null;

  const currentTarget = targets.find((t) => t.id === selectedTargetId);

  const handleExecute = async () => {
    setIsProcessing(true);
    try {
      if (runnerMode === "force-kill") {
        if (selectedTargetId === "all") {
          for (const t of targets) {
            await onStopTarget(t.id, true);
          }
        } else {
          await onStopTarget(selectedTargetId, true);
        }
      } else {
        await onLaunchWithOptions(selectedTargetId, {
          mode: runnerMode,
          port: customPort.trim() || undefined,
          customArgs: customArgs.trim() || undefined,
          openLogs: true,
        });
      }
      onClose();
    } catch (err) {
      console.error("Runner options error:", err);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content dev-options-modal-box" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="modal-header">
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <Sliders size={16} color="#06b6d4" />
            <h2 className="modal-title">Dev Server & Runner Options</h2>
          </div>
          <button className="btn-modal-close" onClick={onClose}>
            <X size={16} />
          </button>
        </div>

        {/* Body */}
        <div className="modal-body dev-options-body">
          {/* Target Selector */}
          <div className="options-field-group">
            <label className="options-field-label">Target Subproject</label>
            <select
              className="options-select"
              value={selectedTargetId}
              onChange={(e) => setSelectedTargetId(e.target.value)}
            >
              <option value="all">⚡ All Workspace Targets ({targets.length})</option>
              {targets.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name} [{t.framework} / {t.platform}]
                </option>
              ))}
            </select>
            {currentTarget && (
              <div className="target-quick-meta">
                <span className="font-mono">{currentTarget.path}</span>
              </div>
            )}
          </div>

          {/* Mode Selection Cards */}
          <div className="options-field-group">
            <label className="options-field-label">Execution & Runner Profile</label>
            <div className="runner-modes-grid">
              {/* 1. Dev Mode */}
              <div
                className={`runner-mode-card ${runnerMode === "dev" ? "selected" : ""}`}
                onClick={() => setRunnerMode("dev")}
              >
                <div className="runner-mode-header">
                  <Zap size={14} color="#38bdf8" />
                  <span className="runner-mode-title">Standard Dev Server</span>
                </div>
                <p className="runner-mode-desc">
                  Launches with Hot Reload, live file watcher, and streaming compiler logs.
                </p>
              </div>

              {/* 2. Clean & Rebuild */}
              <div
                className={`runner-mode-card ${runnerMode === "clean" ? "selected" : ""}`}
                onClick={() => setRunnerMode("clean")}
              >
                <div className="runner-mode-header">
                  <RotateCw size={14} color="#06b6d4" />
                  <span className="runner-mode-title">Clean Build & Run</span>
                </div>
                <p className="runner-mode-desc">
                  Cleans build artifacts and caches before compiling from scratch.
                </p>
              </div>

              {/* 3. Release Mode */}
              <div
                className={`runner-mode-card ${runnerMode === "release" ? "selected" : ""}`}
                onClick={() => setRunnerMode("release")}
              >
                <div className="runner-mode-header">
                  <Cpu size={14} color="#a855f7" />
                  <span className="runner-mode-title">Release / Prod Build</span>
                </div>
                <p className="runner-mode-desc">
                  Builds optimized production binaries without debugger overhead.
                </p>
              </div>

              {/* 4. Test & Run */}
              <div
                className={`runner-mode-card ${runnerMode === "test" ? "selected" : ""}`}
                onClick={() => setRunnerMode("test")}
              >
                <div className="runner-mode-header">
                  <CheckCircle2 size={14} color="#10b981" />
                  <span className="runner-mode-title">Test Suite & Run</span>
                </div>
                <p className="runner-mode-desc">
                  Executes unit and integration test gates before launching the app.
                </p>
              </div>

              {/* 5. Force Kill */}
              <div
                className={`runner-mode-card danger ${runnerMode === "force-kill" ? "selected" : ""}`}
                onClick={() => setRunnerMode("force-kill")}
              >
                <div className="runner-mode-header">
                  <Flame size={14} color="#ef4444" />
                  <span className="runner-mode-title">Force Kill (SIGKILL)</span>
                </div>
                <p className="runner-mode-desc">
                  Force-kills running app processes, simulator sessions, and port bindings.
                </p>
              </div>
            </div>
          </div>

          {/* Advanced Args & Ports */}
          {runnerMode !== "force-kill" && (
            <div className="options-field-row">
              <div className="options-field-col">
                <label className="options-field-label">Custom Port (Optional)</label>
                <input
                  type="text"
                  className="options-input"
                  placeholder="e.g. 3000, 8080"
                  value={customPort}
                  onChange={(e) => setCustomPort(e.target.value)}
                />
              </div>

              <div className="options-field-col">
                <label className="options-field-label">Extra CLI Args / Flags</label>
                <input
                  type="text"
                  className="options-input font-mono"
                  placeholder="e.g. --verbose --features=foo"
                  value={customArgs}
                  onChange={(e) => setCustomArgs(e.target.value)}
                />
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="modal-footer">
          <button className="btn-modal-cancel" onClick={onClose} disabled={isProcessing}>
            Cancel
          </button>

          <button
            className={`btn-modal-confirm ${runnerMode === "force-kill" ? "danger" : "primary"}`}
            onClick={handleExecute}
            disabled={isProcessing}
          >
            {runnerMode === "force-kill" ? (
              <>
                <Square size={12} fill="currentColor" />
                <span>{isProcessing ? "Killing..." : "Force Terminate"}</span>
              </>
            ) : (
              <>
                <Play size={12} fill="currentColor" />
                <span>{isProcessing ? "Starting..." : "Launch Target"}</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
