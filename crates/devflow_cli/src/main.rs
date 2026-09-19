use colored::*;
use devflow_cli_core::{run_cli_args, CliAction};
use devflow_tui::TuiRunner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let action = run_cli_args(args).await?;

    match action {
        CliAction::Executed => Ok(()),
        CliAction::LaunchTui { dir } => {
            TuiRunner::run_hub(dir).await.map_err(|e| anyhow::anyhow!("{}", e))
        }
        CliAction::LaunchGui { dir, port, open_browser } => {
            // Check if devflow-desktop is available or running
            println!("{} Starting DevFlow GUI on port {} for workspace '{}'...", "⚡".cyan().bold(), port, dir.display());
            let url = format!("http://localhost:{}?dir={}", port, urlencoding_simple(&dir.display().to_string()));
            if open_browser {
                let _ = open::that(&url);
            }
            println!("GUI URL: {}", url.underline());
            println!("Tip: If you have DevFlow Desktop installed, you can also launch the native window.");
            Ok(())
        }
    }
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(' ', "%20").replace('/', "%2F")
}
