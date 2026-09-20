import React, { useState, useMemo } from "react";
import {
  ChevronDown,
  ChevronRight,
  AlertCircle,
  AlertTriangle,
  Info,
  Layers,
  Terminal,
  Cpu,
  Smartphone,
  Flame,
  ArrowUp,
  ArrowDown,
  Activity,
  ListTree,
} from "lucide-react";
import type { LogEntry } from "../types";

export interface TerminalLogTreeProps {
  logs: LogEntry[];
  activeLevelFilter: string;
  activeTagFilter: string;
  onSelectLevelFilter: (level: string) => void;
  onSelectTagFilter: (tag: string) => void;
  onScrollToTop?: () => void;
  onScrollToBottom?: () => void;
  onJumpToLastError?: () => void;
}

export const TerminalLogTree: React.FC<TerminalLogTreeProps> = ({
  logs,
  activeLevelFilter,
  activeTagFilter,
  onSelectLevelFilter,
  onSelectTagFilter,
  onScrollToTop,
  onScrollToBottom,
  onJumpToLastError,
}) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(
    new Set(["levels", "tags", "categories"])
  );

  const toggleNode = (nodeId: string) => {
    setExpandedNodes((prev) => {
      const next = new Set(prev);
      if (next.has(nodeId)) {
        next.delete(nodeId);
      } else {
        next.add(nodeId);
      }
      return next;
    });
  };

  // Compute log statistics
  const stats = useMemo(() => {
    let errorCount = 0;
    let warningCount = 0;
    let infoCount = 0;
    const tagMap: Record<string, number> = {};

    for (const log of logs) {
      if (log.level === "E") errorCount++;
      else if (log.level === "W") warningCount++;
      else if (log.level === "I") infoCount++;

      const tag = log.tag || "general";
      tagMap[tag] = (tagMap[tag] || 0) + 1;
    }

    const sortedTags = Object.entries(tagMap)
      .sort((a, b) => b[1] - a[1])
      .map(([tag, count]) => ({ tag, count }));

    return {
      total: logs.length,
      errors: errorCount,
      warnings: warningCount,
      info: infoCount,
      tags: sortedTags,
    };
  }, [logs]);

  // Tag Icon helper
  const getTagIcon = (tag: string) => {
    switch (tag.toLowerCase()) {
      case "build":
      case "compiler":
        return <Cpu size={12} color="#38bdf8" />;
      case "stdout":
      case "stderr":
        return <Terminal size={12} color="#10b981" />;
      case "logcat":
      case "simctl":
      case "device":
        return <Smartphone size={12} color="#a855f7" />;
      case "lifecycle":
        return <Flame size={12} color="#f59e0b" />;
      default:
        return <Activity size={12} color="#94a3b8" />;
    }
  };

  return (
    <div className="terminal-log-tree-panel">
      {/* Header with quick scroll controls */}
      <div className="log-tree-header">
        <div className="log-tree-title">
          <ListTree size={13} color="#06b6d4" />
          <span>Log Inspector</span>
        </div>

        <div className="log-tree-actions">
          {onScrollToTop && (
            <button
              className="btn-log-action-mini"
              onClick={onScrollToTop}
              title="Scroll to Top of Terminal"
            >
              <ArrowUp size={12} />
            </button>
          )}
          {onScrollToBottom && (
            <button
              className="btn-log-action-mini"
              onClick={onScrollToBottom}
              title="Scroll to Bottom (Latest Logs)"
            >
              <ArrowDown size={12} />
            </button>
          )}
          {onJumpToLastError && stats.errors > 0 && (
            <button
              className="btn-log-action-mini danger"
              onClick={onJumpToLastError}
              title={`Jump to Error (${stats.errors} errors)`}
            >
              <AlertCircle size={12} />
            </button>
          )}
        </div>
      </div>

      {/* Tree Content */}
      <div className="log-tree-body">
        {/* All Logs Root Node */}
        <div
          className={`log-tree-node ${(!activeLevelFilter || activeLevelFilter === "ALL" || activeLevelFilter === "*") && !activeTagFilter ? "active" : ""}`}
          onClick={() => {
            onSelectLevelFilter("ALL");
            onSelectTagFilter("");
          }}
        >
          <div className="node-content">
            <Layers size={12} color="#38bdf8" />
            <span className="node-label">All Streams</span>
          </div>
          <span className="node-badge neutral">{stats.total}</span>
        </div>

        {/* Section 1: Levels */}
        <div className="log-tree-group">
          <div className="group-header" onClick={() => toggleNode("levels")}>
            <span className="group-chevron">
              {expandedNodes.has("levels") ? <ChevronDown size={11} /> : <ChevronRight size={11} />}
            </span>
            <span className="group-title">Severity Levels</span>
          </div>

          {expandedNodes.has("levels") && (
            <div className="group-items">
              <div
                className={`log-tree-node level-err ${activeLevelFilter === "E" || activeLevelFilter === "ERR" ? "active" : ""}`}
                onClick={() => {
                  onSelectLevelFilter("E");
                  onSelectTagFilter("");
                }}
              >
                <div className="node-content">
                  <AlertCircle size={12} color="#ef4444" />
                  <span className="node-label">Errors (ERR)</span>
                </div>
                {stats.errors > 0 && (
                  <span className="node-badge danger">{stats.errors}</span>
                )}
              </div>

              <div
                className={`log-tree-node level-wrn ${activeLevelFilter === "W" || activeLevelFilter === "WRN" ? "active" : ""}`}
                onClick={() => {
                  onSelectLevelFilter("W");
                  onSelectTagFilter("");
                }}
              >
                <div className="node-content">
                  <AlertTriangle size={12} color="#f59e0b" />
                  <span className="node-label">Warnings (WRN)</span>
                </div>
                {stats.warnings > 0 && (
                  <span className="node-badge warning">{stats.warnings}</span>
                )}
              </div>

              <div
                className={`log-tree-node level-inf ${activeLevelFilter === "I" || activeLevelFilter === "INF" ? "active" : ""}`}
                onClick={() => {
                  onSelectLevelFilter("I");
                  onSelectTagFilter("");
                }}
              >
                <div className="node-content">
                  <Info size={12} color="#3b82f6" />
                  <span className="node-label">Info (INF)</span>
                </div>
                {stats.info > 0 && (
                  <span className="node-badge info">{stats.info}</span>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Section 2: Subsystems & Tags */}
        <div className="log-tree-group">
          <div className="group-header" onClick={() => toggleNode("tags")}>
            <span className="group-chevron">
              {expandedNodes.has("tags") ? <ChevronDown size={11} /> : <ChevronRight size={11} />}
            </span>
            <span className="group-title">Subsystems & Tags</span>
            <span className="group-count">{stats.tags.length}</span>
          </div>

          {expandedNodes.has("tags") && (
            <div className="group-items">
              {stats.tags.length === 0 ? (
                <div className="log-tree-empty">No log tags recorded</div>
              ) : (
                stats.tags.map(({ tag, count }) => (
                  <div
                    key={tag}
                    className={`log-tree-node tag-node ${activeTagFilter === tag ? "active" : ""}`}
                    onClick={() => onSelectTagFilter(activeTagFilter === tag ? "" : tag)}
                  >
                    <div className="node-content">
                      {getTagIcon(tag)}
                      <span className="node-label">[{tag}]</span>
                    </div>
                    <span className="node-badge tag">{count}</span>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
