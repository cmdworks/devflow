import React, { useState, useEffect, useCallback } from "react";
import { TopBar } from "./components/TopBar";
import { Sidebar } from "./components/Sidebar";
import { TerminalGrid } from "./components/TerminalGrid";
import { DoctorModal } from "./components/DoctorModal";
import { WorkspaceModal } from "./components/WorkspaceModal";
import { WorkspaceTabBar, type LayoutMode } from "./components/WorkspaceTabBar";
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
} from "./types";

export const App: React.FC = () => {
  const api = useDevFlowApi();

  const [workspaceName, setWorkspaceName] = useState<string>("Workspace");
  const [workspacePath, setWorkspacePath] = useState<string>("");
  const [targets, setTargets] = useState<ProjectTarget[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [activeSessions, setActiveSessions] = useState<ActiveSessionInfo[]>([]);

  const [openPanes, setOpenPanes] = useState<PaneInfo[]>([]);
  const [logsByPaneId, setLogsByPaneId] = useState<Record<string, LogEntry[]>>({});

  const [layoutMode, setLayoutMode] = useState<LayoutMode>(() => {
    return (localStorage.getItem("devflow_layout_mode") as LayoutMode) || "tabs";
  });
  const [activePaneId, setActivePaneId] = useState<string>("");
  const [maximizedPaneId, setMaximizedPaneId] = useState<string | null>(null);

  useEffect(() => {
    localStorage.setItem("devflow_layout_mode", layoutMode);
  }, [layoutMode]);

  const [isDoctorOpen, setIsDoctorOpen] = useState<boolean>(false);
  const [doctorReport, setDoctorReport] = useState<DoctorReport | null>(null);
  const [isDoctorLoading, setIsDoctorLoading] = useState<boolean>(false);

  const [isWorkspaceModalOpen, setIsWorkspaceModalOpen] = useState<boolean>(false);

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

  // Load workspace data
  const loadWorkspace = useCallback(
    async (dirPath?: string) => {
      try {
        const data = await api.fetchWorkspace(dirPath);
        setWorkspaceName(data.workspace_name);
        setWorkspacePath(data.workspace_path);
        setTargets(data.targets || []);
        setDevices(data.devices || []);
        setActiveSessions(data.active_sessions || []);

        // Open separate tab for each discovered project target + Combined Stream tab
        if (data.targets && data.targets.length > 0) {
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
          setActivePaneId((prev) => {
            // If the previous tab was a target from the old workspace, focus the first new target
            if (prev && prev !== "pane-combined" && initialPanes.some((p) => p.id === prev)) {
              return prev;
            }
            return initialPanes[0]?.id || "pane-combined";
          });
        }
      } catch (err) {
        console.error("Failed to load workspace:", err);
      }
    },
    [api]
  );

  // Initial load
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const dir = params.get("dir") || "";
    loadWorkspace(dir);
  }, [loadWorkspace]);

  // Switch workspace path
  const handleSwitchWorkspace = (newPath: string) => {
    setOpenPanes([]);
    setLogsByPaneId({});
    try {
      const url = new URL(window.location.href);
      url.searchParams.set("dir", newPath);
      window.history.pushState({}, "", url.toString());
    } catch (_) {}
    loadWorkspace(newPath);
  };

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

  const handleOpenDoctor = async () => {
    setIsDoctorOpen(true);
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

  const handleInstallCli = async () => {
    const res = await api.installShellCli();
    if (res.success) {
      alert(`✓ ${res.message || "Successfully installed 'devflow' command symlink!"}`);
    } else {
      alert(`✗ ${res.error || "Failed to install symlink."}`);
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
    if (maximizedPaneId === paneId) {
      setMaximizedPaneId(null);
    }
  };

  const handleClearLogs = (paneId: string) => {
    setLogsByPaneId((prev) => ({
      ...prev,
      [paneId]: [],
    }));
  };

  const handleOpenAllPanes = () => {
    for (const t of targets) {
      openTargetPane(t);
    }
  };

  return (
    <div className="app-container">
      <TopBar
        workspaceName={workspaceName}
        workspacePath={workspacePath}
        devices={devices}
        onOpenWorkspaceManager={() => setIsWorkspaceModalOpen(true)}
        onRunAll={handleRunAll}
        onReloadAll={handleReloadAll}
        onRestartAll={handleRestartAll}
        onStopAll={handleStopAll}
        onOpenDoctor={handleOpenDoctor}
        onInstallCli={handleInstallCli}
      />

      <div className="main-body">
        <Sidebar
          targets={targets}
          devices={devices}
          activeSessions={activeSessions}
          openPanes={openPanes}
          onOpenWorkspaceManager={() => setIsWorkspaceModalOpen(true)}
          onOpenTargetPane={openTargetPane}
          onOpenCombinedPane={openCombinedPane}
          onRefreshDevices={async () => {
            const devs = await api.fetchDevices();
            setDevices(devs);
          }}
          onBootEmulator={async (name) => {
            const res = await api.bootEmulator(name);
            if (res.success) {
              alert(`✓ ${res.message || "Emulator booted."}`);
            } else {
              alert(`✗ ${res.error || "Failed to boot emulator."}`);
            }
          }}
        />

        <main className="viewport">
          <WorkspaceTabBar
            panes={openPanes}
            activePaneId={activePaneId}
            layoutMode={layoutMode}
            availableTargets={targets}
            onSelectTab={(id) => {
              setActivePaneId(id);
              setMaximizedPaneId(null);
            }}
            onCloseTab={handleClosePane}
            onChangeLayoutMode={(mode) => {
              setLayoutMode(mode);
              setMaximizedPaneId(null);
            }}
            onOpenTarget={openTargetPane}
            onOpenCombinedStream={openCombinedPane}
          />

          <TerminalGrid
            panes={openPanes}
            activePaneId={activePaneId}
            maximizedPaneId={maximizedPaneId}
            layoutMode={layoutMode}
            logsByPaneId={logsByPaneId}
            onFocusPane={(id) => setActivePaneId(id)}
            onToggleRun={handleToggleRun}
            onReload={handleReload}
            onRestart={handleRestart}
            onSplitRight={(id) => {
              setActivePaneId(id);
              setLayoutMode("split-h");
              setMaximizedPaneId(null);
            }}
            onSplitDown={(id) => {
              setActivePaneId(id);
              setLayoutMode("split-v");
              setMaximizedPaneId(null);
            }}
            onToggleMaximize={(id) => {
              setMaximizedPaneId((prev) => (prev === id ? null : id));
            }}
            onClearLogs={handleClearLogs}
            onClosePane={handleClosePane}
            onOpenAllPanes={handleOpenAllPanes}
            onOpenCombinedPane={openCombinedPane}
          />
        </main>
      </div>

      {isDoctorOpen && (
        <DoctorModal
          report={doctorReport}
          isLoading={isDoctorLoading}
          onClose={() => setIsDoctorOpen(false)}
        />
      )}

      {isWorkspaceModalOpen && (
        <WorkspaceModal
          currentPath={workspacePath}
          isOpen={isWorkspaceModalOpen}
          onClose={() => setIsWorkspaceModalOpen(false)}
          onSelectWorkspace={handleSwitchWorkspace}
        />
      )}
    </div>
  );
};
export default App;
