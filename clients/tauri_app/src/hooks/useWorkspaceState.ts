import { useState, useEffect, useCallback, useRef } from "react";
import { useDevFlowApi } from "./useDevFlowApi";
import type {
  ProjectTarget,
  Device,
  ActiveSessionInfo,
  KnownWorkspace,
  WorkspaceResponse,
  PaneInfo,
  LogEntry,
  ViewSection,
} from "../types";

export interface WorkspaceCacheItem {
  targets: ProjectTarget[];
  openPanes: PaneInfo[];
  activePaneId: string;
  logsByPaneId: Record<string, LogEntry[]>;
  activeSection: ViewSection;
}

interface UseWorkspaceStateProps {
  openPanes: PaneInfo[];
  activePaneId: string;
  logsByPaneId: Record<string, LogEntry[]>;
  activeSection: ViewSection;
  setOpenPanes: React.Dispatch<React.SetStateAction<PaneInfo[]>>;
  setActivePaneId: (id: string) => void;
  setLogsByPaneId: React.Dispatch<React.SetStateAction<Record<string, LogEntry[]>>>;
}

export function useWorkspaceState({
  openPanes,
  activePaneId,
  logsByPaneId,
  activeSection,
  setOpenPanes,
  setActivePaneId,
  setLogsByPaneId,
}: UseWorkspaceStateProps) {
  const api = useDevFlowApi();

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
  const [isWorkspaceModalOpen, setIsWorkspaceModalOpen] = useState<boolean>(false);

  // In-Memory Stateful Workspace Cache (Zero-flicker 0ms switching)
  const workspaceCacheRef = useRef<Record<string, WorkspaceCacheItem>>({});

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
  }, [api, customNames]);

  useEffect(() => {
    loadKnownWorkspaces();
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
    [customNames, setActivePaneId, setLogsByPaneId, setOpenPanes]
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
      setActivePaneId,
      setLogsByPaneId,
      setOpenPanes,
    ]
  );

  // Rename Workspace Alias
  const handleRenameWorkspace = useCallback(
    (path: string, newName: string) => {
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
    },
    [workspacePath]
  );

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

  return {
    workspaceName,
    workspacePath,
    knownWorkspaces,
    customNames,
    targets,
    devices,
    activeSessions,
    isWorkspaceModalOpen,
    setIsWorkspaceModalOpen,
    setTargets,
    setDevices,
    setActiveSessions,
    loadKnownWorkspaces,
    applyWorkspaceData,
    loadWorkspace,
    handleSwitchWorkspace,
    handleRenameWorkspace,
    handleRemoveWorkspace,
  };
}
