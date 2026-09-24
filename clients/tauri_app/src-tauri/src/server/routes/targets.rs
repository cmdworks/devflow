use axum::{extract::State, http::StatusCode, Json};
use devflow_frameworks::kotlin::KotlinFrameworkAdapter;
use devflow_frameworks::session::SessionManager;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct StartTargetRequest {
    pub target_id: String,
    pub target_path: String,
    pub framework: String,
    pub device_id: Option<String>,
}

#[derive(Deserialize)]
pub struct TargetActionRequest {
    pub target_id: String,
}

#[derive(Deserialize)]
pub struct CustomTargetActionRequest {
    pub target_id: String,
    pub target_path: String,
    pub framework: String,
    pub action: String,
    pub device_id: Option<String>,
    pub port: Option<u16>,
}

pub async fn handle_start_target(
    State(state): State<AppState>,
    Json(req): Json<StartTargetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = PathBuf::from(&req.target_path);
    let target_id = req.target_id.clone();
    let event_bus = state.event_bus.clone();

    match SessionManager::create_with_id(
        target_id.clone(),
        &path,
        req.device_id.as_deref(),
        Some(&req.framework),
        event_bus,
    )
    .await
    {
        Ok(sess) => {
            let sess_arc = Arc::new(sess);
            let sess_clone = sess_arc.clone();
            tokio::spawn(async move {
                let _ = sess_clone.start_session().await;
            });

            state.active_sessions.lock().await.insert(target_id.clone(), sess_arc);
            Ok(Json(serde_json::json!({ "success": true, "target_id": target_id })))
        }
        Err(e) => {
            warn!("Failed to start target {}: {}", req.target_id, e);
            Ok(Json(serde_json::json!({ "success": false, "error": e.to_string() })))
        }
    }
}

pub async fn handle_stop_target(
    State(state): State<AppState>,
    Json(req): Json<TargetActionRequest>,
) -> Json<serde_json::Value> {
    let mut sessions = state.active_sessions.lock().await;
    if let Some(sess) = sessions.remove(&req.target_id) {
        let _ = sess.stop().await;
        Json(serde_json::json!({ "success": true, "message": format!("Stopped target {}", req.target_id) }))
    } else {
        // Fallback: kill any orphan process matching target ID or binary name
        let target_name = req.target_id.clone();
        #[cfg(unix)]
        {
            let _ = tokio::process::Command::new("pkill")
                .arg("-9")
                .arg("-f")
                .arg(&target_name)
                .output()
                .await;
        }
        Json(serde_json::json!({ "success": true, "message": format!("Cleaned up target {}", req.target_id) }))
    }
}

pub async fn handle_reload_target(
    State(state): State<AppState>,
    Json(req): Json<TargetActionRequest>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    if let Some(sess) = sessions.get(&req.target_id) {
        let _ = sess.reload(vec![]).await;
        Json(serde_json::json!({ "success": true, "message": format!("Reloaded {}", req.target_id) }))
    } else {
        Json(serde_json::json!({ "success": false, "message": "Target not running" }))
    }
}

pub async fn handle_restart_target(
    State(state): State<AppState>,
    Json(req): Json<TargetActionRequest>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    if let Some(sess) = sessions.get(&req.target_id) {
        let _ = sess.restart().await;
        Json(serde_json::json!({ "success": true, "message": format!("Restarted {}", req.target_id) }))
    } else {
        Json(serde_json::json!({ "success": false, "message": "Target not running" }))
    }
}

pub async fn handle_reload_all(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    for sess in sessions.values() {
        let _ = sess.reload(vec![]).await;
    }
    Json(serde_json::json!({ "success": true, "message": format!("Reloaded {} active targets", sessions.len()) }))
}

pub async fn handle_restart_all(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    for sess in sessions.values() {
        let _ = sess.restart().await;
    }
    Json(serde_json::json!({ "success": true, "message": format!("Restarted {} active targets", sessions.len()) }))
}

pub async fn handle_custom_target_action(
    State(_state): State<AppState>,
    Json(req): Json<CustomTargetActionRequest>,
) -> Json<serde_json::Value> {
    let path = PathBuf::from(&req.target_path);
    let fw = req.framework.to_lowercase();
    let action = req.action.as_str();

    info!("Executing custom action '{}' for target '{}' ({})", action, req.target_id, fw);

    match action {
        "launch_app" => {
            if fw.contains("kotlin") || fw.contains("android") {
                let (pkg, activity) = KotlinFrameworkAdapter::inspect_android_project(&path);
                if let Some(p) = pkg {
                    let mut cmd = Command::new("adb");
                    if let Some(ref d) = req.device_id {
                        cmd.args(["-s", d]);
                    }
                    if let Some(act) = activity {
                        let comp = if act.starts_with('.') {
                            format!("{}{}", p, act)
                        } else if act.contains('/') {
                            act
                        } else {
                            format!("{}/{}", p, act)
                        };
                        cmd.args(["shell", "am", "start", "-n", &comp]);
                    } else {
                        cmd.args(["shell", "monkey", "-p", &p, "-c", "android.intent.category.LAUNCHER", "1"]);
                    }
                    let out = cmd.output().await;
                    match out {
                        Ok(o) if o.status.success() => {
                            let msg = String::from_utf8_lossy(&o.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                            if msg.contains("Error:") || msg.contains("does not exist") || stderr.contains("no devices") {
                                return Json(serde_json::json!({
                                    "success": false,
                                    "error": if !stderr.is_empty() { stderr.trim().to_string() } else { msg.trim().to_string() }
                                }));
                            }
                            return Json(serde_json::json!({
                                "success": true,
                                "message": format!("Launched {}: {}", p, msg.trim())
                            }));
                        }
                        Ok(o) => {
                            let err = String::from_utf8_lossy(&o.stderr).to_string();
                            return Json(serde_json::json!({
                                "success": false,
                                "error": if !err.is_empty() { err.trim().to_string() } else { "ADB am start failed".to_string() }
                            }));
                        }
                        Err(e) => {
                            return Json(serde_json::json!({
                                "success": false,
                                "error": format!("Failed to run adb: {}", e)
                            }));
                        }
                    }
                }
            } else if fw.contains("swift") {
                let bin = path.join(".build/debug").join(&req.target_id);
                if bin.exists() {
                    let _ = Command::new("open").arg(&bin).output().await;
                    return Json(serde_json::json!({ "success": true, "message": format!("Launched binary {}", bin.display()) }));
                } else {
                    return Json(serde_json::json!({ "success": false, "error": format!("Binary not found at {}. Build with Run first.", bin.display()) }));
                }
            } else if fw.contains("web") || fw.contains("generic") || fw.contains("node") || fw.contains("react") {
                let port = req.port.unwrap_or(3000);
                let url = format!("http://localhost:{}", port);
                let _ = Command::new("open").arg(&url).output().await;
                return Json(serde_json::json!({ "success": true, "message": format!("Opened {}", url) }));
            }
            Json(serde_json::json!({ "success": true, "message": "Launch requested" }))
        }
        "force_stop" => {
            if fw.contains("kotlin") || fw.contains("android") {
                let (pkg, _) = KotlinFrameworkAdapter::inspect_android_project(&path);
                if let Some(p) = pkg {
                    let mut cmd = Command::new("adb");
                    if let Some(ref d) = req.device_id {
                        cmd.args(["-s", d]);
                    }
                    cmd.args(["shell", "am", "force-stop", &p]);
                    let _ = cmd.output().await;
                    return Json(serde_json::json!({ "success": true, "message": format!("Force stopped package {}", p) }));
                }
            }
            #[cfg(unix)]
            {
                let _ = Command::new("pkill").arg("-9").arg("-f").arg(&req.target_id).output().await;
            }
            Json(serde_json::json!({ "success": true, "message": format!("Force stopped target {}", req.target_id) }))
        }
        "clear_data" => {
            if fw.contains("kotlin") || fw.contains("android") {
                let (pkg, _) = KotlinFrameworkAdapter::inspect_android_project(&path);
                if let Some(p) = pkg {
                    let mut cmd = Command::new("adb");
                    if let Some(ref d) = req.device_id {
                        cmd.args(["-s", d]);
                    }
                    cmd.args(["shell", "pm", "clear", &p]);
                    let _ = cmd.output().await;
                    return Json(serde_json::json!({ "success": true, "message": format!("Cleared data for {}", p) }));
                }
            }
            Json(serde_json::json!({ "success": false, "error": "Clear data only supported for Android targets" }))
        }
        "reinstall_apk" => {
            if fw.contains("kotlin") || fw.contains("android") {
                // 1. Fast path: If an APK is already built, install directly via ADB (super fast, <1s)
                if let Some(apk_path) = KotlinFrameworkAdapter::find_built_apk(&path, false)
                    .or_else(|| KotlinFrameworkAdapter::find_built_apk(&path, true))
                {
                    let mut adb_cmd = Command::new("adb");
                    if let Some(ref d) = req.device_id {
                        adb_cmd.args(["-s", d]);
                    }
                    adb_cmd.args(["install", "-r"]).arg(&apk_path);
                    if let Ok(out) = adb_cmd.output().await {
                        if out.status.success() {
                            let apk_name = apk_path.file_name().and_then(|n| n.to_str()).unwrap_or("app.apk");
                            return Json(serde_json::json!({
                                "success": true,
                                "message": format!("Directly installed {} to device via ADB.", apk_name)
                            }));
                        }
                    }
                }

                // 2. Build path: If APK not found or direct install failed, compile with Gradle
                let gradlew = if path.join("gradlew").exists() { "./gradlew" } else { "gradle" };
                let mut cmd = Command::new(gradlew);
                cmd.arg("installDebug").current_dir(&path);
                let out = cmd.output().await;
                match out {
                    Ok(o) if o.status.success() => {
                        return Json(serde_json::json!({
                            "success": true,
                            "message": "Successfully compiled & installed APK on device via Gradle."
                        }));
                    }
                    Ok(o) => {
                        let err = String::from_utf8_lossy(&o.stderr);
                        return Json(serde_json::json!({
                            "success": false,
                            "error": format!("Gradle installDebug failed: {}", err)
                        }));
                    }
                    Err(e) => {
                        return Json(serde_json::json!({
                            "success": false,
                            "error": format!("Failed to execute {}: {}", gradlew, e)
                        }));
                    }
                }
            }
            Json(serde_json::json!({ "success": false, "error": "Install APK only supported for Android targets" }))
        }
        "clean_cache" | "clean_build" => {
            if fw.contains("kotlin") || fw.contains("android") {
                let gradlew = if path.join("gradlew").exists() { "./gradlew" } else { "gradle" };
                let _ = Command::new(gradlew).arg("clean").current_dir(&path).output().await;
                return Json(serde_json::json!({ "success": true, "message": "Gradle clean finished" }));
            } else if fw.contains("swift") {
                let _ = Command::new("swift").args(["package", "clean"]).current_dir(&path).output().await;
                return Json(serde_json::json!({ "success": true, "message": "Swift package clean finished" }));
            } else if fw.contains("rust") {
                let _ = Command::new("cargo").arg("clean").current_dir(&path).output().await;
                return Json(serde_json::json!({ "success": true, "message": "Cargo clean finished" }));
            } else {
                let _ = Command::new("rm").args(["-rf", "node_modules/.vite", ".next", "dist"]).current_dir(&path).output().await;
                return Json(serde_json::json!({ "success": true, "message": "Cleaned build caches" }));
            }
        }
        "open_ide" => {
            if fw.contains("kotlin") || fw.contains("android") {
                let res = Command::new("open").args(["-a", "Android Studio"]).arg(&path).output().await;
                if res.is_err() || !res.as_ref().unwrap().status.success() {
                    let _ = Command::new("open").arg(&path).output().await;
                }
                return Json(serde_json::json!({ "success": true, "message": "Opened in Android Studio" }));
            } else if fw.contains("swift") {
                let res = Command::new("open").args(["-a", "Xcode"]).arg(&path).output().await;
                if res.is_err() || !res.as_ref().unwrap().status.success() {
                    let _ = Command::new("open").arg(&path).output().await;
                }
                return Json(serde_json::json!({ "success": true, "message": "Opened in Xcode" }));
            } else {
                let res = Command::new("open").args(["-a", "Visual Studio Code"]).arg(&path).output().await;
                if res.is_err() || !res.as_ref().unwrap().status.success() {
                    let _ = Command::new("open").arg(&path).output().await;
                }
                return Json(serde_json::json!({ "success": true, "message": "Opened project in IDE" }));
            }
        }
        "free_port" => {
            let port = req.port.unwrap_or(3000);
            #[cfg(unix)]
            {
                let cmd = format!("lsof -ti :{} | xargs kill -9", port);
                let _ = Command::new("sh").args(["-c", &cmd]).output().await;
            }
            Json(serde_json::json!({ "success": true, "message": format!("Freed port {}", port) }))
        }
        "reveal_finder" => {
            let _ = Command::new("open").args(["-R"]).arg(&path).output().await;
            Json(serde_json::json!({ "success": true, "message": "Revealed in macOS Finder" }))
        }
        _ => {
            Json(serde_json::json!({ "success": false, "error": format!("Unknown action '{}'", action) }))
        }
    }
}
