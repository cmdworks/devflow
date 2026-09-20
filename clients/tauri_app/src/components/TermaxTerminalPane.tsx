import React, { useEffect, useRef, useCallback, useState } from "react";
import { Terminal } from "@termaxjs/web/canvas";
import { FitAddon, WebLinksAddon } from "@termaxjs/web/addons";
import { ArrowDown, ArrowUp, AlertCircle, ChevronDown } from "lucide-react";
import type { LogEntry, PaneInfo } from "../types";

export function normalizeLogLevel(lvl?: string): string {
  if (!lvl || lvl === "ALL" || lvl === "*") return "ALL";
  const upper = lvl.toUpperCase();
  if (upper === "E" || upper === "ERR" || upper === "ERROR") return "E";
  if (upper === "W" || upper === "WRN" || upper === "WARN" || upper === "WARNING") return "W";
  if (upper === "I" || upper === "INF" || upper === "INFO") return "I";
  if (upper === "D" || upper === "DBG" || upper === "DEBUG") return "D";
  if (upper === "V" || upper === "VRB" || upper === "VERBOSE") return "V";
  return upper;
}

interface TermaxTerminalPaneProps {
  pane: PaneInfo;
  logs: LogEntry[];
  isActive: boolean;
  isMaximized: boolean;
  levelFilter?: string;
  tagFilter?: string;
  searchQuery?: string;
  onFocusPane?: (paneId: string) => void;
}

export const TermaxTerminalPane: React.FC<TermaxTerminalPaneProps> = ({
  pane,
  logs,
  isActive,
  isMaximized,
  levelFilter = "ALL",
  tagFilter = "",
  searchQuery = "",
  onFocusPane,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const trackRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const isAtBottomRef = useRef<boolean>(true);
  const isDraggingThumbRef = useRef<boolean>(false);
  const dragStartYRef = useRef<number>(0);
  const dragStartScrollYRef = useRef<number>(0);

  const [isScrolledUp, setIsScrolledUp] = useState<boolean>(false);
  const [unreadCount, setUnreadCount] = useState<number>(0);
  const [showScrollBadge, setShowScrollBadge] = useState<boolean>(false);
  const scrollBadgeTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const [scrollState, setScrollState] = useState<{
    viewportY: number;
    baseY: number;
    totalLines: number;
    rows: number;
  }>({
    viewportY: 0,
    baseY: 0,
    totalLines: 0,
    rows: 24,
  });

  const prevFilterRef = useRef<{ levelFilter: string; tagFilter: string; searchQuery: string }>({
    levelFilter: "ALL",
    tagFilter: "",
    searchQuery: "",
  });
  const renderedLogsCountRef = useRef<number>(0);

  // Sync scroll metrics from terminal buffer
  const updateScrollMetrics = useCallback((y?: number) => {
    const term = terminalRef.current;
    if (!term) return;
    const buf = (term as any).buffer?.active;
    if (!buf) return;

    const viewportY = typeof y === "number" ? y : (buf.viewportY ?? 0);
    const baseY = buf.baseY ?? 0;
    const totalLines = buf.length ?? 0;
    const rows = term.rows || 24;
    const atBottom = viewportY >= baseY;

    isAtBottomRef.current = atBottom;
    setIsScrolledUp(!atBottom);
    if (atBottom) {
      setUnreadCount(0);
    }

    setScrollState({
      viewportY,
      baseY,
      totalLines,
      rows,
    });

    setShowScrollBadge(true);
    if (scrollBadgeTimeoutRef.current) clearTimeout(scrollBadgeTimeoutRef.current);
    scrollBadgeTimeoutRef.current = setTimeout(() => {
      setShowScrollBadge(false);
    }, 1600);
  }, []);

  // Initialize Termax Terminal instance
  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      fontSize: 12,
      fontFamily: "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
      cursorBlink: false,
      cursorStyle: "bar",
      scrollback: 10000,
      theme: {
        background: "#080c14",
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

    // Patch Terminal scroll methods to sync Termax canvas renderer grid with vt.getVisibleLines()
    const syncViewport = (t: any) => {
      if (t.renderer && t.vt) {
        t.renderer.grid = t.vt.getVisibleLines();
        t.renderer.updateScroll(t.vt.viewportY, t.buffer?.active?.length || t.vt.scrollback.length);
        if (typeof t.renderer.renderAll === "function") {
          t.renderer.renderAll();
        }
      }
    };

    term.scrollLines = function (amount: number) {
      const t = this as any;
      t.vt.scrollLines(amount);
      syncViewport(t);
      for (const listener of t.scrollListeners) {
        listener(t.vt.viewportY);
      }
    };

    term.scrollToTop = function () {
      const t = this as any;
      t.vt.scrollToTop();
      syncViewport(t);
      for (const listener of t.scrollListeners) {
        listener(t.vt.viewportY);
      }
    };

    term.scrollToBottom = function () {
      const t = this as any;
      t.vt.scrollToBottom();
      syncViewport(t);
      for (const listener of t.scrollListeners) {
        listener(t.vt.viewportY);
      }
    };

    term.scrollToLine = function (line: number) {
      const t = this as any;
      t.vt.scrollToLine(line);
      syncViewport(t);
      for (const listener of t.scrollListeners) {
        listener(t.vt.viewportY);
      }
    };

    // Override getSelection to extract text from visible renderer grid instead of bottom vt.lines
    term.getSelection = function () {
      const t = this as any;
      if (!t.selection) return "";
      let sRow = t.selection.start.row;
      let sCol = t.selection.start.col;
      let eRow = t.selection.end.row;
      let eCol = t.selection.end.col;
      if (sRow > eRow || (sRow === eRow && sCol > eCol)) {
        const tR = sRow;
        sRow = eRow;
        eRow = tR;
        const tC = sCol;
        sCol = eCol;
        eCol = tC;
      }
      const sourceGrid = (t.renderer && t.renderer.grid) || t.vt.lines;
      const lines: string[] = [];
      for (let r = sRow; r <= eRow; r++) {
        if (r < sourceGrid.length) {
          const rowCells = sourceGrid[r];
          const startC = r === sRow ? sCol : 0;
          const endC = r === eRow ? eCol : t.cols - 1;
          const lineText = rowCells
            .slice(Math.max(0, startC), Math.min(t.cols, endC + 1))
            .map((c: any) => c.char || " ")
            .join("");
          lines.push(lineText.trimEnd());
        }
      }
      return lines.join("\n");
    };

    const fit = new FitAddon();
    term.loadAddon(fit);
    try {
      term.loadAddon(new WebLinksAddon());
    } catch (_) {}

    term.open(containerRef.current);
    fit.fit();

    terminalRef.current = term;
    fitAddonRef.current = fit;

    // Viewport scroll listener from Termax core
    term.onScroll((viewportY: number) => {
      updateScrollMetrics(viewportY);
    });

    // Keyboard Shortcuts for scrolling & Clipboard Copy (⌘C / Ctrl+C)
    const handleKeyDown = (e: KeyboardEvent) => {
      if (document.activeElement?.tagName === "INPUT" || document.activeElement?.tagName === "TEXTAREA") {
        return;
      }

      // Handle ⌘C / Ctrl+C for copying selected terminal logs
      if ((e.metaKey || e.ctrlKey) && (e.key === "c" || e.key === "C")) {
        if (term.hasSelection()) {
          const text = term.getSelection();
          if (text) {
            navigator.clipboard?.writeText(text).catch(() => {});
            e.preventDefault();
          }
        }
        return;
      }

      if (e.key === "PageUp") {
        e.preventDefault();
        term.scrollLines(-Math.max(5, term.rows - 2));
      } else if (e.key === "PageDown") {
        e.preventDefault();
        term.scrollLines(Math.max(5, term.rows - 2));
      } else if (e.key === "Home" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        term.scrollToTop();
      } else if (e.key === "End" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        term.scrollToBottom();
      }
    };

    // Native Window Copy Event Listener
    const handleCopy = (e: ClipboardEvent) => {
      if (!term.hasSelection()) return;
      const text = term.getSelection();
      if (!text) return;
      e.clipboardData?.setData("text/plain", text);
      e.preventDefault();
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("copy", handleCopy);

    const handleResize = () => {
      try {
        fit.fit();
        updateScrollMetrics();
      } catch (_) {}
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("copy", handleCopy);
      window.removeEventListener("resize", handleResize);
      if (scrollBadgeTimeoutRef.current) clearTimeout(scrollBadgeTimeoutRef.current);
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, [updateScrollMetrics]);

  // Format a LogEntry into ANSI escape strings
  const formatLogToAnsi = useCallback((entry: LogEntry): string => {
    const timeStr = entry.timestamp ? entry.timestamp.split("T")[1]?.slice(0, 12) || "" : "";
    let lvlAnsi = "";

    const normLvl = normalizeLogLevel(entry.level);
    switch (normLvl) {
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

  // Re-render logs when logs, filter, tag, or search change using incremental writes
  useEffect(() => {
    const term = terminalRef.current;
    if (!term) return;

    const normFilter = normalizeLogLevel(levelFilter);
    const filtered = logs.filter((log) => {
      if (normFilter !== "ALL" && normalizeLogLevel(log.level) !== normFilter) {
        return false;
      }
      if (tagFilter && (log.tag || "").toLowerCase() !== tagFilter.toLowerCase()) {
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

    const filterChanged =
      prevFilterRef.current.levelFilter !== levelFilter ||
      prevFilterRef.current.tagFilter !== tagFilter ||
      prevFilterRef.current.searchQuery !== searchQuery;

    if (filterChanged || filtered.length < renderedLogsCountRef.current) {
      term.clear();
      for (const log of filtered) {
        term.writeln(formatLogToAnsi(log));
      }
      renderedLogsCountRef.current = filtered.length;
      term.scrollToBottom();
      isAtBottomRef.current = true;
      setIsScrolledUp(false);
      setUnreadCount(0);
      updateScrollMetrics();
    } else if (filtered.length > renderedLogsCountRef.current) {
      const newLogs = filtered.slice(renderedLogsCountRef.current);
      for (const log of newLogs) {
        term.writeln(formatLogToAnsi(log));
      }
      renderedLogsCountRef.current = filtered.length;

      if (isAtBottomRef.current) {
        term.scrollToBottom();
      } else {
        setUnreadCount((c) => c + newLogs.length);
      }
      updateScrollMetrics();
    }

    prevFilterRef.current = { levelFilter, tagFilter, searchQuery };
  }, [logs, levelFilter, tagFilter, searchQuery, formatLogToAnsi, updateScrollMetrics]);

  useEffect(() => {
    const timer = setTimeout(() => {
      try {
        fitAddonRef.current?.fit();
        updateScrollMetrics();
      } catch (_) {}
    }, 60);
    return () => clearTimeout(timer);
  }, [isMaximized, updateScrollMetrics]);

  const handleScrollToTop = () => {
    const term = terminalRef.current;
    if (!term) return;
    term.scrollToTop();
    isAtBottomRef.current = false;
    setIsScrolledUp(true);
    updateScrollMetrics(0);
  };

  const handleScrollToBottom = () => {
    const term = terminalRef.current;
    if (!term) return;
    term.scrollToBottom();
    isAtBottomRef.current = true;
    setIsScrolledUp(false);
    setUnreadCount(0);
    updateScrollMetrics();
  };

  const handleJumpToLastError = () => {
    const term = terminalRef.current;
    if (!term) return;
    const normLogs = logs.filter((log) => {
      const normFilter = normalizeLogLevel(levelFilter);
      if (normFilter !== "ALL" && normalizeLogLevel(log.level) !== normFilter) return false;
      if (tagFilter && (log.tag || "").toLowerCase() !== tagFilter.toLowerCase()) return false;
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        return (log.message || "").toLowerCase().includes(q) || (log.tag || "").toLowerCase().includes(q);
      }
      return true;
    });
    const lastErrIdx = normLogs.map((l) => normalizeLogLevel(l.level)).lastIndexOf("E");
    if (lastErrIdx >= 0) {
      term.scrollToLine(lastErrIdx);
      isAtBottomRef.current = false;
      setIsScrolledUp(true);
      updateScrollMetrics(lastErrIdx);
    }
  };

  // Drag Scrollbar Thumb Handler
  const handleThumbMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    isDraggingThumbRef.current = true;
    dragStartYRef.current = e.clientY;
    dragStartScrollYRef.current = scrollState.viewportY;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isDraggingThumbRef.current || !trackRef.current || !terminalRef.current) return;
      const trackHeight = trackRef.current.clientHeight;
      const thumbHeight = Math.max(24, (scrollState.rows / Math.max(1, scrollState.totalLines)) * trackHeight);
      const availableTrack = trackHeight - thumbHeight;
      if (availableTrack <= 0) return;

      const deltaY = moveEvent.clientY - dragStartYRef.current;
      const scrollRatio = deltaY / availableTrack;
      const lineDelta = Math.round(scrollRatio * scrollState.baseY);
      const targetLine = Math.max(0, Math.min(scrollState.baseY, dragStartScrollYRef.current + lineDelta));
      terminalRef.current.scrollToLine(targetLine);
      updateScrollMetrics(targetLine);
    };

    const handleMouseUp = () => {
      isDraggingThumbRef.current = false;
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);
  };

  // Click on Scrollbar Track
  const handleTrackClick = (e: React.MouseEvent) => {
    if (!trackRef.current || !terminalRef.current) return;
    const rect = trackRef.current.getBoundingClientRect();
    const clickY = e.clientY - rect.top;
    const trackHeight = rect.height;
    const targetRatio = clickY / trackHeight;
    const targetLine = Math.round(targetRatio * scrollState.baseY);
    terminalRef.current.scrollToLine(targetLine);
    updateScrollMetrics(targetLine);
  };

  // Scrollbar dimensions
  const hasScrollbar = scrollState.baseY > 0;
  const thumbPercent = hasScrollbar ? (scrollState.viewportY / scrollState.baseY) * 100 : 0;
  const thumbHeightPercent = Math.max(
    8,
    Math.min(100, (scrollState.rows / Math.max(1, scrollState.totalLines)) * 100)
  );

  return (
    <div
      className={`terminal-pane ${isActive ? "active-pane" : ""} ${isMaximized ? "maximized" : ""}`}
      onMouseDown={() => onFocusPane?.(pane.id)}
    >
      <div ref={containerRef} className="pane-terminal-container" />

      {/* Interactive Custom Scrollbar Track & Thumb */}
      {hasScrollbar && (
        <div
          ref={trackRef}
          className="terminal-scrollbar-track"
          onClick={handleTrackClick}
          title="Click to jump / drag to scroll"
        >
          <div
            className="terminal-scrollbar-thumb"
            style={{
              top: `calc(${thumbPercent}% - ${(thumbPercent / 100) * thumbHeightPercent}%)`,
              height: `${thumbHeightPercent}%`,
            }}
            onMouseDown={handleThumbMouseDown}
          />
        </div>
      )}

      {/* Floating Scroll Position Indicator */}
      {hasScrollbar && showScrollBadge && (
        <div className="terminal-scroll-position-badge">
          {scrollState.viewportY >= scrollState.baseY
            ? "Live (Bottom)"
            : scrollState.viewportY === 0
            ? "Top"
            : `Line ${scrollState.viewportY + 1} / ${scrollState.totalLines} (${Math.round(
                (scrollState.viewportY / scrollState.baseY) * 100
              )}%)`}
        </div>
      )}

      {/* Floating Scroll Actions Toolbar */}
      <div className="terminal-floating-controls">
        <button
          className="btn-term-float"
          onClick={handleScrollToTop}
          title="Scroll to Top (Home / ⌘↑)"
        >
          <ArrowUp size={11} />
        </button>

        <button
          className="btn-term-float"
          onClick={handleScrollToBottom}
          title="Scroll to Bottom (End / ⌘↓)"
        >
          <ArrowDown size={11} />
        </button>

        {logs.some((l) => normalizeLogLevel(l.level) === "E") && (
          <button
            className="btn-term-float danger"
            onClick={handleJumpToLastError}
            title="Jump to Error"
          >
            <AlertCircle size={11} />
          </button>
        )}
      </div>

      {/* Sticky Unread New Logs Banner */}
      {isScrolledUp && (
        <button
          className="terminal-unread-banner"
          onClick={handleScrollToBottom}
          title="Scroll to latest logs"
        >
          <ChevronDown size={12} className="animate-bounce" />
          <span>
            {unreadCount > 0 ? `${unreadCount} new logs` : "Scroll to bottom"}
          </span>
        </button>
      )}
    </div>
  );
};


