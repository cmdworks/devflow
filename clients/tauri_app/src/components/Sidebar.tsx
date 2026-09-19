import React, { useState } from "react";
import {
  FolderGit2,
  Smartphone,
  Cpu,
  Radio,
  Plus,
  Play,
  RotateCcw,
  FolderPlus,
} from "lucide-react";
import type { ProjectTarget, Device, ActiveSessionInfo, PaneInfo } from "../types";

interface SidebarProps {
  targets: ProjectTarget[];
  devices: Device[];
  activeSessions: ActiveSessionInfo[];
  openPanes: PaneInfo[];
  onOpenWorkspaceManager?: () => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
  onRefreshDevices: () => void;
  onBootEmulator: (name: string) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  targets,
  devices,
  activeSessions,
  openPanes,
  onOpenWorkspaceManager,
  onOpenTargetPane,
  onOpenCombinedPane,
  onRefreshDevices,
  onBootEmulator,
}) => {
  const [emuInput, setEmuInput] = useState("");

  const handleBoot = () => {
    const trimmed = emuInput.trim();
    if (trimmed) {
      onBootEmulator(trimmed);
      setEmuInput("");
    }
  };

  return (
    <aside className="sidebar">
      {/* Sub-Projects Section */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <FolderGit2 size={13} color="#06b6d4" />
            <span>Targets</span>
          </div>
          <span className="target-badge-count">{targets.length}</span>
        </div>

        {targets.length === 0 ? (
          <div style={{ color: "var(--text-muted)", fontSize: "11px", padding: "6px 0" }}>
            No recognized project targets
          </div>
        ) : (
          targets.map((target) => {
            const isOpen = openPanes.some((p) => p.targetId === target.id);
            const activePane = openPanes.find((p) => p.targetId === target.id);
            const status = activePane ? activePane.status : "idle";
            const platformClass = (target.platform || "generic").toLowerCase();

            return (
              <div
                key={target.id}
                className={`subproject-card ${isOpen ? "active-in-pane" : ""}`}
                onClick={() => onOpenTargetPane(target)}
                title={`Click to open pane for ${target.name} (${target.path})`}
              >
                <div className="subproject-info">
                  <div className="subproject-title">{target.name}</div>
                  <div className="subproject-meta">
                    <span className={`platform-tag ${platformClass}`}>{target.platform}</span>
                    <span className={`pulse-dot ${status === "running" ? "green" : status === "building" ? "yellow" : status === "error" ? "red" : "gray"}`} />
                    <span>{target.framework}</span>
                  </div>
                </div>
                <button
                  className="subproject-action-btn"
                  style={{
                    background: "transparent",
                    border: "none",
                    color: "var(--text-muted)",
                    cursor: "pointer",
                    fontSize: "13px",
                  }}
                  title="Open Pane"
                >
                  <Plus size={14} />
                </button>
              </div>
            );
          })
        )}

        <button className="sidebar-btn-combine" onClick={onOpenCombinedPane}>
          <Radio size={13} />
          <span>Combine All Streams</span>
        </button>

        {onOpenWorkspaceManager && (
          <button
            className="sidebar-btn-combine"
            style={{
              marginTop: "6px",
              background: "rgba(6, 182, 212, 0.08)",
              borderColor: "rgba(6, 182, 212, 0.3)",
              color: "#06b6d4",
            }}
            onClick={onOpenWorkspaceManager}
          >
            <FolderPlus size={13} />
            <span style={{ fontWeight: 600 }}>Switch / Add Workspace</span>
          </button>
        )}
      </div>

      {/* Connected Devices & Emulators */}
      <div className="sidebar-section">
        <div className="sidebar-section-header">
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <Smartphone size={13} color="#3b82f6" />
            <span>Devices & Emulators</span>
          </div>
          <button
            onClick={onRefreshDevices}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-muted)",
              cursor: "pointer",
            }}
            title="Refresh Devices"
          >
            <RotateCcw size={12} />
          </button>
        </div>

        {devices.length === 0 ? (
          <div style={{ color: "var(--text-muted)", fontSize: "11px", padding: "4px 0" }}>
            No devices discovered
          </div>
        ) : (
          devices.map((d) => {
            const isOnline = d.state === "Connected" || d.state === "connected" || d.state === "Booted" || d.online;
            return (
              <div key={d.id} className="sidebar-device-item">
                <div style={{ display: "flex", alignItems: "center", gap: "6px", overflow: "hidden" }}>
                  <span className={`pulse-dot ${isOnline ? "green" : "gray"}`} />
                  <span style={{ fontSize: "11.5px", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                    {d.name}
                  </span>
                </div>
                <span className="platform-tag" style={{ fontSize: "9px" }}>
                  {d.platform}
                </span>
              </div>
            );
          })
        )}

        <div className="sidebar-emu-boot-group">
          <input
            type="text"
            className="sidebar-emu-input"
            placeholder="AVD Name (e.g. Pixel_7)"
            value={emuInput}
            onChange={(e) => setEmuInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleBoot()}
          />
          <button className="sidebar-btn-boot" onClick={handleBoot} title="Boot Android Emulator">
            <Play size={11} fill="currentColor" />
          </button>
        </div>
      </div>

      {/* External Background Sessions */}
      <div className="sidebar-section" style={{ borderBottom: "none" }}>
        <div className="sidebar-section-header">
          <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <Cpu size={13} color="#8b5cf6" />
            <span>Active Sessions</span>
          </div>
          <span className="target-badge-count">{activeSessions.length}</span>
        </div>

        {activeSessions.length === 0 ? (
          <div style={{ color: "var(--text-muted)", fontSize: "11px", padding: "4px 0" }}>
            No external processes
          </div>
        ) : (
          activeSessions.map((s) => (
            <div
              key={s.session_id}
              className="sidebar-device-item"
              title={`PID: ${s.pid} | Socket: ${s.socket_path}`}
            >
              <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                <span className="pulse-dot blue" />
                <span style={{ fontSize: "11px" }}>{s.project_name}</span>
              </div>
              <span style={{ color: "var(--text-muted)", fontSize: "10px", fontFamily: "var(--font-mono)" }}>
                PID {s.pid}
              </span>
            </div>
          ))
        )}
      </div>
    </aside>
  );
};
