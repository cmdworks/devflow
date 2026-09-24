use axum::Json;

#[cfg(target_os = "macos")]
static MACOS_DIALOG_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/devflow-dialog-macos"));

pub async fn handle_pick_folder() -> Json<serde_json::Value> {
    #[cfg(target_os = "macos")]
    {
        let res = tokio::task::spawn_blocking(|| -> Result<String, anyhow::Error> {
            // 1. Ensure /tmp/devflow-dialog-macos binary exists and has execute permissions
            let tmp_bin = std::path::PathBuf::from("/tmp/devflow-dialog-macos");
            if !tmp_bin.exists()
                && !MACOS_DIALOG_BIN.is_empty()
                && std::fs::write(&tmp_bin, MACOS_DIALOG_BIN).is_ok()
            {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ =
                        std::fs::set_permissions(&tmp_bin, std::fs::Permissions::from_mode(0o755));
                }
            }

            // 2. Fast compiled native Cocoa NSOpenPanel helper (< 10ms launch)
            let helper_candidates = [
                Some(tmp_bin),
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|dir| dir.join("devflow-dialog-macos"))),
            ];

            for candidate in helper_candidates.into_iter().flatten() {
                if candidate.exists() {
                    if let Ok(output) = std::process::Command::new(&candidate).output() {
                        if output.status.success() {
                            let raw = String::from_utf8_lossy(&output.stdout);
                            let path_str = raw.trim().trim_end_matches('/').to_string();
                            if !path_str.is_empty() {
                                return Ok(path_str);
                            }
                        } else {
                            // User clicked cancel in native dialog
                            return Ok(String::new());
                        }
                    }
                }
            }

            // 3. Fallback to AppleScript with foreground activation
            let script = r#"
                tell application "System Events"
                    activate
                    try
                        set chosenFolder to choose folder with prompt "Select DevFlow Workspace Directory"
                        return POSIX path of chosenFolder
                    on error number -128
                        return ""
                    end try
                end tell
            "#;
            let out = std::process::Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output()?;
            if out.status.success() {
                let raw = String::from_utf8_lossy(&out.stdout);
                Ok(raw.trim().trim_end_matches('/').to_string())
            } else {
                Ok(String::new())
            }
        })
        .await;

        if let Ok(Ok(path_str)) = res {
            if !path_str.is_empty() {
                return Json(serde_json::json!({ "success": true, "path": path_str }));
            }
        }
        Json(serde_json::json!({ "success": false }))
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(folder) = rfd::AsyncFileDialog::new()
            .set_title("Select DevFlow Workspace Directory")
            .pick_folder()
            .await
        {
            let path_str = folder.path().display().to_string();
            return Json(serde_json::json!({ "success": true, "path": path_str }));
        }
        Json(serde_json::json!({ "success": false }))
    }
}
