use axum::Json;
use devflow_cli_core::shell;

pub async fn handle_shell_install() -> Json<serde_json::Value> {
    match shell::install_cli_symlink(None) {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

pub async fn handle_shell_uninstall() -> Json<serde_json::Value> {
    match shell::uninstall_cli_symlink() {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}
