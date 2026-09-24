use axum::Json;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct UpdateCheckResponse {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: String,
    pub release_notes: String,
    pub published_at: String,
    pub download_url: Option<String>,
    pub html_url: String,
    pub target_platform: String,
}

#[derive(Deserialize)]
pub struct UpdateInstallRequest {
    pub download_url: String,
}

pub fn parse_version_tuple(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_version_newer(latest: &str, current: &str) -> bool {
    let l = parse_version_tuple(latest);
    let c = parse_version_tuple(current);
    l > c
}

pub async fn handle_updater_check() -> Json<UpdateCheckResponse> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    #[cfg(target_os = "macos")]
    let os_name = "macos";
    #[cfg(target_os = "linux")]
    let os_name = "linux";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let os_name = "other";

    #[cfg(target_arch = "aarch64")]
    let arch_name = "arm64";
    #[cfg(target_arch = "x86_64")]
    let arch_name = "x86_64";
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    let arch_name = "unknown";

    let target_platform = format!("{}-{}", os_name, arch_name);
    let expected_asset = format!("devflow-gui-{}-{}.tar.gz", os_name, arch_name);

    let output = tokio::process::Command::new("curl")
        .args([
            "-sSL",
            "-H", "User-Agent: DevFlow-Desktop-Companion",
            "-H", "Accept: application/vnd.github.v3+json",
            "https://api.github.com/repos/cmdworks/devflow/releases/latest",
        ])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                let tag_name = val["tag_name"].as_str().unwrap_or("").to_string();
                let latest_clean = tag_name.trim_start_matches('v').to_string();
                let update_available = !latest_clean.is_empty() && is_version_newer(&latest_clean, &current_version);
                let release_name = val["name"].as_str().unwrap_or(&tag_name).to_string();
                let release_notes = val["body"].as_str().unwrap_or("").to_string();
                let published_at = val["published_at"].as_str().unwrap_or("").to_string();
                let html_url = val["html_url"].as_str().unwrap_or("https://github.com/cmdworks/devflow/releases").to_string();

                let mut download_url = None;
                if let Some(assets) = val["assets"].as_array() {
                    for asset in assets {
                        if let Some(name) = asset["name"].as_str() {
                            if name == expected_asset {
                                download_url = asset["browser_download_url"].as_str().map(|s| s.to_string());
                                break;
                            }
                        }
                    }
                }

                return Json(UpdateCheckResponse {
                    current_version: current_version.clone(),
                    latest_version: if latest_clean.is_empty() { current_version } else { latest_clean },
                    update_available,
                    release_name,
                    release_notes,
                    published_at,
                    download_url,
                    html_url,
                    target_platform,
                });
            }
        }
        _ => {}
    }

    Json(UpdateCheckResponse {
        current_version: current_version.clone(),
        latest_version: current_version,
        update_available: false,
        release_name: "DevFlow".to_string(),
        release_notes: String::new(),
        published_at: String::new(),
        download_url: None,
        html_url: "https://github.com/cmdworks/devflow/releases".to_string(),
        target_platform,
    })
}

pub async fn handle_updater_install(
    Json(req): Json<UpdateInstallRequest>,
) -> Json<serde_json::Value> {
    let url = req.download_url.trim().to_string();
    if url.is_empty() {
        return Json(serde_json::json!({ "success": false, "error": "Download URL cannot be empty" }));
    }

    let tmp_dir = std::env::temp_dir().join(format!(
        "devflow-update-{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()
    ));
    let _ = std::fs::create_dir_all(&tmp_dir);
    let archive_path = tmp_dir.join("update.tar.gz");

    // 1. Download archive
    let dl_status = tokio::process::Command::new("curl")
        .args(["-fSL", &url, "-o", archive_path.to_str().unwrap_or_default()])
        .status()
        .await;

    match dl_status {
        Ok(s) if s.success() => {}
        _ => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Json(serde_json::json!({ "success": false, "error": "Failed to download update package from GitHub Releases" }));
        }
    }

    // 2. Extract archive
    let ext_status = tokio::process::Command::new("tar")
        .args(["-xzf", archive_path.to_str().unwrap_or_default(), "-C", tmp_dir.to_str().unwrap_or_default()])
        .status()
        .await;

    if ext_status.is_err() || !ext_status.unwrap().success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Json(serde_json::json!({ "success": false, "error": "Failed to extract update package" }));
    }

    // 3. Platform-specific installation
    #[cfg(target_os = "macos")]
    {
        let extracted_app = tmp_dir.join("DevFlow.app");
        let app_target = if PathBuf::from("/Applications").exists() {
            PathBuf::from("/Applications/DevFlow.app")
        } else if let Some(home) = dirs::home_dir() {
            let user_apps = home.join("Applications");
            let _ = std::fs::create_dir_all(&user_apps);
            user_apps.join("DevFlow.app")
        } else {
            PathBuf::from("/Applications/DevFlow.app")
        };

        if extracted_app.exists() {
            let _ = std::fs::remove_dir_all(&app_target);
            let copy_res = std::process::Command::new("cp")
                .args(["-R", extracted_app.to_str().unwrap_or_default(), app_target.to_str().unwrap_or_default()])
                .status();

            if copy_res.is_err() || !copy_res.unwrap().success() {
                let _ = std::fs::remove_dir_all(&tmp_dir);
                return Json(serde_json::json!({ "success": false, "error": "Failed to copy DevFlow.app to Applications directory" }));
            }

            let _ = std::process::Command::new("xattr")
                .args(["-cr", app_target.to_str().unwrap_or_default()])
                .status();
        } else {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Json(serde_json::json!({ "success": false, "error": "DevFlow.app not found in downloaded update archive" }));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let extracted_bin = tmp_dir.join("devflow-gui");
        if let Some(home) = dirs::home_dir() {
            let target_bin = home.join(".local/bin/devflow-gui");
            let _ = std::fs::create_dir_all(home.join(".local/bin"));
            if extracted_bin.exists() {
                let _ = std::fs::copy(&extracted_bin, &target_bin);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755));
                }
            }
        }
    }

    let _ = std::fs::remove_dir_all(&tmp_dir);

    Json(serde_json::json!({
        "success": true,
        "message": "Update successfully downloaded and installed. Restart DevFlow to apply."
    }))
}

pub async fn handle_updater_restart() -> Json<serde_json::Value> {
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        #[cfg(target_os = "macos")]
        {
            let candidates = [
                PathBuf::from("/Applications/DevFlow.app"),
                dirs::home_dir().map(|h| h.join("Applications/DevFlow.app")).unwrap_or_default(),
            ];
            for app_path in &candidates {
                if app_path.exists() {
                    let _ = std::process::Command::new("open")
                        .arg("-n")
                        .arg("-a")
                        .arg(app_path)
                        .spawn();
                    break;
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::process::Command::new(exe).spawn();
            }
        }

        std::process::exit(0);
    });

    Json(serde_json::json!({ "success": true, "restarting": true }))
}
