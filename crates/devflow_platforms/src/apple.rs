use crate::runner::PlatformRunner;
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_logs::LogParser;
use devflow_protocol::{Device, LogEntry, LogLevel, Platform};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

pub struct ApplePlatformRunner {
    active_child: Arc<Mutex<Option<Child>>>,
    active_bundle_id: Arc<Mutex<Option<String>>>,
}

impl ApplePlatformRunner {
    pub fn new() -> Self {
        Self {
            active_child: Arc::new(Mutex::new(None)),
            active_bundle_id: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for ApplePlatformRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformRunner for ApplePlatformRunner {
    async fn install(
        &self,
        _project_dir: &Path,
        device: &Device,
        artifact_path: Option<&str>,
    ) -> Result<()> {
        let artifact = artifact_path.ok_or_else(|| {
            DevflowError::Install("No artifact path specified for Apple install".to_string())
        })?;
        info!("Installing app {} on device {}", artifact, device.id);

        if device.is_emulator {
            // Simulator install via xcrun simctl
            let output = Command::new("xcrun")
                .args(["simctl", "install", &device.id, artifact])
                .output()
                .await
                .map_err(|e| {
                    DevflowError::Install(format!("xcrun simctl install failed: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Install(format!(
                    "simctl install failed: {}",
                    stderr
                )));
            }
        } else if device.platform == Platform::Ios || device.platform == Platform::Apple {
            // Physical iOS device install via xcrun devicectl (iOS 17+)
            let output = Command::new("xcrun")
                .args([
                    "devicectl",
                    "device",
                    "install",
                    "app",
                    "--device",
                    &device.id,
                    artifact,
                ])
                .output()
                .await
                .map_err(|e| {
                    DevflowError::Install(format!("xcrun devicectl install failed: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Install(format!(
                    "devicectl install failed: {}",
                    stderr
                )));
            }
        }

        Ok(())
    }

    async fn launch(
        &self,
        project_dir: &Path,
        device: &Device,
        launch_cmd: Option<&str>,
    ) -> Result<()> {
        self.stop(project_dir, device).await?;

        let launch_arg = launch_cmd.ok_or_else(|| {
            DevflowError::Launch("No bundle id or app path specified for launch".to_string())
        })?;
        info!(
            "Launching Apple app {} on device {} ({})",
            launch_arg, device.name, device.platform
        );

        let mut b_lock = self.active_bundle_id.lock().await;
        *b_lock = Some(launch_arg.to_string());

        if device.is_emulator {
            let output = Command::new("xcrun")
                .args(["simctl", "launch", &device.id, launch_arg])
                .output()
                .await
                .map_err(|e| DevflowError::Launch(format!("xcrun simctl launch failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Launch(format!(
                    "simctl launch failed: {}",
                    stderr
                )));
            }
        } else if device.platform == Platform::Ios || device.platform == Platform::Apple {
            let output = Command::new("xcrun")
                .args([
                    "devicectl",
                    "device",
                    "process",
                    "launch",
                    "--device",
                    &device.id,
                    launch_arg,
                ])
                .output()
                .await
                .map_err(|e| {
                    DevflowError::Launch(format!("xcrun devicectl launch failed: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Launch(format!(
                    "devicectl launch failed: {}",
                    stderr
                )));
            }
        } else {
            // macOS Desktop target:
            let app_path = Path::new(launch_arg);
            let executable = if app_path.extension().map(|e| e == "app").unwrap_or(false) {
                let macos_dir = app_path.join("Contents/MacOS");
                if let Ok(entries) = std::fs::read_dir(&macos_dir) {
                    let mut found = None;
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() {
                            found = Some(p);
                            break;
                        }
                    }
                    found.unwrap_or_else(|| app_path.to_path_buf())
                } else {
                    app_path.to_path_buf()
                }
            } else if app_path.is_relative() && project_dir.join(app_path).exists() {
                project_dir.join(app_path)
            } else {
                app_path.to_path_buf()
            };

            let child = Command::new(&executable)
                .current_dir(project_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    DevflowError::Launch(format!(
                        "Failed to spawn macOS app '{}': {}",
                        executable.display(),
                        e
                    ))
                })?;

            let mut c_lock = self.active_child.lock().await;
            *c_lock = Some(child);
        }

        Ok(())
    }

    async fn stop(&self, _project_dir: &Path, device: &Device) -> Result<()> {
        let bundle_id = self.active_bundle_id.lock().await.take();

        if device.is_emulator {
            if let Some(ref bid) = bundle_id {
                debug!("Terminating simulator app: {} on {}", bid, device.id);
                let _ = Command::new("xcrun")
                    .args(["simctl", "terminate", &device.id, bid])
                    .output()
                    .await;
            }
        } else if device.platform == Platform::Ios || device.platform == Platform::Apple {
            if let Some(ref bid) = bundle_id {
                let _ = Command::new("xcrun")
                    .args([
                        "devicectl",
                        "device",
                        "process",
                        "terminate",
                        "--device",
                        &device.id,
                        bid,
                    ])
                    .output()
                    .await;
            }
        }

        let mut lock = self.active_child.lock().await;
        if let Some(mut child) = lock.take() {
            debug!("Stopping running macOS process...");
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                let _ = Command::new("pkill")
                    .arg("-9")
                    .arg("-P")
                    .arg(pid.to_string())
                    .output()
                    .await;
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        let dir_str = _project_dir.to_string_lossy();
        if !dir_str.is_empty() {
            #[cfg(unix)]
            {
                let _ = Command::new("pkill")
                    .arg("-9")
                    .arg("-f")
                    .arg(format!("{}/.build", dir_str))
                    .output()
                    .await;
            }
        }

        Ok(())
    }

    async fn stream_logs(
        &self,
        _project_dir: &Path,
        device: &Device,
        tx: mpsc::Sender<LogEntry>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let device_id = device.id.clone();
        let is_sim = device.is_emulator;

        let mut lock = self.active_child.lock().await;
        let local_stdout = lock.as_mut().and_then(|c| c.stdout.take());
        let local_stderr = lock.as_mut().and_then(|c| c.stderr.take());

        let handle = tokio::spawn(async move {
            if is_sim {
                let mut child = match Command::new("xcrun")
                    .args([
                        "simctl", "spawn", &device_id, "log", "stream", "--style", "compact",
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(e) => {
                        warn!("Failed to spawn simctl log stream: {}", e);
                        return;
                    }
                };

                if let Some(stdout) = child.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let entry = LogParser::parse_line(&line, Some("simctl"));
                        if tx.send(entry).await.is_err() {
                            break;
                        }
                    }
                }

                let _ = child.kill().await;
            } else if local_stdout.is_some() || local_stderr.is_some() {
                let tx_err = tx.clone();
                let stdout_task = tokio::spawn(async move {
                    if let Some(out) = local_stdout {
                        let mut reader = BufReader::new(out).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let entry = LogParser::parse_line(&line, Some("stdout"));
                            if tx.send(entry).await.is_err() {
                                break;
                            }
                        }
                    }
                });

                let stderr_task = tokio::spawn(async move {
                    if let Some(err) = local_stderr {
                        let mut reader = BufReader::new(err).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let mut entry = LogParser::parse_line(&line, Some("stderr"));
                            if entry.level == LogLevel::I {
                                entry.level = LogLevel::E;
                            }
                            if tx_err.send(entry).await.is_err() {
                                break;
                            }
                        }
                    }
                });

                let _ = tokio::join!(stdout_task, stderr_task);
            }
        });

        Ok(handle)
    }
}
