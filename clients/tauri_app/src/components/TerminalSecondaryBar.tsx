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
  Play,
  Zap,
  RotateCw,
  ArrowUpRight,
  ListTree,
} from "lucide-react";
import type { PaneInfo, ProjectTarget } from "../types";
import { LayoutPickerModal } from "./LayoutPickerModal";

export type LayoutMode = "tabs" | "split-h" | "split-v" | "grid";

interface TerminalSecondaryBarProps {
  panes: PaneInfo[];
  activePaneId: string;
  layoutMode: LayoutMode;
  availableTargets: ProjectTarget[];
  isMaximized: boolean;
  isLogTreeOpen?: boolean;
  levelFilter?: string;
  searchQuery?: string;
  onSelectTab: (paneId: string) => void;
  onClosePane: (paneId: string) => void;
  onOpenTargetPane: (target: ProjectTarget) => void;
  onOpenCombinedPane: () => void;
  onChangeLayout: (mode: LayoutMode) => void;
  onToggleMaximize: () => void;
  onToggleLogTree?: () => void;
  onClearActiveLogs: () => void;
  onSelectLevel?: (lvl: string) => void;
  onChangeSearch?: (query: string) => void;
  onToggleRunTarget?: (targetId: string) => void;
  onReloadTarget?: (targetId: string) => void;
  onRestartTarget?: (targetId: string) => void;
}

export const TerminalSecondaryBar: React.FC<TerminalSecondaryBarProps> = ({
  panes,
  activePaneId,
  layoutMode,
  availableTargets,
  isMaximized,
  isLogTreeOpen = false,
  onSelectTab,
  onClosePane,
  onOpenTargetPane,
  onOpenCombinedPane,
  onChangeLayout,
  onToggleMaximize,
  onToggleLogTree,
  onClearActiveLogs,
  onToggleRunTarget,
  onReloadTarget,
  onRestartTarget,
}) => {
  const [isAddMenuOpen, setIsAddMenuOpen] = useState(false);
  const [isLayoutModalOpen, setIsLayoutModalOpen] = useState(false);
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

  const isCombinedOpen = panes.some((p) => p.isCombined);
  const activePane = panes.find((p) => p.id === activePaneId);
  const isRunning = activePane && (activePane.status === "running" || activePane.status === "building");

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
            className={`btn-tab-add ${isAddMenuOpen ? "active" : ""}`}
            onClick={() => setIsAddMenuOpen(!isAddMenuOpen)}
            title="Open Target Log Panes & Layouts (+)"
          >
            <Plus size={13} />
            <ChevronDown size={10} />
          </button>

          {isAddMenuOpen && (
            <div className="tab-add-dropdown">
              <div className="dropdown-label">TARGET LOG PANES</div>

              {availableTargets.length === 0 ? (
                <div className="dropdown-empty">No targets discovered</div>
              ) : (
                availableTargets.map((target) => {
                  const matchingPane = panes.find((p) => p.targetId === target.id);
                  const isOpen = !!matchingPane;
                  const status = matchingPane?.status || "idle";
                  const isTargetRunning = status === "running";
                  const isTargetBuilding = status === "building";
                  const isTargetError = status === "error";

                  return (
                    <button
                      key={target.id}
                      className={`dropdown-item ${isOpen ? "is-open" : "is-closed"}`}
                      onClick={() => {
                        if (isOpen && matchingPane) {
                          onSelectTab(matchingPane.id);
                        } else {
                          onOpenTargetPane(target);
                        }
                        setIsAddMenuOpen(false);
                      }}
                      title={isOpen ? `Focus open tab (${target.name})` : `Open log pane for ${target.name}`}
                    >
                      <span
                        className={`pulse-dot ${
                          isTargetRunning
                            ? "green"
                            : isTargetBuilding
                            ? "yellow"
                            : isTargetError
                            ? "red"
                            : "gray"
                        }`}
                      />
                      <span className={`platform-tag ${(target.platform || "generic").toLowerCase()}`}>
                        {target.platform}
                      </span>
                      <span className="dropdown-item-title">{target.name}</span>
                      {isOpen ? (
                        <span className="dropdown-item-badge open" title="Pane open — Click to focus">
                          <ArrowUpRight size={12} />
                        </span>
                      ) : (
                        <span className="dropdown-item-badge closed" title="Closed — Click to open">
                          <Plus size={11} />
                        </span>
                      )}
                    </button>
                  );
                })
              )}

              <div className="dropdown-divider" />
              <div className="dropdown-label">AGGREGATED STREAM</div>

              <button
                className={`dropdown-item ${isCombinedOpen ? "is-open" : "is-closed"}`}
                onClick={() => {
                  const combinedPane = panes.find((p) => p.isCombined);
                  if (combinedPane) {
                    onSelectTab(combinedPane.id);
                  } else {
                    onOpenCombinedPane();
                  }
                  setIsAddMenuOpen(false);
                }}
              >
                <Radio size={12} color="#06b6d4" />
                <span className="dropdown-item-title">Combined Live Stream</span>
                {isCombinedOpen ? (
                  <span className="dropdown-item-badge open" title="Pane open — Click to focus">
                    <ArrowUpRight size={12} />
                  </span>
                ) : (
                  <span className="dropdown-item-badge closed" title="Closed — Click to open">
                    <Plus size={11} />
                  </span>
                )}
              </button>

              <div className="dropdown-divider" />
              <div className="dropdown-label">SPLIT SHORTCUTS</div>

              <button
                className="dropdown-item"
                onClick={() => {
                  onChangeLayout("split-h");
                  setIsAddMenuOpen(false);
                }}
              >
                <Columns size={12} color="#38bdf8" />
                <span>Split Right (Side by Side)</span>
              </button>

              <button
                className="dropdown-item"
                onClick={() => {
                  onChangeLayout("split-v");
                  setIsAddMenuOpen(false);
                }}
              >
                <Rows size={12} color="#38bdf8" />
                <span>Split Down (Stacked)</span>
              </button>

              <button
                className="dropdown-item"
                onClick={() => {
                  onChangeLayout("grid");
                  setIsAddMenuOpen(false);
                }}
              >
                <Grid size={12} color="#38bdf8" />
                <span>2×2 Quad Grid</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* 2. Right: Active Target Controls, Layout Customizer Modal, Clear & Maximize */}
      <div className="terminal-secondary-tools">
        {/* Active Target Actions (if single target pane) */}
        {activePane && !activePane.isCombined && onToggleRunTarget && (
          <div className="pane-quick-actions">
            <button
              className={`btn-pane-action ${isRunning ? "running" : "run"}`}
              onClick={() => onToggleRunTarget(activePane.targetId)}
              title={isRunning ? "Stop Target" : "Run Target"}
            >
              {isRunning ? <Square size={11} fill="currentColor" /> : <Play size={11} fill="currentColor" />}
            </button>

            {onReloadTarget && (
              <button
                className="btn-pane-action"
                onClick={() => onReloadTarget(activePane.targetId)}
                title="Hot Reload"
              >
                <Zap size={11} color="#f59e0b" />
              </button>
            )}

            {onRestartTarget && (
              <button
                className="btn-pane-action"
                onClick={() => onRestartTarget(activePane.targetId)}
                title="Restart Process"
              >
                <RotateCw size={11} color="#06b6d4" />
              </button>
            )}
          </div>
        )}

        <div className="tool-divider" />

        {/* Layout Switcher Floating Modal Trigger */}
        <div className="layout-picker-wrapper">
          <button
            className={`btn-layout-tool-picker ${isLayoutModalOpen ? "active" : ""}`}
            onClick={() => setIsLayoutModalOpen(!isLayoutModalOpen)}
            title="Customize Terminal Split Layout"
          >
            {layoutMode === "tabs" && <Square size={12} />}
            {layoutMode === "split-h" && <Columns size={12} />}
            {layoutMode === "split-v" && <Rows size={12} />}
            {layoutMode === "grid" && <Grid size={12} />}
            <span className="layout-btn-label">Layout</span>
            <ChevronDown size={10} />
          </button>

          <LayoutPickerModal
            isOpen={isLayoutModalOpen}
            onClose={() => setIsLayoutModalOpen(false)}
            layoutMode={layoutMode}
            onChangeLayout={onChangeLayout}
          />
        </div>

        <div className="tool-divider" />

        {/* Log Tree Inspector Toggle */}
        {onToggleLogTree && (
          <button
            className={`btn-terminal-util ${isLogTreeOpen ? "active" : ""}`}
            onClick={onToggleLogTree}
            title={isLogTreeOpen ? "Hide Log Inspector Tree" : "Show Log Inspector Tree"}
          >
            <ListTree size={12} color={isLogTreeOpen ? "#06b6d4" : "currentColor"} />
            <span>Inspector</span>
          </button>
        )}

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

