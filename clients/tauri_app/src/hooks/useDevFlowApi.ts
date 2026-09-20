import { useCallback, useMemo } from "react";
import type { WorkspaceResponse, Device, DoctorReport } from "../types";

export interface FetchMcpLogsParams {
  limit?: number;
  offset?: number;
  session_id?: string;
  client?: string;
  tool_name?: string;
  status?: string;
  search?: string;
}

export function getApiBase(): string {
  if (typeof window !== "undefined") {
    try {
      const params = new URLSearchParams(window.location.search);
      const queryPort = params.get("port");
      if (queryPort) {
        return `http://localhost:${queryPort}`;
      }
    } catch (_) {}

    if (
      (window.location.protocol === "http:" || window.location.protocol === "https:") &&
      window.location.port
    ) {
      return window.location.origin;
    }
  }
  return "http://localhost:9292";
}

async function parseJsonResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    let detail = "";
    try {
      const data = await res.json();
      if (data?.error) detail = `: ${data.error}`;
      else if (data?.message) detail = `: ${data.message}`;
    } catch (_) {
      try {
        const text = await res.text();
        if (text && text.length < 80) detail = `: ${text.trim()}`;
      } catch (_) {}
    }
    throw new Error(`HTTP error ${res.status}${detail}`);
  }

  const contentType = res.headers.get("content-type") || "";
  if (!contentType.includes("application/json")) {
    const text = await res.text();
    throw new Error(
      `Invalid server response format (${contentType || "non-JSON"}). ${
        text.startsWith("<!") ? "Received HTML instead of JSON API response." : ""
      }`
    );
  }

  return await res.json();
}

export function useDevFlowApi() {
  const apiBase = getApiBase();

  const fetchWorkspace = useCallback(
    async (dirPath?: string): Promise<WorkspaceResponse> => {
      const url = dirPath
        ? `${apiBase}/api/workspace?dir=${encodeURIComponent(dirPath)}`
        : `${apiBase}/api/workspace`;
      const res = await fetch(url);
      return parseJsonResponse<WorkspaceResponse>(res);
    },
    [apiBase]
  );

  const fetchDevices = useCallback(async (): Promise<Device[]> => {
    const res = await fetch(`${apiBase}/api/devices`);
    return parseJsonResponse<Device[]>(res);
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

  const fetchMcpStatus = useCallback(async (): Promise<import("../types").McpStatusResponse> => {
    const res = await fetch(`${apiBase}/api/mcp/status`);
    return parseJsonResponse<import("../types").McpStatusResponse>(res);
  }, [apiBase]);

  const toggleMcpServer = useCallback(
    async (enabled: boolean): Promise<{ success: boolean; enabled: boolean }> => {
      const res = await fetch(`${apiBase}/api/mcp/toggle`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ enabled }),
      });
      return parseJsonResponse<{ success: boolean; enabled: boolean }>(res);
    },
    [apiBase]
  );

  const fetchMcpLogs = useCallback(
    async (params?: FetchMcpLogsParams | number): Promise<import("../types").McpAccessLogEntry[]> => {
      let query = "";
      if (typeof params === "number") {
        query = `?limit=${params}`;
      } else if (params) {
        const q = new URLSearchParams();
        if (params.limit) q.set("limit", params.limit.toString());
        if (params.offset) q.set("offset", params.offset.toString());
        if (params.session_id) q.set("session_id", params.session_id);
        if (params.client) q.set("client", params.client);
        if (params.tool_name) q.set("tool_name", params.tool_name);
        if (params.status) q.set("status", params.status);
        if (params.search) q.set("search", params.search);
        query = `?${q.toString()}`;
      }
      const res = await fetch(`${apiBase}/api/mcp/logs${query}`);
      return parseJsonResponse<import("../types").McpAccessLogEntry[]>(res);
    },
    [apiBase]
  );

  const fetchMcpSessions = useCallback(async (): Promise<import("../types").McpSessionDescriptor[]> => {
    const res = await fetch(`${apiBase}/api/mcp/sessions`);
    return parseJsonResponse<import("../types").McpSessionDescriptor[]>(res);
  }, [apiBase]);

  const fetchMcpAgents = useCallback(async (): Promise<string[]> => {
    const res = await fetch(`${apiBase}/api/mcp/agents`);
    return parseJsonResponse<string[]>(res);
  }, [apiBase]);

  const deleteMcpLog = useCallback(
    async (id: string): Promise<{ success: boolean }> => {
      const res = await fetch(`${apiBase}/api/mcp/logs?id=${encodeURIComponent(id)}`, {
        method: "DELETE",
      });
      return parseJsonResponse<{ success: boolean }>(res);
    },
    [apiBase]
  );

  const clearMcpLogs = useCallback(
    async (sessionId?: string): Promise<{ success: boolean }> => {
      const query = sessionId ? `?session_id=${encodeURIComponent(sessionId)}` : "";
      const res = await fetch(`${apiBase}/api/mcp/logs${query}`, {
        method: "DELETE",
      });
      return parseJsonResponse<{ success: boolean }>(res);
    },
    [apiBase]
  );

  return useMemo(
    () => ({
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
      fetchMcpStatus,
      toggleMcpServer,
      fetchMcpLogs,
      fetchMcpSessions,
      fetchMcpAgents,
      deleteMcpLog,
      clearMcpLogs,
    }),
    [
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
      fetchMcpStatus,
      toggleMcpServer,
      fetchMcpLogs,
      fetchMcpSessions,
      fetchMcpAgents,
      deleteMcpLog,
      clearMcpLogs,
    ]
  );
}
