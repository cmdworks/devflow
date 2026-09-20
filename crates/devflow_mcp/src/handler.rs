use crate::access_log::{McpAccessLogEntry, McpAccessLogStore};
use crate::tools::get_tool_definitions;
use devflow_core::doctor::DoctorEngine;
use devflow_core::event::{DevflowEvent, EventBus};
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use devflow_frameworks::adapter::BuildContext;
use devflow_frameworks::registry::FrameworkRegistry;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::{JsonRpcRequest, JsonRpcResponse, LogFilter, LogLevel};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct McpHandler {
    sessions: Arc<Mutex<HashMap<String, Arc<SessionManager>>>>,
    event_bus: EventBus,
    access_logs: Arc<McpAccessLogStore>,
    client_name: Arc<Mutex<String>>,
    agent_session_id: Arc<Mutex<String>>,
    last_target_session: Arc<Mutex<Option<String>>>,
}

pub fn clean_client_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("claude") {
        "Claude Desktop".to_string()
    } else if lower.contains("cursor") {
        "Cursor".to_string()
    } else if lower.contains("antigravity") {
        "Antigravity".to_string()
    } else if lower.contains("visual studio code") || lower.contains("vscode") {
        "VS Code".to_string()
    } else if lower.contains("windsurf") {
        "Windsurf".to_string()
    } else if lower.contains("roo") {
        "Roo-Code".to_string()
    } else if lower.contains("devflow-cli") || lower.contains("cli") {
        "devflow-cli".to_string()
    } else if raw.trim().is_empty() {
        "AI Agent".to_string()
    } else {
        raw.trim().to_string()
    }
}

fn client_to_slug(client: &str) -> String {
    let lower = client.to_lowercase();
    if lower.contains("claude") {
        "claude".to_string()
    } else if lower.contains("cursor") {
        "cursor".to_string()
    } else if lower.contains("antigravity") {
        "antigravity".to_string()
    } else if lower.contains("code") {
        "vscode".to_string()
    } else if lower.contains("cli") {
        "cli".to_string()
    } else {
        "agent".to_string()
    }
}

fn detect_initial_client() -> String {
    if let Ok(val) = std::env::var("DEVFLOW_AGENT_NAME") {
        if !val.trim().is_empty() {
            return clean_client_name(&val);
        }
    }
    if std::env::var("CURSOR_VERSION").is_ok() || std::env::var("CURSOR_PID").is_ok() {
        return "Cursor".to_string();
    }
    if std::env::var("ANTIGRAVITY_IDE").is_ok() || std::env::var("ANTIGRAVITY_CSRF_TOKEN").is_ok() {
        return "Antigravity".to_string();
    }
    if std::env::var("VSCODE_PID").is_ok() || std::env::var("VSCODE_GIT_IPC_HANDLE").is_ok() {
        return "VS Code".to_string();
    }
    "AI Agent".to_string()
}

impl McpHandler {
    pub fn new() -> Self {
        let initial_client = detect_initial_client();
        let slug = client_to_slug(&initial_client);
        let session_id = format!("{}-{}", slug, &Uuid::new_v4().to_string()[..8]);

        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            event_bus: EventBus::default(),
            access_logs: Arc::new(McpAccessLogStore::default()),
            client_name: Arc::new(Mutex::new(initial_client)),
            agent_session_id: Arc::new(Mutex::new(session_id)),
            last_target_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_client_hint(self, hint: Option<String>) -> Self {
        if let Some(h) = hint {
            let cleaned = clean_client_name(&h);
            let slug = client_to_slug(&cleaned);
            let sess = format!("{}-{}", slug, &Uuid::new_v4().to_string()[..8]);
            *self.client_name.lock().unwrap() = cleaned;
            *self.agent_session_id.lock().unwrap() = sess;
        }
        self
    }

    pub fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.event_bus = event_bus;
        self
    }

    pub fn get_client_name(&self) -> String {
        self.client_name.lock().unwrap().clone()
    }

    pub fn set_client_name(&self, name: &str) {
        let cleaned = clean_client_name(name);
        let slug = client_to_slug(&cleaned);
        let sess = format!("{}-{}", slug, &Uuid::new_v4().to_string()[..8]);
        *self.client_name.lock().unwrap() = cleaned;
        *self.agent_session_id.lock().unwrap() = sess;
    }

    pub fn get_agent_session_id(&self) -> String {
        self.agent_session_id.lock().unwrap().clone()
    }

    pub fn get_last_target_session(&self) -> Option<String> {
        self.last_target_session.lock().unwrap().clone()
    }

    pub fn get_access_logs(&self, limit: usize) -> Vec<McpAccessLogEntry> {
        self.access_logs.get_recent(limit)
    }

    pub fn query_access_logs(&self, filter: &crate::db::McpLogFilter) -> Vec<McpAccessLogEntry> {
        self.access_logs.query(filter)
    }

    pub fn get_access_log_sessions(&self) -> Vec<String> {
        self.access_logs.get_distinct_sessions()
    }

    pub fn get_access_log_clients(&self) -> Vec<String> {
        self.access_logs.get_distinct_clients()
    }

    pub fn get_access_log_descriptors(&self) -> Vec<crate::db::McpSessionDescriptor> {
        self.access_logs.get_session_descriptors()
    }

    pub fn delete_access_log(&self, id: &str) -> bool {
        self.access_logs.delete_by_id(id)
    }

    pub fn delete_access_logs_by_session(&self, session_id: &str) -> usize {
        self.access_logs.delete_by_session(session_id)
    }

    pub fn clear_access_logs(&self) {
        self.access_logs.clear();
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let start = std::time::Instant::now();
        let id = req.id.clone();
        let method = req.method.clone();

        let (resp, log_entry) = match method.as_str() {
            "initialize" => {
                // Extract clientInfo if supplied in MCP initialize payload
                if let Some(client_name) = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("clientInfo"))
                    .and_then(|ci| ci.get("name"))
                    .and_then(|n| n.as_str())
                {
                    self.set_client_name(client_name);
                }

                let current_client = self.get_client_name();
                let current_session = self.get_agent_session_id();

                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "devflow",
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    }
                });
                let elapsed = start.elapsed().as_millis() as u64;
                let log = McpAccessLogEntry::new(
                    current_client.clone(),
                    "initialize",
                    None,
                    Some(current_session),
                    None,
                    req.params,
                    Some(result.clone()),
                    elapsed,
                    "success",
                    format!(
                        "{} initialized DevFlow MCP protocol v2024-11-05",
                        current_client
                    ),
                    None,
                );
                (JsonRpcResponse::success(id, result), Some(log))
            }
            "notifications/initialized" | "initialized" => {
                (JsonRpcResponse::success(id, json!({})), None)
            }
            "ping" => (JsonRpcResponse::success(id, json!({})), None),
            "tools/list" => {
                let current_client = self.get_client_name();
                let current_session = self.get_agent_session_id();
                let tools = get_tool_definitions();
                let tools_count = tools.len();
                let elapsed = start.elapsed().as_millis() as u64;
                let log = McpAccessLogEntry::new(
                    current_client.clone(),
                    "tools/list",
                    None,
                    Some(current_session),
                    None,
                    None,
                    Some(json!({ "tools_count": tools_count })),
                    elapsed,
                    "success",
                    format!(
                        "{} queried {} MCP tool definitions",
                        current_client, tools_count
                    ),
                    None,
                );
                (
                    JsonRpcResponse::success(id, json!({ "tools": tools })),
                    Some(log),
                )
            }
            "tools/call" => {
                let current_client = self.get_client_name();
                let agent_sess = self.get_agent_session_id();

                let params = req.params.unwrap_or(Value::Null);
                let name = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut args = params.get("arguments").cloned().unwrap_or(json!({}));

                let mut session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                // Smart Session Awareness: Auto-resolve session_id if omitted for active session operations
                if session_id.is_none()
                    && matches!(
                        name.as_str(),
                        "devflow_reload"
                            | "devflow_restart"
                            | "devflow_get_logs"
                            | "devflow_get_session_state"
                            | "devflow_select_device"
                            | "devflow_stop_session"
                    )
                {
                    if let Some(ref last_target) = *self.last_target_session.lock().unwrap() {
                        session_id = Some(last_target.clone());
                        if let Some(obj) = args.as_object_mut() {
                            obj.insert("session_id".to_string(), json!(last_target));
                        }
                    }
                }

                // If session_id is explicitly given, remember it as last active target
                if let Some(ref sid) = session_id {
                    *self.last_target_session.lock().unwrap() = Some(sid.clone());
                }

                let project_path = args
                    .get("project_path")
                    .or_else(|| args.get("target_path"))
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string());

                match self.dispatch_tool(&name, args.clone()).await {
                    Ok(val) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        let summary = generate_tool_summary(&name, &args, &val);

                        // If devflow_start_session, extract newly created session_id
                        let resolved_session_id = session_id.or_else(|| {
                            val.get("session_id")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string())
                        });

                        if let Some(ref sid) = resolved_session_id {
                            *self.last_target_session.lock().unwrap() = Some(sid.clone());
                        }

                        // Every single log entry has an effective session (target session or agent session fallback)
                        let effective_session =
                            resolved_session_id.unwrap_or_else(|| agent_sess.clone());

                        let log = McpAccessLogEntry::new(
                            current_client,
                            "tools/call",
                            Some(name.clone()),
                            Some(effective_session),
                            project_path,
                            Some(args),
                            Some(val.clone()),
                            elapsed,
                            "success",
                            summary,
                            None,
                        );
                        let content = vec![json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string())
                        })];
                        (
                            JsonRpcResponse::success(
                                id,
                                json!({ "content": content, "structured": val }),
                            ),
                            Some(log),
                        )
                    }
                    Err(err_msg) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        let effective_session = session_id.unwrap_or_else(|| agent_sess.clone());

                        let log = McpAccessLogEntry::new(
                            current_client,
                            "tools/call",
                            Some(name.clone()),
                            Some(effective_session),
                            project_path,
                            Some(args),
                            Some(json!({ "error": err_msg })),
                            elapsed,
                            "error",
                            format!("Failed tool call '{}': {}", name, err_msg),
                            Some(err_msg.clone()),
                        );
                        (JsonRpcResponse::error(id, -32000, err_msg, None), Some(log))
                    }
                }
            }
            other => {
                let current_client = self.get_client_name();
                let agent_sess = self.get_agent_session_id();
                let params = req.params.unwrap_or(Value::Null);
                match self.dispatch_tool(other, params.clone()).await {
                    Ok(val) => {
                        let elapsed = start.elapsed().as_millis() as u64;
                        let log = McpAccessLogEntry::new(
                            current_client,
                            other,
                            Some(other.to_string()),
                            Some(agent_sess),
                            None,
                            Some(params),
                            Some(val.clone()),
                            elapsed,
                            "success",
                            format!("Executed method {}", other),
                            None,
                        );
                        (JsonRpcResponse::success(id, val), Some(log))
                    }
                    Err(err_msg) => (
                        JsonRpcResponse::error(
                            id,
                            -32601,
                            format!("Method not found: {} ({})", other, err_msg),
                            None,
                        ),
                        None,
                    ),
                }
            }
        };

        if let Some(entry) = log_entry {
            self.access_logs.record(entry.clone());
            if let Ok(val) = serde_json::to_value(&entry) {
                self.event_bus
                    .publish(DevflowEvent::McpAccessLog { entry: val });
            }
        }

        resp
    }

    pub async fn dispatch_tool(&self, name: &str, args: Value) -> Result<Value, String> {
        match name {
            "devflow_start_session" => {
                let project_path = args
                    .get("project_path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| "Missing required parameter 'project_path'".to_string())?;

                let target = args.get("target").and_then(|t| t.as_str());
                let framework = args.get("framework").and_then(|f| f.as_str());

                let session =
                    SessionManager::create(project_path, target, framework, self.event_bus.clone())
                        .await
                        .map_err(|e| format!("Failed to create session: {}", e))?;

                let session_arc = Arc::new(session);
                let session_id = session_arc.get_state().session_id.clone();

                session_arc
                    .start_session()
                    .await
                    .map_err(|e| format!("Failed to start session: {}", e))?;

                let mut lock = self.sessions.lock().unwrap();
                lock.insert(session_id.clone(), session_arc);

                Ok(json!({
                    "session_id": session_id,
                    "status": "running",
                    "message": "DevFlow session successfully started"
                }))
            }

            "devflow_stop_session" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let mut lock = self.sessions.lock().unwrap();
                    lock.remove(session_id)
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.stop().await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id }))
            }

            "devflow_reload" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.reload(vec![]).await.map_err(|e| e.to_string())?;
                Ok(
                    json!({ "success": true, "session_id": session_id, "state": session.get_state() }),
                )
            }

            "devflow_restart" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.restart().await.map_err(|e| e.to_string())?;
                Ok(
                    json!({ "success": true, "session_id": session_id, "state": session.get_state() }),
                )
            }

            "devflow_get_logs" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                let mut filter = LogFilter::default();
                if let Some(levels) = args.get("levels").and_then(|l| l.as_array()) {
                    let parsed: Vec<LogLevel> = levels
                        .iter()
                        .filter_map(|v| v.as_str())
                        .filter_map(|s| LogLevel::parse_char(s.chars().next().unwrap_or('I')))
                        .collect();
                    filter.levels = Some(parsed);
                }

                if let Some(tags) = args.get("tags").and_then(|t| t.as_array()) {
                    let parsed: Vec<String> = tags
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    filter.tags = Some(parsed);
                }

                if let Some(q) = args.get("query").and_then(|q| q.as_str()) {
                    filter.query = Some(q.to_string());
                }

                if let Some(lim) = args.get("limit").and_then(|l| l.as_u64()) {
                    filter.limit = Some(lim as usize);
                }

                let lines = session.get_logs(&filter);
                let crashes = session.log_buffer.get_crash_clusters();
                Ok(json!({ "session_id": session_id, "lines": lines, "crashes": crashes }))
            }

            "devflow_list_devices" => {
                let devices = DeviceManager::discover_all().await;
                Ok(json!({ "devices": devices }))
            }

            "devflow_boot_emulator" => {
                let name = args
                    .get("name")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| "Missing required parameter 'name'".to_string())?;

                let msg = DeviceManager::boot_emulator(name).await?;
                Ok(json!({ "success": true, "message": msg, "target": name }))
            }

            "devflow_select_device" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;
                let device_id = args
                    .get("device_id")
                    .and_then(|d| d.as_str())
                    .ok_or_else(|| "Missing 'device_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                let target_device = DeviceManager::find_best_match(None, Some(device_id))
                    .await
                    .ok_or_else(|| format!("Device '{}' not found", device_id))?;

                {
                    let mut st = session.state.write().unwrap();
                    st.target_device = Some(target_device);
                }

                session.restart().await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id, "device_id": device_id }))
            }

            "devflow_build" => {
                let project_path = args
                    .get("project_path")
                    .and_then(|p| p.as_str())
                    .unwrap_or(".");

                let project = Project::detect(project_path)
                    .map_err(|e| format!("Project detection failed: {}", e))?;

                let registry = FrameworkRegistry::new();
                let adapter = registry.select_adapter(&project);

                let build_ctx = BuildContext {
                    project_dir: project.root_dir.clone(),
                    config: project.effective_config(),
                    target_device: None,
                    is_release: false,
                };

                let res = adapter
                    .build(&build_ctx)
                    .await
                    .map_err(|e| format!("Build failed: {}", e))?;

                Ok(json!(res))
            }

            "devflow_doctor" => {
                let project_path = args
                    .get("project_path")
                    .and_then(|p| p.as_str())
                    .unwrap_or(".");

                let report = DoctorEngine::run_diagnostics(project_path).await;
                Ok(json!(report))
            }

            "devflow_get_session_state" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }
                .ok_or_else(|| format!("Session '{}' not found", session_id))?;

                Ok(json!(session.get_state()))
            }

            _ => Err(format!("Unknown tool: {}", name)),
        }
    }
}

fn generate_tool_summary(name: &str, args: &Value, val: &Value) -> String {
    match name {
        "devflow_start_session" => {
            let path = args
                .get("project_path")
                .and_then(|p| p.as_str())
                .unwrap_or(".");
            format!("Started session for project '{}'", path)
        }
        "devflow_stop_session" => {
            let session_id = args
                .get("session_id")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            format!("Stopped session '{}'", session_id)
        }
        "devflow_reload" => {
            let session_id = args
                .get("session_id")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            format!("Reloaded active session '{}'", session_id)
        }
        "devflow_restart" => {
            let session_id = args
                .get("session_id")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            format!("Restarted active session '{}'", session_id)
        }
        "devflow_get_logs" => {
            let lines_count = val
                .get("lines")
                .and_then(|l| l.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            format!("Retrieved {} log entries", lines_count)
        }
        "devflow_list_devices" => {
            let count = val
                .get("devices")
                .and_then(|d| d.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            format!("Discovered {} connected device(s)/emulator(s)", count)
        }
        "devflow_boot_emulator" => {
            let target = args.get("name").and_then(|n| n.as_str()).unwrap_or("");
            format!("Booted emulator/simulator '{}'", target)
        }
        "devflow_select_device" => {
            let dev_id = args.get("device_id").and_then(|d| d.as_str()).unwrap_or("");
            format!("Selected active device '{}'", dev_id)
        }
        "devflow_build" => {
            let path = args
                .get("project_path")
                .and_then(|p| p.as_str())
                .unwrap_or(".");
            format!("Executed standalone build for '{}'", path)
        }
        "devflow_doctor" => {
            let checks_count = val
                .get("checks")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let passed = val
                .get("summary")
                .and_then(|s| s.get("passed"))
                .and_then(|p| p.as_u64())
                .unwrap_or(0);
            format!(
                "Doctor diagnostics: {}/{} checks passed",
                passed, checks_count
            )
        }
        "devflow_get_session_state" => {
            let status = val
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            format!("Retrieved session state (status: {})", status)
        }
        _ => format!("Executed tool '{}'", name),
    }
}

impl Default for McpHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_initialize_and_agent_detection() {
        let handler = McpHandler::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "clientInfo": {
                    "name": "Cursor",
                    "version": "0.45.0"
                }
            })),
        };

        let resp = handler.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(handler.get_client_name(), "Cursor");
        assert!(handler.get_agent_session_id().starts_with("cursor-"));

        // Subsequent doctor call inherits Cursor and agent session id
        let doc_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "devflow_doctor",
                "arguments": { "project_path": "." }
            })),
        };
        let doc_resp = handler.handle_request(doc_req).await;
        assert!(doc_resp.error.is_none());

        let logs = handler.get_access_logs(5);
        assert!(!logs.is_empty());
        assert_eq!(logs[0].client, "Cursor");
        assert!(logs[0].session_id.is_some());
    }

    #[tokio::test]
    async fn test_mcp_tools_list() {
        let handler = McpHandler::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        let resp = handler.handle_request(req).await;
        assert!(resp.error.is_none());
        let tools = resp
            .result
            .unwrap()
            .get("tools")
            .unwrap()
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(tools.len(), 11);
    }
}
