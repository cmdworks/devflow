import React from "react";
import {
  Layers,
  Play,
  Zap,
  RotateCw,
  Square,
  Activity,
  Terminal as TerminalIcon,
  Folder,
  ChevronDown,
} from "lucide-react";
import type { Device } from "../types";

interface TopBarProps {
  workspaceName: string;
  workspacePath: string;
  devices: Device[];
  onOpenWorkspaceManager: () => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onOpenDoctor: () => void;
  onInstallCli: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  workspaceName,
  workspacePath,
  devices,
  onOpenWorkspaceManager,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onOpenDoctor,
  onInstallCli,
}) => {
  return (
    <header className="top-bar">
      <div className="top-bar-left">
        <div className="app-brand">
          <Layers size={18} color="#06b6d4" />
          <span>DevFlow</span>
          <span className="app-brand-badge">Companion</span>
        </div>

        <button
          className="workspace-badge-btn"
          onClick={onOpenWorkspaceManager}
          title="Click to open Workspace Manager (switch or add workspace)"
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            padding: "4px 10px",
            background: "rgba(30, 41, 59, 0.7)",
            border: "1px solid var(--border-color)",
            borderRadius: "6px",
            cursor: "pointer",
            transition: "all 0.15s ease",
          }}
        >
          <Folder size={14} color="#06b6d4" />
          <span style={{ fontWeight: 600, color: "#fff" }}>{workspaceName || "Workspace"}</span>
          <span style={{ color: "var(--text-muted)", fontSize: "11px" }}>
            ({workspacePath.length > 24 ? `...${workspacePath.slice(-20)}` : workspacePath || "none"})
          </span>
          <ChevronDown size={13} color="var(--text-muted)" />
        </button>

        <div className="top-devices-bar">
          {devices.length === 0 ? (
            <div className="device-pill">
              <span className="pulse-dot yellow" />
              <span>No devices detected</span>
            </div>
          ) : (
            devices.map((d) => {
              const isOnline = d.state === "Connected" || d.state === "connected" || d.state === "Booted" || d.online;
              return (
                <div
                  key={d.id}
                  className="device-pill"
                  title={`ID: ${d.id} | Platform: ${d.platform} | State: ${d.state}`}
                >
                  <span className={`pulse-dot ${isOnline ? "green" : "gray"}`} />
                  <span>
                    {d.name} [{d.platform}]
                  </span>
                </div>
              );
            })
          )}
        </div>
      </div>

      <div className="top-bar-right">
        <button className="btn-top-action primary" onClick={onRunAll} title="Run all workspace targets">
          <Play size={13} fill="currentColor" />
          <span>Run All</span>
        </button>

        <button className="btn-top-action" onClick={onReloadAll} title="Hot reload all active sessions">
          <Zap size={13} color="#f59e0b" />
          <span>Reload</span>
        </button>

        <button className="btn-top-action" onClick={onRestartAll} title="Restart all active apps">
          <RotateCw size={13} color="#06b6d4" />
          <span>Restart</span>
        </button>

        <button className="btn-top-action danger" onClick={onStopAll} title="Stop all running targets">
          <Square size={13} fill="currentColor" />
          <span>Stop All</span>
        </button>

        <button className="btn-top-action" onClick={onOpenDoctor} title="Check environment health & diagnostics">
          <Activity size={13} color="#10b981" />
          <span>Doctor</span>
        </button>

        <button className="btn-top-action" onClick={onInstallCli} title="Install 'devflow' symlink into system PATH">
          <TerminalIcon size={13} color="#a855f7" />
          <span>Shell CLI</span>
        </button>
      </div>
    </header>
  );
};
