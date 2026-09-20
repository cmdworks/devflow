import React, { useState, useEffect, useCallback, useRef } from "react";
import { TopBar } from "./components/TopBar";
import { PrimarySidebar } from "./components/PrimarySidebar";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { TerminalSecondaryBar, type LayoutMode } from "./components/TerminalSecondaryBar";
import { TerminalGrid } from "./components/TerminalGrid";
import { TerminalLogTree } from "./components/TerminalLogTree";
import { WorkspaceOverviewView } from "./components/WorkspaceOverviewView";
import { TargetsView } from "./components/TargetsView";
import { DevicesView } from "./components/DevicesView";
import { DoctorView } from "./components/DoctorView";
import { McpView } from "./components/McpView";
import { SettingsView } from "./components/SettingsView";
import { WorkspaceModal } from "./components/WorkspaceModal";
import { UpdateModal } from "./components/UpdateModal";
import { ProcessStatusToast, type BatchProgressInfo } from "./components/ProcessStatusToast";
import { DevServerOptionsModal, type RunnerOptionsConfig } from "./components/DevServerOptionsModal";
import { useDevFlowApi } from "./hooks/useDevFlowApi";
import { useDevFlowEvents } from "./hooks/useDevFlowEvents";
import type {
  ProjectTarget,
  Device,
  ActiveSessionInfo,
  PaneInfo,
  LogEntry,
  DoctorReport,
  ServerEvent,
  KnownWorkspace,
  ViewSection,
  WorkspaceResponse,
  UpdateCheckResponse,
} from "./types";

interface WorkspaceCacheItem {
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activePaneId: string;
  logsByPaneId: Record<string, LogEntry[]>;
  activeSection: ViewSection;
}

export const App: React.FC = () => {
  const api = useDevFlowApi();

  // Navigation View State (persisted across sessions)
  const [activeSection, setActiveSection] = useState<ViewSection>(() => {
    return (localStorage.getItem("devflow_active_section") as ViewSection) || "overview";
  });
  const [isPrimarySidebarCollapsed, setIsPrimarySidebarCollapsed] = useState<boolean>(() => {
    return localStorage.getItem("devflow_primary_sidebar_collapsed") === "true";
  });
  const [isSecondarySidebarCollapsed, setIsSecondarySidebarCollapsed] = useState<boolean>(() => {
    return localStorage.getItem("devflow_secondary_sidebar_collapsed") === "true";
  });

  // Log Filtering State
  const [levelFilter, setLevelFilter] = useState<string>("ALL");
  const [tagFilter, setTagFilter] = useState<string>("");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [isLogTreeOpen, setIsLogTreeOpen] = useState<boolean>(() => {
    return localStorage.getItem("devflow_log_tree_open") === "true";
  });

  // Custom Workspace Names (persisted)
  const [customNames, setCustomNames] = useState<Record<string, string>>(() => {
    try {
      return JSON.parse(localStorage.getItem("devflow_custom_workspace_names") || "{}");
    } catch {
      return {};
    }
  });

  // Workspace & Environment State
  const [workspaceName, setWorkspaceName] = useState<string>("Workspace");
  const [workspacePath, setWorkspacePath] = useState<string>("");
  const [knownWorkspaces, setKnownWorkspaces] = useState<KnownWorkspace[]>([]);
  const [targets, setTargets] = useState<ProjectTarget[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [activeSessions, setActiveSessions] = useState<ActiveSessionInfo[]>([]);

  // Terminal & Logs State
  const [openPanes, setOpenPanes] = useState<PaneInfo[]>([]);
  const [logsByPaneId, setLogsByPaneId] = useState<Record<string, LogEntry[]>>({});
  const [activePaneId, setActivePaneId] = useState<string>("");
  const [maximizedPaneId, setMaximizedPaneId] = useState<string | null>(null);
  const [layoutMode, setLayoutMode] = useState<LayoutMode>(() => {
    return (localStorage.getItem("devflow_layout_mode") as LayoutMode) || "tabs";
  });

  // Diagnostics / Doctor State
  const [doctorReport, setDoctorReport] = useState<DoctorReport | null>(null);
  const [isDoctorLoading, setIsDoctorLoading] = useState<boolean>(false);

  // Modal States
  const [isWorkspaceModalOpen, setIsWorkspaceModalOpen] = useState<boolean>(false);
  const [devOptionsModal, setDevOptionsModal] = useState<{ isOpen: boolean; targetId?: string }>({
    isOpen: false,
  });
  const [isUpdateModalOpen, setIsUpdateModalOpen] = useState<boolean>(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResponse | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState<boolean>(false);

  // Batch Lifecycle Loading & Status Toast States
  const [isStartingAll, setIsStartingAll] = useState<boolean>(false);
  const [isStoppingAll, setIsStoppingAll] = useState<boolean>(false);
  const [batchProgress, setBatchProgress] = useState<BatchProgressInfo | null>(null);

  // Auto-Updater handlers
  const handleCheckUpdates = useCallback(async () => {
    setIsCheckingUpdate(true);
    try {
      const info = await api.checkUpdate();
      setUpdateInfo(info);
      if (info.update_available) {
        setIsUpdateModalOpen(true);
      }
    } catch (e) {
      console.error("Failed to check for updates", e);
    } finally {
      setIsCheckingUpdate(false);
    }
  }, [api]);

  const handleInstallUpdate = useCallback(
    async (downloadUrl: string) => {
      return await api.installUpdate(downloadUrl);
    },
    [api]
  );

  const handleRestartApp = useCallback(async () => {
    await api.restartApp();
  }, [api]);

  // Silent background update check on app launch
  useEffect(() => {
    const timer = setTimeout(() => {
      api
        .checkUpdate()
        .then((info) => {
          setUpdateInfo(info);
        })
        .catch((e) => {
          console.debug("Background update check skipped:", e);
        });
    }, 2000);
    return () => clearTimeout(timer);
  }, [api]);

  // In-Memory Stateful Workspace Cache (Zero-flicker 0ms switching)
  const workspaceCacheRef = useRef<Record<string, WorkspaceCacheItem>>({});

  useEffect(() => {
    localStorage.setItem("devflow_active_section", activeSection);
  }, [activeSection]);

  useEffect(() => {
    localStorage.setItem("devflow_layout_mode", layoutMode);
  }, [layoutMode]);

  useEffect(() => {
    localStorage.setItem(
      "devflow_primary_sidebar_collapsed",
      isPrimarySidebarCollapsed ? "true" : "false"
    );
  }, [isPrimarySidebarCollapsed]);

  useEffect(() => {
    localStorage.setItem(
      "devflow_secondary_sidebar_collapsed",
      isSecondarySidebarCollapsed ? "true" : "false"
    );
  }, [isSecondarySidebarCollapsed]);

  useEffect(() => {
    localStorage.setItem("devflow_log_tree_open", isLogTreeOpen ? "true" : "false");
  }, [isLogTreeOpen]);

  useEffect(() => {
    localStorage.setItem("devflow_custom_workspace_names", JSON.stringify(customNames));
    setKnownWorkspaces((prev) =>
      prev.map((w) => ({
        ...w,
        custom_name: customNames[w.path] || w.name,
      }))
    );
  }, [customNames]);

  // Load known workspaces once at startup
  const loadKnownWorkspaces = useCallback(async () => {
    try {
      const list = await api.fetchWorkspaces();
      if (list && Array.isArray(list)) {
        setKnownWorkspaces(
          list.map((w) => ({
            ...w,
            custom_name: customNames[w.path] || w.name,
          }))
        );
      }
    } catch (err) {
      console.error("Failed to load known workspaces:", err);
    }
  }, [api.fetchWorkspaces, customNames]);

  useEffect(() => {
    loadKnownWorkspaces();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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

  // Connect to SSE stream
  useDevFlowEvents(handleServerEvent);

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

  // Core unified stateful workspace applier
  const applyWorkspaceData = useCallback(
    (data: WorkspaceResponse) => {
      const resolvedPath = data.workspace_path;
      const displayName = customNames[resolvedPath] || data.workspace_name;

      setWorkspaceName(displayName);
      setWorkspacePath(resolvedPath);
      setTargets(data.targets || []);
      setDevices(data.devices || []);
      setActiveSessions(data.active_sessions || []);

      // Optimistically update known workspaces
      setKnownWorkspaces((prev) => {
        const exists = prev.some((w) => w.path === resolvedPath);
        if (exists) {
          return prev.map((w) =>
            w.path === resolvedPath
              ? { ...w, last_opened: new Date().toISOString(), custom_name: displayName }
              : w
          );
        }
        return [
          {
            name: data.workspace_name,
            custom_name: displayName,
            path: resolvedPath,
            platform: data.targets?.[0]?.platform || "desktop",
            framework: data.targets?.[0]?.framework || "generic",
            last_opened: new Date().toISOString(),
          },
          ...prev,
        ];
      });

      // Restore or build terminal panes for this workspace
      const cached = workspaceCacheRef.current[resolvedPath];
      if (cached && cached.openPanes && cached.openPanes.length > 0) {
        setOpenPanes(cached.openPanes);
        setActivePaneId(cached.activePaneId);
        setLogsByPaneId(cached.logsByPaneId || {});
      } else {
        // Construct fresh panes for this workspace's discovered targets
        const targetPanes: PaneInfo[] = (data.targets || []).map((t: ProjectTarget) => ({
          id: `pane-${t.id}`,
          targetId: t.id,
          title: `${t.name} [${t.framework}]`,
          platform: t.platform,
          framework: t.framework,
          isCombined: false,
          status: "idle",
          target: t,
        }));

        const combinedPane: PaneInfo = {
          id: "pane-combined",
          targetId: "all",
          title: "Combined Stream",
          platform: "universal",
          isCombined: true,
          status: "running",
        };

        const initialPanes = targetPanes.length > 0 ? [...targetPanes, combinedPane] : [combinedPane];
        setOpenPanes(initialPanes);
        setActivePaneId(initialPanes[0]?.id || "pane-combined");

        workspaceCacheRef.current[resolvedPath] = {
          targets: data.targets || [],
          openPanes: initialPanes,
          activePaneId: initialPanes[0]?.id || "pane-combined",
          logsByPaneId: {},
          activeSection: "terminal",
        };
      }

      // Sync URL without page reload
      try {
        const url = new URL(window.location.href);
        url.searchParams.set("dir", resolvedPath);
        window.history.pushState({}, "", url.toString());
      } catch (_) {}
    },
    [customNames]
  );

  // Load workspace data with caching
  const loadWorkspace = useCallback(
    async (dirPath?: string) => {
      try {
        const data = await api.fetchWorkspace(dirPath);
        applyWorkspaceData(data);
      } catch (err) {
        console.error("Failed to load workspace:", err);
      }
    },
    [api, applyWorkspaceData]
  );

  // Initial load from URL query (runs once on mount)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const dir = params.get("dir") || "";
    loadWorkspace(dir);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Stateful Zero-Flicker Workspace Switcher
  const handleSwitchWorkspace = useCallback(
    (newPath: string, immediateData?: WorkspaceResponse) => {
      if (!newPath) return;

      // 1. Cache current workspace state in memory
      if (workspacePath && workspacePath !== newPath) {
        workspaceCacheRef.current[workspacePath] = {
          targets,
          openPanes,
          activePaneId,
          logsByPaneId,
          activeSection,
        };
      }

      // 2. Optimistically switch path & name immediately for 0ms response
      setWorkspacePath(newPath);
      const known = knownWorkspaces.find((w) => w.path === newPath);
      if (known) {
        setWorkspaceName(customNames[newPath] || known.custom_name || known.name);
      } else {
        const basename = newPath.split("/").filter(Boolean).pop() || "Workspace";
        setWorkspaceName(customNames[newPath] || basename);
      }

      // 3. Update URL search param without page reload
      try {
        const url = new URL(window.location.href);
        url.searchParams.set("dir", newPath);
        window.history.pushState({}, "", url.toString());
      } catch (_) {}

      // 4. If immediate data is provided (e.g. from addWorkspace), apply immediately
      if (immediateData) {
        applyWorkspaceData(immediateData);
        return;
      }

      // 5. Restore cached panes if already visited in this session
      const cached = workspaceCacheRef.current[newPath];
      if (cached) {
        setTargets(cached.targets);
        setOpenPanes(cached.openPanes);
        setActivePaneId(cached.activePaneId);
        setLogsByPaneId(cached.logsByPaneId || {});
      }

      // 6. Fetch fresh workspace targets & details from backend
      loadWorkspace(newPath);
    },
    [
      workspacePath,
      targets,
      openPanes,
      activePaneId,
      logsByPaneId,
      activeSection,
      knownWorkspaces,
      customNames,
      applyWorkspaceData,
      loadWorkspace,
    ]
  );

  // Rename Workspace Alias
  const handleRenameWorkspace = useCallback((path: string, newName: string) => {
    setCustomNames((prev) => {
      const updated = { ...prev, [path]: newName };
      return updated;
    });
    setKnownWorkspaces((prev) =>
      prev.map((w) => (w.path === path ? { ...w, custom_name: newName } : w))
    );
    if (path === workspacePath) {
      setWorkspaceName(newName);
    }
  }, [workspacePath]);

  // Remove Workspace from List
  const handleRemoveWorkspace = useCallback(
    async (path: string) => {
      try {
        await api.removeWorkspace(path);
        setKnownWorkspaces((prev) => {
          const remaining = prev.filter((w) => w.path !== path);
          // If the removed workspace was the currently active workspace, switch to next available!
          if (path === workspacePath && remaining.length > 0) {
            handleSwitchWorkspace(remaining[0].path);
          }
          return remaining;
        });
      } catch (err) {
        console.error("Failed to remove workspace:", err);
      }
    },
    [api, workspacePath, handleSwitchWorkspace]
  );

  // Run / Stop target toggle
  const handleToggleRun = async (targetId: string) => {
    const pane = openPanes.find((p) => p.targetId === targetId);
    if (!pane) return;

    const isRunning = pane.status === "running" || pane.status === "building";

    if (isRunning) {
      updatePaneStatus(targetId, "idle");
      appendLogToPane(pane.id, {
        timestamp: new Date().toISOString(),
        level: "I",
        tag: "client",
        message: `■ Stopping target '${pane.title}'...`,
      });
      await api.stopTarget(targetId);
      return;
    }

    updatePaneStatus(targetId, "building");
    appendLogToPane(pane.id, {
      timestamp: new Date().toISOString(),
      level: "I",
      tag: "client",
      message: `▶ Requesting build & run for '${pane.title}'...`,
    });

    const target = pane.target || targets.find((t) => t.id === targetId);
    if (!target) return;

    // Match device
    const matchedDev = devices.find((d) => {
      if (target.platform.toLowerCase() === "android") return d.platform.toLowerCase() === "android";
      if (target.platform.toLowerCase() === "macos" || target.platform.toLowerCase() === "desktop") {
        return d.platform.toLowerCase() === "desktop";
      }
      return true;
    });

    const res = await api.startTarget(target.id, target.path, target.framework, matchedDev?.id);
    if (!res.success) {
      updatePaneStatus(targetId, "error");
      appendLogToPane(pane.id, {
        timestamp: new Date().toISOString(),
        level: "E",
        tag: "client",
        message: `✗ Failed to start target: ${res.error || "Unknown error"}`,
      });
    }
  };

  const handleReload = async (targetId: string) => {
    await api.reloadTarget(targetId);
  };

  const handleRestart = async (targetId: string) => {
    await api.restartTarget(targetId);
  };

  const handleLaunchWithOptions = async (
    targetId: string,
    options: RunnerOptionsConfig
  ) => {
    if (targetId === "all") {
      await handleRunAll();
      return;
    }

    const target = targets.find((t) => t.id === targetId);
    if (!target) return;

    openTargetPane(target);
    const paneId = `pane-${target.id}`;
    if (options.openLogs) {
      setActivePaneId(paneId);
      setActiveSection("terminal");
    }

    updatePaneStatus(target.id, "building");
    appendLogToPane(paneId, {
      timestamp: new Date().toISOString(),
      level: "I",
      tag: "client",
      message: `⚡ Launching '${target.name}' with profile [${options.mode.toUpperCase()}]${
        options.port ? ` on port ${options.port}` : ""
      }${options.customArgs ? ` args: ${options.customArgs}` : ""}...`,
    });

    const matchedDev = devices.find((d) => {
      if (target.platform.toLowerCase() === "android") return d.platform.toLowerCase() === "android";
      if (target.platform.toLowerCase() === "macos" || target.platform.toLowerCase() === "desktop") {
        return d.platform.toLowerCase() === "desktop";
      }
      return true;
    });

    const res = await api.startTarget(target.id, target.path, target.framework, matchedDev?.id);
    if (!res.success) {
      updatePaneStatus(target.id, "error");
      appendLogToPane(paneId, {
        timestamp: new Date().toISOString(),
        level: "E",
        tag: "client",
        message: `✗ Failed to start target: ${res.error || "Unknown error"}`,
      });
    }
  };

  const handleRunAll = async () => {
    if (isStartingAll || isStoppingAll) return;
    setIsStartingAll(true);
    try {
      const targetsToStart = targets.filter((t) => {
        const pane = openPanes.find((p) => p.targetId === t.id);
        return !(pane?.status === "running" || pane?.status === "building");
      });

      if (targetsToStart.length === 0) return;

      for (const t of targetsToStart) {
        openTargetPane(t);
      }

      setBatchProgress({
        active: true,
        type: "start",
        step: 0,
        total: targetsToStart.length,
        targetName: targetsToStart[0]?.name,
      });

      for (let i = 0; i < targetsToStart.length; i++) {
        const target = targetsToStart[i];
        const paneId = `pane-${target.id}`;

        setBatchProgress({
          active: true,
          type: "start",
          step: i + 1,
          total: targetsToStart.length,
          targetName: target.name,
        });

        updatePaneStatus(target.id, "building");
        appendLogToPane(paneId, {
          timestamp: new Date().toISOString(),
          level: "I",
          tag: "client",
          message: `▶ Requesting build & run for '${target.name}'...`,
        });

        const matchedDev = devices.find((d) => {
          if (target.platform.toLowerCase() === "android") return d.platform.toLowerCase() === "android";
          if (target.platform.toLowerCase() === "macos" || target.platform.toLowerCase() === "desktop") {
            return d.platform.toLowerCase() === "desktop";
          }
          return true;
        });

        const res = await api.startTarget(target.id, target.path, target.framework, matchedDev?.id);
        if (!res.success) {
          updatePaneStatus(target.id, "error");
          appendLogToPane(paneId, {
            timestamp: new Date().toISOString(),
            level: "E",
            tag: "client",
            message: `✗ Failed to start target: ${res.error || "Unknown error"}`,
          });
        }
      }

      setBatchProgress({
        active: true,
        type: "start",
        step: targetsToStart.length,
        total: targetsToStart.length,
        completed: true,
        message: `Successfully launched ${targetsToStart.length} target${
          targetsToStart.length === 1 ? "" : "s"
        }`,
      });

      setTimeout(() => {
        setBatchProgress((prev) => (prev?.completed ? null : prev));
      }, 2500);
    } catch (e) {
      console.error("Run all error:", e);
    } finally {
      setIsStartingAll(false);
    }
  };

  const handleReloadAll = async () => {
    setBatchProgress({
      active: true,
      type: "reload",
      step: 1,
      total: 1,
      message: "Triggering framework hot reload on active sessions...",
    });
    try {
      await api.reloadAll();
      setBatchProgress({
        active: true,
        type: "reload",
        step: 1,
        total: 1,
        completed: true,
        message: "Hot reload applied successfully",
      });
      setTimeout(() => {
        setBatchProgress((prev) => (prev?.completed ? null : prev));
      }, 2000);
    } catch (e) {
      console.error("Reload all error:", e);
      setBatchProgress(null);
    }
  };

  const handleRestartAll = async () => {
    setBatchProgress({
      active: true,
      type: "restart",
      step: 1,
      total: 1,
      message: "Restarting active application processes...",
    });
    try {
      await api.restartAll();
      setBatchProgress({
        active: true,
        type: "restart",
        step: 1,
        total: 1,
        completed: true,
        message: "All targets restarted cleanly",
      });
      setTimeout(() => {
        setBatchProgress((prev) => (prev?.completed ? null : prev));
      }, 2000);
    } catch (e) {
      console.error("Restart all error:", e);
      setBatchProgress(null);
    }
  };

  const handleStopAll = async () => {
    if (isStoppingAll || isStartingAll) return;
    setIsStoppingAll(true);
    try {
      const runningPanes = openPanes.filter(
        (p) => !p.isCombined && (p.status === "running" || p.status === "building")
      );

      if (runningPanes.length === 0) return;

      setBatchProgress({
        active: true,
        type: "stop",
        step: 0,
        total: runningPanes.length,
        targetName: runningPanes[0]?.title,
      });

      for (let i = 0; i < runningPanes.length; i++) {
        const p = runningPanes[i];
        setBatchProgress({
          active: true,
          type: "stop",
          step: i + 1,
          total: runningPanes.length,
          targetName: p.title,
        });

        updatePaneStatus(p.targetId, "idle");
        appendLogToPane(p.id, {
          timestamp: new Date().toISOString(),
          level: "I",
          tag: "client",
          message: `■ Stopping target '${p.title}'...`,
        });
        await api.stopTarget(p.targetId);
      }

      setBatchProgress({
        active: true,
        type: "stop",
        step: runningPanes.length,
        total: runningPanes.length,
        completed: true,
        message: `Stopped ${runningPanes.length} target process${
          runningPanes.length === 1 ? "" : "es"
        }`,
      });

      setTimeout(() => {
        setBatchProgress((prev) => (prev?.completed ? null : prev));
      }, 2500);
    } catch (e) {
      console.error("Stop all error:", e);
    } finally {
      setIsStoppingAll(false);
    }
  };

  const handleRefreshDoctor = async () => {
    setIsDoctorLoading(true);
    try {
      const report = await api.runDoctor(workspacePath);
      setDoctorReport(report);
    } catch (e) {
      console.error("Doctor error:", e);
    } finally {
      setIsDoctorLoading(false);
    }
  };

  const handleClosePane = (paneId: string) => {
    setOpenPanes((prev) => {
      const filtered = prev.filter((p) => p.id !== paneId);
      if (activePaneId === paneId && filtered.length > 0) {
        setActivePaneId(filtered[0].id);
      }
      return filtered;
    });
  };

  const handleClearActiveLogs = () => {
    if (activePaneId) {
      setLogsByPaneId((prev) => ({
        ...prev,
        [activePaneId]: [],
      }));
    }
  };

  const handleToggleMaximize = () => {
    setMaximizedPaneId((prev) => (prev ? null : activePaneId));
  };

  const handleBootEmulator = async (name: string) => {
    try {
      const res = await api.bootEmulator(name);
      if (res.success) {
        alert(`✓ ${res.message || `Booting emulator '${name}'...`}`);
        const devs = await api.fetchDevices();
        setDevices(devs);
      } else {
        alert(`✗ ${res.error || "Failed to boot emulator."}`);
      }
    } catch (e) {
      console.error("Boot emulator error:", e);
    }
  };

  const runningTargetsCount = openPanes.filter(
    (p) => !p.isCombined && (p.status === "running" || p.status === "building")
  ).length;

  const allTargetsRunning =
    targets.length > 0 &&
    targets.every((t) => {
      const pane = openPanes.find((p) => p.targetId === t.id);
      return pane?.status === "running" || pane?.status === "building";
    });

  const anyTargetRunning = runningTargetsCount > 0 || activeSessions.length > 0;

  return (
    <div className="app-container">
      {/* 1. Global Top Bar */}
      <TopBar
        workspaceName={workspaceName}
        activeSection={activeSection}
        isPrimarySidebarCollapsed={isPrimarySidebarCollapsed}
        isSecondarySidebarCollapsed={isSecondarySidebarCollapsed}
        searchQuery={searchQuery}
        levelFilter={levelFilter}
        targetsCount={targets.length}
        runningTargetsCount={runningTargetsCount}
        allTargetsRunning={allTargetsRunning}
        anyTargetRunning={anyTargetRunning}
        isStartingAll={isStartingAll}
        isStoppingAll={isStoppingAll}
        onChangeSearch={setSearchQuery}
        onSelectLevel={setLevelFilter}
        onTogglePrimarySidebar={() => setIsPrimarySidebarCollapsed(!isPrimarySidebarCollapsed)}
        onToggleSecondarySidebar={() => setIsSecondarySidebarCollapsed(!isSecondarySidebarCollapsed)}
        onSelectSection={setActiveSection}
        onRunAll={handleRunAll}
        onReloadAll={handleReloadAll}
        onRestartAll={handleRestartAll}
        onStopAll={handleStopAll}
        onOpenDevOptions={() => setDevOptionsModal({ isOpen: true })}
        updateAvailable={updateInfo}
        onOpenUpdateModal={() => setIsUpdateModalOpen(true)}
      />

      <div className="main-body">
        {/* 2. Primary Sidebar (Workspaces Rail / Drawer) */}
        <PrimarySidebar
          activeWorkspacePath={workspacePath}
          knownWorkspaces={knownWorkspaces}
          activeSection={activeSection}
          devices={devices}
          isCollapsed={isPrimarySidebarCollapsed}
          onToggleCollapse={() => setIsPrimarySidebarCollapsed(!isPrimarySidebarCollapsed)}
          onSelectWorkspace={handleSwitchWorkspace}
          onAddWorkspace={() => setIsWorkspaceModalOpen(true)}
          onSelectSection={setActiveSection}
          onRenameWorkspace={handleRenameWorkspace}
          onRemoveWorkspace={handleRemoveWorkspace}
        />

        {/* 3. Secondary Sidebar (Active Workspace Sub-Navigation) */}
        <SecondarySidebar
          workspaceName={workspaceName}
          workspacePath={workspacePath}
          activeSection={activeSection}
          targets={targets}
          openPanes={openPanes}
          activeSessions={activeSessions}
          isCollapsed={isSecondarySidebarCollapsed}
          onToggleCollapse={() => setIsSecondarySidebarCollapsed(!isSecondarySidebarCollapsed)}
          onSelectSection={setActiveSection}
          onOpenTargetPane={openTargetPane}
          onOpenCombinedPane={openCombinedPane}
        />

        {/* 4. Main Viewport with Persistent DOM Views (Zero Blinking, Instant Switching) */}
        <main className="viewport">
          {/* Terminal Mode with Secondary Top Bar */}
          <div
            className={`viewport-view-container ${activeSection === "terminal" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "terminal" ? "flex" : "none",
              flexDirection: "column",
              height: "100%",
              width: "100%",
            }}
          >
            <TerminalSecondaryBar
              panes={openPanes}
              activePaneId={activePaneId}
              layoutMode={layoutMode}
              availableTargets={targets}
              isMaximized={!!maximizedPaneId}
              isLogTreeOpen={isLogTreeOpen}
              levelFilter={levelFilter}
              searchQuery={searchQuery}
              onSelectTab={setActivePaneId}
              onClosePane={handleClosePane}
              onOpenTargetPane={openTargetPane}
              onOpenCombinedPane={openCombinedPane}
              onChangeLayout={setLayoutMode}
              onToggleMaximize={handleToggleMaximize}
              onToggleLogTree={() => setIsLogTreeOpen((prev) => !prev)}
              onClearActiveLogs={handleClearActiveLogs}
              onSelectLevel={setLevelFilter}
              onChangeSearch={setSearchQuery}
              onToggleRunTarget={handleToggleRun}
              onReloadTarget={handleReload}
              onRestartTarget={handleRestart}
            />

            <div className="terminal-workspace-area">
              <div className="terminal-workspace-split">
                {isLogTreeOpen && (
                  <TerminalLogTree
                    logs={logsByPaneId[activePaneId] || []}
                    activeLevelFilter={levelFilter}
                    activeTagFilter={tagFilter}
                    onSelectLevelFilter={setLevelFilter}
                    onSelectTagFilter={setTagFilter}
                  />
                )}
                <div className="terminal-grid-container">
                  <TerminalGrid
                    panes={openPanes}
                    activePaneId={activePaneId}
                    maximizedPaneId={maximizedPaneId}
                    logsByPaneId={logsByPaneId}
                    layoutMode={layoutMode}
                    levelFilter={levelFilter}
                    tagFilter={tagFilter}
                    searchQuery={searchQuery}
                    onFocusPane={setActivePaneId}
                    onToggleRun={handleToggleRun}
                    onReload={handleReload}
                    onRestart={handleRestart}
                    onSplitRight={() => setLayoutMode("split-h")}
                    onSplitDown={() => setLayoutMode("split-v")}
                    onToggleMaximize={(id) => setMaximizedPaneId(maximizedPaneId === id ? null : id)}
                    onClearLogs={(id) => setLogsByPaneId((prev) => ({ ...prev, [id]: [] }))}
                    onClosePane={handleClosePane}
                    onOpenAllPanes={() => {}}
                    onOpenCombinedPane={openCombinedPane}
                  />
                </div>
              </div>
            </div>
          </div>

          {/* Overview Mode */}
          <div
            className={`viewport-view-container ${activeSection === "overview" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "overview" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <WorkspaceOverviewView
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              devices={devices}
              isStartingAll={isStartingAll}
              isStoppingAll={isStoppingAll}
              allTargetsRunning={allTargetsRunning}
              anyTargetRunning={anyTargetRunning}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onToggleRunTarget={handleToggleRun}
              onReloadTarget={handleReload}
              onRestartTarget={handleRestart}
              onOpenDevOptions={(id) => setDevOptionsModal({ isOpen: true, targetId: id })}
              onSwitchToTerminal={() => setActiveSection("terminal")}
              onSwitchToTargets={() => setActiveSection("targets")}
            />
          </div>

          {/* Targets & Process Matrix Mode */}
          <div
            className={`viewport-view-container ${activeSection === "targets" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "targets" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <TargetsView
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              isStartingAll={isStartingAll}
              isStoppingAll={isStoppingAll}
              allTargetsRunning={allTargetsRunning}
              anyTargetRunning={anyTargetRunning}
              onToggleRun={handleToggleRun}
              onReload={handleReload}
              onRestart={handleRestart}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onOpenDevOptions={(id) => setDevOptionsModal({ isOpen: true, targetId: id })}
              onSwitchToTerminal={(paneId) => {
                if (paneId) setActivePaneId(paneId);
                setActiveSection("terminal");
              }}
            />
          </div>

          {/* Devices & Emulators Hub */}
          <div
            className={`viewport-view-container ${activeSection === "devices" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "devices" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <DevicesView
              devices={devices}
              onRefreshDevices={async () => {
                const devs = await api.fetchDevices();
                setDevices(devs);
              }}
              onBootEmulator={handleBootEmulator}
            />
          </div>

          {/* Doctor Diagnostics Hub */}
          <div
            className={`viewport-view-container ${activeSection === "doctor" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "doctor" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <DoctorView
              report={doctorReport}
              isLoading={isDoctorLoading}
              onRefreshDoctor={handleRefreshDoctor}
            />
          </div>

          {/* Model Context Protocol (MCP) Hub Tab */}
          <div
            className={`viewport-view-container ${activeSection === "mcp" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "mcp" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <McpView
              onFetchStatus={api.fetchMcpStatus}
              onToggleServer={api.toggleMcpServer}
              onFetchLogs={api.fetchMcpLogs}
              onFetchSessions={api.fetchMcpSessions}
              onFetchAgents={api.fetchMcpAgents}
              onDeleteLog={api.deleteMcpLog}
              onClearLogs={api.clearMcpLogs}
            />
          </div>

          {/* Settings & System Environment Tab */}
          <div
            className={`viewport-view-container ${activeSection === "settings" ? "active" : "hidden"}`}
            style={{
              display: activeSection === "settings" ? "block" : "none",
              height: "100%",
              width: "100%",
              overflowY: "auto",
            }}
          >
            <SettingsView
              onInstallCli={async () => {
                const res = await api.installShellCli();
                if (!res.success) throw new Error(res.error);
              }}
              onUninstallCli={async () => {
                const res = await api.uninstallShellCli();
                if (!res.success) throw new Error(res.error);
              }}
              onNavigateToMcp={() => setActiveSection("mcp")}
              workspacePath={workspacePath}
              updateInfo={updateInfo}
              isCheckingUpdate={isCheckingUpdate}
              onCheckUpdates={handleCheckUpdates}
              onOpenUpdateModal={() => setIsUpdateModalOpen(true)}
            />
          </div>
        </main>
      </div>

      {/* 5. Process Lifecycle Status Toast Notification */}
      <ProcessStatusToast
        progress={batchProgress}
        onDismiss={() => setBatchProgress(null)}
      />

      {/* 6. Dev Server & Runner Options Modal */}
      <DevServerOptionsModal
        isOpen={devOptionsModal.isOpen}
        onClose={() => setDevOptionsModal({ isOpen: false })}
        targets={targets}
        initialTargetId={devOptionsModal.targetId}
        onLaunchWithOptions={handleLaunchWithOptions}
        onStopTarget={async (id) => {
          await api.stopTarget(id);
        }}
      />

      {/* 7. Workspace Add / Switch Modal */}
      <WorkspaceModal
        isOpen={isWorkspaceModalOpen}
        onClose={() => setIsWorkspaceModalOpen(false)}
        currentPath={workspacePath}
        knownWorkspaces={knownWorkspaces}
        onRemoveWorkspace={handleRemoveWorkspace}
        onSelectWorkspace={(path, immediateData) => {
          handleSwitchWorkspace(path, immediateData);
          setIsWorkspaceModalOpen(false);
        }}
      />

      {/* 8. Integrated Software Update Modal */}
      <UpdateModal
        isOpen={isUpdateModalOpen}
        onClose={() => setIsUpdateModalOpen(false)}
        updateInfo={updateInfo}
        onInstallUpdate={handleInstallUpdate}
        onRestartApp={handleRestartApp}
      />
    </div>
  );
};


export default App;
