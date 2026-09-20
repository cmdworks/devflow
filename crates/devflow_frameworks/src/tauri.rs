use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_platforms::{DesktopPlatformRunner, PlatformRunner};
use devflow_protocol::{BuildResult, LogEntry};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::info;

pub struct TauriFrameworkAdapter {
    desktop_runner: Arc<DesktopPlatformRunner>,
}

impl TauriFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            desktop_runner: Arc::new(DesktopPlatformRunner::new()),
        }
    }

    pub fn find_tauri_binary(dir: &Path, is_release: bool, project_name: &str) -> Option<PathBuf> {
        let config = if is_release { "release" } else { "debug" };

        // 1. Check src-tauri/target/<config>/<project_name>
        let candidate = dir
            .join("src-tauri")
            .join("target")
            .join(config)
            .join(project_name);
        if candidate.exists() {
            return Some(candidate);
        }

        // 2. Check root target/<config>/<project_name>
        let root_candidate = dir.join("target").join(config).join(project_name);
        if root_candidate.exists() {
            return Some(root_candidate);
        }

        None
    }
}

impl Default for TauriFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for TauriFrameworkAdapter {
    fn name(&self) -> &'static str {
        "tauri"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        project_dir.join("src-tauri/Cargo.toml").exists()
            || project_dir.join("src-tauri/tauri.conf.json").exists()
            || project_dir.join("tauri.conf.json").exists()
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        info!("Building Tauri project in {}", ctx.project_dir.display());

        let cmd_str = if let Some(ref b) = ctx.config.build {
            b.command.clone()
        } else if ctx.project_dir.join("src-tauri/Cargo.toml").exists() {
            let mode = if ctx.is_release { "--release" } else { "" };
            format!("cargo build --manifest-path src-tauri/Cargo.toml {}", mode)
                .trim()
                .to_string()
        } else {
            "npm run tauri build".to_string()
        };

        let mut parts = cmd_str.split_whitespace();
        let program = parts.next().unwrap_or("cargo");
        let args: Vec<&str> = parts.collect();

        let mut cmd = Command::new(program);
        cmd.args(&args).current_dir(&ctx.project_dir);

        let output = cmd.output().await.map_err(|e| {
            DevflowError::Build(format!("Failed to run Tauri build '{}': {}", cmd_str, e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let duration = start.elapsed().as_millis() as u64;

        if output.status.success() {
            let artifact_path =
                Self::find_tauri_binary(&ctx.project_dir, ctx.is_release, &ctx.config.project.name)
                    .map(|p| p.to_string_lossy().to_string());
            Ok(BuildResult::ok(duration, artifact_path, stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("Tauri build failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, _ctx: &DeviceContext) -> Result<()> {
        Ok(())
    }

    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let default_bin = format!("src-tauri/target/debug/{}", ctx.config.project.name);
        let bin_path = ctx.artifact_path.as_deref().unwrap_or(&default_bin);
        info!("Launching Tauri desktop application: {}", bin_path);
        self.desktop_runner
            .launch(&ctx.project_dir, &ctx.device, Some(bin_path))
            .await
    }

    async fn stop(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Stopping Tauri application...");
        self.desktop_runner
            .stop(&ctx.project_dir, &ctx.device)
            .await
    }

    async fn reload(&self, ctx: &ReloadContext) -> Result<()> {
        // Classify whether changes are frontend-only or backend Rust
        let has_native_changes = ctx.changed_files.iter().any(|p| {
            p.extension()
                .map(|ext| ext == "rs" || ext == "toml")
                .unwrap_or(false)
        });

        if has_native_changes {
            info!("Native Rust/Cargo changes detected in Tauri project -> triggering recompilation and restart...");
            let dev_ctx = DeviceContext {
                project_dir: ctx.project_dir.clone(),
                config: ctx.config.clone(),
                device: ctx.device.clone(),
                artifact_path: None,
            };
            self.restart(&dev_ctx).await
        } else {
            info!("Frontend-only changes detected in Tauri project -> web dev server HMR handles update live without restart.");
            Ok(())
        }
    }

    async fn restart(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Restarting Tauri application...");
        self.desktop_runner
            .stop(&ctx.project_dir, &ctx.device)
            .await?;
        self.launch(ctx).await
    }

    async fn stream_logs(
        &self,
        ctx: &DeviceContext,
        tx: mpsc::Sender<LogEntry>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        self.desktop_runner
            .stream_logs(&ctx.project_dir, &ctx.device, tx)
            .await
    }
}
