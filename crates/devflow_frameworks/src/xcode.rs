use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_platforms::{ApplePlatformRunner, PlatformRunner};
use devflow_protocol::{BuildResult, LogEntry, Platform};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::info;

pub struct XcodeAdapter {
    apple_runner: Arc<ApplePlatformRunner>,
}

impl XcodeAdapter {
    pub fn new() -> Self {
        Self {
            apple_runner: Arc::new(ApplePlatformRunner),
        }
    }

    pub fn find_xcode_container(dir: &Path) -> Option<(PathBuf, bool)> {
        // Return (path, is_workspace)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if ext == "xcworkspace" {
                        return Some((p, true));
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if ext == "xcodeproj" {
                        return Some((p, false));
                    }
                }
            }
        }

        None
    }

    pub async fn list_schemes(dir: &Path) -> Vec<String> {
        let output = match Command::new("xcodebuild")
            .args(["-list", "-json"])
            .current_dir(dir)
            .output()
            .await
        {
            Ok(out) if out.status.success() => out,
            _ => return Vec::new(),
        };

        if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
            let container = json.get("project").or_else(|| json.get("workspace"));
            if let Some(c) = container {
                if let Some(schemes) = c.get("schemes").and_then(|s| s.as_array()) {
                    return schemes
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }

        Vec::new()
    }

    pub async fn get_build_settings(dir: &Path, scheme: Option<&str>, destination: Option<&str>) -> Option<Value> {
        let mut cmd = Command::new("xcodebuild");
        cmd.args(["-showBuildSettings", "-json"]).current_dir(dir);

        if let Some(s) = scheme {
            cmd.args(["-scheme", s]);
        }
        if let Some(d) = destination {
            cmd.args(["-destination", d]);
        }

        let output = cmd.output().await.ok()?;
        if output.status.success() {
            serde_json::from_slice::<Value>(&output.stdout).ok()
        } else {
            None
        }
    }

    pub fn extract_bundle_id(app_path: &Path) -> Option<String> {
        let plist_path = app_path.join("Info.plist");
        if plist_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&plist_path) {
                if let Some(pos) = content.find("CFBundleIdentifier") {
                    let rest = &content[pos..];
                    if let Some(start_str) = rest.find("<string>") {
                        let after_tag = &rest[start_str + 8..];
                        if let Some(end_str) = after_tag.find("</string>") {
                            return Some(after_tag[..end_str].trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

impl Default for XcodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for XcodeAdapter {
    fn name(&self) -> &'static str {
        "xcode"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        Self::find_xcode_container(project_dir).is_some()
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        let (container_path, is_workspace) = Self::find_xcode_container(&ctx.project_dir)
            .ok_or_else(|| DevflowError::Build("No .xcodeproj or .xcworkspace found".to_string()))?;

        let schemes = Self::list_schemes(&ctx.project_dir).await;
        let scheme = schemes.first().cloned().unwrap_or_else(|| ctx.config.project.name.clone());

        let mut cmd = Command::new("xcodebuild");
        cmd.current_dir(&ctx.project_dir);

        if is_workspace {
            cmd.arg("-workspace").arg(container_path.file_name().unwrap());
        } else {
            cmd.arg("-project").arg(container_path.file_name().unwrap());
        }

        cmd.arg("-scheme").arg(&scheme);

        // Configure destination
        let dest_str = if let Some(ref dev) = ctx.target_device {
            if dev.platform == Platform::Ios || dev.platform == Platform::Apple {
                if dev.is_emulator {
                    format!("id={}", dev.id)
                } else {
                    format!("id={}", dev.id)
                }
            } else {
                "generic/platform=macOS".to_string()
            }
        } else {
            "generic/platform=iOS Simulator".to_string()
        };

        cmd.arg("-destination").arg(&dest_str);

        if ctx.is_release {
            cmd.args(["-configuration", "Release"]);
        } else {
            cmd.args(["-configuration", "Debug"]);
        }

        info!("Running 'xcodebuild -scheme {} -destination {}' in {}", scheme, dest_str, ctx.project_dir.display());

        let output = cmd.output().await
            .map_err(|e| DevflowError::Build(format!("Failed to run xcodebuild: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let duration = start.elapsed().as_millis() as u64;

        if output.status.success() {
            // Find build product path
            let mut app_path = None;
            for line in stdout.lines().rev() {
                if line.contains(".app") && line.contains("BUILD SUCCESSFUL") || line.contains("Signing") {
                    if let Some(pos) = line.find('/') {
                        let path_candidate = &line[pos..];
                        if let Some(end) = path_candidate.find(".app") {
                            let full = &path_candidate[..end + 4];
                            if Path::new(full).exists() {
                                app_path = Some(full.to_string());
                                break;
                            }
                        }
                    }
                }
            }

            Ok(BuildResult::ok(duration, app_path, stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("xcodebuild failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, ctx: &DeviceContext) -> Result<()> {
        let app_path = ctx.artifact_path.as_deref();
        self.apple_runner.install(&ctx.project_dir, &ctx.device, app_path).await
    }

    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let bundle_id = ctx.config.project.package_id.clone().or_else(|| {
            if let Some(ref path) = ctx.artifact_path {
                Self::extract_bundle_id(Path::new(path))
            } else {
                None
            }
        }).unwrap_or_else(|| format!("com.example.{}", ctx.config.project.name));

        info!("Launching Xcode app bundle: {}", bundle_id);
        self.apple_runner.launch(&ctx.project_dir, &ctx.device, Some(&bundle_id)).await
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
        self.apple_runner.stop(&ctx.project_dir, &ctx.device).await?;
        self.launch(ctx).await
    }

    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        self.apple_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await
    }
}
