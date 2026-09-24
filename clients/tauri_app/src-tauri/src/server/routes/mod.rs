pub mod devices;
pub mod dialog;
pub mod doctor;
pub mod events;
pub mod mcp;
pub mod shell;
pub mod targets;
pub mod updater;
pub mod workspace;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::server::assets::{fallback_handler, index_handler};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        // Workspace
        .route("/api/workspace", get(workspace::handle_workspace))
        .route(
            "/api/workspaces",
            get(workspace::handle_list_workspaces)
                .post(workspace::handle_add_workspace)
                .delete(workspace::handle_remove_workspace),
        )
        // Devices
        .route("/api/devices", get(devices::handle_devices))
        .route("/api/devices/boot", post(devices::handle_boot_emulator))
        // Diagnostics
        .route("/api/doctor", get(doctor::handle_doctor))
        // MCP Hub
        .route("/api/mcp/status", get(mcp::handle_mcp_status))
        .route("/api/mcp/toggle", post(mcp::handle_mcp_toggle))
        .route("/api/mcp/logs", get(mcp::handle_mcp_logs).delete(mcp::handle_delete_mcp_logs))
        .route("/api/mcp/sessions", get(mcp::handle_mcp_sessions))
        .route("/api/mcp/agents", get(mcp::handle_mcp_agents))
        .route("/rpc", post(mcp::handle_mcp_rpc))
        // Shell CLI Integration
        .route("/api/shell/install", post(shell::handle_shell_install))
        .route("/api/shell/uninstall", post(shell::handle_shell_uninstall))
        // Targets Lifecycle
        .route("/api/target/start", post(targets::handle_start_target))
        .route("/api/target/stop", post(targets::handle_stop_target))
        .route("/api/target/reload", post(targets::handle_reload_target))
        .route("/api/target/restart", post(targets::handle_restart_target))
        .route("/api/target/action", post(targets::handle_custom_target_action))
        .route("/api/workspace/reload-all", post(targets::handle_reload_all))
        .route("/api/workspace/restart-all", post(targets::handle_restart_all))
        // Native Dialog
        .route("/api/dialog/pick-folder", post(dialog::handle_pick_folder))
        // Software Updater
        .route("/api/updater/check", get(updater::handle_updater_check))
        .route("/api/updater/install", post(updater::handle_updater_install))
        .route("/api/updater/restart", post(updater::handle_updater_restart))
        // SSE Real-time Events
        .route("/api/events", get(events::handle_events_sse))
        // Static Single-Page App Assets
        .route("/", get(index_handler))
        .fallback(fallback_handler)
        .layer(cors)
        .layer(axum::middleware::from_fn(
            |req: axum::extract::Request, next: axum::middleware::Next| async move {
                let path = req.uri().path().to_string();
                let is_polling_route = path == "/api/mcp/logs"
                    || path == "/api/mcp/sessions"
                    || path == "/api/mcp/agents"
                    || path == "/api/events"
                    || path == "/api/devices";

                let method = req.method().clone();
                let start = std::time::Instant::now();
                let response = next.run(req).await;
                let duration = start.elapsed();
                let status = response.status();

                if is_polling_route {
                    tracing::debug!(
                        target: "devflow_desktop",
                        "HTTP {} {} -> {} ({:?})",
                        method,
                        path,
                        status,
                        duration
                    );
                } else if !path.starts_with("/assets/") && path != "/favicon.ico" {
                    tracing::info!(
                        target: "devflow_desktop",
                        "HTTP {} {} -> {} ({:?})",
                        method,
                        path,
                        status,
                        duration
                    );
                }

                response
            },
        ))
        .with_state(state)
}
