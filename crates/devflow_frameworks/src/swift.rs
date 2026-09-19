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

pub struct SwiftFrameworkAdapter {
    desktop_runner: Arc<DesktopPlatformRunner>,
}

impl SwiftFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            desktop_runner: Arc::new(DesktopPlatformRunner::new()),
        }
    }

    pub fn find_built_binary(dir: &Path, is_release: bool, project_name: &str) -> Option<PathBuf> {
        let config = if is_release { "release" } else { "debug" };
        let candidate = dir.join(".build").join(config).join(project_name);
        if candidate.exists() {
            return Some(candidate);
        }

        // Search in .build/debug / release
        let build_dir = dir.join(".build").join(config);
        if let Ok(entries) = std::fs::read_dir(&build_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && !p.file_name().unwrap().to_string_lossy().contains('.') {
                    return Some(p);
                }
            }
        }

        None
    }

    pub fn package_macos_app(bin_path: &Path, app_name: &str, output_dir: &Path) -> std::io::Result<PathBuf> {
        let app_bundle = output_dir.join(format!("{}.app", app_name));
        let contents = app_bundle.join("Contents");
        let macos_dir = contents.join("MacOS");
        let resources_dir = contents.join("Resources");

        std::fs::create_dir_all(&macos_dir)?;
        std::fs::create_dir_all(&resources_dir)?;

        let target_bin = macos_dir.join(app_name);
        std::fs::copy(bin_path, &target_bin)?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
</dict>
</plist>"#,
            app_name, app_name, app_name
        );

        std::fs::write(contents.join("Info.plist"), plist_content)?;
        Ok(app_bundle)
    }
}

impl Default for SwiftFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for SwiftFrameworkAdapter {
    fn name(&self) -> &'static str {
        "swift"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        project_dir.join("Package.swift").exists()
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        info!("Running 'swift build' in {}", ctx.project_dir.display());

        let mut cmd = Command::new("swift");
        cmd.arg("build").current_dir(&ctx.project_dir);

        if ctx.is_release {
            cmd.args(["-c", "release"]);
        }

        let output = cmd.output().await
            .map_err(|e| DevflowError::Build(format!("Failed to run swift build: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let duration = start.elapsed().as_millis() as u64;

        if output.status.success() {
            let target_config = if ctx.is_release { "release" } else { "debug" };
            let artifact_path = Self::find_built_binary(&ctx.project_dir, ctx.is_release, &ctx.config.project.name)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| format!(".build/{}/{}", target_config, ctx.config.project.name));

            Ok(BuildResult::ok(duration, Some(artifact_path), stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("swift build failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, _ctx: &DeviceContext) -> Result<()> {
        Ok(())
    }

    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let default_bin = format!("./.build/debug/{}", ctx.config.project.name);
        let bin_path = ctx.artifact_path.as_deref().unwrap_or(&default_bin);
        info!("Launching Swift executable: {}", bin_path);
        self.desktop_runner.launch(&ctx.project_dir, &ctx.device, Some(bin_path)).await
    }

    async fn reload(&self, ctx: &ReloadContext) -> Result<()> {
        let dev_ctx = DeviceContext {
            project_dir: ctx.project_dir.clone(),
            config: ctx.config.clone(),
            device: ctx.device.clone(),
            artifact_path: None,
        };
        self.restart(&dev_ctx).await
    }

    async fn restart(&self, ctx: &DeviceContext) -> Result<()> {
        self.desktop_runner.stop(&ctx.project_dir, &ctx.device).await?;
        self.launch(ctx).await
    }

    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        self.desktop_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await
    }
}
