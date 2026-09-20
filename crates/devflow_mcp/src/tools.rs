use serde_json::{json, Value};

pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "devflow_start_session",
            "description": "Start a development session: detect project, build, install, launch, attach logs, start watcher.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Absolute or relative path to project directory" },
                    "target": { "type": "string", "description": "Device ID or name (optional)" },
                    "framework": { "type": "string", "description": "Override framework adapter (optional)" }
                },
                "required": ["project_path"]
            }
        }),
        json!({
            "name": "devflow_stop_session",
            "description": "Stop a development session: stop watcher, detach logs, terminate running app.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" }
                },
                "required": ["session_id"]
            }
        }),
        json!({
            "name": "devflow_reload",
            "description": "Trigger framework-aware reload (HMR/VM reload) or restart for an active session.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" }
                },
                "required": ["session_id"]
            }
        }),
        json!({
            "name": "devflow_restart",
            "description": "Trigger a full app restart (kill + relaunch) for an active session.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" }
                },
                "required": ["session_id"]
            }
        }),
        json!({
            "name": "devflow_get_logs",
            "description": "Get recent log lines and clustered crashes for a session with optional filtering.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" },
                    "levels": { "type": "array", "items": { "type": "string", "enum": ["E", "W", "I", "D"] }, "description": "Filter by log levels" },
                    "tags": { "type": "array", "items": { "type": "string" }, "description": "Filter by tag names" },
                    "query": { "type": "string", "description": "Search query filter" },
                    "limit": { "type": "integer", "default": 100, "description": "Max lines to return" },
                    "since": { "type": "string", "format": "date-time", "description": "ISO-8601 timestamp filter" }
                },
                "required": ["session_id"]
            }
        }),
        json!({
            "name": "devflow_list_devices",
            "description": "Return unified device list (Android devices & AVDs, iOS simulators, macOS targets) with status.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "devflow_boot_emulator",
            "description": "Boot an Android Virtual Device (AVD) or simulator by name/id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Name or ID of the AVD/simulator to boot" }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "devflow_select_device",
            "description": "Change the active device for a session and relaunch if needed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" },
                    "device_id": { "type": "string", "description": "Device ID to switch to" }
                },
                "required": ["session_id", "device_id"]
            }
        }),
        json!({
            "name": "devflow_build",
            "description": "Perform a one-off build; returns artifact path and duration.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Project directory path" },
                    "config": { "type": "object", "description": "Optional build config overrides" }
                },
                "required": ["project_path"]
            }
        }),
        json!({
            "name": "devflow_doctor",
            "description": "Run environment and project diagnostics and return structured findings with fix hints.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_path": { "type": "string", "description": "Project directory path (defaults to current dir)" }
                }
            }
        }),
        json!({
            "name": "devflow_get_session_state",
            "description": "Return the current session state (status, device, build duration, counts, last error).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Active session ID" }
                },
                "required": ["session_id"]
            }
        }),
    ]
}
