use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use devflow_mcp::get_tool_definitions;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tracing::info;

use crate::state::AppState;

#[derive(Serialize)]
pub struct McpStatusResponse {
    pub enabled: bool,
    pub http_endpoint: String,
    pub port: u16,
    pub tools_count: usize,
    pub tools: Vec<serde_json::Value>,
    pub configs: McpClientConfigs,
}

#[derive(Serialize)]
pub struct McpClientConfigs {
    pub claude_desktop: serde_json::Value,
    pub cursor: serde_json::Value,
    pub antigravity: serde_json::Value,
    pub vscode: serde_json::Value,
}

#[derive(Deserialize)]
pub struct McpToggleRequest {
    pub enabled: bool,
}

pub async fn handle_mcp_status(State(state): State<AppState>) -> Json<McpStatusResponse> {
    let enabled = state.mcp_enabled.load(Ordering::Relaxed);
    let port = state.actual_port.load(Ordering::Relaxed);
    let endpoint = format!("http://localhost:{}/rpc", port);
    let tools = get_tool_definitions();
    let tools_count = tools.len();

    let configs = McpClientConfigs {
        claude_desktop: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "command": "devflow",
                    "args": ["mcp", "serve"]
                }
            }
        }),
        cursor: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "url": endpoint
                }
            }
        }),
        antigravity: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "command": "devflow",
                    "args": ["mcp", "serve"]
                }
            }
        }),
        vscode: serde_json::json!({
            "servers": {
                "devflow": {
                    "type": "http",
                    "url": endpoint
                }
            }
        }),
    };

    Json(McpStatusResponse {
        enabled,
        http_endpoint: endpoint,
        port,
        tools_count,
        tools,
        configs,
    })
}

pub async fn handle_mcp_toggle(
    State(state): State<AppState>,
    Json(req): Json<McpToggleRequest>,
) -> Json<serde_json::Value> {
    state.mcp_enabled.store(req.enabled, Ordering::Relaxed);
    info!("DevFlow MCP server toggle: enabled={}", req.enabled);
    Json(serde_json::json!({ "success": true, "enabled": req.enabled }))
}

#[derive(Deserialize)]
pub struct McpLogsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub session_id: Option<String>,
    pub client: Option<String>,
    pub agent: Option<String>,
    pub tool_name: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

pub async fn handle_mcp_logs(
    State(state): State<AppState>,
    Query(query): Query<McpLogsQuery>,
) -> Json<Vec<devflow_mcp::McpAccessLogEntry>> {
    let filter = devflow_mcp::McpLogFilter {
        limit: query.limit.or(Some(100)),
        offset: query.offset,
        session_id: query.session_id,
        client: query.client.or(query.agent),
        tool_name: query.tool_name,
        status: query.status,
        search: query.search,
    };
    let logs = state.mcp_handler.query_access_logs(&filter);
    Json(logs)
}

pub async fn handle_mcp_sessions(
    State(state): State<AppState>,
) -> Json<Vec<devflow_mcp::McpSessionDescriptor>> {
    let sessions = state.mcp_handler.get_access_log_descriptors();
    Json(sessions)
}

pub async fn handle_mcp_agents(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let agents = state.mcp_handler.get_access_log_clients();
    Json(agents)
}

#[derive(Deserialize)]
pub struct DeleteMcpLogsQuery {
    pub id: Option<String>,
    pub session_id: Option<String>,
}

pub async fn handle_delete_mcp_logs(
    State(state): State<AppState>,
    Query(query): Query<DeleteMcpLogsQuery>,
) -> Json<serde_json::Value> {
    if let Some(ref id) = query.id {
        let deleted = state.mcp_handler.delete_access_log(id);
        Json(serde_json::json!({ "success": deleted, "deleted_id": id }))
    } else if let Some(ref session_id) = query.session_id {
        let count = state.mcp_handler.delete_access_logs_by_session(session_id);
        Json(serde_json::json!({ "success": true, "deleted_count": count, "session_id": session_id }))
    } else {
        state.mcp_handler.clear_access_logs();
        Json(serde_json::json!({ "success": true, "cleared_all": true }))
    }
}

pub async fn handle_mcp_rpc(
    State(state): State<AppState>,
    Json(req): Json<devflow_protocol::JsonRpcRequest>,
) -> Result<Json<devflow_protocol::JsonRpcResponse>, StatusCode> {
    if !state.mcp_enabled.load(Ordering::Relaxed) {
        let resp = devflow_protocol::JsonRpcResponse::error(
            req.id,
            -32000,
            "DevFlow MCP server is currently disabled in companion app".to_string(),
            None,
        );
        return Ok(Json(resp));
    }
    let resp = state.mcp_handler.handle_request(req).await;
    Ok(Json(resp))
}
