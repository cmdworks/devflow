mod gui;
mod server;
mod state;

use devflow_cli_core::{run_cli_args, CliAction};
use devflow_tui::TuiRunner;
use std::path::PathBuf;

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
                if let Err(e) = gui::run_desktop_app(dir, port, open_browser) {
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

    if let Err(e) = gui::run_desktop_app(current_dir, port, false) {
        eprintln!("Desktop GUI Error: {}", e);
        std::process::exit(1);
    }
}
