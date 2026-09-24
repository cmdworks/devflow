import React, { useEffect, useRef, useCallback, useState } from "react";
import { Terminal } from "@termaxjs/web/canvas";
import { FitAddon, WebLinksAddon } from "@termaxjs/web/addons";
import {
  ArrowDown,
  ArrowUp,
  AlertCircle,
  ChevronDown,
  Copy,
  Trash2,
  Check,
} from "lucide-react";
import type { LogEntry, PaneInfo, ActiveSessionInfo } from "../types";
import { TargetActionHeader } from "./TargetActionHeader";
import { copyToClipboard } from "../utils/clipboard";

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
  activeSession?: ActiveSessionInfo;
  executingAction?: string;
  onFocusPane?: (paneId: string) => void;
  onToggleRun?: (targetId: string) => void;
  onReload?: (targetId: string) => void;
  onRestart?: (targetId: string) => void;
  onExecuteAction?: (targetId: string, action: string, port?: number) => void;
  onOpenDevOptions?: (targetId?: string) => void;
  onClearLogs?: (paneId: string) => void;
}

export const TermaxTerminalPane: React.FC<TermaxTerminalPaneProps> = ({
  pane,
  logs,
  isActive,
  isMaximized,
  levelFilter = "ALL",
  tagFilter = "",
  searchQuery = "",
  activeSession,
  executingAction,
  onFocusPane,
  onToggleRun = () => {},
  onReload = () => {},
  onRestart = () => {},
  onExecuteAction = () => {},
  onOpenDevOptions,
  onClearLogs,
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
  const [copied, setCopied] = useState<boolean>(false);
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
        brightCyan: "#22d3ee",
        brightWhite: "#ffffff",
      },
    });

    const fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    term.loadAddon(fitAddon);
    term.loadAddon(webLinksAddon);
    term.open(containerRef.current);

    try {
      fitAddon.fit();
    } catch (_) {}

    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    term.onScroll((y: number) => {
      updateScrollMetrics(y);
    });

    term.onLineFeed(() => {
      if (isAtBottomRef.current) {
        term.scrollToBottom();
      }
      updateScrollMetrics();
    });

    term.writeln(
      `\x1b[90m═══ DevFlow Live Stream [${pane.title}] ═══\x1b[0m`
    );

    const handleResize = () => {
      try {
        fitAddon.fit();
        updateScrollMetrics();
      } catch (_) {}
    };
    window.addEventListener("resize", handleResize);

    const resizeObserver = new ResizeObserver(() => {
      try {
        fitAddon.fit();
        updateScrollMetrics();
      } catch (_) {}
    });
    const container = containerRef.current;
    const handleWheel = (e: WheelEvent) => {
      const term = terminalRef.current;
      if (!term) return;
      e.preventDefault();
      const delta = e.deltaY;
      if (delta !== 0) {
        const lines = Math.max(1, Math.round(Math.abs(delta) / 25)) * (delta > 0 ? 1 : -1);
        if (typeof (term as any).scrollLines === "function") {
          (term as any).scrollLines(lines);
        }
        updateScrollMetrics();
      }
    };
    if (container) {
      container.addEventListener("wheel", handleWheel, { passive: false });
    }

    const handleKeyDown = async (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "c") {
        const term = terminalRef.current;
        if (term) {
          const hasSel = typeof (term as any).hasSelection === "function" ? (term as any).hasSelection() : false;
          const sel = typeof (term as any).getSelection === "function" ? (term as any).getSelection() : "";
          if (hasSel || (sel && sel.length > 0)) {
            e.preventDefault();
            await copyToClipboard(sel);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
          }
        }
      }
    };
    const handleCopyEvent = async (e: ClipboardEvent) => {
      const term = terminalRef.current;
      if (term) {
        const hasSel = typeof (term as any).hasSelection === "function" ? (term as any).hasSelection() : false;
        const sel = typeof (term as any).getSelection === "function" ? (term as any).getSelection() : "";
        if (hasSel || (sel && sel.length > 0)) {
          e.preventDefault();
          if (e.clipboardData) {
            e.clipboardData.setData("text/plain", sel);
          } else {
            await copyToClipboard(sel);
          }
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("copy", handleCopyEvent);

    return () => {
      if (container) {
        container.removeEventListener("wheel", handleWheel);
      }
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("copy", handleCopyEvent);
      resizeObserver.disconnect();
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pane.id]);

  // Refit on pane maximize/minimize or focus
  useEffect(() => {
    if (fitAddonRef.current) {
      setTimeout(() => {
        try {
          fitAddonRef.current?.fit();
          updateScrollMetrics();
        } catch (_) {}
      }, 50);
    }
  }, [isMaximized, isActive, updateScrollMetrics]);

  // Render log line formatting
  const formatLogLine = useCallback(
    (entry: LogEntry): string | null => {
      const normalizedLevel = normalizeLogLevel(entry.level);
      const activeNormalizedFilter = normalizeLogLevel(levelFilter);

      // 1. Level Filter check
      if (activeNormalizedFilter !== "ALL") {
        if (normalizedLevel !== activeNormalizedFilter) {
          return null;
        }
      }

      // 2. Tag Filter check
      if (tagFilter && tagFilter.trim() !== "") {
        const entryTag = (entry.tag || "system").toLowerCase();
        if (!entryTag.includes(tagFilter.toLowerCase().trim())) {
          return null;
        }
      }

      // 3. Search Query check
      if (searchQuery && searchQuery.trim() !== "") {
        const search = searchQuery.toLowerCase().trim();
        const msg = (entry.message || "").toLowerCase();
        const tag = (entry.tag || "").toLowerCase();
        if (!msg.includes(search) && !tag.includes(search)) {
          return null;
        }
      }

      // Format line
      let levelBadge = `\x1b[90m[${normalizedLevel}]\x1b[0m`;
      if (normalizedLevel === "E") {
        levelBadge = `\x1b[41;97;1m ERR \x1b[0m`;
      } else if (normalizedLevel === "W") {
        levelBadge = `\x1b[43;30;1m WRN \x1b[0m`;
      } else if (normalizedLevel === "I") {
        levelBadge = `\x1b[36;1m INF \x1b[0m`;
      } else if (normalizedLevel === "D") {
        levelBadge = `\x1b[35m DBG \x1b[0m`;
      }

      const timeStr = entry.timestamp
        ? new Date(entry.timestamp).toLocaleTimeString("en-US", {
            hour12: false,
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          })
        : "--:--:--";

      const tagPart = entry.tag ? `\x1b[33m<${entry.tag}>\x1b[0m ` : "";
      return `\x1b[90m${timeStr}\x1b[0m ${levelBadge} ${tagPart}${entry.message}`;
    },
    [levelFilter, tagFilter, searchQuery]
  );

  // Re-render logs when logs change or filters change
  useEffect(() => {
    const term = terminalRef.current;
    if (!term) return;

    const filtersChanged =
      prevFilterRef.current.levelFilter !== levelFilter ||
      prevFilterRef.current.tagFilter !== tagFilter ||
      prevFilterRef.current.searchQuery !== searchQuery;

    if (filtersChanged) {
      term.clear();
      term.writeln(`\x1b[90m═══ Filter applied: [${levelFilter}] search: "${searchQuery}" ═══\x1b[0m`);

      for (const entry of logs) {
        const formatted = formatLogLine(entry);
        if (formatted !== null) {
          term.writeln(formatted);
        }
      }

      renderedLogsCountRef.current = logs.length;
      prevFilterRef.current = { levelFilter, tagFilter, searchQuery };

      if (isAtBottomRef.current) {
        term.scrollToBottom();
      }
      updateScrollMetrics();
      return;
    }

    // Append incremental new logs
    const prevCount = renderedLogsCountRef.current;
    if (logs.length > prevCount) {
      const newEntries = logs.slice(prevCount);
      let addedVisible = 0;

      for (const entry of newEntries) {
        const formatted = formatLogLine(entry);
        if (formatted !== null) {
          term.writeln(formatted);
          addedVisible++;
        }
      }

      renderedLogsCountRef.current = logs.length;

      if (isAtBottomRef.current) {
        term.scrollToBottom();
      } else if (addedVisible > 0) {
        setUnreadCount((prev) => prev + addedVisible);
      }
      updateScrollMetrics();
    } else if (logs.length < prevCount) {
      // Clear was called
      term.clear();
      renderedLogsCountRef.current = 0;
      updateScrollMetrics();
    }
  }, [logs, levelFilter, tagFilter, searchQuery, formatLogLine, updateScrollMetrics]);

  const handleScrollToBottom = () => {
    const term = terminalRef.current;
    if (term) {
      term.scrollToBottom();
      isAtBottomRef.current = true;
      setIsScrolledUp(false);
      setUnreadCount(0);
      updateScrollMetrics();
    }
  };

  const handleScrollToTop = () => {
    const term = terminalRef.current;
    if (term) {
      term.scrollToTop();
      isAtBottomRef.current = false;
      setIsScrolledUp(true);
      updateScrollMetrics();
    }
  };

  const handleJumpToLastError = () => {
    const term = terminalRef.current;
    if (!term) return;
    handleScrollToBottom();
  };

  const handleCopyAllLogs = async () => {
    const text = logs.map((l) => `[${l.level}] ${l.timestamp || ""} ${l.message}`).join("\n");
    if (!text) return;
    const ok = await copyToClipboard(text);
    if (ok) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleThumbMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    isDraggingThumbRef.current = true;
    dragStartYRef.current = e.clientY;
    dragStartScrollYRef.current = scrollState.viewportY;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isDraggingThumbRef.current || !trackRef.current || !terminalRef.current) return;
      const trackHeight = trackRef.current.clientHeight;
      const deltaY = moveEvent.clientY - dragStartYRef.current;
      const deltaRatio = deltaY / trackHeight;
      const deltaLines = deltaRatio * scrollState.baseY;
      const targetLine = Math.max(
        0,
        Math.min(scrollState.baseY, Math.round(dragStartScrollYRef.current + deltaLines))
      );

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

  const errorCount = logs.filter((l) => normalizeLogLevel(l.level) === "E").length;
  const warnCount = logs.filter((l) => normalizeLogLevel(l.level) === "W").length;

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
      {/* 1. Top Action Toolbar */}
      <TargetActionHeader
        pane={pane}
        target={pane.target}
        activeSession={activeSession}
        executingAction={executingAction}
        onToggleRun={onToggleRun}
        onReload={onReload}
        onRestart={onRestart}
        onExecuteAction={onExecuteAction}
        onOpenDevOptions={onOpenDevOptions}
      />

      {/* 2. Center Terminal Canvas Area */}
      <div className="terminal-canvas-wrapper">
        <div ref={containerRef} className="pane-terminal-container" />

        {/* Custom Interactive Scrollbar Track */}
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

          {errorCount > 0 && (
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

      {/* 3. Bottom Status & Utility Bar */}
      <div className="terminal-bottom-utility-bar">
        <div className="bottom-bar-left">
          <span className="log-metric-item">
            <strong>{logs.length}</strong> lines
          </span>
          {errorCount > 0 && (
            <span className="log-metric-item error">
              <strong>{errorCount}</strong> errors
            </span>
          )}
          {warnCount > 0 && (
            <span className="log-metric-item warn">
              <strong>{warnCount}</strong> warnings
            </span>
          )}
          {levelFilter !== "ALL" && (
            <span className="log-metric-item filter">
              Level: <strong>{levelFilter}</strong>
            </span>
          )}
        </div>

        <div className="bottom-bar-right">
          <button
            className="btn-bottom-util"
            onClick={handleCopyAllLogs}
            title="Copy logs to clipboard"
          >
            {copied ? <Check size={11} color="#10b981" /> : <Copy size={11} />}
            <span>{copied ? "Copied" : "Copy"}</span>
          </button>

          {onClearLogs && (
            <button
              className="btn-bottom-util"
              onClick={() => onClearLogs(pane.id)}
              title="Clear log buffer"
            >
              <Trash2 size={11} />
              <span>Clear</span>
            </button>
          )}

          <button
            className={`btn-bottom-util ${!isScrolledUp ? "active" : ""}`}
            onClick={handleScrollToBottom}
            title="Toggle Auto-Scroll"
          >
            <span>Auto-Scroll: {!isScrolledUp ? "ON" : "OFF"}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
