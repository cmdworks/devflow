import React, { useState } from "react";
import {
  FolderPlus,
  Smartphone,
  Activity,
  Settings,
  Pencil,
  Info,
  Check,
  FolderGit2,
} from "lucide-react";
import type { KnownWorkspace, ViewSection, Device } from "../types";
import { WorkspaceInfoPopover } from "./WorkspaceInfoPopover";

interface PrimarySidebarProps {
  activeWorkspacePath: string;
  knownWorkspaces: KnownWorkspace[];
  activeSection: ViewSection;
  devices: Device[];
  onSelectWorkspace: (path: string) => void;
  onAddWorkspace: () => void;
  onSelectSection: (section: ViewSection) => void;
  onRenameWorkspace: (path: string, newName: string) => void;
  onRemoveWorkspace: (path: string) => void;
  onOpenSettings: () => void;
}

export const PrimarySidebar: React.FC<PrimarySidebarProps> = ({
  activeWorkspacePath,
  knownWorkspaces,
  activeSection,
  devices,
  onSelectWorkspace,
  onAddWorkspace,
  onSelectSection,
  onRenameWorkspace,
  onRemoveWorkspace,
  onOpenSettings,
}) => {
  const [renamingPath, setRenamingPath] = useState<string | null>(null);
  const [editNameValue, setEditNameValue] = useState("");
  const [popoverWorkspace, setPopoverWorkspace] = useState<KnownWorkspace | null>(null);
  const [popoverAnchor, setPopoverAnchor] = useState<DOMRect | null>(null);

  const startRename = (ws: KnownWorkspace, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    setRenamingPath(ws.path);
    setEditNameValue(ws.custom_name || ws.name);
  };

  const saveRename = (path: string) => {
    const trimmed = editNameValue.trim();
    if (trimmed) {
      onRenameWorkspace(path, trimmed);
    }
    setRenamingPath(null);
  };

  const handleInfoClick = (ws: KnownWorkspace, e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    const rect = e.currentTarget.getBoundingClientRect();
    setPopoverAnchor(rect);
    setPopoverWorkspace(ws);
  };

  const onlineDevicesCount = devices.filter((d) => d.online || d.state === "device" || d.state === "booted").length;

  return (
    <aside className="primary-sidebar">
      {/* 1. Header: Workspaces Title & Add Trigger */}
      <div className="primary-sidebar-header">
        <div className="primary-header-title">
          <FolderGit2 size={13} color="#06b6d4" />
          <span>WORKSPACES</span>
        </div>
        <button
          className="btn-add-workspace-primary"
          onClick={onAddWorkspace}
          title="Add / Open Workspace Directory"
        >
          <FolderPlus size={14} />
        </button>
      </div>

      {/* 2. Workspace List */}
      <div className="primary-workspace-list">
        {knownWorkspaces.length === 0 ? (
          <div className="empty-workspaces-hint">
            <span>No workspaces yet</span>
            <button className="btn-hint-add" onClick={onAddWorkspace}>
              + Add Workspace
            </button>
          </div>
        ) : (
          knownWorkspaces.map((ws) => {
            const isActive = ws.path === activeWorkspacePath && activeSection !== "devices" && activeSection !== "doctor";
            const displayName = ws.custom_name || ws.name;
            const isEditing = renamingPath === ws.path;
            const initials = displayName.slice(0, 2).toUpperCase();
            const platformClass = (ws.platform || "generic").toLowerCase();

            return (
              <div
                key={ws.path}
                className={`primary-ws-item ${isActive ? "active" : ""}`}
                onClick={() => {
                  onSelectWorkspace(ws.path);
                  if (activeSection === "devices" || activeSection === "doctor") {
                    onSelectSection("terminal");
                  }
                }}
                title={ws.path}
              >
                {/* Avatar Badge */}
                <div className={`ws-avatar-badge ${platformClass}`}>
                  {initials}
                </div>

                {/* Name / Editable input */}
                <div className="ws-item-info">
                  {isEditing ? (
                    <div className="ws-edit-row" onClick={(e) => e.stopPropagation()}>
                      <input
                        type="text"
                        className="ws-rename-input"
                        value={editNameValue}
                        autoFocus
                        onChange={(e) => setEditNameValue(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") saveRename(ws.path);
                          if (e.key === "Escape") setRenamingPath(null);
                        }}
                        onBlur={() => saveRename(ws.path)}
                      />
                      <button
                        className="btn-save-rename"
                        onClick={() => saveRename(ws.path)}
                      >
                        <Check size={12} color="#10b981" />
                      </button>
                    </div>
                  ) : (
                    <div
                      className="ws-item-name"
                      onDoubleClick={(e) => startRename(ws, e)}
                    >
                      {displayName}
                    </div>
                  )}
                  <div className="ws-item-meta">
                    <span className="ws-platform-label">{ws.platform || "Generic"}</span>
                    {isActive && <span className="ws-active-pulse" />}
                  </div>
                </div>

                {/* Actions: Edit & Info */}
                {!isEditing && (
                  <div className="ws-item-actions">
                    <button
                      className="btn-ws-subaction"
                      onClick={(e) => startRename(ws, e)}
                      title="Rename Workspace"
                    >
                      <Pencil size={11} />
                    </button>
                    <button
                      className="btn-ws-subaction"
                      onClick={(e) => handleInfoClick(ws, e)}
                      title="Workspace Details & Path"
                    >
                      <Info size={12} />
                    </button>
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* 3. Bottom Items: Devices, Doctor & Settings */}
      <div className="primary-sidebar-footer">
        <button
          className={`primary-footer-btn ${activeSection === "devices" ? "active" : ""}`}
          onClick={() => onSelectSection("devices")}
          title="Devices & Emulators Hub"
        >
          <div className="footer-btn-left">
            <Smartphone size={14} color="#38bdf8" />
            <span>Devices</span>
          </div>
          <span className="footer-count-badge">{onlineDevicesCount}</span>
        </button>

        <button
          className={`primary-footer-btn ${activeSection === "doctor" ? "active" : ""}`}
          onClick={() => onSelectSection("doctor")}
          title="Toolchain & System Diagnostics"
        >
          <div className="footer-btn-left">
            <Activity size={14} color="#10b981" />
            <span>Doctor</span>
          </div>
          <span className="doctor-status-dot" />
        </button>

        <button
          className="primary-footer-btn"
          onClick={onOpenSettings}
          title="Settings & Environment"
        >
          <div className="footer-btn-left">
            <Settings size={14} color="#94a3b8" />
            <span>Settings</span>
          </div>
        </button>
      </div>

      {/* Floating Info Popover */}
      {popoverWorkspace && (
        <WorkspaceInfoPopover
          workspace={popoverWorkspace}
          anchorRect={popoverAnchor}
          onClose={() => {
            setPopoverWorkspace(null);
            setPopoverAnchor(null);
          }}
          onRemove={onRemoveWorkspace}
        />
      )}
    </aside>
  );
};
