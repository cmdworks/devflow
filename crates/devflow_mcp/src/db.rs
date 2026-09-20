use crate::access_log::McpAccessLogEntry;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpLogFilter {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub session_id: Option<String>,
    pub client: Option<String>,
    pub tool_name: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSessionDescriptor {
    pub session_id: String,
    pub client: String,
    pub total_calls: usize,
    pub last_tool: Option<String>,
    pub last_timestamp: String,
}

#[derive(Clone)]
pub struct McpSqliteStore {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

pub fn get_mcp_db_path() -> Option<PathBuf> {
    if let Ok(custom) = std::env::var("DEVFLOW_DB_PATH") {
        return Some(PathBuf::from(custom));
    }
    dirs::home_dir().map(|h| h.join(".devflow").join("mcp_logs.db"))
}

impl McpSqliteStore {
    pub fn open_default() -> anyhow::Result<Self> {
        let db_path = get_mcp_db_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine DevFlow home directory"))?;
        Self::open(db_path)
    }

    pub fn open_in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open(db_path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            let _ = create_dir_all(parent);
        }
        let conn = Connection::open(&db_path)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };
        store.init_schema()?;
        Ok(store)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn init_schema(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS mcp_access_logs (
                 id TEXT PRIMARY KEY,
                 timestamp TEXT NOT NULL,
                 client TEXT NOT NULL,
                 method TEXT NOT NULL,
                 tool_name TEXT,
                 session_id TEXT,
                 project_path TEXT,
                 arguments_json TEXT,
                 response_json TEXT,
                 duration_ms INTEGER NOT NULL,
                 status TEXT NOT NULL,
                 summary TEXT NOT NULL,
                 error_message TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_mcp_logs_timestamp ON mcp_access_logs(timestamp DESC);
             CREATE INDEX IF NOT EXISTS idx_mcp_logs_session ON mcp_access_logs(session_id);
             CREATE INDEX IF NOT EXISTS idx_mcp_logs_client ON mcp_access_logs(client);
             CREATE INDEX IF NOT EXISTS idx_mcp_logs_tool ON mcp_access_logs(tool_name);
             CREATE INDEX IF NOT EXISTS idx_mcp_logs_status ON mcp_access_logs(status);",
        )?;
        Ok(())
    }

    pub fn insert(&self, entry: &McpAccessLogEntry) -> anyhow::Result<()> {
        let args_json = entry.arguments.as_ref().map(|v| v.to_string());
        let resp_json = entry.response.as_ref().map(|v| v.to_string());

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO mcp_access_logs (
                id, timestamp, client, method, tool_name, session_id, project_path,
                arguments_json, response_json, duration_ms, status, summary, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                entry.id,
                entry.timestamp,
                entry.client,
                entry.method,
                entry.tool_name,
                entry.session_id,
                entry.project_path,
                args_json,
                resp_json,
                entry.duration_ms as i64,
                entry.status,
                entry.summary,
                entry.error_message,
            ],
        )?;
        Ok(())
    }

    pub fn query(&self, filter: &McpLogFilter) -> anyhow::Result<Vec<McpAccessLogEntry>> {
        let mut query = String::from(
            "SELECT id, timestamp, client, method, tool_name, session_id, project_path,
                    arguments_json, response_json, duration_ms, status, summary, error_message
             FROM mcp_access_logs WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref sid) = filter.session_id {
            if !sid.is_empty() && sid != "all" {
                query.push_str(" AND session_id = ?");
                params_vec.push(Box::new(sid.clone()));
            }
        }

        if let Some(ref cl) = filter.client {
            if !cl.is_empty() && cl != "all" {
                query.push_str(" AND (client = ? OR client LIKE ?)");
                params_vec.push(Box::new(cl.clone()));
                params_vec.push(Box::new(format!("%{}%", cl)));
            }
        }

        if let Some(ref tool) = filter.tool_name {
            if !tool.is_empty() && tool != "all" {
                query.push_str(" AND tool_name = ?");
                params_vec.push(Box::new(tool.clone()));
            }
        }

        if let Some(ref st) = filter.status {
            if !st.is_empty() && st != "all" {
                query.push_str(" AND status = ?");
                params_vec.push(Box::new(st.clone()));
            }
        }

        if let Some(ref q) = filter.search {
            let trimmed = q.trim();
            if !trimmed.is_empty() {
                let pattern = format!("%{}%", trimmed);
                query.push_str(" AND (summary LIKE ? OR tool_name LIKE ? OR method LIKE ? OR client LIKE ? OR arguments_json LIKE ? OR response_json LIKE ? OR session_id LIKE ?)");
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        query.push_str(" ORDER BY timestamp DESC");

        let limit = filter.limit.unwrap_or(100);
        query.push_str(" LIMIT ?");
        params_vec.push(Box::new(limit as i64));

        if let Some(offset) = filter.offset {
            query.push_str(" OFFSET ?");
            params_vec.push(Box::new(offset as i64));
        }

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;

        let slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(slice.as_slice(), |row| {
            let id: String = row.get(0)?;
            let timestamp: String = row.get(1)?;
            let client: String = row.get(2)?;
            let method: String = row.get(3)?;
            let tool_name: Option<String> = row.get(4)?;
            let session_id: Option<String> = row.get(5)?;
            let project_path: Option<String> = row.get(6)?;
            let args_raw: Option<String> = row.get(7)?;
            let resp_raw: Option<String> = row.get(8)?;
            let duration_ms: i64 = row.get(9)?;
            let status: String = row.get(10)?;
            let summary: String = row.get(11)?;
            let error_message: Option<String> = row.get(12)?;

            let arguments = args_raw.and_then(|s| serde_json::from_str(&s).ok());
            let response = resp_raw.and_then(|s| serde_json::from_str(&s).ok());

            Ok(McpAccessLogEntry {
                id,
                timestamp,
                client,
                method,
                tool_name,
                session_id,
                project_path,
                arguments,
                response,
                duration_ms: duration_ms.max(0) as u64,
                status,
                summary,
                error_message,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn get_distinct_sessions(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT session_id FROM mcp_access_logs WHERE session_id IS NOT NULL AND session_id != '' ORDER BY timestamp DESC LIMIT 50",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut sessions = Vec::new();
        for s in rows.flatten() {
            sessions.push(s);
        }
        Ok(sessions)
    }

    pub fn get_distinct_clients(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT client FROM mcp_access_logs WHERE client IS NOT NULL AND client != '' ORDER BY timestamp DESC LIMIT 50",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut clients = Vec::new();
        for c in rows.flatten() {
            clients.push(c);
        }
        Ok(clients)
    }

    pub fn get_session_descriptors(&self) -> anyhow::Result<Vec<McpSessionDescriptor>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT session_id, client, COUNT(*) as total_calls,
                    MAX(tool_name) as last_tool, MAX(timestamp) as last_timestamp
             FROM mcp_access_logs
             WHERE session_id IS NOT NULL AND session_id != ''
             GROUP BY session_id
             ORDER BY last_timestamp DESC
             LIMIT 50",
        )?;
        let rows = stmt.query_map([], |row| {
            let session_id: String = row.get(0)?;
            let client: String = row.get(1)?;
            let total_calls: i64 = row.get(2)?;
            let last_tool: Option<String> = row.get(3)?;
            let last_timestamp: String = row.get(4)?;
            Ok(McpSessionDescriptor {
                session_id,
                client,
                total_calls: total_calls.max(0) as usize,
                last_tool,
                last_timestamp,
            })
        })?;
        let mut descriptors = Vec::new();
        for d in rows.flatten() {
            descriptors.push(d);
        }
        Ok(descriptors)
    }

    pub fn delete_by_id(&self, id: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let prefix = format!("{}%", id);
        let affected = conn.execute(
            "DELETE FROM mcp_access_logs WHERE id = ?1 OR id LIKE ?2",
            params![id, prefix],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_by_session(&self, session_id: &str) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "DELETE FROM mcp_access_logs WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(affected)
    }

    pub fn clear_all(&self) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM mcp_access_logs", [])?;
        let _ = conn.execute("VACUUM", []);
        Ok(affected)
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM mcp_access_logs", [], |r| r.get(0))?;
        Ok(count.max(0) as usize)
    }

    pub fn migrate_from_jsonl_if_empty(&self, jsonl_path: &Path) -> anyhow::Result<usize> {
        if !jsonl_path.exists() {
            return Ok(0);
        }
        if self.count()? > 0 {
            return Ok(0);
        }

        let file = match std::fs::File::open(jsonl_path) {
            Ok(f) => f,
            Err(_) => return Ok(0),
        };
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(entry) = serde_json::from_str::<McpAccessLogEntry>(&line) {
                if self.insert(&entry).is_ok() {
                    count += 1;
                }
            }
        }

        if count > 0 {
            info!(
                "Migrated {} legacy MCP access logs from JSONL to SQLite",
                count
            );
            let migrated_path = jsonl_path.with_extension("jsonl.migrated");
            let _ = std::fs::rename(jsonl_path, migrated_path);
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sqlite_insert_and_query() {
        let store = McpSqliteStore::open_in_memory().expect("open memory db");
        let entry = McpAccessLogEntry::new(
            "test-agent",
            "tools/call",
            Some("devflow_doctor".to_string()),
            Some("sess-123".to_string()),
            Some("/tmp/project".to_string()),
            Some(json!({ "project_path": "/tmp/project" })),
            Some(json!({ "passed_count": 17 })),
            150,
            "success",
            "Doctor ran 17 checks",
            None,
        );
        store.insert(&entry).expect("insert entry");

        let filter = McpLogFilter::default();
        let logs = store.query(&filter).expect("query logs");
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].client, "test-agent");
        assert_eq!(logs[0].tool_name.as_deref(), Some("devflow_doctor"));
        assert_eq!(logs[0].session_id.as_deref(), Some("sess-123"));
        assert_eq!(logs[0].project_path.as_deref(), Some("/tmp/project"));
        assert_eq!(
            logs[0].arguments,
            Some(json!({ "project_path": "/tmp/project" }))
        );
        assert_eq!(logs[0].response, Some(json!({ "passed_count": 17 })));
    }

    #[test]
    fn test_sqlite_filtering_and_sessions() {
        let store = McpSqliteStore::open_in_memory().expect("open memory db");
        let e1 = McpAccessLogEntry::new(
            "Claude Desktop",
            "tools/call",
            Some("devflow_doctor".to_string()),
            Some("sess-A".to_string()),
            None,
            Some(json!({})),
            Some(json!({"ok": true})),
            50,
            "success",
            "Summary A",
            None,
        );
        let e2 = McpAccessLogEntry::new(
            "Cursor",
            "tools/call",
            Some("devflow_build".to_string()),
            Some("sess-B".to_string()),
            None,
            Some(json!({})),
            Some(json!({"error": "build failed"})),
            120,
            "error",
            "Summary B error",
            Some("build failed".to_string()),
        );

        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();

        // Distinct sessions
        let sessions = store.get_distinct_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"sess-A".to_string()));
        assert!(sessions.contains(&"sess-B".to_string()));

        // Distinct clients
        let clients = store.get_distinct_clients().unwrap();
        assert_eq!(clients.len(), 2);
        assert!(clients.contains(&"Claude Desktop".to_string()));
        assert!(clients.contains(&"Cursor".to_string()));

        // Filter by client
        let filter_client = McpLogFilter {
            client: Some("Cursor".to_string()),
            ..Default::default()
        };
        let cursor_logs = store.query(&filter_client).unwrap();
        assert_eq!(cursor_logs.len(), 1);
        assert_eq!(cursor_logs[0].client, "Cursor");

        // Session descriptors
        let descriptors = store.get_session_descriptors().unwrap();
        assert_eq!(descriptors.len(), 2);
        let desc_a = descriptors
            .iter()
            .find(|d| d.session_id == "sess-A")
            .unwrap();
        assert_eq!(desc_a.client, "Claude Desktop");
        assert_eq!(desc_a.total_calls, 1);
        assert_eq!(desc_a.last_tool.as_deref(), Some("devflow_doctor"));
    }

    #[test]
    fn test_sqlite_deletion() {
        let store = McpSqliteStore::open_in_memory().expect("open memory db");
        let e1 = McpAccessLogEntry::new(
            "agent",
            "tools/call",
            Some("devflow_doctor".to_string()),
            Some("sess-1".to_string()),
            None,
            None,
            None,
            10,
            "success",
            "S1",
            None,
        );
        let e2 = McpAccessLogEntry::new(
            "agent",
            "tools/call",
            Some("devflow_build".to_string()),
            Some("sess-1".to_string()),
            None,
            None,
            None,
            10,
            "success",
            "S2",
            None,
        );

        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();
        assert_eq!(store.count().unwrap(), 2);

        // Delete single by id
        let deleted = store.delete_by_id(&e1.id).unwrap();
        assert!(deleted);
        assert_eq!(store.count().unwrap(), 1);

        // Clear all
        store.clear_all().unwrap();
        assert_eq!(store.count().unwrap(), 0);
    }
}
