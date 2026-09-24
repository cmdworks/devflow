use colored::*;
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use std::path::Path;

pub async fn handle_hub_summary(current_dir: &Path) -> anyhow::Result<()> {
    println!(
        "\n{}",
        "═══ DevFlow Multi-Target Workspace Hub ═══".cyan().bold()
    );
    let folder_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    println!(
        "Workspace: {} ({})",
        folder_name.bold(),
        current_dir.display().to_string().dimmed()
    );

    let targets = Project::discover_workspace_targets(current_dir);
    println!("\n{}", "Runnable Targets:".bold());
    if targets.is_empty() {
        println!(
            "  {}",
            "No recognized framework targets found in this workspace.".yellow()
        );
    } else {
        for (i, target) in targets.iter().enumerate() {
            println!(
                "  [{}] {} [{:?}] ({}) -> {}",
                i + 1,
                target.name.bold(),
                target.platform,
                target.framework.green(),
                target.path.display().to_string().dimmed()
            );
        }
    }

    let sessions = devflow_core::registry::GlobalRegistry::list_active_sessions();
    println!("\n{}", "Active Sessions Across Terminals:".bold());
    if sessions.is_empty() {
        println!(
            "  {}",
            "No other active DevFlow sessions running on system.".dimmed()
        );
    } else {
        for s in &sessions {
            println!(
                "  ● PID {}: {} [{:?}] (socket: {})",
                s.pid,
                s.project_name.bold(),
                s.platform,
                s.socket_path.dimmed()
            );
        }
    }

    let devices = DeviceManager::discover_all().await;
    println!("\n{}", "Available Devices:".bold());
    if devices.is_empty() {
        println!("  {}", "No connected devices found.".dimmed());
    } else {
        for d in &devices {
            let state_str = match d.state {
                devflow_protocol::DeviceState::Connected
                | devflow_protocol::DeviceState::Booted => "ONLINE".green(),
                _ => "OFFLINE".dimmed(),
            };
            let emu_str = if d.is_emulator { " [emulator]" } else { "" };
            println!(
                "  ○ {} [{:?}]{} ({})",
                d.name.bold(),
                d.platform,
                emu_str,
                state_str
            );
        }
    }

    println!(
        "\n{}",
        "Tip: Run 'devflow' in an interactive terminal to launch the interactive TUI Hub.".dimmed()
    );
    println!(
        "     Run 'devflow dev' to start live session, or 'devflow --help' for CLI commands.\n"
    );
    Ok(())
}
