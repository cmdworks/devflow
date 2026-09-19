use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_platforms::{AndroidPlatformRunner, ApplePlatformRunner, DesktopPlatformRunner, PlatformRunner};
use devflow_protocol::{BuildResult, LogEntry, Platform};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct ReactNativeFrameworkAdapter {
    android_runner: Arc<AndroidPlatformRunner>,
    apple_runner: Arc<ApplePlatformRunner>,
    desktop_runner: Arc<DesktopPlatformRunner>,
}

impl ReactNativeFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            android_runner: Arc::new(AndroidPlatformRunner),
            apple_runner: Arc::new(ApplePlatformRunner),
            desktop_runner: Arc::new(DesktopPlatformRunner::new()),
        }
    }

    /// Trigger Metro bundler reload over HTTP
    pub async fn trigger_metro_reload(port: u16) -> bool {
        reqwest_like_ping(port).await
    }
}

async fn reqwest_like_ping(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(mut stream) = tokio::net::TcpStream::connect(&addr).await {
        use tokio::io::AsyncWriteExt;
        let req = format!("GET /reload HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n", port);
        let _ = stream.write_all(req.as_bytes()).await;
        return true;
    }
    false
}

impl Default for ReactNativeFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for ReactNativeFrameworkAdapter {
    fn name(&self) -> &'static str {
        "react-native"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        if project_dir.join("metro.config.js").exists() {
            return true;
        }

        let pkg_path = project_dir.join("package.json");
        if pkg_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_path) {
                return content.contains("\"react-native\"") || content.contains("'react-native'");
            }
        }

        false
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        info!("Building React Native project in {}", ctx.project_dir.display());

        let cmd_str = if let Some(ref b) = ctx.config.build {
            b.command.clone()
        } else {
            "npx react-native bundle --platform android --dev false --entry-file index.js --bundle-output android/app/src/main/assets/index.android.bundle".to_string()
        };

        let mut cmd = Command::new("sh");
        cmd.args(["-c", &cmd_str]).current_dir(&ctx.project_dir);

        let output = cmd.output().await
            .map_err(|e| DevflowError::Build(format!("Failed to run React Native build '{}': {}", cmd_str, e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let duration = start.elapsed().as_millis() as u64;

        if output.status.success() {
            Ok(BuildResult::ok(duration, None, stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("React Native build failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, ctx: &DeviceContext) -> Result<()> {
        match ctx.device.platform {
            Platform::Android => self.android_runner.install(&ctx.project_dir, &ctx.device, ctx.artifact_path.as_deref()).await,
            Platform::Apple | Platform::Ios => self.apple_runner.install(&ctx.project_dir, &ctx.device, ctx.artifact_path.as_deref()).await,
            _ => Ok(()),
        }
    }

    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let package_id = ctx.config.project.package_id.as_deref().unwrap_or("com.example.app");
        info!("Launching React Native app on device {} ({})", ctx.device.id, ctx.device.platform);

        match ctx.device.platform {
            Platform::Android => self.android_runner.launch(&ctx.project_dir, &ctx.device, Some(package_id)).await,
            Platform::Apple | Platform::Ios => self.apple_runner.launch(&ctx.project_dir, &ctx.device, Some(package_id)).await,
            _ => self.desktop_runner.launch(&ctx.project_dir, &ctx.device, None).await,
        }
    }

    async fn reload(&self, ctx: &ReloadContext) -> Result<()> {
        info!("React Native reload triggered: notifying Metro bundler on port 8081...");
        let metro_reloaded = Self::trigger_metro_reload(8081).await;

        if metro_reloaded {
            info!("Metro bundler reload signal successfully sent.");
            return Ok(());
        }

        // If Metro isn't listening, send reload keyevent for Android
        if ctx.device.platform == Platform::Android {
            info!("Metro not directly reachable; sending reload keyevent to Android device...");
            let mut cmd = Command::new("adb");
            cmd.args(["-s", &ctx.device.id, "shell", "input", "text", "rr"]);
            let _ = cmd.output().await;
            return Ok(());
        }

        warn!("Metro server not reachable at localhost:8081; falling back to full restart.");
        let dev_ctx = DeviceContext {
            project_dir: ctx.project_dir.clone(),
            config: ctx.config.clone(),
            device: ctx.device.clone(),
            artifact_path: None,
        };
        self.restart(&dev_ctx).await
    }

    async fn restart(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Restarting React Native app...");
        match ctx.device.platform {
            Platform::Android => {
                self.android_runner.stop(&ctx.project_dir, &ctx.device).await?;
                self.launch(ctx).await
            }
            Platform::Apple | Platform::Ios => {
                self.apple_runner.stop(&ctx.project_dir, &ctx.device).await?;
                self.launch(ctx).await
            }
            _ => {
                self.desktop_runner.stop(&ctx.project_dir, &ctx.device).await?;
                self.launch(ctx).await
            }
        }
    }

    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        match ctx.device.platform {
            Platform::Android => self.android_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await,
            Platform::Apple | Platform::Ios => self.apple_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await,
            _ => self.desktop_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await,
        }
    }
}
