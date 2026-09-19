use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use colored::*;
use devflow_cli_core::{run_cli_args, shell, CliAction};
use devflow_core::doctor::DoctorEngine;
use devflow_core::event::EventBus;
use devflow_core::project::{Project, ProjectTarget};
use devflow_core::registry::GlobalRegistry;
use devflow_devices::DeviceManager;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::{Device, DoctorReport};
use devflow_tui::TuiRunner;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    event_bus: EventBus,
    active_sessions: Arc<Mutex<HashMap<String, Arc<SessionManager>>>>,
}

#[derive(Deserialize)]
struct WorkspaceQuery {
    dir: Option<String>,
}

#[derive(Serialize)]
struct WorkspaceResponse {
    workspace_name: String,
    workspace_path: String,
    targets: Vec<ProjectTarget>,
    devices: Vec<Device>,
    active_sessions: Vec<devflow_core::registry::ActiveSessionInfo>,
}

#[derive(Deserialize)]
struct StartTargetRequest {
    target_id: String,
    target_path: String,
    framework: String,
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct TargetActionRequest {
    target_id: String,
}

#[derive(Deserialize)]
struct BootEmulatorRequest {
    name: String,
}

fn main() {
    devflow_core::init_environment();
    let raw_args: Vec<String> = std::env::args().collect();

    // If CLI arguments were provided beyond binary name, evaluate via shared CLI engine
    if raw_args.len() > 1 {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        let action_result = rt.block_on(async {
            run_cli_args(raw_args).await
        });

        match action_result {
            Ok(CliAction::Executed) => return,
            Ok(CliAction::LaunchTui { dir }) => {
                if let Err(e) = rt.block_on(TuiRunner::run_hub(dir)) {
                    eprintln!("TUI Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            Ok(CliAction::LaunchGui { dir, port, open_browser }) => {
                drop(rt);
                if let Err(e) = run_desktop_app(dir, port, open_browser) {
                    eprintln!("GUI Error: {}", e);
                    std::process::exit(1);
                }
                return;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Default: launched without args -> launch Desktop GUI window & backend
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9292);

    if let Err(e) = run_desktop_app(current_dir, port, false) {
        eprintln!("Desktop GUI Error: {}", e);
        std::process::exit(1);
    }
}

fn run_desktop_app(workspace_dir: PathBuf, base_port: u16, open_browser: bool) -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init();

    // 1. Start embedded Axum REST & SSE backend on a background thread with dedicated Tokio runtime
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create background Tokio runtime");
        rt.block_on(async move {
            let event_bus = EventBus::new(2000);
            let state = AppState {
                event_bus,
                active_sessions: Arc::new(Mutex::new(HashMap::new())),
            };

            let cors = CorsLayer::permissive();

            let app = Router::new()
                .route("/", get(handle_index))
                .route("/index.html", get(handle_index))
                .route("/style.css", get(handle_style))
                .route("/app.js", get(handle_app_js))
                .route("/terminal-engine.js", get(handle_terminal_engine_js))
                .route("/api/workspace", get(handle_workspace))
                .route("/api/devices", get(handle_devices))
                .route("/api/devices/boot", post(handle_boot_emulator))
                .route("/api/doctor", get(handle_doctor))
                .route("/api/shell/install", post(handle_shell_install))
                .route("/api/shell/uninstall", post(handle_shell_uninstall))
                .route("/api/target/start", post(handle_start_target))
                .route("/api/target/stop", post(handle_stop_target))
                .route("/api/target/reload", post(handle_reload_target))
                .route("/api/target/restart", post(handle_restart_target))
                .route("/api/workspace/reload-all", post(handle_reload_all))
                .route("/api/workspace/restart-all", post(handle_restart_all))
                .route("/api/events", get(handle_events_sse))
                .layer(cors)
                .with_state(state);

            // Attempt to bind starting from base_port up to base_port + 20
            let mut bound_listener = None;
            let mut actual_port = base_port;

            for offset in 0..20 {
                let candidate_port = base_port + offset;
                let addr = SocketAddr::from(([127, 0, 0, 1], candidate_port));
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => {
                        actual_port = candidate_port;
                        bound_listener = Some((listener, addr));
                        break;
                    }
                    Err(_) => continue,
                }
            }

            match bound_listener {
                Some((listener, addr)) => {
                    info!("DevFlow backend server listening on http://{}", addr);
                    let _ = ready_tx.send(Ok(actual_port));
                    if let Err(e) = axum::serve(listener, app).await {
                        eprintln!("Axum server error: {}", e);
                    }
                }
                None => {
                    let err_msg = format!("Failed to find open port between {} and {}", base_port, base_port + 20);
                    eprintln!("{}", err_msg);
                    let _ = ready_tx.send(Err(err_msg));
                }
            }
        });
    });

    // Wait until server is bound and retrieve the actual port
    let bound_port = match ready_rx.recv() {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => return Err(anyhow::anyhow!("{}", e)),
        Err(e) => return Err(anyhow::anyhow!("Failed to initialize backend thread: {}", e)),
    };

    let url_str = format!("http://localhost:{}?dir={}", bound_port, urlencoding_simple(&workspace_dir.display().to_string()));
    let url_for_nav = url_str.clone();

    println!("\n{}", "═══ DevFlow Desktop Companion App ═══".cyan().bold());
    println!("⚡ Native Desktop Window & Server running at: {}", url_str.underline());
    println!("Workspace: {}", workspace_dir.display().to_string().dimmed());

    if open_browser {
        let _ = open::that(&url_str);
    }

    // 2. Launch Tauri v2 native window on the main thread and navigate to the running backend
    tauri::Builder::default()
        .setup(move |app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(parsed_url) = url_for_nav.parse() {
                    let _ = window.navigate(parsed_url);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(' ', "%20").replace('/', "%2F")
}

async fn handle_index() -> impl IntoResponse {
    let content = include_str!("../../ui/index.html");
    Html(content)
}

async fn handle_style() -> impl IntoResponse {
    let content = include_str!("../../ui/style.css");
    ([(axum::http::header::CONTENT_TYPE, "text/css")], content)
}

async fn handle_app_js() -> impl IntoResponse {
    let content = include_str!("../../ui/app.js");
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], content)
}

async fn handle_terminal_engine_js() -> impl IntoResponse {
    let content = include_str!("../../ui/terminal-engine.js");
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], content)
}

async fn handle_workspace(
    Query(query): Query<WorkspaceQuery>,
) -> Json<WorkspaceResponse> {
    let current_dir = query
        .dir
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let workspace_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());

    let targets = Project::discover_workspace_targets(&current_dir);
    let devices = DeviceManager::discover_all().await;
    let active_sessions = GlobalRegistry::list_active_sessions();

    Json(WorkspaceResponse {
        workspace_name,
        workspace_path: current_dir.display().to_string(),
        targets,
        devices,
        active_sessions,
    })
}

async fn handle_devices() -> Json<Vec<Device>> {
    let devices = DeviceManager::discover_all().await;
    Json(devices)
}

async fn handle_boot_emulator(
    Json(req): Json<BootEmulatorRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match DeviceManager::boot_emulator(&req.name).await {
        Ok(msg) => Ok(Json(serde_json::json!({ "success": true, "message": msg }))),
        Err(e) => Ok(Json(serde_json::json!({ "success": false, "error": e.to_string() }))),
    }
}

async fn handle_doctor(
    Query(query): Query<WorkspaceQuery>,
) -> Json<DoctorReport> {
    let dir = query.dir.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let report = DoctorEngine::run_diagnostics(&dir).await;
    Json(report)
}

async fn handle_shell_install() -> Json<serde_json::Value> {
    match shell::install_cli_symlink(None) {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn handle_shell_uninstall() -> Json<serde_json::Value> {
    match shell::uninstall_cli_symlink() {
        Ok(msg) => Json(serde_json::json!({ "success": true, "message": msg })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": e.to_string() })),
    }
}

async fn handle_start_target(
    State(state): State<AppState>,
    Json(req): Json<StartTargetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = PathBuf::from(&req.target_path);
    let target_id = req.target_id.clone();
    let event_bus = state.event_bus.clone();

    match SessionManager::create_with_id(target_id.clone(), &path, req.device_id.as_deref(), Some(&req.framework), event_bus).await {
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

async fn handle_stop_target(
    State(state): State<AppState>,
    Json(req): Json<TargetActionRequest>,
) -> Json<serde_json::Value> {
    let mut sessions = state.active_sessions.lock().await;
    if let Some(sess) = sessions.remove(&req.target_id) {
        let _ = sess.stop().await;
        Json(serde_json::json!({ "success": true, "message": format!("Stopped target {}", req.target_id) }))
    } else {
        Json(serde_json::json!({ "success": false, "message": "Target not running" }))
    }
}

async fn handle_reload_target(
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

async fn handle_restart_target(
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

async fn handle_reload_all(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    for (_id, sess) in sessions.iter() {
        let _ = sess.reload(vec![]).await;
    }
    Json(serde_json::json!({ "success": true, "message": format!("Reloaded {} active targets", sessions.len()) }))
}

async fn handle_restart_all(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    for (_id, sess) in sessions.iter() {
        let _ = sess.restart().await;
    }
    Json(serde_json::json!({ "success": true, "message": format!("Restarted {} active targets", sessions.len()) }))
}

async fn handle_events_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.event_bus.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(evt) => {
                    if let Ok(json_str) = serde_json::to_string(&evt) {
                        yield Ok(Event::default().data(json_str));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
