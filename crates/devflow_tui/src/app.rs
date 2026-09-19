use devflow_core::doctor::DoctorEngine;
use devflow_core::event::{DevflowEvent, EventBus};
use devflow_core::ipc::{IpcClient, IpcRequest};
use devflow_core::project::{Project, ProjectTarget};
use devflow_core::registry::{ActiveSessionInfo, GlobalRegistry, KnownProject};
use devflow_devices::DeviceManager;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::{Device, DoctorReport, LogEntry, LogFilter, LogLevel};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Hub,
    Session,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubFocus {
    Targets,
    Sessions,
    Projects,
    Devices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPanel {
    Logs,
    Devices,
    Changes,
    Build,
}

pub struct TuiApp {
    pub session: Option<Arc<SessionManager>>,
    pub mode: AppMode,
    pub hub_focus: HubFocus,
    pub session_panel: SessionPanel,
    pub targets: Vec<ProjectTarget>,
    pub selected_target_idx: usize,
    pub active_sessions: Vec<ActiveSessionInfo>,
    pub selected_session_idx: usize,
    pub known_projects: Vec<KnownProject>,
    pub selected_project_idx: usize,
    pub devices: Vec<Device>,
    pub selected_device_idx: usize,
    pub logs: VecDeque<LogEntry>,
    pub changes: Vec<String>,
    pub doctor_report: Option<DoctorReport>,
    pub log_level_filter: Option<LogLevel>,
    pub should_quit: bool,
    pub auto_scroll: bool,
    pub log_scroll_offset: usize,
    pub project_dir: PathBuf,
    pub status_message: Option<String>,
}

impl TuiApp {
    /// Initialize TUI in Hub Mode (Zero-arg devflow entry)
    pub async fn new_hub(dir: PathBuf) -> Self {
        let targets = Project::discover_workspace_targets(&dir);
        let active_sessions = GlobalRegistry::list_active_sessions();
        let known_projects = GlobalRegistry::list_projects();
        let devices = DeviceManager::discover_all().await;

        let initial_focus = if !targets.is_empty() {
            HubFocus::Targets
        } else if !active_sessions.is_empty() {
            HubFocus::Sessions
        } else {
            HubFocus::Projects
        };

        Self {
            session: None,
            mode: AppMode::Hub,
            hub_focus: initial_focus,
            session_panel: SessionPanel::Logs,
            targets,
            selected_target_idx: 0,
            active_sessions,
            selected_session_idx: 0,
            known_projects,
            selected_project_idx: 0,
            devices,
            selected_device_idx: 0,
            logs: VecDeque::new(),
            changes: Vec::new(),
            doctor_report: None,
            log_level_filter: None,
            should_quit: false,
            auto_scroll: true,
            log_scroll_offset: 0,
            project_dir: dir,
            status_message: None,
        }
    }

    /// Initialize TUI directly with an active live session (`devflow dev`)
    pub async fn new(session: Arc<SessionManager>) -> Self {
        let devices = DeviceManager::discover_all().await;
        let logs = VecDeque::from(session.get_logs(&LogFilter::default()));
        let dir = session.project.root_dir.clone();
        let targets = Project::discover_workspace_targets(&dir);

        Self {
            session: Some(session),
            mode: AppMode::Session,
            hub_focus: HubFocus::Targets,
            session_panel: SessionPanel::Logs,
            targets,
            selected_target_idx: 0,
            active_sessions: GlobalRegistry::list_active_sessions(),
            selected_session_idx: 0,
            known_projects: GlobalRegistry::list_projects(),
            selected_project_idx: 0,
            devices,
            selected_device_idx: 0,
            logs,
            changes: Vec::new(),
            doctor_report: None,
            log_level_filter: None,
            should_quit: false,
            auto_scroll: true,
            log_scroll_offset: 0,
            project_dir: dir,
            status_message: None,
        }
    }

    pub fn handle_event(&mut self, event: DevflowEvent) {
        match event {
            DevflowEvent::LogAppended { entry, .. } => {
                let matches_filter = match self.log_level_filter {
                    Some(lvl) => entry.level == lvl,
                    None => true,
                };
                if matches_filter {
                    if self.logs.len() >= 1000 {
                        self.logs.pop_front();
                    }
                    self.logs.push_back(entry);
                }
            }
            DevflowEvent::WatcherTriggered { paths, action, .. } => {
                for p in paths {
                    self.changes.push(format!("[{}] {}", action, p));
                }
                if self.changes.len() > 50 {
                    let start = self.changes.len() - 50;
                    self.changes = self.changes.split_off(start);
                }
            }
            DevflowEvent::DeviceDiscovered { device } => {
                if !self.devices.iter().any(|d| d.id == device.id) {
                    self.devices.push(device);
                }
            }
            DevflowEvent::SessionStateChanged { status, .. } => {
                self.status_message = Some(format!("Session status: {}", status));
            }
            _ => {}
        }
    }

    /// Refresh cross-terminal sessions and known projects periodically
    pub fn refresh_hub(&mut self) {
        self.active_sessions = GlobalRegistry::list_active_sessions();
        self.known_projects = GlobalRegistry::list_projects();

        if self.selected_session_idx >= self.active_sessions.len() && !self.active_sessions.is_empty() {
            self.selected_session_idx = self.active_sessions.len() - 1;
        }
        if self.selected_project_idx >= self.known_projects.len() && !self.known_projects.is_empty() {
            self.selected_project_idx = self.known_projects.len() - 1;
        }
        if self.selected_target_idx >= self.targets.len() && !self.targets.is_empty() {
            self.selected_target_idx = self.targets.len() - 1;
        }
    }

    /// Launch live session for selected target
    pub async fn start_selected_target(&mut self) -> Result<(), String> {
        let Some(target) = self.targets.get(self.selected_target_idx).cloned() else {
            return Err("No target selected".to_string());
        };

        let target_device_id = self.devices.get(self.selected_device_idx).map(|d| d.id.as_str());
        let event_bus = EventBus::default();

        match SessionManager::create(&target.path, target_device_id, Some(&target.framework), event_bus).await {
            Ok(sess) => {
                let sess_arc = Arc::new(sess);
                let sess_clone = sess_arc.clone();
                tokio::spawn(async move {
                    let _ = sess_clone.start_session().await;
                });

                self.session = Some(sess_arc);
                self.mode = AppMode::Session;
                self.logs.clear();
                self.changes.clear();
                self.status_message = Some(format!("Live dev session active for {}", target.name));
                Ok(())
            }
            Err(e) => {
                self.status_message = Some(format!("Launch failed: {}", e));
                Err(e.to_string())
            }
        }
    }

    /// Reload session (local or remote via IPC)
    pub async fn reload(&mut self) {
        if self.mode == AppMode::Session {
            if let Some(ref sess) = self.session {
                let _ = sess.reload(vec![]).await;
                self.status_message = Some("Sent reload signal".to_string());
            }
        } else if let Some(active) = self.active_sessions.get(self.selected_session_idx) {
            match IpcClient::send_command(&active.socket_path, IpcRequest::Reload { files: vec![] }).await {
                Ok(resp) => self.status_message = Some(resp.message),
                Err(e) => self.status_message = Some(format!("IPC reload error: {}", e)),
            }
        }
    }

    /// Restart session (local or remote via IPC)
    pub async fn restart(&mut self) {
        if self.mode == AppMode::Session {
            if let Some(ref sess) = self.session {
                let _ = sess.restart().await;
                self.status_message = Some("Sent restart signal".to_string());
            }
        } else if let Some(active) = self.active_sessions.get(self.selected_session_idx) {
            match IpcClient::send_command(&active.socket_path, IpcRequest::Restart).await {
                Ok(resp) => self.status_message = Some(resp.message),
                Err(e) => self.status_message = Some(format!("IPC restart error: {}", e)),
            }
        }
    }

    pub async fn run_doctor(&mut self) {
        let dir = if let Some(t) = self.targets.get(self.selected_target_idx) {
            &t.path
        } else {
            &self.project_dir
        };
        let rep = DoctorEngine::run_diagnostics(dir).await;
        self.doctor_report = Some(rep);
        self.status_message = Some("Doctor diagnostics completed".to_string());
    }

    pub fn toggle_log_level(&mut self) {
        self.log_level_filter = match self.log_level_filter {
            None => Some(LogLevel::E),
            Some(LogLevel::E) => Some(LogLevel::W),
            Some(LogLevel::W) => Some(LogLevel::I),
            Some(LogLevel::I) => Some(LogLevel::D),
            Some(LogLevel::D) => None,
        };
        if let Some(ref sess) = self.session {
            let mut filter = LogFilter::default();
            if let Some(l) = self.log_level_filter {
                filter.levels = Some(vec![l]);
            }
            self.logs = VecDeque::from(sess.get_logs(&filter));
        }
    }

    pub fn next_panel(&mut self) {
        if self.mode == AppMode::Hub {
            self.hub_focus = match self.hub_focus {
                HubFocus::Targets => HubFocus::Sessions,
                HubFocus::Sessions => HubFocus::Projects,
                HubFocus::Projects => HubFocus::Devices,
                HubFocus::Devices => HubFocus::Targets,
            };
        } else {
            self.session_panel = match self.session_panel {
                SessionPanel::Logs => SessionPanel::Devices,
                SessionPanel::Devices => SessionPanel::Changes,
                SessionPanel::Changes => SessionPanel::Build,
                SessionPanel::Build => SessionPanel::Logs,
            };
        }
    }

    pub fn nav_up(&mut self) {
        if self.mode == AppMode::Hub {
            match self.hub_focus {
                HubFocus::Targets if !self.targets.is_empty() => {
                    self.selected_target_idx = self.selected_target_idx.saturating_sub(1);
                }
                HubFocus::Sessions if !self.active_sessions.is_empty() => {
                    self.selected_session_idx = self.selected_session_idx.saturating_sub(1);
                }
                HubFocus::Projects if !self.known_projects.is_empty() => {
                    self.selected_project_idx = self.selected_project_idx.saturating_sub(1);
                }
                HubFocus::Devices if !self.devices.is_empty() => {
                    self.selected_device_idx = self.selected_device_idx.saturating_sub(1);
                }
                _ => {}
            }
        } else {
            self.scroll_up();
        }
    }

    pub fn nav_down(&mut self) {
        if self.mode == AppMode::Hub {
            match self.hub_focus {
                HubFocus::Targets if !self.targets.is_empty() => {
                    if self.selected_target_idx + 1 < self.targets.len() {
                        self.selected_target_idx += 1;
                    }
                }
                HubFocus::Sessions if !self.active_sessions.is_empty() => {
                    if self.selected_session_idx + 1 < self.active_sessions.len() {
                        self.selected_session_idx += 1;
                    }
                }
                HubFocus::Projects if !self.known_projects.is_empty() => {
                    if self.selected_project_idx + 1 < self.known_projects.len() {
                        self.selected_project_idx += 1;
                    }
                }
                HubFocus::Devices if !self.devices.is_empty() => {
                    if self.selected_device_idx + 1 < self.devices.len() {
                        self.selected_device_idx += 1;
                    }
                }
                _ => {}
            }
        } else {
            self.scroll_down();
        }
    }

    pub fn scroll_up(&mut self) {
        self.auto_scroll = false;
        self.log_scroll_offset = self.log_scroll_offset.saturating_add(1);
    }

    pub fn scroll_down(&mut self) {
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset -= 1;
            if self.log_scroll_offset == 0 {
                self.auto_scroll = true;
            }
        }
    }
}
