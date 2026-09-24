use axum::{http::StatusCode, Json};
use devflow_devices::DeviceManager;
use devflow_protocol::Device;
use serde::Deserialize;

static CACHED_DEVICES: std::sync::LazyLock<std::sync::RwLock<Vec<Device>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(Vec::new()));
static DEVICE_CACHE_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn get_cached_devices() -> Vec<Device> {
    if let Ok(lock) = CACHED_DEVICES.read() {
        lock.clone()
    } else {
        Vec::new()
    }
}

pub async fn get_or_refresh_devices() -> Vec<Device> {
    if DEVICE_CACHE_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        let cached = get_cached_devices();
        tokio::spawn(async {
            let fresh = DeviceManager::discover_all().await;
            if let Ok(mut lock) = CACHED_DEVICES.write() {
                *lock = fresh;
            }
        });
        return cached;
    }

    let desktop = devflow_devices::DesktopDiscoverer::discover().await;
    if let Ok(mut lock) = CACHED_DEVICES.write() {
        *lock = desktop.clone();
    }
    DEVICE_CACHE_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);

    tokio::spawn(async {
        let fresh = DeviceManager::discover_all().await;
        if let Ok(mut lock) = CACHED_DEVICES.write() {
            *lock = fresh;
        }
    });

    desktop
}

#[derive(Deserialize)]
pub struct BootEmulatorRequest {
    pub name: String,
}

pub async fn handle_devices() -> Json<Vec<Device>> {
    let devices = DeviceManager::discover_all().await;
    if let Ok(mut lock) = CACHED_DEVICES.write() {
        *lock = devices.clone();
    }
    DEVICE_CACHE_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
    Json(devices)
}

pub async fn handle_boot_emulator(
    Json(req): Json<BootEmulatorRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match DeviceManager::boot_emulator(&req.name).await {
        Ok(msg) => Ok(Json(serde_json::json!({ "success": true, "message": msg }))),
        Err(e) => Ok(Json(serde_json::json!({ "success": false, "error": e.to_string() }))),
    }
}
