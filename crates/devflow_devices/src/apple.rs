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
                            let os_name = runtime.split('.').next_back().unwrap_or(runtime);
                            if let Some(list) = dev_list.as_array() {
                                for dev in list {
                                    let is_available = dev
                                        .get("isAvailable")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(true);
                                    if !is_available {
                                        continue;
                                    }

                                    let udid =
                                        dev.get("udid").and_then(|u| u.as_str()).unwrap_or("");
                                    let name = dev
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("Simulator");
                                    let state_str =
                                        dev.get("state").and_then(|s| s.as_str()).unwrap_or("");

                                    let state = match state_str {
                                        "Booted" => DeviceState::Booted,
                                        "Shutdown" => DeviceState::Shutdown,
                                        _ => DeviceState::Unavailable,
                                    };

                                    let mut d = Device::new(
                                        udid,
                                        format!("{} ({})", name, os_name),
                                        Platform::Ios,
                                        state,
                                    );
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
            .args([
                "devicectl",
                "list",
                "devices",
                "--json-output",
                "/dev/stdout",
            ])
            .output()
            .await
        {
            if output.status.success() {
                if let Ok(json) = serde_json::from_slice::<Value>(&output.stdout) {
                    if let Some(list) = json.pointer("/result/devices").and_then(|d| d.as_array()) {
                        for dev in list {
                            let identifier = dev
                                .pointer("/hardwareProperties/udid")
                                .or_else(|| dev.get("identifier"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("");
                            let name = dev
                                .pointer("/deviceProperties/name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Apple Device");
                            let state_str = dev
                                .pointer("/connectionProperties/transportType")
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

    pub async fn boot_simulator(target: &str) -> Result<String, String> {
        let xcrun_bin = crate::emulator::which_binary("xcrun")
            .or_else(|_| {
                let candidate = std::path::PathBuf::from("/usr/bin/xcrun");
                if candidate.exists() {
                    Ok(candidate)
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "xcrun not found"))
                }
            })
            .map_err(|_| {
                "Xcode Command Line Tools ('xcrun' / 'simctl') not found. Please ensure Xcode is installed and run 'xcode-select --install'.".to_string()
            })?;

        // Resolve target to UDID if it was passed as a simulator name
        let mut udid = target.to_string();
        let is_udid = target.contains('-') && target.len() >= 36;
        if !is_udid {
            let discovered = Self::discover().await;
            if let Some(matched) = discovered.iter().find(|d| {
                d.id == target
                    || d.name.eq_ignore_ascii_case(target)
                    || d.name.to_lowercase().starts_with(&target.to_lowercase())
            }) {
                udid = matched.id.clone();
            } else {
                let available_names: Vec<String> = discovered
                    .iter()
                    .filter(|d| d.is_emulator)
                    .map(|d| d.name.clone())
                    .collect();
                let names_str = if available_names.is_empty() {
                    "none (install runtimes in Xcode > Settings > Platforms)".to_string()
                } else {
                    available_names.join(", ")
                };
                return Err(format!(
                    "iOS Simulator '{}' not found in xcrun simctl. Available simulators: [{}].",
                    target, names_str
                ));
            }
        }

        info!("Booting Apple iOS Simulator '{}'...", udid);

        let output = Command::new(&xcrun_bin)
            .args(["simctl", "boot", &udid])
            .output()
            .await
            .map_err(|e| format!("Failed to execute 'xcrun simctl boot': {}", e))?;

        // Also open Simulator.app GUI
        let _ = Command::new("open")
            .args(["-a", "Simulator", "--args", "-CurrentDeviceUDID", &udid])
            .output()
            .await;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success()
            || stderr.contains("booted")
            || stderr.contains("current state: Booted")
        {
            Ok(format!("Simulator '{}' booted successfully", udid))
        } else {
            Err(format!(
                "Failed to boot simulator '{}': {}",
                udid,
                stderr.trim()
            ))
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
