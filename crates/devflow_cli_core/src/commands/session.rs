use colored::*;
use devflow_tui::TuiRunner;

pub async fn handle_reload() -> anyhow::Result<()> {
    if let Some(session) = devflow_core::registry::GlobalRegistry::find_active_session(None) {
        println!(
            "{} Sending reload signal to active session '{}' (PID {})...",
            "⚡".cyan(),
            session.project_name.bold(),
            session.pid
        );
        let resp = devflow_core::ipc::IpcClient::send_command(
            &session.socket_path,
            devflow_core::ipc::IpcRequest::Reload { files: vec![] },
        )
        .await?;
        if resp.success {
            println!("{} {}", "✓".green().bold(), resp.message);
        } else {
            println!("{} {}", "✗".red().bold(), resp.message);
        }
    } else {
        println!("{}", "No active DevFlow session found on system to reload. Run 'devflow' or 'devflow dev' first.".yellow());
    }
    Ok(())
}

pub async fn handle_restart() -> anyhow::Result<()> {
    if let Some(session) = devflow_core::registry::GlobalRegistry::find_active_session(None) {
        println!(
            "{} Sending restart signal to active session '{}' (PID {})...",
            "⚡".cyan(),
            session.project_name.bold(),
            session.pid
        );
        let resp = devflow_core::ipc::IpcClient::send_command(
            &session.socket_path,
            devflow_core::ipc::IpcRequest::Restart,
        )
        .await?;
        if resp.success {
            println!("{} {}", "✓".green().bold(), resp.message);
        } else {
            println!("{} {}", "✗".red().bold(), resp.message);
        }
    } else {
        println!("{}", "No active DevFlow session found on system to restart. Run 'devflow' or 'devflow dev' first.".yellow());
    }
    Ok(())
}

pub async fn handle_preview() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    TuiRunner::run_hub(current_dir)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}
