use devflow_protocol::{Device, DeviceState, Platform};
use tokio::process::Command;
use tracing::debug;

pub struct AdbDiscoverer;

impl AdbDiscoverer {
    pub fn resolve_adb_binary() -> String {
        if std::process::Command::new("adb")
            .arg("version")
            .output()
            .is_ok()
        {
            return "adb".to_string();
        }
        if let Ok(home) = std::env::var("ANDROID_HOME") {
            let p = std::path::Path::new(&home)
                .join("platform-tools")
                .join("adb");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        if let Ok(home) = std::env::var("ANDROID_SDK_ROOT") {
            let p = std::path::Path::new(&home)
                .join("platform-tools")
                .join("adb");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let p = std::path::Path::new(&home).join("Library/Android/sdk/platform-tools/adb");
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        "adb".to_string()
    }

    pub async fn discover() -> Vec<Device> {
        let adb_bin = Self::resolve_adb_binary();
        let output = match Command::new(&adb_bin)
            .args(["devices", "-l"])
            .output()
            .await
        {
            Ok(out) if out.status.success() => out,
            Ok(_) | Err(_) => {
                debug!("adb not found or adb devices failed");
                return Vec::new();
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let id = parts[0].to_string();
            let state_str = parts[1];

            let state = match state_str {
                "device" => DeviceState::Connected,
                "offline" => DeviceState::Unavailable,
                "bootloader" | "recovery" => DeviceState::Busy,
                _ => DeviceState::Unavailable,
            };

            // Parse model/device name if available
            let mut name = id.clone();
            for part in &parts[2..] {
                if let Some(m) = part.strip_prefix("model:") {
                    name = m.replace('_', " ");
                } else if let Some(d) = part.strip_prefix("device:") {
                    if name == id {
                        name = d.to_string();
                    }
                }
            }

            let is_emulator = id.starts_with("emulator-");

            let mut device = Device::new(id, name, Platform::Android, state);
            device.is_emulator = is_emulator;
            devices.push(device);
        }

        devices
    }
}
