import React, { useEffect, useRef, useState, useCallback } from "react";
import { Play, Square, Zap, RotateCw, Trash2, X } from "lucide-react";
import { Terminal } from "@termaxjs/web/canvas";
import { FitAddon } from "@termaxjs/web/addons";
import type { LogEntry, PaneInfo } from "../types";

interface TermaxTerminalPaneProps {
  pane: PaneInfo;
  logs: LogEntry[];
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onClearLogs: (paneId: string) => void;
  onClose: (paneId: string) => void;
}

export const TermaxTerminalPane: React.FC<TermaxTerminalPaneProps> = ({
  pane,
  logs,
  onToggleRun,
  onReload,
  onRestart,
  onClearLogs,
  onClose,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  const [levelFilter, setLevelFilter] = useState<string>("ALL");
  const [searchQuery, setSearchQuery] = useState<string>("");

  // Initialize Termax Terminal instance
  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      fontSize: 12,
      fontFamily: "'JetBrains Mono', monospace",
      cursorBlink: false,
      cursorStyle: "bar",
      theme: {
        background: "#090d16",
        foreground: "#cbd5e1",
        cursor: "#06b6d4",
        selectionBackground: "rgba(56, 189, 248, 0.3)",
        black: "#1e293b",
        red: "#ef4444",
        green: "#10b981",
        yellow: "#f59e0b",
        blue: "#3b82f6",
        magenta: "#d946ef",
        cyan: "#06b6d4",
        white: "#f8fafc",
        brightBlack: "#475569",
        brightRed: "#f87171",
        brightGreen: "#34d399",
        brightYellow: "#fbbf24",
        brightBlue: "#60a5fa",
        brightMagenta: "#e879f9",
        brightCyan: "#38bdf8",
        brightWhite: "#ffffff",
      },
    });

    const fit = new FitAddon();
    term.loadAddon(fit);

    term.open(containerRef.current);
    fit.fit();

    terminalRef.current = term;
    fitAddonRef.current = fit;

    const handleResize = () => {
      try {
        fit.fit();
      } catch (_) {}
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, []);

  // Format a LogEntry into ANSI escape strings
  const formatLogToAnsi = useCallback((entry: LogEntry): string => {
    const timeStr = entry.timestamp ? entry.timestamp.split("T")[1]?.slice(0, 12) || "" : "";
    let lvlAnsi = "";

    switch (entry.level) {
      case "E":
        lvlAnsi = "\x1b[41;1;37m ERR \x1b[0m";
        break;
      case "W":
        lvlAnsi = "\x1b[43;1;30m WRN \x1b[0m";
        break;
      case "I":
        lvlAnsi = "\x1b[44;1;37m INF \x1b[0m";
        break;
      case "D":
        lvlAnsi = "\x1b[100;1;37m DBG \x1b[0m";
        break;
      default:
        lvlAnsi = "\x1b[36m LOG \x1b[0m";
    }

    const tagStr = entry.tag ? ` \x1b[36m[${entry.tag}]\x1b[0m` : "";
    return `\x1b[90m${timeStr}\x1b[0m ${lvlAnsi}${tagStr} ${entry.message}`;
  }, []);

  // Re-render logs when logs, filter, or search change
  useEffect(() => {
    const term = terminalRef.current;
    if (!term) return;

    term.clear();

    const filtered = logs.filter((log) => {
      if (levelFilter !== "ALL" && log.level !== levelFilter) {
        return false;
      }
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        const msg = (log.message || "").toLowerCase();
        const tag = (log.tag || "").toLowerCase();
        if (!msg.includes(q) && !tag.includes(q)) {
          return false;
        }
      }
      return true;
    });

    for (const log of filtered) {
      term.writeln(formatLogToAnsi(log));
    }

    term.scrollToBottom();
  }, [logs, levelFilter, searchQuery, formatLogToAnsi]);

  const isRunning = pane.status === "running" || pane.status === "building";

  return (
    <div className={`terminal-pane ${isRunning ? "focus" : ""}`}>
      <div className="pane-header">
        <div className="pane-header-left">
          <span className="pane-title">{pane.title}</span>
          <span className={`pane-status-pill ${pane.status}`}>{pane.status}</span>
        </div>

        <div className="pane-header-right">
          {/* Level Filter Chips */}
          <div className="pane-filter-group">
            {["ALL", "E", "W", "I", "D"].map((lvl) => (
              <button
                key={lvl}
                className={`filter-btn ${levelFilter === lvl ? "active" : ""}`}
                onClick={() => setLevelFilter(lvl)}
              >
                {lvl === "E" ? "ERR" : lvl === "W" ? "WRN" : lvl === "I" ? "INF" : lvl === "D" ? "DBG" : "ALL"}
              </button>
            ))}
          </div>

          {/* Search Input */}
          <div style={{ position: "relative", display: "flex", alignItems: "center" }}>
            <input
              type="text"
              className="pane-search-input"
              placeholder="Search..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>

          {/* Target Action Controls (for non-combined panes) */}
          {!pane.isCombined && (
            <>
              <button
                className={`btn-pane-action ${isRunning ? "stop-btn" : "run-btn"}`}
                onClick={() => onToggleRun(pane.targetId)}
                title={isRunning ? "Stop Target" : "Run Target"}
              >
                {isRunning ? <Square size={11} fill="currentColor" /> : <Play size={11} fill="currentColor" />}
                <span>{isRunning ? "Stop" : "Run"}</span>
              </button>

              <button
                className="btn-pane-action"
                onClick={() => onReload(pane.targetId)}
                title="Hot Reload Target"
              >
                <Zap size={11} color="#f59e0b" />
              </button>

              <button
                className="btn-pane-action"
                onClick={() => onRestart(pane.targetId)}
                title="Restart Target App"
              >
                <RotateCw size={11} color="#06b6d4" />
              </button>
            </>
          )}

          {/* Clear Console */}
          <button
            className="btn-pane-action"
            onClick={() => onClearLogs(pane.id)}
            title="Clear Console"
          >
            <Trash2 size={11} />
          </button>

          {/* Close Pane */}
          <button
            className="btn-pane-close"
            onClick={() => onClose(pane.id)}
            title="Close Pane"
          >
            <X size={13} />
          </button>
        </div>
      </div>

      <div ref={containerRef} className="pane-terminal-container" />
    </div>
  );
};
