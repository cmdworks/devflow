use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use crate::registry::FrameworkRegistry;
use chrono::Utc;
use devflow_core::error::{DevflowError, Result};
use devflow_core::event::{DevflowEvent, EventBus};
use devflow_core::ipc::{IpcRequest, IpcResponse, IpcServer};
use devflow_core::project::Project;
use devflow_core::registry::GlobalRegistry;
use devflow_devices::DeviceManager;
use devflow_logs::LogBuffer;
use devflow_protocol::{Device, LogEntry, LogFilter, SessionState, SessionStatus};
use devflow_watcher::{ChangeAction, FileWatcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::info;

pub struct SessionManager {
    pub project: Project,
    pub adapter: Arc<dyn FrameworkAdapter>,
    pub device: Device,
    pub state: Arc<RwLock<SessionState>>,
    pub log_buffer: LogBuffer,
    pub event_bus: EventBus,
}

impl SessionManager {
    pub async fn create_with_id(
        session_id: String,
        project_dir: impl AsRef<Path>,
        target_device_id: Option<&str>,
        framework_override: Option<&str>,
        event_bus: EventBus,
    ) -> Result<Self> {
        let mut project = Project::detect(project_dir.as_ref())?;

        if let Some(fw) = framework_override {
            let mut cfg = project.effective_config();
            cfg.project.framework = fw.to_string();
            project.config = Some(cfg);
        }

        let registry = FrameworkRegistry::new();
        let adapter = registry.select_adapter(&project);

        let device = DeviceManager::find_best_match(Some(project.detected_platform), target_device_id)
            .await
            .unwrap_or_else(Device::host_desktop);

        let state = SessionState {
            session_id: session_id.clone(),
            project_name: project.name.clone(),
            project_path: project.root_dir.to_string_lossy().to_string(),
            framework: adapter.name().to_string(),
            platform: device.platform.to_string(),
            target_device: Some(device.clone()),
            status: SessionStatus::Idle,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            last_error: None,
            build_duration_ms: None,
            reload_count: 0,
            restart_count: 0,
        };

        GlobalRegistry::record_project(&project.root_dir, &project.name, device.platform, adapter.name());

        Ok(Self {
            project,
            adapter,
            device,
            state: Arc::new(RwLock::new(state)),
            log_buffer: LogBuffer::default(),
            event_bus,
        })
    }

    pub async fn create(
        project_dir: impl AsRef<Path>,
        target_device_id: Option<&str>,
        framework_override: Option<&str>,
        event_bus: EventBus,
    ) -> Result<Self> {
        let session_id = uuid::Uuid::new_v4().to_string();
        Self::create_with_id(session_id, project_dir, target_device_id, framework_override, event_bus).await
    }

    pub fn set_status(&self, status: SessionStatus) {
        let mut state = self.state.write().unwrap();
        state.status = status;
        state.updated_at = Utc::now();
        GlobalRegistry::update_session(&state);
        self.event_bus.publish(DevflowEvent::SessionStateChanged {
            session_id: state.session_id.clone(),
            status,
            state: Box::new(state.clone()),
        });
    }

    pub fn set_last_error(&self, err: Option<String>) {
        let mut state = self.state.write().unwrap();
        state.last_error = err;
        state.updated_at = Utc::now();
    }

    pub fn get_state(&self) -> SessionState {
        self.state.read().unwrap().clone()
    }

    pub async fn start_session(self: &Arc<Self>) -> Result<()> {
        info!("Starting DevFlow session for project '{}' on device '{}'", self.project.name, self.device.name);
        self.set_status(SessionStatus::Building);

        // Register session with GlobalRegistry and start IPC server
        let sock_path = GlobalRegistry::sessions_dir().join(format!("{}.sock", self.get_state().session_id));
        GlobalRegistry::register_session(&self.get_state(), &sock_path);

        let session_self = self.clone();
        let ipc_server = IpcServer::new(&sock_path);
        let _ = ipc_server.start(move |req| {
            let s = session_self.clone();
            async move {
                match req {
                    IpcRequest::Reload { files } => {
                        let paths = files.into_iter().map(PathBuf::from).collect();
                        match s.reload(paths).await {
                            Ok(_) => IpcResponse::ok("Reload succeeded"),
                            Err(e) => IpcResponse::err(format!("Reload failed: {}", e)),
                        }
                    }
                    IpcRequest::Restart => {
                        match s.restart().await {
                            Ok(_) => IpcResponse::ok("Restart succeeded"),
                            Err(e) => IpcResponse::err(format!("Restart failed: {}", e)),
                        }
                    }
                    IpcRequest::Status => {
                        let st = s.get_state();
                        IpcResponse::ok(serde_json::to_string(&st).unwrap_or_default())
                    }
                    IpcRequest::Stop => {
                        let _ = s.stop().await;
                        IpcResponse::ok("Session stopped")
                    }
                }
            }
        }).await;

        let build_ctx = BuildContext {
            project_dir: self.project.root_dir.clone(),
            config: self.project.effective_config(),
            target_device: Some(self.device.clone()),
            is_release: false,
        };

        let session_id = self.get_state().session_id;

        self.event_bus.publish(DevflowEvent::BuildStarted {
            session_id: session_id.clone(),
            project_name: self.project.name.clone(),
        });

        let start_msg = format!("⚡ Starting build for '{}' [{}] on device '{}'...", self.project.name, self.adapter.name(), self.device.name);
        let mut start_entry = LogEntry::new(devflow_protocol::LogLevel::I, start_msg);
        start_entry.tag = Some("build".to_string());
        self.log_buffer.push(start_entry.clone());
        self.event_bus.publish(DevflowEvent::LogAppended {
            session_id: session_id.clone(),
            entry: start_entry,
        });

        let build_res = self.adapter.build(&build_ctx).await?;

        // Stream compiler stdout lines to log buffer and UI
        for line in build_res.stdout.lines() {
            if !line.trim().is_empty() {
                let mut entry = LogEntry::new(devflow_protocol::LogLevel::I, line);
                entry.tag = Some("compiler".to_string());
                self.log_buffer.push(entry.clone());
                self.event_bus.publish(DevflowEvent::LogAppended {
                    session_id: session_id.clone(),
                    entry,
                });
            }
        }

        // Stream compiler stderr lines
        for line in build_res.stderr.lines() {
            if !line.trim().is_empty() {
                let lvl = if build_res.success { devflow_protocol::LogLevel::W } else { devflow_protocol::LogLevel::E };
                let mut entry = LogEntry::new(lvl, line);
                entry.tag = Some("compiler".to_string());
                self.log_buffer.push(entry.clone());
                self.event_bus.publish(DevflowEvent::LogAppended {
                    session_id: session_id.clone(),
                    entry,
                });
            }
        }

        self.event_bus.publish(DevflowEvent::BuildCompleted {
            session_id: session_id.clone(),
            result: build_res.clone(),
        });

        if !build_res.success {
            self.set_status(SessionStatus::Failed);
            self.set_last_error(build_res.error_message.clone());
            let fail_msg = format!("✗ Build failed in {}ms: {}", build_res.duration_ms, build_res.error_message.unwrap_or_else(|| "Unknown compiler error".to_string()));
            let mut fail_entry = LogEntry::new(devflow_protocol::LogLevel::E, fail_msg);
            fail_entry.tag = Some("build".to_string());
            self.log_buffer.push(fail_entry.clone());
            self.event_bus.publish(DevflowEvent::LogAppended {
                session_id: session_id.clone(),
                entry: fail_entry,
            });
            return Err(DevflowError::Build("Build failed".to_string()));
        }

        let ok_msg = format!("✓ Build succeeded in {}ms", build_res.duration_ms);
        let mut ok_entry = LogEntry::new(devflow_protocol::LogLevel::I, ok_msg);
        ok_entry.tag = Some("build".to_string());
        self.log_buffer.push(ok_entry.clone());
        self.event_bus.publish(DevflowEvent::LogAppended {
            session_id: session_id.clone(),
            entry: ok_entry,
        });

        {
            let mut state = self.state.write().unwrap();
            state.build_duration_ms = Some(build_res.duration_ms);
        }

        // Install
        self.set_status(SessionStatus::Installing);
        let dev_ctx = DeviceContext {
            project_dir: self.project.root_dir.clone(),
            config: self.project.effective_config(),
            device: self.device.clone(),
            artifact_path: build_res.artifact.as_ref().map(|a| a.path.clone()),
        };

        if let Err(e) = self.adapter.install(&dev_ctx).await {
            self.set_status(SessionStatus::Failed);
            self.set_last_error(Some(e.to_string()));
            return Err(e);
        }

        // Launch
        self.set_status(SessionStatus::Launching);
        if let Err(e) = self.adapter.launch(&dev_ctx).await {
            self.set_status(SessionStatus::Failed);
            self.set_last_error(Some(e.to_string()));
            return Err(e);
        }

        // Stream Logs
        let (log_tx, mut log_rx) = mpsc::channel::<LogEntry>(256);
        let log_buffer = self.log_buffer.clone();
        let event_bus = self.event_bus.clone();
        let session_id = self.get_state().session_id;

        tokio::spawn(async move {
            while let Some(entry) = log_rx.recv().await {
                log_buffer.push(entry.clone());
                event_bus.publish(DevflowEvent::LogAppended {
                    session_id: session_id.clone(),
                    entry,
                });
            }
        });

        let _ = self.adapter.stream_logs(&dev_ctx, log_tx).await;

        // Start File Watcher
        let (watch_tx, mut watch_rx) = mpsc::channel(64);
        let watch_cfg = self.project.effective_config().watch.unwrap_or_default();
        let watcher = FileWatcher::new(&self.project.root_dir, watch_cfg);

        if let Ok(_w_handle) = watcher.start(watch_tx) {
            let session_clone = self.clone();
            tokio::spawn(async move {
                while let Some(change) = watch_rx.recv().await {
                    let path_strings: Vec<String> = change.paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
                    session_clone.event_bus.publish(DevflowEvent::WatcherTriggered {
                        session_id: session_clone.get_state().session_id,
                        paths: path_strings,
                        action: format!("{:?}", change.action),
                    });

                    match change.action {
                        ChangeAction::Reload => {
                            let _ = session_clone.reload(change.paths).await;
                        }
                        ChangeAction::Restart => {
                            let _ = session_clone.restart().await;
                        }
                        ChangeAction::Ignore => {}
                    }
                }
            });
        }

        self.set_status(SessionStatus::Running);
        Ok(())
    }

    pub async fn reload(&self, changed_files: Vec<PathBuf>) -> Result<()> {
        info!("Reloading session...");
        self.set_status(SessionStatus::Reloading);

        let ctx = ReloadContext {
            project_dir: self.project.root_dir.clone(),
            config: self.project.effective_config(),
            device: self.device.clone(),
            changed_files,
        };

        let res = self.adapter.reload(&ctx).await;
        match res {
            Ok(_) => {
                let mut state = self.state.write().unwrap();
                state.reload_count += 1;
                self.set_status(SessionStatus::Running);
                Ok(())
            }
            Err(e) => {
                self.set_status(SessionStatus::Running);
                self.set_last_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn restart(&self) -> Result<()> {
        info!("Restarting app...");
        self.set_status(SessionStatus::Restarting);

        let dev_ctx = DeviceContext {
            project_dir: self.project.root_dir.clone(),
            config: self.project.effective_config(),
            device: self.device.clone(),
            artifact_path: None,
        };

        let res = self.adapter.restart(&dev_ctx).await;
        match res {
            Ok(_) => {
                let mut state = self.state.write().unwrap();
                state.restart_count += 1;
                self.set_status(SessionStatus::Running);
                Ok(())
            }
            Err(e) => {
                self.set_status(SessionStatus::Running);
                self.set_last_error(Some(e.to_string()));
                Err(e)
            }
        }
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping session...");
        self.set_status(SessionStatus::Stopped);
        GlobalRegistry::unregister_session(&self.get_state().session_id);
        Ok(())
    }

    pub fn get_logs(&self, filter: &LogFilter) -> Vec<LogEntry> {
        self.log_buffer.query(filter)
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        if let Ok(state) = self.state.read() {
            GlobalRegistry::unregister_session(&state.session_id);
        }
    }
}
