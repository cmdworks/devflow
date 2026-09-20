use crate::config::DevflowConfig;
use crate::error::Result;
use devflow_protocol::Platform;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFramework {
    KotlinAndroid,
    SwiftPM,
    XcodeProject,
    ReactNative,
    Flutter,
    Tauri,
    RustCargo,
    Generic,
}

impl DetectedFramework {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectedFramework::KotlinAndroid => "kotlin",
            DetectedFramework::SwiftPM => "swift",
            DetectedFramework::XcodeProject => "xcode",
            DetectedFramework::ReactNative => "react-native",
            DetectedFramework::Flutter => "flutter",
            DetectedFramework::Tauri => "tauri",
            DetectedFramework::RustCargo => "rust",
            DetectedFramework::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root_dir: PathBuf,
    pub name: String,
    pub detected_framework: DetectedFramework,
    pub detected_platform: Platform,
    pub config: Option<DevflowConfig>,
}

impl Project {
    pub fn detect(dir: impl AsRef<Path>) -> Result<Self> {
        let root = dir
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| dir.as_ref().to_path_buf());

        // Check if devflow.toml exists
        let config = DevflowConfig::find_and_load(&root)?;

        let (framework, platform, default_name) = if let Some(ref cfg) = config {
            let fw = match cfg.project.framework.to_lowercase().as_str() {
                "kotlin" | "android" => DetectedFramework::KotlinAndroid,
                "swift" | "swiftpm" => DetectedFramework::SwiftPM,
                "xcode" | "ios" => DetectedFramework::XcodeProject,
                "react-native" | "rn" => DetectedFramework::ReactNative,
                "flutter" => DetectedFramework::Flutter,
                "tauri" => DetectedFramework::Tauri,
                "rust" | "cargo" => DetectedFramework::RustCargo,
                _ => DetectedFramework::Generic,
            };
            (fw, cfg.target_platform(), cfg.project.name.clone())
        } else {
            Self::infer_from_filesystem(&root)?
        };

        let project_name = if let Some(ref cfg) = config {
            cfg.project.name.clone()
        } else {
            default_name
        };

        Ok(Self {
            root_dir: root,
            name: project_name,
            detected_framework: framework,
            detected_platform: platform,
            config,
        })
    }

    fn infer_from_filesystem(dir: &Path) -> Result<(DetectedFramework, Platform, String)> {
        let dir_name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string());

        // 1. Android / Gradle
        if dir.join("build.gradle").exists()
            || dir.join("build.gradle.kts").exists()
            || dir.join("app/build.gradle").exists()
            || dir.join("app/build.gradle.kts").exists()
            || dir.join("settings.gradle").exists()
            || dir.join("settings.gradle.kts").exists()
        {
            return Ok((
                DetectedFramework::KotlinAndroid,
                Platform::Android,
                dir_name,
            ));
        }

        // 2. SwiftPM
        if dir.join("Package.swift").exists() {
            return Ok((DetectedFramework::SwiftPM, Platform::Macos, dir_name));
        }

        // 3. Flutter
        if dir.join("pubspec.yaml").exists() {
            return Ok((DetectedFramework::Flutter, Platform::Android, dir_name));
        }

        // 4. React Native
        if dir.join("package.json").exists() {
            if let Ok(pkg_json) = std::fs::read_to_string(dir.join("package.json")) {
                if pkg_json.contains("react-native") {
                    return Ok((DetectedFramework::ReactNative, Platform::Android, dir_name));
                }
            }
        }

        // 5. Tauri
        if dir.join("src-tauri").exists() || dir.join("tauri.conf.json").exists() {
            return Ok((DetectedFramework::Tauri, Platform::Desktop, dir_name));
        }

        // 6. Rust / Cargo
        if dir.join("Cargo.toml").exists() {
            return Ok((DetectedFramework::RustCargo, Platform::Desktop, dir_name));
        }

        Ok((DetectedFramework::Generic, Platform::Generic, dir_name))
    }

    pub fn effective_config(&self) -> DevflowConfig {
        if let Some(ref cfg) = self.config {
            cfg.clone()
        } else {
            DevflowConfig {
                project: crate::config::ProjectConfig {
                    name: self.name.clone(),
                    platform: self.detected_platform.to_string().to_lowercase(),
                    framework: self.detected_framework.as_str().to_string(),
                    package_id: None,
                },
                build: None,
                install: None,
                launch: None,
                logs: None,
                watch: Some(crate::config::WatchConfig::default()),
                env: Default::default(),
            }
        }
    }

    /// Auto-discover all runnable project targets in a workspace or repository (root and child targets).
    pub fn discover_workspace_targets(dir: impl AsRef<Path>) -> Vec<ProjectTarget> {
        let root = dir.as_ref();
        let mut targets = Vec::new();

        // 1. Check root directory
        if let Ok(root_proj) = Self::detect(root) {
            if root_proj.detected_framework != DetectedFramework::Generic
                || root.join("devflow.toml").exists()
            {
                targets.push(ProjectTarget {
                    id: format!("{}-root", root_proj.name.to_lowercase().replace(' ', "-")),
                    name: root_proj.name.clone(),
                    path: root.to_path_buf(),
                    platform: root_proj.detected_platform,
                    framework: root_proj.detected_framework.as_str().to_string(),
                    is_default: true,
                });
            }
        }

        // 2. Scan child subdirectories (e.g. android/, ios/, macos/, client/, app/)
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let dir_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if dir_name.starts_with('.')
                        || dir_name == "target"
                        || dir_name == "build"
                        || dir_name == ".build"
                        || dir_name == "node_modules"
                    {
                        continue;
                    }

                    if let Ok(sub_proj) = Self::detect(&p) {
                        if sub_proj.detected_framework != DetectedFramework::Generic
                            || p.join("devflow.toml").exists()
                        {
                            targets.push(ProjectTarget {
                                id: format!(
                                    "{}-{}",
                                    sub_proj.name.to_lowercase().replace(' ', "-"),
                                    dir_name
                                ),
                                name: format!("{} ({})", sub_proj.name, dir_name),
                                path: p,
                                platform: sub_proj.detected_platform,
                                framework: sub_proj.detected_framework.as_str().to_string(),
                                is_default: targets.is_empty(),
                            });
                        }
                    }
                }
            }
        }

        targets
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectTarget {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub platform: Platform,
    pub framework: String,
    pub is_default: bool,
}
