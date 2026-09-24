use devflow_protocol::{Device, DeviceState, Platform};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::info;

pub struct EmulatorManager;

impl EmulatorManager {
    pub fn find_emulator_binary() -> Option<PathBuf> {
        // 1. Check in PATH
        if let Ok(path) = which_binary("emulator") {
            return Some(path);
        }

        // 2. Check standard environment variables and platform paths
        let env_roots = [
            std::env::var("ANDROID_HOME").ok(),
            std::env::var("ANDROID_SDK_ROOT").ok(),
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/Library/Android/sdk", h)),
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/Android/Sdk", h)),
            std::env::var("LOCALAPPDATA")
                .ok()
                .map(|h| format!("{}/Android/Sdk", h)),
        ];

        for root in env_roots.into_iter().flatten() {
            let candidate1 = Path::new(&root).join("emulator/emulator");
            if candidate1.exists() {
                return Some(candidate1);
            }
            let candidate2 = Path::new(&root).join("tools/emulator");
            if candidate2.exists() {
                return Some(candidate2);
            }
        }

        None
    }

    pub async fn list_avds() -> Vec<String> {
        let emulator_bin = match Self::find_emulator_binary() {
            Some(b) => b,
            None => return Vec::new(),
        };

        let output = match Command::new(&emulator_bin).arg("-list-avds").output().await {
            Ok(out) if out.status.success() => out,
            _ => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    pub async fn discover_avd_devices() -> Vec<Device> {
        let avds = Self::list_avds().await;
        let mut devices = Vec::new();

        for avd in avds {
            let mut device = Device::new(
                format!("avd:{}", avd),
                format!("Android Virtual Device ({})", avd),
                Platform::Android,
                DeviceState::Shutdown,
            );
            device.is_emulator = true;
            devices.push(device);
        }

        devices
    }

    pub async fn boot_avd(avd_name: &str) -> Result<String, String> {
        let emulator_bin = Self::find_emulator_binary().ok_or_else(|| {
            "Android SDK 'emulator' tool not found. Ensure Android SDK is installed and ANDROID_HOME or ANDROID_SDK_ROOT is set (e.g. export ANDROID_HOME=$HOME/Library/Android/sdk).".to_string()
        })?;

        let available_avds = Self::list_avds().await;
        if !available_avds.is_empty() && !available_avds.iter().any(|a| a == avd_name) {
            return Err(format!(
                "Android Virtual Device '{}' not found. Available AVDs: [{}]. You can create one via Android Studio Device Manager.",
                avd_name,
                available_avds.join(", ")
            ));
        }

        info!("Booting Android AVD '{}'...", avd_name);

        let mut child = Command::new(&emulator_bin)
            .args(["-avd", avd_name, "-no-boot-anim"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn emulator '{}': {}", avd_name, e))?;

        // Give process a brief moment to check if it immediately crashed
        tokio::time::sleep(Duration::from_millis(600)).await;
        if let Ok(Some(status)) = child.try_wait() {
            if !status.success() {
                let mut err_msg = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let mut buf = Vec::new();
                    let _ = stderr.read_to_end(&mut buf).await;
                    err_msg = String::from_utf8_lossy(&buf).trim().to_string();
                }
                return Err(format!(
                    "Android emulator failed to start (exit code {}): {}",
                    status.code().unwrap_or(-1),
                    if err_msg.is_empty() {
                        "Check AVD configuration and hardware virtualization"
                    } else {
                        &err_msg
                    }
                ));
            }
        }

        // Detach process so it keeps running
        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        let adb_bin = crate::AdbDiscoverer::resolve_adb_binary();

        // Wait up to 40 seconds for ADB to detect booted device
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let output = Command::new(&adb_bin)
                .args(["shell", "getprop", "sys.boot_completed"])
                .output()
                .await;

            if let Ok(out) = output {
                if out.status.success() {
                    let prop = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if prop == "1" {
                        info!("AVD '{}' successfully booted and ready!", avd_name);
                        return Ok(format!("AVD '{}' booted and ready", avd_name));
                    }
                }
            }
        }

        Ok(format!(
            "AVD '{}' launched in background (booting in progress)",
            avd_name
        ))
    }
}

pub fn which_binary(name: &str) -> Result<PathBuf, std::io::Error> {
    if let Ok(paths) = std::env::var("PATH") {
        for p in paths.split(':') {
            let candidate = Path::new(p).join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found",
    ))
}
