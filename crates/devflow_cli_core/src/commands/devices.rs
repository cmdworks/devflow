use colored::*;
use devflow_devices::DeviceManager;

pub async fn handle_devices(boot: Option<String>, json: bool) -> anyhow::Result<()> {
    if let Some(avd_name) = boot {
        println!("{} Booting emulator '{}'...", "🚀".cyan(), avd_name.bold());
        match DeviceManager::boot_emulator(&avd_name).await {
            Ok(msg) => println!("{} {}", "✓".green().bold(), msg),
            Err(e) => println!("{} Failed to boot: {}", "✗".red().bold(), e),
        }
        return Ok(());
    }

    let devices = DeviceManager::discover_all().await;

    if json {
        println!("{}", serde_json::to_string_pretty(&devices)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ Discovered Devices & Emulators ═══".cyan().bold()
    );
    if devices.is_empty() {
        println!("{}", "No devices or emulators found.".yellow());
        return Ok(());
    }

    for (i, d) in devices.iter().enumerate() {
        let default_badge = if d.is_default {
            " (default)".cyan()
        } else {
            "".normal()
        };
        let emu_badge = if d.is_emulator {
            " [emulator/avd]".magenta()
        } else {
            "".normal()
        };
        let state_badge = match d.state {
            devflow_protocol::DeviceState::Connected | devflow_protocol::DeviceState::Booted => {
                "Connected".green()
            }
            devflow_protocol::DeviceState::Shutdown => "Shutdown (bootable)".yellow(),
            _ => format!("{}", d.state).dimmed(),
        };

        println!(
            " {}. {} [ID: {}]{}{}",
            (i + 1).to_string().bold(),
            d.name.bold(),
            d.id.dimmed(),
            default_badge,
            emu_badge
        );
        println!("    Platform: {} | Status: {}", d.platform, state_badge);
    }
    println!();
    Ok(())
}
