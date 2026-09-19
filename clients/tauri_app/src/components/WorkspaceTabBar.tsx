import React from "react";
import {
  Columns2,
  Rows2,
  LayoutGrid,
  Square,
  Radio,
  X,
  Plus,
  FolderGit2,
} from "lucide-react";
import type { PaneInfo, ProjectTarget } from "../types";

export type LayoutMode = "tabs" | "split-h" | "split-v" | "grid";

interface WorkspaceTabBarProps {
  panes: PaneInfo[];
  activePaneId: string;
  layoutMode: LayoutMode;
  availableTargets: ProjectTarget[];
  onSelectTab: (paneId: string) => void;
  onCloseTab: (paneId: string) => void;
  onChangeLayoutMode: (mode: LayoutMode) => void;
  onOpenTarget: (target: ProjectTarget) => void;
  onOpenCombinedStream: () => void;
}

export const WorkspaceTabBar: React.FC<WorkspaceTabBarProps> = ({
  panes,
  activePaneId,
  layoutMode,
  availableTargets,
  onSelectTab,
  onCloseTab,
  onChangeLayoutMode,
  onOpenTarget,
  onOpenCombinedStream,
}) => {
  const unopenedTargets = availableTargets.filter(
    (t) => !panes.some((p) => p.targetId === t.id)
  );
  const hasCombined = panes.some((p) => p.isCombined);

  return (
    <div className="workspace-tab-bar">
      <div className="tab-bar-tabs">
        {panes.map((pane) => {
          const isActive = pane.id === activePaneId;
          const status = pane.status || "idle";

          return (
            <div
              key={pane.id}
              className={`tab-item ${isActive ? "active" : ""}`}
              onClick={() => onSelectTab(pane.id)}
              title={pane.title}
            >
              {pane.isCombined ? (
                <Radio size={13} color="#06b6d4" className="shrink-0" />
              ) : (
                <FolderGit2 size={13} color="#94a3b8" className="shrink-0" />
              )}

              <span
                className={`pulse-dot ${
                  status === "running"
                    ? "green"
                    : status === "building"
                    ? "yellow"
                    : status === "error"
                    ? "red"
                    : "gray"
                }`}
              />

              <span className="tab-title truncate">{pane.title}</span>

              {pane.framework && !pane.isCombined && (
                <span className="tab-framework-tag">{pane.framework}</span>
              )}

              {panes.length > 1 && (
                <button
                  className="tab-close-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onCloseTab(pane.id);
                  }}
                  title="Close Tab"
                >
                  <X size={12} />
                </button>
              )}
            </div>
          );
        })}

        {/* Quick Add Target Tab Button / Dropdown */}
        {unopenedTargets.length > 0 && (
          <div className="tab-dropdown-wrap">
            <button
              className="tab-add-btn"
              title="Open Target Tab"
              onClick={() => {
                if (unopenedTargets[0]) onOpenTarget(unopenedTargets[0]);
              }}
            >
              <Plus size={13} />
              <span>{unopenedTargets[0]?.name}</span>
            </button>
          </div>
        )}

        {!hasCombined && (
          <button
            className="tab-add-btn"
            title="Open Combined Stream Tab"
            onClick={onOpenCombinedStream}
          >
            <Radio size={12} />
            <span>Combined</span>
          </button>
        )}
      </div>

      {/* Right side Layout Mode Switcher */}
      <div className="tab-bar-actions">
        <div className="layout-switch-group">
          <button
            className={`layout-switch-btn ${layoutMode === "tabs" ? "active" : ""}`}
            onClick={() => onChangeLayoutMode("tabs")}
            title="Tabs Mode (Default: 1 focused pane at a time)"
          >
            <Square size={13} />
            <span className="layout-btn-label">Tabs</span>
          </button>

          <button
            className={`layout-switch-btn ${layoutMode === "split-h" ? "active" : ""}`}
            onClick={() => onChangeLayoutMode("split-h")}
            title="Split Side-by-Side (Horizontal Split)"
          >
            <Columns2 size={13} />
            <span className="layout-btn-label">Split</span>
          </button>

          <button
            className={`layout-switch-btn ${layoutMode === "split-v" ? "active" : ""}`}
            onClick={() => onChangeLayoutMode("split-v")}
            title="Split Stacked (Vertical Split)"
          >
            <Rows2 size={13} />
            <span className="layout-btn-label">Stack</span>
          </button>

          {panes.length >= 3 && (
            <button
              className={`layout-switch-btn ${layoutMode === "grid" ? "active" : ""}`}
              onClick={() => onChangeLayoutMode("grid")}
              title="Grid Layout (Auto-balanced Multi-pane)"
            >
              <LayoutGrid size={13} />
              <span className="layout-btn-label">Grid</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
