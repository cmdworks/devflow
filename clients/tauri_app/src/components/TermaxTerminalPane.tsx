import React, { useEffect, useRef, useState, useCallback } from "react";
import { Terminal } from "@termaxjs/web/canvas";
import { FitAddon } from "@termaxjs/web/addons";
import { TermaxPaneHeader } from "./TermaxPaneHeader";
import type { LogEntry, PaneInfo } from "../types";

interface TermaxTerminalPaneProps {
  pane: PaneInfo;
  logs: LogEntry[];
  isActive: boolean;
  isMaximized: boolean;
  onFocusPane?: (paneId: string) => void;
  onToggleRun: (targetId: string) => void;
  onReload: (targetId: string) => void;
  onRestart: (targetId: string) => void;
  onSplitRight: (paneId: string) => void;
  onSplitDown: (paneId: string) => void;
  onToggleMaximize: (paneId: string) => void;
  onClearLogs: (paneId: string) => void;
  onClose: (paneId: string) => void;
}

export const TermaxTerminalPane: React.FC<TermaxTerminalPaneProps> = ({
  pane,
  logs,
  isActive,
  isMaximized,
  onFocusPane,
  onToggleRun,
  onReload,
  onRestart,
  onSplitRight,
  onSplitDown,
  onToggleMaximize,
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

  useEffect(() => {
    const timer = setTimeout(() => {
      try {
        fitAddonRef.current?.fit();
      } catch (_) {}
    }, 60);
    return () => clearTimeout(timer);
  }, [isMaximized]);

  return (
    <div
      className={`terminal-pane ${isActive ? "active-pane" : ""} ${isMaximized ? "maximized" : ""}`}
      onClick={() => onFocusPane?.(pane.id)}
    >
      <TermaxPaneHeader
        pane={pane}
        isActive={isActive}
        isMaximized={isMaximized}
        levelFilter={levelFilter}
        searchQuery={searchQuery}
        onSelectLevel={setLevelFilter}
        onChangeSearch={setSearchQuery}
        onToggleRun={onToggleRun}
        onReload={onReload}
        onRestart={onRestart}
        onSplitRight={onSplitRight}
        onSplitDown={onSplitDown}
        onToggleMaximize={onToggleMaximize}
        onClearLogs={onClearLogs}
        onClose={onClose}
      />

      <div ref={containerRef} className="pane-terminal-container" />
    </div>
  );
};
