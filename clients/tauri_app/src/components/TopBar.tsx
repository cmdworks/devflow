import React from "react";
import {
  Layers,
  Play,
  Zap,
  RotateCw,
  Square,
  Activity,
  Terminal,
  Smartphone,
  FolderGit2,
} from "lucide-react";
import type { ViewMode } from "./Sidebar";

interface TopBarProps {
  activeView: ViewMode;
  onSelectView: (view: ViewMode) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onInstallCli: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  activeView,
  onSelectView,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onInstallCli,
}) => {
  return (
    <header className="top-bar">
      {/* Left: Brand + Navigation View Mode Switcher */}
      <div className="top-bar-left">
        <div className="app-brand">
          <Layers size={17} color="#06b6d4" />
          <span>DevFlow</span>
          <span className="app-brand-badge">PRO</span>
        </div>

        <nav className="top-nav-tabs">
          <button
            className={`top-nav-btn ${activeView === "terminal" ? "active" : ""}`}
            onClick={() => onSelectView("terminal")}
            title="Terminal Log Streams (Termax Web Engine)"
          >
            <Terminal size={13} />
            <span>Terminal</span>
          </button>

          <button
            className={`top-nav-btn ${activeView === "targets" ? "active" : ""}`}
            onClick={() => onSelectView("targets")}
            title="Workspace Targets & Subprojects"
          >
            <FolderGit2 size={13} />
            <span>Targets</span>
          </button>

          <button
            className={`top-nav-btn ${activeView === "devices" ? "active" : ""}`}
            onClick={() => onSelectView("devices")}
            title="Devices & Emulators Hub"
          >
            <Smartphone size={13} />
            <span>Devices</span>
          </button>

          <button
            className={`top-nav-btn ${activeView === "doctor" ? "active" : ""}`}
            onClick={() => onSelectView("doctor")}
            title="Toolchain & Environment Diagnostics"
          >
            <Activity size={13} />
            <span>Doctor</span>
          </button>
        </nav>
      </div>

      {/* Right: Global Process Lifecycle & System Actions */}
      <div className="top-bar-right">
        <button className="btn-top-action primary" onClick={onRunAll} title="Run all workspace targets">
          <Play size={12} fill="currentColor" />
          <span>Run All</span>
        </button>

        <button className="btn-top-action" onClick={onReloadAll} title="Hot reload all active target sessions">
          <Zap size={12} color="#f59e0b" />
          <span>Reload</span>
        </button>

        <button className="btn-top-action" onClick={onRestartAll} title="Restart all active apps">
          <RotateCw size={12} color="#06b6d4" />
          <span>Restart</span>
        </button>

        <button className="btn-top-action danger" onClick={onStopAll} title="Stop all running targets">
          <Square size={12} fill="currentColor" />
          <span>Stop All</span>
        </button>

        <div className="top-bar-divider" />

        <button
          className="btn-top-action glass-btn"
          onClick={onInstallCli}
          title="Install / Link 'devflow' CLI into system PATH (/usr/local/bin)"
        >
          <Terminal size={12} color="#a855f7" />
          <span>Shell CLI</span>
        </button>
      </div>
    </header>
  );
};
