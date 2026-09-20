use crate::db::{McpLogFilter, McpSessionDescriptor, McpSqliteStore};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAccessLogEntry {
    pub id: String,
    pub timestamp: String,
    pub client: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
    pub duration_ms: u64,
    pub status: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl McpAccessLogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: impl Into<String>,
        method: impl Into<String>,
        tool_name: Option<String>,
        session_id: Option<String>,
        project_path: Option<String>,
        arguments: Option<serde_json::Value>,
        response: Option<serde_json::Value>,
        duration_ms: u64,
        status: impl Into<String>,
        summary: impl Into<String>,
        error_message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            client: client.into(),
            method: method.into(),
            tool_name,
            session_id,
            project_path,
            arguments,
            response,
            duration_ms,
            status: status.into(),
            summary: summary.into(),
            error_message,
        }
    }
}

pub fn get_mcp_log_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".devflow").join("mcp_access.jsonl"))
}

pub fn record_access_log(entry: &McpAccessLogEntry) {
    if let Ok(store) = McpSqliteStore::open_default() {
        let _ = store.insert(entry);
    }
}

pub fn append_entry_to_disk(entry: &McpAccessLogEntry) {
    record_access_log(entry);
}

pub fn load_entries_from_disk(limit: usize) -> Vec<McpAccessLogEntry> {
    if let Ok(store) = McpSqliteStore::open_default() {
        let filter = McpLogFilter {
            limit: Some(limit),
            ..Default::default()
        };
        if let Ok(entries) = store.query(&filter) {
            return entries;
        }
    }

    // Fallback to legacy JSONL if SQLite is unavailable
    let mut entries = Vec::new();
    if let Some(path) = get_mcp_log_file_path() {
        if path.exists() {
            if let Ok(file) = std::fs::File::open(&path) {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    if let Ok(entry) = serde_json::from_str::<McpAccessLogEntry>(&line) {
                        entries.push(entry);
                    }
                }
            }
        }
    }
    if entries.len() > limit {
        entries.split_off(entries.len() - limit)
    } else {
        entries
    }
}

#[derive(Clone)]
pub struct McpAccessLogStore {
    sqlite: McpSqliteStore,
}

impl McpAccessLogStore {
    pub fn new() -> Self {
        let sqlite = McpSqliteStore::open_default().unwrap_or_else(|_| {
            McpSqliteStore::open_in_memory().expect("Failed in-memory fallback")
        });

        // Migrate legacy JSONL logs if SQLite database is empty
        if let Some(jsonl) = get_mcp_log_file_path() {
            let _ = sqlite.migrate_from_jsonl_if_empty(&jsonl);
        }

        Self { sqlite }
    }

    pub fn in_memory() -> Self {
        let sqlite = McpSqliteStore::open_in_memory().expect("In-memory SQLite failed");
        Self { sqlite }
    }

    pub fn record(&self, entry: McpAccessLogEntry) {
        let _ = self.sqlite.insert(&entry);
    }

    pub fn query(&self, filter: &McpLogFilter) -> Vec<McpAccessLogEntry> {
        self.sqlite.query(filter).unwrap_or_default()
    }

    pub fn get_recent(&self, limit: usize) -> Vec<McpAccessLogEntry> {
        let filter = McpLogFilter {
            limit: Some(limit),
            ..Default::default()
        };
        self.sqlite.query(&filter).unwrap_or_default()
    }

    pub fn get_distinct_sessions(&self) -> Vec<String> {
        self.sqlite.get_distinct_sessions().unwrap_or_default()
    }

    pub fn get_distinct_clients(&self) -> Vec<String> {
        self.sqlite.get_distinct_clients().unwrap_or_default()
    }

    pub fn get_session_descriptors(&self) -> Vec<McpSessionDescriptor> {
        self.sqlite.get_session_descriptors().unwrap_or_default()
    }

    pub fn delete_by_id(&self, id: &str) -> bool {
        self.sqlite.delete_by_id(id).unwrap_or(false)
    }

    pub fn delete_by_session(&self, session_id: &str) -> usize {
        self.sqlite.delete_by_session(session_id).unwrap_or(0)
    }

    pub fn clear(&self) {
        let _ = self.sqlite.clear_all();
        if let Some(path) = get_mcp_log_file_path() {
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_file(path.with_extension("jsonl.migrated"));
        }
    }
}

impl Default for McpAccessLogStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn query_access_logs(filter: &McpLogFilter) -> Vec<McpAccessLogEntry> {
    let store = McpAccessLogStore::new();
    store.query(filter)
}

pub fn get_distinct_log_sessions() -> Vec<String> {
    let store = McpAccessLogStore::new();
    store.get_distinct_sessions()
}

pub fn get_distinct_log_clients() -> Vec<String> {
    let store = McpAccessLogStore::new();
    store.get_distinct_clients()
}

pub fn get_log_session_descriptors() -> Vec<McpSessionDescriptor> {
    let store = McpAccessLogStore::new();
    store.get_session_descriptors()
}

pub fn delete_access_log(id: &str) -> bool {
    let store = McpAccessLogStore::new();
    store.delete_by_id(id)
}

pub fn delete_access_logs_by_session(session_id: &str) -> usize {
    let store = McpAccessLogStore::new();
    store.delete_by_session(session_id)
}

pub fn clear_all_access_logs() {
    let store = McpAccessLogStore::new();
    store.clear();
}
