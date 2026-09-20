import React, { useState, useRef, useEffect } from "react";
import {
  Plus,
  X,
  Radio,
  Maximize2,
  Minimize2,
  Trash2,
  Columns,
  Rows,
  Grid,
  Square,
  ChevronDown,
} from "lucide-react";
import type { PaneInfo, ProjectTarget } from "../types";

export type LayoutMode = "tabs" | "split-h" | "split-v" | "grid";

interface TerminalSecondaryBarProps {
  panes: PaneInfo[];
  activePaneId: string;
  layoutMode: LayoutMode;
  availableTargets: ProjectTarget[];
  isMaximized: boolean;
  onSelectTab: (paneId: string) => void;
  onClosePane: (paneId: string) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
  onChangeLayout: (mode: LayoutMode) => void;
  onToggleMaximize: () => void;
  onClearActiveLogs: () => void;
}

export const TerminalSecondaryBar: React.FC<TerminalSecondaryBarProps> = ({
  panes,
  activePaneId,
  layoutMode,
  availableTargets,
  isMaximized,
  onSelectTab,
  onClosePane,
  onOpenTargetPane,
  onOpenCombinedPane,
  onChangeLayout,
  onToggleMaximize,
  onClearActiveLogs,
}) => {
  const [isAddMenuOpen, setIsAddMenuOpen] = useState(false);
  const addMenuRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (addMenuRef.current && !addMenuRef.current.contains(e.target as Node)) {
        setIsAddMenuOpen(false);
      }
    };
    if (isAddMenuOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isAddMenuOpen]);

  // Targets that are not already open as a pane
  const closedTargets = availableTargets.filter(
    (target) => !panes.some((p) => p.targetId === target.id)
  );
  const isCombinedOpen = panes.some((p) => p.isCombined);

  return (
    <div className="terminal-secondary-bar">
      {/* 1. Left: Tabs of Open Terminal Panes */}
      <div className="terminal-tabs-strip">
        {panes.map((pane) => {
          const isActive = pane.id === activePaneId;
          const platformClass = (pane.platform || "generic").toLowerCase();

          return (
            <div
              key={pane.id}
              className={`terminal-tab-item ${isActive ? "active" : ""}`}
              onClick={() => onSelectTab(pane.id)}
            >
              {pane.isCombined ? (
                <Radio size={12} color="#06b6d4" />
              ) : (
                <span className={`platform-tag ${platformClass}`}>{pane.platform}</span>
              )}

              <span className="terminal-tab-title">{pane.title}</span>

              <span
                className={`pulse-dot ${
                  pane.status === "running"
                    ? "green"
                    : pane.status === "building"
                    ? "yellow"
                    : pane.status === "error"
                    ? "red"
                    : "gray"
                }`}
              />

              {/* Close Tab button */}
              {panes.length > 1 && (
                <button
                  className="btn-tab-close"
                  onClick={(e) => {
                    e.stopPropagation();
                    onClosePane(pane.id);
                  }}
                  title="Close Pane Tab"
                >
                  <X size={11} />
                </button>
              )}
            </div>
          );
        })}

        {/* 2. Add Tab Trigger (+) */}
        <div className="tab-add-wrapper" ref={addMenuRef}>
          <button
            className="btn-tab-add"
            onClick={() => setIsAddMenuOpen(!isAddMenuOpen)}
            title="Open Additional Target Log Pane"
          >
            <Plus size={13} />
            <ChevronDown size={10} />
          </button>

          {isAddMenuOpen && (
            <div className="tab-add-dropdown">
              <div className="dropdown-label">OPEN TARGET LOGS</div>

              {!isCombinedOpen && (
                <button
                  className="dropdown-item"
                  onClick={() => {
                    onOpenCombinedPane();
                    setIsAddMenuOpen(false);
                  }}
                >
                  <Radio size={12} color="#06b6d4" />
                  <span>Combined Live Stream</span>
                </button>
              )}

              {closedTargets.length === 0 && isCombinedOpen ? (
                <div className="dropdown-empty">All targets are open</div>
              ) : (
                closedTargets.map((target) => (
                  <button
                    key={target.id}
                    className="dropdown-item"
                    onClick={() => {
                      onOpenTargetPane(target);
                      setIsAddMenuOpen(false);
                    }}
                  >
                    <span className={`platform-tag ${(target.platform || "generic").toLowerCase()}`}>
                      {target.platform}
                    </span>
                    <span>{target.name}</span>
                  </button>
                ))
              )}
            </div>
          )}
        </div>
      </div>

      {/* 3. Right: Split Layout Controls & Utilities */}
      <div className="terminal-secondary-tools">
        {/* Layout Switcher */}
        <div className="layout-mode-group">
          <button
            className={`btn-layout-tool ${layoutMode === "tabs" ? "active" : ""}`}
            onClick={() => onChangeLayout("tabs")}
            title="Single Full Tab (100% Focus)"
          >
            <Square size={12} />
          </button>

          <button
            className={`btn-layout-tool ${layoutMode === "split-h" ? "active" : ""}`}
            onClick={() => onChangeLayout("split-h")}
            title="Split Horizontal (Side-by-Side ◫)"
          >
            <Columns size={12} />
          </button>

          <button
            className={`btn-layout-tool ${layoutMode === "split-v" ? "active" : ""}`}
            onClick={() => onChangeLayout("split-v")}
            title="Split Vertical (Stacked ⬒)"
          >
            <Rows size={12} />
          </button>

          <button
            className={`btn-layout-tool ${layoutMode === "grid" ? "active" : ""}`}
            onClick={() => onChangeLayout("grid")}
            title="2x2 Grid Layout (⊞)"
          >
            <Grid size={12} />
          </button>
        </div>

        <div className="tool-divider" />

        {/* Clear Logs */}
        <button
          className="btn-terminal-util"
          onClick={onClearActiveLogs}
          title="Clear Terminal Logs for Active Pane"
        >
          <Trash2 size={12} />
          <span>Clear</span>
        </button>

        {/* Maximize Toggle */}
        <button
          className={`btn-terminal-util ${isMaximized ? "active" : ""}`}
          onClick={onToggleMaximize}
          title={isMaximized ? "Restore Layout" : "Maximize Pane (⛶)"}
        >
          {isMaximized ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
        </button>
      </div>
    </div>
  );
};
