import { useState, useCallback } from "react";
import { useDevFlowApi } from "./useDevFlowApi";
import type { ProjectTarget, Device, PaneInfo, LogEntry, ViewSection, ActiveSessionInfo } from "../types";
import type { BatchProgressInfo } from "../components/ProcessStatusToast";
import type { RunnerOptionsConfig } from "../components/DevServerOptionsModal";
import type { TargetActionFeedback } from "../components/ActionStatusToast";

export type { TargetActionFeedback };

interface UseTargetOperationsProps {
  targets: ProjectTarget[];
  devices: Device[];
  openPanes: PaneInfo[];
  activeSessions: ActiveSessionInfo[];
  updatePaneStatus: (targetId: string, status: PaneInfo["status"]) => void;
  appendLogToPane: (paneId: string, entry: LogEntry) => void;
  openTargetPane: (target: ProjectTarget) => void;
  setActivePaneId: (paneId: string) => void;
  setActiveSection: (section: ViewSection) => void;
}

export function useTargetOperations({
  targets,
  devices,
  openPanes,
  activeSessions,
  updatePaneStatus,
  appendLogToPane,
  openTargetPane,
  setActivePaneId,
  setActiveSection,
}: UseTargetOperationsProps) {
  const api = useDevFlowApi();

  const [isStartingAll, setIsStartingAll] = useState<boolean>(false);
  const [isStoppingAll, setIsStoppingAll] = useState<boolean>(false);
  const [batchProgress, setBatchProgress] = useState<BatchProgressInfo | null>(null);
  const [actionFeedback, setActionFeedback] = useState<TargetActionFeedback | null>(null);
  const [executingActions, setExecutingActions] = useState<Record<string, string>>({});
  const [devOptionsModal, setDevOptionsModal] = useState<{ isOpen: boolean; targetId?: string }>({
    isOpen: false,
  });

  const dismissActionFeedback = useCallback(() => {
    setActionFeedback(null);
  }, []);

  // Run / Stop target toggle
  const handleToggleRun = useCallback(
    async (targetId: string) => {
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
    },
    [api, appendLogToPane, devices, openPanes, targets, updatePaneStatus]
  );

  const handleReload = useCallback(
    async (targetId: string) => {
      await api.reloadTarget(targetId);
    },
    [api]
  );

  const handleRestart = useCallback(
    async (targetId: string) => {
      await api.restartTarget(targetId);
    },
    [api]
  );

  const handleRunAll = useCallback(async () => {
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
  }, [
    api,
    appendLogToPane,
    devices,
    isStartingAll,
    isStoppingAll,
    openPanes,
    openTargetPane,
    targets,
    updatePaneStatus,
  ]);

  const handleLaunchWithOptions = useCallback(
    async (targetId: string, options: RunnerOptionsConfig) => {
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
    },
    [
      api,
      appendLogToPane,
      devices,
      handleRunAll,
      openTargetPane,
      setActivePaneId,
      setActiveSection,
      targets,
      updatePaneStatus,
    ]
  );

  const handleReloadAll = useCallback(async () => {
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
  }, [api]);

  const handleRestartAll = useCallback(async () => {
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
  }, [api]);

  const handleStopAll = useCallback(async () => {
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
  }, [api, appendLogToPane, isStartingAll, isStoppingAll, openPanes, updatePaneStatus]);

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

  const getActionTitle = (action: string, framework?: string): string => {
    switch (action) {
      case "launch_app":
        return framework?.toLowerCase().includes("android") ? "Launch on Device" : "Open App";
      case "reinstall_apk":
        return "Install APK";
      case "force_stop":
        return "Kill App Process";
      case "clear_data":
        return "Clear App Data";
      case "clean_cache":
      case "clean_build":
        return "Clean Cache";
      case "open_ide":
        return framework?.toLowerCase().includes("android")
          ? "Open in Android Studio"
          : framework?.toLowerCase().includes("swift")
          ? "Open in Xcode"
          : "Open in IDE";
      case "open_browser":
        return "Open in Browser";
      case "free_port":
        return "Free Port";
      case "clippy_scan":
        return "Cargo Clippy";
      case "reveal_finder":
        return "Reveal in Finder";
      default:
        return action.replace(/_/g, " ");
    }
  };

  const handleExecuteAction = useCallback(
    async (targetId: string, action: string, port?: number) => {
      const target = targets.find((t) => t.id === targetId);
      if (!target) return;

      const pane = openPanes.find((p) => p.targetId === targetId);
      const paneId = pane ? pane.id : `pane-${target.id}`;
      const actionTitle = getActionTitle(action, target.framework);
      const isAndroid = (target.framework || "").toLowerCase().includes("android") || (target.framework || "").toLowerCase().includes("kotlin");
      
      let pendingMessage = `Executing ${actionTitle}...`;
      if (action === "reinstall_apk") {
        pendingMessage = `Installing Android App (${target.name}) on device...`;
      } else if (action === "launch_app") {
        pendingMessage = isAndroid
          ? `Launching ${target.name} on Android device...`
          : `Opening ${target.name}...`;
      } else if (action === "force_stop") {
        pendingMessage = `Force stopping ${target.name}...`;
      } else if (action === "clear_data") {
        pendingMessage = `Clearing app data & cache for ${target.name}...`;
      } else if (action === "clean_cache" || action === "clean_build") {
        pendingMessage = `Cleaning build cache for ${target.name}...`;
      } else if (action === "open_ide") {
        pendingMessage = `Opening ${target.name} in IDE...`;
      }

      setExecutingActions((prev) => ({ ...prev, [targetId]: action }));
      setActionFeedback({
        id: `${targetId}-${action}-${Date.now()}`,
        targetId,
        targetName: target.name,
        framework: target.framework,
        action,
        actionTitle,
        status: "pending",
        message: pendingMessage,
        timestamp: Date.now(),
      });

      appendLogToPane(paneId, {
        timestamp: new Date().toISOString(),
        level: "I",
        tag: "action",
        message: `⚡ Executing action '${actionTitle}' for '${target.name}'...`,
      });

      const matchedDev = devices.find((d) => {
        if (target.platform.toLowerCase() === "android") return d.platform.toLowerCase() === "android";
        if (target.platform.toLowerCase() === "macos" || target.platform.toLowerCase() === "desktop") {
          return d.platform.toLowerCase() === "desktop";
        }
        return true;
      });

      try {
        const res = await api.executeTargetAction(
          target.id,
          target.path,
          target.framework,
          action,
          matchedDev?.id,
          port
        );

        if (res.success) {
          const successMsg = res.message || `${actionTitle} completed successfully.`;
          setActionFeedback({
            id: `${targetId}-${action}-${Date.now()}`,
            targetId,
            targetName: target.name,
            framework: target.framework,
            action,
            actionTitle,
            status: "success",
            message: successMsg,
            timestamp: Date.now(),
          });
          appendLogToPane(paneId, {
            timestamp: new Date().toISOString(),
            level: "I",
            tag: "action",
            message: `✓ ${successMsg}`,
          });
        } else {
          const errorMsg = res.error || "Action failed";
          setActionFeedback({
            id: `${targetId}-${action}-${Date.now()}`,
            targetId,
            targetName: target.name,
            framework: target.framework,
            action,
            actionTitle,
            status: "error",
            message: errorMsg,
            timestamp: Date.now(),
          });
          appendLogToPane(paneId, {
            timestamp: new Date().toISOString(),
            level: "E",
            tag: "action",
            message: `✗ ${actionTitle} failed: ${errorMsg}`,
          });
        }
      } catch (err: any) {
        const errMsg = err?.message || String(err);
        setActionFeedback({
          id: `${targetId}-${action}-${Date.now()}`,
          targetId,
          targetName: target.name,
          framework: target.framework,
          action,
          actionTitle,
          status: "error",
          message: errMsg,
          timestamp: Date.now(),
        });
        appendLogToPane(paneId, {
          timestamp: new Date().toISOString(),
          level: "E",
          tag: "action",
          message: `✗ ${actionTitle} failed: ${errMsg}`,
        });
      } finally {
        setExecutingActions((prev) => {
          const next = { ...prev };
          delete next[targetId];
          return next;
        });
      }
    },
    [api, appendLogToPane, devices, openPanes, targets]
  );

  return {
    isStartingAll,
    isStoppingAll,
    batchProgress,
    setBatchProgress,
    actionFeedback,
    dismissActionFeedback,
    executingActions,
    devOptionsModal,
    setDevOptionsModal,
    handleToggleRun,
    handleReload,
    handleRestart,
    handleLaunchWithOptions,
    handleExecuteAction,
    handleRunAll,
    handleReloadAll,
    handleRestartAll,
    handleStopAll,
    runningTargetsCount,
    allTargetsRunning,
    anyTargetRunning,
  };
}
