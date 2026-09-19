use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Idle,
    Detecting,
    Building,
    Installing,
    Launching,
    Running,
    Reloading,
    Restarting,
    Stopped,
    Failed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Idle => write!(f, "Idle"),
            SessionStatus::Detecting => write!(f, "Detecting"),
            SessionStatus::Building => write!(f, "Building"),
            SessionStatus::Installing => write!(f, "Installing"),
            SessionStatus::Launching => write!(f, "Launching"),
            SessionStatus::Running => write!(f, "Running"),
            SessionStatus::Reloading => write!(f, "Reloading"),
            SessionStatus::Restarting => write!(f, "Restarting"),
            SessionStatus::Stopped => write!(f, "Stopped"),
            SessionStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub project_name: String,
    pub project_path: String,
    pub framework: String,
    pub platform: String,
    pub target_device: Option<crate::device::Device>,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_duration_ms: Option<u64>,
    #[serde(default)]
    pub reload_count: u32,
    #[serde(default)]
    pub restart_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Build,
    Install,
    Launch,
    Reload,
    Restart,
    Stop,
    Doctor,
}
