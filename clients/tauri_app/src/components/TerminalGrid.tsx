import React from "react";
import { FolderPlus, Radio } from "lucide-react";
import { TermaxTerminalPane } from "./TermaxTerminalPane";
import type { LayoutMode } from "./TerminalSecondaryBar";
import type { PaneInfo, LogEntry } from "../types";

interface TerminalGridProps {
  panes: PaneInfo[];
  activePaneId: string;
  maximizedPaneId: string | null;
  layoutMode: LayoutMode;
  logsByPaneId: Record<string, LogEntry[]>;
  levelFilter?: string;
  tagFilter?: string;
  searchQuery?: string;
  executingActions?: Record<string, string>;
  onFocusPane: (paneId: string) => void;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onExecuteAction?: (targetId: string, action: string, port?: number) => void;
  onOpenDevOptions?: (targetId?: string) => void;
  onSplitRight: (paneId: string) => void;
  onSplitDown: (paneId: string) => void;
  onToggleMaximize: (paneId: string) => void;
  onClearLogs: (paneId: string) => void;
  onClosePane: (paneId: string) => void;
  onOpenAllPanes: () => void;
  onOpenCombinedPane: () => void;
}

export const TerminalGrid: React.FC<TerminalGridProps> = ({
  panes,
  activePaneId,
  maximizedPaneId,
  layoutMode,
  logsByPaneId,
  levelFilter = "ALL",
  tagFilter = "",
  searchQuery = "",
  executingActions = {},
  onFocusPane,
  onToggleRun,
  onReload,
  onRestart,
  onExecuteAction,
  onOpenDevOptions,
  onClearLogs,
  onOpenAllPanes,
  onOpenCombinedPane,
}) => {
  if (panes.length === 0) {
    return (
      <div className="empty-viewport">
        <div className="empty-viewport-title">No Active Terminal Panes</div>
        <div style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
          Select a project target from the sidebar or open a workspace tab to start streaming logs.
        </div>
        <div className="empty-viewport-actions">
          <button className="btn-top-action primary" onClick={onOpenAllPanes}>
            <FolderPlus size={14} />
            <span>Open All Targets</span>
          </button>
          <button className="btn-top-action" onClick={onOpenCombinedPane}>
            <Radio size={14} />
            <span>Combined Stream</span>
          </button>
        </div>
      </div>
    );
  }

  // 1. Maximized Solo Mode: Expand single pane to 100% viewport
  if (maximizedPaneId) {
    const maxPane = panes.find((p) => p.id === maximizedPaneId);
    if (maxPane) {
      return (
        <div className="viewport-layout layout-maximized">
          <TermaxTerminalPane
            key={maxPane.id}
            pane={maxPane}
            logs={logsByPaneId[maxPane.id] || []}
            isActive={true}
            isMaximized={true}
            levelFilter={levelFilter}
            tagFilter={tagFilter}
            searchQuery={searchQuery}
            executingAction={executingActions[maxPane.targetId]}
            onFocusPane={onFocusPane}
            onToggleRun={onToggleRun}
            onReload={onReload}
            onRestart={onRestart}
            onExecuteAction={onExecuteAction}
            onOpenDevOptions={onOpenDevOptions}
            onClearLogs={onClearLogs}
          />
        </div>
      );
    }
  }

  // 2. Tabs Mode (Default): Only render the active tab pane at 100% viewport
  if (layoutMode === "tabs") {
    const currentPane = panes.find((p) => p.id === activePaneId) || panes[0];
    if (currentPane) {
      return (
        <div className="viewport-layout layout-tabs">
          <TermaxTerminalPane
            key={currentPane.id}
            pane={currentPane}
            logs={logsByPaneId[currentPane.id] || []}
            isActive={true}
            isMaximized={false}
            levelFilter={levelFilter}
            tagFilter={tagFilter}
            searchQuery={searchQuery}
            executingAction={executingActions[currentPane.targetId]}
            onFocusPane={onFocusPane}
            onToggleRun={onToggleRun}
            onReload={onReload}
            onRestart={onRestart}
            onExecuteAction={onExecuteAction}
            onOpenDevOptions={onOpenDevOptions}
            onClearLogs={onClearLogs}
          />
        </div>
      );
    }
  }

  // 3. Multi-Pane Layout Modes: Split-H, Split-V, or Grid
  const layoutClass =
    layoutMode === "split-h"
      ? "layout-split-h"
      : layoutMode === "split-v"
      ? "layout-split-v"
      : "layout-grid";

  return (
    <div className={`viewport-layout ${layoutClass}`}>
      {panes.map((pane) => (
        <TermaxTerminalPane
          key={pane.id}
          pane={pane}
          logs={logsByPaneId[pane.id] || []}
          isActive={pane.id === activePaneId}
          isMaximized={false}
          levelFilter={levelFilter}
          tagFilter={tagFilter}
          searchQuery={searchQuery}
          executingAction={executingActions[pane.targetId]}
          onFocusPane={onFocusPane}
          onToggleRun={onToggleRun}
          onReload={onReload}
          onRestart={onRestart}
          onExecuteAction={onExecuteAction}
          onOpenDevOptions={onOpenDevOptions}
          onClearLogs={onClearLogs}
        />
      ))}
    </div>
  );
};
