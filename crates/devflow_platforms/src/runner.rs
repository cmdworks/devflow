use async_trait::async_trait;
use devflow_core::error::Result;
use devflow_protocol::{Device, LogEntry};
use std::path::Path;
use tokio::sync::mpsc;

#[async_trait]
pub trait PlatformRunner: Send + Sync {
    async fn install(
        &self,
        project_dir: &Path,
        device: &Device,
        artifact_path: Option<&str>,
    ) -> Result<()>;
    async fn launch(
        &self,
        project_dir: &Path,
        device: &Device,
        launch_cmd: Option<&str>,
    ) -> Result<()>;
    async fn stop(&self, project_dir: &Path, device: &Device) -> Result<()>;
    async fn restart(
        &self,
        project_dir: &Path,
        device: &Device,
        launch_cmd: Option<&str>,
    ) -> Result<()> {
        self.stop(project_dir, device).await?;
        self.launch(project_dir, device, launch_cmd).await
    }
    async fn stream_logs(
        &self,
        project_dir: &Path,
        device: &Device,
        tx: mpsc::Sender<LogEntry>,
    ) -> Result<tokio::task::JoinHandle<()>>;
}
