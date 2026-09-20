# Model Context Protocol (MCP) Integration Reference 🔌

DevFlow implements a high-performance, specification-compliant **Model Context Protocol (MCP)** server (`protocolVersion: 2024-11-05`) designed for AI coding agents.

---

## ⚡ 1-Command Auto-Connection

DevFlow can automatically discover installed AI IDEs and configure their MCP configuration files:

```bash
devflow mcp connect all
```

Supported AI Hosts:
- **Google Antigravity**: Writes to `~/.gemini/antigravity-ide/mcp_config.json` or `.agents/mcp_config.json`.
- **Claude Desktop**: Writes to `~/Library/Application Support/Claude/claude_desktop_config.json`.
- **Cursor IDE**: Writes to `.cursor/mcp.json` or `~/.cursor/mcp.json`.
- **VS Code / Cline / Roo-Code**: Writes to `.vscode/mcp.json`.

---

## 🛠️ Registered MCP Tools Catalog

DevFlow exposes 11 native MCP tools:

### 1. `devflow_start_session`
- **Description**: Start a complete development session: detects framework, builds project, installs to device/emulator, launches application, attaches log streaming, and starts kernel file watcher.
- **Parameters**:
  - `project_path` (string, required): Absolute or relative path to project directory.
  - `target` (string, optional): Specific device ID or name.
  - `framework` (string, optional): Override framework adapter (e.g. `kotlin`, `swift`, `flutter`, `react_native`, `tauri`).

### 2. `devflow_stop_session`
- **Description**: Stop a development session: stops file watcher, detaches logs, and terminates the running application.
- **Parameters**:
  - `session_id` (string, required): Active session identifier.

### 3. `devflow_reload`
- **Description**: Trigger framework-aware hot reload (HMR / VM reload) for an active development session.
- **Parameters**:
  - `session_id` (string, optional): Active session ID (auto-resolved if omitted).

### 4. `devflow_restart`
- **Description**: Trigger a full application restart (kill process + rebuild if needed + relaunch).
- **Parameters**:
  - `session_id` (string, optional): Active session ID (auto-resolved if omitted).

### 5. `devflow_get_logs`
- **Description**: Get recent log lines and clustered crash reports from the $O(1)$ circular ring buffer.
- **Parameters**:
  - `session_id` (string, optional): Active session ID.
  - `levels` (array of strings, optional): Filter by log levels (e.g. `["E", "W", "I", "D"]`).
  - `tags` (array of strings, optional): Filter by log tags.
  - `query` (string, optional): Search text query.
  - `limit` (integer, optional, default: 100): Maximum number of log lines to return.

### 6. `devflow_list_devices`
- **Description**: Return unified list of connected physical devices, bootable Android AVDs, Apple iOS simulators, and desktop hosts.
- **Parameters**: None.

### 7. `devflow_boot_emulator`
- **Description**: Boot an Android Virtual Device (AVD) or iOS Simulator by name or ID.
- **Parameters**:
  - `name` (string, required): Name or UUID of the emulator to boot.

### 8. `devflow_select_device`
- **Description**: Switch the active target device for a session and relaunch if necessary.
- **Parameters**:
  - `session_id` (string, required): Active session ID.
  - `device_id` (string, required): Target device identifier.

### 9. `devflow_build`
- **Description**: Perform a one-off build for a project and return artifact path and compilation duration.
- **Parameters**:
  - `project_path` (string, required): Project root directory.
  - `config` (object, optional): Build configuration overrides.

### 10. `devflow_doctor`
- **Description**: Run comprehensive environment, SDK, toolchain, and project health diagnostics with actionable fix hints.
- **Parameters**:
  - `project_path` (string, optional): Target project directory (defaults to current dir).

### 11. `devflow_get_session_state`
- **Description**: Return the current real-time state of a session (status, target device, build duration, event counts, last error).
- **Parameters**:
  - `session_id` (string, required): Active session ID.

---

## 🗄️ SQLite WAL Logging Engine

Every MCP request and response is automatically persisted in `~/.devflow/mcp_logs.db`:

```sql
CREATE TABLE mcp_access_logs (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    client TEXT NOT NULL,
    method TEXT NOT NULL,
    tool_name TEXT,
    session_id TEXT,
    project_path TEXT,
    arguments TEXT,
    response TEXT,
    duration_ms INTEGER NOT NULL,
    status TEXT NOT NULL,
    summary TEXT NOT NULL,
    error_message TEXT
);
```

### Querying Logs from CLI:
```bash
# Filter by agent
devflow mcp logs --agent claude

# Filter by session
devflow mcp logs --session cursor-9bda175f

# View full input parameters and output responses
devflow mcp logs --limit 5 --full
```
