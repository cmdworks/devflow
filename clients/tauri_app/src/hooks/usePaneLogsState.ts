import { useState, useEffect, useCallback, useRef } from "react";
import type { LayoutMode } from "../components/TerminalSecondaryBar";
import { getApiBase } from "./useDevFlowApi";
import type { PaneInfo, LogEntry, ProjectTarget, ServerEvent } from "../types";

export function usePaneLogsState() {
  const [openPanes, setOpenPanes] = useState<PaneInfo[]>([]);
  const [logsByPaneId, setLogsByPaneId] = useState<Record<string, LogEntry[]>>({});
  const [activePaneId, setActivePaneId] = useState<string>("");
  const [maximizedPaneId, setMaximizedPaneId] = useState<string | null>(null);

  const [layoutMode, setLayoutMode] = useState<LayoutMode>(() => {
    return (localStorage.getItem("devflow_layout_mode") as LayoutMode) || "tabs";
  });

  const [levelFilter, setLevelFilter] = useState<string>("ALL");
  const [tagFilter, setTagFilter] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [isLogTreeOpen, setIsLogTreeOpen] = useState<boolean>(() => {
    return localStorage.getItem("devflow_log_tree_open") === "true";
  });

  useEffect(() => {
    localStorage.setItem("devflow_layout_mode", layoutMode);
  }, [layoutMode]);

  useEffect(() => {
    localStorage.setItem("devflow_log_tree_open", isLogTreeOpen ? "true" : "false");
  }, [isLogTreeOpen]);

  // Append a log entry to a specific pane
  const appendLogToPane = useCallback((paneId: string, entry: LogEntry) => {
    setLogsByPaneId((prev) => {
      const existing = prev[paneId] || [];
      return {
        ...prev,
        [paneId]: [...existing, entry],
      };
    });
  }, []);

  // Update a pane's status
  const updatePaneStatus = useCallback((targetId: string, status: PaneInfo["status"]) => {
    setOpenPanes((prev) =>
      prev.map((p) => {
        if (p.targetId === targetId) {
          return { ...p, status };
        }
        return p;
      })
    );
  }, []);

  // Handle SSE events from backend
  const handleServerEvent = useCallback(
    (event: ServerEvent) => {
      if (!event || !event.type) return;

      if (event.type === "LogAppended") {
        const { session_id, entry } = event.payload;
        if (!entry) return;

        // Route to matching target pane(s)
        if (session_id) {
          const targetPaneId = `pane-${session_id}`;
          appendLogToPane(targetPaneId, entry);
        }

        // Also route to combined pane
        const taggedEntry: LogEntry = {
          ...entry,
          tag: entry.tag || session_id || "system",
        };
        appendLogToPane("pane-combined", taggedEntry);
      } else if (event.type === "BuildStarted") {
        const { session_id, project_name } = event.payload;
        updatePaneStatus(session_id, "building");
        appendLogToPane(`pane-${session_id}`, {
          timestamp: new Date().toISOString(),
          level: "I",
          tag: "build",
          message: `⚡ Build started for '${project_name}'...`,
        });
      } else if (event.type === "BuildCompleted") {
        const { session_id, result } = event.payload;
        if (result.success) {
          updatePaneStatus(session_id, "running");
          appendLogToPane(`pane-${session_id}`, {
            timestamp: new Date().toISOString(),
            level: "I",
            tag: "build",
            message: `✓ Build succeeded in ${result.duration_ms}ms`,
          });
        } else {
          updatePaneStatus(session_id, "error");
          appendLogToPane(`pane-${session_id}`, {
            timestamp: new Date().toISOString(),
            level: "E",
            tag: "build",
            message: `✗ Build failed in ${result.duration_ms}ms: ${result.error_message || "Unknown error"}`,
          });
        }
      } else if (event.type === "SessionStateChanged") {
        const { session_id, status } = event.payload;
        const normalized = status.toLowerCase() as PaneInfo["status"];
        updatePaneStatus(session_id, normalized);
      }
    },
    [appendLogToPane, updatePaneStatus]
  );

  // Connect to backend SSE event stream
  const eventHandlerRef = useRef(handleServerEvent);
  eventHandlerRef.current = handleServerEvent;

  useEffect(() => {
    const apiBase = getApiBase();
    const es = new EventSource(`${apiBase}/api/events`);
    es.onmessage = (msg) => {
      try {
        const parsed: ServerEvent = JSON.parse(msg.data);
        eventHandlerRef.current(parsed);
      } catch (err) {
        console.warn("Failed to parse SSE event:", err);
      }
    };
    return () => {
      es.close();
    };
  }, []);

  // Open pane for a target
  const openTargetPane = useCallback((target: ProjectTarget) => {
    const paneId = `pane-${target.id}`;
    setOpenPanes((prev) => {
      if (prev.some((p) => p.id === paneId)) {
        return prev;
      }
      const newPane: PaneInfo = {
        id: paneId,
        targetId: target.id,
        title: `${target.name} [${target.framework}]`,
        platform: target.platform,
        framework: target.framework,
        isCombined: false,
        status: "idle",
        target,
      };
      return [...prev, newPane];
    });
    setActivePaneId(paneId);
    setMaximizedPaneId(null);
  }, []);

  // Open combined aggregator pane
  const openCombinedPane = useCallback(() => {
    setOpenPanes((prev) => {
      if (prev.some((p) => p.id === "pane-combined")) {
        return prev;
      }
      const combinedPane: PaneInfo = {
        id: "pane-combined",
        targetId: "all",
        title: "Combined Stream",
        platform: "universal",
        isCombined: true,
        status: "running",
      };
      return [...prev, combinedPane];
    });
    setActivePaneId("pane-combined");
    setMaximizedPaneId(null);
  }, []);

  const handleClosePane = useCallback((paneId: string) => {
    setOpenPanes((prev) => {
      const filtered = prev.filter((p) => p.id !== paneId);
      if (activePaneId === paneId && filtered.length > 0) {
        setActivePaneId(filtered[0].id);
      }
      return filtered;
    });
  }, [activePaneId]);

  const handleClearActiveLogs = useCallback(() => {
    if (activePaneId) {
      setLogsByPaneId((prev) => ({
        ...prev,
        [activePaneId]: [],
      }));
    }
  }, [activePaneId]);

  const handleToggleMaximize = useCallback(() => {
    setMaximizedPaneId((prev) => (prev ? null : activePaneId));
  }, [activePaneId]);

  return {
    openPanes,
    setOpenPanes,
    logsByPaneId,
    setLogsByPaneId,
    activePaneId,
    setActivePaneId,
    maximizedPaneId,
    setMaximizedPaneId,
    layoutMode,
    setLayoutMode,
    levelFilter,
    setLevelFilter,
    tagFilter,
    setTagFilter,
    searchQuery,
    setSearchQuery,
    isLogTreeOpen,
    setIsLogTreeOpen,
    appendLogToPane,
    updatePaneStatus,
    openTargetPane,
    openCombinedPane,
    handleClosePane,
    handleClearActiveLogs,
    handleToggleMaximize,
  };
}
