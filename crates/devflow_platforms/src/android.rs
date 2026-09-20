use crate::runner::PlatformRunner;
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_logs::LogParser;
use devflow_protocol::{Device, LogEntry};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

pub struct AndroidPlatformRunner {
    active_package: Arc<Mutex<Option<String>>>,
}

impl AndroidPlatformRunner {
    pub fn new() -> Self {
        Self {
            active_package: Arc::new(Mutex::new(None)),
        }
    }

    pub fn resolve_adb() -> String {
        if std::process::Command::new("adb").arg("version").output().is_ok() {
            return "adb".to_string();
        }
        if let Ok(home) = std::env::var("ANDROID_HOME") {
            let p = std::path::Path::new(&home).join("platform-tools").join("adb");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let p = std::path::Path::new(&home).join("Library/Android/sdk/platform-tools/adb");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        "adb".to_string()
    }
}

impl Default for AndroidPlatformRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlatformRunner for AndroidPlatformRunner {
    async fn install(&self, _project_dir: &Path, device: &Device, artifact_path: Option<&str>) -> Result<()> {
        let artifact = artifact_path.ok_or_else(|| DevflowError::Install("No artifact path specified for Android install".to_string()))?;
        info!("Installing APK {} on device {}", artifact, device.id);

        let adb = Self::resolve_adb();
        let mut cmd = Command::new(&adb);
        cmd.args(["-s", &device.id, "install", "-r", artifact]);

        let output = cmd.output().await
            .map_err(|e| DevflowError::Install(format!("Failed to run adb install: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(DevflowError::Install(format!("adb install failed: {}\n{}", stdout, stderr)));
        }

        Ok(())
    }

    async fn launch(&self, project_dir: &Path, device: &Device, launch_cmd: Option<&str>) -> Result<()> {
        self.stop(project_dir, device).await?;

        let launch_target = launch_cmd.ok_or_else(|| DevflowError::Launch("No launch command or activity specified".to_string()))?;
        info!("Launching Android activity/intent: {}", launch_target);

        let pkg = launch_target.split('/').next().unwrap_or(launch_target).to_string();
        let mut p_lock = self.active_package.lock().await;
        *p_lock = Some(pkg);

        // If launch_cmd is a full adb command or component name
        let output = if launch_target.starts_with("adb") {
            let parts: Vec<&str> = launch_target.split_whitespace().collect();
            let mut cmd = Command::new(parts[0]);
            cmd.args(&parts[1..]);
            cmd.output().await
        } else {
            let adb = Self::resolve_adb();
            let mut cmd = Command::new(&adb);
            cmd.args(["-s", &device.id, "shell", "am", "start", "-n", launch_target]);
            cmd.output().await
        };

        match output {
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                Err(DevflowError::Launch(format!("adb launch failed: {}\n{}", stdout, stderr)))
            }
            Err(e) => Err(DevflowError::Launch(format!("Failed to execute adb launch: {}", e))),
        }
    }

    async fn stop(&self, _project_dir: &Path, device: &Device) -> Result<()> {
        let pkg = self.active_package.lock().await.take();
        if let Some(p) = pkg {
            debug!("Stopping Android package {} on device {}", p, device.id);
            let adb = Self::resolve_adb();
            let mut cmd = Command::new(&adb);
            cmd.args(["-s", &device.id, "shell", "am", "force-stop", &p]);
            let _ = cmd.output().await;
        }
        Ok(())
    }

    async fn stream_logs(&self, _project_dir: &Path, device: &Device, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        let device_id = device.id.clone();
        let adb = Self::resolve_adb();

        let handle = tokio::spawn(async move {
            let mut child = match Command::new(&adb)
                .args(["-s", &device_id, "logcat", "-v", "time"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    warn!("Failed to spawn adb logcat: {}", e);
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let entry = LogParser::parse_line(&line, Some("logcat"));
                    if tx.send(entry).await.is_err() {
                        break;
                    }
                }
            }

            let _ = child.kill().await;
        });

        Ok(handle)
    }
}
