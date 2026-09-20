export type TargetStatus = "idle" | "building" | "running" | "error";

export interface ProjectTarget {
  id: string;
  name: string;
  path: string;
  platform: string;
  framework: string;
  is_default: boolean;
}

export interface Device {
  id: string;
  name: string;
  platform: string;
  state: string;
  target_arch?: string;
  os_version?: string;
  is_emulator?: boolean;
  is_default?: boolean;
  online?: boolean;
}

export interface ActiveSessionInfo {
  session_id: string;
  project_name: string;
  project_path: string;
  platform: string;
  pid: number;
  status: string;
  socket_path: string;
}

export interface WorkspaceResponse {
  workspace_name: string;
  workspace_path: string;
  targets: ProjectTarget[];
  devices: Device[];
  active_sessions: ActiveSessionInfo[];
}

export type ViewSection = "overview" | "targets" | "terminal" | "devices" | "doctor" | "mcp" | "settings";

export interface KnownWorkspace {
  name: string;
  custom_name?: string;
  path: string;
  platform: string;
  framework: string;
  last_opened: string;
}

export interface LogEntry {
  timestamp: string;
  level: "E" | "W" | "I" | "D" | "T" | string;
  message: string;
  tag?: string;
  metadata?: Record<string, unknown>;
}

export interface DoctorCheck {
  name: string;
  status: "passed" | "warning" | "failed" | "skipped" | "Pass" | "Warn" | "Fail" | string;
  message: string;
  category?: string;
  detected_version?: string;
  details?: string;
  fix_hint?: string;
}

export interface DoctorReport {
  project_path?: string;
  workspace_path?: string;
  checks: DoctorCheck[];
  passed_count?: number;
  warning_count?: number;
  failure_count?: number;
  summary?: {
    passed: number;
    warnings: number;
    failed: number;
  };
}

export interface PaneInfo {
  id: string;
  targetId: string; // 'all' for combined
  title: string;
  platform: string;
  framework?: string;
  isCombined: boolean;
  status: TargetStatus;
  buildTime?: number;
  target?: ProjectTarget;
}

export interface McpToolProperty {
  type: string;
  description?: string;
  enum?: string[];
}

export interface McpToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: string;
    properties?: Record<string, McpToolProperty>;
    required?: string[];
  };
}

export interface McpClientConfigs {
  claude_desktop: Record<string, unknown>;
  cursor: Record<string, unknown>;
  antigravity: Record<string, unknown>;
  vscode: Record<string, unknown>;
}

export interface McpStatusResponse {
  enabled: boolean;
  http_endpoint: string;
  port: number;
  tools_count: number;
  tools: McpToolDefinition[];
  configs: McpClientConfigs;
}

export type ServerEvent =
  | {
      type: "LogAppended";
      payload: {
        session_id?: string;
        entry: LogEntry;
      };
    }
  | {
      type: "BuildStarted";
      payload: {
        session_id: string;
        project_name: string;
      };
    }
  | {
      type: "BuildCompleted";
      payload: {
        session_id: string;
        result: {
          success: boolean;
          duration_ms: number;
          error_message?: string;
        };
      };
    }
  | {
      type: "SessionStateChanged";
      payload: {
        session_id: string;
        status: string;
      };
    }
  | {
      type: "McpAccessLog";
      payload: McpAccessLogEntry;
    };

export interface McpAccessLogEntry {
  id: string;
  timestamp: string;
  client: string;
  method: string;
  tool_name?: string;
  session_id?: string;
  project_path?: string;
  arguments?: Record<string, unknown> | unknown;
  response?: Record<string, unknown> | unknown;
  duration_ms: number;
  status: "success" | "error";
  summary: string;
  error_message?: string;
}

export interface McpSessionDescriptor {
  session_id: string;
  client: string;
  total_calls: number;
  last_tool?: string;
  last_timestamp: string;
}

export interface UpdateCheckResponse {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  release_name: string;
  release_notes: string;
  published_at: string;
  download_url?: string | null;
  html_url: string;
  target_platform: string;
}
