import React from "react";
import {
  Layers,
  Play,
  Zap,
  RotateCw,
  Square,
  Activity,
  Smartphone,
  ChevronRight,
  Settings,
} from "lucide-react";
import type { ViewSection, Device } from "../types";

interface TopBarProps {
  workspaceName: string;
  activeSection: ViewSection;
  devices: Device[];
  onSelectSection: (section: ViewSection) => void;
  onRunAll: () => void;
  onReloadAll: () => void;
  onRestartAll: () => void;
  onStopAll: () => void;
  onOpenSettings: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  workspaceName,
  activeSection,
  devices,
  onSelectSection,
  onRunAll,
  onReloadAll,
  onRestartAll,
  onStopAll,
  onOpenSettings,
}) => {
  const onlineDevicesCount = devices.filter(
    (d) => d.online || d.state === "device" || d.state === "booted"
  ).length;

  const sectionLabelMap: Record<ViewSection, string> = {
    overview: "Overview",
    targets: "Targets & Matrix",
    terminal: "Terminal Logs",
    devices: "Devices & Emulators",
    doctor: "Doctor Diagnostics",
    settings: "Settings",
  };

  return (
    <header className="top-bar">
      {/* 1. Left: Brand & Breadcrumb */}
      <div className="top-bar-left">
        <div className="app-brand" onClick={() => onSelectSection("overview")} style={{ cursor: "pointer" }}>
          <Layers size={16} color="#06b6d4" />
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
          className="btn-top-action primary"
          onClick={onRunAll}
          title="Run all discovered workspace targets"
        >
          <Play size={12} fill="currentColor" />
          <span>Run All</span>
        </button>

        <button
          className="btn-top-action"
          onClick={onReloadAll}
          title="Hot reload all active running sessions"
        >
          <Zap size={12} color="#f59e0b" />
          <span>Reload</span>
        </button>

        <button
          className="btn-top-action"
          onClick={onRestartAll}
          title="Restart all active apps"
        >
          <RotateCw size={12} color="#06b6d4" />
          <span>Restart</span>
        </button>

        <button
          className="btn-top-action danger"
          onClick={onStopAll}
          title="Stop all running processes"
        >
          <Square size={12} fill="currentColor" />
          <span>Stop All</span>
        </button>
      </div>

      {/* 3. Right: System & Tool Status Pills */}
      <div className="top-bar-right">
        {/* Device Status Pill */}
        <button
          className={`top-status-pill ${activeSection === "devices" ? "active" : ""}`}
          onClick={() => onSelectSection("devices")}
          title="View Connected Devices & AVD Launcher"
        >
          <Smartphone size={12} color="#38bdf8" />
          <span>{onlineDevicesCount} Device{onlineDevicesCount === 1 ? "" : "s"}</span>
          <span className={`pulse-dot ${onlineDevicesCount > 0 ? "green" : "yellow"}`} />
        </button>

        {/* Doctor Status Pill */}
        <button
          className={`top-status-pill ${activeSection === "doctor" ? "active" : ""}`}
          onClick={() => onSelectSection("doctor")}
          title="View Toolchain Health Diagnostics"
        >
          <Activity size={12} color="#10b981" />
          <span>Doctor</span>
        </button>

        <div className="top-bar-divider" />

        {/* Settings Action */}
        <button
          className="btn-icon-top"
          onClick={onOpenSettings}
          title="DevFlow Preferences & CLI Setup"
        >
          <Settings size={14} />
        </button>
      </div>
    </header>
  );
};
