use devflow_protocol::{Device, DeviceState, Platform};
use serde_json::Value;
use tokio::process::Command;
use tracing::info;

pub struct AppleDiscoverer;

impl AppleDiscoverer {
    pub async fn discover() -> Vec<Device> {
        let mut devices = Vec::new();

        // 1. Simulators via xcrun simctl
        if let Ok(output) = Command::new("xcrun")
            .args(["simctl", "list", "devices", "--json"])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(dev_map) = json.get("devices").and_then(|d| d.as_object()) {
                        for (runtime, dev_list) in dev_map {
                            let os_name = runtime.split('.').last().unwrap_or(runtime);
                            if let Some(list) = dev_list.as_array() {
                                for dev in list {
                                    let is_available = dev
                                        .get("isAvailable")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true);
                                    if !is_available {
                                        continue;
                                    }

                                    let udid = dev.get("udid").and_then(|u| u.as_str()).unwrap_or("");
                                    let name = dev.get("name").and_then(|n| n.as_str()).unwrap_or("Simulator");
                                    let state_str = dev.get("state").and_then(|s| s.as_str()).unwrap_or("");

                                    let state = match state_str {
                                        "Booted" => DeviceState::Booted,
                                        "Shutdown" => DeviceState::Shutdown,
                                        _ => DeviceState::Unavailable,
                                    };

                                    let mut d = Device::new(udid, format!("{} ({})", name, os_name), Platform::Ios, state);
                                    d.is_emulator = true;
                                    d.os_version = Some(os_name.to_string());
                                    devices.push(d);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Physical iOS devices via devicectl (iOS 17+)
        if let Ok(output) = Command::new("xcrun")
            .args(["devicectl", "list", "devices", "--json-output", "/dev/stdout"])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(list) = json.pointer("/result/devices").and_then(|d| d.as_array()) {
                        for dev in list {
                            let identifier = dev.pointer("/hardwareProperties/udid")
                                .or_else(|| dev.get("identifier"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("");
                            let name = dev.pointer("/deviceProperties/name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Apple Device");
                            let state_str = dev.pointer("/connectionProperties/transportType")
                                .and_then(|s| s.as_str())
                                .unwrap_or("connected");

                            let state = if !state_str.is_empty() {
                                DeviceState::Connected
                            } else {
                                DeviceState::Unavailable
                            };

                            let mut d = Device::new(identifier, name, Platform::Ios, state);
                            d.is_emulator = false;
                            devices.push(d);
                        }
                    }
                }
            }
        }

        devices
    }

    pub async fn boot_simulator(udid: &str) -> Result<String, String> {
        info!("Booting Apple iOS Simulator '{}'...", udid);

        let output = Command::new("xcrun")
            .args(["simctl", "boot", udid])
            .output()
            .await
            .map_err(|e| format!("Failed to boot simulator: {}", e))?;

        // Also open Simulator.app GUI
        let _ = Command::new("open")
            .args(["-a", "Simulator", "--args", "-CurrentDeviceUDID", udid])
            .output()
            .await;

        if output.status.success() || String::from_utf8_lossy(&output.stderr).contains("booted") {
            Ok(format!("Simulator '{}' booted successfully", udid))
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to boot simulator: {}", err))
        }
    }

    pub async fn shutdown_simulator(udid: &str) -> Result<String, String> {
        info!("Shutting down Apple iOS Simulator '{}'...", udid);

        let output = Command::new("xcrun")
            .args(["simctl", "shutdown", udid])
            .output()
            .await
            .map_err(|e| format!("Failed to shutdown simulator: {}", e))?;

        if output.status.success() {
            Ok(format!("Simulator '{}' shut down", udid))
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("Shutdown returned: {}", err))
        }
    }
}
