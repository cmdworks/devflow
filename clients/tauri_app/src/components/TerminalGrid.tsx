import React from "react";
import { FolderPlus, Radio } from "lucide-react";
import { TermaxTerminalPane } from "./TermaxTerminalPane";
import type { PaneInfo, LogEntry } from "../types";

interface TerminalGridProps {
  panes: PaneInfo[];
  logsByPaneId: Record<string, LogEntry[]>;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onClearLogs: (paneId: string) => void;
  onClosePane: (paneId: string) => void;
  onOpenAllPanes: () => void;
  onOpenCombinedPane: () => void;
}

export const TerminalGrid: React.FC<TerminalGridProps> = ({
  panes,
  logsByPaneId,
  onToggleRun,
  onReload,
  onRestart,
  onClearLogs,
  onClosePane,
  onOpenAllPanes,
  onOpenCombinedPane,
}) => {
  if (panes.length === 0) {
    return (
      <div className="empty-viewport">
        <div className="empty-viewport-title">No Active Terminal Panes</div>
        <div style={{ fontSize: "13px" }}>
          Select a project target from the sidebar or open all workspace targets to start streaming logs.
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

  const gridClass =
    panes.length === 1
      ? "panes-1"
      : panes.length === 2
      ? "panes-2"
      : panes.length === 3
      ? "panes-3"
      : panes.length === 4
      ? "panes-4"
      : "panes-many";

  return (
    <div className={`panes-grid ${gridClass}`}>
      {panes.map((pane) => (
        <TermaxTerminalPane
          key={pane.id}
          pane={pane}
          logs={logsByPaneId[pane.id] || []}
          onToggleRun={onToggleRun}
          onReload={onReload}
          onRestart={onRestart}
          onClearLogs={onClearLogs}
          onClose={onClosePane}
        />
      ))}
    </div>
  );
};
