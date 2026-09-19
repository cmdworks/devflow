use crate::adapter::{BuildContext, DeviceContext, FrameworkAdapter, ReloadContext};
use async_trait::async_trait;
use devflow_core::error::{DevflowError, Result};
use devflow_platforms::{AndroidPlatformRunner, PlatformRunner};
use devflow_protocol::{BuildResult, LogEntry};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{debug, info};

static NAMESPACE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:namespace|applicationId)\s*=?\s*["']([^"']+)["']"#).unwrap()
});

static LAUNCHER_ACTIVITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<activity[^>]*android:name=["']([^"']+)["'][^>]*>([\s\S]*?)</activity>"#).unwrap()
});

pub struct KotlinFrameworkAdapter {
    android_runner: Arc<AndroidPlatformRunner>,
}

impl KotlinFrameworkAdapter {
    pub fn new() -> Self {
        Self {
            android_runner: Arc::new(AndroidPlatformRunner),
        }
    }

    fn find_gradlew(dir: &Path) -> String {
        if dir.join("gradlew").exists() {
            "./gradlew".to_string()
        } else {
            "gradle".to_string()
        }
    }

    pub fn inspect_android_project(dir: &Path) -> (Option<String>, Option<String>) {
        let mut package_id = None;
        let mut main_activity = None;

        // 1. Search build.gradle / build.gradle.kts for namespace or applicationId
        let gradle_candidates = [
            dir.join("app/build.gradle.kts"),
            dir.join("app/build.gradle"),
            dir.join("build.gradle.kts"),
            dir.join("build.gradle"),
        ];

        for path in &gradle_candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(caps) = NAMESPACE_RE.captures(&content) {
                        if let Some(ns) = caps.get(1) {
                            package_id = Some(ns.as_str().to_string());
                            break;
                        }
                    }
                }
            }
        }

        // 2. Search AndroidManifest.xml for launcher activity
        let manifest_candidates = [
            dir.join("app/src/main/AndroidManifest.xml"),
            dir.join("src/main/AndroidManifest.xml"),
            dir.join("AndroidManifest.xml"),
        ];

        for path in &manifest_candidates {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    for cap in LAUNCHER_ACTIVITY_RE.captures_iter(&content) {
                        let activity_name = cap.get(1).map(|m| m.as_str().to_string());
                        let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                        if body.contains("android.intent.action.MAIN")
                            && (body.contains("android.intent.category.LAUNCHER") || body.contains("LAUNCHER"))
                        {
                            main_activity = activity_name;
                            break;
                        }
                    }
                }
            }
        }

        (package_id, main_activity)
    }

    pub fn find_built_apk(dir: &Path, is_release: bool) -> Option<PathBuf> {
        let config_str = if is_release { "release" } else { "debug" };
        let search_dirs = [
            dir.join("app/build/outputs/apk").join(config_str),
            dir.join("build/outputs/apk").join(config_str),
        ];

        for d in &search_dirs {
            if d.exists() {
                if let Ok(entries) = std::fs::read_dir(d) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().map(|e| e == "apk").unwrap_or(false) {
                            return Some(p);
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for KotlinFrameworkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkAdapter for KotlinFrameworkAdapter {
    fn name(&self) -> &'static str {
        "kotlin"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        project_dir.join("build.gradle").exists()
            || project_dir.join("build.gradle.kts").exists()
            || project_dir.join("settings.gradle").exists()
            || project_dir.join("settings.gradle.kts").exists()
            || project_dir.join("app/build.gradle").exists()
            || project_dir.join("app/build.gradle.kts").exists()
    }

    async fn build(&self, ctx: &BuildContext) -> Result<BuildResult> {
        let start = Instant::now();
        let gradlew = Self::find_gradlew(&ctx.project_dir);
        let task = if ctx.is_release { "assembleRelease" } else { "assembleDebug" };

        info!("Running '{} {}' in {}", gradlew, task, ctx.project_dir.display());

        let output = Command::new(&gradlew)
            .arg(task)
            .current_dir(&ctx.project_dir)
            .output()
            .await
            .map_err(|e| DevflowError::Build(format!("Failed to run Gradle build: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let duration = start.elapsed().as_millis() as u64;

        if output.status.success() {
            let apk_path = Self::find_built_apk(&ctx.project_dir, ctx.is_release)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    if ctx.is_release {
                        "app/build/outputs/apk/release/app-release.apk".to_string()
                    } else {
                        "app/build/outputs/apk/debug/app-debug.apk".to_string()
                    }
                });

            Ok(BuildResult::ok(duration, Some(apk_path), stdout, stderr))
        } else {
            Ok(BuildResult::failed(
                duration,
                stdout,
                stderr.clone(),
                format!("Gradle build failed: {}", stderr),
            ))
        }
    }

    async fn install(&self, ctx: &DeviceContext) -> Result<()> {
        let dynamic_apk = Self::find_built_apk(&ctx.project_dir, false)
            .map(|p| p.to_string_lossy().to_string());
        let apk_path = ctx.artifact_path.as_deref().or(dynamic_apk.as_deref());

        self.android_runner.install(&ctx.project_dir, &ctx.device, apk_path).await
    }


    async fn launch(&self, ctx: &DeviceContext) -> Result<()> {
        let (inferred_pkg, inferred_activity) = Self::inspect_android_project(&ctx.project_dir);

        let launch_component = if let Some(configured) = &ctx.config.project.package_id {
            configured.clone()
        } else if let (Some(pkg), Some(activity)) = (inferred_pkg.as_deref(), inferred_activity.as_deref()) {
            let full_activity = if activity.starts_with('.') {
                format!("{}{}", pkg, activity)
            } else if !activity.contains('.') {
                format!("{}.{}", pkg, activity)
            } else {
                activity.to_string()
            };
            format!("{}/{}", pkg, full_activity)
        } else if let Some(pkg) = inferred_pkg.as_deref() {
            format!("{}/.MainActivity", pkg)
        } else {
            "com.example.app/.MainActivity".to_string()
        };

        debug!("Resolved Android launch intent: {}", launch_component);
        self.android_runner.launch(&ctx.project_dir, &ctx.device, Some(&launch_component)).await
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
        self.android_runner.stop(&ctx.project_dir, &ctx.device).await?;
        self.launch(ctx).await
    }

    async fn stream_logs(&self, ctx: &DeviceContext, tx: mpsc::Sender<LogEntry>) -> Result<tokio::task::JoinHandle<()>> {
        self.android_runner.stream_logs(&ctx.project_dir, &ctx.device, tx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_android_project() {
        let sample_manifest = r#"
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <application>
        <activity android:name=".ui.MainActivity" android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#;
        let mut matched_activity = None;
        for cap in LAUNCHER_ACTIVITY_RE.captures_iter(sample_manifest) {
            let activity_name = cap.get(1).map(|m| m.as_str().to_string());
            let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            if body.contains("android.intent.action.MAIN") && body.contains("android.intent.category.LAUNCHER") {
                matched_activity = activity_name;
                break;
            }
        }
        assert_eq!(matched_activity, Some(".ui.MainActivity".to_string()));

        let sample_gradle = r#"
android {
    namespace = "com.maclink.companion"
    compileSdk = 34
}
"#;
        let caps = NAMESPACE_RE.captures(sample_gradle).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "com.maclink.companion");
    }
}
