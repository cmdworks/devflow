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
use devflow_protocol::{Device, DoctorReport, Platform};
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

#[cfg(target_os = "macos")]
static MACOS_DIALOG_BIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/devflow-dialog-macos"));

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
                .route("/{*path}", get(static_handler))
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
            if let Ok(menu) = tauri::menu::Menu::default(app.handle()) {
                let _ = app.set_menu(menu);
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

async fn static_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    let clean_path = path.trim_start_matches('/');
    serve_asset(clean_path)
}

fn serve_asset(path: &str) -> impl IntoResponse {
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

async fn handle_pick_folder() -> Json<serde_json::Value> {
    #[cfg(target_os = "macos")]
    {
        let res = tokio::task::spawn_blocking(|| -> Result<String, anyhow::Error> {
            // 1. Ensure /tmp/devflow-dialog-macos binary exists and has execute permissions
            let tmp_bin = PathBuf::from("/tmp/devflow-dialog-macos");
            if !tmp_bin.exists() && !MACOS_DIALOG_BIN.is_empty() {
                if let Ok(_) = std::fs::write(&tmp_bin, MACOS_DIALOG_BIN) {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&tmp_bin, std::fs::Permissions::from_mode(0o755));
                    }
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
        return Json(serde_json::json!({ "success": false }));
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

