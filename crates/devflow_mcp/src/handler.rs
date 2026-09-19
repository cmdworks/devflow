use crate::tools::get_tool_definitions;
use devflow_core::doctor::DoctorEngine;
use devflow_core::event::EventBus;
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use devflow_frameworks::adapter::BuildContext;
use devflow_frameworks::registry::FrameworkRegistry;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::{
    JsonRpcRequest, JsonRpcResponse, LogFilter, LogLevel,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct McpHandler {
    sessions: Arc<Mutex<HashMap<String, Arc<SessionManager>>>>,
    event_bus: EventBus,
}

impl McpHandler {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            event_bus: EventBus::default(),
        }
    }

    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
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
                JsonRpcResponse::success(id, result)
            }
            "notifications/initialized" | "initialized" => {
                JsonRpcResponse::success(id, json!({}))
            }
            "ping" => {
                JsonRpcResponse::success(id, json!({}))
            }
            "tools/list" => {
                let tools = get_tool_definitions();
                JsonRpcResponse::success(id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.dispatch_tool(name, args).await {
                    Ok(val) => {
                        let content = vec![json!({
                            "type": "text",
                            "text": serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string())
                        })];
                        JsonRpcResponse::success(id, json!({ "content": content, "structured": val }))
                    }
                    Err(err_msg) => {
                        JsonRpcResponse::error(id, -32000, err_msg, None)
                    }
                }
            }
            other => {
                let params = req.params.unwrap_or(Value::Null);
                match self.dispatch_tool(other, params).await {
                    Ok(val) => JsonRpcResponse::success(id, val),
                    Err(err_msg) => JsonRpcResponse::error(id, -32601, format!("Method not found: {} ({})", other, err_msg), None),
                }
            }
        }
    }

    pub async fn dispatch_tool(&self, name: &str, args: Value) -> Result<Value, String> {
        match name {
            "devflow_start_session" => {
                let project_path = args.get("project_path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| "Missing required parameter 'project_path'".to_string())?;

                let target = args.get("target").and_then(|t| t.as_str());
                let framework = args.get("framework").and_then(|f| f.as_str());

                let session = SessionManager::create(
                    project_path,
                    target,
                    framework,
                    self.event_bus.clone(),
                ).await.map_err(|e| format!("Failed to create session: {}", e))?;

                let session_arc = Arc::new(session);
                let session_id = session_arc.get_state().session_id.clone();

                session_arc.start_session().await
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
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let mut lock = self.sessions.lock().unwrap();
                    lock.remove(session_id)
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.stop().await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id }))
            }

            "devflow_reload" => {
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.reload(vec![]).await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id, "state": session.get_state() }))
            }

            "devflow_restart" => {
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                session.restart().await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id, "state": session.get_state() }))
            }

            "devflow_get_logs" => {
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                let mut filter = LogFilter::default();
                if let Some(levels) = args.get("levels").and_then(|l| l.as_array()) {
                    let parsed: Vec<LogLevel> = levels.iter()
                        .filter_map(|v| v.as_str())
                        .filter_map(|s| LogLevel::parse_char(s.chars().next().unwrap_or('I')))
                        .collect();
                    filter.levels = Some(parsed);
                }

                if let Some(tags) = args.get("tags").and_then(|t| t.as_array()) {
                    let parsed: Vec<String> = tags.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
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
                let name = args.get("name")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| "Missing required parameter 'name'".to_string())?;

                let msg = DeviceManager::boot_emulator(name).await?;
                Ok(json!({ "success": true, "message": msg, "target": name }))
            }

            "devflow_select_device" => {
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;
                let device_id = args.get("device_id")
                    .and_then(|d| d.as_str())
                    .ok_or_else(|| "Missing 'device_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                let target_device = DeviceManager::find_best_match(None, Some(device_id)).await
                    .ok_or_else(|| format!("Device '{}' not found", device_id))?;

                {
                    let mut st = session.state.write().unwrap();
                    st.target_device = Some(target_device);
                }

                session.restart().await.map_err(|e| e.to_string())?;
                Ok(json!({ "success": true, "session_id": session_id, "device_id": device_id }))
            }

            "devflow_build" => {
                let project_path = args.get("project_path")
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

                let res = adapter.build(&build_ctx).await
                    .map_err(|e| format!("Build failed: {}", e))?;

                Ok(json!(res))
            }

            "devflow_doctor" => {
                let project_path = args.get("project_path")
                    .and_then(|p| p.as_str())
                    .unwrap_or(".");

                let report = DoctorEngine::run_diagnostics(project_path).await;
                Ok(json!(report))
            }

            "devflow_get_session_state" => {
                let session_id = args.get("session_id")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Missing 'session_id'".to_string())?;

                let session = {
                    let lock = self.sessions.lock().unwrap();
                    lock.get(session_id).cloned()
                }.ok_or_else(|| format!("Session '{}' not found", session_id))?;

                Ok(json!(session.get_state()))
            }

            _ => Err(format!("Unknown tool: {}", name)),
        }
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
    async fn test_mcp_initialize() {
        let handler = McpHandler::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let resp = handler.handle_request(req).await;
        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        assert_eq!(res.get("protocolVersion").unwrap().as_str().unwrap(), "2024-11-05");
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
        let tools = resp.result.unwrap().get("tools").unwrap().as_array().unwrap().clone();
        assert_eq!(tools.len(), 11);
    }

    #[tokio::test]
    async fn test_mcp_doctor_call() {
        let handler = McpHandler::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "devflow_doctor",
                "arguments": {
                    "project_path": "."
                }
            })),
        };

        let resp = handler.handle_request(req).await;
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_mcp_list_devices() {
        let handler = McpHandler::new();
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "devflow_list_devices",
                "arguments": {}
            })),
        };

        let resp = handler.handle_request(req).await;
        assert!(resp.error.is_none());
    }
}
