use chrono::{DateTime, Utc};
use devflow_protocol::{Device, Platform, SessionState, SessionStatus};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownProject {
    pub name: String,
    pub path: String,
    pub platform: Platform,
    pub framework: String,
    pub last_opened: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSessionInfo {
    pub session_id: String,
    pub pid: u32,
    pub project_name: String,
    pub project_path: String,
    pub framework: String,
    pub platform: String,
    pub target_device: Option<Device>,
    pub status: SessionStatus,
    pub socket_path: String,
    pub build_duration_ms: Option<u64>,
    pub reload_count: u32,
    pub restart_count: u32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct GlobalRegistry;

impl GlobalRegistry {
    pub fn root_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("DEVFLOW_HOME") {
            PathBuf::from(dir)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".devflow")
        } else {
            std::env::temp_dir().join(".devflow")
        }
    }

    pub fn sessions_dir() -> PathBuf {
        Self::root_dir().join("sessions")
    }

    pub fn projects_file() -> PathBuf {
        Self::root_dir().join("projects.json")
    }

    pub fn ensure_dirs() {
        let _ = fs::create_dir_all(Self::sessions_dir());
    }

    /// Record a project in the global known projects registry.
    pub fn record_project(path: &Path, name: &str, platform: Platform, framework: &str) {
        Self::ensure_dirs();
        let file_path = Self::projects_file();
        let mut projects = Self::list_projects();

        let path_str = path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).to_string_lossy().to_string();

        if let Some(existing) = projects.iter_mut().find(|p| p.path == path_str) {
            existing.name = name.to_string();
            existing.platform = platform;
            existing.framework = framework.to_string();
            existing.last_opened = Utc::now();
        } else {
            projects.push(KnownProject {
                name: name.to_string(),
                path: path_str,
                platform,
                framework: framework.to_string(),
                last_opened: Utc::now(),
            });
        }

        // Sort by most recently opened
        projects.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
        if projects.len() > 50 {
            projects.truncate(50);
        }

        if let Ok(json) = serde_json::to_string_pretty(&projects) {
            let _ = fs::write(&file_path, json);
        }
    }

    /// List all known projects across the machine, filtering out deleted paths.
    pub fn list_projects() -> Vec<KnownProject> {
        let file_path = Self::projects_file();
        if !file_path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => match serde_json::from_str::<Vec<KnownProject>>(&content) {
                Ok(list) => list.into_iter().filter(|p| Path::new(&p.path).exists()).collect(),
                Err(e) => {
                    warn!("Failed to parse projects.json: {}", e);
                    Vec::new()
                }
            },
            Err(_) => Vec::new(),
        }
    }

    /// Register an active live session into ~/.devflow/sessions/<id>.json
    pub fn register_session(state: &SessionState, socket_path: &Path) {
        Self::ensure_dirs();
        let session_file = Self::sessions_dir().join(format!("{}.json", state.session_id));
        let pid = std::process::id();

        let info = ActiveSessionInfo {
            session_id: state.session_id.clone(),
            pid,
            project_name: state.project_name.clone(),
            project_path: state.project_path.clone(),
            framework: state.framework.clone(),
            platform: state.platform.clone(),
            target_device: state.target_device.clone(),
            status: state.status,
            socket_path: socket_path.to_string_lossy().to_string(),
            build_duration_ms: state.build_duration_ms,
            reload_count: state.reload_count,
            restart_count: state.restart_count,
            started_at: state.started_at,
            updated_at: state.updated_at,
        };

        if let Ok(json) = serde_json::to_string_pretty(&info) {
            let _ = fs::write(&session_file, json);
            debug!("Registered active session {} at {}", state.session_id, session_file.display());
        }
    }

    /// Update status of an active session.
    pub fn update_session(state: &SessionState) {
        let session_file = Self::sessions_dir().join(format!("{}.json", state.session_id));
        if !session_file.exists() {
            return;
        }

        if let Ok(content) = fs::read_to_string(&session_file) {
            if let Ok(mut info) = serde_json::from_str::<ActiveSessionInfo>(&content) {
                info.status = state.status;
                info.build_duration_ms = state.build_duration_ms;
                info.reload_count = state.reload_count;
                info.restart_count = state.restart_count;
                info.target_device = state.target_device.clone();
                info.updated_at = state.updated_at;

                if let Ok(json) = serde_json::to_string_pretty(&info) {
                    let _ = fs::write(&session_file, json);
                }
            }
        }
    }

    /// Unregister a session and delete its socket and metadata file.
    pub fn unregister_session(session_id: &str) {
        let session_file = Self::sessions_dir().join(format!("{}.json", session_id));
        let socket_file = Self::sessions_dir().join(format!("{}.sock", session_id));

        let _ = fs::remove_file(session_file);
        let _ = fs::remove_file(socket_file);
        debug!("Unregistered session {}", session_id);
    }

    /// List all currently running active sessions across terminals, auto-pruning dead PIDs.
    pub fn list_active_sessions() -> Vec<ActiveSessionInfo> {
        Self::ensure_dirs();
        let sessions_dir = Self::sessions_dir();
        let mut active = Vec::new();

        let entries = match fs::read_dir(&sessions_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(info) = serde_json::from_str::<ActiveSessionInfo>(&content) {
                        if Self::is_pid_alive(info.pid) {
                            active.push(info);
                        } else {
                            // Process is dead: prune stale session files
                            debug!("Pruning stale session {} (PID {} not running)", info.session_id, info.pid);
                            let _ = fs::remove_file(&path);
                            let sock_path = PathBuf::from(&info.socket_path);
                            if sock_path.exists() {
                                let _ = fs::remove_file(sock_path);
                            }
                        }
                    }
                }
            }
        }

        active.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        active
    }

    /// Find an active session matching the current directory, a target name, or the sole active session.
    pub fn find_active_session(target_or_path: Option<&str>) -> Option<ActiveSessionInfo> {
        let sessions = Self::list_active_sessions();
        if sessions.is_empty() {
            return None;
        }

        if let Some(target) = target_or_path {
            let target_lower = target.to_lowercase();
            // Check by session ID prefix
            if let Some(s) = sessions.iter().find(|s| s.session_id.starts_with(&target_lower)) {
                return Some(s.clone());
            }
            // Check by project name
            if let Some(s) = sessions.iter().find(|s| s.project_name.to_lowercase().contains(&target_lower)) {
                return Some(s.clone());
            }
            // Check by path
            if let Some(s) = sessions.iter().find(|s| s.project_path.to_lowercase().contains(&target_lower)) {
                return Some(s.clone());
            }
        } else {
            // Try matching current directory
            if let Ok(current_dir) = std::env::current_dir() {
                let current_str = current_dir.to_string_lossy().to_string();
                if let Some(s) = sessions.iter().find(|s| s.project_path == current_str) {
                    return Some(s.clone());
                }
            }
            // If only one session is running on the whole system, default to it
            if sessions.len() == 1 {
                return Some(sessions[0].clone());
            }
        }

        None
    }

    /// Check if process PID is currently alive on the system.
    pub fn is_pid_alive(pid: u32) -> bool {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .args(["-0", &pid.to_string()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_liveness() {
        let my_pid = std::process::id();
        assert!(GlobalRegistry::is_pid_alive(my_pid));
        assert!(!GlobalRegistry::is_pid_alive(99999999));
    }
}
