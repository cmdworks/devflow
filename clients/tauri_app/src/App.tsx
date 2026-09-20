import React, { useState, useEffect, useCallback, useRef } from "react";
import { TopBar } from "./components/TopBar";
import { PrimarySidebar } from "./components/PrimarySidebar";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { TerminalSecondaryBar, type LayoutMode } from "./components/TerminalSecondaryBar";
import { TerminalGrid } from "./components/TerminalGrid";
import { WorkspaceOverviewView } from "./components/WorkspaceOverviewView";
import { TargetsView } from "./components/TargetsView";
import { DevicesView } from "./components/DevicesView";
import { DoctorView } from "./components/DoctorView";
import { WorkspaceModal } from "./components/WorkspaceModal";
import { SettingsModal } from "./components/SettingsModal";
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

  // Navigation View State
  const [activeSection, setActiveSection] = useState<ViewSection>("terminal");
  const [isSubsidebarCollapsed, setIsSubsidebarCollapsed] = useState<boolean>(false);

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
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState<boolean>(false);

  // In-Memory Stateful Workspace Cache (Zero-flicker 0ms switching)
  const workspaceCacheRef = useRef<Record<string, WorkspaceCacheItem>>({});

  useEffect(() => {
    localStorage.setItem("devflow_layout_mode", layoutMode);
  }, [layoutMode]);

  useEffect(() => {
    localStorage.setItem("devflow_custom_workspace_names", JSON.stringify(customNames));
  }, [customNames]);

  // Load known workspaces once at startup
  const loadKnownWorkspaces = useCallback(async () => {
    try {
      const list = await api.fetchWorkspaces();
      if (list && Array.isArray(list)) {
        const enriched = list.map((w) => ({
          ...w,
          custom_name: customNames[w.path] || w.name,
        }));
        setKnownWorkspaces(enriched);
      }
    } catch (err) {
      console.error("Failed to load known workspaces:", err);
    }
  }, [api, customNames]);

  useEffect(() => {
    loadKnownWorkspaces();
  }, [loadKnownWorkspaces]);

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

  // Load workspace data with caching
  const loadWorkspace = useCallback(
    async (dirPath?: string) => {
      try {
        const data = await api.fetchWorkspace(dirPath);
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

        // Initialize target tabs if no panes exist yet
        if (data.targets && data.targets.length > 0 && openPanes.length === 0) {
          const targetPanes: PaneInfo[] = data.targets.map((t) => ({
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

          const initialPanes = [...targetPanes, combinedPane];
          setOpenPanes(initialPanes);
          setActivePaneId(initialPanes[0]?.id || "pane-combined");
        }
      } catch (err) {
        console.error("Failed to load workspace:", err);
      }
    },
    [api, customNames, openPanes.length]
  );

  // Initial load from URL query
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const dir = params.get("dir") || "";
    loadWorkspace(dir);
  }, [loadWorkspace]);

  // Stateful Zero-Flicker Workspace Switcher
  const handleSwitchWorkspace = useCallback(
    (newPath: string) => {
      // 1. Cache current workspace state in memory
      if (workspacePath) {
        workspaceCacheRef.current[workspacePath] = {
          targets,
          openPanes,
          activePaneId,
          logsByPaneId,
          activeSection,
        };
      }

      // 2. Check if destination workspace is cached in memory
      const cached = workspaceCacheRef.current[newPath];
      if (cached) {
        setTargets(cached.targets);
        setOpenPanes(cached.openPanes);
        setActivePaneId(cached.activePaneId);
        setLogsByPaneId(cached.logsByPaneId);
        setActiveSection(cached.activeSection || "terminal");
      }

      // 3. Update URL without page reload
      try {
        const url = new URL(window.location.href);
        url.searchParams.set("dir", newPath);
        window.history.pushState({}, "", url.toString());
      } catch (_) {}

      // 4. Background refresh workspace targets and devices
      loadWorkspace(newPath);
    },
    [workspacePath, targets, openPanes, activePaneId, logsByPaneId, activeSection, loadWorkspace]
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
        setKnownWorkspaces((prev) => prev.filter((w) => w.path !== path));
      } catch (err) {
        console.error("Failed to remove workspace:", err);
      }
    },
    [api]
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

  const handleRunAll = () => {
    for (const t of targets) {
      openTargetPane(t);
      handleToggleRun(t.id);
    }
  };

  const handleReloadAll = async () => {
    await api.reloadAll();
  };

  const handleRestartAll = async () => {
    await api.restartAll();
  };

  const handleStopAll = async () => {
    for (const p of openPanes) {
      if (!p.isCombined) {
        updatePaneStatus(p.targetId, "idle");
        await api.stopTarget(p.targetId);
      }
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

  return (
    <div className="app-container">
      {/* 1. Global Top Bar */}
      <TopBar
        workspaceName={workspaceName}
        activeSection={activeSection}
        devices={devices}
        onSelectSection={setActiveSection}
        onRunAll={handleRunAll}
        onReloadAll={handleReloadAll}
        onRestartAll={handleRestartAll}
        onStopAll={handleStopAll}
        onOpenSettings={() => setIsSettingsModalOpen(true)}
      />

      <div className="main-body">
        {/* 2. Primary Sidebar (Workspaces + Global Tools) */}
        <PrimarySidebar
          activeWorkspacePath={workspacePath}
          knownWorkspaces={knownWorkspaces}
          activeSection={activeSection}
          devices={devices}
          onSelectWorkspace={handleSwitchWorkspace}
          onAddWorkspace={() => setIsWorkspaceModalOpen(true)}
          onSelectSection={setActiveSection}
          onRenameWorkspace={handleRenameWorkspace}
          onRemoveWorkspace={handleRemoveWorkspace}
          onOpenSettings={() => setIsSettingsModalOpen(true)}
        />

        {/* 3. Secondary Sidebar (Active Workspace Sub-Navigation) */}
        <SecondarySidebar
          workspaceName={workspaceName}
          workspacePath={workspacePath}
          activeSection={activeSection}
          targets={targets}
          openPanes={openPanes}
          activeSessions={activeSessions}
          isCollapsed={isSubsidebarCollapsed}
          onToggleCollapse={() => setIsSubsidebarCollapsed(!isSubsidebarCollapsed)}
          onSelectSection={setActiveSection}
          onOpenTargetPane={openTargetPane}
          onOpenCombinedPane={openCombinedPane}
        />

        {/* 4. Main Viewport */}
        <main className="viewport">
          {/* Terminal Mode with Secondary Top Bar */}
          {activeSection === "terminal" && (
            <>
              <TerminalSecondaryBar
                panes={openPanes}
                activePaneId={activePaneId}
                layoutMode={layoutMode}
                availableTargets={targets}
                isMaximized={!!maximizedPaneId}
                onSelectTab={setActivePaneId}
                onClosePane={handleClosePane}
                onOpenTargetPane={openTargetPane}
                onOpenCombinedPane={openCombinedPane}
                onChangeLayout={setLayoutMode}
                onToggleMaximize={handleToggleMaximize}
                onClearActiveLogs={handleClearActiveLogs}
              />

              <div className="terminal-workspace-area">
                <TerminalGrid
                  panes={openPanes}
                  activePaneId={activePaneId}
                  maximizedPaneId={maximizedPaneId}
                  logsByPaneId={logsByPaneId}
                  layoutMode={layoutMode}
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
            </>
          )}

          {/* Overview Mode */}
          {activeSection === "overview" && (
            <WorkspaceOverviewView
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              devices={devices}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onToggleRunTarget={handleToggleRun}
              onReloadTarget={handleReload}
              onRestartTarget={handleRestart}
              onSwitchToTerminal={() => setActiveSection("terminal")}
            />
          )}

          {/* Targets & Process Matrix Mode */}
          {activeSection === "targets" && (
            <TargetsView
              targets={targets}
              openPanes={openPanes}
              activeSessions={activeSessions}
              workspaceName={workspaceName}
              workspacePath={workspacePath}
              onToggleRun={handleToggleRun}
              onReload={handleReload}
              onRestart={handleRestart}
              onOpenTargetPane={openTargetPane}
              onRunAll={handleRunAll}
              onReloadAll={handleReloadAll}
              onRestartAll={handleRestartAll}
              onStopAll={handleStopAll}
              onSwitchToTerminal={(paneId) => {
                if (paneId) setActivePaneId(paneId);
                setActiveSection("terminal");
              }}
            />
          )}

          {/* Devices & Emulators Hub */}
          {activeSection === "devices" && (
            <DevicesView
              devices={devices}
              onRefreshDevices={async () => {
                const devs = await api.fetchDevices();
                setDevices(devs);
              }}
              onBootEmulator={handleBootEmulator}
            />
          )}

          {/* Doctor Diagnostics Hub */}
          {activeSection === "doctor" && (
            <DoctorView
              report={doctorReport}
              isLoading={isDoctorLoading}
              onRefreshDoctor={handleRefreshDoctor}
            />
          )}
        </main>
      </div>

      {/* 5. Workspace Add / Switch Modal */}
      <WorkspaceModal
        isOpen={isWorkspaceModalOpen}
        onClose={() => setIsWorkspaceModalOpen(false)}
        currentPath={workspacePath}
        onSelectWorkspace={(path) => {
          handleSwitchWorkspace(path);
          setIsWorkspaceModalOpen(false);
        }}
      />

      {/* 6. Settings Modal */}
      <SettingsModal
        isOpen={isSettingsModalOpen}
        onClose={() => setIsSettingsModalOpen(false)}
        onInstallCli={async () => {
          const res = await api.installShellCli();
          if (!res.success) throw new Error(res.error);
        }}
        onUninstallCli={async () => {
          const res = await api.uninstallShellCli();
          if (!res.success) throw new Error(res.error);
        }}
      />
    </div>
  );
};

export default App;
