use crate::runner::PlatformRunner;
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_logs::LogParser;
use devflow_protocol::{Device, LogEntry};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct ApplePlatformRunner;

#[async_trait]
impl PlatformRunner for ApplePlatformRunner {
    async fn install(&self, _project_dir: &Path, device: &Device, artifact_path: Option<&str>) -> Result<()> {
        let artifact = artifact_path.ok_or_else(|| DevflowError::Install("No artifact path specified for Apple install".to_string()))?;
        info!("Installing app {} on device {}", artifact, device.id);

        if device.is_emulator {
            // Simulator install via xcrun simctl
            let output = Command::new("xcrun")
                .args(["simctl", "install", &device.id, artifact])
                .output()
                .await
                .map_err(|e| DevflowError::Install(format!("xcrun simctl install failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Install(format!("simctl install failed: {}", stderr)));
            }
        } else {
            // Physical iOS device install via xcrun devicectl (iOS 17+)
            let output = Command::new("xcrun")
                .args(["devicectl", "device", "install", "app", "--device", &device.id, artifact])
                .output()
                .await
                .map_err(|e| DevflowError::Install(format!("xcrun devicectl install failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Install(format!("devicectl install failed: {}", stderr)));
            }
        }

        Ok(())
    }

    async fn launch(&self, _project_dir: &Path, device: &Device, launch_cmd: Option<&str>) -> Result<()> {
        let bundle_id = launch_cmd.ok_or_else(|| DevflowError::Launch("No bundle id specified for launch".to_string()))?;
        info!("Launching Apple app {} on device {}", bundle_id, device.id);

        if device.is_emulator {
            let output = Command::new("xcrun")
                .args(["simctl", "launch", &device.id, bundle_id])
                .output()
                .await
                .map_err(|e| DevflowError::Launch(format!("xcrun simctl launch failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Launch(format!("simctl launch failed: {}", stderr)));
            }
        } else {
            let output = Command::new("xcrun")
                .args(["devicectl", "device", "process", "launch", "--device", &device.id, bundle_id])
                .output()
                .await
                .map_err(|e| DevflowError::Launch(format!("xcrun devicectl launch failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DevflowError::Launch(format!("devicectl launch failed: {}", stderr)));
            }
        }

        Ok(())
    }

    async fn stop(&self, _project_dir: &Path, device: &Device) -> Result<()> {
        if device.is_emulator {
            // simctl terminate if needed
        }
        Ok(())
    }

    async fn stream_logs(&self, _project_dir: &Path, device: &Device, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        let device_id = device.id.clone();
        let is_sim = device.is_emulator;

        let handle = tokio::spawn(async move {
            if is_sim {
                let mut child = match Command::new("xcrun")
                    .args(["simctl", "spawn", &device_id, "log", "stream", "--style", "compact"])
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
            }
        });

        Ok(handle)
    }
}
