pub mod adb;
pub mod apple;
pub mod desktop;
pub mod emulator;

pub use adb::AdbDiscoverer;
pub use apple::AppleDiscoverer;
pub use desktop::DesktopDiscoverer;
use devflow_protocol::{Device, Platform};
pub use emulator::EmulatorManager;

pub struct DeviceManager;

impl DeviceManager {
    pub async fn discover_all() -> Vec<Device> {
        let mut all_devices = Vec::new();

        // 1. Host Desktop is always available
        all_devices.extend(DesktopDiscoverer::discover().await);

        // 2. Android Devices / Running Emulators
        let running_adb = AdbDiscoverer::discover().await;
        all_devices.extend(running_adb);

        // 3. Android Virtual Devices (AVDs available to boot)
        let avds = EmulatorManager::discover_avd_devices().await;
        for avd in avds {
            if !all_devices
                .iter()
                .any(|d| d.id == avd.id || d.name == avd.name)
            {
                all_devices.push(avd);
            }
        }

        // 4. Apple Devices / Simulators (if on macOS or xcrun available)
        all_devices.extend(AppleDiscoverer::discover().await);

        all_devices
    }

    pub async fn find_best_match(
        preferred_platform: Option<Platform>,
        target_id: Option<&str>,
    ) -> Option<Device> {
        let devices = Self::discover_all().await;

        if let Some(id) = target_id {
            if let Some(d) = devices
                .iter()
                .find(|d| d.id == id || d.name.eq_ignore_ascii_case(id))
            {
                return Some(d.clone());
            }
        }

        if let Some(pref) = preferred_platform {
            // First look for connected/booted device matching platform
            if let Some(d) = devices.iter().find(|d| {
                (d.platform == pref || (pref == Platform::Ios && d.platform == Platform::Apple))
                    && (d.state == devflow_protocol::DeviceState::Connected
                        || d.state == devflow_protocol::DeviceState::Booted)
            }) {
                return Some(d.clone());
            }

            // Then any device matching platform
            if let Some(d) = devices.iter().find(|d| {
                d.platform == pref || (pref == Platform::Ios && d.platform == Platform::Apple)
            }) {
                return Some(d.clone());
            }
        }

        // Fallback to desktop host
        devices
            .into_iter()
            .find(|d| d.platform == Platform::Desktop)
    }

    pub async fn boot_device(target_id: &str) -> Result<String, String> {
        // If it's a simulator UDID or iOS simulator name
        if target_id.contains('-') && target_id.len() >= 36 {
            return AppleDiscoverer::boot_simulator(target_id).await;
        }

        // Otherwise try AVD boot
        let clean_name = target_id.strip_prefix("avd:").unwrap_or(target_id);
        EmulatorManager::boot_avd(clean_name).await
    }

    pub async fn boot_emulator(name: &str) -> Result<String, String> {
        Self::boot_device(name).await
    }
}
