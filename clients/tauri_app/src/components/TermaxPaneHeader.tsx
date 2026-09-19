import React from "react";
import {
  Play,
  Square,
  Zap,
  RotateCw,
  Columns2,
  Rows2,
  Maximize2,
  Minimize2,
  Trash2,
  X,
  Search,
  Radio,
  Terminal as TerminalIcon,
} from "lucide-react";
import type { PaneInfo } from "../types";

interface TermaxPaneHeaderProps {
  pane: PaneInfo;
  isActive: boolean;
  isMaximized: boolean;
  levelFilter: string;
  searchQuery: string;
  onSelectLevel: (lvl: string) => void;
  onChangeSearch: (query: string) => void;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onSplitRight: (paneId: string) => void;
  onSplitDown: (paneId: string) => void;
  onToggleMaximize: (paneId: string) => void;
  onClearLogs: (paneId: string) => void;
  onClose: (paneId: string) => void;
}

export const TermaxPaneHeader: React.FC<TermaxPaneHeaderProps> = ({
  pane,
  isActive,
  isMaximized,
  levelFilter,
  searchQuery,
  onSelectLevel,
  onChangeSearch,
  onToggleRun,
  onReload,
  onRestart,
  onSplitRight,
  onSplitDown,
  onToggleMaximize,
  onClearLogs,
  onClose,
}) => {
  const isRunning = pane.status === "running" || pane.status === "building";

  return (
    <div className={`termax-pane-header ${isActive ? "active" : ""}`}>
      {/* Left: Status & Identity */}
      <div className="pane-header-left">
        {pane.isCombined ? (
          <Radio size={13} color="#06b6d4" />
        ) : (
          <TerminalIcon size={13} color={isActive ? "#06b6d4" : "var(--text-muted)"} />
        )}

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

        <span className="pane-header-title truncate">{pane.title}</span>

        {pane.platform && (
          <span className="pane-platform-badge">{pane.platform}</span>
        )}
      </div>

      {/* Center: Search & Level Filters */}
      <div className="pane-header-center">
        <div className="pane-search-box">
          <Search size={11} className="pane-search-icon" />
          <input
            type="text"
            className="pane-search-input"
            placeholder="Filter logs..."
            value={searchQuery}
            onChange={(e) => onChangeSearch(e.target.value)}
          />
        </div>

        <div className="pane-filter-chips">
          {["ALL", "ERR", "WRN", "INF"].map((lvl) => (
            <button
              key={lvl}
              className={`filter-chip ${levelFilter === lvl ? "active" : ""}`}
              onClick={() => onSelectLevel(lvl)}
            >
              {lvl}
            </button>
          ))}
        </div>
      </div>

      {/* Right: Actions, Split, Maximize & Close */}
      <div className="pane-header-right">
        {!pane.isCombined && (
          <div className="pane-lifecycle-actions">
            <button
              className={`pane-ctrl-btn ${isRunning ? "running" : "run"}`}
              onClick={() => onToggleRun(pane.targetId)}
              title={isRunning ? "Stop Target" : "Run Target"}
            >
              {isRunning ? <Square size={12} fill="#ef4444" /> : <Play size={12} fill="#10b981" />}
            </button>

            <button
              className="pane-ctrl-btn"
              onClick={() => onReload(pane.targetId)}
              title="Hot Reload"
              disabled={!isRunning}
            >
              <Zap size={12} />
            </button>

            <button
              className="pane-ctrl-btn"
              onClick={() => onRestart(pane.targetId)}
              title="Full App Restart"
              disabled={!isRunning}
            >
              <RotateCw size={12} />
            </button>
          </div>
        )}

        <div className="pane-split-actions">
          <button
            className="pane-ctrl-btn"
            onClick={() => onSplitRight(pane.id)}
            title="Split Side-by-Side (Split Right)"
          >
            <Columns2 size={12} />
          </button>

          <button
            className="pane-ctrl-btn"
            onClick={() => onSplitDown(pane.id)}
            title="Split Stacked (Split Down)"
          >
            <Rows2 size={12} />
          </button>

          <button
            className="pane-ctrl-btn"
            onClick={() => onToggleMaximize(pane.id)}
            title={isMaximized ? "Restore Split View" : "Maximize Pane"}
          >
            {isMaximized ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
          </button>

          <button
            className="pane-ctrl-btn"
            onClick={() => onClearLogs(pane.id)}
            title="Clear Terminal Output"
          >
            <Trash2 size={12} />
          </button>

          <button
            className="pane-ctrl-btn close"
            onClick={() => onClose(pane.id)}
            title="Close Pane"
          >
            <X size={13} />
          </button>
        </div>
      </div>
    </div>
  );
};
