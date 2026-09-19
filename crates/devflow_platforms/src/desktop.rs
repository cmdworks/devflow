use crate::runner::PlatformRunner;
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_logs::LogParser;
use devflow_protocol::{Device, LogEntry, LogLevel};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};

pub struct DesktopPlatformRunner {
    active_child: Arc<Mutex<Option<Child>>>,
}

impl DesktopPlatformRunner {
    pub fn new() -> Self {
        Self {
            active_child: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for DesktopPlatformRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformRunner for DesktopPlatformRunner {
    async fn install(&self, _project_dir: &Path, _device: &Device, _artifact_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn launch(&self, project_dir: &Path, _device: &Device, launch_cmd: Option<&str>) -> Result<()> {
        self.stop(project_dir, _device).await?;

        let cmd_str = launch_cmd.ok_or_else(|| DevflowError::Launch("No launch command specified for desktop target".to_string()))?;
        info!("Launching desktop process: {}", cmd_str);

        let mut parts = cmd_str.split_whitespace();
        let program = parts.next().ok_or_else(|| DevflowError::Launch("Empty launch command".to_string()))?;
        let args: Vec<&str> = parts.collect();

        let resolved_program = if std::path::Path::new(program).is_relative() && project_dir.join(program).exists() {
            project_dir.join(program)
        } else {
            std::path::PathBuf::from(program)
        };

        let child = Command::new(&resolved_program)
            .args(&args)
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| DevflowError::Launch(format!("Failed to spawn process '{}' ({}): {}", cmd_str, resolved_program.display(), e)))?;

        let mut lock = self.active_child.lock().await;
        *lock = Some(child);

        Ok(())
    }

    async fn stop(&self, _project_dir: &Path, _device: &Device) -> Result<()> {
        let mut lock = self.active_child.lock().await;
        if let Some(mut child) = lock.take() {
            debug!("Stopping running desktop process...");
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        Ok(())
    }

    async fn stream_logs(&self, _project_dir: &Path, _device: &Device, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        let mut lock = self.active_child.lock().await;
        let child = match lock.as_mut() {
            Some(c) => c,
            None => {
                return Ok(tokio::spawn(async {}));
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let handle = tokio::spawn(async move {
            let tx_err = tx.clone();

            let stdout_task = tokio::spawn(async move {
                if let Some(out) = stdout {
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
                if let Some(err) = stderr {
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
        });

        Ok(handle)
    }
}
