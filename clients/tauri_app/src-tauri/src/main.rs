use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use colored::*;
use devflow_cli_core::{run_cli_args, shell, CliAction};
use devflow_core::doctor::DoctorEngine;
use devflow_core::event::EventBus;
use devflow_core::project::{Project, ProjectTarget};
use devflow_core::registry::{GlobalRegistry, KnownProject};
use devflow_devices::DeviceManager;
use devflow_frameworks::session::SessionManager;
use devflow_mcp::{get_tool_definitions, McpHandler};
use devflow_protocol::{Device, DoctorReport, Platform};
use devflow_tui::TuiRunner;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    event_bus: EventBus,
    active_sessions: Arc<Mutex<HashMap<String, Arc<SessionManager>>>>,
    mcp_handler: Arc<McpHandler>,
    mcp_enabled: Arc<AtomicBool>,
    actual_port: Arc<AtomicU16>,
}

#[cfg(target_os = "macos")]
static MACOS_DIALOG_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/devflow-dialog-macos"));

#[cfg(target_os = "macos")]
extern "C" {
    fn devflow_set_macos_dock_icon(bytes: *const u8, len: usize);
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
            let mcp_handler = Arc::new(McpHandler::new().with_event_bus(event_bus.clone()));
            let mcp_enabled = Arc::new(AtomicBool::new(true));
            let actual_port_holder = Arc::new(AtomicU16::new(base_port));

            let state = AppState {
                event_bus,
                active_sessions: Arc::new(Mutex::new(HashMap::new())),
                mcp_handler,
                mcp_enabled,
                actual_port: actual_port_holder.clone(),
            };

            let cors = CorsLayer::permissive();

            let app = Router::new()
                .route("/api/workspace", get(handle_workspace))
                .route(
                    "/api/workspaces",
                    get(handle_list_workspaces)
                        .post(handle_add_workspace)
                        .delete(handle_remove_workspace),
                )
                .route("/api/devices", get(handle_devices))
                .route("/api/devices/boot", post(handle_boot_emulator))
                .route("/api/doctor", get(handle_doctor))
                .route("/api/mcp/status", get(handle_mcp_status))
                .route("/api/mcp/toggle", post(handle_mcp_toggle))
                .route("/api/mcp/logs", get(handle_mcp_logs).delete(handle_delete_mcp_logs))
                .route("/api/mcp/sessions", get(handle_mcp_sessions))
                .route("/api/mcp/agents", get(handle_mcp_agents))
                .route("/rpc", post(handle_mcp_rpc))
                .route("/api/shell/install", post(handle_shell_install))
                .route("/api/shell/uninstall", post(handle_shell_uninstall))
                .route("/api/target/start", post(handle_start_target))
                .route("/api/target/stop", post(handle_stop_target))
                .route("/api/target/reload", post(handle_reload_target))
                .route("/api/target/restart", post(handle_restart_target))
                .route("/api/workspace/reload-all", post(handle_reload_all))
                .route("/api/workspace/restart-all", post(handle_restart_all))
                .route("/api/dialog/pick-folder", post(handle_pick_folder))
                .route("/api/events", get(handle_events_sse))
                .route("/", get(index_handler))
                .fallback(fallback_handler)
                .layer(cors)
                .layer(axum::middleware::from_fn(|req: axum::extract::Request, next: axum::middleware::Next| async move {
                    let method = req.method().clone();
                    let uri = req.uri().clone();
                    let response = next.run(req).await;
                    let status = response.status();
                    if status.is_server_error() || status.is_client_error() {
                        warn!("HTTP {} {} -> {}", method, uri, status);
                    } else {
                        info!("HTTP {} {} -> {}", method, uri, status);
                    }
                    response
                }))
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
                        actual_port_holder.store(actual_port, Ordering::Relaxed);
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

    let url_str = format!(
        "http://localhost:{}?dir={}&port={}",
        bound_port,
        urlencoding_simple(&workspace_dir.display().to_string()),
        bound_port
    );
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
            if let Ok(menu) = tauri::menu::Menu::default(app.handle()) {
                let _ = app.set_menu(menu);
            }

            // Set macOS Dock icon dynamically from embedded icon PNG
            #[cfg(target_os = "macos")]
            {
                static ICON_PNG: &[u8] = include_bytes!("../icons/icon.png");
                unsafe {
                    devflow_set_macos_dock_icon(ICON_PNG.as_ptr(), ICON_PNG.len());
                }
            }

            // Apply default window icon if available
            if let Some(icon) = app.default_window_icon() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon.clone());
                }
            }

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

#[derive(rust_embed::Embed)]
#[folder = "../dist"]
struct FrontendAssets;

async fn index_handler() -> impl IntoResponse {
    serve_asset("index.html")
}

async fn fallback_handler(req: axum::extract::Request) -> axum::response::Response {
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // If an API or RPC endpoint was requested but not matched, return 404 JSON (never HTML!)
    if path.starts_with("/api/") || path == "/rpc" {
        warn!("API route not found: {} {}", method, path);
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            serde_json::json!({
                "error": format!("API route not found: {} {}", method, path)
            })
            .to_string(),
        )
            .into_response();
    }

    // Static assets only respond to GET and HEAD
    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        warn!("Method not allowed for static asset: {} {}", method, path);
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }

    let clean_path = path.trim_start_matches('/');
    serve_asset(clean_path)
}

fn serve_asset(path: &str) -> axum::response::Response {
    let asset_path = if path.is_empty() { "index.html" } else { path };

    match FrontendAssets::get(asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            if let Some(index_content) = FrontendAssets::get("index.html") {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    index_content.data.into_owned(),
                )
                    .into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct WorkspacePathRequest {
    path: String,
}

fn resolve_workspace_path(input: &str) -> Option<PathBuf> {
    let raw = input.trim().trim_matches('"').trim_matches('\'');
    let raw = raw.strip_prefix("file://").unwrap_or(raw);
    let raw = raw.trim_end_matches('/');
    if raw.is_empty() {
        return None;
    }

    // 1. Tilde expansion (~/Dev/...)
    let expanded = if raw.starts_with("~/") || raw == "~" {
        if let Some(home) = dirs::home_dir() {
            if raw == "~" {
                home
            } else {
                home.join(&raw[2..])
            }
        } else {
            PathBuf::from(raw)
        }
    } else {
        PathBuf::from(raw)
    };

    if expanded.exists() && expanded.is_dir() {
        return Some(expanded.canonicalize().unwrap_or(expanded));
    }

    // 2. Relative to current working dir
    if let Ok(cwd) = std::env::current_dir() {
        let joined = cwd.join(raw);
        if joined.exists() && joined.is_dir() {
            return Some(joined.canonicalize().unwrap_or(joined));
        }
    }

    // 3. Match against known projects by name or path (case-insensitive)
    let known = GlobalRegistry::list_projects();
    for p in known {
        let p_clean = p.path.trim_end_matches('/');
        if p.name.eq_ignore_ascii_case(raw) || p_clean.eq_ignore_ascii_case(raw) {
            let pb = PathBuf::from(&p.path);
            if pb.exists() && pb.is_dir() {
                return Some(pb.canonicalize().unwrap_or(pb));
            }
        }
    }

    None
}

static CACHED_DEVICES: std::sync::LazyLock<std::sync::RwLock<Vec<Device>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(Vec::new()));
static DEVICE_CACHE_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn get_cached_devices() -> Vec<Device> {
    if let Ok(lock) = CACHED_DEVICES.read() {
        lock.clone()
    } else {
        Vec::new()
    }
}

async fn get_or_refresh_devices() -> Vec<Device> {
    if DEVICE_CACHE_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        let cached = get_cached_devices();
        // Background async refresh without blocking caller
        tokio::spawn(async {
            let fresh = DeviceManager::discover_all().await;
            if let Ok(mut lock) = CACHED_DEVICES.write() {
                *lock = fresh;
            }
        });
        return cached;
    }

    // Fast initial discovery: return host desktop device immediately (< 1ms)
    let desktop = devflow_devices::DesktopDiscoverer::discover().await;
    if let Ok(mut lock) = CACHED_DEVICES.write() {
        *lock = desktop.clone();
    }
    DEVICE_CACHE_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);

    // Spawn full background scan for emulators, ADB, simulators
    tokio::spawn(async {
        let fresh = DeviceManager::discover_all().await;
        if let Ok(mut lock) = CACHED_DEVICES.write() {
            *lock = fresh;
        }
    });

    desktop
}

async fn handle_list_workspaces() -> Json<Vec<KnownProject>> {
    Json(GlobalRegistry::list_projects())
}

async fn handle_add_workspace(
    Json(req): Json<WorkspacePathRequest>,
) -> Result<Json<WorkspaceResponse>, (StatusCode, String)> {
    let canonical = match resolve_workspace_path(&req.path) {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Directory does not exist or cannot be resolved: {}", req.path),
            ));
        }
    };
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());
    let targets = Project::discover_workspace_targets(&canonical);
    let devices = get_or_refresh_devices().await;
    let active_sessions = GlobalRegistry::list_active_sessions();

    let primary_platform = targets
        .first()
        .map(|t| t.platform)
        .unwrap_or(Platform::Generic);
    let primary_framework = targets
        .first()
        .map(|t| t.framework.clone())
        .unwrap_or_else(|| "generic".to_string());

    GlobalRegistry::record_project(&canonical, &name, primary_platform, &primary_framework);

    Ok(Json(WorkspaceResponse {
        workspace_name: name,
        workspace_path: canonical.display().to_string(),
        targets,
        devices,
        active_sessions,
    }))
}

async fn handle_remove_workspace(
    Json(req): Json<WorkspacePathRequest>,
) -> Json<serde_json::Value> {
    let clean = req.path.trim().trim_matches('"').trim_matches('\'');
    let clean = clean.strip_prefix("file://").unwrap_or(clean);
    let p = PathBuf::from(clean);
    GlobalRegistry::remove_project(&p);
    Json(serde_json::json!({ "success": true }))
}

async fn handle_workspace(
    Query(query): Query<WorkspaceQuery>,
) -> Json<WorkspaceResponse> {
    let canonical = query
        .dir
        .as_deref()
        .and_then(resolve_workspace_path)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("."))
        });

    let workspace_name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());

    let targets = Project::discover_workspace_targets(&canonical);
    let devices = get_or_refresh_devices().await;
    let active_sessions = GlobalRegistry::list_active_sessions();

    Json(WorkspaceResponse {
        workspace_name,
        workspace_path: canonical.display().to_string(),
        targets,
        devices,
        active_sessions,
    })
}

async fn handle_devices() -> Json<Vec<Device>> {
    let devices = DeviceManager::discover_all().await;
    if let Ok(mut lock) = CACHED_DEVICES.write() {
        *lock = devices.clone();
    }
    DEVICE_CACHE_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
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
    for sess in sessions.values() {
        let _ = sess.reload(vec![]).await;
    }
    Json(serde_json::json!({ "success": true, "message": format!("Reloaded {} active targets", sessions.len()) }))
}

async fn handle_restart_all(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.active_sessions.lock().await;
    for sess in sessions.values() {
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

async fn handle_pick_folder() -> Json<serde_json::Value> {
    #[cfg(target_os = "macos")]
    {
        let res = tokio::task::spawn_blocking(|| -> Result<String, anyhow::Error> {
            // 1. Ensure /tmp/devflow-dialog-macos binary exists and has execute permissions
            let tmp_bin = PathBuf::from("/tmp/devflow-dialog-macos");
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

#[derive(Serialize)]
struct McpStatusResponse {
    enabled: bool,
    http_endpoint: String,
    port: u16,
    tools_count: usize,
    tools: Vec<serde_json::Value>,
    configs: McpClientConfigs,
}

#[derive(Serialize)]
struct McpClientConfigs {
    claude_desktop: serde_json::Value,
    cursor: serde_json::Value,
    antigravity: serde_json::Value,
    vscode: serde_json::Value,
}

#[derive(Deserialize)]
struct McpToggleRequest {
    enabled: bool,
}

async fn handle_mcp_status(State(state): State<AppState>) -> Json<McpStatusResponse> {
    let enabled = state.mcp_enabled.load(Ordering::Relaxed);
    let port = state.actual_port.load(Ordering::Relaxed);
    let endpoint = format!("http://localhost:{}/rpc", port);
    let tools = get_tool_definitions();
    let tools_count = tools.len();

    let configs = McpClientConfigs {
        claude_desktop: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "command": "devflow",
                    "args": ["mcp", "serve"]
                }
            }
        }),
        cursor: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "url": endpoint
                }
            }
        }),
        antigravity: serde_json::json!({
            "mcpServers": {
                "devflow": {
                    "command": "devflow",
                    "args": ["mcp", "serve"]
                }
            }
        }),
        vscode: serde_json::json!({
            "servers": {
                "devflow": {
                    "type": "http",
                    "url": endpoint
                }
            }
        }),
    };

    Json(McpStatusResponse {
        enabled,
        http_endpoint: endpoint,
        port,
        tools_count,
        tools,
        configs,
    })
}

async fn handle_mcp_toggle(
    State(state): State<AppState>,
    Json(req): Json<McpToggleRequest>,
) -> Json<serde_json::Value> {
    state.mcp_enabled.store(req.enabled, Ordering::Relaxed);
    info!("DevFlow MCP server toggle: enabled={}", req.enabled);
    Json(serde_json::json!({ "success": true, "enabled": req.enabled }))
}

#[derive(Deserialize)]
struct McpLogsQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    session_id: Option<String>,
    client: Option<String>,
    agent: Option<String>,
    tool_name: Option<String>,
    status: Option<String>,
    search: Option<String>,
}

async fn handle_mcp_logs(
    State(state): State<AppState>,
    Query(query): Query<McpLogsQuery>,
) -> Json<Vec<devflow_mcp::McpAccessLogEntry>> {
    let filter = devflow_mcp::McpLogFilter {
        limit: query.limit.or(Some(100)),
        offset: query.offset,
        session_id: query.session_id,
        client: query.client.or(query.agent),
        tool_name: query.tool_name,
        status: query.status,
        search: query.search,
    };
    let logs = state.mcp_handler.query_access_logs(&filter);
    Json(logs)
}

async fn handle_mcp_sessions(
    State(state): State<AppState>,
) -> Json<Vec<devflow_mcp::McpSessionDescriptor>> {
    let sessions = state.mcp_handler.get_access_log_descriptors();
    Json(sessions)
}

async fn handle_mcp_agents(
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let agents = state.mcp_handler.get_access_log_clients();
    Json(agents)
}

#[derive(Deserialize)]
struct DeleteMcpLogsQuery {
    id: Option<String>,
    session_id: Option<String>,
}

async fn handle_delete_mcp_logs(
    State(state): State<AppState>,
    Query(query): Query<DeleteMcpLogsQuery>,
) -> Json<serde_json::Value> {
    if let Some(ref id) = query.id {
        let deleted = state.mcp_handler.delete_access_log(id);
        Json(serde_json::json!({ "success": deleted, "deleted_id": id }))
    } else if let Some(ref session_id) = query.session_id {
        let count = state.mcp_handler.delete_access_logs_by_session(session_id);
        Json(serde_json::json!({ "success": true, "deleted_count": count, "session_id": session_id }))
    } else {
        state.mcp_handler.clear_access_logs();
        Json(serde_json::json!({ "success": true, "cleared_all": true }))
    }
}

async fn handle_mcp_rpc(
    State(state): State<AppState>,
    Json(req): Json<devflow_protocol::JsonRpcRequest>,
) -> Result<Json<devflow_protocol::JsonRpcResponse>, StatusCode> {
    if !state.mcp_enabled.load(Ordering::Relaxed) {
        let resp = devflow_protocol::JsonRpcResponse::error(
            req.id,
            -32000,
            "DevFlow MCP server is currently disabled in companion app".to_string(),
            None,
        );
        return Ok(Json(resp));
    }
    let resp = state.mcp_handler.handle_request(req).await;
    Ok(Json(resp))
}


