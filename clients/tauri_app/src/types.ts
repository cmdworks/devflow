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

export type ViewSection = "overview" | "targets" | "terminal" | "devices" | "doctor" | "settings";

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
  status: "Pass" | "Warn" | "Fail";
  message: string;
  details?: string;
  fix_hint?: string;
}

export interface DoctorReport {
  workspace_path: string;
  checks: DoctorCheck[];
  summary: {
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
    };
