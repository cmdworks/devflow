import { useCallback } from "react";
import type { WorkspaceResponse, Device, DoctorReport } from "../types";

export function getApiBase(): string {
  if (typeof window !== "undefined" && (window.location.protocol === "http:" || window.location.protocol === "https:")) {
    return window.location.origin;
  }
  return "http://localhost:9292";
}

export function useDevFlowApi() {
  const apiBase = getApiBase();

  const fetchWorkspace = useCallback(
    async (dirPath?: string): Promise<WorkspaceResponse> => {
      const url = dirPath
        ? `${apiBase}/api/workspace?dir=${encodeURIComponent(dirPath)}`
        : `${apiBase}/api/workspace`;
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP error ${res.status}`);
      return await res.json();
    },
    [apiBase]
  );

  const fetchDevices = useCallback(async (): Promise<Device[]> => {
    const res = await fetch(`${apiBase}/api/devices`);
    if (!res.ok) throw new Error(`HTTP error ${res.status}`);
    return await res.json();
  }, [apiBase]);

  const bootEmulator = useCallback(
    async (name: string): Promise<{ success: boolean; message?: string; error?: string }> => {
      const res = await fetch(`${apiBase}/api/devices/boot`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const startTarget = useCallback(
    async (
      targetId: string,
      targetPath: string,
      framework: string,
      deviceId?: string
    ): Promise<{ success: boolean; error?: string; target_id?: string }> => {
      const res = await fetch(`${apiBase}/api/target/start`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          target_id: targetId,
          target_path: targetPath,
          framework,
          device_id: deviceId || null,
        }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const stopTarget = useCallback(
    async (targetId: string): Promise<{ success: boolean; message?: string }> => {
      const res = await fetch(`${apiBase}/api/target/stop`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ target_id: targetId }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const reloadTarget = useCallback(
    async (targetId: string): Promise<{ success: boolean; message?: string }> => {
      const res = await fetch(`${apiBase}/api/target/reload`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ target_id: targetId }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const restartTarget = useCallback(
    async (targetId: string): Promise<{ success: boolean; message?: string }> => {
      const res = await fetch(`${apiBase}/api/target/restart`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ target_id: targetId }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const reloadAll = useCallback(async (): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch(`${apiBase}/api/workspace/reload-all`, { method: "POST" });
    return await res.json();
  }, [apiBase]);

  const restartAll = useCallback(async (): Promise<{ success: boolean; message?: string }> => {
    const res = await fetch(`${apiBase}/api/workspace/restart-all`, { method: "POST" });
    return await res.json();
  }, [apiBase]);

  const runDoctor = useCallback(
    async (dirPath?: string): Promise<DoctorReport> => {
      const url = dirPath
        ? `${apiBase}/api/doctor?dir=${encodeURIComponent(dirPath)}`
        : `${apiBase}/api/doctor`;
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP error ${res.status}`);
      return await res.json();
    },
    [apiBase]
  );

  const installShellCli = useCallback(async (): Promise<{ success: boolean; message?: string; error?: string }> => {
    const res = await fetch(`${apiBase}/api/shell/install`, { method: "POST" });
    return await res.json();
  }, [apiBase]);

  const uninstallShellCli = useCallback(async (): Promise<{ success: boolean; message?: string; error?: string }> => {
    const res = await fetch(`${apiBase}/api/shell/uninstall`, { method: "POST" });
    return await res.json();
  }, [apiBase]);

  const fetchWorkspaces = useCallback(async (): Promise<import("../types").KnownWorkspace[]> => {
    const res = await fetch(`${apiBase}/api/workspaces`);
    if (!res.ok) throw new Error(`HTTP error ${res.status}`);
    return await res.json();
  }, [apiBase]);

  const addWorkspace = useCallback(
    async (path: string): Promise<WorkspaceResponse> => {
      const res = await fetch(`${apiBase}/api/workspaces`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      });
      if (!res.ok) {
        const errText = await res.text();
        throw new Error(errText || `HTTP error ${res.status}`);
      }
      return await res.json();
    },
    [apiBase]
  );

  const removeWorkspace = useCallback(
    async (path: string): Promise<{ success: boolean }> => {
      const res = await fetch(`${apiBase}/api/workspaces`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      });
      return await res.json();
    },
    [apiBase]
  );

  const pickFolder = useCallback(async (): Promise<{ success: boolean; path?: string }> => {
    try {
      const res = await fetch(`${apiBase}/api/dialog/pick-folder`, { method: "POST" });
      if (!res.ok) return { success: false };
      return await res.json();
    } catch (_) {
      return { success: false };
    }
  }, [apiBase]);

  return {
    fetchWorkspace,
    fetchWorkspaces,
    addWorkspace,
    removeWorkspace,
    pickFolder,
    fetchDevices,
    bootEmulator,
    startTarget,
    stopTarget,
    reloadTarget,
    restartTarget,
    reloadAll,
    restartAll,
    runDoctor,
    installShellCli,
    uninstallShellCli,
  };
}
