use colored::*;
use devflow_cli_core::{init_environment, run_cli_args, CliAction};
use devflow_tui::TuiRunner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_environment();
    let args: Vec<String> = std::env::args().collect();
    let action = run_cli_args(args).await?;

    match action {
        CliAction::Executed => Ok(()),
        CliAction::LaunchTui { dir } => TuiRunner::run_hub(dir)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e)),
        CliAction::LaunchGui {
            dir,
            port,
            open_browser,
        } => {
            let dir_str = dir.display().to_string();
            let url = format!(
                "http://localhost:{}?dir={}",
                port,
                urlencoding_simple(&dir_str)
            );

            if open_browser {
                println!(
                    "{} Opening DevFlow Web Companion in browser: {}",
                    "⚡".cyan().bold(),
                    url.underline()
                );
                let _ = open::that(&url);
                return Ok(());
            }

            // 1. Check for macOS DevFlow.app in /Applications or ~/Applications
            #[cfg(target_os = "macos")]
            {
                let app_candidates = [
                    std::path::PathBuf::from("/Applications/DevFlow.app"),
                    dirs::home_dir().map(|h| h.join("Applications/DevFlow.app")).unwrap_or_default(),
                ];
                for app_path in &app_candidates {
                    if app_path.exists() {
                        let _ = std::process::Command::new("open")
                            .arg("-a")
                            .arg(app_path)
                            .arg("--args")
                            .arg(&dir_str)
                            .spawn();
                        println!("{} Launched DevFlow Desktop for '{}'", "⚡".cyan().bold(), dir_str.green());
                        return Ok(());
                    }
                }
            }

            // 2. Check for devflow-gui binary in PATH or standard local dirs
            let local_gui_candidates = [
                dirs::home_dir().map(|h| h.join(".local/bin/devflow-gui")).unwrap_or_default(),
                std::path::PathBuf::from("/usr/local/bin/devflow-gui"),
            ];

            for bin_path in &local_gui_candidates {
                if bin_path.exists() {
                    let _ = std::process::Command::new(bin_path)
                        .arg(&dir_str)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                    println!("{} Launched DevFlow Desktop for '{}'", "⚡".cyan().bold(), dir_str.green());
                    return Ok(());
                }
            }

            // 3. Fallback: Open web companion in default browser
            println!(
                "{} Starting DevFlow Web Companion on port {}...",
                "⚡".cyan().bold(),
                port
            );
            let _ = open::that(&url);
            println!("GUI URL: {}", url.underline());
            println!("Tip: Install DevFlow Desktop Companion with 'curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install-gui.sh | bash'");
            Ok(())
        }
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(' ', "%20").replace('/', "%2F")
}
