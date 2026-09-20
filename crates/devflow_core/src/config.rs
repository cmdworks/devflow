use crate::error::{DevflowError, Result};
use devflow_protocol::Platform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DevflowConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub install: Option<InstallConfig>,
    #[serde(default)]
    pub launch: Option<LaunchConfig>,
    #[serde(default)]
    pub logs: Option<LogsConfig>,
    #[serde(default)]
    pub watch: Option<WatchConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_platform_str")]
    pub platform: String,
    #[serde(default = "default_framework_str")]
    pub framework: String,
    #[serde(default)]
    pub package_id: Option<String>,
}

fn default_platform_str() -> String {
    "generic".to_string()
}

fn default_framework_str() -> String {
    "generic".to_string()
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "unnamed-project".to_string(),
            platform: default_platform_str(),
            framework: default_framework_str(),
            package_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    pub command: String,
    #[serde(default)]
    pub artifact: Option<String>,
    #[serde(default)]
    pub clean_command: Option<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallConfig {
    pub command: String,
    #[serde(default)]
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LaunchConfig {
    pub command: String,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub stop_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogsConfig {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_watch_paths")]
    pub paths: Vec<String>,
    #[serde(default = "default_watch_extensions")]
    pub extensions: Vec<String>,
    #[serde(default = "default_watch_action")]
    pub action: String,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

fn default_watch_paths() -> Vec<String> {
    vec!["src".to_string()]
}

fn default_watch_extensions() -> Vec<String> {
    vec![
        "rs".to_string(),
        "kt".to_string(),
        "swift".to_string(),
        "ts".to_string(),
        "js".to_string(),
    ]
}

fn default_watch_action() -> String {
    "reload".to_string()
}

fn default_debounce_ms() -> u64 {
    300
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            paths: default_watch_paths(),
            extensions: default_watch_extensions(),
            action: default_watch_action(),
            debounce_ms: default_debounce_ms(),
        }
    }
}

impl DevflowConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DevflowError::Config(format!(
                "Failed to read config file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        let config: DevflowConfig = toml::from_str(&content)
            .map_err(|e| DevflowError::Config(format!("Failed to parse TOML config: {}", e)))?;

        Ok(config)
    }

    pub fn find_and_load(dir: impl AsRef<Path>) -> Result<Option<Self>> {
        let candidate = dir.as_ref().join("devflow.toml");
        if candidate.exists() {
            return Self::load_from_file(candidate).map(Some);
        }
        Ok(None)
    }

    pub fn template_generic(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "desktop"
framework = "generic"

[build]
command = "cargo build"
artifact = "target/debug/{project_name}"

[launch]
command = "./target/debug/{project_name}"

[watch]
paths = ["src"]
extensions = ["rs", "toml"]
action = "restart"
debounce_ms = 300
"#
        )
    }

    pub fn template_android(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "android"
framework = "kotlin"

[build]
command = "./gradlew assembleDebug"
artifact = "app/build/outputs/apk/debug/app-debug.apk"

[install]
command = "adb install -r {{artifact}}"

[launch]
command = "adb shell am start -n com.example.{project_name}/.MainActivity"

[logs]
command = "adb logcat"

[watch]
paths = ["app/src"]
extensions = ["kt", "xml", "gradle"]
action = "restart"
debounce_ms = 500
"#
        )
    }

    pub fn template_swift(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "macos"
framework = "swift"

[build]
command = "swift build"
artifact = ".build/debug/{project_name}"

[launch]
command = ".build/debug/{project_name}"

[watch]
paths = ["Sources"]
extensions = ["swift"]
action = "restart"
debounce_ms = 300
"#
        )
    }

    pub fn template_react_native(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "android"
framework = "react-native"

[build]
command = "npx react-native bundle --platform android --dev false"

[launch]
command = "npx react-native run-android"

[watch]
paths = ["src", "App.tsx", "App.js"]
extensions = ["ts", "tsx", "js", "jsx", "json"]
action = "reload"
debounce_ms = 200
"#
        )
    }

    pub fn template_flutter(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "android"
framework = "flutter"

[build]
command = "flutter build apk --debug"

[launch]
command = "flutter run"

[watch]
paths = ["lib"]
extensions = ["dart"]
action = "reload"
debounce_ms = 200
"#
        )
    }

    pub fn template_tauri(project_name: &str) -> String {
        format!(
            r#"[project]
name = "{project_name}"
platform = "desktop"
framework = "tauri"

[build]
command = "cargo build --manifest-path src-tauri/Cargo.toml"

[launch]
command = "cargo tauri dev"

[watch]
paths = ["src", "src-tauri/src"]
extensions = ["rs", "html", "css", "js", "ts", "tsx", "vue", "svelte"]
action = "reload"
debounce_ms = 300
"#
        )
    }

    pub fn expand_template(&self, template: &str, target_device_id: Option<&str>) -> String {
        let mut result = template.to_string();
        result = result.replace("{project_name}", &self.project.name);
        result = result.replace("{platform}", &self.project.platform);
        result = result.replace("{framework}", &self.project.framework);

        if let Some(target) = target_device_id {
            result = result.replace("{target}", target);
            result = result.replace("{device_id}", target);
        }

        if let Some(build) = &self.build {
            if let Some(art) = &build.artifact {
                result = result.replace("{artifact}", art);
            }
        }

        result
    }

    pub fn target_platform(&self) -> Platform {
        match self.project.platform.to_lowercase().as_str() {
            "android" => Platform::Android,
            "apple" | "ios" => Platform::Ios,
            "macos" => Platform::Macos,
            "desktop" => Platform::Desktop,
            _ => Platform::Generic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devflow_toml() {
        let toml_str = r#"
[project]
name = "sample-app"
platform = "android"
framework = "kotlin"

[build]
command = "./gradlew assembleDebug"
artifact = "app/build/outputs/apk/debug/app-debug.apk"

[install]
command = "adb install -r {artifact}"

[launch]
command = "adb shell am start -n com.example/.MainActivity"

[watch]
paths = ["app/src"]
extensions = ["kt", "xml"]
action = "restart"
debounce_ms = 400
"#;
        let config: DevflowConfig = toml::from_str(toml_str).expect("Failed to parse toml");
        assert_eq!(config.project.name, "sample-app");
        assert_eq!(config.project.platform, "android");
        assert_eq!(config.project.framework, "kotlin");

        let build = config.build.as_ref().unwrap();
        assert_eq!(build.command, "./gradlew assembleDebug");
        assert_eq!(
            build.artifact.as_deref().unwrap(),
            "app/build/outputs/apk/debug/app-debug.apk"
        );

        let install = config.install.as_ref().unwrap();
        let expanded = config.expand_template(&install.command, Some("emulator-5554"));
        assert_eq!(
            expanded,
            "adb install -r app/build/outputs/apk/debug/app-debug.apk"
        );

        let watch = config.watch.as_ref().unwrap();
        assert_eq!(watch.paths, vec!["app/src"]);
        assert_eq!(watch.extensions, vec!["kt", "xml"]);
        assert_eq!(watch.action, "restart");
        assert_eq!(watch.debounce_ms, 400);
    }

    #[test]
    fn test_template_generation() {
        let generic_tpl = DevflowConfig::template_generic("my-service");
        assert!(generic_tpl.contains("name = \"my-service\""));
        assert!(generic_tpl.contains("platform = \"desktop\""));
    }
}
