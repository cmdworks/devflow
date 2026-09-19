use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_platforms::{DesktopPlatformRunner, PlatformRegistry, PlatformRunner};
use devflow_protocol::{BuildResult, LogEntry};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::info;

pub struct GenericFrameworkAdapter {
    runner: Arc<dyn PlatformRunner>,
}

impl GenericFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            runner: Arc::new(DesktopPlatformRunner::new()),
        }
    }

    async fn run_shell_command(cmd_str: &str, dir: &Path) -> Result<(bool, String, String)> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .current_dir(dir)
            .output()
            .await
            .map_err(|e| DevflowError::ToolExecution(format!("Failed to execute '{}': {}", cmd_str, e)))?;

        let stdout = String::from_utf8_lossy(&child.stdout).to_string();
        let stderr = String::from_utf8_lossy(&child.stderr).to_string();

        Ok((child.status.success(), stdout, stderr))
    }
}

impl Default for GenericFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for GenericFrameworkAdapter {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        project_dir.join("devflow.toml").exists()
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        let build_cfg = match &ctx.config.build {
            Some(b) => b,
            None => {
                return Ok(BuildResult::ok(0, None, "No build step configured".to_string(), String::new()));
            }
        };

        let target_id = ctx.target_device.as_ref().map(|d| d.id.as_str());
        let expanded_cmd = ctx.config.expand_template(&build_cfg.command, target_id);
        info!("Executing build command: {}", expanded_cmd);

        let (success, stdout, stderr) = Self::run_shell_command(&expanded_cmd, &ctx.project_dir).await?;
        let duration = start.elapsed().as_millis() as u64;

        if success {
            let artifact = build_cfg.artifact.as_ref().map(|a| ctx.config.expand_template(a, target_id));
            Ok(BuildResult::ok(duration, artifact, stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("Build command failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, ctx: &DeviceContext) -> Result<()> {
        let platform_runner = PlatformRegistry::get_runner(ctx.device.platform);

        if let Some(install_cfg) = &ctx.config.install {
            let expanded_cmd = ctx.config.expand_template(&install_cfg.command, Some(&ctx.device.id));
            info!("Executing custom install command: {}", expanded_cmd);
            let (success, stdout, stderr) = Self::run_shell_command(&expanded_cmd, &ctx.project_dir).await?;
            if !success {
                return Err(DevflowError::Install(format!("Install failed: {}\n{}", stdout, stderr)));
            }
            Ok(())
        } else {
            platform_runner.install(&ctx.project_dir, &ctx.device, ctx.artifact_path.as_deref()).await
        }
    }

    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let platform_runner = if ctx.device.platform == devflow_protocol::Platform::Desktop || ctx.device.platform == devflow_protocol::Platform::Generic {
            self.runner.clone()
        } else {
            PlatformRegistry::get_runner(ctx.device.platform)
        };

        if let Some(launch_cfg) = &ctx.config.launch {
            let expanded_cmd = ctx.config.expand_template(&launch_cfg.command, Some(&ctx.device.id));
            info!("Executing launch command: {}", expanded_cmd);

            if ctx.device.platform == devflow_protocol::Platform::Desktop || ctx.device.platform == devflow_protocol::Platform::Generic {
                platform_runner.launch(&ctx.project_dir, &ctx.device, Some(&expanded_cmd)).await
            } else {
                let (success, stdout, stderr) = Self::run_shell_command(&expanded_cmd, &ctx.project_dir).await?;
                if !success {
                    return Err(DevflowError::Launch(format!("Launch failed: {}\n{}", stdout, stderr)));
                }
                Ok(())
            }
        } else if let Some(artifact) = &ctx.artifact_path {
            platform_runner.launch(&ctx.project_dir, &ctx.device, Some(artifact)).await
        } else {
            Ok(())
        }
    }

    async fn reload(&self, ctx: &ReloadContext) -> Result<()> {
        info!("Generic reload triggered — performing restart for changed files");
        let dev_ctx = DeviceContext {
            project_dir: ctx.project_dir.clone(),
            config: ctx.config.clone(),
            device: ctx.device.clone(),
            artifact_path: None,
        };
        self.restart(&dev_ctx).await
    }

    async fn restart(&self, ctx: &DeviceContext) -> Result<()> {
        info!("Restarting application on device {}", ctx.device.name);
        let platform_runner = if ctx.device.platform == devflow_protocol::Platform::Desktop || ctx.device.platform == devflow_protocol::Platform::Generic {
            self.runner.clone()
        } else {
            PlatformRegistry::get_runner(ctx.device.platform)
        };

        platform_runner.stop(&ctx.project_dir, &ctx.device).await?;
        self.launch(ctx).await
    }

    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        let platform_runner = if ctx.device.platform == devflow_protocol::Platform::Desktop || ctx.device.platform == devflow_protocol::Platform::Generic {
            self.runner.clone()
        } else {
            PlatformRegistry::get_runner(ctx.device.platform)
        };

        platform_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await
    }
}
