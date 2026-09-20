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
use tracing::info;

pub struct FlutterFrameworkAdapter {
    android_runner: Arc<AndroidPlatformRunner>,
    apple_runner: Arc<ApplePlatformRunner>,
    desktop_runner: Arc<DesktopPlatformRunner>,
}

impl FlutterFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            android_runner: Arc::new(AndroidPlatformRunner::new()),
            apple_runner: Arc::new(ApplePlatformRunner::new()),
            desktop_runner: Arc::new(DesktopPlatformRunner::new()),
        }
    }
}

impl Default for FlutterFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for FlutterFrameworkAdapter {
    fn name(&self) -> &'static str {
        "flutter"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let pubspec = project_dir.join("pubspec.yaml");
        if pubspec.exists() {
            if let Ok(content) = std::fs::read_to_string(&pubspec) {
                return content.contains("flutter:") || content.contains("sdk: flutter");
            }
        }
        false
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        info!("Building Flutter project in {}", ctx.project_dir.display());

        let cmd_str = if let Some(ref b) = ctx.config.build {
            b.command.clone()
        } else {
            match ctx.target_device.as_ref().map(|d| d.platform).unwrap_or(Platform::Android) {
                Platform::Android => "flutter build apk --debug".to_string(),
                Platform::Apple | Platform::Ios => "flutter build ios --simulator --debug".to_string(),
                Platform::Macos | Platform::Desktop => "flutter build macos --debug".to_string(),
                _ => "flutter build apk --debug".to_string(),
            }
        };

        let mut cmd = Command::new("sh");
        cmd.args(["-c", &cmd_str]).current_dir(&ctx.project_dir);

        let output = cmd.output().await
            .map_err(|e| DevflowError::Build(format!("Failed to run Flutter build '{}': {}", cmd_str, e)))?;

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
                format!("Flutter build failed: {}", stderr),
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
        info!("Launching Flutter app on device {} ({})", ctx.device.id, ctx.device.platform);
        match ctx.device.platform {
            Platform::Android => self.android_runner.launch(&ctx.project_dir, &ctx.device, ctx.config.project.package_id.as_deref()).await,
            Platform::Apple | Platform::Ios => self.apple_runner.launch(&ctx.project_dir, &ctx.device, ctx.config.project.package_id.as_deref()).await,
            _ => self.desktop_runner.launch(&ctx.project_dir, &ctx.device, None).await,
        }
    }

    async fn stop(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Stopping Flutter application on device {} ({})", ctx.device.id, ctx.device.platform);
        match ctx.device.platform {
            Platform::Android => self.android_runner.stop(&ctx.project_dir, &ctx.device).await,
            Platform::Apple | Platform::Ios => self.apple_runner.stop(&ctx.project_dir, &ctx.device).await,
            _ => self.desktop_runner.stop(&ctx.project_dir, &ctx.device).await,
        }
    }

    async fn reload(&self, ctx: &ReloadContext) -> Result<()> {
        info!("Flutter Hot Reload triggered ('r')...");
        // In full daemon mode, the daemon stdin is sent 'r' or JSON reload request.
        // For general CLI fallback, attempt flutter attach / reload or restart
        let dev_ctx = DeviceContext {
            project_dir: ctx.project_dir.clone(),
            config: ctx.config.clone(),
            device: ctx.device.clone(),
            artifact_path: None,
        };
        info!("Performing rapid Hot Reload for {} changed files", ctx.changed_files.len());
        // If hot reload not attached directly, fallback to restart
        self.restart(&dev_ctx).await
    }

    async fn restart(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Restarting Flutter app (Hot Restart 'R')...");
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
