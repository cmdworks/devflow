use devflow_core::event::EventBus;
use devflow_frameworks::session::SessionManager;
use devflow_mcp::McpHandler;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU16};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub event_bus: EventBus,
    pub active_sessions: Arc<Mutex<HashMap<String, Arc<SessionManager>>>>,
    pub mcp_handler: Arc<McpHandler>,
    pub mcp_enabled: Arc<AtomicBool>,
    pub actual_port: Arc<AtomicU16>,
}
