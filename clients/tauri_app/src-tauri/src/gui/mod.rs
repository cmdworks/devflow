pub mod macos;

use colored::*;
use std::path::PathBuf;

use crate::server;

pub fn run_desktop_app(
    workspace_dir: PathBuf,
    base_port: u16,
    open_browser: bool,
) -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .try_init();

    // 1. Start embedded Axum REST & SSE backend on a background thread
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    server::start_server_thread(base_port, ready_tx);

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

    // 2. Launch Tauri native window on the main thread and navigate to the running backend
    tauri::Builder::default()
        .setup(move |app| {
            use tauri::Manager;
            if let Ok(menu) = tauri::menu::Menu::default(app.handle()) {
                let _ = app.set_menu(menu);
            }

            // Apply macOS Dock icon
            macos::apply_macos_dock_icon();

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
