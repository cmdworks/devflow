use devflow_core::doctor::DoctorEngine;
use devflow_core::event::{DevflowEvent, EventBus};
use devflow_core::ipc::{IpcClient, IpcRequest};
use devflow_core::project::{Project, ProjectTarget};
use devflow_core::registry::{ActiveSessionInfo, GlobalRegistry, KnownProject};
use devflow_devices::DeviceManager;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::{Device, DoctorReport, LogEntry, LogFilter, LogLevel, Platform};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetStatus {
    Idle,
    Building,
    Running,
    Error(String),
    Stopped,
}

impl std::fmt::Display for TargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetStatus::Idle => write!(f, "Idle"),
            TargetStatus::Building => write!(f, "Building..."),
            TargetStatus::Running => write!(f, "Running"),
            TargetStatus::Error(e) => write!(f, "Error: {}", e),
            TargetStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

pub struct TargetSessionState {
    pub target: ProjectTarget,
    pub session: Option<Arc<SessionManager>>,
    pub status: TargetStatus,
    pub logs: VecDeque<LogEntry>,
    pub log_scroll_offset: usize,
    pub auto_scroll: bool,
    pub log_level_filter: Option<LogLevel>,
    pub changes: Vec<String>,
    pub last_build_duration_ms: Option<u64>,
    pub last_error: Option<String>,
    pub assigned_device: Option<Device>,
}

impl TargetSessionState {
    pub fn new(target: ProjectTarget) -> Self {
        Self {
            target,
            session: None,
            status: TargetStatus::Idle,
            logs: VecDeque::with_capacity(1000),
            log_scroll_offset: 0,
            auto_scroll: true,
            log_level_filter: None,
            changes: Vec::new(),
            last_build_duration_ms: None,
            last_error: None,
            assigned_device: None,
        }
    }
}

pub struct TuiApp {
    pub mode: AppMode,
    pub hub_focus: HubFocus,
    pub session_panel: SessionPanel,
    pub targets: Vec<ProjectTarget>,
    pub selected_target_idx: usize,
    pub target_states: Vec<TargetSessionState>,
    pub active_tab_idx: usize, // 0..N-1 are target tabs, N is Combined view
    pub combined_logs: VecDeque<LogEntry>,
    pub combined_scroll_offset: usize,
    pub combined_auto_scroll: bool,
    pub combined_level_filter: Option<LogLevel>,
    pub active_sessions: Vec<ActiveSessionInfo>,
    pub selected_session_idx: usize,
    pub known_projects: Vec<KnownProject>,
    pub selected_project_idx: usize,
    pub devices: Vec<Device>,
    pub selected_device_idx: usize,
    pub doctor_report: Option<DoctorReport>,
    pub should_quit: bool,
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

        let target_states = targets.iter().map(|t| TargetSessionState::new(t.clone())).collect();

        let initial_focus = if !targets.is_empty() {
            HubFocus::Targets
        } else if !active_sessions.is_empty() {
            HubFocus::Sessions
        } else {
            HubFocus::Projects
        };

        Self {
            mode: AppMode::Hub,
            hub_focus: initial_focus,
            session_panel: SessionPanel::Logs,
            targets,
            selected_target_idx: 0,
            target_states,
            active_tab_idx: 0,
            combined_logs: VecDeque::with_capacity(2000),
            combined_scroll_offset: 0,
            combined_auto_scroll: true,
            combined_level_filter: None,
            active_sessions,
            selected_session_idx: 0,
            known_projects,
            selected_project_idx: 0,
            devices,
            selected_device_idx: 0,
            doctor_report: None,
            should_quit: false,
            project_dir: dir,
            status_message: None,
        }
    }

    /// Initialize TUI directly attached to an active live session (`devflow dev`)
    pub async fn new(session: Arc<SessionManager>) -> Self {
        let devices = DeviceManager::discover_all().await;
        let dir = session.project.root_dir.clone();
        let targets = Project::discover_workspace_targets(&dir);

        let mut target_states: Vec<TargetSessionState> = targets.iter().map(|t| TargetSessionState::new(t.clone())).collect();

        // Attach the passed session to matching target or create target state
        if let Some(state) = target_states.iter_mut().find(|s| s.target.name == session.project.name || s.target.path == session.project.root_dir) {
            state.session = Some(session.clone());
            state.status = TargetStatus::Running;
            state.logs = VecDeque::from(session.get_logs(&LogFilter::default()));
        } else if !target_states.is_empty() {
            target_states[0].session = Some(session.clone());
            target_states[0].status = TargetStatus::Running;
            target_states[0].logs = VecDeque::from(session.get_logs(&LogFilter::default()));
        }

        Self {
            mode: AppMode::Session,
            hub_focus: HubFocus::Targets,
            session_panel: SessionPanel::Logs,
            targets,
            selected_target_idx: 0,
            target_states,
            active_tab_idx: 0,
            combined_logs: VecDeque::from(session.get_logs(&LogFilter::default())),
            combined_scroll_offset: 0,
            combined_auto_scroll: true,
            combined_level_filter: None,
            active_sessions: GlobalRegistry::list_active_sessions(),
            selected_session_idx: 0,
            known_projects: GlobalRegistry::list_projects(),
            selected_project_idx: 0,
            devices,
            selected_device_idx: 0,
            doctor_report: None,
            should_quit: false,
            project_dir: dir,
            status_message: None,
        }
    }

    /// Handle an event received from a specific target's session event bus
    pub fn handle_target_event(&mut self, target_idx: usize, event: DevflowEvent) {
        let target_name = self.target_states.get(target_idx).map(|s| s.target.name.clone()).unwrap_or_else(|| format!("target-{}", target_idx));

        match event {
            DevflowEvent::LogAppended { entry, .. } => {
                // Route to individual target buffer
                if let Some(state) = self.target_states.get_mut(target_idx) {
                    let matches_filter = match state.log_level_filter {
                        Some(lvl) => entry.level == lvl,
                        None => true,
                    };
                    if matches_filter {
                        if state.logs.len() >= 1000 {
                            state.logs.pop_front();
                        }
                        state.logs.push_back(entry.clone());
                    }
                }

                // Also append to combined log buffer with target tag
                let mut combined_entry = entry;
                if combined_entry.tag.is_none() {
                    combined_entry.tag = Some(target_name);
                }
                let matches_comb = match self.combined_level_filter {
                    Some(lvl) => combined_entry.level == lvl,
                    None => true,
                };
                if matches_comb {
                    if self.combined_logs.len() >= 2000 {
                        self.combined_logs.pop_front();
                    }
                    self.combined_logs.push_back(combined_entry);
                }
            }
            DevflowEvent::WatcherTriggered { paths, action, .. } => {
                if let Some(state) = self.target_states.get_mut(target_idx) {
                    for p in paths {
                        state.changes.push(format!("[{}] {}", action, p));
                    }
                    if state.changes.len() > 50 {
                        let start = state.changes.len() - 50;
                        state.changes = state.changes.split_off(start);
                    }
                }
            }
            DevflowEvent::BuildCompleted { result, .. } => {
                if let Some(state) = self.target_states.get_mut(target_idx) {
                    state.last_build_duration_ms = Some(result.duration_ms);
                    if result.success {
                        state.status = TargetStatus::Running;
                        state.last_error = None;
                    } else {
                        state.status = TargetStatus::Error(result.error_message.unwrap_or_else(|| "Build failed".to_string()));
                    }
                }
            }
            DevflowEvent::SessionStateChanged { status, state: sess_state, .. } => {
                if let Some(state) = self.target_states.get_mut(target_idx) {
                    state.status = match status {
                        devflow_protocol::SessionStatus::Building | devflow_protocol::SessionStatus::Installing | devflow_protocol::SessionStatus::Launching => TargetStatus::Building,
                        devflow_protocol::SessionStatus::Running => TargetStatus::Running,
                        devflow_protocol::SessionStatus::Failed => TargetStatus::Error(sess_state.last_error.clone().unwrap_or_else(|| "Failed".to_string())),
                        devflow_protocol::SessionStatus::Stopped => TargetStatus::Stopped,
                        _ => TargetStatus::Idle,
                    };
                    state.last_error = sess_state.last_error;
                }
            }
            DevflowEvent::DeviceDiscovered { device } => {
                if !self.devices.iter().any(|d| d.id == device.id) {
                    self.devices.push(device);
                }
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

    /// Select active tab by index (0..N-1: Target tabs, N: Combined view)
    pub fn select_tab(&mut self, idx: usize) {
        let max_tab = self.target_states.len(); // inclusive of combined tab
        if idx <= max_tab {
            self.active_tab_idx = idx;
        }
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        let total_tabs = self.target_states.len() + 1; // targets + combined
        if total_tabs > 0 {
            self.active_tab_idx = (self.active_tab_idx + 1) % total_tabs;
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        let total_tabs = self.target_states.len() + 1;
        if total_tabs > 0 {
            self.active_tab_idx = if self.active_tab_idx == 0 {
                total_tabs - 1
            } else {
                self.active_tab_idx - 1
            };
        }
    }

    /// Start target session by index with platform-aware device pairing
    pub async fn start_target(&mut self, idx: usize) -> Result<tokio::sync::broadcast::Receiver<DevflowEvent>, String> {
        let Some(state) = self.target_states.get_mut(idx) else {
            return Err("Invalid target index".to_string());
        };

        // Find best device matching target platform
        let target_platform = state.target.platform;
        let matched_device = self.devices.iter().find(|d| {
            let is_available = matches!(d.state, devflow_protocol::DeviceState::Connected | devflow_protocol::DeviceState::Booted);
            is_available && match target_platform {
                Platform::Android => d.platform == Platform::Android,
                Platform::Apple | Platform::Macos | Platform::Ios => d.platform == Platform::Apple || d.platform == Platform::Desktop || d.platform == Platform::Macos,
                Platform::Desktop => d.platform == Platform::Desktop,
                _ => true,
            }
        }).or_else(|| self.devices.first()).cloned();

        state.assigned_device = matched_device.clone();
        let target_device_id = matched_device.map(|d| d.id);
        let event_bus = EventBus::default();
        let rx = event_bus.subscribe();

        state.status = TargetStatus::Building;

        match SessionManager::create(&state.target.path, target_device_id.as_deref(), Some(&state.target.framework), event_bus).await {
            Ok(sess) => {
                let sess_arc = Arc::new(sess);
                let sess_clone = sess_arc.clone();
                tokio::spawn(async move {
                    let _ = sess_clone.start_session().await;
                });

                state.session = Some(sess_arc);
                self.status_message = Some(format!("Started live dev session for '{}'", state.target.name));
                Ok(rx)
            }
            Err(e) => {
                state.status = TargetStatus::Error(e.to_string());
                self.status_message = Some(format!("Failed to start '{}': {}", state.target.name, e));
                Err(e.to_string())
            }
        }
    }

    /// Start selected target from Hub view and transition to Session mode
    pub async fn start_selected_target(&mut self) -> Result<(usize, tokio::sync::broadcast::Receiver<DevflowEvent>), String> {
        let idx = self.selected_target_idx;
        let rx = self.start_target(idx).await?;
        self.mode = AppMode::Session;
        self.active_tab_idx = idx;
        Ok((idx, rx))
    }

    /// Start ALL targets in parallel (e.g. run both Mac app and Android companion)
    pub async fn start_all_targets(&mut self) -> Vec<(usize, tokio::sync::broadcast::Receiver<DevflowEvent>)> {
        let mut launched = Vec::new();
        for idx in 0..self.target_states.len() {
            if self.target_states[idx].session.is_none() {
                if let Ok(rx) = self.start_target(idx).await {
                    launched.push((idx, rx));
                }
            }
        }
        self.mode = AppMode::Session;
        self.status_message = Some(format!("Running all {} workspace targets", self.target_states.len()));
        launched
    }

    /// Stop target by index
    pub async fn stop_target(&mut self, idx: usize) {
        if let Some(state) = self.target_states.get_mut(idx) {
            if let Some(sess) = state.session.take() {
                let _ = sess.stop().await;
            }
            state.status = TargetStatus::Stopped;
            self.status_message = Some(format!("Stopped '{}'", state.target.name));
        }
    }

    /// Stop all targets
    pub async fn stop_all_targets(&mut self) {
        for state in &mut self.target_states {
            if let Some(sess) = state.session.take() {
                let _ = sess.stop().await;
            }
            state.status = TargetStatus::Stopped;
        }
        self.status_message = Some("All targets stopped".to_string());
    }

    /// Reload active target (or all if combined)
    pub async fn reload(&mut self) {
        if self.mode == AppMode::Session {
            if self.active_tab_idx < self.target_states.len() {
                let state = &self.target_states[self.active_tab_idx];
                if let Some(ref sess) = state.session {
                    let _ = sess.reload(vec![]).await;
                    self.status_message = Some(format!("Sent reload to '{}'", state.target.name));
                }
            } else {
                for state in &self.target_states {
                    if let Some(ref sess) = state.session {
                        let _ = sess.reload(vec![]).await;
                    }
                }
                self.status_message = Some("Sent reload to all targets".to_string());
            }
        } else if let Some(active) = self.active_sessions.get(self.selected_session_idx) {
            match IpcClient::send_command(&active.socket_path, IpcRequest::Reload { files: vec![] }).await {
                Ok(resp) => self.status_message = Some(resp.message),
                Err(e) => self.status_message = Some(format!("IPC reload error: {}", e)),
            }
        }
    }

    /// Restart active target (or all if combined)
    pub async fn restart(&mut self) {
        if self.mode == AppMode::Session {
            if self.active_tab_idx < self.target_states.len() {
                let state = &self.target_states[self.active_tab_idx];
                if let Some(ref sess) = state.session {
                    let _ = sess.restart().await;
                    self.status_message = Some(format!("Sent restart to '{}'", state.target.name));
                }
            } else {
                for state in &self.target_states {
                    if let Some(ref sess) = state.session {
                        let _ = sess.restart().await;
                    }
                }
                self.status_message = Some("Sent restart to all targets".to_string());
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
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.log_level_filter = match state.log_level_filter {
                None => Some(LogLevel::E),
                Some(LogLevel::E) => Some(LogLevel::W),
                Some(LogLevel::W) => Some(LogLevel::I),
                Some(LogLevel::I) => Some(LogLevel::D),
                Some(LogLevel::D) => None,
            };
            if let Some(ref sess) = state.session {
                let mut filter = LogFilter::default();
                if let Some(l) = state.log_level_filter {
                    filter.levels = Some(vec![l]);
                }
                state.logs = VecDeque::from(sess.get_logs(&filter));
            }
        } else {
            self.combined_level_filter = match self.combined_level_filter {
                None => Some(LogLevel::E),
                Some(LogLevel::E) => Some(LogLevel::W),
                Some(LogLevel::W) => Some(LogLevel::I),
                Some(LogLevel::I) => Some(LogLevel::D),
                Some(LogLevel::D) => None,
            };
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
            self.scroll_up(1);
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
            self.scroll_down(1);
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.auto_scroll = false;
            state.log_scroll_offset = state.log_scroll_offset.saturating_add(amount);
        } else {
            self.combined_auto_scroll = false;
            self.combined_scroll_offset = self.combined_scroll_offset.saturating_add(amount);
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.log_scroll_offset = state.log_scroll_offset.saturating_sub(amount);
            if state.log_scroll_offset == 0 {
                state.auto_scroll = true;
            }
        } else {
            self.combined_scroll_offset = self.combined_scroll_offset.saturating_sub(amount);
            if self.combined_scroll_offset == 0 {
                self.combined_auto_scroll = true;
            }
        }
    }

    pub fn scroll_to_top(&mut self) {
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.auto_scroll = false;
            state.log_scroll_offset = state.logs.len();
        } else {
            self.combined_auto_scroll = false;
            self.combined_scroll_offset = self.combined_logs.len();
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.log_scroll_offset = 0;
            state.auto_scroll = true;
        } else {
            self.combined_scroll_offset = 0;
            self.combined_auto_scroll = true;
        }
    }

    pub fn toggle_autoscroll(&mut self) {
        if self.active_tab_idx < self.target_states.len() {
            let state = &mut self.target_states[self.active_tab_idx];
            state.auto_scroll = !state.auto_scroll;
            if state.auto_scroll {
                state.log_scroll_offset = 0;
            }
        } else {
            self.combined_auto_scroll = !self.combined_auto_scroll;
            if self.combined_auto_scroll {
                self.combined_scroll_offset = 0;
            }
        }
    }
}
