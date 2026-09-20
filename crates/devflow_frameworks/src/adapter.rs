use async_trait::async_trait;
use devflow_core::config::DevflowConfig;
use devflow_core::error::Result;
use devflow_protocol::{BuildResult, Device, LogEntry};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct BuildContext {
    pub project_dir: PathBuf,
    pub config: DevflowConfig,
    pub target_device: Option<Device>,
    pub is_release: bool,
}

#[derive(Debug, Clone)]
pub struct DeviceContext {
    pub project_dir: PathBuf,
    pub config: DevflowConfig,
    pub device: Device,
    pub artifact_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReloadContext {
    pub project_dir: PathBuf,
    pub config: DevflowConfig,
    pub device: Device,
    pub changed_files: Vec<PathBuf>,
}

#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, project_dir: &Path) -> bool;
    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult>;
    async fn install(&self, ctx: &DeviceContext) -> Result<()>;
    async fn launch(&self, ctx: &DeviceContext) -> Result<()>;
    async fn stop(&self, ctx: &DeviceContext) -> Result<()>;
    async fn reload(&self, ctx: &ReloadContext) -> Result<()>;
    async fn restart(&self, ctx: &DeviceContext) -> Result<()>;
    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>>;
}
